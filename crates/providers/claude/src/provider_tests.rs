//! End-to-end tests of `ClaudeProvider` against a local HTTP server standing in for Anthropic,
//! with a fake keychain, a manual clock and a temporary cache directory. Nothing here touches the
//! real Keychain, Claude Code's files, the network or the user's cache.

use std::collections::HashMap;
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use chrono::{DateTime, TimeDelta, Utc};
use quotadial_core::{Secret, Source, WindowKind};
use serde_json::{Value, json};
use tempfile::TempDir;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::*;
use crate::keychain::ItemInfo;

const TOKEN: &str = "sk-ant-oat01-QUOTADIAL-TEST-TOKEN-0123456789";
const SERVICE: &str = "Claude Code-credentials";
const USAGE_FIXTURE: &str = include_str!("../../../../fixtures/claude/oauth-usage.synthetic.json");
const PROFILE_FIXTURE: &str = include_str!("../../../../fixtures/claude/oauth-profile.json");

struct ManualClock(Mutex<DateTime<Utc>>);

impl ManualClock {
    fn advance(&self, by: TimeDelta) {
        let mut now = self.0.lock().expect("clock lock");
        *now += by;
    }
}

impl Clock for ManualClock {
    fn now(&self) -> DateTime<Utc> {
        *self.0.lock().expect("clock lock")
    }
}

/// Something a test does while the token is being read, such as switching accounts.
type ReadHook = Box<dyn Fn() + Send + Sync>;

/// Keychain items by service name: (modification time, secret).
#[derive(Default)]
struct FakeKeychain {
    items: Mutex<HashMap<String, (String, String)>>,
    reads: AtomicUsize,
    on_read: Mutex<Option<ReadHook>>,
}

impl FakeKeychain {
    fn put(&self, service: &str, modified: &str, secret: &str) {
        self.items
            .lock()
            .expect("keychain lock")
            .insert(service.to_owned(), (modified.to_owned(), secret.to_owned()));
    }

    fn remove(&self, service: &str) {
        self.items.lock().expect("keychain lock").remove(service);
    }

    fn reads(&self) -> usize {
        self.reads.load(Ordering::SeqCst)
    }

    fn on_read(&self, hook: impl Fn() + Send + Sync + 'static) {
        *self.on_read.lock().expect("hook lock") = Some(Box::new(hook));
    }
}

impl Keychain for FakeKeychain {
    fn find(
        &self,
        service: &str,
        _account: Option<&str>,
    ) -> std::result::Result<Option<ItemInfo>, String> {
        let items = self.items.lock().expect("keychain lock");
        Ok(items.get(service).map(|(modified, _)| ItemInfo {
            modified: Some(modified.clone()),
        }))
    }

    fn read(
        &self,
        service: &str,
        _account: Option<&str>,
    ) -> std::result::Result<Option<Secret>, String> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        let secret = self
            .items
            .lock()
            .expect("keychain lock")
            .get(service)
            .map(|(_, secret)| Secret::new(secret.clone()));
        if let Some(hook) = self.on_read.lock().expect("hook lock").as_ref() {
            hook();
        }
        Ok(secret)
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
            home: tempfile::tempdir().expect("home dir"),
            cache: tempfile::tempdir().expect("cache dir"),
            clock: Arc::new(ManualClock(Mutex::new(ts("2099-01-01T00:00:00Z")))),
            keychain: Arc::new(FakeKeychain::default()),
        }
    }

    fn env(&self) -> Environment {
        Environment {
            home: Some(self.home.path().to_path_buf()),
            user: Some("example".into()),
            ..Environment::default()
        }
    }

    fn provider(&self) -> ClaudeProvider {
        self.provider_with(self.env(), &self.server.uri())
    }

    fn provider_with(&self, env: Environment, base_url: &str) -> ClaudeProvider {
        ClaudeProvider::for_tests(
            env,
            Arc::clone(&self.keychain) as Arc<dyn Keychain>,
            Arc::clone(&self.clock) as Arc<dyn Clock>,
            self.cache.path().join("claude"),
            base_url,
        )
    }

    /// Signs Claude Code in: a Keychain item holding `oauth` and the account config naming the
    /// account.
    fn sign_in(&self, oauth: Value) {
        self.keychain.put(
            SERVICE,
            "20990101000000Z",
            &json!({ "claudeAiOauth": oauth }).to_string(),
        );
        self.write_account_config("00000000-0000-0000-0000-000000000001");
    }

    fn write_account_config(&self, account_uuid: &str) {
        std::fs::write(
            self.home.path().join(".claude.json"),
            json!({
                "oauthAccount": {
                    "accountUuid": account_uuid,
                    "emailAddress": "user@example.com",
                    "organizationUuid": "00000000-0000-0000-0000-000000000002",
                    "organizationType": "claude_max"
                }
            })
            .to_string(),
        )
        .expect("writes account config");
    }

    async fn mount_usage(&self, response: ResponseTemplate) {
        Mock::given(method("GET"))
            .and(path("/api/oauth/usage"))
            .respond_with(response)
            .mount(&self.server)
            .await;
    }

    async fn requests(&self) -> usize {
        self.server
            .received_requests()
            .await
            .map_or(0, |requests| requests.len())
    }

    /// Requests to the usage endpoint only, which is what the floor limits.
    async fn usage_requests(&self) -> usize {
        self.server.received_requests().await.map_or(0, |requests| {
            requests
                .iter()
                .filter(|request| request.url.path() == "/api/oauth/usage")
                .count()
        })
    }

    fn advance(&self, minutes: i64) {
        self.clock.advance(TimeDelta::minutes(minutes));
    }
}

fn ts(rfc3339: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(rfc3339)
        .expect("valid timestamp")
        .with_timezone(&Utc)
}

fn oauth() -> Value {
    json!({
        "accessToken": TOKEN,
        "refreshToken": "sk-ant-ort01-QUOTADIAL-TEST-REFRESH-0123456789",
        "expiresAt": 4_102_444_800_000_u64,
        "scopes": ["user:inference", "user:profile"],
        "subscriptionType": "max",
        "rateLimitTier": "default_claude_max_20x"
    })
}

