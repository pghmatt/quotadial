//! The tray app's settings: which providers it shows and the names given to accounts. They are kept
//! as JSON in the app's own config folder, readable only by the user. Whether QuotaDial opens at
//! login is the operating system's to keep, and which alerts are on is the daemon's history's
//! (D-093), so neither is stored here.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use std::collections::BTreeMap;

use quotadial_core::{Account, Condition, Scope, StatusReport};
use quotadial_daemon::History;
use serde::{Deserialize, Serialize};

/// The settings, with every provider shown until changed.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Providers left out of the tray, the popover and the notifications, by id.
    pub hidden_providers: Vec<String>,
    /// A name the user gave an account, by [`account_key`], shown in place of its email address
    /// (#34).
    pub aliases: BTreeMap<String, String>,
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

    /// `report` without the providers that are hidden.
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
        }
    }
}

/// Writes `bytes` to `path`, readable only by the user, through a temporary file and a rename.
pub(crate) fn write_private(path: &Path, bytes: &[u8]) -> io::Result<()> {
    if let Some(folder) = path.parent() {
        let mut builder = fs::DirBuilder::new();
        builder.recursive(true);
        #[cfg(unix)]
        std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
        builder.create(folder)?;
    }
    let temporary = path.with_extension("tmp");
    let mut options = OpenOptions::new();
    options.create(true).truncate(true).write(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    let mut file = options.open(&temporary)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    drop(file);
    fs::rename(temporary, path)
}

#[cfg(test)]
mod tests {
    use quotadial_core::{Account, ProviderError, Snapshot, Status};

    use super::*;

    #[test]
    fn settings_start_with_everything_shown_and_survive_a_restart() {
        let temp = tempfile::tempdir().expect("temp dir");
        let path = temp.path().join("config/settings.json");

        assert_eq!(Settings::load(&path), Settings::default());
        let changed = Settings {
            hidden_providers: vec!["codex".into()],
            aliases: BTreeMap::from([("claude user-1 -".into(), "Work".into())]),
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
    }

    #[test]
    fn a_notification_switched_off_in_m3_turns_its_rule_off_once() {
        let temp = tempfile::tempdir().expect("temp dir");
        let path = temp.path().join("settings.json");
        let history = History::open(&temp.path().join("history.db")).expect("opens");
        let custom = history
            .add_rule("warn me when codex passes 90%", chrono::Utc::now())
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
}
