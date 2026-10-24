//! The daemon's HTTP server, bound to `127.0.0.1` only: `GET /v1/status`, `GET /v1/headroom`,
//! `GET /v1/history` (D-095), and the alert rules at `/v1/alerts` (D-093). Status and history read
//! the same on every copy; headroom and the alert rules are QuotaDial Pro's, and a Free copy
//! answers them with 403 and one line naming the feature (D-100).
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
use axum::extract::{Path, Query, Request, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Json, Response};
use axum::routing::{get, patch};
use chrono::{DateTime, TimeDelta, Utc};
use quotadial_core::{ProFeature, StatusReport};
use quotadial_daemon::{
    AddError, Alerts, History, HistoryError, Latest, Licensing, Series, SeriesQuery,
};
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
    licensing: Licensing,
}

/// Listens on `127.0.0.1:port`; port 0 lets the operating system choose.
///
/// # Errors
///
/// When the port is taken or cannot be used.
pub async fn bind(port: u16) -> io::Result<TcpListener> {
    TcpListener::bind((Ipv4Addr::LOCALHOST, port)).await
}

/// Serves the API on `listener` from `latest` and `history`, doing what `licensing` allows, until
/// `shutdown` completes.
///
/// # Errors
///
/// When the listener fails.
pub async fn serve(
    listener: TcpListener,
    latest: Latest,
    history: Arc<History>,
    token: Token,
    licensing: Licensing,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> io::Result<()> {
    let port = listener.local_addr()?.port();
    let app = router(AppState {
        latest,
        history,
        token,
        port,
        licensing,
    });
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await
}

fn router(state: AppState) -> Router {
    let pro = |feature| middleware::from_fn_with_state((state.clone(), feature), pro_only);
    Router::new()
        .route("/v1/status", get(status).fallback(only_reads))
        .route(
            "/v1/headroom",
            get(headroom)
                .fallback(only_reads)
                .route_layer(pro(ProFeature::Headroom)),
        )
        .route("/v1/history", get(history).fallback(only_reads))
        .route(
            "/v1/alerts",
            get(alerts)
                .post(add_alert)
                .fallback(alerts_methods)
                .route_layer(pro(ProFeature::RulesApi)),
        )
        .route(
            "/v1/alerts/{id}",
            patch(set_alert)
                .delete(remove_alert)
                .fallback(alert_methods)
                .route_layer(pro(ProFeature::RulesApi)),
        )
        .fallback(not_found)
        .layer(middleware::from_fn_with_state(state.clone(), guard))
        .with_state(state)
}

/// Lets a request through to a QuotaDial Pro feature only on a copy that has it. A Free copy
/// answers 403, naming the feature in `pro_feature`, with the line that says where Pro is.
async fn pro_only(
    State((state, feature)): State<(AppState, ProFeature)>,
    request: Request,
    next: Next,
) -> Response {
    if state.licensing.allows(feature) {
        return next.run(request).await;
    }
    (
        StatusCode::FORBIDDEN,
        Json(json!({ "error": feature.line(), "pro_feature": feature })),
    )
        .into_response()
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

/// What `GET /v1/history` takes: since when, from 7 days ago by default, and which provider,
/// account and window.
#[derive(Deserialize)]
struct HistoryParams {
    since: Option<String>,
    provider: Option<String>,
    account: Option<String>,
    window: Option<String>,
}

/// What `GET /v1/history` answers.
#[derive(serde::Serialize)]
struct HistoryAnswer {
    series: Vec<Series>,
}

async fn history(State(state): State<AppState>, query: Query<HistoryParams>) -> Response {
    let Query(params) = query;
    let since = match params.since.as_deref().map(DateTime::parse_from_rfc3339) {
        None => Utc::now() - TimeDelta::days(7),
        Some(Ok(since)) => since.with_timezone(&Utc),
        Some(Err(_)) => {
            return error(
                StatusCode::BAD_REQUEST,
                "since must be a time in RFC 3339, such as 2099-01-01T00:00:00Z.",
            );
        }
    };
    let query = SeriesQuery {
        since,
        provider: params.provider,
        account: params.account,
        window: params.window,
    };
    let history = Arc::clone(&state.history);
    match tokio::task::spawn_blocking(move || history.series(&query)).await {
        Ok(Ok(series)) => Json(HistoryAnswer { series }).into_response(),
        Ok(Err(error)) => unreadable(&error),
        Err(_) => stopped(),
    }
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
    let (history, tier) = (Arc::clone(&state.history), state.licensing.tier());
    match tokio::task::spawn_blocking(move || history.add_rule(&rule, Utc::now(), tier)).await {
        Ok(Ok(added)) => (StatusCode::CREATED, Json(added)).into_response(),
        Ok(Err(AddError::Unread(message))) => error(StatusCode::BAD_REQUEST, &message),
        Ok(Err(AddError::Pro)) => error(StatusCode::FORBIDDEN, &ProFeature::AlertRules.line()),
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
        "QuotaDial's API has GET /v1/status, GET /v1/headroom, GET /v1/history, GET and POST \
         /v1/alerts, and PATCH and DELETE /v1/alerts/<id>.",
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
