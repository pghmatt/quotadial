//! What the app shows. The tray gets the share used of the tightest window across every account
//! and how close that is to the limit; the popover gets every account and window, each number with
//! its reset, its source and its age, said once for the account when its windows were all read from
//! one source at one moment (D-135).

use std::collections::BTreeMap;

use chrono::{DateTime, NaiveDate, Utc};
use quotadial_core::{Snapshot, Status, StatusReport, Window, text};
use serde::Serialize;

use crate::feed::Role;
use crate::settings::account_key;

/// From this share used, the meter warns.
pub const WARNING_PCT: f64 = 70.0;
/// From this share used, the meter is high: the window is close to its limit.
pub const HIGH_PCT: f64 = 90.0;

/// How close a window is to its limit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Level {
    /// No numbers yet.
    Unknown,
    /// Below [`WARNING_PCT`].
    Normal,
    /// From [`WARNING_PCT`] to below [`HIGH_PCT`].
    Warning,
    /// From [`HIGH_PCT`].
    High,
}

impl Level {
    fn of(used_pct: f64) -> Self {
        if used_pct >= HIGH_PCT {
            Self::High
        } else if used_pct >= WARNING_PCT {
            Self::Warning
        } else {
            Self::Normal
        }
    }
}

/// Everything the tray and the popover show.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Meter {
    /// The tightest window's share used, rounded down, if any account has numbers.
    pub percent: Option<u8>,
    pub level: Level,
    /// What that number is: its window, the account it belongs to and its reset, such as
    /// "Weekly · Claude Work · resets in 1 d 15 h" (D-132).
    pub headline: Option<String>,
    pub accounts: Vec<AccountView>,
    /// What needs the user's attention and belongs to no account, each naming the fix.
    pub problems: Vec<String>,
    /// The providers whose status pages report an incident, such as "Codex: Partial System
    /// Degradation" (D-096).
    pub incidents: Vec<String>,
    /// Where the numbers come from, such as "Running the daemon, port 47115".
    pub origin: Option<String>,
    /// What each provider's week would have cost at API prices, while the daemon reads the logs
    /// (D-097).
    pub costs: Vec<CostView>,
    /// What this copy leaves out because it is QuotaDial Free, a line each, shown under the
    /// accounts where the feature would be (D-100).
    pub left_out: Vec<String>,
    /// The window that sets the number, which the headline is about, kept so that the name the
    /// user gives its account can join the headline. Not sent: the headline says it.
    #[serde(skip)]
    tightest: Option<Tightest>,
}

/// The window that sets the tray's number, as the headline names it.
#[derive(Debug, Clone, PartialEq)]
struct Tightest {
    /// Its account, as the settings name it ([`account_key`]).
    key: String,
    /// Its account as named before any name the user gave it, such as "Claude dev@example.com", in
    /// case the account is not in the meter's list.
    account: String,
    /// Such as "Weekly".
    window: String,
    /// Such as "resets in 1 d 15 h".
    resets: String,
}

/// What a provider's week would have cost at API prices.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CostView {
    /// Such as "Claude".
    pub provider: String,
    /// Such as "$1,234 at API prices over the last 7 days, from 51,234 requests".
    pub summary: String,
    /// When the logs were read, and the prices' date.
    pub detail: String,
    /// Each model's part, the costliest first, when there is more than one model.
    pub models: Vec<String>,
    /// What the cost leaves out.
    pub notes: Vec<String>,
}

/// One account, as the popover shows it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AccountView {
    /// The account, as the settings name it ([`account_key`]).
    pub key: String,
    /// The name the user gave the account, if any, shown in place of `name`.
    pub alias: Option<String>,
    /// One line for the tray's menu and tooltip, such as
    /// "Claude user@example.com (max): Session 42%, resets in 2 h 10 min".
    pub summary: String,
    /// The provider's name, such as "Claude".
    pub provider: String,
    /// The account, such as its email address; empty when that says nothing beyond `provider`,
    /// as for an account called Example under the provider Example, so the provider shows once.
    pub name: String,
    pub plan: Option<String>,
    /// Such as "ok", "stale" or "sign-in required".
    pub status: String,
    pub ok: bool,
    /// Where and when the windows were read, said once for the card, such as "from the API 3 min
    /// ago", when every one was read from the same source at the same moment, as the windows of
    /// one answer are (D-135). None when they differ, and with no window, and then each window
    /// says its own, in [`WindowView::fetched`].
    pub fetched: Option<String>,
    /// Tightest first.
    pub windows: Vec<WindowView>,
    /// Such as "$12.50 of $50.00 used (25%)", "on" or "off", in `text::extra_usage`'s words.
    pub extra_usage: Option<String>,
    /// Why the status is not ok, naming the fix.
    pub note: Option<String>,
    /// What QuotaDial does with this account's sign-in, in its provider's words, for "What
    /// QuotaDial does with this sign-in" (P-020). Empty for an account from an older daemon, which
    /// says none, and then the panel is left out.
    pub sign_in: String,
}

/// One limit window, as the popover shows it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WindowView {
    pub label: String,
    /// The share used, from 0 to 100.
    pub used_pct: f64,
    pub level: Level,
    /// Such as "resets in 2 h 10 min".
    pub resets: String,
    /// Where and when it was read, such as "from the API 4 min ago"; none when the card says so
    /// once for all its windows ([`AccountView::fetched`]).
    pub fetched: Option<String>,
    /// A sudden 0% that a second fetch has not confirmed yet.
    pub unconfirmed: bool,
    /// How fast the window is filling, such as "at 8.5%/h, runs out in 1 h 30 min, 40 min before it
    /// resets", when the daemon's history has enough to go on.
    pub pace: Option<String>,
    /// Whether, at that pace, the window runs out before it resets.
    pub runs_out: bool,
}

