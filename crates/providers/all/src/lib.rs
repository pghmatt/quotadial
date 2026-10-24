//! Every usage provider QuotaDial ships, in display order, set up from QuotaDial's settings and
//! licence. The CLI and the tray app both start from this list, so a new provider is added in one
//! place.

use std::path::PathBuf;
use std::sync::Arc;

use quotadial_core::Provider;
use quotadial_daemon::{Config, DataDir, Licensing};
use quotadial_provider_claude::ClaudeProvider;
use quotadial_provider_codex::CodexProvider;

/// A new instance of every provider, in display order, set up from `config.json` and the licence
/// key in QuotaDial's data folder.
#[must_use]
pub fn all() -> Vec<Arc<dyn Provider>> {
    let dir = DataDir::from_env();
    with_config(
        &dir.as_ref().map(Config::load).unwrap_or_default(),
        &Licensing::new(dir.as_ref()),
    )
}

/// A new instance of every provider, in display order, set up from `config`. Each shows as many
/// accounts as `licensing` allows at each discovery: one per provider on QuotaDial Free (D-100).
#[must_use]
pub fn with_config(config: &Config, licensing: &Licensing) -> Vec<Arc<dyn Provider>> {
    vec![
        Arc::new(
            ClaudeProvider::new()
                .with_extra_config_dirs(
                    config.extra_config_dirs.iter().map(PathBuf::from).collect(),
                )
                .with_max_accounts(licensing.account_limit(config.max_accounts)),
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
