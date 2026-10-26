//! GitHub's Copilot usage endpoint, `GET https://api.github.com/copilot_internal/user`, which
//! GitHub's own Copilot clients read their limits from.
//!
//! Ported from CodexBar's `Providers/Copilot/CopilotUsageFetcher.swift` (at 4654968e9): the token
//! as `Authorization: token <token>`, `Accept: application/json` and `X-GitHub-Api-Version:
//! 2025-04-01`. QuotaDial does not send the VS Code extension's `Editor-Version`,
//! `Editor-Plugin-Version` and `User-Agent` that CodexBar adds: they would say a request came from
//! an editor it did not come from. It names itself, `quotadial/<version>`, as QuotaDial's status
//! page reader does: there is no one official CLI whose request this is to borrow a name from, as
//! D-034 and D-038 do. Whether the endpoint takes GitHub CLI's and the Copilot CLI's tokens, and
//! needs anything more, is what #51's live check records (D-127).
//!
//! Redirects are never followed, cookies are never kept and only HTTPS is spoken outside tests, so
//! the token only ever goes to `api.github.com`.
//!
//! GitHub's rate limits: a 429, or a 403 that says a limit was reached, with `Retry-After` (seconds
//! or a date) or, once `x-ratelimit-remaining` is 0, `x-ratelimit-reset` (seconds since 1970), as
//! GitHub's "Rate limits for the REST API" describes, holds requests back until that time (the
//! manager's Copilot rules on #73). A 403 with neither is GitHub refusing the request, not a limit.

use std::time::Duration;

use chrono::{DateTime, Utc};
use quotadial_core::{REDACTED, Secret, outside_text};
use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue, RETRY_AFTER, USER_AGENT};
use reqwest::{Client, Response, StatusCode, redirect};
use serde_json::Value;
use zeroize::Zeroizing;

pub(crate) const DEFAULT_BASE_URL: &str = "https://api.github.com";
const USER_PATH: &str = "/copilot_internal/user";
/// The API version GitHub's VS Code extension and CodexBar send to this endpoint.
pub(crate) const API_VERSION: &str = "2025-04-01";
/// QuotaDial's own name and version.
pub(crate) const USER_AGENT_VALUE: &str = concat!("quotadial/", env!("CARGO_PKG_VERSION"));
/// How long a request may take, start to finish.
pub(crate) const TIMEOUT: Duration = Duration::from_secs(30);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// The answer is a few kilobytes. Anything this large is not one.
const MAX_BODY_BYTES: usize = 1024 * 1024;

/// What GitHub answered to a usage request.
#[derive(Debug)]
pub(crate) enum UserResponse {
    /// 200 with a JSON object.
    Usage(Value),
    /// 401: the token was rejected.
    Unauthorized,
    /// A 429, or a 403 that says a rate limit was reached, with the raw header that says until
    /// when, if any: `Retry-After`, or `x-ratelimit-reset` once no requests remain.
    RateLimited {
        retry_after: Option<String>,
        reset: Option<String>,
    },
    /// Any other status, with GitHub's error message if it gave one.
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

/// The headers that say whether an answer is a rate limit, and until when, as sent.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Limits {
    pub(crate) retry_after: Option<String>,
    pub(crate) remaining: Option<String>,
    pub(crate) reset: Option<String>,
}

impl Limits {
    fn of(headers: &HeaderMap) -> Self {
        let text = |name: &str| {
            headers
                .get(name)
                .and_then(|value| value.to_str().ok())
                .map(str::to_owned)
        };
        Self {
            retry_after: text(RETRY_AFTER.as_str()),
            remaining: text("x-ratelimit-remaining"),
            reset: text("x-ratelimit-reset"),
        }
    }

    /// `x-ratelimit-reset`, when it is the time to wait for: no requests remain, or the answer does
    /// not say how many do.
    fn reset_to_wait_for(&self) -> Option<String> {
        let spent = self
            .remaining
            .as_deref()
            .is_none_or(|remaining| remaining.trim() == "0");
        self.reset.clone().filter(|_| spent)
    }