impl Meter {
    /// The meter for `report`, or for no report yet, with `problem` saying what is wrong.
    #[must_use]
    pub fn new(
        report: Option<&StatusReport>,
        problem: Option<&str>,
        role: Option<Role>,
        now: DateTime<Utc>,
    ) -> Self {
        let mut meter = Self {
            percent: None,
            level: Level::Unknown,
            headline: None,
            accounts: Vec::new(),
            problems: problem.map(str::to_owned).into_iter().collect(),
            incidents: Vec::new(),
            costs: Vec::new(),
            left_out: Vec::new(),
            tightest: None,
            origin: role.map(|role| match role {
                Role::Running { port } => format!("Running the daemon, port {port}"),
                Role::Following => "Reading `quotadial daemon`".to_owned(),
            }),
        };
        let Some(report) = report else {
            return meter;
        };
        // An account waiting for its owner to sign in again keeps its last numbers on show in the
        // popover, but they are too old to set the tray's number.
        let tightest = report
            .snapshots
            .iter()
            .filter(|snapshot| {
                !matches!(snapshot.status, Status::AuthRequired | Status::Unsupported)
            })
            .flat_map(|snapshot| {
                snapshot
                    .windows
                    .iter()
                    .map(move |window| (snapshot, window))
            })
            .max_by(|(_, a), (_, b)| a.used_pct.total_cmp(&b.used_pct));
        if let Some((snapshot, window)) = tightest {
            let used_pct = window.used_pct.clamp(0.0, 100.0);
            meter.percent = Some(used_pct.floor() as u8);
            meter.level = Level::of(used_pct);
            let account = &snapshot.account;
            meter.tightest = Some(Tightest {
                key: account_key(account),
                account: text::labelled(
                    account.provider_label(),
                    account.name_beside_provider().unwrap_or_default(),
                ),
                window: window.label.clone(),
                resets: text::resets(window.resets_at, now),
            });
        }
        meter.accounts = report
            .snapshots
            .iter()
            .map(|snapshot| account_view(snapshot, now))
            .collect();
        meter.headline = meter.headline();
        // A provider's under its own name, as every surface shows it (D-128).
        meter.problems.extend(report.error_lines());
        // What is wrong with QuotaDial itself, such as a history it could not save (D-107).
        meter.problems.extend(report.problems.iter().cloned());
        meter.incidents = report
            .status_pages
            .iter()
            .filter(|page| page.indicator.is_incident())
            .map(|page| {
                format!(
                    "{}: {}",
                    report.provider_name(&page.provider),
                    page.description
                )
            })
            .collect();
        meter.left_out = report
            .left_out
            .iter()
            .map(|left_out| left_out.text.clone())
            .collect();
        meter.costs = report
            .costs
            .iter()
            .map(|cost| CostView {
                provider: report.provider_name(&cost.provider).to_owned(),
                summary: text::cost(cost),
                detail: format!(
                    "Read {}, at the prices of {}",
                    text::ago(cost.until, now),
                    prices_of(&cost.prices_updated)
                ),
                models: if cost.models.len() > 1 {
                    cost.models.iter().map(text::model_cost).collect()
                } else {
                    Vec::new()
                },
                notes: cost.notes.clone(),
            })
            .collect();
        meter
    }

    /// The text beside the tray icon, such as "42%", or "–" before there are numbers, with "⚠"
    /// while a provider has an incident.
    #[must_use]
    pub fn title(&self) -> String {
        let number = self
            .percent
            .map_or_else(|| "–".to_owned(), |percent| format!("{percent}%"));
        if self.incidents.is_empty() {
            number
        } else {
            format!("{number} \u{26a0}")
        }
    }

    /// One line per account, then the problems: the tray menu's information.
    #[must_use]
    pub fn lines(&self) -> Vec<String> {
        let mut lines: Vec<String> = Vec::new();
        for account in &self.accounts {
            lines.push(account.summary.clone());
            lines.extend(account.note.iter().filter(|_| !account.ok).cloned());
        }
        lines.extend(self.problems.iter().cloned());
        lines.extend(
            self.incidents
                .iter()
                .map(|incident| format!("Status: {incident}")),
        );
        lines.extend(
            self.costs
                .iter()
                .map(|cost| format!("{}: {}", cost.provider, cost.summary)),
        );
        lines
    }

    /// The meter with the names the user gave accounts in place of their email addresses. Each
    /// provider's accounts come sorted by email address in the report, and are sorted here by the
    /// name shown instead.
    #[must_use]
    pub fn with_aliases(mut self, aliases: &BTreeMap<String, String>) -> Self {
        for account in &mut self.accounts {
            if let Some(alias) = aliases.get(&account.key) {
                let unnamed = account.label();
                account.alias = Some(alias.clone());
                account.summary = account.summary.replacen(&unnamed, &account.label(), 1);
            }
        }
        for provider in self.accounts.chunk_by_mut(|a, b| a.provider == b.provider) {
            provider.sort_by_cached_key(|account| {
                account
                    .alias
                    .as_deref()
                    .unwrap_or(&account.name)
                    .to_lowercase()
            });
        }
        self.headline = self.headline();
        self
    }

    /// The headline for the window that sets the number: the window, its account as the popover
    /// names it, and its reset, such as "Weekly · Claude Work · resets in 1 d 15 h". It names the
    /// account because two accounts of one provider must not be confused: by the name given it,
    /// or else by its address (D-132).
    fn headline(&self) -> Option<String> {
        let tightest = self.tightest.as_ref()?;
        let account = self
            .accounts
            .iter()
            .find(|account| account.key == tightest.key)
            .map_or_else(|| tightest.account.clone(), AccountView::label);
        Some(format!(
            "{} · {account} · {}",
            tightest.window, tightest.resets
        ))
    }

    /// The tray icon's tooltip: the app's name, the number and its headline, as the popover's
    /// summary says them, then [`Meter::lines`].
    #[must_use]
    pub fn tooltip(&self) -> String {
        let mut lines = vec!["QuotaDial".to_owned()];
        if let (Some(percent), Some(headline)) = (self.percent, &self.headline) {
            lines.push(format!("{percent}% · {headline}"));
        }
        lines.extend(self.lines());
        if lines.len() == 1 {
            lines.push("Looking for the accounts on this machine…".to_owned());
        }
        lines.join("\n")
    }
}

impl AccountView {
    /// The account as the headline and the tray's lines name it: its provider's name with the
    /// name the user gave it, or else its address, or the provider's name alone when the address
    /// adds nothing, such as "Claude Work", "Claude dev@example.com" or "Example".
    fn label(&self) -> String {
        match &self.alias {
            Some(alias) => format!("{} {alias}", self.provider),
            None => text::labelled(&self.provider, &self.name),
        }
    }
}

