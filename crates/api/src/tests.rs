//! The server and the client together, over a real socket on `127.0.0.1`.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use quotadial_core::licence::TEST_SEED;
use quotadial_core::{
    Account, Provider, Result, Snapshot, Source, Status, StatusReport, Window, WindowKind,
};
use quotadial_core::{Licence, Plan};
use quotadial_daemon::{DataDir, History, Instance, Latest, Licensing, Poller, licence};
use reqwest::header::{ALLOW, AUTHORIZATION, CONTENT_TYPE, HOST, WWW_AUTHENTICATE};
use reqwest::{Method, StatusCode};
use serde_json::Value;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

use crate::{HeadroomReport, Host, HostError, Runtime, Token, bind, client, rank, serve};

/// The licensing of `dir` with a QuotaDial Pro key kept in it, signed with the test key a debug
/// build trusts.
pub(crate) fn pro(dir: &DataDir) -> Licensing {
    let key = Licence {
        id: "api-test".into(),
        plan: Plan::Pro,
        issued: "2026-10-24".parse().expect("a date"),
        until: None,
    }
    .sign(&TEST_SEED);
    licence::add(dir, &key, quotadial_core::licence::today()).expect("adds the key");
    Licensing::new(Some(dir))
}

/// A provider with one account at a fixed share used.
struct Fixed(f64);

#[async_trait]
impl Provider for Fixed {
    fn id(&self) -> &'static str {
        "claude"
    }

    fn name(&self) -> &'static str {
        "Claude"
    }

    fn discover_accounts(&self) -> Result<Vec<Account>> {
        Ok(vec![Account {
            provider: "claude".into(),
            provider_name: "Claude".into(),
            account_id: "user-1".into(),
            display_name: "user@example.com".into(),
            plan: Some("max".into()),
            org: None,
            sign_in: None,
            sign_in_use: "QuotaDial reads this test's sign-in.".into(),
        }])
    }

    async fn fetch(&self, account: &Account) -> Result<Snapshot> {
        Ok(Snapshot {
            account: account.clone(),
            windows: vec![Window {
                kind: WindowKind::Session,
                label: "Session".into(),
                used_pct: self.0,
                resets_at: Some(at("2099-01-01T05:00:00Z")),
                window_seconds: 18_000,
                scope_model: None,
                source: Source::Api,
                fetched_at: at("2099-01-01T00:00:00Z"),
                unconfirmed: false,
                pace: None,
            }],
            extra_usage: None,
            status: Status::Ok,
            error: None,
        })
    }

    fn fallback(&self, _account: &Account) -> Option<Snapshot> {
        None
    }

    fn min_poll_interval(&self) -> Duration {
        Duration::from_secs(5 * 60)
    }
}

fn at(text: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(text)
        .expect("valid timestamp")
        .with_timezone(&Utc)
}

/// A daemon's API, serving what one poll of [`Fixed`] found, with the daemon's lock held.
struct Daemon {
    _temp: tempfile::TempDir,
    dir: DataDir,
    instance: Instance,
    token: Token,
    port: u16,
    latest: Latest,
    stop: oneshot::Sender<()>,
    server: JoinHandle<std::io::Result<()>>,
}

/// A daemon on QuotaDial Pro.
async fn daemon() -> Daemon {
    daemon_with(true).await
}

/// A daemon on QuotaDial Pro, or on QuotaDial Free with no key.
async fn daemon_with(pro: bool) -> Daemon {
    let temp = tempfile::tempdir().expect("temp dir");
    let dir = DataDir::new(temp.path().join("quotadial"));
    let instance = Instance::acquire(&dir).expect("holds the daemon lock");
    let token = Token::load_or_create(&dir).expect("makes a token");
    let history = Arc::new(History::open(&dir.history()).expect("opens history"));
    let licensing = if pro {
        self::pro(&dir)
    } else {
        Licensing::new(Some(&dir))
    };
    let mut poller = Poller::new(
        vec![Arc::new(Fixed(40.0))],
        Arc::clone(&history),
        licensing.clone(),
    );
    poller.poll_due().await;
    let latest = poller.latest();
    let listener = bind(0).await.expect("binds");
    let address = listener.local_addr().expect("address");
    assert!(
        address.ip().is_loopback(),
        "the API listens on 127.0.0.1 only"
    );
    let port = address.port();
    Runtime {
        pid: std::process::id(),
        port,
        started_at: Utc::now(),
    }
    .write(&dir)
    .expect("writes the runtime file");
    let (stop, stopped) = oneshot::channel::<()>();
    let server = tokio::spawn(serve(
        listener,
        latest.clone(),
        history,
        token.clone(),
        licensing,
        async {
            let _ = stopped.await;
        },
    ));
    Daemon {
        _temp: temp,
        dir,
        instance,
        token,
        port,
        latest,
        stop,
        server,
    }
}

