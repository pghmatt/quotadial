//! The history view (D-095): each shown account's windows charted over the last 7 or 30 days, a
//! heatmap of when it was used by hour of the day, and an export of every reading as CSV.

use std::path::{Path, PathBuf};

use chrono::{DateTime, NaiveDate, TimeDelta, TimeZone, Timelike, Utc};
use quotadial_core::{WindowKind, text};
use quotadial_daemon::{History, Series, SeriesQuery, write_csv};
use serde::Serialize;

use crate::settings::{Settings, key};

/// The most points a chart gets: more readings than this are thinned evenly.
const CHART_POINTS: usize = 240;

/// What the history view shows.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct HistoryView {
    /// How many days back it goes.
    pub days: u32,
    /// What the charts span, such as "The last 7 days." or, for a history that began since,
    /// "Since Fri 23 at 09:16, when the history begins."
    pub span: String,
    pub accounts: Vec<AccountHistory>,
}

/// One account's history.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AccountHistory {
    /// As the settings name the account.
    pub key: String,
    /// Such as "Claude".
    pub provider: String,
    /// The name the user gave the account, or its email address.
    pub name: String,
    pub windows: Vec<WindowChart>,
    pub heatmap: Heatmap,
}

/// One window over the period.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WindowChart {
    pub label: String,
    /// Each reading's place along the period, from 0 to 1, and its share used.
    pub points: Vec<[f64; 2]>,
    pub latest: f64,
    pub highest: f64,
    /// Such as "Session: latest 8%, highest 11% in the last 7 days", for screen readers.
    pub summary: String,
}

/// When an account was used: per day of the period and hour of the day, the share of its
/// shortest window used then.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Heatmap {
    /// The window the use is counted in, such as "Session".
    pub window: String,
    /// A label per day, oldest first, such as "Mon 20".
    pub days: Vec<String>,
    /// Per day, the share used in each hour, local time.
    pub cells: Vec<[f64; 24]>,
    /// The largest cell, which the others are shaded against.
    pub most: f64,
}

/// The history view of the last `days` before `now`, with the days and hours of `zone`, under
/// `settings`: hidden providers left out and accounts called by the names given them.
///
/// # Errors
///
/// When the history cannot be read.
pub fn view<Tz: TimeZone>(
    history: &History,
    settings: &Settings,
    days: u32,
    now: DateTime<Utc>,
    zone: &Tz,
) -> Result<HistoryView, String> {
    let since = now - TimeDelta::days(i64::from(days));
    let mut series = history
        .series(&SeriesQuery::since(since))
        .map_err(|error| format!("QuotaDial could not read its history: {error}"))?;
    series.retain(|one| settings.shows(&one.provider));
    // A history younger than the period is charted from its first reading, so its lines span the
    // charts rather than hug their right edge.
    let start = series
        .iter()
        .filter_map(|one| one.readings.first())
        .map(|point| point.fetched_at)
        .min()
        .map_or(since, |first| first.max(since));
    let span = if start - since > TimeDelta::hours(1) {
        format!(
            "Since {}, when the history begins.",
            start
                .with_timezone(zone)
                .naive_local()
                .format("%a %-d at %H:%M")
        )
    } else {
        format!("The last {days} days.")
    };
    let mut accounts: Vec<AccountHistory> = Vec::new();
    let mut group: Vec<&Series> = Vec::new();
    for one in &series {
        if group.last().is_some_and(|last| !same_account(last, one)) {
            accounts.push(account(&group, settings, days, start, now, zone));
            group.clear();
        }
        group.push(one);
    }
    if !group.is_empty() {
        accounts.push(account(&group, settings, days, start, now, zone));
    }
    Ok(HistoryView {
        days,
        span,
        accounts,
    })
}

/// Writes every reading of the last `days` before `now` as CSV to `folder`, leaving out hidden
/// providers, and returns where.
///
/// # Errors
///
/// When the history cannot be read or the file written.
pub fn export(
    history: &History,
    settings: &Settings,
    days: u32,
    now: DateTime<Utc>,
    folder: &Path,
) -> Result<PathBuf, String> {
    let mut series = history
        .series(&SeriesQuery::since(now - TimeDelta::days(i64::from(days))))
        .map_err(|error| format!("QuotaDial could not read its history: {error}"))?;
    series.retain(|one| settings.shows(&one.provider));
    let path = folder.join(format!("QuotaDial history {}.csv", now.format("%Y-%m-%d")));
    let mut bytes = Vec::new();
    write_csv(&series, &mut bytes).map_err(|error| error.to_string())?;
    std::fs::write(&path, bytes)
        .map_err(|error| format!("QuotaDial could not save {}: {error}", path.display()))?;
    Ok(path)
}

