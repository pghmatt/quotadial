//! The tray app's settings: which notifications it sends and which providers it shows. They are
//! kept as JSON in the app's own config folder, readable only by the user. Whether QuotaDial opens
//! at login is the operating system's to keep, so it is not stored here.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use std::collections::BTreeMap;

use quotadial_core::{Account, StatusReport};
use serde::{Deserialize, Serialize};

/// The settings, with every notification on and every provider shown until changed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Notify when an account needs signing in again.
    pub notify_sign_in: bool,
    /// Notify when a window passes 90%.
    pub notify_high: bool,
    /// Providers left out of the tray, the popover and the notifications, by id.
    pub hidden_providers: Vec<String>,
    /// A name the user gave an account, by [`account_key`], shown in place of its email address
    /// (#34).
    pub aliases: BTreeMap<String, String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            notify_sign_in: true,
            notify_high: true,
            hidden_providers: Vec::new(),
            aliases: BTreeMap::new(),
        }
    }
}

/// An account, as the settings and the record of notifications name it: its provider, its id and
/// its organization, the three that make an account (D-024).
#[must_use]
pub fn account_key(account: &Account) -> String {
    format!(
        "{} {} {}",
        account.provider,
        account.account_id,
        account.org.as_deref().unwrap_or("-")
    )
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
    fn settings_start_with_everything_on_and_survive_a_restart() {
        let temp = tempfile::tempdir().expect("temp dir");
        let path = temp.path().join("config/settings.json");

        assert_eq!(Settings::load(&path), Settings::default());
        let changed = Settings {
            notify_sign_in: false,
            notify_high: true,
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

        fs::write(&path, r#"{"notify_high": false}"#).expect("writes");
        assert_eq!(
            Settings::load(&path),
            Settings {
                notify_high: false,
                ..Settings::default()
            },
            "a setting saved by an older version keeps the rest at their defaults"
        );
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
