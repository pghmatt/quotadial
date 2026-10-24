//! A window's readings over time: what `quotadial history`, `GET /v1/history` and the app's history
//! view show (D-095). They come from the `readings` table, without unconfirmed readings, named by
//! each account's display name from its newest snapshot.

use std::collections::HashMap;
use std::io::{self, Write};

use chrono::{DateTime, Utc};
use quotadial_core::{Source, WindowKind};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::alerts::parse_time;
use crate::history::{History, HistoryError, timestamp};

/// One window's readings over time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Series {
    pub provider: String,
    pub account_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub org: Option<String>,
    /// The account's display name.
    pub account: String,
    /// The window's label, such as `Weekly`.
    pub window: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope_model: Option<String>,
    pub kind: WindowKind,
    /// Oldest first.
    pub readings: Vec<Point>,
}

/// One reading of a window.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub fetched_at: DateTime<Utc>,
    pub used_pct: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resets_at: Option<DateTime<Utc>>,
    pub source: Source,
}

/// Which series to give: those read since `since`, narrowed by provider, account and window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeriesQuery {
    pub since: DateTime<Utc>,
    /// Only this provider's accounts, such as `claude`.
    pub provider: Option<String>,
    /// Only this account: its id or its display name, in any case.
    pub account: Option<String>,
    /// Only windows with this label, in any case.
    pub window: Option<String>,
}

impl SeriesQuery {
    /// Every series read since `since`.
    #[must_use]
    pub fn since(since: DateTime<Utc>) -> Self {
        Self {
            since,
            provider: None,
            account: None,
            window: None,
        }
    }
}

impl Series {
    /// How much of the window was used at each reading since the one before: the rise in its share
    /// used, or after a reset, the share used since. A reading that is no higher than the one
    /// before used nothing.
    #[must_use]
    pub fn activity(&self) -> Vec<(DateTime<Utc>, f64)> {
        self.readings
            .windows(2)
            .map(|pair| {
                let (before, now) = (&pair[0], &pair[1]);
                let reset = now.used_pct < before.used_pct
                    || before.resets_at.is_some_and(|at| at <= now.fetched_at);
                let used = if reset {
                    now.used_pct
                } else {
                    now.used_pct - before.used_pct
                };
                (now.fetched_at, used.max(0.0))
            })
            .collect()
    }
}

impl History {
    /// The series `query` asks for, each provider's in order of account name, then window.
    ///
    /// # Errors
    ///
    /// When the database cannot be read.
    pub fn series(&self, query: &SeriesQuery) -> Result<Vec<Series>, HistoryError> {
        let names: HashMap<(String, String, String), String> = self
            .latest()?
            .into_iter()
            .map(|snapshot| {
                let account = snapshot.account;
                (
                    (
                        account.provider,
                        account.org.unwrap_or_default(),
                        account.account_id,
                    ),
                    account.display_name,
                )
            })
            .collect();
        let rows: Vec<Row> = {
            let connection = self.lock();
            let mut statement = connection.prepare_cached(
                "SELECT provider, org, account_id, label, scope_model, kind, fetched_at, used_pct,
                        resets_at, source
                 FROM readings WHERE fetched_at >= ?1 AND unconfirmed = 0
                 ORDER BY provider, org, account_id, label, scope_model, fetched_at",
            )?;
            let rows = statement.query_map(params![timestamp(query.since)], |row| {
                Ok(Row {
                    key: (row.get(0)?, row.get(1)?, row.get(2)?),
                    label: row.get(3)?,
                    scope_model: row.get(4)?,
                    kind: row.get(5)?,
                    fetched_at: row.get(6)?,
                    used_pct: row.get(7)?,
                    resets_at: row.get(8)?,
                    source: row.get(9)?,
                })
            })?;
            rows.collect::<Result<_, _>>()?
        };
        let mut found: Vec<Series> = Vec::new();
        for row in rows {
            let (Some(kind), Some(source), Some(fetched_at)) = (
                named(&row.kind),
                named(&row.source),
                parse_time(&row.fetched_at),
            ) else {
                continue;
            };
            let point = Point {
                fetched_at,
                used_pct: row.used_pct,
                resets_at: row.resets_at.as_deref().and_then(parse_time),
                source,
            };
            let (provider, org, account_id) = row.key;
            let same = found.last().is_some_and(|series| {
                series.provider == provider
                    && series.org.as_deref().unwrap_or_default() == org
                    && series.account_id == account_id
                    && series.window == row.label
                    && series.scope_model.as_deref().unwrap_or_default() == row.scope_model
            });
            if same {
                if let Some(series) = found.last_mut() {
                    series.readings.push(point);
                }
                continue;
            }
            let account = names
                .get(&(provider.clone(), org.clone(), account_id.clone()))
                .cloned()
                .unwrap_or_else(|| account_id.clone());
            found.push(Series {
                provider,
                account_id,
                org: (!org.is_empty()).then_some(org),
                account,
                window: row.label,
                scope_model: (!row.scope_model.is_empty()).then_some(row.scope_model),
                kind,
                readings: vec![point],
            });
        }
        found.retain(|series| query.covers(series));
        found.sort_by_cached_key(|series| {
            (
                series.provider.clone(),
                series.account.to_lowercase(),
                series.window.clone(),
            )
        });
        Ok(found)
    }
}

