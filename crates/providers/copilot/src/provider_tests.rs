//! End-to-end tests of `CopilotProvider` against a local HTTP server standing in for GitHub, with a
//! temporary home folder for GitHub CLI's `hosts.yml`, a made-up Keychain for the Copilot CLI's
//! item, a manual clock and a temporary cache directory. Nothing here touches the real `~/.config/gh`,
//! the real Keychain, the network or the user's cache.

use std::fs;
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;
use std::time::SystemTime;

use chrono::{DateTime, TimeDelta, Utc};
use quotadial_core::{Source as WindowSource, WindowKind};
use serde_json::json;
use tempfile::TempDir;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::*;
use crate::advice::tests::gh_place;
use crate::keychain::Item;
use crate::usage::tests::{BUSINESS, FREE, PRO_CREDITS};

/// GitHub CLI's made-up token and the Copilot CLI's, in GitHub's OAuth shape, written in pieces so
/// no secret scanner takes this file for one holding a token.
const GH_TOKEN: &str = concat!("gho_", "MadeUpGitHubCliTokenForQuotaDial0001");
const CLI_TOKEN: &str = concat!("gho_", "MadeUpCopilotCliTokenForQuotaDial002");

/// The Copilot CLI's item's account attribute, in one of the forms QuotaDial reads.
const CLI_ACCOUNT: &str = "https://github.com:example-user";

struct ManualClock(Mutex<DateTime<Utc>>);

impl ManualClock {
    fn advance(&self, by: TimeDelta) {
        *self.0.lock().expect("clock lock") += by;
    }
}

impl Clock for ManualClock {
    fn now(&self) -> DateTime<Utc> {
        *self.0.lock().expect("clock lock")
    }
}

/// The Copilot CLI's Keychain item, made up, counting every look at it.
#[derive(Default)]
struct FakeKeychain {
    item: Mutex<Option<Item>>,
    secret: Mutex<Option<String>>,
    /// What a read gives in place of the secret.
    read_error: Mutex<Option<ReadError>>,
    /// What a search gives in place of the item.
    find_error: Mutex<Option<String>>,
    /// The item as the Copilot CLI writes it again while it is being read.
    rewritten_while_read: Mutex<Option<Item>>,
    finds: AtomicUsize,
    reads: AtomicUsize,
}

impl FakeKeychain {
    fn finds(&self) -> usize {
        self.finds.load(Ordering::SeqCst)
    }

    fn reads(&self) -> usize {
        self.reads.load(Ordering::SeqCst)
    }
}

impl Keychain for FakeKeychain {
    fn find(&self, service: &str) -> std::result::Result<Option<Item>, String> {
        assert_eq!(service, "copilot-cli");
        self.finds.fetch_add(1, Ordering::SeqCst);
        if let Some(error) = self.find_error.lock().expect("lock").clone() {
            return Err(error);
        }
        Ok(self.item.lock().expect("lock").clone())
    }

    fn read(
        &self,
        service: &str,
        account: Option<&str>,
    ) -> std::result::Result<Option<Secret>, ReadError> {
        assert_eq!(service, "copilot-cli");
        self.reads.fetch_add(1, Ordering::SeqCst);
        if let Some(error) = self.read_error.lock().expect("lock").clone() {
            return Err(error);
        }
        let mut item = self.item.lock().expect("lock");
        let Some(found) = item.clone() else {
            return Ok(None);
        };
        assert_eq!(account, found.account.as_deref(), "read by its own account");
        if let Some(rewritten) = self.rewritten_while_read.lock().expect("lock").take() {
            *item = Some(rewritten);
        }
        Ok(self.secret.lock().expect("lock").clone().map(Secret::new))
    }
}

struct Harness {
    server: MockServer,
    home: TempDir,
    cache: TempDir,
    clock: Arc<ManualClock>,
    keychain: Arc<FakeKeychain>,
}

impl Harness {
    async fn new() -> Self {
        Self {
            server: MockServer::start().await,
            home: tempfile::tempdir().expect("home"),
            cache: tempfile::tempdir().expect("cache dir"),
            clock: Arc::new(ManualClock(Mutex::new(ts("2026-01-15T12:00:00Z")))),
            keychain: Arc::new(FakeKeychain::default()),
        }
    }

    fn env(&self) -> Environment {
        Environment {
            home: Some(self.home.path().to_path_buf()),
            ..Environment::default()
        }
    }

    fn provider(&self) -> CopilotProvider {
        self.provider_with(&self.server.uri())
    }

    fn provider_with(&self, base_url: &str) -> CopilotProvider {
        CopilotProvider::for_tests(
            self.env(),
            Arc::clone(&self.keychain) as Arc<dyn Keychain>,
            Arc::clone(&self.clock) as Arc<dyn Clock>,
            self.cache.path().join("copilot"),
            base_url,
        )
    }

    fn hosts_file(&self) -> PathBuf {
        self.home
            .path()
            .join(".config")
            .join("gh")
            .join("hosts.yml")
    }