fn usage_ok() -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_raw(USAGE_FIXTURE, "application/json")
}

/// Fails the test if the token shows up anywhere in what QuotaDial would print.
fn assert_no_token(output: &str) {
    assert!(!output.contains(TOKEN), "the token leaked into: {output}");
    assert!(!output.contains("sk-ant-"), "a token leaked into: {output}");
}

fn printed(result: &Result<Snapshot>) -> String {
    match result {
        Ok(snapshot) => serde_json::to_string(snapshot).expect("serializes"),
        Err(error) => format!("{error} / {error:?}"),
    }
}

async fn discovered(provider: &ClaudeProvider) -> Account {
    let mut accounts = provider.discover_accounts().expect("discovers");
    assert_eq!(accounts.len(), 1, "one Claude account per shell");
    accounts.remove(0)
}

#[test]
fn the_floor_matches_the_public_constant() {
    assert_eq!(POLICY.floor.to_std().expect("positive"), MIN_POLL_INTERVAL);
}

#[tokio::test]
async fn discovers_the_signed_in_account_without_reading_the_token() {
    let harness = Harness::new().await;
    harness.sign_in(oauth());

    let account = discovered(&harness.provider()).await;

    assert_eq!(
        account,
        Account {
            provider: "claude".into(),
            account_id: "00000000-0000-0000-0000-000000000001".into(),
            display_name: "user@example.com".into(),
            plan: Some("max".into()),
            org: Some("00000000-0000-0000-0000-000000000002".into()),
        }
    );
    assert_eq!(harness.keychain.reads(), 0);
}

#[tokio::test]
async fn a_signed_out_machine_has_no_accounts() {
    let harness = Harness::new().await;

    assert!(
        harness
            .provider()
            .discover_accounts()
            .expect("discovers")
            .is_empty()
    );
}

#[tokio::test]
async fn fetches_usage_with_the_headers_claude_code_sends() {
    let harness = Harness::new().await;
    harness.sign_in(oauth());
    Mock::given(method("GET"))
        .and(path("/api/oauth/usage"))
        .and(header("authorization", format!("Bearer {TOKEN}").as_str()))
        .and(header("anthropic-beta", "oauth-2025-04-20"))
        .and(header("user-agent", "claude-code/2.1.286"))
        .and(header("accept", "application/json"))
        .respond_with(usage_ok())
        .expect(1)
        .mount(&harness.server)
        .await;
    let provider = harness.provider();
    let account = discovered(&provider).await;

    let result = provider.fetch(&account).await;

    assert_no_token(&printed(&result));
    let snapshot = result.expect("fetches");
    assert_eq!(snapshot.status, Status::Ok);
    assert_eq!(snapshot.account, account);
    let windows: Vec<_> = snapshot
        .windows
        .iter()
        .map(|window| (window.kind, window.label.as_str(), window.used_pct))
        .collect();
    assert_eq!(
        windows,
        [
            (WindowKind::Session, "Session", 12.0),
            (WindowKind::Weekly, "Weekly", 34.0),
            (WindowKind::ModelWeekly, "Weekly (Opus)", 0.0),
            (WindowKind::ModelWeekly, "Weekly (Sonnet)", 56.0),
        ]
    );
    for window in &snapshot.windows {
        assert_eq!(window.source, Source::Api);
        assert_eq!(window.fetched_at, ts("2099-01-01T00:00:00Z"));
    }
    // The account config named the account, so the profile endpoint was not asked.
    assert_eq!(harness.requests().await, 1);
}

#[tokio::test]
async fn within_five_minutes_a_fetch_replays_from_the_cache_without_reading_the_token() {
    let harness = Harness::new().await;
    harness.sign_in(oauth());
    harness.mount_usage(usage_ok()).await;
    let provider = harness.provider();
    let account = discovered(&provider).await;
    provider.fetch(&account).await.expect("first fetch");

    harness.advance(4);
    // A separate provider, like a second `quotadial status` process sharing the cache.
    let second = harness
        .provider()
        .fetch(&account)
        .await
        .expect("second fetch");

    assert_eq!(harness.requests().await, 1);
    assert_eq!(harness.keychain.reads(), 1);
    assert_eq!(second.status, Status::Ok);
    assert!(
        second
            .windows
            .iter()
            .all(|window| window.source == Source::Cache)
    );
    assert_eq!(second.windows[0].fetched_at, ts("2099-01-01T00:00:00Z"));
}

#[tokio::test]
async fn after_five_minutes_it_asks_again() {
    let harness = Harness::new().await;
    harness.sign_in(oauth());
    harness.mount_usage(usage_ok()).await;
    let provider = harness.provider();
    let account = discovered(&provider).await;
    provider.fetch(&account).await.expect("first fetch");

    harness.advance(5);
    let second = provider.fetch(&account).await.expect("second fetch");

    assert_eq!(harness.requests().await, 2);
    assert!(
        second
            .windows
            .iter()
            .all(|window| window.source == Source::Api)
    );
    assert_eq!(second.windows[0].fetched_at, ts("2099-01-01T00:05:00Z"));
}

#[tokio::test]
async fn concurrent_fetches_send_one_request() {
    let harness = Harness::new().await;
    harness.sign_in(oauth());
    harness.mount_usage(usage_ok()).await;
    let (first, second) = (harness.provider(), harness.provider());
    let account = discovered(&first).await;

    let (a, b) = tokio::join!(first.fetch(&account), second.fetch(&account));

    assert_eq!(harness.requests().await, 1);
    let sources: Vec<Source> = [a.expect("first"), b.expect("second")]
        .iter()
        .map(|snapshot| snapshot.windows[0].source)
        .collect();
    assert!(sources.contains(&Source::Api));
    assert!(sources.contains(&Source::Cache));
}

