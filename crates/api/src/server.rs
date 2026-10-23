//! The daemon's HTTP server, bound to `127.0.0.1` only: `GET /v1/status`, `GET /v1/headroom`, and
//! the alert rules at `/v1/alerts` (D-093).
//!
//! Every request must carry the bearer token and be addressed to `127.0.0.1` or `localhost` at the
//! API's own port. The token keeps other users and web pages from reading usage or changing rules;
//! the host check also stops a web page that rebinds its own name to `127.0.0.1` from even reaching
//! the token check. Answers and errors are JSON.

use std::future::Future;
use std::io;
use std::net::Ipv4Addr;
use std::sync::Arc;

use axum::Router;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, Request, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Json, Response};
use axum::routing::{get, patch};
use chrono::Utc;
use quotadial_core::StatusReport;
use quotadial_daemon::{AddError, Alerts, History, HistoryError, Latest};
use serde::Deserialize;
use serde_json::json;
use tokio::net::TcpListener;

use crate::{HeadroomReport, Token, rank};

/// The port the API listens on unless told otherwise.
pub const DEFAULT_PORT: u16 = 47_115;

#[derive(Clone)]
struct AppState {
    latest: Latest,
    history: Arc<History>,
    token: Token,
    port: u16,
}

/// Listens on `127.0.0.1:port`; port 0 lets the operating system choose.
///
/// # Errors
///
/// When the port is taken or cannot be used.
pub async fn bind(port: u16) -> io::Result<TcpListener> {
    TcpListener::bind((Ipv4Addr::LOCALHOST, port)).await
}