fn same_account(a: &Series, b: &Series) -> bool {
    a.provider == b.provider && a.account_id == b.account_id && a.org == b.org
}

fn account<Tz: TimeZone>(
    group: &[&Series],
    settings: &Settings,
    days: u32,
    since: DateTime<Utc>,
    now: DateTime<Utc>,
    zone: &Tz,
) -> AccountHistory {
    let first = group[0];
    let key = key(&first.provider, &first.account_id, first.org.as_deref());
    let name = settings
        .aliases
        .get(&key)
        .cloned()
        .unwrap_or_else(|| first.account.clone());
    let period = format!("in the last {days} days");
    AccountHistory {
        provider: text::provider_name(&first.provider).to_owned(),
        name,
        windows: group
            .iter()
            .map(|series| chart(series, since, now, &period))
            .collect(),
        heatmap: heatmap(group, days, now, zone),
        key,
    }
}

fn chart(series: &Series, since: DateTime<Utc>, now: DateTime<Utc>, period: &str) -> WindowChart {
    let span = (now - since).num_seconds().max(1) as f64;
    let readings = &series.readings;
    let step = readings.len().div_ceil(CHART_POINTS).max(1);
    let mut points: Vec<[f64; 2]> = readings
        .iter()
        .step_by(step)
        .map(|point| {
            let along = (point.fetched_at - since).num_seconds() as f64 / span;
            [along.clamp(0.0, 1.0), point.used_pct.clamp(0.0, 100.0)]
        })
        .collect();
    // The latest reading always ends the line.
    if let Some(last) = readings.last()
        && !(readings.len() - 1).is_multiple_of(step)
    {
        let along = (last.fetched_at - since).num_seconds() as f64 / span;
        points.push([along.clamp(0.0, 1.0), last.used_pct.clamp(0.0, 100.0)]);
    }
    let latest = readings.last().map_or(0.0, |point| point.used_pct);
    let highest = readings
        .iter()
        .map(|point| point.used_pct)
        .fold(0.0, f64::max);
    WindowChart {
        summary: format!(
            "{}: latest {:.0}%, highest {:.0}% {period}",
            series.window,
            latest.floor(),
            highest.floor()
        ),
        label: series.window.clone(),
        points,
        latest,
        highest,
    }
}

/// The heatmap of `group`'s shortest window: a session if there is one.
fn heatmap<Tz: TimeZone>(group: &[&Series], days: u32, now: DateTime<Utc>, zone: &Tz) -> Heatmap {
    let shortest = group
        .iter()
        .min_by_key(|series| match series.kind {
            WindowKind::Session => 0,
            WindowKind::Daily => 1,
            WindowKind::Weekly => 2,
            WindowKind::ModelWeekly => 3,
            WindowKind::Credits | WindowKind::Other => 4,
        })
        .copied();
    let today = now.with_timezone(zone).date_naive();
    let first = today - TimeDelta::days(i64::from(days) - 1);
    let mut cells = vec![[0.0; 24]; days as usize];
    for (at, used) in shortest.map(Series::activity).unwrap_or_default() {
        let local = at.with_timezone(zone);
        let day = (local.date_naive() - first).num_days();
        if let Ok(day) = usize::try_from(day)
            && let Some(row) = cells.get_mut(day)
        {
            row[local.hour() as usize] += used;
        }
    }
    let most = cells.iter().flatten().copied().fold(0.0, f64::max);
    Heatmap {
        window: shortest
            .map(|series| series.window.clone())
            .unwrap_or_default(),
        days: (0..days)
            .map(|offset| day_label(first + TimeDelta::days(i64::from(offset))))
            .collect(),
        cells,
        most,
    }
}