fn account_view(snapshot: &Snapshot, now: DateTime<Utc>) -> AccountView {
    let account = &snapshot.account;
    // As the provider names itself and says what QuotaDial does with the sign-in (P-020); an
    // account from an older daemon shows its provider's id and no sign-in panel until the next
    // poll.
    let provider = account.provider_label().to_owned();
    // An account called as its provider is, such as the example build's, shows the label once.
    let name = account
        .name_beside_provider()
        .unwrap_or_default()
        .to_owned();
    let windows = snapshot.windows_tightest_first();
    let fetched = fetched_together(&windows, now);
    let said_by_card = fetched.is_some();
    let mut summary = text::labelled(&provider, &name);
    if let Some(plan) = &account.plan {
        summary.push_str(&format!(" ({plan})"));
    }
    match windows.first() {
        Some(window) => summary.push_str(&format!(
            ": {} {:.0}%, {}",
            window.label,
            window.used_pct.floor(),
            text::resets(window.resets_at, now)
        )),
        None => summary.push_str(&format!(": {}", text::status_label(snapshot.status))),
    }
    AccountView {
        key: account_key(account),
        alias: None,
        summary,
        sign_in: account.sign_in_use.clone(),
        provider,
        name,
        plan: account.plan.clone(),
        status: text::status_label(snapshot.status).to_owned(),
        ok: snapshot.status == Status::Ok,
        fetched,
        windows: windows
            .into_iter()
            .map(|window| window_view(window, now, said_by_card))
            .collect(),
        extra_usage: snapshot.extra_usage.as_ref().map(text::extra_usage),
        note: snapshot.error.clone(),
    }
}

/// The day the price table was updated, `2026-10-23` in the cost, as a person reads a day alone
/// (D-142): `23 Oct 2026`. Anything that is not a day, which QuotaDial's own table never holds, is
/// written as it is.
fn prices_of(updated: &str) -> String {
    updated
        .parse::<NaiveDate>()
        .map_or_else(|_| updated.to_owned(), text::date)
}

/// Where and when `window` was read, such as "from the API 4 min ago".
fn fetched_line(window: &Window, now: DateTime<Utc>) -> String {
    format!(
        "from {} {}",
        text::source_label(window.source),
        text::ago(window.fetched_at, now)
    )
}

/// What a card says once of where its windows were read, when every one was read from the same
/// source at exactly the same moment, as the windows of one answer are: Claude's and Codex's from
/// one response, Antigravity's from one handover, the example's from one clock reading, and a
/// replay from the cache keeps each window's time (D-135). Exactly, and not to the same minute as
/// written: the rule reads the times, not the words, so the layout stays as it is while the
/// minutes tick. A card with one window says it for the card like any other, so the line stays
/// where it is when a limit leaves a card after its reset, as Antigravity's do, and is in the same
/// place on every card; a card with no window has nothing to say.
fn fetched_together(windows: &[&Window], now: DateTime<Utc>) -> Option<String> {
    let (first, rest) = windows.split_first()?;
    rest.iter()
        .all(|window| window.source == first.source && window.fetched_at == first.fetched_at)
        .then(|| fetched_line(first, now))
}

/// `window` as the popover shows it. When its card says where every window was read
/// (`said_by_card`), the window leaves that out.
fn window_view(window: &Window, now: DateTime<Utc>, said_by_card: bool) -> WindowView {
    let used_pct = window.used_pct.clamp(0.0, 100.0);
    WindowView {
        label: window.label.clone(),
        used_pct,
        level: Level::of(used_pct),
        resets: text::resets(window.resets_at, now),
        fetched: (!said_by_card).then(|| fetched_line(window, now)),
        unconfirmed: window.unconfirmed,
        pace: window
            .pace
            .as_ref()
            .map(|pace| text::pace(pace, window.resets_at, now)),
        runs_out: window
            .pace
            .as_ref()
            .is_some_and(|pace| pace.runs_out_at.is_some()),
    }
}

#[cfg(test)]
mod tests {
    use quotadial_core::{Account, ExtraUsage, Pace, ProviderError, Source, WindowKind};

    use super::*;

    fn at(text: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(text)
            .expect("valid timestamp")
            .with_timezone(&Utc)
    }

    fn now() -> DateTime<Utc> {
        at("2099-01-01T00:04:00Z")
    }

    fn window(label: &str, used_pct: f64, resets_at: &str) -> Window {
        Window {
            kind: WindowKind::Weekly,
            label: label.into(),
            used_pct,
            resets_at: Some(at(resets_at)),
            window_seconds: 604_800,
            scope_model: None,
            source: Source::Api,
            fetched_at: at("2099-01-01T00:00:00Z"),
            unconfirmed: false,
            pace: None,
        }
    }

    /// A snapshot of an account of `provider`, which names itself as its id capitalised and says
    /// what QuotaDial does with the sign-in, as every provider does (P-020).
    fn snapshot(provider: &str, plan: Option<&str>, windows: Vec<Window>) -> Snapshot {
        let mut letters = provider.chars();
        let name: String = letters
            .next()
            .map(|first| first.to_uppercase().chain(letters).collect())
            .unwrap_or_default();
        Snapshot {
            account: Account {
                provider: provider.into(),
                provider_name: name,
                account_id: "user-1".into(),
                display_name: "user@example.com".into(),
                plan: plan.map(str::to_owned),
                org: None,
                sign_in: None,
                sign_in_use: format!("QuotaDial reads the {provider} test's sign-in."),
            },
            windows,
            extra_usage: None,
            status: Status::Ok,
            error: None,
        }
    }

    fn two_accounts() -> StatusReport {
        StatusReport {
            snapshots: vec![
                snapshot(
                    "claude",
                    Some("max"),
                    vec![
                        window("Weekly", 12.0, "2099-01-04T00:04:00Z"),
                        window("Session", 42.6, "2099-01-01T02:14:00Z"),
                    ],
                ),
                snapshot(
                    "codex",
                    Some("pro"),
                    vec![window("Weekly", 73.9, "2099-01-07T03:04:00Z")],
                ),
            ],
            errors: Vec::new(),
            status_pages: Vec::new(),
            costs: Vec::new(),
            left_out: Vec::new(),
            problems: Vec::new(),
            providers: [("claude", "Claude"), ("codex", "Codex")]
                .map(|(id, name)| quotadial_core::ProviderName {
                    id: id.into(),
                    name: name.into(),
                })
                .to_vec(),
        }
    }