impl Daemon {
    /// Sends `method path` with `token` as the bearer, if any, and `host` as the `Host` header, if
    /// given; otherwise the client's own, `127.0.0.1:<port>`.
    async fn send(
        &self,
        method: Method,
        path: &str,
        token: Option<&str>,
        host: Option<&str>,
    ) -> (StatusCode, reqwest::header::HeaderMap, Value) {
        let client = reqwest::Client::builder()
            .no_proxy()
            .build()
            .expect("client");
        let mut request = client.request(method, format!("http://127.0.0.1:{}{path}", self.port));
        if let Some(token) = token {
            request = request.header(AUTHORIZATION, format!("Bearer {token}"));
        }
        if let Some(host) = host {
            request = request.header(HOST, host);
        }
        let response = request.send().await.expect("the server answers");
        let (status, headers) = (response.status(), response.headers().clone());
        let body = response.bytes().await.expect("a body");
        let json = serde_json::from_slice(&body).expect("the body is JSON");
        (status, headers, json)
    }

    /// Sends `GET path` with `authorization` as the whole `Authorization` header.
    async fn get_with(&self, path: &str, authorization: &str) -> StatusCode {
        reqwest::Client::builder()
            .no_proxy()
            .build()
            .expect("client")
            .get(format!("http://127.0.0.1:{}{path}", self.port))
            .header(AUTHORIZATION, authorization)
            .send()
            .await
            .expect("the server answers")
            .status()
    }

    /// Sends `method path` with the token and `body` as JSON, if any. An empty answer reads as
    /// `null`.
    async fn change(&self, method: Method, path: &str, body: Option<Value>) -> (StatusCode, Value) {
        let mut request = reqwest::Client::builder()
            .no_proxy()
            .build()
            .expect("client")
            .request(method, format!("http://127.0.0.1:{}{path}", self.port))
            .header(AUTHORIZATION, format!("Bearer {}", self.token.expose()));
        if let Some(body) = body {
            request = request
                .header(CONTENT_TYPE, "application/json")
                .body(body.to_string());
        }
        let response = request.send().await.expect("the server answers");
        let status = response.status();
        let body = response.bytes().await.expect("a body");
        let json = if body.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&body).expect("the body is JSON")
        };
        (status, json)
    }

    async fn get(&self, path: &str) -> (StatusCode, Value) {
        let (status, _, body) = self
            .send(Method::GET, path, Some(self.token.expose()), None)
            .await;
        (status, body)
    }

    /// Stops the API and returns what is left: the folder and, until it is dropped, the lock.
    async fn stop(self) -> (tempfile::TempDir, DataDir, Instance) {
        self.stop.send(()).expect("the server is running");
        self.server
            .await
            .expect("the server task finishes")
            .expect("the server stops cleanly");
        (self._temp, self.dir, self.instance)
    }
}

#[tokio::test]
async fn status_and_headroom_answer_with_the_token() {
    let daemon = daemon().await;

    let (status, body) = daemon.get("/v1/status").await;
    assert_eq!(status, StatusCode::OK);
    let report: StatusReport = serde_json::from_value(body).expect("a status report");
    assert_eq!(report, daemon.latest.report());
    assert_eq!(report.snapshots.len(), 1);

    let (status, body) = daemon.get("/v1/headroom").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["accounts"][0]["headroom_pct"], 60.0);
    let headroom: HeadroomReport = serde_json::from_value(body).expect("a headroom report");
    assert_eq!(headroom, rank(&daemon.latest.report()));

    daemon.stop().await;
}

