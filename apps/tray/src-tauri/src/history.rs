//! The history view (D-095): each shown account's windows charted over the last 7 or 30 days, a
//! heatmap of when it was used by hour of the day, and an export of every reading as CSV. It also
//! carries what the window needs to draw a picture of an account's history to share (D-115): each
//! account's label, the days covered, and on QuotaDial Free the line said in the buttons' place.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Datelike, NaiveDate, TimeDelta, TimeZone, Timelike, Utc};
use quotadial_core::{ProFeature, Tier, WindowKind, text};
use quotadial_daemon::{History, Series, SeriesQuery, write_csv};
use serde::Serialize;

use crate::settings::{Settings, account_key, key};
use crate::share;

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
    /// The days the view covers, as people write them, such as "19 to 25 October 2026": from the
    /// period's first day, or the history's when it began since, to today.
    pub dates: String,
    pub accounts: Vec<AccountHistory>,
    /// On QuotaDial Free, the line saying a picture of an account's history to share is part of
    /// QuotaDial Pro ([`ProFeature::ShareImage`]), which the view shows once where the Share image
    /// buttons would be; `None` on Pro, which shows them (D-100, D-115).
    pub share_left_out: Option<String>,
}

/// One account's history.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AccountHistory {
    /// As the settings name the account.
    pub key: String,
    /// Such as "Claude".
    pub provider: String,
    /// The name the user gave the account, or its email address; empty when that says nothing
    /// beyond `provider`, so the provider shows once.
    pub name: String,
    /// What the picture of this account's history calls it ([`share::label`]): its provider and
    /// the name given it, or its plan, and never its address.
    pub label: String,
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
    /// The same facts in words, a line per day, for a screen reader (#52): such as "Mon 20: used
    /// 9:00 to 12:00 and 14:00 to 15:00, most at 14:00 (12%)", or "Tue 21: not used".
    pub said: Vec<String>,
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
    tier: Tier,
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
    let today = now.with_timezone(zone).date_naive();
    let first_day =
        (today - TimeDelta::days(i64::from(days) - 1)).max(start.with_timezone(zone).date_naive());
    let newest = newest(history)?;
    let mut accounts: Vec<AccountHistory> = Vec::new();
    let mut group: Vec<&Series> = Vec::new();
    for one in &series {
        if group.last().is_some_and(|last| !same_account(last, one)) {
            accounts.push(account(&group, settings, &newest, days, start, now, zone));
            group.clear();
        }
        group.push(one);
    }
    if !group.is_empty() {
        accounts.push(account(&group, settings, &newest, days, start, now, zone));
    }
    Ok(HistoryView {
        days,
        span,
        dates: dates(first_day, today),
        accounts,
        share_left_out: (!tier.allows(ProFeature::ShareImage))
            .then(|| ProFeature::ShareImage.line()),
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

/// What the picture of the account `key` calls it ([`share::label`]): the label [`view`] gives it,
/// from its newest snapshot in `history` and the name `settings` give it.
///
/// # Errors
///
/// When the history cannot be read, or does not know the account.
pub fn label(history: &History, settings: &Settings, key: &str) -> Result<String, String> {
    let snapshot = history
        .latest()
        .map_err(|error| format!("QuotaDial could not read its history: {error}"))?
        .into_iter()
        .find(|snapshot| account_key(&snapshot.account) == key)
        .ok_or_else(share::unknown_account)?;
    let account = snapshot.account;
    Ok(share::label(
        account.provider_label(),
        settings.aliases.get(key).map(String::as_str),
        account.plan.as_deref(),
    ))
}

/// What the newest snapshots say that a series does not carry: each account's plan, by
/// [`account_key`], and what each provider calls itself, by its id (P-020).
#[derive(Default)]
struct Newest {
    plans: HashMap<String, String>,
    names: HashMap<String, String>,
}

impl Newest {
    /// What the provider `id` calls itself, or `id` itself while its newest snapshots all come
    /// from an older QuotaDial, which named none, until the next poll.
    fn name<'a>(&'a self, id: &'a str) -> &'a str {
        self.names.get(id).map_or(id, String::as_str)
    }
}

fn newest(history: &History) -> Result<Newest, String> {
    let mut newest = Newest::default();
    for snapshot in history
        .latest()
        .map_err(|error| format!("QuotaDial could not read its history: {error}"))?
    {
        let account = snapshot.account;
        // An older QuotaDial named no provider.
        if !account.provider_name.is_empty() {
            newest
                .names
                .insert(account.provider.clone(), account.provider_name.clone());
        }
        if let Some(plan) = account.plan.clone() {
            newest.plans.insert(account_key(&account), plan);
        }
    }
    Ok(newest)
}

/// The days from `first` to `last`, as people write them: "19 to 25 October 2026", "26 September
/// to 25 October 2026", "29 December 2026 to 4 January 2027", or "25 October 2026" for one day.
fn dates(first: NaiveDate, last: NaiveDate) -> String {
    let whole = |day: NaiveDate| day.format("%-d %B %Y").to_string();
    if first >= last {
        whole(last)
    } else if first.year() != last.year() {
        format!("{} to {}", whole(first), whole(last))
    } else if first.month() != last.month() {
        format!("{} to {}", first.format("%-d %B"), whole(last))
    } else {
        format!("{} to {}", first.format("%-d"), whole(last))
    }
}

fn same_account(a: &Series, b: &Series) -> bool {
    a.provider == b.provider && a.account_id == b.account_id && a.org == b.org
}

fn account<Tz: TimeZone>(
    group: &[&Series],
    settings: &Settings,
    newest: &Newest,
    days: u32,
    since: DateTime<Utc>,
    now: DateTime<Utc>,
    zone: &Tz,
) -> AccountHistory {
    let first = group[0];
    let key = key(&first.provider, &first.account_id, first.org.as_deref());
    let alias = settings.aliases.get(&key);
    let provider = newest.name(&first.provider);
    // The name given, or the account's own when it says more than the provider's label.
    let name = alias.cloned().unwrap_or_else(|| {
        text::name_beside(provider, &first.account)
            .unwrap_or_default()
            .to_owned()
    });
    let period = format!("in the last {days} days");
    AccountHistory {
        provider: provider.to_owned(),
        name,
        label: share::label(
            provider,
            alias.map(String::as_str),
            newest.plans.get(&key).map(String::as_str),
        ),
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
    let days: Vec<String> = (0..days)
        .map(|offset| day_label(first + TimeDelta::days(i64::from(offset))))
        .collect();
    Heatmap {
        window: shortest
            .map(|series| series.window.clone())
            .unwrap_or_default(),
        said: days
            .iter()
            .zip(&cells)
            .map(|(day, hours)| said(day, hours))
            .collect(),
        days,
        cells,
        most,
    }
}

/// One day of the heatmap in words: the hours it was used, run together, and its busiest hour.
fn said(day: &str, hours: &[f64; 24]) -> String {
    let mut runs: Vec<(usize, usize)> = Vec::new();
    for (hour, used) in hours.iter().enumerate() {
        if *used <= 0.0 {
            continue;
        }
        match runs.last_mut() {
            Some((_, end)) if *end == hour => *end = hour + 1,
            _ => runs.push((hour, hour + 1)),
        }
    }
    // `max_by` keeps the last of equals, so searching backwards names the earliest busiest hour.
    let Some((busiest, most)) = hours
        .iter()
        .enumerate()
        .rev()
        .filter(|(_, used)| **used > 0.0)
        .max_by(|(_, a), (_, b)| a.total_cmp(b))
    else {
        return format!("{day}: not used");
    };
    let mut spans: Vec<String> = runs
        .iter()
        .map(|(start, end)| format!("{start}:00 to {end}:00"))
        .collect();
    let last = spans.pop().unwrap_or_default();
    let spans = if spans.is_empty() {
        last
    } else {
        format!("{} and {last}", spans.join(", "))
    };
    format!("{day}: used {spans}, most at {busiest}:00 ({most:.0}%)")
}

/// Such as "Mon 20".
fn day_label(day: NaiveDate) -> String {
    day.format("%a %-d").to_string()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    #[test]
    fn the_heatmap_is_said_in_words_a_day_a_line() {
        let mut hours = [0.0; 24];
        for hour in [9, 10, 11] {
            hours[hour] = 2.0;
        }
        hours[14] = 12.4;
        hours[16] = 1.0;
        hours[17] = 1.0;

        assert_eq!(
            super::said("Mon 20", &hours),
            "Mon 20: used 9:00 to 12:00, 14:00 to 15:00 and 16:00 to 18:00, most at 14:00 (12%)"
        );
        assert_eq!(super::said("Tue 21", &[0.0; 24]), "Tue 21: not used");
        let mut late = [0.0; 24];
        late[23] = 3.0;
        assert_eq!(
            super::said("Wed 22", &late),
            "Wed 22: used 23:00 to 24:00, most at 23:00 (3%)"
        );
        assert_eq!(
            super::said("Thu 23", &[1.0; 24]),
            "Thu 23: used 0:00 to 24:00, most at 0:00 (1%)"
        );
    }

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
                provider_name: if provider == "claude" {
                    "Claude"
                } else {
                    "Codex"
                }
                .into(),
                account_id: "user-1".into(),
                display_name: format!("{provider}@example.com"),
                plan: (provider == "claude").then(|| "max".to_owned()),
                org: None,
                sign_in: None,
                sign_in_use: "QuotaDial reads this test's sign-in.".into(),
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
            ..Settings::default()
        };

        let view = view(
            &history,
            &settings,
            7,
            at("2099-01-07T00:00:00Z"),
            &Utc,
            Tier::Pro,
        )
        .expect("reads");

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
            Tier::Pro,
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
    fn each_account_is_labelled_for_its_picture_by_name_or_plan_never_address() {
        let (_dir, history) = history();
        let named = Settings {
            aliases: BTreeMap::from([("claude user-1 -".into(), "Work".into())]),
            ..Settings::default()
        };
        let now = at("2099-01-07T00:00:00Z");

        let labels = |settings: &Settings| {
            view(&history, settings, 7, now, &Utc, Tier::Pro)
                .expect("reads")
                .accounts
                .into_iter()
                .map(|account| account.label)
                .collect::<Vec<_>>()
        };

        assert_eq!(labels(&named), ["Claude Work", "Codex"]);
        assert_eq!(
            labels(&Settings::default()),
            ["Claude Max", "Codex"],
            "the plan from the account's newest snapshot, or the provider alone"
        );
        assert_eq!(
            label(&history, &named, "claude user-1 -").as_deref(),
            Ok("Claude Work"),
            "the command names the file by the label the view gave"
        );
        assert_eq!(
            label(&history, &Settings::default(), "codex user-1 -").as_deref(),
            Ok("Codex")
        );
        assert_eq!(
            label(&history, &named, "claude someone-else -"),
            Err(crate::share::unknown_account())
        );
        for account in view(&history, &named, 7, now, &Utc, Tier::Pro)
            .expect("reads")
            .accounts
        {
            assert!(!account.label.contains('@'), "{}", account.label);
        }
    }

    #[test]
    fn a_history_from_an_older_quotadial_shows_the_providers_id_until_the_next_poll() {
        let dir = tempfile::tempdir().expect("temp dir");
        let history = History::open(&dir.path().join("history.db")).expect("opens");
        let mut older = snapshot(
            "codex",
            &[(WindowKind::Weekly, "Weekly", 60.0)],
            "2099-01-06T08:00:00Z",
        );
        older.account.provider_name = String::new();
        history
            .record(&older, at("2099-01-06T08:00:00Z"))
            .expect("records");
        let now = at("2099-01-07T00:00:00Z");

        let view = view(&history, &Settings::default(), 7, now, &Utc, Tier::Pro).expect("reads");

        assert_eq!(view.accounts[0].provider, "codex");
        assert_eq!(view.accounts[0].label, "codex");
        assert_eq!(
            label(&history, &Settings::default(), "codex user-1 -").as_deref(),
            Ok("codex")
        );
    }

    #[test]
    fn an_account_called_as_its_provider_is_named_once_in_history() {
        let dir = tempfile::tempdir().expect("temp dir");
        let history = History::open(&dir.path().join("history.db")).expect("opens");
        let mut example = snapshot(
            "codex",
            &[(WindowKind::Weekly, "Weekly", 60.0)],
            "2099-01-06T08:00:00Z",
        );
        example.account.provider_name = "Example".into();
        example.account.display_name = "Example".into();
        history
            .record(&example, at("2099-01-06T08:00:00Z"))
            .expect("records");
        let now = at("2099-01-07T00:00:00Z");

        let view = view(&history, &Settings::default(), 7, now, &Utc, Tier::Pro).expect("reads");

        assert_eq!(view.accounts[0].provider, "Example");
        assert_eq!(
            view.accounts[0].name, "",
            "no name beside the label it repeats"
        );
    }

    #[test]
    fn the_view_says_which_days_it_covers() {
        let (_dir, history) = history();
        let now = at("2099-01-07T00:00:00Z");

        let young = view(&history, &Settings::default(), 7, now, &Utc, Tier::Pro).expect("reads");
        assert_eq!(
            young.dates, "5 to 7 January 2099",
            "from the history's first day, as the charts begin"
        );
        let later = at("2099-01-20T12:00:00Z");
        for (fetched_at, used) in [
            ("2099-01-13T12:30:00Z", 5.0),
            ("2099-01-20T11:00:00Z", 40.0),
        ] {
            history
                .record(
                    &snapshot(
                        "claude",
                        &[(WindowKind::Session, "Session", used)],
                        fetched_at,
                    ),
                    at(fetched_at),
                )
                .expect("records");
        }
        let full = view(&history, &Settings::default(), 7, later, &Utc, Tier::Pro).expect("reads");
        assert_eq!(
            full.dates, "14 to 20 January 2099",
            "the heatmap's seven days"
        );
        assert_eq!(full.span, "The last 7 days.");

        assert_eq!(
            dates(day("2026-10-19"), day("2026-10-25")),
            "19 to 25 October 2026"
        );
        assert_eq!(
            dates(day("2026-09-26"), day("2026-10-25")),
            "26 September to 25 October 2026"
        );
        assert_eq!(
            dates(day("2026-12-29"), day("2027-01-04")),
            "29 December 2026 to 4 January 2027"
        );
        assert_eq!(
            dates(day("2026-10-25"), day("2026-10-25")),
            "25 October 2026"
        );
    }

    fn day(text: &str) -> NaiveDate {
        text.parse().expect("a date")
    }

    #[test]
    fn on_free_the_view_carries_the_line_in_place_of_the_share_buttons() {
        let (_dir, history) = history();
        let now = at("2099-01-07T00:00:00Z");

        let free = view(&history, &Settings::default(), 7, now, &Utc, Tier::Free).expect("reads");
        let pro = view(&history, &Settings::default(), 7, now, &Utc, Tier::Pro).expect("reads");

        assert_eq!(free.share_left_out, Some(ProFeature::ShareImage.line()));
        assert_eq!(pro.share_left_out, None);
        assert_eq!(free.accounts, pro.accounts, "Free sees the same history");
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
