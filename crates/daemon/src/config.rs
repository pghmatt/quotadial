//! QuotaDial's settings for every process: `config.json` in its data folder. A missing or damaged
//! file means the defaults, and so does a setting of the wrong kind, setting by setting.
//!
//! The webhooks alerts are sent to (D-094) are addresses that work like passwords: a Slack
//! address holds its channel's key. They are kept as [`Secret`]s, and QuotaDial only ever shows a
//! webhook's host. A file that holds one and that other users can read gets a warning.

use std::path::Path;

use quotadial_core::{ProFeature, Secret, tier};
use reqwest::Url;
use serde::de::DeserializeOwned;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::{DataDir, KEEP_DAYS, MAX_KEEP_DAYS};

/// QuotaDial's settings.
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    /// More folders where Claude Code keeps sign-ins, as `CLAUDE_CONFIG_DIR` would name them
    /// (#34). `~` is the home folder, and a folder ending in `/*` adds every folder inside it.
    pub extra_config_dirs: Vec<String>,
    /// At most this many accounts per provider, the shell's own first, and at least 1. All when
    /// unset; M5 ties it to the free and Pro tiers.
    pub max_accounts: Option<usize>,
    /// Where each alert is sent besides the app's notifications: `alert_webhooks` as JSON, and
    /// `slack_webhook` as a Slack message.
    pub webhooks: Vec<Webhook>,
    /// How many days of history to keep: 30 unless `history_days` says otherwise, up to 365.
    /// Keeping more than 30 is a Pro feature (D-090).
    pub history_days: u32,
    /// What the settings could not be used for, each naming the fix, such as a webhook that is not
    /// an `https` address. Never the address itself.
    pub warnings: Vec<String>,
}

/// A webhook to send alerts to.
#[derive(Debug, Clone)]
pub struct Webhook {
    url: Secret,
    host: String,
    /// What the webhook expects.
    pub kind: WebhookKind,
}

/// What a webhook expects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebhookKind {
    /// The alert's fields as JSON.
    Json,
    /// A Slack incoming webhook's `{"text": ...}`.
    Slack,
}

impl Webhook {
    /// Reads `address` as a webhook of `kind`: an `https` address, or an `http` one on this machine
    /// for a local receiver. `None` when it is neither.
    #[must_use]
    pub fn new(address: &str, kind: WebhookKind) -> Option<Self> {
        let url = Url::parse(address.trim()).ok()?;
        let host = url.host_str()?.to_owned();
        let local = matches!(host.as_str(), "localhost" | "127.0.0.1" | "[::1]");
        match url.scheme() {
            "https" => {}
            "http" if local => {}
            _ => return None,
        }
        Some(Self {
            url: Secret::new(url.as_str()),
            host,
            kind,
        })
    }

    /// The address, for the one place it is used: the request.
    #[must_use]
    pub fn url(&self) -> &str {
        self.url.expose()
    }

    /// The webhook's host, which is all QuotaDial shows of it, such as `hooks.slack.com`.
    #[must_use]
    pub fn host(&self) -> &str {
        &self.host
    }

    /// A name for the webhook that tells it from others without holding its address: the start of
    /// the address's SHA-256, kept beside each alert sent to it.
    #[must_use]
    pub fn key(&self) -> String {
        Sha256::digest(self.url().as_bytes())
            .iter()
            .take(8)
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }
}

impl PartialEq for Webhook {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.url() == other.url()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            extra_config_dirs: Vec::new(),
            max_accounts: None,
            webhooks: Vec::new(),
            history_days: KEEP_DAYS,
            warnings: Vec::new(),
        }
    }
}

