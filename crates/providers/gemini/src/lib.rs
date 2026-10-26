//! Gemini CLI for QuotaDial: sees that Gemini CLI is signed in on this machine, shows it as an
//! account, and says in one line why QuotaDial shows no numbers for it and where Gemini CLI shows
//! them (D-118).
//!
//! Google's terms do not let another tool use Gemini CLI's sign-in to reach Google's servers, and
//! the manager decided on #73 that QuotaDial does not, whatever a setting says. So this provider
//! never opens the sign-in and sends nothing; it has no HTTP client at all. It reads two things:
//! whether the file Gemini CLI keeps a Google sign-in in is there, and one field of Gemini CLI's
//! settings, how it signs in (`settings`). Everything else in the settings is skipped unread.
//!
//! Where these are, and which command shows the limits in Gemini CLI, come from Gemini CLI's own
//! source, google-gemini/gemini-cli at cd1b45c, read on 2026-10-25 (`environment`, `advice`).
//! CodexBar's Gemini provider (`Providers/Gemini/GeminiStatusProbe.swift`) reads the same field and
//! then the sign-in and Google's endpoints; only the field is followed here.

mod advice;
mod environment;
mod settings;

use std::fmt;
use std::time::Duration;

use async_trait::async_trait;
use quotadial_core::{Account, Error, Provider, Result, Snapshot};

use crate::environment::Environment;
use crate::settings::Method;

/// The provider id used in [`Account::provider`].
pub const ID: &str = "gemini";

/// What the provider calls itself (P-020).
pub const NAME: &str = "Gemini CLI";

/// How often the daemon may look at a Gemini CLI account again. Nothing is asked, so this only
/// keeps its history short; discovery, every minute, already sees a sign-in come or go.
pub const MIN_POLL_INTERVAL: Duration = Duration::from_secs(5 * 60);

/// Sees whether Gemini CLI is signed in, without reading its sign-in or asking anyone.
pub struct GeminiProvider {
    env: Environment,
}

impl GeminiProvider {
    /// Creates the provider for this process's environment.
    #[must_use]
    pub fn new() -> Self {
        Self {
            env: Environment::from_process(),
        }
    }

    /// How Gemini CLI on this machine is signed in now, as far as QuotaDial can see.
    fn signed_in(&self) -> Option<SignedIn> {
        let file_is_there = self.env.sign_in_file().is_some_and(|path| path.is_file());
        let method = self
            .env
            .settings_file()
            .and_then(|path| settings::read(&path));
        SignedIn::of(file_is_there, method)
    }
}

impl Default for GeminiProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for GeminiProvider {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("GeminiProvider")
            .finish_non_exhaustive()
    }
}

/// How Gemini CLI is signed in, as far as QuotaDial can see, and so the account it shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SignedIn {
    /// With a Google account: the sign-in file is there, and the settings name Google or nothing.
    Google,
    /// With a Gemini API key, as the settings name.
    ApiKey,
    /// With Vertex AI, as the settings name.
    VertexAi,
}

impl SignedIn {
    const ALL: [Self; 3] = [Self::Google, Self::ApiKey, Self::VertexAi];

    /// What the sign-in file being there and the way the settings name add up to: `None` when
    /// Gemini CLI is not signed in, or is signed in a way QuotaDial says nothing about. The settings
    /// win, since Gemini CLI uses the way they name and removes the file when it signs out or moves
    /// away from Google.
    fn of(file_is_there: bool, method: Option<Method>) -> Option<Self> {
        match method {
            Some(Method::ApiKey) => Some(Self::ApiKey),
            Some(Method::VertexAi) => Some(Self::VertexAi),
            Some(Method::Other) => None,
            Some(Method::Google) | None => file_is_there.then_some(Self::Google),
        }
    }

