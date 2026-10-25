//! The tray app's settings: which providers it shows, the names given to accounts, and whether it
//! checks for updates. They are kept as JSON in the app's own config folder, readable only by the
//! user. Whether QuotaDial opens at login is the operating system's to keep, and which alerts are
//! on is the daemon's history's (D-093), so neither is stored here.

use std::fs;
use std::io;
use std::path::Path;

use std::collections::BTreeMap;

use chrono::NaiveDate;
use quotadial_core::{Account, Condition, ProFeature, Scope, Standing, StatusReport, Tier};
use quotadial_daemon::{DataDir, History, licence, write_private};
use serde::{Deserialize, Serialize};

/// The settings, with every provider shown and the daily update check on until changed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Providers left out of the tray, the popover and the notifications, by id.
    pub hidden_providers: Vec<String>,
    /// A name the user gave an account, by [`account_key`], shown in place of its email address
    /// (#34).
    pub aliases: BTreeMap<String, String>,
    /// Whether the app asks GitHub once a day for a newer version (D-103). Checking by hand works
    /// either way.
    pub check_updates: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            hidden_providers: Vec::new(),
            aliases: BTreeMap::new(),
            check_updates: true,
        }
    }
}

/// The licence, as Settings shows it (D-099, D-100).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LicenceView {
    /// One sentence saying whether the copy is QuotaDial Free or Pro, and why a key gives nothing.
    pub text: String,
    /// Whether a key is kept, which Settings then offers to remove.
    pub kept: bool,
    /// On QuotaDial Free, what Pro adds, one feature each; empty on Pro.
    pub adds: Vec<String>,
}

/// What the copy keeping its key in `dir` is on `today`. With no data folder there is no key.
#[must_use]
pub fn licence_view(dir: Option<&DataDir>, today: NaiveDate) -> LicenceView {
    let (standing, kept) = match dir {
        Some(dir) => (licence::standing(dir, today), dir.licence().exists()),
        None => (Standing::of(None, today), false),
    };
    let adds = if standing.tier == Tier::Free {
        ProFeature::ALL
            .iter()
            .map(|feature| feature.what().to_owned())
            .collect()
    } else {
        Vec::new()
    };
    LicenceView {
        text: standing.text,
        kept,
        adds,
    }
}

/// An account, as the settings name it: its provider, its id and its organization, the three that
/// make an account (D-024).
#[must_use]
pub fn account_key(account: &Account) -> String {
    key(
        &account.provider,
        &account.account_id,
        account.org.as_deref(),
    )
}

/// [`account_key`] from the three parts, as a firing carries them.
#[must_use]
pub fn key(provider: &str, account_id: &str, org: Option<&str>) -> String {
    format!("{provider} {account_id} {}", org.unwrap_or("-"))
}

/// The two notifications M3 kept as switches in the settings file. They are rules now (D-093):
/// a switch that was off turns its rule off, once, and the file is then saved without it. Returns
/// whether the file had the switches.
pub fn carry_over(path: &Path, history: &History) -> bool {
    #[derive(Deserialize)]
    struct Switches {
        notify_sign_in: Option<bool>,
        notify_high: Option<bool>,
    }
    let Some(switches) = fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Switches>(&bytes).ok())
    else {
        return false;
    };
    let off = [
        (switches.notify_high, Condition::Threshold { pct: 90.0 }),
        (switches.notify_sign_in, Condition::SignIn),
    ];
    for (_, condition) in off.iter().filter(|(on, _)| *on == Some(false)) {
        for rule in history.rules().unwrap_or_default() {
            if rule.rule.condition == *condition && rule.rule.scope == Scope::default() {
                let _ = history.set_rule_enabled(rule.id, false);
            }
        }
    }
    switches.notify_high.is_some() || switches.notify_sign_in.is_some()
}

