//! How the CLI and the MCP server ask the running daemon instead of polling themselves. While a
//! daemon runs it is the only thing that polls, so they ask it, and only when none is running do
//! they fetch directly, through the same gate that keeps every provider's floor.
//!
//! A running daemon holds the lock on `daemon.lock` and announces its port in `daemon.json` once
//! its API listens. The token is sent only while the lock shows a daemon running, and a starting
//! daemon deletes the `daemon.json` a crashed one left behind, so the token never goes to whatever
//! program has since taken an old port.
//!
//! A daemon can stop while it is being asked, as when the tray app quits. If no answer comes and
//! the daemon releases its lock within two seconds, nothing polls any more, so the consumer
//! fetches for itself. A daemon that keeps its lock but does not answer is reported instead.

use std::fmt;
use std::time::Duration;

use quotadial_core::StatusReport;
use quotadial_daemon::{DataDir, Instance};
use reqwest::StatusCode;
use serde::de::DeserializeOwned;

use crate::{HeadroomReport, Runtime, Token};

/// How long to wait for the daemon's answer.
const TIMEOUT: Duration = Duration::from_secs(3);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(1);
/// How long a daemon that has just started gets to announce its API.
const START_WAIT: Duration = Duration::from_secs(2);
/// How long a daemon that stopped answering gets to show it is stopping, by releasing its lock.
const STOP_WAIT: Duration = Duration::from_secs(2);
/// The fix for a daemon that does not answer.
const RESTART: &str = "restart it with `quotadial daemon`";
/// A report is a few kilobytes. Anything this large is not one.
const MAX_BODY_BYTES: usize = 4 * 1024 * 1024;

/// Why a running daemon gave no answer, naming the fix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unanswered(pub(crate) String);

impl fmt::Display for Unanswered {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for Unanswered {}

/// The running daemon's report, or `None` when no daemon is running.
///
/// # Errors
///
/// When a daemon is running but does not answer. Nothing else should poll in its place.
pub async fn status(dir: &DataDir) -> Result<Option<StatusReport>, Unanswered> {
    get(dir, "/v1/status").await
}

/// The running daemon's headroom ranking, or `None` when no daemon is running.
///
/// # Errors
///
/// When a daemon is running but does not answer. Nothing else should poll in its place.
pub async fn headroom(dir: &DataDir) -> Result<Option<HeadroomReport>, Unanswered> {
    get(dir, "/v1/headroom").await
}

async fn get<T: DeserializeOwned>(dir: &DataDir, path: &str) -> Result<Option<T>, Unanswered> {
    let Some(runtime) = announced(dir).await? else {
        return Ok(None);
    };
    let Some(token) = Token::load(dir) else {
        return Err(Unanswered(format!(
            "the QuotaDial daemon is running, but its token at {} cannot be read; {RESTART}.",
            dir.token().display()
        )));
    };
    let url = format!("http://127.0.0.1:{}{path}", runtime.port);
    let unanswered = |what: &str| {
        Unanswered(format!(
            "the QuotaDial daemon is running, but its API at {url} {what}; {RESTART}."
        ))
    };
    match ask(&url, &token, dir).await {
        Ok(answer) => Ok(Some(answer)),
        Err(Failure::BadAnswer(what)) => Err(unanswered(&what)),
        // The daemon may be stopping, as the tray app's does when it quits mid-answer. Once its
        // lock is released nothing else polls, so fetching directly keeps the floor.
        Err(Failure::NoAnswer(_)) if stopped(dir).await => Ok(None),
        Err(Failure::NoAnswer(what)) => Err(unanswered(what)),
    }
}

/// Why a request to the daemon brought no report.
enum Failure {
    /// No answer came: the connection failed, broke off or timed out.
    NoAnswer(&'static str),
    /// The daemon answered, but not with a report.
    BadAnswer(String),
}

async fn ask<T: DeserializeOwned>(url: &str, token: &Token, dir: &DataDir) -> Result<T, Failure> {
    let client = reqwest::Client::builder()
        .timeout(TIMEOUT)
        .connect_timeout(CONNECT_TIMEOUT)
        .redirect(reqwest::redirect::Policy::none())
        // A system proxy must never carry the token, or see this traffic at all.
        .no_proxy()
        .build()
        .map_err(|error| Failure::BadAnswer(format!("could not be asked: {error}")))?;
    let response = client
        .get(url)
        .bearer_auth(token.expose())
        .send()
        .await
        .map_err(|error| {
            // A refused connection can surface as a connect timeout (Windows retries a refusal for
            // longer than `CONNECT_TIMEOUT`), so both are "could not be reached".
            Failure::NoAnswer(if error.is_connect() {
                "could not be reached"
            } else if error.is_timeout() {
                "did not answer in time"
            } else {
                "stopped answering"
            })
        })?;
    match response.status() {
        StatusCode::OK => {}
        StatusCode::UNAUTHORIZED => {
            return Err(Failure::BadAnswer(format!(
                "refused the token at {}",
                dir.token().display()
            )));
        }
        status => return Err(Failure::BadAnswer(format!("answered {status}"))),
    }
    let body = response
        .bytes()
        .await
        .map_err(|_| Failure::NoAnswer("stopped answering"))?;
    if body.len() > MAX_BODY_BYTES {
        return Err(Failure::BadAnswer(
            "sent an answer too large to be a report".to_owned(),
        ));
    }
    serde_json::from_slice(&body)
        .map_err(|_| Failure::BadAnswer("sent an answer QuotaDial cannot read".to_owned()))
}

/// Whether the daemon releases its lock within [`STOP_WAIT`], as one that is stopping does.
async fn stopped(dir: &DataDir) -> bool {
    let deadline = tokio::time::Instant::now() + STOP_WAIT;
    loop {
        if !Instance::is_running(dir) {
            return true;
        }
        if tokio::time::Instant::now() >= deadline {
            return false;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

/// Where the running daemon's API listens, waiting briefly for one that is starting. `None` when
/// no daemon is running.
async fn announced(dir: &DataDir) -> Result<Option<Runtime>, Unanswered> {
    let deadline = tokio::time::Instant::now() + START_WAIT;
    loop {
        if !Instance::is_running(dir) {
            return Ok(None);
        }
        if let Some(runtime) = Runtime::read(dir) {
            return Ok(Some(runtime));
        }
        if tokio::time::Instant::now() >= deadline {
            return Err(Unanswered(format!(
                "a QuotaDial daemon is running, but it has not said where its API listens in {}; \
                 {RESTART}.",
                dir.runtime().display()
            )));
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}