#[tokio::test]
async fn a_429_keeps_the_last_numbers_and_honours_retry_after() {
    let harness = Harness::new().await;
    harness.sign_in(oauth());
    harness.mount_usage(usage_ok()).await;
    let provider = harness.provider();
    let account = discovered(&provider).await;
    provider.fetch(&account).await.expect("first fetch");

    harness.server.reset().await;
    harness
        .mount_usage(ResponseTemplate::new(429).insert_header("retry-after", "1200"))
        .await;
    harness.advance(6);
    let limited = provider.fetch(&account).await.expect("rate-limited fetch");

    assert_eq!(limited.status, Status::RateLimited);
    assert_eq!(limited.windows.len(), 4);
    assert!(
        limited
            .windows
            .iter()
            .all(|window| window.source == Source::Cache)
    );
    assert_eq!(limited.windows[0].used_pct, 12.0);
    assert_eq!(
        limited.error.as_deref(),
        Some(
            "Anthropic is rate-limiting usage requests, so QuotaDial will ask again after \
             2099-01-01T00:26:00Z."
        )
    );

    // Blocked for the 20 minutes Retry-After asked for, well past the 5-minute floor.
    harness.advance(10);
    let still = provider.fetch(&account).await.expect("blocked fetch");
    assert_eq!(still.status, Status::RateLimited);
    assert_eq!(harness.requests().await, 1);

    harness.server.reset().await;
    harness.mount_usage(usage_ok()).await;
    harness.advance(10);
    let recovered = provider.fetch(&account).await.expect("recovered fetch");
    assert_eq!(recovered.status, Status::Ok);
    assert_eq!(harness.requests().await, 1);
}

#[tokio::test]
async fn repeated_429s_back_off_exponentially() {
    let harness = Harness::new().await;
    harness.sign_in(oauth());
    harness.mount_usage(ResponseTemplate::new(429)).await;
    let provider = harness.provider();
    let account = discovered(&provider).await;

    let first = provider.fetch(&account).await.expect("first 429");
    assert_eq!(first.status, Status::RateLimited);
    assert!(first.windows.is_empty());

    // 5 minutes after the first 429, then 10 after the second.
    harness.advance(4);
    provider.fetch(&account).await.expect("blocked");
    assert_eq!(harness.requests().await, 1);
    harness.advance(1);
    provider.fetch(&account).await.expect("second 429");
    assert_eq!(harness.requests().await, 2);
    harness.advance(9);
    provider.fetch(&account).await.expect("blocked");
    assert_eq!(harness.requests().await, 2);
    harness.advance(1);
    provider.fetch(&account).await.expect("third 429");
    assert_eq!(harness.requests().await, 3);
}

#[tokio::test]
async fn a_401_asks_for_a_new_sign_in_which_is_then_tried_at_once() {
    let harness = Harness::new().await;
    harness.sign_in(oauth());
    harness.mount_usage(ResponseTemplate::new(401)).await;
    let provider = harness.provider();
    let account = discovered(&provider).await;

    let rejected = provider.fetch(&account).await.expect("fetch");
    assert_eq!(rejected.status, Status::AuthRequired);
    assert_eq!(
        rejected.error.as_deref(),
        Some("Anthropic rejected Claude Code's stored sign-in. Run `claude` to sign in again.")
    );

    harness.advance(1);
    let replayed = provider.fetch(&account).await.expect("fetch");
    assert_eq!(replayed.status, Status::AuthRequired);
    assert_eq!(harness.requests().await, 1);

    // The user runs `claude`, which writes a fresh token.
    harness.server.reset().await;
    harness.mount_usage(usage_ok()).await;
    harness.keychain.put(
        SERVICE,
        "20990101000100Z",
        &json!({ "claudeAiOauth": oauth() }).to_string(),
    );
    let signed_in = provider.fetch(&account).await.expect("fetch");
    assert_eq!(signed_in.status, Status::Ok);
    assert_eq!(harness.requests().await, 1);
}

#[tokio::test]
async fn an_expired_token_is_never_sent() {
    let harness = Harness::new().await;
    let mut expired = oauth();
    expired["expiresAt"] = json!(4_070_908_799_000_u64);
    harness.sign_in(expired);
    harness.mount_usage(usage_ok()).await;
    let provider = harness.provider();
    let account = discovered(&provider).await;

    let snapshot = provider.fetch(&account).await.expect("fetch");

    assert_eq!(snapshot.status, Status::AuthRequired);
    assert_eq!(
        snapshot.error.as_deref(),
        Some(
            "Claude Code's sign-in expired at 2098-12-31T23:59:59Z. Run `claude` once to refresh it."
        )
    );
    assert_eq!(harness.requests().await, 0);
}

#[tokio::test]
async fn a_token_without_the_profile_scope_is_never_sent() {
    let harness = Harness::new().await;
    let mut limited = oauth();
    limited["scopes"] = json!(["user:inference"]);
    harness.sign_in(limited);
    harness.mount_usage(usage_ok()).await;
    let provider = harness.provider();
    let account = discovered(&provider).await;

    let snapshot = provider.fetch(&account).await.expect("fetch");

    assert_eq!(snapshot.status, Status::AuthRequired);
    assert!(
        snapshot
            .error
            .as_deref()
            .is_some_and(|error| error.contains("user:profile"))
    );
    assert_eq!(harness.requests().await, 0);
}

#[tokio::test]
async fn an_environment_token_is_named_by_the_profile_endpoint() {
    let harness = Harness::new().await;
    harness.mount_usage(usage_ok()).await;
    Mock::given(method("GET"))
        .and(path("/api/oauth/profile"))
        .and(header("authorization", format!("Bearer {TOKEN}").as_str()))
        .respond_with(ResponseTemplate::new(200).set_body_raw(PROFILE_FIXTURE, "application/json"))
        .expect(1)
        .mount(&harness.server)
        .await;
    let env = Environment {
        oauth_token: Some(Secret::new(TOKEN)),
        ..harness.env()
    };
    let provider = harness.provider_with(env, &harness.server.uri());

    let account = discovered(&provider).await;
    assert!(account.account_id.starts_with("env:"));
    assert_eq!(account.display_name, "CLAUDE_CODE_OAUTH_TOKEN");
    let result = provider.fetch(&account).await;

    assert_no_token(&printed(&result));
    let snapshot = result.expect("fetch");
    assert_eq!(snapshot.status, Status::Ok);
    assert_eq!(
        snapshot.account,
        Account {
            provider: "claude".into(),
            account_id: "00000000-0000-0000-0000-000000000001".into(),
            display_name: "user@example.com".into(),
            plan: Some("max".into()),
            org: Some("00000000-0000-0000-0000-000000000002".into()),
        }
    );
    assert_eq!(harness.keychain.reads(), 0);
}

