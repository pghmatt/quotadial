//! ChatGPT's usage endpoint, called the way the Codex CLI calls it.
//!
//! Ported from CodexBar's `Providers/Codex/CodexOAuth/CodexOAuthUsageFetcher.swift` and
//! `CodexAuthenticatedHTTPTransport.swift`: `GET https://chatgpt.com/backend-api/wham/usage` with
//! the bearer token and `ChatGPT-Account-Id`. QuotaDial sends the Codex CLI's User-Agent where
//! CodexBar sends its own name (D-038). Redirects are never followed, cookies are never kept and
//! only HTTPS is spoken outside tests, so the token only ever goes to the configured host.

use std::time::Duration;

use quotadial_core::{REDACTED, Secret, outside_text};
use reqwest::header::{ACCEPT, RETRY_AFTER, USER_AGENT};
use reqwest::{Client, Response, StatusCode, redirect};
use serde_json::Value;

pub(crate) const DEFAULT_BASE_URL: &str = "https://chatgpt.com";
const USAGE_PATH: &str = "/backend-api/wham/usage";
/// The Codex CLI's User-Agent, at the version QuotaDial was checked against (D-038).
pub(crate) const USER_AGENT_VALUE: &str = "codex_cli_rs/0.163.0";
/// How long a request may take, start to finish.
pub(crate) const TIMEOUT: Duration = Duration::from_secs(30);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// Usage responses are a few kilobytes. Anything this large is not one.
const MAX_BODY_BYTES: usize = 1024 * 1024;

/// What ChatGPT answered to a usage request.
#[derive(Debug)]
pub(crate) enum UsageResponse {
    /// 200 with a JSON object.
    Usage(Value),
    /// 401: the token was rejected.
    Unauthorized,
    /// 429, with the raw `Retry-After` header if there was one.
    RateLimited { retry_after: Option<String> },
    /// Any other status, with ChatGPT's error message if it gave one.
    Failed {
        status: u16,
        message: Option<String>,
    },
    /// 200 with a body that is not a JSON object, or that did not arrive in full.
    Invalid(String),
}

/// Why a usage request has no answer to classify.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum UsageError {
    /// No connection could be made, so the request never left this machine. It does not count
    /// against the floor.
    NotSent(String),
    /// The request was sent, but no answer came back in time or the connection broke.
    NoAnswer(String),
}

/// A client for ChatGPT's usage endpoint.
#[derive(Debug)]
pub(crate) struct Api {
    client: Client,
    base_url: String,
    timeout: Duration,
}

impl Api {
    /// `timeout` is [`TIMEOUT`] except in tests.
    pub(crate) fn new(base_url: &str, timeout: Duration) -> Result<Self, String> {
        let client = Client::builder()
            .timeout(timeout)
            .connect_timeout(CONNECT_TIMEOUT.min(timeout))
            .redirect(redirect::Policy::none())
            // Tests stand in for ChatGPT with a plain-HTTP local server.
            .https_only(!cfg!(test))
            .build()
            .map_err(|error| format!("could not set up the HTTP client: {error}"))?;
        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_owned(),
            timeout,
        })
    }

    /// `GET /backend-api/wham/usage`. An `Err` means there is no answer to classify, and says
    /// whether the request was sent at all.
    pub(crate) async fn usage(
        &self,
        token: &Secret,
        account_id: Option<&str>,
    ) -> Result<UsageResponse, UsageError> {
        let mut request = self
            .client
            .get(format!("{}{USAGE_PATH}", self.base_url))
            .bearer_auth(token.expose())
            .header(USER_AGENT, USER_AGENT_VALUE)
            .header(ACCEPT, "application/json");
        if let Some(account_id) = account_id {
            request = request.header("ChatGPT-Account-Id", account_id);
        }
        let response = request.send().await.map_err(|error| {
            let reason = self.transport_error(&error);
            if error.is_connect() || error.is_builder() {
                UsageError::NotSent(reason)
            } else {
                UsageError::NoAnswer(reason)
            }
        })?;

        let status = response.status();
        let retry_after = response
            .headers()
            .get(RETRY_AFTER)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        // A 429 or 401 is decided by its status, so a body that fails to arrive cannot hide it.
        let body = if matches!(
            status,
            StatusCode::TOO_MANY_REQUESTS | StatusCode::UNAUTHORIZED
        ) {
            Ok(Vec::new())
        } else {
            self.read_body(response).await
        };
        Ok(classify(status, retry_after, body, token.expose()))
    }

    async fn read_body(&self, mut response: Response) -> Result<Vec<u8>, String> {
        let mut body = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|error| self.transport_error(&error))?
        {
            if body.len() + chunk.len() > MAX_BODY_BYTES {
                return Err("the response is larger than 1 MiB".to_owned());
            }
            body.extend_from_slice(&chunk);
        }
        Ok(body)
    }

    /// Describes a failure that left no answer: no connection, a timeout, or a broken response.
    fn transport_error(&self, error: &reqwest::Error) -> String {
        if error.is_timeout() {
            return if error.is_connect() {
                format!(
                    "could not connect within {} seconds",
                    CONNECT_TIMEOUT.min(self.timeout).as_secs_f32()
                )
            } else {
                format!("no answer within {} seconds", self.timeout.as_secs_f32())
            };
        }
        // The innermost cause says the most ("connection refused", "dns error: …").
        let mut cause: &dyn std::error::Error = error;
        while let Some(source) = cause.source() {
            cause = source;
        }
        cause.to_string()
    }
}