    /// Whether a 403 is a rate limit rather than a refusal: it says when to try again.
    fn says_limited(&self) -> bool {
        self.retry_after.is_some() || self.reset_to_wait_for().is_some()
    }
}

/// When requests may go again after a rate limit, from the headers [`UserResponse::RateLimited`]
/// carries: `Retry-After` first, then `x-ratelimit-reset`. None leaves the poll gate's own backoff.
pub(crate) fn limited_until(
    retry_after: Option<&str>,
    reset: Option<&str>,
    now: DateTime<Utc>,
) -> Option<DateTime<Utc>> {
    retry_after
        .and_then(|value| quotadial_poll::retry_after(value, now))
        .or_else(|| {
            let seconds = reset?
                .trim()
                .parse::<i64>()
                .ok()
                .filter(|seconds| *seconds > 0)?;
            DateTime::from_timestamp(seconds, 0)
        })
}

/// A client for GitHub's Copilot usage endpoint.
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
            // Tests stand in for GitHub with a plain-HTTP local server.
            .https_only(!cfg!(test))
            .build()
            .map_err(|error| format!("could not set up the HTTP client: {error}"))?;
        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_owned(),
            timeout,
        })
    }

    /// `GET /copilot_internal/user`. An `Err` means there is no answer to classify, and says
    /// whether the request was sent at all.
    pub(crate) async fn user(&self, token: &Secret) -> Result<UserResponse, UsageError> {
        let mut authorization = HeaderValue::from_str(&Zeroizing::new(format!(
            "token {}",
            token.expose()
        )))
        .map_err(|_| {
            UsageError::NotSent("the sign-in holds characters a request cannot carry".to_owned())
        })?;
        authorization.set_sensitive(true);
        let response = self
            .client
            .get(format!("{}{USER_PATH}", self.base_url))
            .header(AUTHORIZATION, authorization)
            .header(USER_AGENT, USER_AGENT_VALUE)
            .header(ACCEPT, "application/json")
            .header("X-GitHub-Api-Version", API_VERSION)
            .send()
            .await
            .map_err(|error| {
                let reason = self.transport_error(&error);
                if error.is_connect() || error.is_builder() {
                    UsageError::NotSent(reason)
                } else {
                    UsageError::NoAnswer(reason)
                }
            })?;

        let status = response.status();
        let limits = Limits::of(response.headers());
        // A limit or a rejection is decided by its status and headers, so a body that fails to
        // arrive cannot hide it.
        let body = if decided_without_body(status, &limits) {
            Ok(Vec::new())
        } else {
            self.read_body(response).await
        };
        Ok(classify(status, &limits, body, token.expose()))
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

fn decided_without_body(status: StatusCode, limits: &Limits) -> bool {
    matches!(
        status,
        StatusCode::TOO_MANY_REQUESTS | StatusCode::UNAUTHORIZED
    ) || (status == StatusCode::FORBIDDEN && limits.says_limited())
}

/// Classifies an answer to a request sent with `sent_token`. `body` is an error when the body did
/// not arrive in full.
pub(crate) fn classify(
    status: StatusCode,
    limits: &Limits,
    body: Result<Vec<u8>, String>,
    sent_token: &str,
) -> UserResponse {
    match status {
        StatusCode::UNAUTHORIZED => UserResponse::Unauthorized,
        StatusCode::TOO_MANY_REQUESTS => UserResponse::RateLimited {
            retry_after: limits.retry_after.clone(),
            reset: limits.reset_to_wait_for(),
        },
        StatusCode::FORBIDDEN if limits.says_limited() => UserResponse::RateLimited {
            retry_after: limits.retry_after.clone(),
            reset: limits.reset_to_wait_for(),
        },
        StatusCode::OK => match body {
            Ok(body) => match serde_json::from_slice::<Value>(&body) {
                Ok(value) if value.is_object() => UserResponse::Usage(value),
                _ => UserResponse::Invalid("the usage response is not a JSON object".to_owned()),
            },
            Err(reason) => UserResponse::Invalid(format!(
                "the usage response did not arrive in full ({reason})"
            )),
        },
        _ => UserResponse::Failed {
            status: status.as_u16(),
            message: error_message(&body.unwrap_or_default(), sent_token),
        },
    }
}

/// GitHub's error message, `{ "message": … }`, shortened to one line, with the token that was sent
/// and anything else token-like blanked out. The message is shown and saved, and an answer could
/// quote the token, whatever its form.
fn error_message(body: &[u8], sent_token: &str) -> Option<String> {
    let value: Value = serde_json::from_slice(body).ok()?;
    let message = value.get("message")?.as_str()?;
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

    #[test]
    fn the_usage_endpoint_is_named_in_what_quotadial_sends() {
        assert!(quotadial_core::privacy::named(DEFAULT_BASE_URL));
        assert!(quotadial_core::privacy::named(&format!(
            "{DEFAULT_BASE_URL}{USER_PATH}"
        )));
    }

    #[test]
    fn quotadial_names_itself_and_no_editor() {
        assert!(USER_AGENT_VALUE.starts_with("quotadial/"));
        assert_eq!(API_VERSION, "2025-04-01");
    }

    fn body(value: &Value) -> Result<Vec<u8>, String> {
        Ok(value.to_string().into_bytes())
    }

    fn limits(retry_after: Option<&str>, remaining: Option<&str>, reset: Option<&str>) -> Limits {
        Limits {
            retry_after: retry_after.map(str::to_owned),
            remaining: remaining.map(str::to_owned),
            reset: reset.map(str::to_owned),
        }
    }

    #[test]
    fn answers_are_classified_by_status() {
        let none = Limits::default();
        assert!(matches!(
            classify(StatusCode::OK, &none, body(&json!({ "copilot_plan": "individual" })), SENT),
            UserResponse::Usage(value) if value["copilot_plan"] == "individual"
        ));
        assert!(matches!(
            classify(StatusCode::OK, &none, body(&json!([1, 2])), SENT),
            UserResponse::Invalid(_)
        ));
        assert!(matches!(
            classify(StatusCode::OK, &none, Err("connection reset".into()), SENT),
            UserResponse::Invalid(reason) if reason.contains("did not arrive in full")
        ));
        assert!(matches!(
            classify(
                StatusCode::UNAUTHORIZED,
                &none,
                Err("never read".into()),
                SENT
            ),
            UserResponse::Unauthorized
        ));
        // GitHub's 401 body, made up in its documented shape.
        let fixture: Value = serde_json::from_str(include_str!(
            "../../../../fixtures/copilot/unauthorized.synthetic.json"
        ))
        .expect("JSON");
        assert!(matches!(
            classify(StatusCode::UNAUTHORIZED, &none, body(&fixture), SENT),
            UserResponse::Unauthorized
        ));
        assert!(matches!(
            classify(StatusCode::NOT_FOUND, &none, body(&json!({ "message": "Not Found" })), SENT),
            UserResponse::Failed { status: 404, message: Some(message) } if message == "Not Found"
        ));
        assert!(matches!(
            classify(StatusCode::BAD_GATEWAY, &none, Ok(b"<html>".to_vec()), SENT),
            UserResponse::Failed {
                status: 502,
                message: None
            }
        ));
    }

    #[test]
    fn a_429_or_a_403_that_says_when_is_a_rate_limit() {
        let limited = |status, limits: Limits| match classify(status, &limits, Ok(Vec::new()), SENT)
        {
            UserResponse::RateLimited { retry_after, reset } => Some((retry_after, reset)),
            _ => None,
        };
        let (some, no) = (|text: &str| Some(text.to_owned()), None::<String>);

        assert_eq!(
            limited(
                StatusCode::TOO_MANY_REQUESTS,
                limits(Some("60"), None, None)
            ),
            Some((some("60"), no.clone()))
        );
        assert_eq!(
            limited(
                StatusCode::TOO_MANY_REQUESTS,
                limits(None, Some("0"), Some("4102444800"))
            ),
            Some((no.clone(), some("4102444800")))
        );
        assert_eq!(
            limited(StatusCode::TOO_MANY_REQUESTS, Limits::default()),
            Some((no.clone(), no.clone())),
            "a 429 with neither header is still a limit, with QuotaDial's own backoff"
        );
        assert_eq!(
            limited(
                StatusCode::TOO_MANY_REQUESTS,
                limits(None, Some("12"), Some("4102444800"))
            ),
            Some((no.clone(), no.clone())),
            "the reset of a window with requests left is not the time to wait for"
        );
        assert_eq!(
            limited(StatusCode::FORBIDDEN, limits(Some("120"), None, None)),
            Some((some("120"), no.clone()))
        );
        assert_eq!(
            limited(
                StatusCode::FORBIDDEN,
                limits(None, Some("0"), Some("4102444800"))
            ),
            Some((no.clone(), some("4102444800")))
        );
        // A 403 with requests left and no Retry-After is GitHub refusing it, not a limit.
        assert_eq!(
            limited(
                StatusCode::FORBIDDEN,
                limits(None, Some("4999"), Some("4102444800"))
            ),
            None
        );
        assert_eq!(limited(StatusCode::FORBIDDEN, Limits::default()), None);
    }

    #[test]
    fn a_limit_lasts_until_retry_after_or_the_reset() {
        let now = DateTime::parse_from_rfc3339("2099-01-01T00:00:00Z")
            .expect("a time")
            .with_timezone(&Utc);
        let at = |seconds| DateTime::from_timestamp(seconds, 0);

        assert_eq!(
            limited_until(Some("120"), Some("4102444800"), now),
            Some(now + chrono::TimeDelta::minutes(2)),
            "Retry-After first"
        );
        assert_eq!(
            limited_until(None, Some("4102444800"), now),
            at(4_102_444_800)
        );
        assert_eq!(
            limited_until(Some("soon"), Some(" 4102444800 "), now),
            at(4_102_444_800)
        );
        assert_eq!(limited_until(None, Some("never"), now), None);
        assert_eq!(limited_until(None, Some("0"), now), None);
        assert_eq!(limited_until(None, None, now), None);
    }

    #[test]
    fn error_messages_are_one_short_line_without_tokens() {
        let fixture = include_str!("../../../../fixtures/copilot/unauthorized.synthetic.json");
        assert_eq!(
            error_message(fixture.as_bytes(), SENT).as_deref(),
            Some("Bad credentials")
        );

        let echoed = json!({ "message": format!("Token {SENT} cannot read this") });
        assert_eq!(
            error_message(echoed.to_string().as_bytes(), SENT).as_deref(),
            Some("Token [redacted token] cannot read this"),
            "the token sent is blanked whatever its form"
        );
        let shaped = format!(
            "bad {}",
            concat!("ghu_", "MadeUpGitHubAppUserTokenForQuotaDial01")
        );
        assert_eq!(
            error_message(json!({ "message": shaped }).to_string().as_bytes(), SENT).as_deref(),
            Some("bad [redacted token]")
        );
        let long = json!({ "message": format!("a\nb {}", "x".repeat(300)) });
        let message = error_message(long.to_string().as_bytes(), SENT).expect("message");
        assert!(message.starts_with("a b "));
        assert_eq!(message.chars().count(), 201);
        assert_eq!(error_message(b"not json", SENT), None);
        assert_eq!(error_message(br#"{"message":"  "}"#, SENT), None);
    }

    proptest::proptest! {
        #[test]
        fn no_answer_makes_the_classifier_panic(
            status in 100_u16..600,
            retry_after in proptest::option::of(".{0,20}"),
            remaining in proptest::option::of(".{0,8}"),
            reset in proptest::option::of(".{0,20}"),
            body in quotadial_core::testing::line(),
        ) {
            let status = StatusCode::from_u16(status).expect("a status");
            let limits = Limits { retry_after, remaining, reset };
            let _ = error_message(&body, SENT);
            if let UserResponse::RateLimited { retry_after, reset } =
                classify(status, &limits, Ok(body), SENT)
            {
                let now = DateTime::from_timestamp(4_102_444_800, 0).expect("a time");
                let _ = limited_until(retry_after.as_deref(), reset.as_deref(), now);
            }
        }
    }
}