#[tokio::test]
async fn a_403_for_a_missing_scope_names_the_environment_variable() {
    let harness = Harness::new().await;
    harness
        .mount_usage(ResponseTemplate::new(403).set_body_json(json!({
            "type": "error",
            "error": { "type": "permission_error",
                       "message": "OAuth token does not meet scope requirement user:profile" }
        })))
        .await;
    let env = Environment {
        oauth_token: Some(Secret::new(TOKEN)),
        ..harness.env()
    };
    let provider = harness.provider_with(env, &harness.server.uri());
    let account = discovered(&provider).await;

    let snapshot = provider.fetch(&account).await.expect("fetch");

    assert_eq!(snapshot.status, Status::AuthRequired);
    assert_eq!(
        snapshot.error.as_deref(),
        Some(
            "The token in CLAUDE_CODE_OAUTH_TOKEN cannot read usage because it lacks the \
             user:profile scope. Unset it to use Claude Code's own sign-in."
        )
    );
}

#[tokio::test]
async fn a_free_plan_is_unsupported_with_a_reason_instead_of_zero() {
    let harness = Harness::new().await;
    let mut free = oauth();
    free["subscriptionType"] = json!("free");
    harness.sign_in(free);
    harness
        .mount_usage(ResponseTemplate::new(200).set_body_json(json!({
            "five_hour": null,
            "seven_day": null,
            "extra_usage": { "is_enabled": false }
        })))
        .await;
    let provider = harness.provider();
    let account = discovered(&provider).await;

    let snapshot = provider.fetch(&account).await.expect("fetch");

    assert_eq!(snapshot.status, Status::Unsupported);
    assert!(snapshot.windows.is_empty());
    assert_eq!(snapshot.account.plan.as_deref(), Some("free"));
    assert!(
        snapshot
            .error
            .as_deref()
            .is_some_and(|error| error.starts_with("Claude Free plans"))
    );
}

#[tokio::test]
async fn a_server_error_shows_the_last_numbers_as_stale() {
    let harness = Harness::new().await;
    harness.sign_in(oauth());
    harness.mount_usage(usage_ok()).await;
    let provider = harness.provider();
    let account = discovered(&provider).await;
    provider.fetch(&account).await.expect("first fetch");

    harness.server.reset().await;
    harness
        .mount_usage(ResponseTemplate::new(503).set_body_json(json!({
            "type": "error", "error": { "type": "overloaded_error", "message": "Overloaded" }
        })))
        .await;
    harness.advance(6);
    let stale = provider.fetch(&account).await.expect("stale fetch");

    assert_eq!(stale.status, Status::Stale);
    assert!(
        stale
            .windows
            .iter()
            .all(|window| window.source == Source::Cache)
    );
    assert_eq!(
        stale.error.as_deref(),
        Some(
            "Anthropic's usage endpoint answered HTTP 503: Overloaded. Showing the last numbers \
             fetched; QuotaDial asks again after 2099-01-01T00:11:00Z."
        )
    );
}

#[tokio::test]
async fn a_server_error_with_nothing_cached_is_an_error_and_still_holds_the_floor() {
    let harness = Harness::new().await;
    harness.sign_in(oauth());
    harness.mount_usage(ResponseTemplate::new(500)).await;
    let provider = harness.provider();
    let account = discovered(&provider).await;

    let first = provider.fetch(&account).await;
    assert_eq!(
        first.expect_err("fails").to_string(),
        "claude: request failed: Anthropic's usage endpoint answered HTTP 500. QuotaDial asks \
         Anthropic at most every 5 minutes, so the next request is after 2099-01-01T00:05:00Z."
    );

    harness.advance(2);
    let second = provider.fetch(&account).await;
    assert!(
        second
            .expect_err("waits")
            .to_string()
            .contains("next request is after 2099-01-01T00:05:00Z")
    );
    assert_eq!(harness.requests().await, 1);
}

#[tokio::test]
async fn an_unreachable_anthropic_shows_stale_numbers_without_using_up_the_floor() {
    let harness = Harness::new().await;
    harness.sign_in(oauth());
    harness.mount_usage(usage_ok()).await;
    let provider = harness.provider();
    let account = discovered(&provider).await;
    provider.fetch(&account).await.expect("first fetch");

    // A port with nothing listening on it.
    let closed = TcpListener::bind("127.0.0.1:0").expect("binds");
    let dead_url = format!("http://{}", closed.local_addr().expect("address"));
    drop(closed);
    harness.advance(6);
    let offline = harness.provider_with(harness.env(), &dead_url);
    let stale = offline.fetch(&account).await.expect("stale fetch");
    assert_eq!(stale.status, Status::Stale);
    assert!(
        stale
            .error
            .as_deref()
            .is_some_and(|error| error.starts_with("Could not reach Anthropic: "))
    );

    // The unanswered attempt did not count, so the next one goes out at once.
    harness.advance(1);
    let back = provider.fetch(&account).await.expect("fetch");
    assert_eq!(back.status, Status::Ok);
    assert_eq!(harness.requests().await, 2);
}

#[tokio::test]
async fn another_account_signed_in_is_never_blended_with_this_one() {
    let harness = Harness::new().await;
    harness.sign_in(oauth());
    harness.mount_usage(usage_ok()).await;
    let provider = harness.provider();
    let account = discovered(&provider).await;

    harness.write_account_config("00000000-0000-0000-0000-000000000009");
    let snapshot = provider.fetch(&account).await.expect("fetch");

    assert_eq!(snapshot.status, Status::AuthRequired);
    assert_eq!(snapshot.error.as_deref(), Some(advice::DIFFERENT_ACCOUNT));
    assert_eq!(harness.requests().await, 0);
}