    /// The meter, at `at`, for one Claude account with `windows`.
    fn claude_meter(windows: Vec<Window>, at: DateTime<Utc>) -> Meter {
        let mut report = two_accounts();
        report.snapshots = vec![snapshot("claude", Some("max"), windows)];
        Meter::new(Some(&report), None, None, at)
    }

    #[test]
    fn an_incident_puts_a_mark_beside_the_number_and_says_what_it_is() {
        let mut report = two_accounts();
        report.status_pages = vec![
            quotadial_core::StatusPage {
                provider: "codex".into(),
                indicator: quotadial_core::Indicator::Minor,
                description: "Partial System Degradation".into(),
                url: "https://status.openai.com".into(),
                since: Some(at("2099-01-01T00:00:00Z")),
                checked_at: now(),
            },
            quotadial_core::StatusPage {
                provider: "claude".into(),
                indicator: quotadial_core::Indicator::Maintenance,
                description: "Scheduled maintenance".into(),
                url: "https://status.claude.com".into(),
                since: None,
                checked_at: now(),
            },
        ];

        let meter = Meter::new(Some(&report), None, None, now());

        assert_eq!(meter.incidents, ["Codex: Partial System Degradation"]);
        assert!(meter.title().ends_with("% \u{26a0}"), "{}", meter.title());
        assert!(
            meter
                .lines()
                .contains(&"Status: Codex: Partial System Degradation".to_owned())
        );
        assert!(
            !Meter::new(Some(&two_accounts()), None, None, now())
                .title()
                .contains('\u{26a0}'),
            "no mark without an incident"
        );
    }

    #[test]
    fn each_provider_s_week_shows_what_it_would_have_cost() {
        let model = |name: &str, usd: Option<f64>, requests: u64| quotadial_core::ModelCost {
            model: name.into(),
            usd,
            requests,
            tokens: quotadial_core::Tokens::default(),
        };
        let cost = |provider: &str, models: Vec<quotadial_core::ModelCost>| quotadial_core::Cost {
            provider: provider.into(),
            since: now() - chrono::TimeDelta::minutes(4) - chrono::TimeDelta::days(7),
            until: now() - chrono::TimeDelta::minutes(4),
            usd: models.iter().filter_map(|model| model.usd).sum(),
            requests: models.iter().map(|model| model.requests).sum(),
            tokens: quotadial_core::Tokens::default(),
            models,
            prices_updated: "2026-10-23".into(),
            notes: Vec::new(),
        };
        let mut report = two_accounts();
        report.costs = vec![
            cost(
                "claude",
                vec![model("claude-opus-5-5", Some(1_234.4), 51_234)],
            ),
            cost(
                "codex",
                vec![
                    model("gpt-6-astra", Some(20.0), 900),
                    model("gpt-5.5", Some(3.5), 100),
                ],
            ),
        ];

        let meter = Meter::new(Some(&report), None, None, now());

        assert_eq!(
            meter.costs[0],
            CostView {
                provider: "Claude".into(),
                summary: "$1,234 at API prices over the last 7 days, from 51,234 requests".into(),
                detail: "Read 4 min ago, at the prices of 23 Oct 2026".into(),
                models: Vec::new(),
                notes: Vec::new(),
            }
        );
        assert_eq!(
            meter.costs[1].models,
            [
                "gpt-6-astra: $20.00, 900 requests",
                "gpt-5.5: $3.50, 100 requests"
            ]
        );
        assert!(meter.lines().contains(
            &"Codex: $23.50 at API prices over the last 7 days, from 1,000 requests".to_owned()
        ));
    }

    #[test]
    fn a_window_shows_its_pace_and_whether_it_runs_out_before_it_resets() {
        let mut filling = window("Weekly", 62.0, "2099-01-03T04:00:00Z");
        filling.pace = Some(Pace {
            pct_per_hour: 2.0,
            runs_out_at: Some(at("2099-01-01T19:00:00Z")),
        });
        let mut lasting = window("Session", 10.0, "2099-01-01T05:00:00Z");
        lasting.pace = Some(Pace {
            pct_per_hour: 1.0,
            runs_out_at: None,
        });
        let report = StatusReport {
            snapshots: vec![snapshot("claude", None, vec![filling, lasting])],
            errors: Vec::new(),
            status_pages: Vec::new(),
            costs: Vec::new(),
            left_out: Vec::new(),
            problems: Vec::new(),
            providers: Vec::new(),
        };

        let meter = Meter::new(Some(&report), None, None, now());

        let windows = &meter.accounts[0].windows;
        assert_eq!(
            windows[0].pace.as_deref(),
            Some("at 2.0%/h, runs out in 18 h 56 min, 1 d 9 h before it resets")
        );
        assert!(windows[0].runs_out);
        assert_eq!(
            windows[1].pace.as_deref(),
            Some("at 1.0%/h, lasts until it resets")
        );
        assert!(!windows[1].runs_out);
    }

    #[test]
    fn the_tray_shows_the_tightest_window_across_every_account() {
        let meter = Meter::new(Some(&two_accounts()), None, None, now());

        assert_eq!(meter.percent, Some(73));
        assert_eq!(meter.level, Level::Warning);
        assert_eq!(meter.title(), "73%");
        assert_eq!(
            meter.headline.as_deref(),
            Some("Weekly · Codex user@example.com · resets in 6 d 3 h")
        );
        assert_eq!(
            meter.lines(),
            [
                "Claude user@example.com (max): Session 42%, resets in 2 h 10 min",
                "Codex user@example.com (pro): Weekly 73%, resets in 6 d 3 h",
            ]
        );
    }

    /// Two Claude accounts, the second of them nearer its limit.
    fn two_claude_accounts() -> StatusReport {
        let mut report = two_accounts();
        let mut second = snapshot(
            "claude",
            Some("pro"),
            vec![window("Weekly", 88.0, "2099-01-02T15:04:00Z")],
        );
        second.account.account_id = "user-2".into();
        second.account.display_name = "dev@example.com".into();
        report.snapshots.insert(1, second);
        report
    }