impl Settings {
    /// The settings saved at `path`, or the defaults when there are none or they cannot be read.
    #[must_use]
    pub fn load(path: &Path) -> Self {
        fs::read(path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    /// Saves the settings at `path`, privately, through a temporary file so a crash never leaves
    /// half of them.
    ///
    /// # Errors
    ///
    /// When the folder or the file cannot be written.
    pub fn save(&self, path: &Path) -> io::Result<()> {
        write_private(
            path,
            &serde_json::to_vec_pretty(self).map_err(io::Error::other)?,
        )
    }

    /// Whether `provider` is shown.
    #[must_use]
    pub fn shows(&self, provider: &str) -> bool {
        !self
            .hidden_providers
            .iter()
            .any(|hidden| hidden == provider)
    }

    /// `report` without the providers that are hidden, their accounts, errors, status pages, costs
    /// and what QuotaDial Free leaves out of them.
    #[must_use]
    pub fn shown(&self, report: &StatusReport) -> StatusReport {
        StatusReport {
            snapshots: report
                .snapshots
                .iter()
                .filter(|snapshot| self.shows(&snapshot.account.provider))
                .cloned()
                .collect(),
            errors: report
                .errors
                .iter()
                .filter(|error| self.shows(&error.provider))
                .cloned()
                .collect(),
            status_pages: report
                .status_pages
                .iter()
                .filter(|page| self.shows(&page.provider))
                .cloned()
                .collect(),
            costs: report
                .costs
                .iter()
                .filter(|cost| self.shows(&cost.provider))
                .cloned()
                .collect(),
            left_out: report
                .left_out
                .iter()
                .filter(|left_out| left_out.provider.as_deref().is_none_or(|of| self.shows(of)))
                .cloned()
                .collect(),
            // QuotaDial's own problems belong to no provider.
            problems: report.problems.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use quotadial_core::{Account, ProviderError, Snapshot, Status};

    use super::*;

    #[test]
    fn the_licence_view_says_what_the_copy_is_and_whether_a_key_is_kept() {
        use quotadial_core::licence::TEST_SEED;
        use quotadial_core::{Licence, Plan};

        let temp = tempfile::tempdir().expect("temp dir");
        let dir = DataDir::new(temp.path().join("quotadial"));
        let today: NaiveDate = "2026-11-01".parse().expect("a date");

        let free = licence_view(Some(&dir), today);
        assert_eq!(free.text, "QuotaDial Free, with no licence key.");
        assert!(!free.kept);
        assert_eq!(
            free.adds.len(),
            ProFeature::ALL.len(),
            "what Pro adds, on Free"
        );
        assert_eq!(free.adds[4], "the week's cost at API prices");
        let key = Licence {
            id: "app-1".into(),
            plan: Plan::Pro,
            issued: "2026-10-24".parse().expect("a date"),
            until: None,
        }
        .sign(&TEST_SEED);
        licence::add(&dir, &key, today).expect("adds");
        assert_eq!(
            licence_view(Some(&dir), today),
            LicenceView {
                text: "QuotaDial Pro, licence app-1.".into(),
                kept: true,
                adds: Vec::new(),
            }
        );
        fs::write(dir.licence(), "QD1.damaged.key").expect("writes");
        let damaged = licence_view(Some(&dir), today);
        assert!(damaged.kept);
        assert!(
            damaged.text.contains("doesn't check out"),
            "{}",
            damaged.text
        );
        assert_eq!(
            licence_view(None, today).text,
            "QuotaDial Free, with no licence key."
        );
    }

    #[test]
    fn settings_start_with_everything_shown_and_survive_a_restart() {
        let temp = tempfile::tempdir().expect("temp dir");
        let path = temp.path().join("config/settings.json");

        assert_eq!(Settings::load(&path), Settings::default());
        assert!(
            Settings::default().check_updates,
            "the daily check starts on (D-103)"
        );
        let changed = Settings {
            hidden_providers: vec!["codex".into()],
            aliases: BTreeMap::from([("claude user-1 -".into(), "Work".into())]),
            check_updates: false,
        };
        changed.save(&path).expect("saves");

        assert_eq!(Settings::load(&path), changed);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(&path).expect("metadata").permissions().mode();
            assert_eq!(mode & 0o777, 0o600);
        }
    }

    #[test]
    fn damaged_or_partial_settings_fall_back_to_the_defaults() {
        let temp = tempfile::tempdir().expect("temp dir");
        let path = temp.path().join("settings.json");

        fs::write(&path, "not json").expect("writes");
        assert_eq!(Settings::load(&path), Settings::default());

        fs::write(
            &path,
            r#"{"hidden_providers": ["codex"], "notify_high": false}"#,
        )
        .expect("writes");
        assert_eq!(
            Settings::load(&path),
            Settings {
                hidden_providers: vec!["codex".into()],
                ..Settings::default()
            },
            "a setting saved by an older version keeps the rest at their defaults"
        );
        assert!(
            Settings::load(&path).check_updates,
            "settings from before updates check for them"
        );
    }

    #[test]
    fn a_notification_switched_off_in_m3_turns_its_rule_off_once() {
        let temp = tempfile::tempdir().expect("temp dir");
        let path = temp.path().join("settings.json");
        let history = History::open(&temp.path().join("history.db")).expect("opens");
        let custom = history
            .add_rule(
                "warn me when codex passes 90%",
                chrono::Utc::now(),
                Tier::Pro,
            )
            .expect("adds");
        fs::write(
            &path,
            r#"{"notify_sign_in": true, "notify_high": false, "hidden_providers": []}"#,
        )
        .expect("writes");

        assert!(carry_over(&path, &history), "the file had the switches");

        let on: Vec<(String, bool)> = history
            .rules()
            .expect("reads")
            .into_iter()
            .map(|rule| (rule.written, rule.enabled))
            .collect();
        assert_eq!(
            on,
            [
                ("warn me when a window passes 90%".to_owned(), false),
                (
                    "tell me when an account needs to sign in again".to_owned(),
                    true
                ),
                (custom.written, true),
            ],
            "only the starting rule for the switch that was off"
        );
        Settings::load(&path).save(&path).expect("saves");
        assert!(!carry_over(&path, &history), "once");
    }

    #[test]
    fn hidden_providers_leave_the_report() {
        let snapshot = |provider: &str| {
            Snapshot::auth_required(
                Account {
                    provider: provider.into(),
                    account_id: "user-1".into(),
                    display_name: "user@example.com".into(),
                    plan: None,
                    org: None,
                    sign_in: None,
                },
                "Sign in again.",
            )
        };
        let report = StatusReport {
            snapshots: vec![snapshot("claude"), snapshot("codex")],
            errors: vec![ProviderError {
                provider: "codex".into(),
                message: "codex: request failed".into(),
            }],
            status_pages: Vec::new(),
            costs: Vec::new(),
            left_out: Vec::new(),
            problems: Vec::new(),
        };
        let settings = Settings {
            hidden_providers: vec!["codex".into()],
            ..Settings::default()
        };

        let shown = settings.shown(&report);

        assert_eq!(shown.snapshots.len(), 1);
        assert_eq!(shown.snapshots[0].account.provider, "claude");
        assert_eq!(shown.snapshots[0].status, Status::AuthRequired);
        assert!(shown.errors.is_empty());
        assert!(settings.shows("claude"));
        assert!(!settings.shows("codex"));
    }

    #[test]
    fn a_hidden_providers_status_page_cost_and_held_back_accounts_leave_the_report_too() {
        let page = |provider: &str| quotadial_core::StatusPage {
            provider: provider.into(),
            indicator: quotadial_core::Indicator::Minor,
            description: "Partial System Degradation".into(),
            url: format!("https://status.{provider}.example"),
            since: None,
            checked_at: chrono::DateTime::<chrono::Utc>::MIN_UTC,
        };
        let cost = |provider: &str| quotadial_core::Cost {
            provider: provider.into(),
            since: chrono::DateTime::<chrono::Utc>::MIN_UTC,
            until: chrono::DateTime::<chrono::Utc>::MIN_UTC,
            usd: 12.5,
            requests: 3,
            tokens: quotadial_core::Tokens::default(),
            models: Vec::new(),
            prices_updated: "2026-10-23".into(),
            notes: Vec::new(),
        };
        let report = StatusReport {
            snapshots: Vec::new(),
            errors: Vec::new(),
            status_pages: vec![page("claude"), page("codex")],
            costs: vec![cost("claude"), cost("codex")],
            left_out: vec![
                quotadial_core::LeftOut::accounts("claude", 2),
                quotadial_core::LeftOut::accounts("codex", 1),
                quotadial_core::LeftOut::feature(ProFeature::CostScan),
            ],
            problems: Vec::new(),
        };
        let settings = Settings {
            hidden_providers: vec!["codex".into()],
            ..Settings::default()
        };

        let shown = settings.shown(&report);
        assert_eq!(shown.status_pages, [page("claude")]);
        assert_eq!(shown.costs, [cost("claude")]);
        assert_eq!(
            shown.left_out,
            [
                quotadial_core::LeftOut::accounts("claude", 2),
                quotadial_core::LeftOut::feature(ProFeature::CostScan),
            ],
            "a hidden provider's held-back accounts go too; what belongs to no provider stays"
        );
    }

    proptest::proptest! {
        #[test]
        fn no_settings_file_makes_loading_panic(bytes in quotadial_core::testing::line()) {
            let temp = tempfile::tempdir().expect("temp dir");
            let path = temp.path().join("settings.json");
            std::fs::write(&path, &bytes).expect("writes");
            let _ = Settings::load(&path);
        }
    }
}