#[tokio::test]
async fn signing_out_after_discovery_asks_for_a_sign_in() {
    let harness = Harness::new().await;
    harness.sign_in(oauth());
    let provider = harness.provider();
    let account = discovered(&provider).await;

    harness.keychain.remove(SERVICE);
    let snapshot = provider.fetch(&account).await.expect("fetch");

    assert_eq!(snapshot.status, Status::AuthRequired);
    assert_eq!(snapshot.error.as_deref(), Some(advice::SIGNED_OUT));
}

#[tokio::test]
async fn a_keychain_item_with_only_mcp_sign_ins_asks_for_a_claude_sign_in() {
    let harness = Harness::new().await;
    harness.keychain.put(
        SERVICE,
        "20990101000000Z",
        &json!({ "mcpOAuth": { "server": { "accessToken": TOKEN } } }).to_string(),
    );
    harness.mount_usage(usage_ok()).await;
    let provider = harness.provider();
    let account = discovered(&provider).await;

    let result = provider.fetch(&account).await;

    assert_no_token(&printed(&result));
    let snapshot = result.expect("fetch");
    assert_eq!(snapshot.status, Status::AuthRequired);
    assert!(
        snapshot
            .error
            .as_deref()
            .is_some_and(|error| error.contains("only MCP server sign-ins"))
    );
    assert_eq!(harness.requests().await, 0);
}

#[tokio::test]
async fn a_damaged_credentials_file_is_an_error_that_names_the_file_but_not_its_contents() {
    let harness = Harness::new().await;
    let config_dir = harness.home.path().join(".claude");
    std::fs::create_dir_all(&config_dir).expect("config dir");
    let file = config_dir.join(".credentials.json");
    std::fs::write(
        &file,
        format!("{{\"claudeAiOauth\": {{\"accessToken\": \"{TOKEN}\""),
    )
    .expect("writes");
    let provider = harness.provider();
    let account = discovered(&provider).await;

    let result = provider.fetch(&account).await;

    assert_no_token(&printed(&result));
    assert_eq!(
        result.expect_err("fails").to_string(),
        format!(
            "claude: could not read credentials from {}: it is not valid JSON; run `claude` to \
             sign in again",
            file.display()
        )
    );
}

#[tokio::test]
async fn reads_the_credentials_file_where_there_is_no_keychain() {
    let harness = Harness::new().await;
    let config_dir = harness.home.path().join(".claude");
    std::fs::create_dir_all(&config_dir).expect("config dir");
    std::fs::write(
        config_dir.join(".credentials.json"),
        json!({ "claudeAiOauth": oauth() }).to_string(),
    )
    .expect("writes");
    harness.write_account_config("00000000-0000-0000-0000-000000000001");
    harness.mount_usage(usage_ok()).await;
    let provider = harness.provider();
    let account = discovered(&provider).await;

    let snapshot = provider.fetch(&account).await.expect("fetch");

    assert_eq!(snapshot.status, Status::Ok);
    assert_eq!(harness.keychain.reads(), 0);
}

#[tokio::test]
async fn an_unwritable_cache_stops_the_request() {
    let harness = Harness::new().await;
    harness.sign_in(oauth());
    harness.mount_usage(usage_ok()).await;
    // A file where the cache directory should be.
    let blocked = harness.cache.path().join("not-a-directory");
    std::fs::write(&blocked, "").expect("writes");
    let provider = ClaudeProvider::for_tests(
        harness.env(),
        Arc::clone(&harness.keychain) as Arc<dyn Keychain>,
        Arc::clone(&harness.clock) as Arc<dyn Clock>,
        blocked.join("claude"),
        &harness.server.uri(),
    );
    let account = discovered(&provider).await;

    let error = provider.fetch(&account).await.expect_err("fails");

    assert!(
        error
            .to_string()
            .starts_with("claude: could not use QuotaDial's cache at ")
    );
    assert_eq!(harness.requests().await, 0);
}

#[test]
fn debug_output_shows_no_token() {
    let env = Environment {
        oauth_token: Some(Secret::new(TOKEN)),
        ..Environment::default()
    };
    let provider = ClaudeProvider::for_tests(
        env,
        Arc::new(FakeKeychain::default()),
        Arc::new(ManualClock(Mutex::new(Utc::now()))),
        PathBuf::from("/nonexistent"),
        "http://127.0.0.1:9",
    );

    assert_no_token(&format!("{provider:?}"));
}

#[test]
fn never_polls_anthropic_more_than_once_every_five_minutes() {
    assert!(ClaudeProvider::new().min_poll_interval() >= std::time::Duration::from_secs(300));
}

fn oversized_body() -> String {
    format!("{{\"padding\": \"{}\"}}", "x".repeat(1024 * 1024 + 1))
}

#[tokio::test]
async fn an_answer_too_large_to_read_still_counts_against_the_floor() {
    let harness = Harness::new().await;
    harness.sign_in(oauth());
    harness
        .mount_usage(ResponseTemplate::new(200).set_body_raw(oversized_body(), "application/json"))
        .await;
    let provider = harness.provider();
    let account = discovered(&provider).await;

    let first = provider.fetch(&account).await;
    assert!(
        matches!(first, Err(Error::InvalidResponse { .. })),
        "{first:?}"
    );

    harness.advance(1);
    let second = provider.fetch(&account).await;
    assert!(
        second
            .expect_err("waits")
            .to_string()
            .contains("next request is after 2099-01-01T00:05:00Z")
    );
    assert_eq!(harness.requests().await, 1);
}