    #[test]
    fn the_headline_names_the_account_the_number_belongs_to_by_its_address() {
        // Two Claude accounts must not be confused (D-132): the window that sets the number is
        // the second's, and only its address says which.
        let meter = Meter::new(Some(&two_claude_accounts()), None, None, now());

        assert_eq!(meter.percent, Some(88));
        assert_eq!(
            meter.headline.as_deref(),
            Some("Weekly · Claude dev@example.com · resets in 1 d 15 h")
        );
    }

    #[test]
    fn the_headline_names_the_account_by_the_name_the_user_gave_it() {
        let meter = Meter::new(Some(&two_claude_accounts()), None, None, now()).with_aliases(
            &BTreeMap::from([
                ("claude user-2 -".to_owned(), "Work".to_owned()),
                ("claude user-1 -".to_owned(), "Home".to_owned()),
            ]),
        );

        assert_eq!(
            meter.headline.as_deref(),
            Some("Weekly · Claude Work · resets in 1 d 15 h"),
            "the name given, not the address, even after the accounts are sorted by name"
        );
        assert_eq!(meter.accounts[0].alias.as_deref(), Some("Home"));
        assert!(
            meter
                .lines()
                .contains(&"Claude Work (pro): Weekly 88%, resets in 1 d 15 h".to_owned()),
            "{:?}",
            meter.lines()
        );
    }

    #[test]
    fn the_headline_names_an_example_account_once() {
        // Called as its provider is (D-124), so its provider's name alone says which account.
        let mut report = two_accounts();
        report.snapshots[1] = Snapshot {
            account: Account {
                provider: "fake".into(),
                provider_name: "Example".into(),
                account_id: "example-2".into(),
                display_name: "Example".into(),
                plan: Some("ChatGPT Pro".into()),
                org: None,
                sign_in: None,
                sign_in_use: String::new(),
            },
            ..snapshot(
                "fake",
                None,
                vec![window("Session", 97.0, "2099-01-01T02:14:00Z")],
            )
        };

        let meter = Meter::new(Some(&report), None, None, now());

        assert_eq!(
            meter.headline.as_deref(),
            Some("Session · Example · resets in 2 h 10 min")
        );
        let named = meter.with_aliases(&BTreeMap::from([(
            "fake example-2 -".to_owned(),
            "Demo".to_owned(),
        )]));
        assert_eq!(
            named.headline.as_deref(),
            Some("Session · Example Demo · resets in 2 h 10 min")
        );
    }

    #[test]
    fn the_tooltip_says_the_number_and_its_headline_as_the_popover_does() {
        let meter = Meter::new(Some(&two_claude_accounts()), None, None, now()).with_aliases(
            &BTreeMap::from([("claude user-2 -".to_owned(), "Work".to_owned())]),
        );

        let tooltip = meter.tooltip();
        let lines: Vec<&str> = tooltip.lines().collect();

        assert_eq!(
            lines[..2],
            [
                "QuotaDial",
                "88% · Weekly · Claude Work · resets in 1 d 15 h"
            ],
            "{tooltip}"
        );
        assert_eq!(
            &lines[2..],
            meter.lines(),
            "then every account, as the menu lists them"
        );
    }

    #[test]
    fn the_popover_shows_every_window_tightest_first_with_its_age() {
        let mut report = two_accounts();
        report.snapshots[0].extra_usage = Some(ExtraUsage {
            enabled: false,
            used_pct: None,
            used: None,
            limit: None,
            currency: None,
            balance: None,
        });
        report.snapshots[0].windows[0].unconfirmed = true;

        let meter = Meter::new(Some(&report), None, None, now());
        let claude = &meter.accounts[0];

        assert_eq!(claude.provider, "Claude");
        assert_eq!(claude.name, "user@example.com");
        assert_eq!(claude.plan.as_deref(), Some("max"));
        assert_eq!((claude.status.as_str(), claude.ok), ("ok", true));
        assert_eq!(claude.extra_usage.as_deref(), Some("off"));
        assert_eq!(claude.sign_in, "QuotaDial reads the claude test's sign-in.");
        // Both windows were read from the API at one moment, so the card says so once (D-135).
        assert_eq!(claude.fetched.as_deref(), Some("from the API 4 min ago"));
        assert_eq!(
            claude.windows,
            [
                WindowView {
                    label: "Session".into(),
                    used_pct: 42.6,
                    level: Level::Normal,
                    resets: "resets in 2 h 10 min".into(),
                    fetched: None,
                    unconfirmed: false,
                    pace: None,
                    runs_out: false,
                },
                WindowView {
                    label: "Weekly".into(),
                    used_pct: 12.0,
                    level: Level::Normal,
                    resets: "resets in 3 d".into(),
                    fetched: None,
                    unconfirmed: true,
                    pace: None,
                    runs_out: false,
                },
            ]
        );
        assert_eq!(
            meter.accounts[1].sign_in,
            "QuotaDial reads the codex test's sign-in."
        );
    }

    /// Claude Work's three windows as one answer gives them, each read at 00:00:00 from the API.
    fn one_answer() -> Vec<Window> {
        vec![
            window("Weekly", 91.0, "2099-01-02T15:04:00Z"),
            window("Weekly (Opus)", 74.0, "2099-01-02T15:04:00Z"),
            window("Session", 38.0, "2099-01-01T02:14:00Z"),
        ]
    }

    /// What each window of the first account says for itself about where it was read.
    fn said_by_each(meter: &Meter) -> Vec<(&str, Option<&str>)> {
        meter.accounts[0]
            .windows
            .iter()
            .map(|window| (window.label.as_str(), window.fetched.as_deref()))
            .collect()
    }