    /// The account's id, the same on every machine, since Gemini CLI has one sign-in at a time.
    fn account_id(self) -> &'static str {
        match self {
            Self::Google => "google",
            Self::ApiKey => "api-key",
            Self::VertexAi => "vertex-ai",
        }
    }

    /// The way of signing in an account id stands for.
    fn of_account_id(account_id: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|signed_in| signed_in.account_id() == account_id)
    }

    fn display_name(self) -> &'static str {
        match self {
            Self::Google => "Google account",
            Self::ApiKey => "Gemini API key",
            Self::VertexAi => "Vertex AI",
        }
    }

    /// What QuotaDial does with the sign-in, in the provider's words (P-020): for a Google account,
    /// that it only checks the sign-in is at `place`; otherwise that it reads the one field of the
    /// settings at `place` that names the way, and, in a sentence of its own so it cannot be read
    /// as part of what the field says, that it never reads the key (the manager's rule 2 on #73).
    fn sign_in_use(self, place: &str) -> String {
        let (way, never) = match self {
            Self::Google => {
                return format!(
                    "QuotaDial only checks that Gemini CLI's sign-in is at {place}. It never opens it."
                );
            }
            Self::ApiKey => ("a Gemini API key", "the key"),
            Self::VertexAi => ("Vertex AI", "a key or sign-in"),
        };
        format!(
            "QuotaDial reads one field of Gemini CLI's settings, in {place}, which says it uses \
             {way}. It never reads {never}."
        )
    }

    /// Why there are no numbers, in one line.
    fn reason(self) -> &'static str {
        match self {
            Self::Google => advice::GOOGLE,
            Self::ApiKey => advice::API_KEY,
            Self::VertexAi => advice::VERTEX_AI,
        }
    }
}

#[async_trait]
impl Provider for GeminiProvider {
    fn id(&self) -> &'static str {
        ID
    }

    fn name(&self) -> &'static str {
        NAME
    }

    fn asks(&self) -> Option<&'static str> {
        None
    }

    /// The one account Gemini CLI is signed in to, if any, named for how it signs in, with where
    /// QuotaDial saw it (P-015): the sign-in file for a Google account, the settings for the rest.
    fn discover_accounts(&self) -> Result<Vec<Account>> {
        let Some(signed_in) = self.signed_in() else {
            return Ok(Vec::new());
        };
        let place = match signed_in {
            SignedIn::Google => self.env.sign_in_file(),
            SignedIn::ApiKey | SignedIn::VertexAi => self.env.settings_file(),
        };
        let place = place.map(|path| quotadial_core::text::tilde(&path, self.env.home.as_deref()));
        Ok(vec![Account {
            provider: ID.to_owned(),
            provider_name: NAME.to_owned(),
            account_id: signed_in.account_id().to_owned(),
            display_name: signed_in.display_name().to_owned(),
            plan: None,
            org: None,
            sign_in_use: signed_in.sign_in_use(place.as_deref().unwrap_or_default()),
            sign_in: place,
        }])
    }

    /// No numbers, and the line saying why. Nothing is read or asked: discovery, every minute,
    /// keeps the account true to how Gemini CLI signs in, and the line is true of the account
    /// either way, so a sign-out never shows as a sign-in to fix.
    async fn fetch(&self, account: &Account) -> Result<Snapshot> {
        let signed_in = SignedIn::of_account_id(&account.account_id)
            .filter(|_| account.provider == ID)
            .ok_or_else(|| Error::InvalidResponse {
                provider: ID,
                reason: format!(
                    "{} is not one of the ways Gemini CLI signs in that QuotaDial shows",
                    quotadial_core::redact_secrets(&account.account_id)
                ),
            })?;
        Ok(Snapshot::unsupported(account.clone(), signed_in.reason()))
    }

    fn fallback(&self, _account: &Account) -> Option<Snapshot> {
        // There are no numbers to fall back on.
        None
    }

    fn min_poll_interval(&self) -> Duration {
        MIN_POLL_INTERVAL
    }
}