    /// Signs GitHub CLI in to `user`, its token in `hosts.yml` or, with none, in its keychain.
    fn sign_in_gh(&self, user: &str, token: Option<&str>) {
        let file = self.hosts_file();
        fs::create_dir_all(file.parent().expect("a folder")).expect("makes ~/.config/gh");
        fs::write(&file, crate::hosts::tests::hosts_yml(user, token)).expect("writes hosts.yml");
    }

    /// Signs the Copilot CLI in: its Keychain item, with `account` as its account attribute, last
    /// written at `modified`, holding `token`.
    fn sign_in_copilot_cli(&self, account: Option<&str>, token: &str, modified: &str) {
        *self.keychain.item.lock().expect("lock") = Some(Item {
            account: account.map(str::to_owned),
            modified: Some(modified.to_owned()),
        });
        *self.keychain.secret.lock().expect("lock") = Some(token.to_owned());
    }

    async fn mount(&self, response: ResponseTemplate) {
        Mock::given(method("GET"))
            .and(path("/copilot_internal/user"))
            .respond_with(response)
            .mount(&self.server)
            .await;
    }

    async fn requests(&self) -> usize {
        self.server.received_requests().await.map_or(0, |requests| {
            requests
                .iter()
                .filter(|request| request.url.path() == "/copilot_internal/user")
                .count()
        })
    }

    fn advance(&self, minutes: i64) {
        self.clock.advance(TimeDelta::minutes(minutes));
    }

    fn now(&self) -> DateTime<Utc> {
        self.clock.now()
    }