#[tokio::test]
async fn a_missing_or_wrong_token_reads_nothing() {
    let daemon = daemon().await;
    let mut wrong = daemon.token.expose().to_owned();
    let last = wrong.pop().expect("a character");
    wrong.push(if last == '0' { '1' } else { '0' });

    let token = daemon.token.expose();
    assert_eq!(
        daemon
            .get_with("/v1/status", &format!("bearer {token}"))
            .await,
        StatusCode::OK,
        "the scheme's case does not matter"
    );
    for authorization in [
        format!("Basic {token}"),
        format!("Bearer{token}"),
        token.to_owned(),
    ] {
        assert_eq!(
            daemon.get_with("/v1/status", &authorization).await,
            StatusCode::UNAUTHORIZED,
            "{authorization}"
        );
    }

    for token in [None, Some(wrong.as_str()), Some("")] {
        for path in ["/v1/status", "/v1/headroom", "/v1/unknown"] {
            let (status, headers, body) = daemon.send(Method::GET, path, token, None).await;
            assert_eq!(status, StatusCode::UNAUTHORIZED, "{path} with {token:?}");
            assert_eq!(headers[WWW_AUTHENTICATE], "Bearer");
            assert!(body["error"].as_str().expect("an error").contains("token"));
            assert!(body.get("snapshots").is_none());
        }
    }

    daemon.stop().await;
}

#[tokio::test]
async fn requests_addressed_to_another_host_are_refused() {
    let daemon = daemon().await;
    let token = Some(daemon.token.expose());

    // A web page that points its own name at 127.0.0.1 sends its name as the host.
    for host in [
        "attacker.example".to_owned(),
        format!("attacker.example:{}", daemon.port),
        format!("127.0.0.1:{}", daemon.port.wrapping_add(1)),
    ] {
        let (status, _, body) = daemon
            .send(Method::GET, "/v1/status", token, Some(&host))
            .await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{host}");
        assert!(body.get("snapshots").is_none());
    }
    for host in [
        format!("localhost:{}", daemon.port),
        format!("LOCALHOST:{}", daemon.port),
    ] {
        let (status, _, _) = daemon
            .send(Method::GET, "/v1/status", token, Some(&host))
            .await;
        assert_eq!(status, StatusCode::OK, "{host}");
    }

    daemon.stop().await;
}

#[tokio::test]
async fn unknown_paths_and_writes_get_json_errors() {
    let daemon = daemon().await;
    let token = Some(daemon.token.expose());

    let (status, body) = daemon.get("/v1/unknown").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(
        body["error"]
            .as_str()
            .expect("an error")
            .contains("/v1/status")
    );

    let (status, headers, body) = daemon.send(Method::POST, "/v1/status", token, None).await;
    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
    assert!(headers[ALLOW].to_str().expect("text").contains("GET"));
    assert!(body["error"].as_str().expect("an error").contains("GET"));

    daemon.stop().await;
}

#[tokio::test]
async fn history_gives_each_windows_readings_since_a_time() {
    let daemon = daemon().await;

    let (status, body) = daemon.get("/v1/history?since=2098-12-31T00:00:00Z").await;
    assert_eq!(status, StatusCode::OK);
    let series = body["series"].as_array().expect("series");
    assert_eq!(series.len(), 1, "the one window Fixed reports");
    assert_eq!(series[0]["window"], "Session");
    assert_eq!(series[0]["account"], "user@example.com");
    assert_eq!(series[0]["readings"][0]["used_pct"], 40.0);

    let (status, body) = daemon
        .get("/v1/history?since=2098-12-31T00:00:00Z&window=Weekly")
        .await;
    assert_eq!(
        (status, body["series"].clone()),
        (StatusCode::OK, Value::Array(Vec::new()))
    );
    let (status, body) = daemon.get("/v1/history?since=yesterday").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        body["error"]
            .as_str()
            .expect("an error")
            .contains("RFC 3339")
    );

    daemon.stop().await;
}