    #[test]
    fn a_card_says_once_where_its_windows_were_read_when_every_one_agrees() {
        let meter = claude_meter(one_answer(), now());

        let claude = &meter.accounts[0];
        assert_eq!(claude.fetched.as_deref(), Some("from the API 4 min ago"));
        assert_eq!(
            said_by_each(&meter),
            [("Weekly", None), ("Weekly (Opus)", None), ("Session", None)],
            "no window says it again"
        );
        // What each window still says for itself is its reset, and its pace when it has one.
        let resets: Vec<&str> = claude.windows.iter().map(|w| w.resets.as_str()).collect();
        assert_eq!(
            resets,
            [
                "resets in 1 d 15 h",
                "resets in 1 d 15 h",
                "resets in 2 h 10 min"
            ]
        );
    }

    #[test]
    fn a_window_read_at_another_moment_leaves_the_card_silent_and_every_window_says_its_own() {
        let mut windows = one_answer();
        windows[2].fetched_at = at("2099-01-01T00:02:30Z");

        let meter = claude_meter(windows, now());

        assert_eq!(
            meter.accounts[0].fetched, None,
            "two of three agree, not all"
        );
        assert_eq!(
            said_by_each(&meter),
            [
                ("Weekly", Some("from the API 4 min ago")),
                ("Weekly (Opus)", Some("from the API 4 min ago")),
                ("Session", Some("from the API 1 min ago")),
            ]
        );
    }

    #[test]
    fn a_window_read_from_another_source_leaves_the_card_silent_too() {
        let mut windows = one_answer();
        windows[2].source = Source::LocalLog;

        let meter = claude_meter(windows, now());

        assert_eq!(meter.accounts[0].fetched, None);
        assert_eq!(
            said_by_each(&meter),
            [
                ("Weekly", Some("from the API 4 min ago")),
                ("Weekly (Opus)", Some("from the API 4 min ago")),
                ("Session", Some("from local logs 4 min ago")),
            ],
            "the same moment, but not the same source"
        );
    }

    #[test]
    fn windows_read_moments_apart_are_never_said_once_so_the_layout_holds_as_the_minutes_tick() {
        // Read 40 seconds apart. Said in minutes they differ at some moments ("4 min ago" and
        // "3 min ago") and agree at others, and a card whose line came and went with them would
        // move under the pointer every minute.
        let mut windows = one_answer();
        windows.truncate(2);
        windows[1].fetched_at = at("2099-01-01T00:00:40Z");

        for moment in [
            "2099-01-01T00:03:30Z",
            "2099-01-01T00:04:10Z",
            "2099-01-01T00:04:50Z",
            "2099-01-01T00:05:20Z",
        ] {
            let meter = claude_meter(windows.clone(), at(moment));
            assert_eq!(meter.accounts[0].fetched, None, "at {moment}");
            assert!(
                said_by_each(&meter).iter().all(|(_, said)| said.is_some()),
                "at {moment}: {:?}",
                said_by_each(&meter)
            );
        }
        let alike = claude_meter(windows, at("2099-01-01T00:04:50Z"));
        assert_eq!(
            said_by_each(&alike),
            [
                ("Weekly", Some("from the API 4 min ago")),
                ("Weekly (Opus)", Some("from the API 4 min ago"))
            ],
            "the words agree at this moment, and the card still keeps to the times"
        );
    }

    #[test]
    fn a_card_with_one_window_says_where_it_was_read_for_the_card_like_any_other() {
        // As a card is when a limit leaves it after its reset, as Antigravity's do, and one window
        // is left.
        let three = claude_meter(one_answer(), now());
        let one = claude_meter(one_answer().into_iter().take(1).collect(), now());

        assert_eq!(
            one.accounts[0].fetched.as_deref(),
            Some("from the API 4 min ago")
        );
        assert_eq!(said_by_each(&one), [("Weekly", None)]);
        assert_eq!(
            one.accounts[0].fetched, three.accounts[0].fetched,
            "the line stays where it is, and does not jump to the window that is left"
        );
    }

    #[test]
    fn an_account_with_no_windows_has_no_line_about_where_they_were_read() {
        let meter = claude_meter(Vec::new(), now());

        assert_eq!(meter.accounts[0].fetched, None);
        assert!(meter.accounts[0].windows.is_empty());
    }

    #[test]
    fn a_snapshot_served_again_from_the_cache_is_still_said_once() {
        // The gate and the providers replay the last snapshot within the polling floor: each
        // window that came from the API now says the cache, with the time of the fetch.
        let mut report = two_accounts();
        report.snapshots = vec![snapshot("claude", Some("max"), one_answer()).served_from_cache()];

        let meter = Meter::new(Some(&report), None, None, now());

        assert_eq!(
            meter.accounts[0].fetched.as_deref(),
            Some("from cache 4 min ago")
        );
        assert_eq!(
            said_by_each(&meter),
            [("Weekly", None), ("Weekly (Opus)", None), ("Session", None)]
        );
    }

    #[test]
    fn the_window_is_sent_the_line_once_for_the_card_or_under_each_window_and_never_both() {
        let sent = |windows| serde_json::to_value(claude_meter(windows, now())).expect("JSON");
        let mut apart = one_answer();
        apart[0].source = Source::Cache;

        let agreeing = sent(one_answer());
        let differing = sent(apart);

        let card = &agreeing["accounts"][0];
        assert_eq!(card["fetched"], "from the API 4 min ago");
        assert!(card["windows"].as_array().is_some_and(|windows| {
            windows.len() == 3 && windows.iter().all(|window| window["fetched"].is_null())
        }));
        let card = &differing["accounts"][0];
        assert!(card["fetched"].is_null());
        assert_eq!(card["windows"][0]["fetched"], "from cache 4 min ago");
        assert_eq!(card["windows"][1]["fetched"], "from the API 4 min ago");
    }

    #[test]
    fn a_provider_the_app_has_never_heard_of_is_shown_as_it_names_itself() {
        let mut report = two_accounts();
        report.snapshots[0] = snapshot(
            "copilot",
            None,
            vec![window("Credits", 97.0, "2099-01-04T00:04:00Z")],
        );
        report.snapshots[0].account.provider_name = "GitHub Copilot".into();
        report.snapshots[0].account.sign_in_use =
            "QuotaDial reads the sign-in the Copilot CLI keeps. It never changes it.".into();

        let meter = Meter::new(Some(&report), None, None, now());

        let shown = &meter.accounts[0];
        assert_eq!(shown.provider, "GitHub Copilot");
        assert_eq!(
            shown.sign_in,
            "QuotaDial reads the sign-in the Copilot CLI keeps. It never changes it."
        );
        assert_eq!(
            meter.headline.as_deref(),
            Some("Credits · GitHub Copilot user@example.com · resets in 3 d")
        );
        assert!(
            meter
                .tooltip()
                .contains("GitHub Copilot user@example.com: Credits 97%"),
            "{}",
            meter.tooltip()
        );
    }