#[cfg(test)]
impl GeminiProvider {
    /// A provider that looks in `env` instead of this process's environment.
    fn in_env(env: Environment) -> Self {
        Self { env }
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use quotadial_core::{Status, collect};
    use serde_json::json;

    use super::*;

    #[test]
    fn its_name_and_every_sentence_it_says_of_a_sign_in_keep_to_plain_words() {
        let said = SignedIn::ALL.map(|signed_in| {
            signed_in.sign_in_use(match signed_in {
                SignedIn::Google => "~/.gemini/oauth_creds.json",
                SignedIn::ApiKey | SignedIn::VertexAi => "~/.gemini/settings.json",
            })
        });

        for words in said.iter().map(String::as_str).chain([NAME]) {
            // #52's plain words: never "quota", "auth" or "credential" outside a name.
            assert_eq!(
                quotadial_core::text::unplain_words(words),
                Vec::<String>::new(),
                "{words}"
            );
        }
    }

    /// A home folder for Gemini CLI, made up by the test.
    struct Home {
        dir: tempfile::TempDir,
    }

    impl Home {
        fn new() -> Self {
            Self {
                dir: tempfile::tempdir().expect("temp dir"),
            }
        }

        fn path(&self) -> &Path {
            self.dir.path()
        }

        fn gemini(&self) -> PathBuf {
            let gemini = self.path().join(".gemini");
            std::fs::create_dir_all(&gemini).expect("makes .gemini");
            gemini
        }

        /// Signs Gemini CLI in with Google, as far as QuotaDial can tell: the file is there. What it
        /// holds is made up, and QuotaDial never reads it.
        fn signed_in(self) -> Self {
            std::fs::write(
                self.gemini().join("oauth_creds.json"),
                SIGN_IN_FILE_CONTENTS,
            )
            .expect("writes the sign-in file");
            self
        }

        /// Writes Gemini CLI's settings.
        fn settings(self, text: &str) -> Self {
            std::fs::write(self.gemini().join("settings.json"), text).expect("writes settings");
            self
        }

        /// Settings that name `value` as the way Gemini CLI signs in.
        fn choosing(self, value: &str) -> Self {
            self.settings(&json!({ "security": { "auth": { "selectedType": value } } }).to_string())
        }

        /// The provider looking in this home, with no `GEMINI_CLI_HOME`.
        fn provider(&self) -> GeminiProvider {
            GeminiProvider::in_env(Environment {
                home: Some(self.path().to_path_buf()),
                gemini_cli_home: None,
            })
        }
    }

    /// What a test puts in the sign-in file: made up, in pieces so no scanner takes it for one.
    const SIGN_IN_FILE_CONTENTS: &str = concat!(
        "{ \"access_token\": \"ya29",
        ".MadeUpAccessTokenThatQuotaDialNeverReads0123\" }"
    );

    /// What one look finds: the accounts discovered, and each one's snapshot.
    async fn look(provider: &GeminiProvider) -> Vec<(Account, Snapshot)> {
        let mut found = Vec::new();
        for account in provider.discover_accounts().expect("discovery never fails") {
            let snapshot = provider.fetch(&account).await.expect("a snapshot");
            found.push((account, snapshot));
        }
        found
    }

    /// `parts` in the home folder, as people write it: `~/.gemini/oauth_creds.json` on macOS and
    /// Linux, with Windows' own separator after the `~/` there.
    fn in_home(parts: &[&str]) -> String {
        format!("~/{}", parts.iter().collect::<PathBuf>().display())
    }

    fn account(account_id: &str, display_name: &str, sign_in: &str, sign_in_use: &str) -> Account {
        Account {
            provider: "gemini".into(),
            provider_name: "Gemini CLI".into(),
            account_id: account_id.into(),
            display_name: display_name.into(),
            plan: None,
            org: None,
            sign_in: Some(sign_in.into()),
            sign_in_use: sign_in_use.into(),
        }
    }

    #[tokio::test]
    async fn not_signed_in_shows_no_account() {
        assert!(look(&Home::new().provider()).await.is_empty(), "no folder");
        assert!(
            look(&Home::new().choosing("oauth-personal").provider())
                .await
                .is_empty(),
            "Google chosen but no sign-in kept"
        );
        let home = Home::new();
        home.gemini();
        assert!(look(&home.provider()).await.is_empty(), "an empty folder");
    }

    #[tokio::test]
    async fn signed_in_with_google_shows_an_account_saying_why_there_are_no_numbers() {
        let home = Home::new().signed_in().choosing("oauth-personal");

        let found = look(&home.provider()).await;

        let place = in_home(&[".gemini", "oauth_creds.json"]);
        let expected = account(
            "google",
            "Google account",
            &place,
            &format!(
                "QuotaDial only checks that Gemini CLI's sign-in is at {place}. It never opens it."
            ),
        );
        assert_eq!(found.len(), 1);
        let (account, snapshot) = &found[0];
        assert_eq!(account, &expected);
        assert_eq!(account.provider_name, home.provider().name());
        assert_eq!(snapshot.account, expected);
        assert_eq!(snapshot.status, Status::Unsupported);
        assert!(snapshot.windows.is_empty());
        assert_eq!(snapshot.extra_usage, None);
        assert_eq!(snapshot.error.as_deref(), Some(advice::GOOGLE));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn the_sign_in_file_is_never_opened() {
        use std::os::unix::fs::PermissionsExt;

        let home = Home::new().signed_in().choosing("oauth-personal");
        let file = home.path().join(".gemini").join("oauth_creds.json");
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o000))
            .expect("takes away every permission");

        let found = look(&home.provider()).await;

        assert_eq!(found.len(), 1, "seen without being opened");
        assert_eq!(found[0].0.account_id, "google");
    }