#[tokio::test]
async fn alert_rules_are_listed_added_turned_off_and_removed() {
    let daemon = daemon().await;

    let (status, body) = daemon.get("/v1/alerts").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["rules"].as_array().expect("rules").len(),
        2,
        "the starting rules"
    );
    assert_eq!(body["firings"], serde_json::json!([]));

    let (status, added) = daemon
        .change(
            Method::POST,
            "/v1/alerts",
            Some(serde_json::json!({ "rule": "warn me when claude's session passes 30%" })),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(added["kind"], "threshold");
    assert_eq!(added["pct"], 30.0);
    assert_eq!(added["provider"], "claude");
    assert_eq!(added["window"], "session");
    assert_eq!(added["enabled"], true);
    let id = added["id"].as_i64().expect("an id");

    let (status, unread) = daemon
        .change(
            Method::POST,
            "/v1/alerts",
            Some(serde_json::json!({ "rule": "make it stop" })),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        unread["error"]
            .as_str()
            .expect("an error")
            .contains("warn me at 80%")
    );
    let (status, _) = daemon
        .change(
            Method::POST,
            "/v1/alerts",
            Some(serde_json::json!({ "text": 1 })),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "not a rule's JSON");

    let (status, off) = daemon
        .change(
            Method::PATCH,
            &format!("/v1/alerts/{id}"),
            Some(serde_json::json!({ "enabled": false })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(off["enabled"], false);

    let (status, gone) = daemon
        .change(Method::DELETE, &format!("/v1/alerts/{id}"), None)
        .await;
    assert_eq!((status, gone), (StatusCode::NO_CONTENT, Value::Null));
    let (status, missing) = daemon
        .change(Method::DELETE, &format!("/v1/alerts/{id}"), None)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(
        missing["error"]
            .as_str()
            .expect("an error")
            .contains("GET /v1/alerts")
    );
    let (status, _) = daemon
        .change(
            Method::PATCH,
            &format!("/v1/alerts/{id}"),
            Some(serde_json::json!({ "enabled": true })),
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, _) = daemon.change(Method::PUT, "/v1/alerts", None).await;
    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
    let (status, _, _) = daemon.send(Method::POST, "/v1/alerts", None, None).await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "changes need the token too"
    );

    daemon.stop().await;
}

#[tokio::test]
async fn the_client_asks_the_running_daemon() {
    let daemon = daemon().await;

    assert_eq!(
        client::status(&daemon.dir).await,
        Ok(Some(daemon.latest.report()))
    );
    assert_eq!(
        client::headroom(&daemon.dir).await,
        Ok(Some(rank(&daemon.latest.report())))
    );

    daemon.stop().await;
}

#[tokio::test]
async fn with_no_daemon_running_the_token_goes_nowhere() {
    let temp = tempfile::tempdir().expect("temp dir");
    let dir = DataDir::new(temp.path().join("quotadial"));
    assert_eq!(client::status(&dir).await, Ok(None), "no daemon has run");

    // A daemon that crashed left its file, and another program now listens on its port.
    drop(Instance::acquire(&dir).expect("creates the lock file"));
    Token::load_or_create(&dir).expect("makes a token");
    let squatter = std::net::TcpListener::bind("127.0.0.1:0").expect("binds");
    squatter.set_nonblocking(true).expect("non-blocking");
    Runtime {
        pid: 1,
        port: squatter.local_addr().expect("address").port(),
        started_at: Utc::now(),
    }
    .write(&dir)
    .expect("writes the runtime file");

    assert_eq!(client::status(&dir).await, Ok(None));
    assert_eq!(client::headroom(&dir).await, Ok(None));
    assert_eq!(
        squatter.accept().map(|_| ()).map_err(|error| error.kind()),
        Err(std::io::ErrorKind::WouldBlock),
        "nothing connected to the old port"
    );
}

#[tokio::test]
async fn a_running_daemon_that_does_not_answer_is_an_error_not_a_reason_to_poll() {
    let daemon = daemon().await;
    let dir = daemon.dir.clone();
    let original = std::fs::read(dir.token()).expect("reads the token");

    std::fs::write(dir.token(), "0".repeat(64)).expect("writes another token");
    let refused = client::status(&dir).await.expect_err("refused");
    assert!(
        refused.to_string().contains("refused the token"),
        "{refused}"
    );
    assert!(!refused.to_string().contains(&"0".repeat(64)), "{refused}");

    std::fs::remove_file(dir.token()).expect("removes the token");
    let unreadable = client::status(&dir).await.expect_err("no token");
    assert!(
        unreadable.to_string().contains("cannot be read"),
        "{unreadable}"
    );
    std::fs::write(dir.token(), original).expect("restores the token");
    assert!(matches!(client::status(&dir).await, Ok(Some(_))));

    // The API has stopped but the daemon still holds its lock.
    let (_temp, dir, instance) = daemon.stop().await;
    let stopped = client::status(&dir).await.expect_err("nothing listens");
    assert!(
        stopped.to_string().contains("could not be reached"),
        "{stopped}"
    );
    assert!(
        stopped.to_string().contains("quotadial daemon"),
        "{stopped}"
    );

    // Still holding the lock, but it never said where its API listens.
    Runtime::remove(&dir);
    let silent = client::status(&dir).await.expect_err("no runtime file");
    assert!(
        silent
            .to_string()
            .contains("has not said where its API listens"),
        "{silent}"
    );

    drop(instance);
    assert_eq!(client::status(&dir).await, Ok(None), "it has stopped");
}

/// A daemon that takes a request and quits before answering, as the tray app might when the user
/// quits it. Its lock goes a moment after its connection, as a process's do.
async fn quits_mid_answer(dir: &DataDir) -> JoinHandle<()> {
    use tokio::io::AsyncReadExt;

    let instance = Instance::acquire(dir).expect("holds the daemon lock");
    Token::load_or_create(dir).expect("makes a token");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("binds");
    Runtime {
        pid: std::process::id(),
        port: listener.local_addr().expect("address").port(),
        started_at: Utc::now(),
    }
    .write(dir)
    .expect("writes the runtime file");
    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("accepts");
        let mut request = [0_u8; 4096];
        let _ = stream.read(&mut request).await;
        drop(stream);
        drop(listener);
        tokio::time::sleep(Duration::from_millis(200)).await;
        drop(instance);
    })
}