    #[test]
    fn an_account_from_an_older_daemon_shows_its_providers_id_and_no_sign_in_panel() {
        let older: Account = serde_json::from_value(serde_json::json!({
            "provider": "claude",
            "account_id": "user-1",
            "display_name": "user@example.com"
        }))
        .expect("an account an older daemon sent");
        let mut report = two_accounts();
        report.snapshots[0].account = older;

        let meter = Meter::new(Some(&report), None, None, now());

        let shown = &meter.accounts[0];
        assert_eq!(shown.provider, "claude", "its id until the next poll");
        assert!(
            shown.summary.starts_with("claude user@example.com"),
            "{shown:?}"
        );
        assert_eq!(shown.sign_in, "", "nothing said, so no panel");
    }

    #[test]
    fn an_example_account_says_its_numbers_are_examples_and_that_it_has_no_sign_in() {
        let example = |source| Window {
            source,
            ..window("Session", 38.0, "2099-01-01T02:14:00Z")
        };
        let mut report = two_accounts();
        report.snapshots[0] = Snapshot {
            account: Account {
                provider: "fake".into(),
                provider_name: "Example".into(),
                account_id: "example-1".into(),
                display_name: "Example".into(),
                plan: Some("Claude Max 5x".into()),
                org: None,
                sign_in: None,
                sign_in_use: "An example account, made up by QuotaDial's example build for \
                              pictures of the app. It has no sign-in, and QuotaDial asks no one \
                              for its numbers."
                    .into(),
            },
            ..snapshot("fake", None, vec![example(Source::Example)])
        };

        let meter = Meter::new(Some(&report), None, None, now());
        let shown = &meter.accounts[0];

        // Called as its provider is, so its provider's label shows once, with no name beside it.
        assert_eq!(shown.provider, "Example");
        assert_eq!(shown.name, "");
        assert!(
            shown
                .summary
                .starts_with("Example (Claude Max 5x): Session 38%"),
            "no doubled word and no double space: {}",
            shown.summary
        );
        let named = meter
            .clone()
            .with_aliases(&BTreeMap::from([(shown.key.clone(), "Work".to_owned())]));
        assert!(
            named.accounts[0]
                .summary
                .starts_with("Example Work (Claude Max 5x): Session 38%"),
            "a name the user gives still shows: {}",
            named.accounts[0].summary
        );
        // One window, which the card says it for as it does for any (D-135).
        assert_eq!(
            shown.fetched.as_deref(),
            Some("from the example build 4 min ago")
        );
        assert_eq!(shown.windows[0].fetched, None);
        assert_eq!(
            shown.sign_in,
            "An example account, made up by QuotaDial's example build for pictures of the app. It \
             has no sign-in, and QuotaDial asks no one for its numbers."
        );
        assert_eq!(
            meter.accounts[1].sign_in, "QuotaDial reads the codex test's sign-in.",
            "a real account says what its provider says"
        );
    }

    #[test]
    fn the_level_rises_at_seventy_and_ninety_percent() {
        let level = |used_pct| {
            let report = StatusReport {
                snapshots: vec![snapshot(
                    "claude",
                    None,
                    vec![window("Session", used_pct, "2099-01-01T02:00:00Z")],
                )],
                errors: Vec::new(),
                status_pages: Vec::new(),
                costs: Vec::new(),
                left_out: Vec::new(),
                problems: Vec::new(),
                providers: Vec::new(),
            };
            let meter = Meter::new(Some(&report), None, None, now());
            assert_eq!(meter.accounts[0].windows[0].level, meter.level);
            meter.level
        };

        assert_eq!(level(69.9), Level::Normal);
        assert_eq!(level(70.0), Level::Warning);
        assert_eq!(level(89.9), Level::Warning);
        assert_eq!(level(90.0), Level::High);
        assert_eq!(level(130.0), Level::High);
    }

    #[test]
    fn without_numbers_the_tray_says_so_and_names_what_is_wrong() {
        let mut expired = snapshot("claude", Some("max"), Vec::new());
        expired.status = Status::AuthRequired;
        expired.error = Some("Run `claude` to sign in again.".into());
        let report = StatusReport {
            snapshots: vec![expired],
            errors: vec![ProviderError {
                provider: "codex".into(),
                message: "could not read the sign-in from ~/.codex/auth.json".into(),
            }],
            status_pages: Vec::new(),
            costs: Vec::new(),
            left_out: Vec::new(),
            problems: Vec::new(),
            providers: Vec::new(),
        };

        let meter = Meter::new(Some(&report), None, None, now());

        assert_eq!(meter.percent, None);
        assert_eq!(meter.level, Level::Unknown);
        assert_eq!(meter.headline, None);
        assert_eq!(meter.title(), "–");
        assert!(!meter.accounts[0].ok);
        assert_eq!(
            meter.tooltip(),
            "QuotaDial\nClaude user@example.com (max): sign-in required\nRun `claude` to sign in \
             again.\ncodex: could not read the sign-in from ~/.codex/auth.json"
        );
    }