    /// Everything QuotaDial wrote to its cache directory, as text.
    fn cache_contents(&self) -> String {
        fs::read_dir(self.cache.path().join("copilot"))
            .map(|entries| {
                entries
                    .map(|entry| {
                        fs::read_to_string(entry.expect("entry").path()).unwrap_or_default()
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

fn ts(rfc3339: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(rfc3339)
        .expect("valid timestamp")
        .with_timezone(&Utc)
}

fn answer(fixture: &str) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_raw(fixture, "application/json")
}

/// What QuotaDial would print for a fetch.
fn printed(result: &Result<Snapshot>) -> String {
    match result {
        Ok(snapshot) => serde_json::to_string(snapshot).expect("serializes"),
        Err(error) => format!("{error} / {error:?}"),
    }
}

/// Fails the test if a token, or anything shaped like GitHub's, shows up in `output`.
fn assert_no_token(output: &str) {
    for token in [GH_TOKEN, CLI_TOKEN] {
        assert!(!output.contains(token), "a token leaked into: {output}");
    }
    assert!(!output.contains("gho_"), "a token leaked into: {output}");
    assert!(!output.contains("MadeUp"), "a token leaked into: {output}");
}

fn discovered(provider: &CopilotProvider) -> Vec<Account> {
    provider.discover_accounts().expect("discovers")
}

fn only(provider: &CopilotProvider) -> Account {
    let mut accounts = discovered(provider);
    assert_eq!(accounts.len(), 1, "{accounts:?}");
    accounts.remove(0)
}

/// Sets `path`'s modified time, as GitHub CLI writing it again would.
fn set_modified(path: &std::path::Path, at: DateTime<Utc>) {
    let file = fs::File::options().write(true).open(path).expect("opens");
    file.set_modified(SystemTime::from(at))
        .expect("sets the time");
}

#[tokio::test]
async fn github_cli_s_account_is_found_in_its_file_without_a_request() {
    let harness = Harness::new().await;
    harness.sign_in_gh("Example-User", Some(GH_TOKEN));

    let provider = harness.provider();
    let account = only(&provider);

    assert_eq!(
        account,
        Account {
            provider: "copilot".into(),
            provider_name: "Copilot".into(),
            account_id: "example-user".into(),
            display_name: "Example-User".into(),
            plan: None,
            org: None,
            sign_in: Some(gh_place()),
            sign_in_use: format!(
                "QuotaDial reads this account's sign-in from {}. It only reads it and never changes \
                 it.",
                gh_place()
            ),
        }
    );
    assert_eq!(account.provider_name, provider.name());
    assert_eq!(harness.requests().await, 0);
    assert_eq!(harness.keychain.reads(), 0);
}

#[tokio::test]
async fn a_github_cli_sign_in_kept_in_its_keychain_and_a_signed_out_machine_have_no_account() {
    let harness = Harness::new().await;
    assert!(discovered(&harness.provider()).is_empty(), "signed out");

    // GitHub CLI's own keychain is never read, so its account has nothing QuotaDial can use.
    harness.sign_in_gh("example-user", None);
    assert!(discovered(&harness.provider()).is_empty());
    assert_eq!(harness.requests().await, 0);
}

#[tokio::test]
async fn the_copilot_cli_s_account_is_found_by_its_item_s_attributes_alone() {
    let harness = Harness::new().await;
    harness.sign_in_copilot_cli(Some(CLI_ACCOUNT), CLI_TOKEN, "20260115100000Z");

    let account = only(&harness.provider());

    assert_eq!(account.account_id, "example-user");
    assert_eq!(account.display_name, "example-user");
    assert_eq!(
        account.sign_in.as_deref(),
        Some("the macOS Keychain item \"copilot-cli\"")
    );
    assert!(
        account.sign_in_use.contains(
            "macOS may ask you to allow that. If you deny it, QuotaDial does not ask again"
        ),
        "the prompt is said before any read: {}",
        account.sign_in_use
    );
    assert!(harness.keychain.finds() >= 1);
    assert_eq!(
        harness.keychain.reads(),
        0,
        "discovery never reads the secret"
    );
}

#[tokio::test]
async fn an_item_for_another_host_is_left_out_and_one_naming_no_login_is_one_account() {
    let harness = Harness::new().await;
    harness.sign_in_copilot_cli(
        Some("https://octocorp.ghe.com:example-user"),
        CLI_TOKEN,
        "20260115100000Z",
    );
    assert!(discovered(&harness.provider()).is_empty());

    harness.sign_in_copilot_cli(Some("not a login"), CLI_TOKEN, "20260115100000Z");
    let account = only(&harness.provider());
    assert_eq!(account.account_id, "copilot-cli");
    assert_eq!(account.display_name, "");
}

#[tokio::test]
async fn fetches_usage_with_github_s_headers_and_quotadial_s_own_name() {
    let harness = Harness::new().await;
    harness.sign_in_gh("example-user", Some(GH_TOKEN));
    Mock::given(method("GET"))
        .and(path("/copilot_internal/user"))
        .and(header(
            "authorization",
            format!("token {GH_TOKEN}").as_str(),
        ))
        .and(header("accept", "application/json"))
        .and(header("x-github-api-version", "2025-04-01"))
        .and(header("user-agent", api::USER_AGENT_VALUE))
        .respond_with(answer(PRO_CREDITS))
        .expect(1)
        .mount(&harness.server)
        .await;
    let provider = harness.provider();
    let account = only(&provider);

    let result = provider.fetch(&account).await;

    assert_no_token(&printed(&result));
    let snapshot = result.expect("fetches");
    assert_eq!(snapshot.status, Status::Ok);
    assert_eq!(snapshot.error, None);
    assert_eq!(snapshot.account.plan.as_deref(), Some("individual_pro"));
    assert!(snapshot.account.same_as(&account));
    assert_eq!(snapshot.windows.len(), 1);
    let credits = &snapshot.windows[0];
    assert_eq!(credits.kind, WindowKind::Credits);
    assert_eq!(credits.label, "AI credits");
    assert_eq!(credits.used_pct, 25.0);
    assert_eq!(credits.resets_at, Some(ts("2026-02-01T00:00:00Z")));
    assert_eq!(credits.window_seconds, 31 * 24 * 60 * 60);
    assert_eq!(credits.source, WindowSource::Api);
    assert_eq!(credits.fetched_at, ts("2026-01-15T12:00:00Z"));
    assert_eq!(
        snapshot.account.sign_in_use,
        format!(
            "QuotaDial reads this account's sign-in from {}. It only reads it and never changes it. \
             GitHub gives this account unlimited chat messages and code completions, so QuotaDial \
             has no share to show for them.",
            gh_place()
        )
    );
}

#[tokio::test]
async fn copilot_free_s_chat_and_completions_show_as_windows() {
    let harness = Harness::new().await;
    harness.sign_in_gh("example-user", Some(GH_TOKEN));
    harness.mount(answer(FREE)).await;
    let provider = harness.provider();

    let snapshot = provider.fetch(&only(&provider)).await.expect("fetches");

    let labels: Vec<(&str, f64)> = snapshot
        .windows
        .iter()
        .map(|window| (window.label.as_str(), window.used_pct))
        .collect();
    assert_eq!(
        labels,
        [("Chat messages", 20.0), ("Code completions", 25.0)]
    );
    assert_eq!(snapshot.status, Status::Ok);
}

#[tokio::test]
async fn a_seat_with_no_allowance_shows_no_meter_and_says_why() {
    let harness = Harness::new().await;
    harness.sign_in_gh("example-user", Some(GH_TOKEN));
    harness.mount(answer(BUSINESS)).await;
    let provider = harness.provider();

    let snapshot = provider.fetch(&only(&provider)).await.expect("fetches");

    assert_eq!(snapshot.status, Status::Unsupported);
    assert!(snapshot.windows.is_empty(), "never 0%");
    assert_eq!(
        snapshot.error.as_deref(),
        Some(
            "GitHub sets no allowance of AI credits for this account and gives it unlimited chat \
             messages and code completions, so QuotaDial has no share to show for them."
        )
    );
    assert_eq!(snapshot.account.plan.as_deref(), Some("business"));

    // An answer with nothing to show at all says so.
    harness.server.reset().await;
    harness.mount(answer("{}")).await;
    harness.advance(5);
    let empty = provider.fetch(&only(&provider)).await.expect("fetches");
    assert_eq!(empty.status, Status::Unsupported);
    assert_eq!(empty.error.as_deref(), Some(advice::NO_LIMITS));
}

#[tokio::test]
async fn the_copilot_cli_s_token_is_read_only_when_a_request_goes_out_at_most_once_a_floor() {
    let harness = Harness::new().await;
    harness.sign_in_copilot_cli(Some(CLI_ACCOUNT), CLI_TOKEN, "20260115100000Z");
    Mock::given(method("GET"))
        .and(path("/copilot_internal/user"))
        .and(header(
            "authorization",
            format!("token {CLI_TOKEN}").as_str(),
        ))
        .respond_with(answer(PRO_CREDITS))
        .mount(&harness.server)
        .await;
    let provider = harness.provider();
    let account = only(&provider);
    assert_eq!(harness.keychain.reads(), 0);

    let first = provider.fetch(&account).await.expect("fetches");
    assert_eq!(first.status, Status::Ok);
    assert_eq!((harness.keychain.reads(), harness.requests().await), (1, 1));

    harness.advance(4);
    let replayed = provider.fetch(&account).await.expect("replays");
    assert_eq!(replayed.windows[0].source, WindowSource::Cache);
    assert_eq!(
        (harness.keychain.reads(), harness.requests().await),
        (1, 1),
        "inside the floor, nothing is read and nothing is asked"
    );

    harness.advance(1);
    let fresh = provider.fetch(&account).await.expect("fetches");
    assert_eq!(fresh.windows[0].source, WindowSource::Api);
    assert_eq!((harness.keychain.reads(), harness.requests().await), (2, 2));
}

#[tokio::test]
async fn a_read_the_person_refuses_is_not_asked_for_again_until_the_item_changes() {
    let harness = Harness::new().await;
    harness.sign_in_copilot_cli(Some(CLI_ACCOUNT), CLI_TOKEN, "20260115100000Z");
    *harness.keychain.read_error.lock().expect("lock") = Some(ReadError::Refused);
    harness.mount(answer(PRO_CREDITS)).await;
    let provider = harness.provider();
    let account = only(&provider);

    let refused = provider.fetch(&account).await.expect("a snapshot");
    assert_eq!(
        refused.status,
        Status::Unsupported,
        "a no to keep, not a sign-in to fix"
    );
    assert_eq!(
        refused.error.as_deref(),
        Some(advice::refused("the macOS Keychain item \"copilot-cli\"").as_str())
    );
    assert_eq!((harness.keychain.reads(), harness.requests().await), (1, 0));

    // Long after the floor, the same item is not asked for again.
    harness.advance(60);
    let again = provider.fetch(&account).await.expect("a snapshot");
    assert_eq!(again.error, refused.error);
    assert_eq!(harness.keychain.reads(), 1, "macOS is not asked again");

    // The Copilot CLI signs in again: the item changed, so it is asked for once more.
    *harness.keychain.read_error.lock().expect("lock") = None;
    harness.sign_in_copilot_cli(Some(CLI_ACCOUNT), CLI_TOKEN, "20260115130000Z");
    let allowed = provider.fetch(&account).await.expect("fetches");
    assert_eq!(allowed.status, Status::Ok);
    assert_eq!((harness.keychain.reads(), harness.requests().await), (2, 1));
}

#[tokio::test]
async fn a_read_left_unanswered_says_to_answer_the_prompt_and_is_tried_at_the_next_check() {
    let harness = Harness::new().await;
    harness.sign_in_copilot_cli(Some(CLI_ACCOUNT), CLI_TOKEN, "20260115100000Z");
    *harness.keychain.read_error.lock().expect("lock") = Some(ReadError::Failed(
        "/usr/bin/security did not answer within 30 seconds; if macOS is asking whether to allow \
         access, answer it and try again"
            .into(),
    ));
    harness.mount(answer(PRO_CREDITS)).await;
    let provider = harness.provider();
    let account = only(&provider);

    let error = provider.fetch(&account).await.expect_err("no token");

    assert_eq!(
        error.to_string(),
        "could not read the sign-in from the macOS Keychain item \"copilot-cli\": \
         /usr/bin/security did not answer within 30 seconds; if macOS is asking whether to allow \
         access, answer it and try again"
    );
    assert_eq!(harness.requests().await, 0);
    *harness.keychain.read_error.lock().expect("lock") = None;
    assert_eq!(
        provider.fetch(&account).await.expect("fetches").status,
        Status::Ok,
        "no request went out, so the floor did not start"
    );
    assert_eq!(harness.keychain.reads(), 2);
}

#[tokio::test]
async fn one_account_in_both_is_read_from_github_cli_s_file_which_macos_never_asks_about() {
    let harness = Harness::new().await;
    harness.sign_in_copilot_cli(Some(CLI_ACCOUNT), CLI_TOKEN, "20260115100000Z");
    harness.sign_in_gh("Example-User", Some(GH_TOKEN));
    Mock::given(method("GET"))
        .and(path("/copilot_internal/user"))
        .and(header(
            "authorization",
            format!("token {GH_TOKEN}").as_str(),
        ))
        .respond_with(answer(PRO_CREDITS))
        .expect(1)
        .mount(&harness.server)
        .await;
    let provider = harness.provider();

    let account = only(&provider);
    assert_eq!(account.sign_in, Some(gh_place()));
    let snapshot = provider.fetch(&account).await.expect("fetches");

    assert_eq!(snapshot.status, Status::Ok);
    assert_eq!(harness.keychain.reads(), 0);
}

#[tokio::test]
async fn two_accounts_list_the_copilot_cli_s_first_and_free_shows_one() {
    let harness = Harness::new().await;
    harness.sign_in_copilot_cli(Some("cli-user"), CLI_TOKEN, "20260115100000Z");
    harness.sign_in_gh("gh-user", Some(GH_TOKEN));

    let all: Vec<String> = discovered(&harness.provider())
        .into_iter()
        .map(|account| account.account_id)
        .collect();
    assert_eq!(all, ["cli-user", "gh-user"]);

    let free = harness
        .provider()
        .with_max_accounts(AccountLimit::fixed(Some(1)));
    let shown: Vec<String> = discovered(&free)
        .into_iter()
        .map(|account| account.account_id)
        .collect();
    assert_eq!(shown, ["cli-user"]);
    assert_eq!(free.held_back(), 1);
}

#[tokio::test]
async fn a_rejected_sign_in_says_to_sign_in_again_and_a_new_one_is_tried_at_once() {
    let harness = Harness::new().await;
    harness.sign_in_gh("example-user", Some(GH_TOKEN));
    set_modified(&harness.hosts_file(), ts("2026-01-15T09:00:00Z"));
    harness
        .mount(ResponseTemplate::new(401).set_body_raw(
            include_str!("../../../../fixtures/copilot/unauthorized.synthetic.json"),
            "application/json",
        ))
        .await;
    let provider = harness.provider();
    let account = only(&provider);

    let rejected = provider.fetch(&account).await.expect("a snapshot");
    assert_eq!(rejected.status, Status::AuthRequired);
    assert_eq!(
        rejected.error.as_deref(),
        Some("GitHub rejected GitHub CLI's sign-in. Run `gh auth login` to sign in again.")
    );

    harness.advance(1);
    let again = provider.fetch(&account).await.expect("replays");
    assert_eq!(again.status, Status::AuthRequired);
    assert_eq!(harness.requests().await, 1);

    // `gh auth login` writes the file again, and the new sign-in is tried at once.
    harness.server.reset().await;
    harness.mount(answer(PRO_CREDITS)).await;
    harness.sign_in_gh("example-user", Some(GH_TOKEN));
    set_modified(&harness.hosts_file(), ts("2026-01-15T12:00:30Z"));
    let signed_in = provider.fetch(&account).await.expect("fetches");
    assert_eq!(signed_in.status, Status::Ok);
    assert_eq!(harness.requests().await, 1);
}

/// A first good answer, then `limited` for the next request five minutes later: what that shows,
/// and whether a request goes out a minute before `until` and at it.
async fn after_a_limit(limited: ResponseTemplate, until: DateTime<Utc>) -> Snapshot {
    let harness = Harness::new().await;
    harness.sign_in_gh("example-user", Some(GH_TOKEN));
    harness.mount(answer(PRO_CREDITS)).await;
    let provider = harness.provider();
    let account = only(&provider);
    provider.fetch(&account).await.expect("first fetch");

    harness.server.reset().await;
    harness.mount(limited).await;
    harness.advance(5);
    let shown = provider.fetch(&account).await.expect("a snapshot");
    assert_eq!(shown.status, Status::RateLimited, "{shown:?}");
    assert_eq!(
        shown.windows[0].source,
        WindowSource::Cache,
        "the last numbers stay"
    );
    assert_eq!(
        shown.error.as_deref(),
        Some(
            format!(
                "GitHub is rate-limiting usage requests, so QuotaDial will ask again after {}.",
                until.to_rfc3339_opts(SecondsFormat::Secs, true)
            )
            .as_str()
        )
    );

    harness.server.reset().await;
    harness.mount(answer(PRO_CREDITS)).await;
    harness
        .clock
        .advance(until - harness.now() - TimeDelta::minutes(1));
    let held = provider.fetch(&account).await.expect("replays");
    assert_eq!(held.status, Status::RateLimited);
    assert_eq!(harness.requests().await, 0, "nothing asked while held back");

    harness.advance(1);
    let back = provider.fetch(&account).await.expect("fetches");
    assert_eq!(back.status, Status::Ok);
    assert_eq!(harness.requests().await, 1);
    shown
}

#[tokio::test]
async fn a_429_with_retry_after_holds_requests_until_then() {
    after_a_limit(
        ResponseTemplate::new(429).insert_header("retry-after", "1200"),
        ts("2026-01-15T12:25:00Z"),
    )
    .await;
}

#[tokio::test]
async fn a_429_with_x_ratelimit_reset_holds_requests_until_the_reset() {
    let reset = ts("2026-01-15T12:45:00Z");
    after_a_limit(
        ResponseTemplate::new(429)
            .insert_header("x-ratelimit-remaining", "0")
            .insert_header("x-ratelimit-reset", reset.timestamp().to_string().as_str()),
        reset,
    )
    .await;
}

#[tokio::test]
async fn a_403_with_retry_after_holds_requests_until_then() {
    after_a_limit(
        ResponseTemplate::new(403)
            .insert_header("retry-after", "900")
            .set_body_json(json!({ "message": "You have exceeded a secondary rate limit." })),
        ts("2026-01-15T12:20:00Z"),
    )
    .await;
}

#[tokio::test]
async fn a_403_with_its_limit_spent_holds_requests_until_x_ratelimit_reset() {
    let reset = ts("2026-01-15T13:00:00Z");
    after_a_limit(
        ResponseTemplate::new(403)
            .insert_header("x-ratelimit-remaining", "0")
            .insert_header("x-ratelimit-reset", reset.timestamp().to_string().as_str())
            .set_body_json(json!({ "message": "API rate limit exceeded." })),
        reset,
    )
    .await;
}

#[tokio::test]
async fn a_limit_with_no_time_given_backs_off_by_the_floor_and_never_less() {
    // No header: QuotaDial's own backoff, 5 minutes for a first limit.
    after_a_limit(ResponseTemplate::new(429), ts("2026-01-15T12:10:00Z")).await;
    // A time sooner than the floor waits for the floor.
    after_a_limit(
        ResponseTemplate::new(429).insert_header("retry-after", "30"),
        ts("2026-01-15T12:10:00Z"),
    )
    .await;
}

#[tokio::test]
async fn a_403_with_requests_left_is_github_refusing_not_a_limit() {
    let harness = Harness::new().await;
    harness.sign_in_gh("example-user", Some(GH_TOKEN));
    harness
        .mount(
            ResponseTemplate::new(403)
                .insert_header("x-ratelimit-remaining", "4999")
                .insert_header("x-ratelimit-reset", "1768482000")
                .set_body_json(json!({ "message": "Resource not accessible by integration" })),
        )
        .await;
    let provider = harness.provider();

    let error = provider
        .fetch(&only(&provider))
        .await
        .expect_err("no numbers");

    assert_eq!(
        error.to_string(),
        "request failed: GitHub's Copilot usage endpoint answered HTTP 403: Resource not \
         accessible by integration. QuotaDial asks GitHub at most every 5 minutes, so the next \
         request is after 2026-01-15T12:05:00Z."
    );
}

#[tokio::test]
async fn signing_out_after_discovery_sends_no_request() {
    let harness = Harness::new().await;
    harness.sign_in_gh("example-user", Some(GH_TOKEN));
    harness.sign_in_copilot_cli(Some("cli-user"), CLI_TOKEN, "20260115100000Z");
    harness.mount(answer(PRO_CREDITS)).await;
    let provider = harness.provider();
    let accounts = discovered(&provider);
    assert_eq!(accounts.len(), 2);

    // GitHub CLI moves its token to its keychain, and the Copilot CLI signs out.
    harness.sign_in_gh("example-user", None);
    *harness.keychain.item.lock().expect("lock") = None;
    let cli = provider.fetch(&accounts[0]).await.expect("a snapshot");
    let gh = provider.fetch(&accounts[1]).await.expect("a snapshot");

    assert_eq!(cli.status, Status::AuthRequired);
    assert_eq!(
        cli.error.as_deref(),
        Some(
            "QuotaDial no longer finds the Copilot CLI's sign-in in the macOS Keychain item \
             \"copilot-cli\". Run `copilot` and sign in again with /login."
        )
    );
    assert_eq!(gh.status, Status::AuthRequired);
    assert_eq!(
        gh.error,
        Some(format!(
            "QuotaDial no longer finds GitHub CLI's sign-in in {}. Run `gh auth login` to sign in \
             again.",
            gh_place()
        ))
    );
    assert_eq!(harness.requests().await, 0);
    assert_eq!(harness.keychain.reads(), 0);
}

#[tokio::test]
async fn an_account_whose_sign_in_was_never_said_is_told_where_to_sign_in() {
    let harness = Harness::new().await;
    harness.mount(answer(PRO_CREDITS)).await;
    let account = Account {
        provider: "copilot".into(),
        provider_name: "Copilot".into(),
        account_id: "example-user".into(),
        display_name: "example-user".into(),
        plan: None,
        org: None,
        sign_in: None,
        sign_in_use: String::new(),
    };

    let snapshot = harness
        .provider()
        .fetch(&account)
        .await
        .expect("a snapshot");

    assert_eq!(snapshot.status, Status::AuthRequired);
    assert_eq!(snapshot.error.as_deref(), Some(advice::NOT_FOUND));
    assert_eq!(harness.requests().await, 0);
}

#[tokio::test]
async fn a_sign_in_now_for_another_account_is_never_read_for_this_one() {
    let harness = Harness::new().await;
    harness.sign_in_gh("example-user", Some(GH_TOKEN));
    harness.mount(answer(PRO_CREDITS)).await;
    let provider = harness.provider();
    let account = only(&provider);

    harness.sign_in_gh("someone-else", Some(GH_TOKEN));
    let snapshot = provider.fetch(&account).await.expect("a snapshot");

    assert_eq!(snapshot.status, Status::AuthRequired);
    assert_eq!(
        snapshot.error,
        Some(format!(
            "GitHub CLI's sign-in in {} is now for another GitHub account, so this one's usage \
             cannot be read there.",
            gh_place()
        ))
    );
    assert_eq!(harness.requests().await, 0);
}

#[tokio::test]
async fn an_item_rewritten_while_it_is_read_is_not_used() {
    let harness = Harness::new().await;
    harness.sign_in_copilot_cli(Some(CLI_ACCOUNT), CLI_TOKEN, "20260115100000Z");
    *harness.keychain.rewritten_while_read.lock().expect("lock") = Some(Item {
        account: Some(CLI_ACCOUNT.into()),
        modified: Some("20260115120000Z".into()),
    });
    harness.mount(answer(PRO_CREDITS)).await;
    let provider = harness.provider();

    let snapshot = provider.fetch(&only(&provider)).await.expect("a snapshot");

    assert_eq!(snapshot.status, Status::Stale);
    assert_eq!(
        snapshot.error.as_deref(),
        Some(advice::changed_while_read("the macOS Keychain item \"copilot-cli\"").as_str())
    );
    assert_eq!(harness.requests().await, 0);
}

#[tokio::test]
async fn what_cannot_be_looked_at_is_a_warning_beside_the_accounts_found() {
    let harness = Harness::new().await;
    harness.sign_in_copilot_cli(Some(CLI_ACCOUNT), CLI_TOKEN, "20260115100000Z");
    // A folder where GitHub CLI's file should be.
    fs::create_dir_all(harness.hosts_file()).expect("makes a folder");
    let provider = harness.provider();

    assert_eq!(
        discovered(&provider).len(),
        1,
        "the Copilot CLI's account still shows"
    );
    assert_eq!(
        provider.discovery_warnings(),
        [format!(
            "could not read GitHub CLI's sign-in in {}: it is not a plain file.",
            gh_place()
        )]
    );

    fs::remove_dir(harness.hosts_file()).expect("removes it");
    harness.sign_in_gh("gh-user", Some(GH_TOKEN));
    *harness.keychain.find_error.lock().expect("lock") =
        Some("/usr/bin/security exited with status 1".into());
    let gh_only: Vec<String> = discovered(&provider)
        .into_iter()
        .map(|account| account.account_id)
        .collect();
    assert_eq!(gh_only, ["gh-user"]);
    assert_eq!(
        provider.discovery_warnings(),
        [
            "could not look for the Copilot CLI's sign-in in the macOS Keychain: \
          /usr/bin/security exited with status 1."
        ]
    );
}

#[tokio::test]
async fn a_token_quoted_in_github_s_error_is_never_shown_or_saved() {
    let harness = Harness::new().await;
    harness.sign_in_gh("example-user", Some(GH_TOKEN));
    harness
        .mount(
            ResponseTemplate::new(404).set_body_json(
                json!({ "message": format!("Token {GH_TOKEN} is not a Copilot user") }),
            ),
        )
        .await;
    let provider = harness.provider();

    let failed = provider.fetch(&only(&provider)).await;

    assert_no_token(&printed(&failed));
    assert_eq!(
        failed.expect_err("fails").to_string(),
        "request failed: GitHub's Copilot usage endpoint answered HTTP 404: Token [redacted \
         token] is not a Copilot user. QuotaDial asks GitHub at most every 5 minutes, so the next \
         request is after 2026-01-15T12:05:00Z."
    );
    assert_no_token(&harness.cache_contents());
}

#[tokio::test]
async fn an_answer_for_another_account_is_never_used() {
    let harness = Harness::new().await;
    harness.sign_in_gh("example-user", Some(GH_TOKEN));
    let mut foreign: serde_json::Value = serde_json::from_str(PRO_CREDITS).expect("JSON");
    foreign["login"] = json!("someone-else");
    harness
        .mount(ResponseTemplate::new(200).set_body_json(foreign))
        .await;
    let provider = harness.provider();

    let result = provider.fetch(&only(&provider)).await;

    assert_eq!(
        result.expect_err("refused").to_string(),
        "unexpected response: GitHub's answer names another account than the sign-in, so it was \
         not used. QuotaDial asks GitHub at most every 5 minutes, so the next request is after \
         2026-01-15T12:05:00Z."
    );
    assert!(
        !harness.cache_contents().contains("used_pct"),
        "no numbers were saved"
    );

    // The same account, written in another case, is the same account.
    harness.server.reset().await;
    let mut same: serde_json::Value = serde_json::from_str(PRO_CREDITS).expect("JSON");
    same["login"] = json!("Example-User");
    harness
        .mount(ResponseTemplate::new(200).set_body_json(same))
        .await;
    harness.advance(5);
    assert_eq!(
        provider
            .fetch(&only(&provider))
            .await
            .expect("fetches")
            .status,
        Status::Ok
    );
}

#[tokio::test]
async fn the_saved_files_and_everything_shown_hold_no_token() {
    let harness = Harness::new().await;
    harness.sign_in_gh("example-user", Some(GH_TOKEN));
    harness.sign_in_copilot_cli(Some("cli-user"), CLI_TOKEN, "20260115100000Z");
    harness.mount(answer(PRO_CREDITS)).await;
    let provider = harness.provider();
    let mut shown = String::new();
    for account in discovered(&provider) {
        shown.push_str(&serde_json::to_string(&account).expect("serializes"));
        let result = provider.fetch(&account).await;
        shown.push_str(&printed(&result));
    }
    shown.push_str(&provider.discovery_warnings().join(" "));

    let saved = harness.cache_contents();
    assert!(
        saved.contains("\"used_pct\":25.0"),
        "the numbers were saved"
    );
    for text in [&shown, &saved] {
        assert_no_token(text);
        // The redact pass finds nothing to blank: no token shape reached them.
        assert!(matches!(
            quotadial_core::redact_secrets(text),
            std::borrow::Cow::Borrowed(_)
        ));
    }
}

#[tokio::test]
async fn an_unreachable_github_shows_stale_numbers_without_using_up_the_floor() {
    let harness = Harness::new().await;
    harness.sign_in_gh("example-user", Some(GH_TOKEN));
    harness.mount(answer(PRO_CREDITS)).await;
    let provider = harness.provider();
    let account = only(&provider);
    provider.fetch(&account).await.expect("first fetch");

    // A port with nothing listening on it.
    let closed = TcpListener::bind("127.0.0.1:0").expect("binds");
    let dead_url = format!("http://{}", closed.local_addr().expect("address"));
    drop(closed);
    harness.advance(6);
    let offline = harness.provider_with(&dead_url);
    let stale = offline.fetch(&account).await.expect("stale fetch");
    assert_eq!(stale.status, Status::Stale);
    assert!(
        stale
            .error
            .as_deref()
            .is_some_and(|error| error.starts_with("Could not reach GitHub: "))
    );

    // The unanswered attempt did not count, so the next one goes out at once.
    let back = provider.fetch(&account).await.expect("fetches");
    assert_eq!(back.status, Status::Ok);
    assert_eq!(harness.requests().await, 2);
}

#[tokio::test]
async fn a_request_that_times_out_still_counts_against_the_floor() {
    let harness = Harness::new().await;
    harness.sign_in_gh("example-user", Some(GH_TOKEN));
    harness
        .mount(answer(PRO_CREDITS).set_delay(std::time::Duration::from_secs(2)))
        .await;
    let provider = harness
        .provider()
        .with_request_timeout(std::time::Duration::from_millis(200));
    let account = only(&provider);

    let first = provider.fetch(&account).await;
    assert!(
        first
            .expect_err("times out")
            .to_string()
            .starts_with("request failed: GitHub did not answer: no answer within")
    );

    harness.advance(1);
    assert!(
        provider
            .fetch(&account)
            .await
            .expect_err("waits")
            .to_string()
            .ends_with("so the next request is after 2026-01-15T12:05:00Z.")
    );
}

#[tokio::test]
async fn a_redirect_is_never_followed_and_an_answer_over_a_mebibyte_is_not_read() {
    let harness = Harness::new().await;
    harness.sign_in_gh("example-user", Some(GH_TOKEN));
    harness
        .mount(ResponseTemplate::new(302).insert_header(
            "location",
            format!("{}/elsewhere", harness.server.uri()).as_str(),
        ))
        .await;
    let provider = harness.provider();
    let account = only(&provider);

    let redirected = provider.fetch(&account).await;
    assert_eq!(
        redirected.expect_err("fails").to_string(),
        "request failed: GitHub's Copilot usage endpoint answered HTTP 302. QuotaDial asks GitHub \
         at most every 5 minutes, so the next request is after 2026-01-15T12:05:00Z."
    );
    let requests = harness.server.received_requests().await.expect("recorded");
    assert_eq!(requests.len(), 1, "only the usage request was sent");

    harness.server.reset().await;
    let huge = format!("{{\"padding\":\"{}\"}}", "x".repeat(1024 * 1024));
    harness
        .mount(ResponseTemplate::new(200).set_body_raw(huge, "application/json"))
        .await;
    harness.advance(5);
    assert_eq!(
        provider
            .fetch(&account)
            .await
            .expect_err("refused")
            .to_string(),
        "unexpected response: GitHub's answer could not be read: the usage response did not \
         arrive in full (the response is larger than 1 MiB). QuotaDial asks GitHub at most every 5 \
         minutes, so the next request is after 2026-01-15T12:10:00Z."
    );
}

#[tokio::test]
async fn an_account_of_another_provider_is_not_one_of_its() {
    let harness = Harness::new().await;
    let stranger = Account {
        provider: "codex".into(),
        provider_name: "Codex".into(),
        account_id: "example-user".into(),
        display_name: String::new(),
        plan: None,
        org: None,
        sign_in: None,
        sign_in_use: String::new(),
    };

    let error = harness
        .provider()
        .fetch(&stranger)
        .await
        .expect_err("not one of Copilot's");

    assert_eq!(
        error.to_string(),
        "unexpected response: an account of codex is not one of Copilot's"
    );
}