#[tokio::test]
async fn a_daemon_that_quits_mid_answer_hands_over_to_a_direct_fetch() {
    let temp = tempfile::tempdir().expect("temp dir");
    let dir = DataDir::new(temp.path().join("quotadial"));
    let quitting = quits_mid_answer(&dir).await;

    assert_eq!(
        client::status(&dir).await,
        Ok(None),
        "no daemon is left, so the consumer fetches for itself"
    );
    quitting.await.expect("the daemon quit");
}

fn fixed() -> Vec<Arc<dyn Provider>> {
    vec![Arc::new(Fixed(40.0))]
}

/// A data folder whose settings leave out the status pages and the cost scan, so a host started in
/// it never reaches the internet or reads this machine's logs.
fn offline(temp: &tempfile::TempDir) -> DataDir {
    let dir = DataDir::new(temp.path().join("quotadial"));
    dir.create().expect("creates");
    std::fs::write(
        dir.path().join("config.json"),
        r#"{"status_pages": false, "cost_scan": false}"#,
    )
    .expect("writes the settings");
    dir
}

#[tokio::test]
async fn the_cost_scan_is_on_unless_turned_off() {
    let temp = tempfile::tempdir().expect("temp dir");
    let host = Host::start(offline(&temp), fixed(), 0)
        .await
        .expect("starts");
    assert!(!host.scans_costs());
    drop(host);

    // Started, never run: nothing is polled or read.
    let temp = tempfile::tempdir().expect("temp dir");
    let dir = DataDir::new(temp.path().join("quotadial"));
    dir.create().expect("creates");
    let host = Host::start(dir.clone(), fixed(), 0).await.expect("starts");
    assert!(host.scans_costs());
    assert!(dir.costs().exists(), "its database is made");
}

#[tokio::test]
async fn a_host_serves_until_stopped_then_leaves_nothing_behind() {
    let temp = tempfile::tempdir().expect("temp dir");
    let dir = offline(&temp);
    let host = Host::start(dir.clone(), fixed(), 0).await.expect("starts");
    let (port, latest) = (host.port(), host.latest());
    assert_eq!(Runtime::read(&dir).map(|runtime| runtime.port), Some(port));
    assert!(Instance::is_running(&dir));
    let (stop, stopped) = oneshot::channel::<()>();
    let running = tokio::spawn(host.run(async {
        let _ = stopped.await;
    }));

    // Discovery shows the account before its first poll does, so wait for the numbers.
    let mut changes = latest.subscribe();
    while latest.report().snapshots.is_empty() {
        changes.changed().await.expect("the poller runs");
    }
    let answered = client::status(&dir)
        .await
        .expect("answers")
        .expect("a report");
    assert_eq!(answered.snapshots.len(), 1);

    stop.send(()).expect("running");
    running.await.expect("joins").expect("stops cleanly");

    assert_eq!(Runtime::read(&dir), None, "daemon.json is gone");
    assert!(!Instance::is_running(&dir), "the lock is released");
    assert_eq!(client::status(&dir).await, Ok(None));
    Host::start(dir, fixed(), 0)
        .await
        .expect("a new daemon can start");
}

