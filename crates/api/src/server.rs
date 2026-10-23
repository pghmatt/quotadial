//! The daemon's HTTP server: `GET /v1/status` and `GET /v1/headroom`, bound to `127.0.0.1` only.
//!
//! Every request must carry the bearer token and be addressed to `127.0.0.1` or `localhost` at the
//! API's own port. The token keeps other users and web pages from reading usage; the host check
//! also stops a web page that rebinds its own name to `127.0.0.1` from even reaching the token
//! check. Answers and errors are JSON.

use std::future::Future;
use std::io;
use std::net::Ipv4Addr;

use axum::Router;
use axum::extract::{Request, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Json, Response};
use axum::routing::get;
use quotadial_core::StatusReport;
use quotadial_daemon::Latest;
use serde_json::json;
use tokio::net::TcpListener;

use crate::{HeadroomReport, Token, rank};

/// The port the API listens on unless told otherwise.
pub const DEFAULT_PORT: u16 = 47_115;

#[derive(Clone)]
struct AppState {
    latest: Latest,
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

/// Serves the API on `listener` from `latest` until `shutdown` completes.
///
/// # Errors
///
/// When the listener fails.
pub async fn serve(
    listener: TcpListener,
    latest: Latest,
    token: Token,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> io::Result<()> {
    let port = listener.local_addr()?.port();
    let app = router(AppState {
        latest,
        token,
        port,
    });
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await
}

fn router(state: AppState) -> Router {
    Router::new()
        .route("/v1/status", get(status).fallback(method_not_allowed))
        .route("/v1/headroom", get(headroom).fallback(method_not_allowed))
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

async fn not_found() -> Response {
    error(
        StatusCode::NOT_FOUND,
        "QuotaDial's API has GET /v1/status and GET /v1/headroom.",
    )
}

async fn method_not_allowed() -> Response {
    error(
        StatusCode::METHOD_NOT_ALLOWED,
        "QuotaDial's API only reads; use GET.",
    )
}

fn error(status: StatusCode, message: &str) -> Response {
    (status, Json(json!({ "error": message }))).into_response()
}