    #[test]
    fn a_providers_problem_is_named_as_the_provider_names_itself() {
        let error = |provider: &str, message: &str| ProviderError {
            provider: provider.into(),
            message: message.into(),
        };
        let report = StatusReport {
            snapshots: Vec::new(),
            errors: vec![
                error(
                    "gemini",
                    "could not read the sign-in from ~/.gemini/settings.json: it is not a plain \
                     file",
                ),
                error("copilot", "request failed: GitHub did not answer."),
                // From a daemon older than D-128, which began each message with its provider's id.
                error(
                    "claude",
                    "claude: request failed: Could not reach Anthropic.",
                ),
            ],
            status_pages: Vec::new(),
            costs: Vec::new(),
            left_out: Vec::new(),
            problems: vec!["The history could not be saved: the disk is full.".into()],
            providers: [("claude", "Claude"), ("gemini", "Gemini CLI")]
                .map(|(id, name)| quotadial_core::ProviderName {
                    id: id.into(),
                    name: name.into(),
                })
                .to_vec(),
        };

        let meter = Meter::new(Some(&report), None, None, now());

        let shown = [
            "Gemini CLI: could not read the sign-in from ~/.gemini/settings.json: it is not a \
             plain file",
            "copilot: request failed: GitHub did not answer.",
            "Claude: request failed: Could not reach Anthropic.",
            "The history could not be saved: the disk is full.",
        ];
        assert_eq!(
            meter.problems, shown,
            "a provider the report does not list keeps its id; QuotaDial's own problems follow"
        );
        assert_eq!(
            meter.tooltip(),
            format!("QuotaDial\n{}", shown.join("\n")),
            "the tray's menu and tooltip say the same"
        );
    }

    #[test]
    fn an_account_waiting_for_a_sign_in_does_not_set_the_tray_number() {
        let mut expired = snapshot(
            "claude",
            Some("max"),
            vec![window("Weekly", 98.0, "2099-01-04T00:04:00Z")],
        );
        expired.status = Status::AuthRequired;
        expired.account.sign_in = Some(
            "the macOS Keychain item \"Claude Code-credentials-1df4ed49\" (CLAUDE_CONFIG_DIR \
             ~/.claude-work)"
                .into(),
        );
        expired.account.sign_in_use = "QuotaDial reads this account's sign-in from the macOS \
                                       Keychain item \"Claude Code-credentials-1df4ed49\" \
                                       (CLAUDE_CONFIG_DIR ~/.claude-work). It only reads it and \
                                       never changes it."
            .into();
        let report = StatusReport {
            snapshots: vec![
                expired,
                snapshot(
                    "codex",
                    Some("pro"),
                    vec![window("Weekly", 40.0, "2099-01-07T03:04:00Z")],
                ),
            ],
            errors: Vec::new(),
            status_pages: Vec::new(),
            costs: Vec::new(),
            left_out: Vec::new(),
            problems: Vec::new(),
            providers: Vec::new(),
        };

        let meter = Meter::new(Some(&report), None, None, now());

        assert_eq!(
            meter.percent,
            Some(40),
            "the expired account's 98% is too old"
        );
        assert_eq!(
            meter.accounts[0].windows.len(),
            1,
            "but the popover still shows it"
        );
        assert_eq!(
            meter.accounts[0].sign_in,
            "QuotaDial reads this account's sign-in from the macOS Keychain item \
             \"Claude Code-credentials-1df4ed49\" (CLAUDE_CONFIG_DIR ~/.claude-work). It only \
             reads it and never changes it."
        );
    }

    #[test]
    fn a_name_the_user_gave_replaces_the_email_address() {
        let meter = Meter::new(Some(&two_accounts()), None, None, now()).with_aliases(
            &BTreeMap::from([("codex user-1 -".to_owned(), "Personal".to_owned())]),
        );

        assert_eq!(meter.accounts[1].alias.as_deref(), Some("Personal"));
        assert_eq!(meter.accounts[1].name, "user@example.com");
        assert_eq!(
            meter.lines()[1],
            "Codex Personal (pro): Weekly 73%, resets in 6 d 3 h"
        );
        assert_eq!(meter.accounts[0].alias, None);
    }

    #[test]
    fn names_the_user_gave_set_the_order_within_a_provider() {
        let named = |account_id: &str, display_name: &str| {
            let mut snapshot = snapshot(
                "claude",
                Some("max"),
                vec![window("Weekly", 10.0, "2099-01-04T00:04:00Z")],
            );
            snapshot.account.account_id = account_id.into();
            snapshot.account.display_name = display_name.into();
            snapshot
        };
        let mut report = two_accounts();
        report.snapshots = vec![
            named("user-m", "m@example.com"),
            named("user-z", "z@example.com"),
            report.snapshots[1].clone(),
        ];

        let meter = Meter::new(Some(&report), None, None, now()).with_aliases(&BTreeMap::from([(
            "claude user-z -".to_owned(),
            "Home".to_owned(),
        )]));

        let order: Vec<(&str, &str)> = meter
            .accounts
            .iter()
            .map(|account| {
                (
                    account.provider.as_str(),
                    account.alias.as_deref().unwrap_or(&account.name),
                )
            })
            .collect();
        assert_eq!(
            order,
            [
                ("Claude", "Home"),
                ("Claude", "m@example.com"),
                ("Codex", "user@example.com"),
            ]
        );
    }

    #[test]
    fn before_the_first_report_the_tray_says_it_is_looking() {
        let starting = Meter::new(None, None, None, now());
        assert_eq!(starting.title(), "–");
        assert_eq!(
            starting.tooltip(),
            "QuotaDial\nLooking for the accounts on this machine…"
        );

        let failed = Meter::new(None, Some("could not use the data folder"), None, now());
        assert_eq!(failed.problems, ["could not use the data folder"]);
    }

    #[test]
    fn the_popover_says_where_the_numbers_come_from() {
        let running = Meter::new(None, None, Some(Role::Running { port: 47_115 }), now());
        assert_eq!(
            running.origin.as_deref(),
            Some("Running the daemon, port 47115")
        );

        let following = Meter::new(None, None, Some(Role::Following), now());
        assert_eq!(
            following.origin.as_deref(),
            Some("Reading `quotadial daemon`")
        );
    }

    #[test]
    fn what_free_leaves_out_is_a_line_each_under_the_accounts() {
        let report = StatusReport {
            left_out: vec![
                quotadial_core::LeftOut::accounts("claude", "Claude", 8),
                quotadial_core::LeftOut::feature(quotadial_core::ProFeature::CostScan),
            ],
            ..StatusReport::default()
        };

        let meter = Meter::new(Some(&report), None, None, chrono::Utc::now());

        assert_eq!(
            meter.left_out,
            [
                "8 more Claude accounts are on this machine. Showing every account is part of \
                 QuotaDial Pro: quotadial.com/pro",
                "The week's cost at API prices is part of QuotaDial Pro: quotadial.com/pro",
            ]
        );
    }
}
