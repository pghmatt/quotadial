//! What the tray shows: the share used of the tightest window across every account, how close
//! that is to the limit, and one line per account for the tooltip, the menu and the window.

use chrono::{DateTime, Utc};
use quotadial_core::{Snapshot, Status, StatusReport, text};
use serde::Serialize;

/// From this share used, the meter warns.
pub const WARNING_PCT: f64 = 70.0;
/// From this share used, the meter is high: the window is close to its limit.
pub const HIGH_PCT: f64 = 90.0;

/// How close the tightest window is to its limit.
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

/// Everything the tray and the window show.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Meter {
    /// The tightest window's share used, rounded down, if any account has numbers.
    pub percent: Option<u8>,
    pub level: Level,
    /// One line per account, in the report's order, such as
    /// "Claude user@example.com (max): Session 42%, resets in 2 h 10 min".
    pub accounts: Vec<String>,
    /// What needs the user's attention, each naming the fix.
    pub problems: Vec<String>,
}

impl Meter {
    /// The meter for `report`, or for no report yet with `problem` saying why.
    #[must_use]
    pub fn new(report: Option<&StatusReport>, problem: Option<&str>, now: DateTime<Utc>) -> Self {
        let mut meter = Self {
            percent: None,
            level: Level::Unknown,
            accounts: Vec::new(),
            problems: problem.map(str::to_owned).into_iter().collect(),
        };
        let Some(report) = report else {
            return meter;
        };
        let tightest = report
            .snapshots
            .iter()
            .flat_map(|snapshot| &snapshot.windows)
            .map(|window| window.used_pct)
            .max_by(f64::total_cmp);
        if let Some(used_pct) = tightest {
            let used_pct = used_pct.clamp(0.0, 100.0);
            meter.percent = Some(used_pct.floor() as u8);
            meter.level = if used_pct >= HIGH_PCT {
                Level::High
            } else if used_pct >= WARNING_PCT {
                Level::Warning
            } else {
                Level::Normal
            };
        }
        for snapshot in &report.snapshots {
            meter.accounts.push(account_line(snapshot, now));
            if let Some(error) = snapshot.error.as_ref().filter(|_| needs_user(snapshot)) {
                meter.problems.push(error.clone());
            }
        }
        meter
            .problems
            .extend(report.errors.iter().map(|error| error.message.clone()));
        meter
    }

    /// The text beside the tray icon, such as "42%", or "–" before there are numbers.
    #[must_use]
    pub fn title(&self) -> String {
        self.percent
            .map_or_else(|| "–".to_owned(), |percent| format!("{percent}%"))
    }

    /// The tray icon's tooltip: the app's name, then every account and problem.
    #[must_use]
    pub fn tooltip(&self) -> String {
        let mut lines = vec!["QuotaDial".to_owned()];
        lines.extend(self.accounts.iter().cloned());
        lines.extend(self.problems.iter().cloned());
        if lines.len() == 1 {
            lines.push("Looking for the accounts on this machine…".to_owned());
        }
        lines.join("\n")
    }
}

/// Whether a snapshot's message asks the user to act, as a sign-in that expired does.
fn needs_user(snapshot: &Snapshot) -> bool {
    matches!(snapshot.status, Status::AuthRequired | Status::Unsupported)
}

/// For example "Claude user@example.com (max): Session 42%, resets in 2 h 10 min".
fn account_line(snapshot: &Snapshot, now: DateTime<Utc>) -> String {
    let account = &snapshot.account;
    let mut line = format!(
        "{} {}",
        provider_name(&account.provider),
        account.display_name
    );
    if let Some(plan) = &account.plan {
        line.push_str(&format!(" ({plan})"));
    }
    match snapshot.windows_tightest_first().first() {
        Some(window) => line.push_str(&format!(
            ": {} {:.0}%, {}",
            window.label,
            window.used_pct.floor(),
            text::resets(window.resets_at, now)
        )),
        None => line.push_str(&format!(": {}", text::status_label(snapshot.status))),
    }
    line
}

/// The name people know a provider by.
fn provider_name(id: &str) -> &str {
    match id {
        "claude" => "Claude",
        "codex" => "Codex",
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use quotadial_core::{Account, ProviderError, Source, Window, WindowKind};

    use super::*;

    fn at(text: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(text)
            .expect("valid timestamp")
            .with_timezone(&Utc)
    }

    fn now() -> DateTime<Utc> {
        at("2099-01-01T00:00:00Z")
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
            fetched_at: now(),
            unconfirmed: false,
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
            },
            windows,
            extra_usage: None,
            status: Status::Ok,
            error: None,
        }
    }

    #[test]
    fn the_tray_shows_the_tightest_window_across_every_account() {
        let report = StatusReport {
            snapshots: vec![
                snapshot(
                    "claude",
                    Some("max"),
                    vec![
                        window("Session", 42.6, "2099-01-01T02:10:00Z"),
                        window("Weekly", 12.0, "2099-01-04T00:00:00Z"),
                    ],
                ),
                snapshot(
                    "codex",
                    Some("pro"),
                    vec![window("Weekly", 63.9, "2099-01-07T03:00:00Z")],
                ),
            ],
            errors: Vec::new(),
        };

        let meter = Meter::new(Some(&report), None, now());

        assert_eq!(meter.percent, Some(63));
        assert_eq!(meter.level, Level::Normal);
        assert_eq!(meter.title(), "63%");
        assert_eq!(
            meter.accounts,
            [
                "Claude user@example.com (max): Session 42%, resets in 2 h 10 min",
                "Codex user@example.com (pro): Weekly 63%, resets in 6 d 3 h",
            ]
        );
        assert!(meter.problems.is_empty());
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
            };
            Meter::new(Some(&report), None, now()).level
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
        };

        let meter = Meter::new(Some(&report), None, now());

        assert_eq!(meter.percent, None);
        assert_eq!(meter.level, Level::Unknown);
        assert_eq!(meter.title(), "–");
        assert_eq!(
            meter.accounts,
            ["Claude user@example.com (max): sign-in required"]
        );
        assert_eq!(
            meter.problems,
            [
                "Run `claude` to sign in again.",
                "codex: could not read credentials from ~/.codex/auth.json",
            ]
        );
        assert_eq!(
            meter.tooltip(),
            "QuotaDial\nClaude user@example.com (max): sign-in required\nRun `claude` to sign in \
             again.\ncodex: could not read credentials from ~/.codex/auth.json"
        );
    }

    #[test]
    fn before_the_first_report_the_tray_says_it_is_looking() {
        let starting = Meter::new(None, None, now());
        assert_eq!(starting.title(), "–");
        assert_eq!(
            starting.tooltip(),
            "QuotaDial\nLooking for the accounts on this machine…"
        );

        let failed = Meter::new(None, Some("could not use the data folder"), now());
        assert_eq!(failed.problems, ["could not use the data folder"]);
    }
}
