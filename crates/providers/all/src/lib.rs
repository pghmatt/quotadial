//! Every usage provider QuotaDial ships, in display order, set up from QuotaDial's settings and
//! licence. The CLI and the tray app both start from this list, so a new provider is added in one
//! place.
//!
//! Each provider after Claude and Codex comes in behind a feature of its own (#73): `gemini`, on by
//! default, adds Gemini CLI after them (D-118, D-124). The example build (the `fake` feature, D-117)
//! lists the fake provider alone, whose made-up accounts are for pictures of the app: no sign-in is
//! read and no provider is asked, whatever else is on. No release has `fake`, and the tests below
//! hold each build's list.

use std::sync::Arc;

use quotadial_core::Provider;
use quotadial_daemon::{Config, DataDir, Licensing};

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
#[cfg(not(feature = "fake"))]
#[must_use]
pub fn with_config(config: &Config, licensing: &Licensing) -> Vec<Arc<dyn Provider>> {
    use std::path::PathBuf;

    use quotadial_provider_claude::ClaudeProvider;
    use quotadial_provider_codex::CodexProvider;

    vec![
        Arc::new(
            ClaudeProvider::new()
                .with_extra_config_dirs(
                    config.extra_config_dirs.iter().map(PathBuf::from).collect(),
                )
                .with_max_accounts(licensing.account_limit(config.max_accounts)),
        ),
        Arc::new(CodexProvider::new()),
        #[cfg(feature = "gemini")]
        Arc::new(quotadial_provider_gemini::GeminiProvider::new()),
    ]
}

/// The example build's list: the fake provider alone, in place of Claude and Codex. Its accounts
/// stand for what a first run finds, one per provider, so they all show whatever the licence, and
/// none of the settings for real sign-ins applies to them.
#[cfg(feature = "fake")]
#[must_use]
pub fn with_config(_config: &Config, _licensing: &Licensing) -> Vec<Arc<dyn Provider>> {
    vec![Arc::new(quotadial_provider_fake::FakeProvider::new())]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The ids and names of the providers listed, made with the default settings and no
    /// licence, so no folder is read.
    fn listed() -> Vec<(&'static str, &'static str)> {
        with_config(&Config::default(), &Licensing::new(None))
            .iter()
            .map(|provider| (provider.id(), provider.name()))
            .collect()
    }

    #[test]
    fn every_name_listed_is_a_short_name_in_plain_words() {
        for (id, name) in listed() {
            // Each provider's own short name for itself, never a sentence (P-020), and #52's plain
            // words: never "quota", "auth" or "credential".
            assert!(
                !name.is_empty() && name.len() <= 20 && !name.ends_with('.'),
                "{id}: {name}"
            );
            assert_eq!(
                quotadial_core::text::unplain_words(name),
                Vec::<String>::new(),
                "{id}: {name}"
            );
        }
    }

    /// What a release ships: Claude, Codex and, since it was turned on in the PR after the one
    /// that merged it, Gemini CLI (D-121, D-122, D-124).
    #[cfg(not(feature = "fake"))]
    #[test]
    fn a_release_lists_exactly_claude_codex_then_gemini() {
        assert_eq!(
            listed(),
            [
                ("claude", "Claude"),
                ("codex", "Codex"),
                ("gemini", "Gemini CLI")
            ]
        );
        assert_eq!(
            quotadial_core::copy::FOLDER,
            "quotadial",
            "and the copy keeps QuotaDial's own folders"
        );
    }

    #[cfg(feature = "fake")]
    #[test]
    fn with_the_fake_feature_the_list_is_exactly_the_fake_provider() {
        assert_eq!(listed(), [("fake", "Example")]);
        assert_eq!(
            quotadial_core::copy::FOLDER,
            "quotadial-fake",
            "and the copy keeps folders of its own"
        );
    }
}