#[tokio::test]
async fn a_damaged_history_is_moved_aside_as_the_daemon_starts_and_the_report_says_so() {
    let temp = tempfile::tempdir().expect("temp dir");
    let dir = offline(&temp);
    std::fs::write(dir.history(), b"a sync tool's conflict copy").expect("writes");

    let host = Host::start(dir.clone(), fixed(), 0).await.expect("starts");

    let recovered = host.recovered().expect("moved aside").clone();
    assert!(recovered.moved_to.exists());
    assert!(
        recovered
            .moved_to
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("history-damaged-")),
        "{}",
        recovered.moved_to.display()
    );
    let problems = host.latest().report().problems;
    assert_eq!(problems.len(), 1);
    assert!(
        problems[0].starts_with(
            "QuotaDial could not read its history, so it started a new one with the starting \
             alert rules; the old file is "
        ),
        "{}",
        problems[0]
    );
    assert_eq!(host.history().rules().expect("reads").len(), 2);
}

#[tokio::test]
async fn after_a_stop_that_was_not_clean_every_page_is_checked() {
    let temp = tempfile::tempdir().expect("temp dir");
    let dir = offline(&temp);
    // A daemon that crashed: its daemon.json is still there.
    Runtime {
        pid: 1,
        port: 9,
        started_at: chrono::Utc::now(),
    }
    .write(&dir)
    .expect("writes");
    {
        let history = quotadial_daemon::History::open(&dir.history()).expect("opens");
        history
            .add_rule(
                "warn me at 50%",
                chrono::Utc::now(),
                quotadial_core::Tier::Pro,
            )
            .expect("adds");
    }
    // Damage the last page, which opening the history does not read.
    {
        use std::io::{Seek, SeekFrom, Write};
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .open(dir.history())
            .expect("opens");
        let length = file.metadata().expect("metadata").len();
        file.seek(SeekFrom::Start(length - 4096)).expect("seeks");
        file.write_all(&[0x5a; 4096]).expect("writes");
    }
    assert!(
        quotadial_daemon::History::open(&dir.history()).is_ok(),
        "the damage does not show on opening"
    );

    let host = Host::start(dir.clone(), fixed(), 0).await.expect("starts");

    let recovered = host.recovered().expect("found by the check");
    assert_eq!(recovered.rules, 3, "the starting rules and the one added");
    host.history().check_pages().expect("a sound history");
}

#[tokio::test]
async fn questions_asked_while_a_host_stops_get_an_answer_or_a_handover() {
    let temp = tempfile::tempdir().expect("temp dir");
    let dir = offline(&temp);
    let host = Host::start(dir.clone(), fixed(), 0).await.expect("starts");
    let (stop, stopped) = oneshot::channel::<()>();
    let running = tokio::spawn(host.run(async {
        let _ = stopped.await;
    }));
    let questions: Vec<_> = (0..20_u64)
        .map(|i| {
            let dir = dir.clone();
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(10 * i)).await;
                client::status(&dir).await
            })
        })
        .collect();

    tokio::time::sleep(Duration::from_millis(80)).await;
    stop.send(()).expect("running");
    running.await.expect("joins").expect("stops cleanly");

    let mut answers = Vec::new();
    for question in questions {
        let answer = question.await.expect("joins");
        assert!(
            answer.is_ok(),
            "never an error while the daemon stops: {answer:?}"
        );
        answers.push(answer.expect("checked"));
    }
    assert!(answers[0].is_some(), "the first question had an answer");
    assert!(
        answers[answers.len() - 1].is_none(),
        "the last was handed over to a direct fetch"
    );
}

#[test]
fn a_release_keeps_its_port_and_the_example_build_takes_another() {
    if quotadial_core::copy::EXAMPLE {
        assert_eq!(crate::DEFAULT_PORT, 47_116, "never a real copy's port");
    } else {
        assert_eq!(
            crate::DEFAULT_PORT,
            47_115,
            "the port docs/CLI.md and the skill name"
        );
    }
}

#[tokio::test]
async fn a_second_host_and_a_taken_port_are_refused_with_the_reason() {
    let temp = tempfile::tempdir().expect("temp dir");
    let dir = offline(&temp);
    let first = Host::start(dir.clone(), fixed(), 0).await.expect("starts");

    let second = Host::start(dir.clone(), fixed(), 0)
        .await
        .expect_err("one daemon at a time");
    assert!(matches!(second, HostError::AlreadyRunning));
    assert_eq!(second.to_string(), "a QuotaDial daemon is already running");
    drop(first);

    let taken = std::net::TcpListener::bind("127.0.0.1:0").expect("binds");
    let port = taken.local_addr().expect("address").port();
    let refused = Host::start(dir.clone(), fixed(), port)
        .await
        .expect_err("the port is taken");
    assert!(matches!(refused, HostError::Listen { .. }), "{refused}");
    assert!(
        refused
            .to_string()
            .starts_with(&format!("could not listen on 127.0.0.1:{port}: ")),
        "{refused}"
    );
    assert!(!Instance::is_running(&dir), "a failed start keeps no lock");
    assert_eq!(Runtime::read(&dir), None);
}

