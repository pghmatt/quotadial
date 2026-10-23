//! Every usage provider QuotaDial ships, in display order, set up from QuotaDial's settings. The
//! CLI and the tray app both start from this list, so a new provider is added in one place.

use std::path::PathBuf;
use std::sync::Arc;

use quotadial_core::Provider;
use quotadial_daemon::DataDir;
use quotadial_provider_claude::ClaudeProvider;
use quotadial_provider_codex::CodexProvider;
use serde::de::DeserializeOwned;
use serde_json::Value;

/// QuotaDial's settings for every process: `config.json` in its data folder. A missing or damaged
/// file means the defaults, and so does a setting of the wrong kind, setting by setting.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Config {
    /// More folders where Claude Code keeps sign-ins, as `CLAUDE_CONFIG_DIR` would name them
    /// (#34). `~` is the home folder, and a folder ending in `/*` adds every folder inside it.
    pub extra_config_dirs: Vec<String>,
    /// At most this many accounts per provider, the shell's own first, and at least 1. All when
    /// unset; M5 ties it to the free and Pro tiers.
    pub max_accounts: Option<usize>,
}

impl Config {
    /// The settings in `dir`, or the defaults.
    #[must_use]
    pub fn load(dir: &DataDir) -> Self {
        let Some(settings) = std::fs::read(dir.path().join("config.json"))
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
        else {
            return Self::default();
        };
        Self {
            extra_config_dirs: setting(&settings, "extra_config_dirs").unwrap_or_default(),
            max_accounts: setting(&settings, "max_accounts").filter(|max| *max >= 1),
        }
    }
}

/// The setting called `name`, if it is there and of the right kind.
fn setting<T: DeserializeOwned>(settings: &Value, name: &str) -> Option<T> {
    serde_json::from_value(settings.get(name)?.clone()).ok()
}

/// A new instance of every provider, in display order, set up from `config.json` in QuotaDial's
/// data folder.
#[must_use]
pub fn all() -> Vec<Arc<dyn Provider>> {
    with_config(
        &DataDir::from_env()
            .map(|dir| Config::load(&dir))
            .unwrap_or_default(),
    )
}

/// A new instance of every provider, in display order, set up from `config`.
#[must_use]
pub fn with_config(config: &Config) -> Vec<Arc<dyn Provider>> {
    vec![
        Arc::new(
            ClaudeProvider::new()
                .with_extra_config_dirs(
                    config.extra_config_dirs.iter().map(PathBuf::from).collect(),
                )
                .with_max_accounts(config.max_accounts),
        ),
        Arc::new(CodexProvider::new()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claude_comes_first_then_codex() {
        let ids: Vec<&str> = all().iter().map(|provider| provider.id()).collect();

        assert_eq!(ids, ["claude", "codex"]);
    }

    #[test]
    fn settings_come_from_config_json_with_defaults_for_the_rest() {
        let temp = tempfile::tempdir().expect("temp dir");
        let dir = DataDir::new(temp.path().to_path_buf());
        assert_eq!(Config::load(&dir), Config::default(), "no file yet");

        std::fs::write(
            temp.path().join("config.json"),
            r#"{"extra_config_dirs": ["~/claude-accounts/*"]}"#,
        )
        .expect("writes");
        assert_eq!(
            Config::load(&dir),
            Config {
                extra_config_dirs: vec!["~/claude-accounts/*".into()],
                max_accounts: None,
            }
        );

        std::fs::write(
            temp.path().join("config.json"),
            r#"{"extra_config_dirs": ["~/claude-accounts/*"], "max_accounts": "3"}"#,
        )
        .expect("writes");
        assert_eq!(
            Config::load(&dir),
            Config {
                extra_config_dirs: vec!["~/claude-accounts/*".into()],
                max_accounts: None,
            },
            "a setting of the wrong kind falls back alone"
        );

        std::fs::write(temp.path().join("config.json"), r#"{"max_accounts": 0}"#).expect("writes");
        assert_eq!(
            Config::load(&dir).max_accounts,
            None,
            "at least one account"
        );

        std::fs::write(temp.path().join("config.json"), "not json").expect("writes");
        assert_eq!(Config::load(&dir), Config::default(), "a damaged file");
    }
}