/// Classifies an answer to a request sent with `sent_token`. `body` is an error when the body did
/// not arrive in full.
fn classify(
    status: StatusCode,
    retry_after: Option<String>,
    body: Result<Vec<u8>, String>,
    sent_token: &str,
) -> UsageResponse {
    match status {
        StatusCode::TOO_MANY_REQUESTS => UsageResponse::RateLimited { retry_after },
        StatusCode::UNAUTHORIZED => UsageResponse::Unauthorized,
        StatusCode::OK => match body {
            Ok(body) => match serde_json::from_slice::<Value>(&body) {
                Ok(value) if value.is_object() => UsageResponse::Usage(value),
                _ => UsageResponse::Invalid("the usage response is not a JSON object".to_owned()),
            },
            Err(reason) => UsageResponse::Invalid(format!(
                "the usage response did not arrive in full ({reason})"
            )),
        },
        _ => UsageResponse::Failed {
            status: status.as_u16(),
            message: error_message(&body.unwrap_or_default(), sent_token),
        },
    }
}

/// ChatGPT's error message, `{ "detail": … }` (or `{ "error": { "message": … } }`), shortened to
/// one line, with the token that was sent and anything else token-like blanked out. The message is
/// shown and saved, and an upstream message could quote the token, whatever its format.
fn error_message(body: &[u8], sent_token: &str) -> Option<String> {
    let value: Value = serde_json::from_slice(body).ok()?;
    let message = value
        .get("detail")
        .and_then(Value::as_str)
        .or_else(|| value.get("error")?.get("message")?.as_str())
        .or_else(|| value.get("message")?.as_str())?;
    let message = if sent_token.is_empty() {
        message.to_owned()
    } else {
        message.replace(sent_token, REDACTED)
    };
    outside_text(&message, 200)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    const SENT: &str = "opaque-token-0123456789";

    fn body(value: &Value) -> Result<Vec<u8>, String> {
        Ok(value.to_string().into_bytes())
    }

    #[test]
    fn answers_are_classified_by_status() {
        assert!(matches!(
            classify(StatusCode::OK, None, body(&json!({ "plan_type": "pro" })), SENT),
            UsageResponse::Usage(value) if value["plan_type"] == "pro"
        ));
        assert!(matches!(
            classify(StatusCode::OK, None, body(&json!([1, 2])), SENT),
            UsageResponse::Invalid(_)
        ));
        assert!(matches!(
            classify(StatusCode::OK, None, Err("connection reset".into()), SENT),
            UsageResponse::Invalid(reason) if reason.contains("did not arrive in full")
        ));
        assert!(matches!(
            classify(
                StatusCode::UNAUTHORIZED,
                None,
                Err("never read".into()),
                SENT
            ),
            UsageResponse::Unauthorized
        ));
        assert!(matches!(
            classify(StatusCode::TOO_MANY_REQUESTS, Some("60".into()), Err("never read".into()), SENT),
            UsageResponse::RateLimited { retry_after: Some(value) } if value == "60"
        ));
        assert!(matches!(
            classify(StatusCode::FORBIDDEN, None, body(&json!({ "detail": "Workspace is deactivated." })), SENT),
            UsageResponse::Failed { status: 403, message: Some(message) }
                if message == "Workspace is deactivated."
        ));
        assert!(matches!(
            classify(StatusCode::BAD_GATEWAY, None, Ok(b"<html>".to_vec()), SENT),
            UsageResponse::Failed {
                status: 502,
                message: None
            }
        ));
    }

    #[test]
    fn error_messages_are_one_short_line_without_tokens() {
        // The live 401 body for a token ChatGPT cannot parse, 2026-10-23.
        let live = json!({
            "detail": "Could not parse your authentication token. Please try signing in again."
        });
        assert_eq!(
            error_message(live.to_string().as_bytes(), SENT).as_deref(),
            Some("Could not parse your authentication token. Please try signing in again.")
        );

        let quoted =
            json!({ "error": { "message": "bad token eyJhbGciOiJIUzI1NiJ9.eyJ4IjoxfQ.c2ln" } });
        assert_eq!(
            error_message(quoted.to_string().as_bytes(), SENT).as_deref(),
            Some("bad token [redacted token]")
        );

        let long = json!({ "detail": format!("a\nb {}", "x".repeat(300)) });
        let message = error_message(long.to_string().as_bytes(), SENT).expect("message");
        assert!(message.starts_with("a b "));
        assert_eq!(message.chars().count(), 201);
        assert_eq!(error_message(b"not json", SENT), None);

        // The token that was sent is blanked whatever its format, even an opaque one.
        let echoed = json!({ "detail": format!("Token {SENT} expired") });
        assert_eq!(
            error_message(echoed.to_string().as_bytes(), SENT).as_deref(),
            Some("Token [redacted token] expired")
        );
        assert_eq!(error_message(br#"{"detail":"  "}"#, SENT), None);
    }
}