impl SeriesQuery {
    fn covers(&self, series: &Series) -> bool {
        let same = |wanted: &Option<String>, value: &str| {
            wanted
                .as_ref()
                .is_none_or(|wanted| wanted.eq_ignore_ascii_case(value))
        };
        same(&self.provider, &series.provider)
            && same(&self.window, &series.window)
            && (same(&self.account, &series.account_id) || same(&self.account, &series.account))
    }
}

/// Every reading of `series` as CSV, a row each, as `quotadial history --csv` prints it and the
/// app exports it.
///
/// # Errors
///
/// When `out` cannot be written.
pub fn write_csv(series: &[Series], out: &mut impl Write) -> io::Result<()> {
    writeln!(
        out,
        "fetched_at,provider,account,window,used_pct,resets_at,source"
    )?;
    for series in series {
        for point in &series.readings {
            let source = serde_json::to_value(point.source)
                .ok()
                .and_then(|value| value.as_str().map(str::to_owned))
                .unwrap_or_default();
            writeln!(
                out,
                "{},{},{},{},{},{},{}",
                csv_time(point.fetched_at),
                csv_field(&series.provider),
                csv_field(&series.account),
                csv_field(&series.window),
                point.used_pct,
                point.resets_at.map(csv_time).unwrap_or_default(),
                source
            )?;
        }
    }
    Ok(())
}

/// `field` as a CSV field: in quotes, with its quotes doubled, when it holds a comma, a quote or a
/// line break.
fn csv_field(field: &str) -> std::borrow::Cow<'_, str> {
    if field.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", field.replace('"', "\"\"")).into()
    } else {
        field.into()
    }
}