impl Config {
    /// The settings in `dir`, or the defaults.
    #[must_use]
    pub fn load(dir: &DataDir) -> Self {
        let path = dir.path().join("config.json");
        let Some(settings) = std::fs::read(&path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
        else {
            return Self::default();
        };
        let mut config = Self {
            extra_config_dirs: setting(&settings, "extra_config_dirs").unwrap_or_default(),
            max_accounts: setting(&settings, "max_accounts").filter(|max| *max >= 1),
            ..Self::default()
        };
        if let Some(days) = setting::<u32>(&settings, "history_days") {
            if !(1..=MAX_KEEP_DAYS).contains(&days) {
                config.warnings.push(format!(
                    "history_days in {} must be from 1 to {MAX_KEEP_DAYS}, so QuotaDial keeps \
                     {KEEP_DAYS} days.",
                    path.display()
                ));
            } else if days > KEEP_DAYS && !tier().allows(ProFeature::LongHistory) {
                config.warnings.push(format!(
                    "Keeping more than {KEEP_DAYS} days of history needs QuotaDial Pro, so it \
                     keeps {KEEP_DAYS}."
                ));
            } else {
                config.history_days = days;
            }
        }
        let addresses = setting::<Vec<String>>(&settings, "alert_webhooks")
            .unwrap_or_default()
            .into_iter()
            .map(|address| (address, WebhookKind::Json))
            .chain(
                setting::<String>(&settings, "slack_webhook")
                    .map(|address| (address, WebhookKind::Slack)),
            );
        for (address, kind) in addresses {
            match Webhook::new(&address, kind) {
                Some(webhook) => config.webhooks.push(webhook),
                None => config.warnings.push(format!(
                    "A webhook in {} is not an https address, so no alerts are sent to it; \
                     correct it there.",
                    path.display()
                )),
            }
        }
        if !config.webhooks.is_empty() && readable_by_others(&path) {
            config.warnings.push(format!(
                "{} holds webhook addresses, which work like passwords, and other users can read \
                 it; run `chmod 600 {}`.",
                path.display(),
                path.display()
            ));
        }
        config
    }
}

/// The setting called `name`, if it is there and of the right kind.
fn setting<T: DeserializeOwned>(settings: &Value, name: &str) -> Option<T> {
    serde_json::from_value(settings.get(name)?.clone()).ok()
}

/// Whether users other than the owner can read the file at `path`.
fn readable_by_others(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(path).is_ok_and(|metadata| metadata.permissions().mode() & 0o077 != 0)
    }
    // Windows keeps a user's profile folder, where the data folder is, to that user.
    #[cfg(not(unix))]
    {
        let _ = path;
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn written(json: &str) -> (tempfile::TempDir, DataDir) {
        let temp = tempfile::tempdir().expect("temp dir");
        let path = temp.path().join("config.json");
        std::fs::write(&path, json).expect("writes");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))
                .expect("private");
        }
        let dir = DataDir::new(temp.path().to_path_buf());
        (temp, dir)
    }

    #[test]
    fn settings_come_from_config_json_with_defaults_for_the_rest() {
        let temp = tempfile::tempdir().expect("temp dir");
        assert_eq!(
            Config::load(&DataDir::new(temp.path().to_path_buf())),
            Config::default(),
            "no file yet"
        );

        let (_temp, dir) = written(r#"{"extra_config_dirs": ["~/claude-accounts/*"]}"#);
        assert_eq!(
            Config::load(&dir),
            Config {
                extra_config_dirs: vec!["~/claude-accounts/*".into()],
                ..Config::default()
            }
        );

        let (_temp, dir) =
            written(r#"{"extra_config_dirs": ["~/claude-accounts/*"], "max_accounts": "3"}"#);
        assert_eq!(
            Config::load(&dir),
            Config {
                extra_config_dirs: vec!["~/claude-accounts/*".into()],
                ..Config::default()
            },
            "a setting of the wrong kind falls back alone"
        );

        let (_temp, dir) = written(r#"{"max_accounts": 0}"#);
        assert_eq!(
            Config::load(&dir).max_accounts,
            None,
            "at least one account"
        );

        let (_temp, dir) = written("not json");
        assert_eq!(Config::load(&dir), Config::default(), "a damaged file");
    }

    #[test]
    fn history_is_kept_30_days_unless_set_from_1_to_365() {
        assert_eq!(Config::default().history_days, 30);
        let (_temp, dir) = written(r#"{"history_days": 90}"#);
        assert_eq!(Config::load(&dir).history_days, 90);

        for wrong in ["0", "400"] {
            let (_temp, dir) = written(&format!(r#"{{"history_days": {wrong}}}"#));
            let config = Config::load(&dir);
            assert_eq!(config.history_days, 30, "{wrong}");
            assert_eq!(config.warnings.len(), 1, "{wrong}");
            assert!(config.warnings[0].starts_with("history_days in "));
            assert!(
                config.warnings[0].ends_with("must be from 1 to 365, so QuotaDial keeps 30 days.")
            );
        }
        let (_temp, dir) = written(r#"{"history_days": "ninety"}"#);
        assert_eq!(Config::load(&dir).history_days, 30, "the wrong kind");
    }

    #[test]
    fn webhooks_are_https_or_on_this_machine_and_never_shown() {
        let (_temp, dir) = written(
            r#"{
                "alert_webhooks": ["https://example.com/hooks/quota", "http://127.0.0.1:8080/in",
                                   "http://example.com/plain", "not an address"],
                "slack_webhook": "https://hooks.slack.com/services/T000/B000/XXXXXXXX"
            }"#,
        );

        let config = Config::load(&dir);

        let kept: Vec<(&str, WebhookKind)> = config
            .webhooks
            .iter()
            .map(|webhook| (webhook.host(), webhook.kind))
            .collect();
        assert_eq!(
            kept,
            [
                ("example.com", WebhookKind::Json),
                ("127.0.0.1", WebhookKind::Json),
                ("hooks.slack.com", WebhookKind::Slack),
            ]
        );
        assert_eq!(config.warnings.len(), 2, "{:?}", config.warnings);
        assert!(
            config.warnings[0].ends_with(
                "is not an https address, so no alerts are sent to it; correct it there."
            )
        );
        let shown = format!("{config:?}");
        assert!(!shown.contains("XXXXXXXX"), "{shown}");
        assert!(!shown.contains("/hooks/quota"), "{shown}");
        assert_ne!(config.webhooks[0].key(), config.webhooks[1].key());
        assert_eq!(config.webhooks[0].key().len(), 16);
    }

    #[cfg(unix)]
    #[test]
    fn a_file_with_webhooks_that_others_can_read_gets_a_warning() {
        use std::os::unix::fs::PermissionsExt;
        let (temp, dir) = written(r#"{"slack_webhook": "https://hooks.slack.com/services/T/B/X"}"#);
        let path = temp.path().join("config.json");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).expect("chmod");

        let warnings = Config::load(&dir).warnings;

        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].ends_with(&format!("run `chmod 600 {}`.", path.display())));
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).expect("chmod");
        assert!(Config::load(&dir).warnings.is_empty());
    }
}
