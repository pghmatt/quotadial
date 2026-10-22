//! ChatGPT and Codex usage provider for QuotaDial.
//!
//! Stub until M1 issue #2. It finds no accounts and reports every account as unsupported rather
//! than guessing at numbers. The port follows CodexBar's OAuth fetcher; the files named in the
//! TODOs below are under `Sources/CodexBarCore/` in `../codexbar-ref`.

use std::time::Duration;

use async_trait::async_trait;
use quotadial_core::{Account, Provider, Result, Snapshot};

/// The provider id used in [`Account::provider`].
pub const ID: &str = "codex";

/// The fastest adaptive poll interval in the PRD: every two minutes while a CLI is active.
pub const MIN_POLL_INTERVAL: Duration = Duration::from_secs(2 * 60);

/// Reads ChatGPT and Codex usage with the login the Codex CLI already stores.
#[derive(Debug, Default)]
pub struct CodexProvider;

impl CodexProvider {
    /// Creates the provider.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Provider for CodexProvider {
    fn id(&self) -> &'static str {
        ID
    }

    fn discover_accounts(&self) -> Result<Vec<Account>> {
        // TODO(M1 #2): read `~/.codex/auth.json` (or `$CODEX_HOME/auth.json`) and the account id
        // from its JWT claims, without writing the file. Port:
        //   Providers/Codex/CodexOAuth/CodexOAuthCredentials.swift
        Ok(Vec::new())
    }

    async fn fetch(&self, account: &Account) -> Result<Snapshot> {
        // TODO(M1 #2): GET https://chatgpt.com/backend-api/wham/usage with ChatGPT-Account-Id and
        // classify windows by length, never by slot. Port:
        //   Providers/Codex/CodexOAuth/CodexOAuthUsageFetcher.swift
        //   Providers/Codex/CodexRateWindowNormalizer.swift
        Ok(Snapshot::unsupported(
            account.clone(),
            "QuotaDial cannot read ChatGPT or Codex usage yet; Codex support arrives in milestone M1.",
        ))
    }

    fn fallback(&self, _account: &Account) -> Option<Snapshot> {
        // TODO(M1 #2): the newest rate-limit event in `~/.codex/sessions` (PRD). CodexBar has no
        // session-log reader to port: its UsageFetcher.swift asks `codex app-server` over JSON-RPC
        // (`account/rateLimits/read`) instead, which the PRD lists as the other fallback.
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

    /// A synthetic `GET /backend-api/wham/usage` response: CodexBar's field names
    /// (`CodexUsageResponse` in `CodexOAuthUsageFetcher.swift`) with obviously fake values. M1
    /// replaces it with a redacted live capture.
    const USAGE_FIXTURE: &str =
        include_str!("../../../../fixtures/codex/wham-usage.synthetic.json");

    fn account() -> Account {
        Account {
            provider: ID.into(),
            account_id: "00000000-0000-0000-0000-000000000000".into(),
            display_name: "user@example.com".into(),
            plan: "plus".into(),
            org: "00000000-0000-0000-0000-000000000003".into(),
        }
    }

    #[test]
    fn usage_fixture_has_the_fields_codexbar_decodes() {
        let usage: Value = serde_json::from_str(USAGE_FIXTURE).expect("fixture is valid JSON");

        assert!(usage["plan_type"].is_string(), "plan_type is a string");
        for slot in ["primary_window", "secondary_window"] {
            let window = &usage["rate_limit"][slot];
            assert!(
                window["used_percent"].is_number(),
                "{slot}.used_percent is a number"
            );
            assert!(
                window["reset_at"].is_u64(),
                "{slot}.reset_at is Unix seconds"
            );
            assert!(
                window["limit_window_seconds"].is_u64(),
                "{slot}.limit_window_seconds is a whole number of seconds"
            );
        }
        let credits = &usage["credits"];
        assert!(
            credits["has_credits"].is_boolean(),
            "credits.has_credits is a boolean"
        );
        assert!(
            credits["unlimited"].is_boolean(),
            "credits.unlimited is a boolean"
        );
        assert!(
            credits["balance"].is_string() || credits["balance"].is_number(),
            "credits.balance is a string or a number"
        );
    }

    #[tokio::test]
    async fn stub_reports_unsupported_instead_of_guessing() {
        let provider = CodexProvider::new();

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
    fn polls_no_faster_than_the_prd_minimum() {
        assert!(CodexProvider::new().min_poll_interval() >= Duration::from_secs(120));
    }
}