/// A time in CSV: RFC 3339 in UTC, to the second.
fn csv_time(at: DateTime<Utc>) -> String {
    at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

/// A row of `readings`, as stored.
struct Row {
    key: (String, String, String),
    label: String,
    scope_model: String,
    kind: String,
    fetched_at: String,
    used_pct: f64,
    resets_at: Option<String>,
    source: String,
}

/// An enum stored by its JSON name, such as `weekly`.
fn named<T: serde::de::DeserializeOwned>(name: &str) -> Option<T> {
    serde_json::from_value(Value::String(name.to_owned())).ok()
}

#[cfg(test)]
mod tests {
    use chrono::TimeDelta;
    use quotadial_core::{Account, Snapshot, Status, Window};

    use super::*;

    fn at(minutes: i64) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2099-01-01T00:00:00Z")
            .expect("valid timestamp")
            .with_timezone(&Utc)
            + TimeDelta::minutes(minutes)
    }

    fn snapshot(provider: &str, name: &str, windows: &[(&str, f64)], minute: i64) -> Snapshot {
        Snapshot {
            account: Account {
                provider: provider.into(),
                account_id: format!("{name}-id"),
                display_name: format!("{name}@example.com"),
                plan: None,
                org: None,
                sign_in: None,
            },
            windows: windows
                .iter()
                .map(|(label, used_pct)| Window {
                    kind: if *label == "Session" {
                        WindowKind::Session
                    } else {
                        WindowKind::Weekly
                    },
                    label: (*label).into(),
                    used_pct: *used_pct,
                    resets_at: Some(at(300)),
                    window_seconds: 18_000,
                    scope_model: None,
                    source: Source::Api,
                    fetched_at: at(minute),
                    unconfirmed: false,
                    pace: None,
                })
                .collect(),
            extra_usage: None,
            status: Status::Ok,
            error: None,
        }
    }

    fn history() -> (tempfile::TempDir, History) {
        let dir = tempfile::tempdir().expect("temp dir");
        let history = History::open(&dir.path().join("history.db")).expect("opens");
        for (minute, session, weekly) in [(0, 10.0, 40.0), (5, 12.0, 40.0), (10, 15.0, 41.0)] {
            history
                .record(
                    &snapshot(
                        "claude",
                        "work",
                        &[("Session", session), ("Weekly", weekly)],
                        minute,
                    ),
                    at(minute),
                )
                .expect("records");
        }
        history
            .record(&snapshot("codex", "home", &[("Weekly", 50.0)], 3), at(3))
            .expect("records");
        (dir, history)
    }

    #[test]
    fn each_windows_readings_come_together_named_by_their_account() {
        let (_dir, history) = history();

        let series = history.series(&SeriesQuery::since(at(-60))).expect("reads");

        let found: Vec<(String, String, String, usize)> = series
            .iter()
            .map(|series| {
                (
                    series.provider.clone(),
                    series.account.clone(),
                    series.window.clone(),
                    series.readings.len(),
                )
            })
            .collect();
        assert_eq!(
            found,
            [
                (
                    "claude".into(),
                    "work@example.com".into(),
                    "Session".into(),
                    3
                ),
                (
                    "claude".into(),
                    "work@example.com".into(),
                    "Weekly".into(),
                    3
                ),
                (
                    "codex".into(),
                    "home@example.com".into(),
                    "Weekly".into(),
                    1
                ),
            ]
        );
        assert_eq!(series[0].kind, WindowKind::Session);
        assert_eq!(
            series[0]
                .readings
                .iter()
                .map(|point| point.used_pct)
                .collect::<Vec<_>>(),
            [10.0, 12.0, 15.0],
            "oldest first"
        );
    }

    #[test]
    fn a_query_narrows_by_time_provider_account_and_window() {
        let (_dir, history) = history();
        let count = |query: SeriesQuery| history.series(&query).expect("reads").len();

        assert_eq!(
            count(SeriesQuery::since(at(4))),
            2,
            "Codex's one reading, at minute 3, is older"
        );
        assert_eq!(
            history.series(&SeriesQuery::since(at(4))).expect("reads")[0]
                .readings
                .len(),
            2
        );
        assert_eq!(
            count(SeriesQuery {
                provider: Some("CODEX".into()),
                ..SeriesQuery::since(at(-60))
            }),
            1
        );
        assert_eq!(
            count(SeriesQuery {
                account: Some("Work@Example.com".into()),
                window: Some("weekly".into()),
                ..SeriesQuery::since(at(-60))
            }),
            1
        );
        assert_eq!(
            count(SeriesQuery {
                account: Some("home-id".into()),
                ..SeriesQuery::since(at(-60))
            }),
            1,
            "by id too"
        );
    }

    #[test]
    fn an_unconfirmed_reading_is_left_out() {
        let (_dir, history) = history();
        let mut doubtful = snapshot("claude", "work", &[("Session", 0.0)], 15);
        doubtful.windows[0].unconfirmed = true;
        history.record(&doubtful, at(15)).expect("records");

        let session = &history.series(&SeriesQuery::since(at(-60))).expect("reads")[0];

        assert_eq!(session.readings.len(), 3);
    }

    #[test]
    fn activity_is_the_rise_at_each_reading_counting_from_zero_after_a_reset() {
        let point = |minute, used_pct, resets_at| Point {
            fetched_at: at(minute),
            used_pct,
            resets_at: Some(at(resets_at)),
            source: Source::Api,
        };
        let series = Series {
            provider: "claude".into(),
            account_id: "work-id".into(),
            org: None,
            account: "work@example.com".into(),
            window: "Session".into(),
            scope_model: None,
            kind: WindowKind::Session,
            readings: vec![
                point(0, 10.0, 300),
                point(60, 30.0, 300),
                point(120, 30.0, 300),
                point(320, 4.0, 620),
                point(330, 9.0, 620),
            ],
        };

        let used: Vec<f64> = series
            .activity()
            .into_iter()
            .map(|(_, used)| used)
            .collect();

        assert_eq!(used, [20.0, 0.0, 4.0, 5.0]);
    }

    #[test]
    fn readings_as_csv_quote_what_needs_quoting() {
        let series = Series {
            provider: "claude".into(),
            account_id: "work-id".into(),
            org: None,
            account: "Work, \"main\"".into(),
            window: "Session".into(),
            scope_model: None,
            kind: WindowKind::Session,
            readings: vec![Point {
                fetched_at: at(0),
                used_pct: 12.5,
                resets_at: None,
                source: Source::Api,
            }],
        };
        let mut out = Vec::new();

        write_csv(&[series], &mut out).expect("writes");

        assert_eq!(
            String::from_utf8(out).expect("UTF-8"),
            "fetched_at,provider,account,window,used_pct,resets_at,source\n\
             2099-01-01T00:00:00Z,claude,\"Work, \"\"main\"\"\",Session,12.5,,api\n"
        );
    }

    #[test]
    fn history_is_kept_as_many_days_as_set() {
        let dir = tempfile::tempdir().expect("temp dir");
        let history = History::open(&dir.path().join("history.db"))
            .expect("opens")
            .keeping(90);
        history
            .record(&snapshot("codex", "home", &[("Weekly", 50.0)], 0), at(0))
            .expect("records");

        history.prune(at(60 * 24 * 60)).expect("prunes");
        assert_eq!(
            history
                .series(&SeriesQuery::since(at(-1)))
                .expect("reads")
                .len(),
            1,
            "60 days on, still kept"
        );
        history.prune(at(91 * 24 * 60)).expect("prunes");
        assert!(
            history
                .series(&SeriesQuery::since(at(-1)))
                .expect("reads")
                .is_empty()
        );
        assert_eq!(
            History::open(&dir.path().join("other.db"))
                .expect("opens")
                .keeping(1000)
                .keep_days(),
            365,
            "a year at most"
        );
    }
}