#[tokio::test]
async fn on_free_headroom_and_the_alert_rules_answer_403_and_status_still_reads() {
    let daemon = daemon_with(false).await;

    for (path, feature, line) in [
        (
            "/v1/headroom",
            "headroom",
            "Headroom through the local API and MCP is part of QuotaDial Pro: quotadial.com/pro",
        ),
        (
            "/v1/alerts",
            "rules_api",
            "Alert rules through the local API and MCP are part of QuotaDial Pro: \
             quotadial.com/pro",
        ),
    ] {
        let (status, body) = daemon.get(path).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{path}");
        assert_eq!(body["pro_feature"], feature);
        assert_eq!(body["error"], line);
    }
    let (status, body) = daemon
        .change(
            Method::POST,
            "/v1/alerts",
            Some(serde_json::json!({"rule": "warn me at 80%"})),
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["pro_feature"], "rules_api");
    let (status, _) = daemon.get("/v1/status").await;
    assert_eq!(status, StatusCode::OK, "read-only status on any copy");
    let (status, _) = daemon.get("/v1/history").await;
    assert_eq!(status, StatusCode::OK, "and history");
    assert_eq!(
        daemon.get_with("/v1/headroom", "Bearer wrong").await,
        StatusCode::UNAUTHORIZED,
        "the token is asked for before anything is said about Pro"
    );
    daemon.stop().await;
}

#[tokio::test]
async fn the_api_answers_as_its_openapi_description_says() {
    let spec: Value =
        serde_json::from_str(include_str!("../../../docs/openapi.json")).expect("the spec is JSON");
    for pro in [true, false] {
        let daemon = daemon_with(pro).await;
        let copy = if pro {
            "QuotaDial Pro"
        } else {
            "QuotaDial Free"
        };
        for (path, operations) in spec["paths"].as_object().expect("paths") {
            // Rule 1 is one of the rules a new history starts with.
            let url = path.replace("{id}", "1");
            for method in ["GET", "POST", "PUT", "PATCH", "DELETE"] {
                let method_name = method.to_lowercase();
                let (status, _) = daemon
                    .change(
                        Method::from_bytes(method.as_bytes()).expect("a method"),
                        &url,
                        None,
                    )
                    .await;
                match operations.get(&method_name) {
                    Some(operation) => assert!(
                        operation["responses"].get(status.as_str()).is_some(),
                        "{copy}: {method} {path} answered {status}, which the spec does not list"
                    ),
                    None => assert_eq!(
                        status,
                        StatusCode::METHOD_NOT_ALLOWED,
                        "{copy}: {method} {path} is not in the spec"
                    ),
                }
            }
        }
        let (status, _) = daemon.change(Method::GET, "/v1/nothing", None).await;
        assert_eq!(
            status,
            StatusCode::NOT_FOUND,
            "{copy}: a path not in the spec"
        );
        daemon.stop().await;
    }
}

#[tokio::test]
async fn the_api_serves_its_openapi_description() {
    let daemon = daemon_with(false).await;
    let response = reqwest::Client::builder()
        .no_proxy()
        .build()
        .expect("client")
        .get(format!("http://127.0.0.1:{}/v1/openapi.json", daemon.port))
        .header(AUTHORIZATION, format!("Bearer {}", daemon.token.expose()))
        .send()
        .await
        .expect("the server answers");
    assert_eq!(response.status(), StatusCode::OK, "on QuotaDial Free too");
    assert_eq!(
        response.headers()[CONTENT_TYPE],
        "application/json",
        "served as JSON"
    );
    assert_eq!(
        response.text().await.expect("a body"),
        include_str!("../../../docs/openapi.json")
    );
    let (status, _, _) = daemon
        .send(Method::GET, "/v1/openapi.json", None, None)
        .await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "it needs the token, as everything does"
    );
    daemon.stop().await;
}
