//! Claude usage provider for QuotaDial.
//!
//! Stub until M1 issue #1. It finds no accounts and reports every account as unsupported rather
//! than guessing at numbers. The port follows CodexBar's OAuth fetcher; the files named in the
//! TODOs below are under `Sources/CodexBarCore/` in `../codexbar-ref`.

use std::time::Duration;

use async_trait::async_trait;
use quotadial_core::{Account, Provider, Result, Snapshot};

/// The provider id used in [`Account::provider`].
pub const ID: &str = "claude";

/// Anthropic rate-limits its usage endpoint aggressively, so QuotaDial polls it at most once every
/// five minutes per account.
pub const MIN_POLL_INTERVAL: Duration = Duration::from_secs(5 * 60);

/// Reads Claude usage with the login Claude Code already stores.
#[derive(Debug, Default)]
pub struct ClaudeProvider;

impl ClaudeProvider {
    /// Creates the provider.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Provider for ClaudeProvider {
    fn id(&self) -> &'static str {
        ID
    }

    fn discover_accounts(&self) -> Result<Vec<Account>> {
        // TODO(M1 #1): read Claude Code's stored login (`~/.claude/.credentials.json`, the macOS
        // Keychain item "Claude Code-credentials", `$CLAUDE_CONFIG_DIR`) without writing it. Port:
        //   Providers/Claude/ClaudeOAuth/ClaudeOAuthCredentials.swift
        //   Providers/Claude/ClaudeConfigPaths.swift
        Ok(Vec::new())
    }

    async fn fetch(&self, account: &Account) -> Result<Snapshot> {
        // TODO(M1 #1): GET https://api.anthropic.com/api/oauth/usage, map five_hour, seven_day and
        // the per-model buckets to windows, and back off on 429. Port:
        //   Providers/Claude/ClaudeOAuth/ClaudeOAuthUsageFetcher.swift
        //   Providers/Claude/ClaudeOAuth/ClaudeOAuthUsageRateLimitGate.swift
        Ok(Snapshot::unsupported(
            account.clone(),
            "QuotaDial cannot read Claude usage yet; Claude support arrives in milestone M1.",
        ))
    }

    fn fallback(&self, _account: &Account) -> Option<Snapshot> {
        // TODO(M1 #1): local fallback from Claude Code's session logs under `~/.claude/projects`
        // (PRD, "Provider data sources").
        None
    }

    fn min_poll_interval(&self) -> Duration {
        MIN_POLL_INTERVAL
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quotadial_core::Status;
    use serde_json::Value;

    /// A synthetic `GET /api/oauth/usage` response: CodexBar's field names (`OAuthUsageResponse` in
    /// `ClaudeOAuthUsageFetcher.swift`) with obviously fake values. M1 replaces it with a redacted
    /// live capture.
    const USAGE_FIXTURE: &str =
        include_str!("../../../../fixtures/claude/oauth-usage.synthetic.json");

    fn account() -> Account {
        Account {
            provider: ID.into(),
            account_id: "00000000-0000-0000-0000-000000000001".into(),
            display_name: "user@example.com".into(),
            plan: Some("max".into()),
            org: Some("00000000-0000-0000-0000-000000000002".into()),
        }
    }

    #[test]
    fn usage_fixture_has_the_fields_codexbar_decodes() {
        let usage: Value = serde_json::from_str(USAGE_FIXTURE).expect("fixture is valid JSON");

        for bucket in [
            "five_hour",
            "seven_day",
            "seven_day_opus",
            "seven_day_sonnet",
        ] {
            let window = &usage[bucket];
            assert!(
                window["utilization"].is_number(),
                "{bucket}.utilization is a number"
            );
            assert!(
                window["resets_at"].is_string() || window["resets_at"].is_null(),
                "{bucket}.resets_at is a timestamp or null"
            );
        }
        for field in [
            "is_enabled",
            "monthly_limit",
            "used_credits",
            "utilization",
            "currency",
        ] {
            assert!(
                usage["extra_usage"].get(field).is_some(),
                "extra_usage.{field} is present"
            );
        }
    }

    #[tokio::test]
    async fn stub_reports_unsupported_instead_of_guessing() {
        let provider = ClaudeProvider::new();

        assert!(
            provider
                .discover_accounts()
                .expect("discovery succeeds")
                .is_empty()
        );
        let snapshot = provider.fetch(&account()).await.expect("fetch succeeds");
        assert_eq!(snapshot.status, Status::Unsupported);
        assert!(snapshot.windows.is_empty());
        assert!(snapshot.error.is_some());
        assert!(provider.fallback(&account()).is_none());
    }

    #[test]
    fn never_polls_anthropic_more_than_once_every_five_minutes() {
        assert!(ClaudeProvider::new().min_poll_interval() >= Duration::from_secs(300));
    }
}