/// Such as "Mon 20".
fn day_label(day: NaiveDate) -> String {
    day.format("%a %-d").to_string()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use quotadial_core::{Account, Snapshot, Source, Status, Window};

    use super::*;

    fn at(text: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(text)
            .expect("valid timestamp")
            .with_timezone(&Utc)
    }

    fn snapshot(provider: &str, windows: &[(WindowKind, &str, f64)], fetched_at: &str) -> Snapshot {
        Snapshot {
            account: Account {
                provider: provider.into(),
                account_id: "user-1".into(),
                display_name: format!("{provider}@example.com"),
                plan: None,
                org: None,
                sign_in: None,
            },
            windows: windows
                .iter()
                .map(|(kind, label, used_pct)| Window {
                    kind: *kind,
                    label: (*label).into(),
                    used_pct: *used_pct,
                    resets_at: Some(at("2099-01-07T00:00:00Z")),
                    window_seconds: 18_000,
                    scope_model: None,
                    source: Source::Api,
                    fetched_at: at(fetched_at),
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
        for (fetched_at, session, weekly) in [
            ("2099-01-05T09:10:00Z", 10.0, 40.0),
            ("2099-01-05T09:40:00Z", 25.0, 42.0),
            ("2099-01-06T14:05:00Z", 30.0, 50.0),
        ] {
            history
                .record(
                    &snapshot(
                        "claude",
                        &[
                            (WindowKind::Session, "Session", session),
                            (WindowKind::Weekly, "Weekly", weekly),
                        ],
                        fetched_at,
                    ),
                    at(fetched_at),
                )
                .expect("records");
        }
        history
            .record(
                &snapshot(
                    "codex",
                    &[(WindowKind::Weekly, "Weekly", 60.0)],
                    "2099-01-06T08:00:00Z",
                ),
                at("2099-01-06T08:00:00Z"),
            )
            .expect("records");
        (dir, history)
    }

    #[test]
    fn each_shown_account_has_its_windows_charted_by_the_name_given_it() {
        let (_dir, history) = history();
        let settings = Settings {
            hidden_providers: vec!["codex".into()],
            aliases: BTreeMap::from([("claude user-1 -".into(), "Work".into())]),
        };

        let view = view(&history, &settings, 7, at("2099-01-07T00:00:00Z"), &Utc).expect("reads");

        assert_eq!(view.accounts.len(), 1, "Codex is hidden");
        let work = &view.accounts[0];
        assert_eq!(
            (work.provider.as_str(), work.name.as_str()),
            ("Claude", "Work")
        );
        let session = &work.windows[0];
        assert_eq!(session.label, "Session");
        assert_eq!((session.latest, session.highest), (30.0, 30.0));
        assert_eq!(session.points.len(), 3);
        assert_eq!(session.points[2][1], 30.0);
        assert_eq!(session.points[0][0], 0.0, "from the first reading");
        assert!(session.points[2][0] < 1.0);
        assert_eq!(view.span, "Since Mon 5 at 09:10, when the history begins.");
        assert_eq!(
            session.summary,
            "Session: latest 30%, highest 30% in the last 7 days"
        );
    }

    #[test]
    fn the_heatmap_counts_the_sessions_use_by_day_and_hour() {
        let (_dir, history) = history();

        let view = view(
            &history,
            &Settings::default(),
            7,
            at("2099-01-07T00:00:00Z"),
            &Utc,
        )
        .expect("reads");

        let heatmap = &view.accounts[0].heatmap;
        assert_eq!(heatmap.window, "Session");
        assert_eq!(heatmap.days.len(), 7);
        assert_eq!(heatmap.days[6], "Wed 7", "today last");
        // 10% to 25% at 09:40 on the 5th, then 25% to 30% at 14:05 on the 6th.
        assert_eq!(heatmap.cells[4][9], 15.0);
        assert_eq!(heatmap.cells[5][14], 5.0);
        assert_eq!(heatmap.most, 15.0);
        let codex = &view.accounts[1].heatmap;
        assert_eq!(
            codex.window, "Weekly",
            "a weekly window when there is no session"
        );
    }

    #[test]
    fn a_long_history_is_thinned_but_keeps_its_latest_reading() {
        let series = Series {
            provider: "claude".into(),
            account_id: "user-1".into(),
            org: None,
            account: "user@example.com".into(),
            window: "Session".into(),
            scope_model: None,
            kind: WindowKind::Session,
            readings: (0..1000)
                .map(|minute| quotadial_daemon::Point {
                    fetched_at: at("2099-01-01T00:00:00Z") + TimeDelta::minutes(minute),
                    used_pct: minute as f64 / 10.0,
                    resets_at: None,
                    source: Source::Api,
                })
                .collect(),
        };

        let chart = chart(
            &series,
            at("2099-01-01T00:00:00Z"),
            at("2099-01-02T00:00:00Z"),
            "in the last 1 days",
        );

        assert!(
            chart.points.len() <= CHART_POINTS + 1,
            "{}",
            chart.points.len()
        );
        assert_eq!(chart.points.last().map(|point| point[1]), Some(99.9));
    }

    #[test]
    fn the_export_is_every_shown_reading_as_csv() {
        let (_dir, history) = history();
        let folder = tempfile::tempdir().expect("temp dir");
        let settings = Settings {
            hidden_providers: vec!["codex".into()],
            ..Settings::default()
        };

        let path = export(
            &history,
            &settings,
            7,
            at("2099-01-07T00:00:00Z"),
            folder.path(),
        )
        .expect("saves");

        assert_eq!(
            path.file_name().and_then(|name| name.to_str()),
            Some("QuotaDial history 2099-01-07.csv")
        );
        let csv = std::fs::read_to_string(&path).expect("reads");
        assert_eq!(
            csv.lines().count(),
            1 + 6,
            "a header and two windows of three readings"
        );
        assert!(!csv.contains("codex"));
    }
}