/// Serves the API on `listener` from `latest` and `history` until `shutdown` completes.
///
/// # Errors
///
/// When the listener fails.
pub async fn serve(
    listener: TcpListener,
    latest: Latest,
    history: Arc<History>,
    token: Token,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> io::Result<()> {
    let port = listener.local_addr()?.port();
    let app = router(AppState {
        latest,
        history,
        token,
        port,
    });
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await
}

fn router(state: AppState) -> Router {
    Router::new()
        .route("/v1/status", get(status).fallback(only_reads))
        .route("/v1/headroom", get(headroom).fallback(only_reads))
        .route(
            "/v1/alerts",
            get(alerts).post(add_alert).fallback(alerts_methods),
        )
        .route(
            "/v1/alerts/{id}",
            patch(set_alert)
                .delete(remove_alert)
                .fallback(alert_methods),
        )
        .fallback(not_found)
        .layer(middleware::from_fn_with_state(state.clone(), guard))
        .with_state(state)
}

/// Lets through only requests to this API's own address that carry its token.
async fn guard(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let host = request
        .headers()
        .get(header::HOST)
        .and_then(|host| host.to_str().ok());
    let allowed = [
        format!("127.0.0.1:{}", state.port),
        format!("localhost:{}", state.port),
    ];
    if !host.is_some_and(|host| {
        allowed
            .iter()
            .any(|allowed| allowed.eq_ignore_ascii_case(host))
    }) {
        return error(
            StatusCode::FORBIDDEN,
            "QuotaDial's API answers only requests addressed to 127.0.0.1 or localhost.",
        );
    }
    // The scheme's case does not matter (RFC 9110), so `bearer` is accepted too.
    let presented = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split_once(' '))
        .filter(|(scheme, _)| scheme.eq_ignore_ascii_case("Bearer"))
        .map(|(_, token)| token.trim());
    if !presented.is_some_and(|token| state.token.matches(token)) {
        let mut response = error(
            StatusCode::UNAUTHORIZED,
            "Send the token from the `token` file in QuotaDial's data directory as \
             `Authorization: Bearer <token>`.",
        );
        response
            .headers_mut()
            .insert(header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
        return response;
    }
    next.run(request).await
}

async fn status(State(state): State<AppState>) -> Json<StatusReport> {
    Json(state.latest.report())
}

async fn headroom(State(state): State<AppState>) -> Json<HeadroomReport> {
    Json(rank(&state.latest.report()))
}

/// What `POST /v1/alerts` takes.
#[derive(Deserialize)]
struct NewRule {
    /// The rule in plain words, such as `warn me at 80%`.
    rule: String,
}

/// What `PATCH /v1/alerts/{id}` takes.
#[derive(Deserialize)]
struct RuleChange {
    enabled: bool,
}

async fn alerts(State(state): State<AppState>) -> Response {
    let history = Arc::clone(&state.history);
    match tokio::task::spawn_blocking(move || history.alerts(Utc::now())).await {
        Ok(Ok(alerts)) => Json::<Alerts>(alerts).into_response(),
        Ok(Err(error)) => unreadable(&error),
        Err(_) => stopped(),
    }
}

async fn add_alert(
    State(state): State<AppState>,
    body: Result<Json<NewRule>, JsonRejection>,
) -> Response {
    let Ok(Json(NewRule { rule })) = body else {
        return error(
            StatusCode::BAD_REQUEST,
            "Send the rule in plain words as JSON, such as {\"rule\": \"warn me at 80%\"}.",
        );
    };
    let history = Arc::clone(&state.history);
    match tokio::task::spawn_blocking(move || history.add_rule(&rule, Utc::now())).await {
        Ok(Ok(added)) => (StatusCode::CREATED, Json(added)).into_response(),
        Ok(Err(AddError::Unread(message))) => error(StatusCode::BAD_REQUEST, &message),
        Ok(Err(AddError::History(error))) => unreadable(&error),
        Err(_) => stopped(),
    }
}

async fn set_alert(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    body: Result<Json<RuleChange>, JsonRejection>,
) -> Response {
    let Ok(Json(RuleChange { enabled })) = body else {
        return error(
            StatusCode::BAD_REQUEST,
            "Send whether the rule is on as JSON, such as {\"enabled\": false}.",
        );
    };
    let history = Arc::clone(&state.history);
    match tokio::task::spawn_blocking(move || history.set_rule_enabled(id, enabled)).await {
        Ok(Ok(Some(rule))) => Json(rule).into_response(),
        Ok(Ok(None)) => no_rule(id),
        Ok(Err(error)) => unreadable(&error),
        Err(_) => stopped(),
    }
}

async fn remove_alert(State(state): State<AppState>, Path(id): Path<i64>) -> Response {
    let history = Arc::clone(&state.history);
    match tokio::task::spawn_blocking(move || history.remove_rule(id)).await {
        Ok(Ok(true)) => StatusCode::NO_CONTENT.into_response(),
        Ok(Ok(false)) => no_rule(id),
        Ok(Err(error)) => unreadable(&error),
        Err(_) => stopped(),
    }
}

async fn not_found() -> Response {
    error(
        StatusCode::NOT_FOUND,
        "QuotaDial's API has GET /v1/status, GET /v1/headroom, GET and POST /v1/alerts, and \
         PATCH and DELETE /v1/alerts/<id>.",
    )
}

async fn only_reads() -> Response {
    error(
        StatusCode::METHOD_NOT_ALLOWED,
        "This part of QuotaDial's API only reads; use GET.",
    )
}

async fn alerts_methods() -> Response {
    error(
        StatusCode::METHOD_NOT_ALLOWED,
        "Use GET to list the alert rules, or POST to add one.",
    )
}

async fn alert_methods() -> Response {
    error(
        StatusCode::METHOD_NOT_ALLOWED,
        "Use PATCH to turn this alert rule on or off, or DELETE to remove it.",
    )
}

fn no_rule(id: i64) -> Response {
    error(
        StatusCode::NOT_FOUND,
        &format!("QuotaDial has no alert rule {id}; GET /v1/alerts lists them."),
    )
}

fn unreadable(history_error: &HistoryError) -> Response {
    error(
        StatusCode::INTERNAL_SERVER_ERROR,
        &format!("QuotaDial could not use its history: {history_error}."),
    )
}

fn stopped() -> Response {
    error(
        StatusCode::INTERNAL_SERVER_ERROR,
        "QuotaDial stopped while answering; try again.",
    )
}

fn error(status: StatusCode, message: &str) -> Response {
    (status, Json(json!({ "error": message }))).into_response()
}