#[tokio::test]
async fn a_429_is_honoured_even_when_its_body_is_unreadable() {
    let harness = Harness::new().await;
    harness.sign_in(oauth());
    harness
        .mount_usage(
            ResponseTemplate::new(429)
                .insert_header("retry-after", "3600")
                .set_body_raw(oversized_body(), "application/json"),
        )
        .await;
    let provider = harness.provider();
    let account = discovered(&provider).await;

    let limited = provider.fetch(&account).await.expect("fetch");
    assert_eq!(limited.status, Status::RateLimited);

    harness.advance(30);
    let still = provider.fetch(&account).await.expect("fetch");
    assert_eq!(still.status, Status::RateLimited);
    assert_eq!(harness.requests().await, 1);
}

#[tokio::test]
async fn a_request_that_times_out_still_counts_against_the_floor() {
    let harness = Harness::new().await;
    harness.sign_in(oauth());
    harness
        .mount_usage(usage_ok().set_delay(std::time::Duration::from_secs(3)))
        .await;
    let provider = harness
        .provider()
        .with_request_timeout(std::time::Duration::from_millis(300));
    let account = discovered(&provider).await;

    let first = provider.fetch(&account).await;
    assert!(first.expect_err("times out").to_string().starts_with(
        "claude: request failed: Anthropic did not answer: no answer within 0.3 seconds"
    ));

    harness.advance(2);
    assert!(provider.fetch(&account).await.is_err());
    assert_eq!(harness.requests().await, 1);
}

#[tokio::test]
async fn two_environment_tokens_never_share_numbers() {
    let harness = Harness::new().await;
    harness.mount_usage(usage_ok()).await;
    let shell = |token: &str| {
        harness.provider_with(
            Environment {
                oauth_token: Some(Secret::new(token)),
                ..harness.env()
            },
            &harness.server.uri(),
        )
    };
    let (first, second) = (
        shell(TOKEN),
        shell("sk-ant-oat01-QUOTADIAL-OTHER-TOKEN-987654321"),
    );
    let (first_account, second_account) = (discovered(&first).await, discovered(&second).await);
    assert_ne!(first_account.account_id, second_account.account_id);

    first.fetch(&first_account).await.expect("first shell");
    harness.advance(1);
    let other = second.fetch(&second_account).await.expect("second shell");

    // The second shell's token was sent, and its numbers are its own.
    assert_eq!(harness.usage_requests().await, 2);
    assert!(
        other
            .windows
            .iter()
            .all(|window| window.source == Source::Api)
    );
}

