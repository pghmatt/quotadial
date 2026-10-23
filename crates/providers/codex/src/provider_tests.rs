//! End-to-end tests of `CodexProvider` against a local HTTP server standing in for ChatGPT, with a
//! temporary Codex home, a manual clock and a temporary cache directory. Nothing here touches the
//! real `~/.codex`, the network or the user's cache.

use std::fs;
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::Mutex;

use chrono::{DateTime, TimeDelta, Utc};
use quotadial_core::{Source, WindowKind};
use serde_json::json;
use tempfile::TempDir;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::*;
use crate::auth::tests::{USER_ID, auth_json, auth_json_for};
use crate::sessions::tests::{ROLLOUT_FIXTURE, set_modified};

/// The live Pro response from 2026-10-23, redacted (D-039).
const LIVE_FIXTURE: &str = include_str!("../../../../fixtures/codex/wham-usage.json");
/// Far in the future, so the test token never expires.
const FAR_FUTURE: i64 = 4_102_444_800;
const WORKSPACE: &str = "00000000-0000-0000-0000-000000000002";

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

struct Harness {
    server: MockServer,
    codex_home: TempDir,
    cache: TempDir,
    clock: Arc<ManualClock>,
}

impl Harness {
    async fn new() -> Self {
        Self {
            server: MockServer::start().await,
            codex_home: tempfile::tempdir().expect("codex home"),
            cache: tempfile::tempdir().expect("cache dir"),
            clock: Arc::new(ManualClock(Mutex::new(ts("2026-10-23T12:00:00Z")))),
        }
    }

    fn env(&self) -> Environment {
        Environment {
            home: None,
            codex_home: Some(self.codex_home.path().to_path_buf()),
        }
    }

    fn provider(&self) -> CodexProvider {
        self.provider_with(&self.server.uri())
    }

    fn provider_with(&self, base_url: &str) -> CodexProvider {
        CodexProvider::for_tests(
            self.env(),
            Arc::clone(&self.clock) as Arc<dyn Clock>,
            self.cache.path().join("codex"),
            base_url,
        )
    }

    fn auth_file(&self) -> PathBuf {
        self.codex_home.path().join("auth.json")
    }

    /// Signs the Codex CLI in to a ChatGPT account on `plan`, its token expiring at `expires`.
    fn sign_in(&self, plan: &str, expires: i64) {
        fs::write(self.auth_file(), auth_json(plan, expires)).expect("writes auth.json");
    }

    /// The access token the Codex CLI's sign-in holds.
    fn token(&self) -> String {
        let text = fs::read_to_string(self.auth_file()).expect("reads auth.json");
        Auth::parse(&text)
            .expect("parses")
            .access_token
            .expose()
            .to_owned()
    }

    async fn mount_usage(&self, response: ResponseTemplate) {
        Mock::given(method("GET"))
            .and(path("/backend-api/wham/usage"))
            .respond_with(response)
            .mount(&self.server)
            .await;
    }

    async fn usage_requests(&self) -> usize {
        self.server.received_requests().await.map_or(0, |requests| {
            requests
                .iter()
                .filter(|request| request.url.path() == "/backend-api/wham/usage")
                .count()
        })
    }

    fn advance(&self, minutes: i64) {
        self.clock.advance(TimeDelta::minutes(minutes));
    }

    /// Writes a Codex CLI session log, last written at `written`.
    fn write_log(&self, contents: &str, written: &str) -> PathBuf {
        let day = self.codex_home.path().join("sessions/2026/10/23");
        fs::create_dir_all(&day).expect("creates");
        let log =
            day.join("rollout-2026-10-23T09-00-00-00000000-0000-0000-0000-000000000010.jsonl");
        fs::write(&log, contents).expect("writes");
        set_modified(&log, ts(written));
        log
    }

    /// Everything QuotaDial wrote to its cache directory, as text.
    fn cache_contents(&self) -> String {
        fs::read_dir(self.cache.path().join("codex"))
            .expect("cache dir")
            .map(|entry| fs::read_to_string(entry.expect("entry").path()).expect("reads"))
            .collect()
    }
}

fn ts(rfc3339: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(rfc3339)
        .expect("valid timestamp")
        .with_timezone(&Utc)
}

fn usage_ok() -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_raw(LIVE_FIXTURE, "application/json")
}

