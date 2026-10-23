//! Every usage provider QuotaDial ships, in display order. The CLI and the tray app both start
//! from this list, so a new provider is added in one place.

use std::sync::Arc;

use quotadial_core::Provider;
use quotadial_provider_claude::ClaudeProvider;
use quotadial_provider_codex::CodexProvider;

/// A new instance of every provider, in display order.
#[must_use]
pub fn all() -> Vec<Arc<dyn Provider>> {
    vec![
        Arc::new(ClaudeProvider::new()),
        Arc::new(CodexProvider::new()),
    ]
}

#[cfg(test)]
mod tests {
    #[test]
    fn claude_comes_first_then_codex() {
        let ids: Vec<&str> = super::all().iter().map(|provider| provider.id()).collect();

        assert_eq!(ids, ["claude", "codex"]);
    }
}
