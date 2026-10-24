//! What the app shows. The tray gets the share used of the tightest window across every account
//! and how close that is to the limit; the popover gets every account and window, each number with
//! its reset, its source and its age.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use quotadial_core::{Account, Snapshot, Status, StatusReport, Window, text};
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
    /// What that number is, such as "Codex Weekly, resets in 6 d 1 h".
    pub headline: Option<String>,
    pub accounts: Vec<AccountView>,
    /// What needs the user's attention and belongs to no account, each naming the fix.
    pub problems: Vec<String>,
    /// The providers whose status pages report an incident, such as "Codex: Partial System
    /// Degradation" (D-096).
    pub incidents: Vec<String>,
    /// Where the numbers come from, such as "Running the daemon, port 47115".
    pub origin: Option<String>,
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
    /// The account, such as its email address.
    pub name: String,
    pub plan: Option<String>,
    /// Such as "ok", "stale" or "sign-in required".
    pub status: String,
    pub ok: bool,
    /// Tightest first.
    pub windows: Vec<WindowView>,
    /// Such as "on, 12.50 of 50.00 USD used (25%)".
    pub extra_usage: Option<String>,
    /// Why the status is not ok, naming the fix.
    pub note: Option<String>,
    /// Where QuotaDial reads this account's sign-in, for "How QuotaDial reads this".
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
    /// Such as "from the API 4 min ago".
    pub fetched: String,
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
            meter.headline = Some(format!(
                "{} {}, {}",
                text::provider_name(&snapshot.account.provider),
                window.label,
                text::resets(window.resets_at, now)
            ));
        }
        meter.accounts = report
            .snapshots
            .iter()
            .map(|snapshot| account_view(snapshot, now))
            .collect();
        meter
            .problems
            .extend(report.errors.iter().map(|error| error.message.clone()));
        meter.incidents = report
            .status_pages
            .iter()
            .filter(|page| page.indicator.is_incident())
            .map(|page| {
                format!(
                    "{}: {}",
                    text::provider_name(&page.provider),
                    page.description
                )
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
        lines
    }

    /// The meter with the names the user gave accounts in place of their email addresses. Each
    /// provider's accounts come sorted by email address in the report, and are sorted here by the
    /// name shown instead.
    #[must_use]
    pub fn with_aliases(mut self, aliases: &BTreeMap<String, String>) -> Self {
        for account in &mut self.accounts {
            if let Some(alias) = aliases.get(&account.key) {
                account.summary = account.summary.replacen(
                    &format!("{} {}", account.provider, account.name),
                    &format!("{} {alias}", account.provider),
                    1,
                );
                account.alias = Some(alias.clone());
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
        self
    }

    /// The tray icon's tooltip: the app's name, then [`Meter::lines`].
    #[must_use]
    pub fn tooltip(&self) -> String {
        let mut lines = vec!["QuotaDial".to_owned()];
        lines.extend(self.lines());
        if lines.len() == 1 {
            lines.push("Looking for the accounts on this machine…".to_owned());
        }
        lines.join("\n")
    }
}

fn account_view(snapshot: &Snapshot, now: DateTime<Utc>) -> AccountView {
    let account = &snapshot.account;
    let provider = text::provider_name(&account.provider).to_owned();
    let windows = snapshot.windows_tightest_first();
    let mut summary = format!("{provider} {}", account.display_name);
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
        sign_in: sign_in(account),
        provider,
        name: account.display_name.clone(),
        plan: account.plan.clone(),
        status: text::status_label(snapshot.status).to_owned(),
        ok: snapshot.status == Status::Ok,
        windows: windows
            .into_iter()
            .map(|window| window_view(window, now))
            .collect(),
        extra_usage: snapshot.extra_usage.as_ref().map(text::extra_usage),
        note: snapshot.error.clone(),
    }
}

fn window_view(window: &Window, now: DateTime<Utc>) -> WindowView {
    let used_pct = window.used_pct.clamp(0.0, 100.0);
    WindowView {
        label: window.label.clone(),
        used_pct,
        level: Level::of(used_pct),
        resets: text::resets(window.resets_at, now),
        fetched: format!(
            "from {} {}",
            text::source_label(window.source),
            text::ago(window.fetched_at, now)
        ),
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

/// Where QuotaDial reads the account's sign-in (P-015), or, if the provider did not say, where it
/// looks.
fn sign_in(account: &Account) -> String {
    if let Some(place) = &account.sign_in {
        return format!(
            "QuotaDial reads this account's sign-in from {place}. It only reads it and never \
             changes it."
        );
    }
    match account.provider.as_str() {
        "claude" => {
            let keychain = if cfg!(target_os = "macos") {
                "then the macOS Keychain item \"Claude Code-credentials\", "
            } else {
                ""
            };
            format!(
                "QuotaDial reads the sign-in Claude Code uses: the CLAUDE_CODE_OAUTH_TOKEN \
                 environment variable when it is set, {keychain}then .credentials.json in \
                 ~/.claude or the folder CLAUDE_CONFIG_DIR names. It only reads the sign-in and \
                 never changes it. If it stops working, run claude once."
            )
        }
        "codex" => "QuotaDial reads the sign-in the Codex CLI keeps in auth.json, in ~/.codex or \
                    the folder CODEX_HOME names. It only reads the sign-in and never changes it. \
                    If it stops working, run codex once."
            .to_owned(),
        _ => "QuotaDial reads this provider's own sign-in and never changes it.".to_owned(),
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

    fn snapshot(provider: &str, plan: Option<&str>, windows: Vec<Window>) -> Snapshot {
        Snapshot {
            account: Account {
                provider: provider.into(),
                account_id: "user-1".into(),
                display_name: "user@example.com".into(),
                plan: plan.map(str::to_owned),
                org: None,
                sign_in: None,
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
        }
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
            Some("Codex Weekly, resets in 6 d 3 h")
        );
        assert_eq!(
            meter.lines(),
            [
                "Claude user@example.com (max): Session 42%, resets in 2 h 10 min",
                "Codex user@example.com (pro): Weekly 73%, resets in 6 d 3 h",
            ]
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
        assert!(claude.sign_in.contains("CLAUDE_CODE_OAUTH_TOKEN"));
        assert_eq!(
            claude.windows,
            [
                WindowView {
                    label: "Session".into(),
                    used_pct: 42.6,
                    level: Level::Normal,
                    resets: "resets in 2 h 10 min".into(),
                    fetched: "from the API 4 min ago".into(),
                    unconfirmed: false,
                    pace: None,
                    runs_out: false,
                },
                WindowView {
                    label: "Weekly".into(),
                    used_pct: 12.0,
                    level: Level::Normal,
                    resets: "resets in 3 d".into(),
                    fetched: "from the API 4 min ago".into(),
                    unconfirmed: true,
                    pace: None,
                    runs_out: false,
                },
            ]
        );
        assert!(meter.accounts[1].sign_in.contains("auth.json"));
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
                message: "codex: could not read credentials from ~/.codex/auth.json".into(),
            }],
            status_pages: Vec::new(),
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
             again.\ncodex: could not read credentials from ~/.codex/auth.json"
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
}