    #[tokio::test]
    async fn gemini_cli_home_moves_where_it_looks() {
        let home = Home::new();
        let elsewhere = home.path().join("work");
        let gemini = elsewhere.join(".gemini");
        std::fs::create_dir_all(&gemini).expect("makes the folder");
        std::fs::write(gemini.join("oauth_creds.json"), SIGN_IN_FILE_CONTENTS).expect("writes");
        let provider = GeminiProvider::in_env(Environment {
            home: Some(home.path().to_path_buf()),
            gemini_cli_home: Some(elsewhere),
        });

        let found = look(&provider).await;

        assert_eq!(found.len(), 1);
        assert_eq!(
            found[0].0.sign_in.as_deref(),
            Some(in_home(&["work", ".gemini", "oauth_creds.json"]).as_str())
        );
    }

    #[tokio::test]
    async fn an_api_key_or_vertex_ai_shows_an_account_with_no_plan_limits() {
        for (value, account_id, name, reason, uses) in [
            (
                "gemini-api-key",
                "api-key",
                "Gemini API key",
                advice::API_KEY,
                "says it uses a Gemini API key. It never reads the key.",
            ),
            (
                "vertex-ai",
                "vertex-ai",
                "Vertex AI",
                advice::VERTEX_AI,
                "says it uses Vertex AI. It never reads a key or sign-in.",
            ),
        ] {
            let settings = in_home(&[".gemini", "settings.json"]);
            for home in [
                Home::new().choosing(value),
                Home::new().signed_in().choosing(value),
            ] {
                let found = look(&home.provider()).await;

                assert_eq!(found.len(), 1, "{value}");
                let (account, snapshot) = &found[0];
                assert_eq!(
                    account,
                    &self::account(
                        account_id,
                        name,
                        &settings,
                        &format!(
                            "QuotaDial reads one field of Gemini CLI's settings, in {settings}, \
                             which {uses}"
                        )
                    )
                );
                assert_eq!(snapshot.status, Status::Unsupported);
                assert!(snapshot.windows.is_empty());
                assert_eq!(snapshot.error.as_deref(), Some(reason));
            }
        }
    }

    #[tokio::test]
    async fn another_way_of_signing_in_shows_no_account() {
        for value in [
            "compute-default-credentials",
            "cloud-shell",
            "gateway",
            "a-new-way",
        ] {
            let home = Home::new().signed_in().choosing(value);

            assert!(look(&home.provider()).await.is_empty(), "{value}");
        }
    }