#[tokio::test]
async fn an_environment_token_is_named_once_and_remembered() {
    let harness = Harness::new().await;
    harness.mount_usage(usage_ok()).await;
    Mock::given(method("GET"))
        .and(path("/api/oauth/profile"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(PROFILE_FIXTURE, "application/json"))
        .expect(1)
        .mount(&harness.server)
        .await;
    let env = Environment {
        oauth_token: Some(Secret::new(TOKEN)),
        ..harness.env()
    };
    let provider = harness.provider_with(env, &harness.server.uri());
    let account = discovered(&provider).await;

    let first = provider.fetch(&account).await.expect("first fetch");
    harness.advance(6);
    let second = provider.fetch(&account).await.expect("second fetch");

    assert_eq!(first.account.display_name, "user@example.com");
    assert_eq!(second.account, first.account);
    assert!(
        second
            .windows
            .iter()
            .all(|window| window.source == Source::Api)
    );
    // Two usage requests and one profile request.
    assert_eq!(harness.requests().await, 3);
}

#[tokio::test]
async fn a_switch_to_another_account_while_reading_the_token_is_not_counted() {
    let harness = Harness::new().await;
    harness.sign_in(oauth());
    harness.mount_usage(usage_ok()).await;
    let provider = harness.provider();
    let account = discovered(&provider).await;
    let home = harness.home.path().to_path_buf();
    harness.keychain.on_read(move || {
        std::fs::write(
            home.join(".claude.json"),
            json!({ "oauthAccount": {
                "accountUuid": "00000000-0000-0000-0000-000000000009",
                "emailAddress": "other@example.com",
                "organizationUuid": "00000000-0000-0000-0000-000000000002"
            } })
            .to_string(),
        )
        .expect("switches account");
    });

    let snapshot = provider.fetch(&account).await.expect("fetch");

    assert_eq!(snapshot.status, Status::AuthRequired);
    assert_eq!(snapshot.error.as_deref(), Some(advice::DIFFERENT_ACCOUNT));
    assert_eq!(harness.requests().await, 0);
}

#[tokio::test]
async fn a_clock_moved_back_does_not_let_a_request_through() {
    let harness = Harness::new().await;
    harness.sign_in(oauth());
    harness.mount_usage(usage_ok()).await;
    let provider = harness.provider();
    let account = discovered(&provider).await;
    provider.fetch(&account).await.expect("first fetch");

    harness.clock.advance(TimeDelta::hours(-3));
    let replayed = provider.fetch(&account).await.expect("replay");
    assert!(
        replayed
            .windows
            .iter()
            .all(|window| window.source == Source::Cache)
    );
    assert_eq!(harness.requests().await, 1);

    harness.advance(5);
    provider.fetch(&account).await.expect("fetch");
    assert_eq!(harness.requests().await, 2);
}

#[tokio::test]
async fn an_unnamed_sign_in_does_not_show_old_numbers_after_a_new_token() {
    let harness = Harness::new().await;
    // A Keychain item but no account config, so the account is unnamed until Anthropic names it.
    harness.keychain.put(
        SERVICE,
        "20990101000000Z",
        &json!({ "claudeAiOauth": oauth() }).to_string(),
    );
    harness.mount_usage(usage_ok()).await;
    let provider = harness.provider();
    let account = discovered(&provider).await;
    assert_eq!(account.account_id, SERVICE);
    provider.fetch(&account).await.expect("first fetch");

    // Claude Code writes a new token, possibly for another account.
    harness.keychain.put(
        SERVICE,
        "20990101000200Z",
        &json!({ "claudeAiOauth": oauth() }).to_string(),
    );
    harness.advance(2);
    let waiting = provider.fetch(&account).await;

    assert!(
        waiting
            .expect_err("waits")
            .to_string()
            .contains("Claude Code's sign-in changed since QuotaDial last asked Anthropic.")
    );
    assert_eq!(harness.usage_requests().await, 1);
}

/// Replaces the cache directory with a file while Anthropic answers 429, so the answer cannot be
/// saved even by root.
#[cfg(unix)]
struct BreakCacheThen429(PathBuf);

#[cfg(unix)]
impl wiremock::Respond for BreakCacheThen429 {
    fn respond(&self, _request: &wiremock::Request) -> ResponseTemplate {
        std::fs::remove_dir_all(&self.0).expect("removes the cache");
        std::fs::write(&self.0, "").expect("puts a file in its place");
        ResponseTemplate::new(429).insert_header("retry-after", "3600")
    }
}

#[cfg(unix)]
#[tokio::test]
async fn an_answer_that_cannot_be_saved_fails_the_fetch() {
    let harness = Harness::new().await;
    harness.sign_in(oauth());
    Mock::given(method("GET"))
        .and(path("/api/oauth/usage"))
        .respond_with(BreakCacheThen429(harness.cache.path().join("claude")))
        .mount(&harness.server)
        .await;
    let provider = harness.provider();
    let account = discovered(&provider).await;

    let result = provider.fetch(&account).await;

    assert!(
        result
            .expect_err("fails")
            .to_string()
            .starts_with("claude: could not use QuotaDial's cache at ")
    );
    assert_eq!(harness.requests().await, 1);
}

/// The account's state file in the test cache directory.
fn state_file(harness: &Harness, account: &Account) -> PathBuf {
    harness
        .cache
        .path()
        .join("claude")
        .join(format!("{}.json", quotadial_poll::account_key(account)))
}

/// Everything QuotaDial wrote to its cache directory, as text.
fn cache_contents(harness: &Harness) -> String {
    let dir = harness.cache.path().join("claude");
    std::fs::read_dir(dir)
        .expect("cache dir")
        .map(|entry| std::fs::read_to_string(entry.expect("entry").path()).expect("reads"))
        .collect()
}

#[tokio::test]
async fn a_token_quoted_in_anthropics_error_is_never_shown_or_saved() {
    let harness = Harness::new().await;
    harness.sign_in(oauth());
    let quoting = |status| {
        ResponseTemplate::new(status).set_body_json(json!({
            "type": "error",
            "error": { "type": "authentication_error", "message": format!("invalid token: {TOKEN}") }
        }))
    };
    let provider = harness.provider();
    let account = discovered(&provider).await;

    // A 401 is judged by its status alone, so its message is never read.
    harness.mount_usage(quoting(401)).await;
    let rejected = provider.fetch(&account).await;
    assert_no_token(&printed(&rejected));
    assert_eq!(rejected.expect("snapshot").status, Status::AuthRequired);

    // Any other status passes the message on, with the token blanked out.
    harness.server.reset().await;
    harness.mount_usage(quoting(400)).await;
    harness.advance(5);
    let failed = provider.fetch(&account).await;
    assert_no_token(&printed(&failed));
    assert_eq!(
        failed.expect_err("fails").to_string(),
        "claude: request failed: Anthropic's usage endpoint answered HTTP 400: invalid token: \
         [redacted token]. QuotaDial asks Anthropic at most every 5 minutes, so the next request \
         is after 2099-01-01T00:10:00Z."
    );
    assert_eq!(harness.usage_requests().await, 1);
    assert_no_token(&cache_contents(&harness));
}

#[tokio::test]
async fn a_damaged_state_file_still_holds_the_floor() {
    let harness = Harness::new().await;
    harness.sign_in(oauth());
    harness.mount_usage(usage_ok()).await;
    let provider = harness.provider();
    let account = discovered(&provider).await;
    provider.fetch(&account).await.expect("first fetch");

    // Two minutes later the saved state is unreadable, so it counts as a request sent now.
    harness.advance(2);
    std::fs::write(
        state_file(&harness, &account),
        r#"{"version":1,"state":{"requ"#,
    )
    .expect("damages the state file");
    let held = provider.fetch(&account).await;
    assert_eq!(
        held.expect_err("holds").to_string(),
        "claude: request failed: QuotaDial could not read its saved state for this account, so \
         the last answer is lost. QuotaDial asks Anthropic at most every 5 minutes, so the next \
         request is after 2099-01-01T00:07:00Z."
    );
    assert_eq!(harness.usage_requests().await, 1);

    // The hold was saved again, so later fetches end it on time instead of restarting it.
    harness.advance(3);
    assert!(
        provider
            .fetch(&account)
            .await
            .expect_err("still holds")
            .to_string()
            .ends_with("next request is after 2099-01-01T00:07:00Z.")
    );
    harness.advance(2);
    let fresh = provider.fetch(&account).await.expect("fetches");
    assert_eq!(fresh.status, Status::Ok);
    assert_eq!(harness.usage_requests().await, 2);
}

#[tokio::test]
async fn a_state_file_that_no_longer_parses_keeps_its_floor_and_block() {
    let harness = Harness::new().await;
    harness.sign_in(oauth());
    harness.mount_usage(usage_ok()).await;
    let provider = harness.provider();
    let account = discovered(&provider).await;
    provider.fetch(&account).await.expect("first fetch");

    // As if a later QuotaDial changed the snapshot shape without a new format version, after a
    // 429 that blocks requests until 00:30.
    std::fs::write(
        state_file(&harness, &account),
        json!({
            "version": 1,
            "state": {
                "last_result": { "kind": "snapshot", "snapshot": { "windows": [{ "shape": "new" }] } },
                "requested_at": "2099-01-01T00:00:00Z",
                "blocked_until": "2099-01-01T00:30:00Z",
                "rate_limit_strikes": 1
            }
        })
        .to_string(),
    )
    .expect("rewrites the state file");

    harness.advance(10);
    let blocked = provider.fetch(&account).await.expect("replays");
    assert_eq!(blocked.status, Status::RateLimited);
    assert_eq!(
        blocked.error.as_deref(),
        Some(
            "Anthropic is rate-limiting usage requests, so QuotaDial will ask again after \
             2099-01-01T00:30:00Z."
        )
    );
    assert_eq!(harness.usage_requests().await, 1);

    harness.advance(20);
    let fresh = provider.fetch(&account).await.expect("fetches");
    assert_eq!(fresh.status, Status::Ok);
    assert_eq!(harness.usage_requests().await, 2);
}

#[tokio::test]
async fn a_redirect_is_never_followed() {
    let harness = Harness::new().await;
    harness.sign_in(oauth());
    harness
        .mount_usage(ResponseTemplate::new(302).insert_header(
            "location",
            format!("{}/elsewhere", harness.server.uri()).as_str(),
        ))
        .await;
    let provider = harness.provider();
    let account = discovered(&provider).await;

    let result = provider.fetch(&account).await;

    assert_eq!(
        result.expect_err("fails").to_string(),
        "claude: request failed: Anthropic's usage endpoint answered HTTP 302. QuotaDial asks \
         Anthropic at most every 5 minutes, so the next request is after 2099-01-01T00:05:00Z."
    );
    assert_eq!(
        harness.requests().await,
        1,
        "only the usage request was sent"
    );
}

/// Requests to the profile endpoint.
async fn profile_requests(harness: &Harness) -> usize {
    harness
        .server
        .received_requests()
        .await
        .map_or(0, |requests| {
            requests
                .iter()
                .filter(|request| request.url.path() == "/api/oauth/profile")
                .count()
        })
}

#[tokio::test]
async fn one_account_in_two_organizations_keeps_two_sets_of_numbers() {
    let harness = Harness::new().await;
    harness.sign_in(oauth());
    harness.mount_usage(usage_ok()).await;
    let provider = harness.provider();
    let personal = discovered(&provider).await;
    provider.fetch(&personal).await.expect("first fetch");

    // The same account, now signed in to another organization.
    std::fs::write(
        harness.home.path().join(".claude.json"),
        json!({ "oauthAccount": {
            "accountUuid": "00000000-0000-0000-0000-000000000001",
            "emailAddress": "user@example.com",
            "organizationUuid": "00000000-0000-0000-0000-000000000005",
            "organizationType": "claude_max"
        } })
        .to_string(),
    )
    .expect("writes account config");
    let work = discovered(&provider).await;
    harness.advance(1);
    let snapshot = provider.fetch(&work).await.expect("fetches");

    assert_eq!(work.account_id, personal.account_id);
    assert_ne!(work.org, personal.org);
    assert_ne!(
        quotadial_poll::account_key(&work),
        quotadial_poll::account_key(&personal)
    );
    assert_eq!(
        snapshot.windows[0].source,
        Source::Api,
        "not the other organization's cached numbers"
    );
    assert_eq!(harness.usage_requests().await, 2);
}

#[tokio::test]
async fn an_unnamed_sign_in_is_named_once_per_token() {
    let harness = Harness::new().await;
    // A Keychain item but no account config, so only the profile endpoint can name the account.
    harness.keychain.put(
        SERVICE,
        "20990101000000Z",
        &json!({ "claudeAiOauth": oauth() }).to_string(),
    );
    harness.mount_usage(usage_ok()).await;
    Mock::given(method("GET"))
        .and(path("/api/oauth/profile"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(PROFILE_FIXTURE, "application/json"))
        .mount(&harness.server)
        .await;
    let provider = harness.provider();
    let account = discovered(&provider).await;

    for _ in 0..3 {
        let snapshot = provider.fetch(&account).await.expect("fetches");
        assert_eq!(snapshot.account.display_name, "user@example.com");
        harness.advance(5);
    }
    assert_eq!(harness.usage_requests().await, 3);
    assert_eq!(
        profile_requests(&harness).await,
        1,
        "named once for this token"
    );

    // A new token may belong to another account, so it is named again.
    harness.keychain.put(
        SERVICE,
        "20990101001500Z",
        &json!({ "claudeAiOauth": oauth() }).to_string(),
    );
    provider.fetch(&account).await.expect("fetches");
    assert_eq!(profile_requests(&harness).await, 2);
}

#[tokio::test]
async fn no_cache_file_holds_the_token_after_keychain_and_file_fetches() {
    let hex = |text: &str| -> String { text.bytes().map(|byte| format!("{byte:02x}")).collect() };
    let refresh = "sk-ant-ort01-QUOTADIAL-TEST-REFRESH-0123456789";
    for from_file in [false, true] {
        let harness = Harness::new().await;
        if from_file {
            let config_dir = harness.home.path().join(".claude");
            std::fs::create_dir_all(&config_dir).expect("config dir");
            std::fs::write(
                config_dir.join(".credentials.json"),
                json!({ "claudeAiOauth": oauth() }).to_string(),
            )
            .expect("writes");
            harness.write_account_config("00000000-0000-0000-0000-000000000001");
        } else {
            harness.sign_in(oauth());
        }
        harness.mount_usage(usage_ok()).await;
        let provider = harness.provider();
        let account = discovered(&provider).await;

        provider.fetch(&account).await.expect("fetches");

        let saved = cache_contents(&harness);
        assert!(saved.contains("\"used_pct\""), "the numbers were saved");
        for secret in [TOKEN, refresh, &hex(TOKEN), &hex(refresh), "sk-ant-"] {
            assert!(
                !saved.contains(secret),
                "a token reached the cache (from file: {from_file})"
            );
        }
    }
}

#[tokio::test]
async fn a_keychain_read_that_stops_never_reports_what_it_held() {
    let harness = Harness::new().await;
    harness.sign_in(oauth());
    harness.mount_usage(usage_ok()).await;
    harness
        .keychain
        .on_read(|| panic!("a panic whose message holds secrets"));
    let provider = harness.provider();
    let account = discovered(&provider).await;

    let error = provider.fetch(&account).await.expect_err("fails");

    let message = error.to_string();
    assert!(
        message.ends_with("reading it stopped unexpectedly; try again"),
        "{message}"
    );
    assert!(!message.contains("secrets"), "{message}");
    assert_eq!(harness.usage_requests().await, 0);
}
