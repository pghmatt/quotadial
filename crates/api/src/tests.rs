//! The server and the client together, over a real socket on `127.0.0.1`.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use quotadial_core::{
    Account, Provider, Result, Snapshot, Source, Status, StatusReport, Window, WindowKind,
};
use quotadial_daemon::{DataDir, History, Instance, Latest, Poller};
use reqwest::header::{ALLOW, AUTHORIZATION, HOST, WWW_AUTHENTICATE};
use reqwest::{Method, StatusCode};
use serde_json::Value;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

use crate::{HeadroomReport, Runtime, Token, bind, client, rank, serve};

/// A provider with one account at a fixed share used.
struct Fixed(f64);

#[async_trait]
impl Provider for Fixed {
    fn id(&self) -> &'static str {
        "claude"
    }

    fn discover_accounts(&self) -> Result<Vec<Account>> {
        Ok(vec![Account {
            provider: "claude".into(),
            account_id: "user-1".into(),
            display_name: "user@example.com".into(),
            plan: Some("max".into()),
            org: None,
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

async fn daemon() -> Daemon {
    let temp = tempfile::tempdir().expect("temp dir");
    let dir = DataDir::new(temp.path().join("quotadial"));
    let instance = Instance::acquire(&dir).expect("holds the daemon lock");
    let token = Token::load_or_create(&dir).expect("makes a token");
    let history = Arc::new(History::open(&dir.history()).expect("opens history"));
    let mut poller = Poller::new(vec![Arc::new(Fixed(40.0))], history);
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
    let server = tokio::spawn(serve(listener, latest.clone(), token.clone(), async {
        let _ = stopped.await;
    }));
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
        stopped.to_string().contains("refused the connection"),
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