    #[tokio::test]
    async fn missing_settings_go_by_the_sign_in_file_alone() {
        let found = look(&Home::new().signed_in().provider()).await;
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].0.account_id, "google");

        let home = Home::new().settings("{}");
        assert!(look(&home.provider()).await.is_empty());
    }

    #[tokio::test]
    async fn unreadable_settings_go_by_the_sign_in_file_alone() {
        let folder = |home: Home| {
            std::fs::create_dir_all(home.gemini().join("settings.json")).expect("a folder");
            home
        };
        for (what, signed_in, unsigned) in [
            (
                "not JSON",
                Home::new().signed_in().settings("{ not JSON"),
                Home::new().settings("{ not JSON"),
            ),
            (
                "a trailing comma",
                Home::new()
                    .signed_in()
                    .settings(r#"{ "security": { "auth": { "selectedType": "vertex-ai", } } }"#),
                Home::new()
                    .settings(r#"{ "security": { "auth": { "selectedType": "vertex-ai", } } }"#),
            ),
            (
                "a folder",
                folder(Home::new().signed_in()),
                folder(Home::new()),
            ),
        ] {
            let found = look(&signed_in.provider()).await;
            assert_eq!(found.len(), 1, "{what}");
            assert_eq!(found[0].0.account_id, "google", "{what}");

            assert!(look(&unsigned.provider()).await.is_empty(), "{what}");
        }
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn settings_quotadial_may_not_read_go_by_the_sign_in_file_alone() {
        use std::os::unix::fs::PermissionsExt;

        let home = Home::new().signed_in().choosing("vertex-ai");
        let settings = home.path().join(".gemini").join("settings.json");
        std::fs::set_permissions(&settings, std::fs::Permissions::from_mode(0o000))
            .expect("takes away every permission");

        let found = look(&home.provider()).await;

        assert_eq!(found.len(), 1);
        assert_eq!(found[0].0.account_id, "google");
    }

    #[tokio::test]
    async fn nothing_else_in_the_settings_or_the_sign_in_reaches_an_account_or_a_snapshot() {
        let secret = concat!("sk-proj-", "MadeUpKeyInAnMcpServersEnvironment0123");
        let settings = json!({
            "mcpServers": { "tool": { "command": "tool", "env": { "TOOL_KEY": secret } } },
            "security": { "auth": { "selectedType": "oauth-personal" } },
            "general": { "preferredEditor": "vim" }
        })
        .to_string();
        let home = Home::new().signed_in().settings(&settings);

        let found = look(&home.provider()).await;

        let shown = serde_json::to_string(
            &found
                .iter()
                .map(|(_, snapshot)| snapshot)
                .collect::<Vec<_>>(),
        )
        .expect("serializes");
        assert_eq!(found.len(), 1);
        for never in [secret, "MadeUpAccessToken", "TOOL_KEY", "mcpServers", "vim"] {
            assert!(!shown.contains(never), "{never} reached {shown}");
        }
    }

    #[tokio::test]
    async fn every_account_it_shows_is_one_it_can_fetch_and_no_other() {
        let provider = Home::new().provider();
        let stranger = account(
            "someone-else",
            "Google account",
            "~/.gemini",
            "QuotaDial reads no one's sign-in.",
        );

        let error = provider
            .fetch(&stranger)
            .await
            .expect_err("not one of ours");

        assert_eq!(
            error.to_string(),
            "unexpected response: someone-else is not one of the ways Gemini CLI signs in \
             that QuotaDial shows"
        );
    }

    #[tokio::test]
    async fn collect_shows_the_account_and_no_error() {
        let home = Home::new().signed_in();
        let provider: std::sync::Arc<dyn Provider> = std::sync::Arc::new(home.provider());

        let report = collect(&[provider]).await;

        assert!(report.errors.is_empty(), "{:?}", report.errors);
        assert_eq!(report.snapshots.len(), 1);
        assert_eq!(report.snapshots[0].status, Status::Unsupported);
    }

    #[test]
    fn it_asks_no_one_so_it_has_no_logs_no_fallback_and_no_commands() {
        let home = Home::new().signed_in();
        let provider = home.provider();
        let account = provider.discover_accounts().expect("discovers")[0].clone();

        assert_eq!(provider.id(), "gemini");
        assert_eq!(provider.asks(), None);
        assert_eq!(provider.min_poll_interval(), Duration::from_secs(300));
        assert_eq!(provider.fallback(&account), None);
        assert!(provider.log_folders().is_empty());
        assert!(provider.commands().is_empty());
        assert_eq!(provider.held_back(), 0);
        assert!(provider.discovery_warnings().is_empty());
    }
}
