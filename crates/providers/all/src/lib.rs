//! Every usage provider QuotaDial ships, in display order, set up from QuotaDial's settings. The
//! CLI and the tray app both start from this list, so a new provider is added in one place.

use std::path::PathBuf;
use std::sync::Arc;

use quotadial_core::Provider;
use quotadial_daemon::{Config, DataDir};
use quotadial_provider_claude::ClaudeProvider;
use quotadial_provider_codex::CodexProvider;

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
}