/// What QuotaDial would print for a fetch.
fn printed(result: &Result<Snapshot>) -> String {
    match result {
        Ok(snapshot) => serde_json::to_string(snapshot).expect("serializes"),
        Err(error) => format!("{error} / {error:?}"),
    }
}

/// Fails the test if `token`, or anything that looks like one, shows up in `output`.
fn assert_no_token(output: &str, token: &str) {
    assert!(!output.contains(token), "the token leaked into: {output}");
    assert!(!output.contains("eyJ"), "a token leaked into: {output}");
}

fn discovered(provider: &CodexProvider) -> Account {
    let mut accounts = provider.discover_accounts().expect("discovers");
    assert_eq!(accounts.len(), 1, "one Codex sign-in per Codex home");
    accounts.remove(0)
}

#[tokio::test]
async fn discovers_the_signed_in_account_without_a_request() {
    let harness = Harness::new().await;
    harness.sign_in("pro", FAR_FUTURE);

    let account = discovered(&harness.provider());

    assert_eq!(
        account,
        Account {
            provider: "codex".into(),
            account_id: USER_ID.into(),
            display_name: "user@example.com".into(),
            plan: Some("pro".into()),
            org: Some(WORKSPACE.into()),
        }
    );
    assert_eq!(harness.usage_requests().await, 0);
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
async fn an_api_key_sign_in_is_shown_as_unsupported_without_a_request() {
    let harness = Harness::new().await;
    fs::write(
        harness.auth_file(),
        json!({ "auth_mode": "apikey", "OPENAI_API_KEY": "sk-proj-quotadial-test" }).to_string(),
    )
    .expect("writes auth.json");
    let provider = harness.provider();
    let account = discovered(&provider);

    let snapshot = provider.fetch(&account).await.expect("fetches");

    assert_eq!(account.display_name, "OpenAI API key");
    assert_eq!(snapshot.status, Status::Unsupported);
    assert_eq!(snapshot.error.as_deref(), Some(advice::API_KEY));
    assert!(!printed(&Ok(snapshot)).contains("sk-proj"));
    assert_eq!(harness.usage_requests().await, 0);
}

#[tokio::test]
async fn fetches_usage_with_the_headers_the_codex_cli_sends() {
    let harness = Harness::new().await;
    harness.sign_in("pro", FAR_FUTURE);
    Mock::given(method("GET"))
        .and(path("/backend-api/wham/usage"))
        .and(header(
            "authorization",
            format!("Bearer {}", harness.token()).as_str(),
        ))
        .and(header("chatgpt-account-id", WORKSPACE))
        .and(header("user-agent", "codex_cli_rs/0.163.0"))
        .and(header("accept", "application/json"))
        .respond_with(usage_ok())
        .expect(1)
        .mount(&harness.server)
        .await;
    let provider = harness.provider();
    let account = discovered(&provider);

    let result = provider.fetch(&account).await;

    assert_no_token(&printed(&result), &harness.token());
    let snapshot = result.expect("fetches");
    assert_eq!(snapshot.status, Status::Ok);
    assert_eq!(snapshot.account, account);
    assert_eq!(snapshot.windows.len(), 1);
    let weekly = &snapshot.windows[0];
    assert_eq!(weekly.kind, WindowKind::Weekly);
    assert_eq!(weekly.used_pct, 54.0);
    assert_eq!(weekly.resets_at, Some(ts("2026-10-29T17:09:48Z")));
    assert_eq!(weekly.source, Source::Api);
    assert_eq!(weekly.fetched_at, ts("2026-10-23T12:00:00Z"));
    assert_eq!(snapshot.extra_usage.map(|extra| extra.enabled), Some(true));
}

#[tokio::test]
async fn within_two_minutes_a_fetch_replays_from_the_cache_then_asks_again() {
    let harness = Harness::new().await;
    harness.sign_in("pro", FAR_FUTURE);
    harness.mount_usage(usage_ok()).await;
    let provider = harness.provider();
    let account = discovered(&provider);
    provider.fetch(&account).await.expect("first fetch");

    harness.advance(1);
    let replayed = provider.fetch(&account).await.expect("replays");
    assert_eq!(replayed.windows[0].source, Source::Cache);
    assert_eq!(replayed.windows[0].fetched_at, ts("2026-10-23T12:00:00Z"));
    assert_eq!(harness.usage_requests().await, 1);

    harness.advance(1);
    let fresh = provider.fetch(&account).await.expect("fetches");
    assert_eq!(fresh.windows[0].source, Source::Api);
    assert_eq!(harness.usage_requests().await, 2);
}

#[tokio::test]
async fn a_rejected_sign_in_asks_for_codex_login_and_a_new_sign_in_is_tried_at_once() {
    let harness = Harness::new().await;
    harness.sign_in("pro", FAR_FUTURE);
    harness.mount_usage(ResponseTemplate::new(401)).await;
    let provider = harness.provider();
    let account = discovered(&provider);

    let rejected = provider.fetch(&account).await.expect("snapshot");
    assert_eq!(rejected.status, Status::AuthRequired);
    assert_eq!(rejected.error.as_deref(), Some(advice::REJECTED));

    // The same sign-in waits out the floor.
    harness.advance(1);
    let again = provider.fetch(&account).await.expect("replays");
    assert_eq!(again.status, Status::AuthRequired);
    assert_eq!(harness.usage_requests().await, 1);

    // `codex login` rewrites auth.json, and the new sign-in is tried at once.
    harness.server.reset().await;
    harness.mount_usage(usage_ok()).await;
    harness.sign_in("pro", FAR_FUTURE);
    set_modified(&harness.auth_file(), ts("2026-10-23T12:01:00Z"));
    let signed_in = provider.fetch(&account).await.expect("fetches");
    assert_eq!(signed_in.status, Status::Ok);
    assert_eq!(harness.usage_requests().await, 1);
}

#[tokio::test]
async fn an_expired_token_is_never_sent() {
    let harness = Harness::new().await;
    harness.sign_in("pro", 1_761_220_800);
    harness.mount_usage(usage_ok()).await;
    let provider = harness.provider();
    let account = discovered(&provider);

    let snapshot = provider.fetch(&account).await.expect("snapshot");

    assert_eq!(snapshot.status, Status::AuthRequired);
    assert_eq!(
        snapshot.error.as_deref(),
        Some(advice::expired(ts("2025-10-23T12:00:00Z")).as_str())
    );
    assert_eq!(harness.usage_requests().await, 0);
}

#[tokio::test]
async fn a_429_blocks_requests_and_keeps_the_last_numbers_on_screen() {
    let harness = Harness::new().await;
    harness.sign_in("pro", FAR_FUTURE);
    harness.mount_usage(usage_ok()).await;
    let provider = harness.provider();
    let account = discovered(&provider);
    provider.fetch(&account).await.expect("first fetch");

    harness.server.reset().await;
    harness
        .mount_usage(ResponseTemplate::new(429).insert_header("retry-after", "600"))
        .await;
    harness.advance(2);
    let limited = provider.fetch(&account).await.expect("snapshot");
    assert_eq!(limited.status, Status::RateLimited);
    assert_eq!(limited.windows[0].source, Source::Cache);
    assert_eq!(
        limited.error.as_deref(),
        Some(
            "ChatGPT is rate-limiting usage requests, so QuotaDial will ask again after \
             2026-10-23T12:12:00Z."
        )
    );

    harness.advance(5);
    let still = provider.fetch(&account).await.expect("replays");
    assert_eq!(still.status, Status::RateLimited);
    assert_eq!(harness.usage_requests().await, 1);
}

#[tokio::test]
async fn a_server_error_shows_the_last_numbers_as_stale() {
    let harness = Harness::new().await;
    harness.sign_in("pro", FAR_FUTURE);
    harness.mount_usage(usage_ok()).await;
    let provider = harness.provider();
    let account = discovered(&provider);
    provider.fetch(&account).await.expect("first fetch");

    harness.server.reset().await;
    harness
        .mount_usage(
            ResponseTemplate::new(503).set_body_json(json!({ "detail": "Service unavailable" })),
        )
        .await;
    harness.advance(3);
    let stale = provider.fetch(&account).await.expect("stale fetch");

    assert_eq!(stale.status, Status::Stale);
    assert_eq!(stale.windows[0].source, Source::Cache);
    assert_eq!(
        stale.error.as_deref(),
        Some(
            "ChatGPT's usage endpoint answered HTTP 503: Service unavailable. Showing the last \
             numbers fetched; QuotaDial asks again after 2026-10-23T12:05:00Z."
        )
    );
}

#[tokio::test]
async fn a_server_error_with_nothing_cached_is_an_error_and_still_holds_the_floor() {
    let harness = Harness::new().await;
    harness.sign_in("pro", FAR_FUTURE);
    harness.mount_usage(ResponseTemplate::new(500)).await;
    let provider = harness.provider();
    let account = discovered(&provider);

    let first = provider.fetch(&account).await;
    assert_eq!(
        first.expect_err("fails").to_string(),
        "codex: request failed: ChatGPT's usage endpoint answered HTTP 500."
    );

    harness.advance(1);
    let second = provider.fetch(&account).await;
    assert_eq!(
        second.expect_err("waits").to_string(),
        "codex: request failed: ChatGPT's usage endpoint answered HTTP 500. QuotaDial asks \
         ChatGPT at most every 2 minutes, so the next request is after 2026-10-23T12:02:00Z."
    );
    assert_eq!(harness.usage_requests().await, 1);
}

#[tokio::test]
async fn an_unreachable_chatgpt_shows_stale_numbers_without_using_up_the_floor() {
    let harness = Harness::new().await;
    harness.sign_in("pro", FAR_FUTURE);
    harness.mount_usage(usage_ok()).await;
    let provider = harness.provider();
    let account = discovered(&provider);
    provider.fetch(&account).await.expect("first fetch");

    // A port with nothing listening on it.
    let closed = TcpListener::bind("127.0.0.1:0").expect("binds");
    let dead_url = format!("http://{}", closed.local_addr().expect("address"));
    drop(closed);
    harness.advance(3);
    let offline = harness.provider_with(&dead_url);
    let stale = offline.fetch(&account).await.expect("stale fetch");
    assert_eq!(stale.status, Status::Stale);
    assert!(
        stale
            .error
            .as_deref()
            .is_some_and(|error| error.starts_with("Could not reach ChatGPT: "))
    );

    // The unanswered attempt did not count, so the next one goes out at once.
    let back = provider.fetch(&account).await.expect("fetches");
    assert_eq!(back.status, Status::Ok);
    assert_eq!(harness.usage_requests().await, 2);
}

#[tokio::test]
async fn a_request_that_times_out_still_counts_against_the_floor() {
    let harness = Harness::new().await;
    harness.sign_in("pro", FAR_FUTURE);
    harness
        .mount_usage(usage_ok().set_delay(std::time::Duration::from_secs(2)))
        .await;
    let provider = harness
        .provider()
        .with_request_timeout(std::time::Duration::from_millis(200));
    let account = discovered(&provider);

    let first = provider.fetch(&account).await;
    assert!(
        first
            .expect_err("times out")
            .to_string()
            .starts_with("codex: request failed: ChatGPT did not answer: no answer within")
    );

    harness.advance(1);
    assert!(
        provider
            .fetch(&account)
            .await
            .expect_err("waits")
            .to_string()
            .ends_with("so the next request is after 2026-10-23T12:02:00Z.")
    );
}

#[tokio::test]
async fn another_account_signed_in_is_never_blended_with_this_one() {
    let harness = Harness::new().await;
    harness.sign_in("pro", FAR_FUTURE);
    harness.mount_usage(usage_ok()).await;
    let provider = harness.provider();
    let account = discovered(&provider);

    let other = auth_json_for("user-000000000000000000000009", "pro", FAR_FUTURE);
    fs::write(harness.auth_file(), other).expect("writes auth.json");
    set_modified(&harness.auth_file(), ts("2026-10-23T08:00:00Z"));
    // A session of the account signed in now, which must not stand in for the old one either.
    harness.write_log(
        &ROLLOUT_FIXTURE.replace(USER_ID, "user-000000000000000000000009"),
        "2026-10-23T09:15:02Z",
    );
    let snapshot = provider.fetch(&account).await.expect("snapshot");

    assert_eq!(snapshot.status, Status::AuthRequired);
    assert_eq!(snapshot.error.as_deref(), Some(advice::DIFFERENT_ACCOUNT));
    assert_eq!(harness.usage_requests().await, 0);
    assert_eq!(provider.fallback(&account), None);
    let current = discovered(&provider);
    assert!(
        provider.fallback(&current).is_some(),
        "the log is the current account's"
    );
}

#[tokio::test]
async fn one_user_in_two_workspaces_is_two_accounts() {
    let harness = Harness::new().await;
    harness.sign_in("pro", FAR_FUTURE);
    let provider = harness.provider();
    let personal = discovered(&provider);

    let team =
        auth_json("team", FAR_FUTURE).replace(WORKSPACE, "00000000-0000-0000-0000-000000000005");
    fs::write(harness.auth_file(), team).expect("writes auth.json");
    let work = discovered(&provider);

    assert_eq!(personal.account_id, work.account_id, "the same user");
    assert_ne!(personal, work);
    assert_ne!(
        quotadial_poll::account_key(&personal),
        quotadial_poll::account_key(&work)
    );
    let snapshot = provider.fetch(&personal).await.expect("snapshot");
    assert_eq!(snapshot.error.as_deref(), Some(advice::DIFFERENT_ACCOUNT));
}

#[tokio::test]
async fn signing_out_after_discovery_asks_for_a_sign_in() {
    let harness = Harness::new().await;
    harness.sign_in("pro", FAR_FUTURE);
    let provider = harness.provider();
    let account = discovered(&provider);
    fs::remove_file(harness.auth_file()).expect("signs out");

    let snapshot = provider.fetch(&account).await.expect("snapshot");

    assert_eq!(snapshot.status, Status::AuthRequired);
    assert_eq!(snapshot.error.as_deref(), Some(advice::SIGNED_OUT));
}

#[tokio::test]
async fn a_token_quoted_in_chatgpts_error_is_never_shown_or_saved() {
    let harness = Harness::new().await;
    harness.sign_in("pro", FAR_FUTURE);
    let token = harness.token();
    harness
        .mount_usage(
            ResponseTemplate::new(403)
                .set_body_json(json!({ "detail": format!("Token {token} is not allowed here.") })),
        )
        .await;
    let provider = harness.provider();
    let account = discovered(&provider);

    let failed = provider.fetch(&account).await;

    assert_no_token(&printed(&failed), &token);
    assert_eq!(
        failed.expect_err("fails").to_string(),
        "codex: request failed: ChatGPT's usage endpoint answered HTTP 403: Token [redacted \
         token] is not allowed here."
    );
    assert_no_token(&harness.cache_contents(), &token);
}

#[tokio::test]
async fn a_damaged_auth_file_is_an_error_that_names_the_file_but_not_its_contents() {
    let harness = Harness::new().await;
    fs::write(
        harness.auth_file(),
        "{ \"tokens\": { \"access_token\": \"eyJ.secret",
    )
    .expect("writes");

    let error = harness.provider().discover_accounts().expect_err("damaged");

    let message = error.to_string();
    assert!(message.starts_with("codex: could not read credentials from "));
    assert!(message.ends_with("auth.json: the file is not valid JSON"));
    assert!(!message.contains("secret"));
}

#[tokio::test]
async fn a_plan_without_limits_is_unsupported_with_a_reason_instead_of_zero() {
    let harness = Harness::new().await;
    harness.sign_in("free", FAR_FUTURE);
    harness
        .mount_usage(ResponseTemplate::new(200).set_body_json(json!({
            "plan_type": "free",
            "rate_limit": null,
            "credits": { "has_credits": false, "unlimited": false, "balance": "0" }
        })))
        .await;
    let provider = harness.provider();
    let account = discovered(&provider);

    let snapshot = provider.fetch(&account).await.expect("snapshot");

    assert_eq!(snapshot.status, Status::Unsupported);
    assert_eq!(snapshot.error.as_deref(), Some(advice::NO_LIMITS));
    assert!(snapshot.windows.is_empty());
}

#[tokio::test]
async fn the_session_log_stands_in_when_chatgpt_cannot_be_asked() {
    let harness = Harness::new().await;
    harness.sign_in("pro", FAR_FUTURE);
    set_modified(&harness.auth_file(), ts("2026-10-23T08:00:00Z"));
    // The fixture's session was created by this account's workspace and user.
    let log = harness.write_log(ROLLOUT_FIXTURE, "2026-10-23T09:15:02Z");
    let provider = harness.provider();
    let account = discovered(&provider);

    let snapshot = provider
        .fallback(&account)
        .expect("a snapshot from the log");

    assert_eq!(snapshot.status, Status::Stale);
    assert_eq!(
        snapshot.error.as_deref(),
        Some(advice::from_session_log(ts("2026-10-23T09:15:00Z")).as_str())
    );
    assert_eq!(snapshot.windows.len(), 1);
    assert_eq!(snapshot.windows[0].source, Source::LocalLog);
    assert_eq!(snapshot.windows[0].used_pct, 54.0);
    assert_eq!(snapshot.windows[0].fetched_at, ts("2026-10-23T09:15:00Z"));
    assert_eq!(harness.usage_requests().await, 0);

    // Another workspace's sessions never stand in.
    fs::write(
        &log,
        ROLLOUT_FIXTURE.replace(WORKSPACE, "00000000-0000-0000-0000-000000000009"),
    )
    .expect("writes");
    set_modified(&log, ts("2026-10-23T09:15:02Z"));
    assert_eq!(provider.fallback(&account), None);
}

#[tokio::test]
async fn the_saved_files_hold_no_token() {
    let harness = Harness::new().await;
    harness.sign_in("pro", FAR_FUTURE);
    harness.mount_usage(usage_ok()).await;
    let provider = harness.provider();
    let account = discovered(&provider);

    provider.fetch(&account).await.expect("fetches");

    let saved = harness.cache_contents();
    assert!(
        saved.contains("\"used_pct\":54.0"),
        "the numbers were saved"
    );
    assert_no_token(&saved, &harness.token());
    assert!(
        !saved.contains("quotadial-test-refresh-token"),
        "the refresh token was never read"
    );
}

#[tokio::test]
async fn session_log_events_from_before_the_current_sign_in_never_stand_in() {
    let harness = Harness::new().await;
    harness.sign_in("pro", FAR_FUTURE);
    harness.write_log(ROLLOUT_FIXTURE, "2026-10-23T09:30:00Z");
    let provider = harness.provider();
    let account = discovered(&provider);

    // `codex login` at 09:20, after the log's last event: another sign-in may have written it.
    set_modified(&harness.auth_file(), ts("2026-10-23T09:20:00Z"));
    assert_eq!(provider.fallback(&account), None);

    set_modified(&harness.auth_file(), ts("2026-10-23T09:00:00Z"));
    assert!(provider.fallback(&account).is_some());
}

#[tokio::test]
async fn an_answer_for_another_account_is_never_used() {
    let harness = Harness::new().await;
    harness.sign_in("pro", FAR_FUTURE);
    let foreign = LIVE_FIXTURE.replace(
        "user-000000000000000000000001",
        "user-000000000000000000000009",
    );
    harness
        .mount_usage(ResponseTemplate::new(200).set_body_raw(foreign, "application/json"))
        .await;
    let provider = harness.provider();
    let account = discovered(&provider);

    let result = provider.fetch(&account).await;

    assert_eq!(
        result.expect_err("refused").to_string(),
        "codex: unexpected response: ChatGPT's answer names another account than the Codex CLI's \
         sign-in, so it was not used."
    );
    assert!(
        !harness.cache_contents().contains("used_pct"),
        "no numbers were saved"
    );
}

#[tokio::test]
async fn a_redirect_is_never_followed() {
    let harness = Harness::new().await;
    harness.sign_in("pro", FAR_FUTURE);
    harness
        .mount_usage(ResponseTemplate::new(302).insert_header(
            "location",
            format!("{}/elsewhere", harness.server.uri()).as_str(),
        ))
        .await;
    let provider = harness.provider();
    let account = discovered(&provider);

    let result = provider.fetch(&account).await;

    assert_eq!(
        result.expect_err("fails").to_string(),
        "codex: request failed: ChatGPT's usage endpoint answered HTTP 302."
    );
    let requests = harness.server.received_requests().await.expect("recorded");
    assert_eq!(requests.len(), 1, "only the usage request was sent");
}

#[tokio::test]
async fn an_answer_over_a_mebibyte_is_not_read() {
    let harness = Harness::new().await;
    harness.sign_in("pro", FAR_FUTURE);
    let huge = format!("{{\"padding\":\"{}\"}}", "x".repeat(1024 * 1024));
    harness
        .mount_usage(ResponseTemplate::new(200).set_body_raw(huge, "application/json"))
        .await;
    let provider = harness.provider();
    let account = discovered(&provider);

    let result = provider.fetch(&account).await;

    assert_eq!(
        result.expect_err("refused").to_string(),
        "codex: unexpected response: ChatGPT's answer could not be read: the usage response did not \
         arrive in full (the response is larger than 1 MiB)."
    );
}
