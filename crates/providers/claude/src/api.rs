//! Anthropic's OAuth endpoints, called the way Claude Code calls them.
//!
//! Ported from CodexBar's `Providers/Claude/ClaudeOAuth/ClaudeOAuthUsageFetcher.swift` and checked
//! against Claude Code 2.1.286, whose plain usage read is `GET /api/oauth/usage` with no query
//! string, `anthropic-beta: oauth-2025-04-20` and `User-Agent: claude-code/<version>`. QuotaDial
//! sends exactly that, so it needs none of CodexBar's `?cedar_ember=1` first attempt and retry.
//! Redirects are never followed, so the bearer token only ever goes to the configured host.

use std::process::Command;
use std::time::Duration;

use reqwest::header::{ACCEPT, CACHE_CONTROL, CONTENT_TYPE, RETRY_AFTER, USER_AGENT};
use reqwest::{Client, Response, StatusCode, redirect};
use serde_json::Value;

use crate::identity::{self, Identity};
use crate::process;
use crate::secret::Secret;

pub(crate) const DEFAULT_BASE_URL: &str = "https://api.anthropic.com";
const USAGE_PATH: &str = "/api/oauth/usage";
const PROFILE_PATH: &str = "/api/oauth/profile";
const OAUTH_BETA: &str = "oauth-2025-04-20";
/// CodexBar's stand-in when `claude --version` cannot be run.
const FALLBACK_CLAUDE_CODE_VERSION: &str = "2.1.0";
const TIMEOUT: Duration = Duration::from_secs(30);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const VERSION_TIMEOUT: Duration = Duration::from_secs(5);
/// Usage responses are a few kilobytes. Anything this large is not one.
const MAX_BODY_BYTES: usize = 1024 * 1024;

/// What Anthropic answered to a usage request.
#[derive(Debug)]
pub(crate) enum UsageResponse {
    /// 200 with a JSON body.
    Usage(Value),
    /// 401: the token was rejected.
    Unauthorized,
    /// 403 because the token lacks the `user:profile` scope.
    MissingScope,
    /// 403 because the token was revoked.
    Revoked,
    /// 429, with the raw `Retry-After` header if there was one.
    RateLimited { retry_after: Option<String> },
    /// Any other status, with Anthropic's error message if it gave one.
    Failed {
        status: u16,
        message: Option<String>,
    },
    /// 200 with a body that is not a JSON object.
    Invalid(String),
}

/// A client for Anthropic's OAuth endpoints.
#[derive(Debug)]
pub(crate) struct Api {
    client: Client,
    base_url: String,
    user_agent: String,
}

impl Api {
    pub(crate) fn new(base_url: &str, user_agent: String) -> Result<Self, String> {
        let client = Client::builder()
            .timeout(TIMEOUT)
            .connect_timeout(CONNECT_TIMEOUT)
            .redirect(redirect::Policy::none())
            .build()
            .map_err(|error| format!("could not set up the HTTP client: {error}"))?;
        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_owned(),
            user_agent,
        })
    }

    /// `GET /api/oauth/usage`. An `Err` means no answer arrived, so nothing reached Anthropic's
    /// rate limiter.
    pub(crate) async fn usage(&self, token: &Secret) -> Result<UsageResponse, String> {
        let response = self
            .client
            .get(format!("{}{USAGE_PATH}", self.base_url))
            .bearer_auth(token.expose())
            .header("anthropic-beta", OAUTH_BETA)
            .header(USER_AGENT, &self.user_agent)
            .header(ACCEPT, "application/json")
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await
            .map_err(|error| transport_error(&error))?;

        let status = response.status();
        let retry_after = response
            .headers()
            .get(RETRY_AFTER)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        let body = read_body(response).await?;
        Ok(classify(status, retry_after, &body))
    }

    /// `GET /api/oauth/profile`, used only when Claude Code's account config does not say who is
    /// signed in. Any failure leaves the account unnamed rather than failing the fetch.
    pub(crate) async fn profile(&self, token: &Secret) -> Option<Identity> {
        let response = self
            .client
            .get(format!("{}{PROFILE_PATH}", self.base_url))
            .bearer_auth(token.expose())
            .header(USER_AGENT, &self.user_agent)
            .header(CONTENT_TYPE, "application/json")
            .header(CACHE_CONTROL, "no-cache")
            .send()
            .await
            .ok()?;
        if response.status() != StatusCode::OK {
            return None;
        }
        let body = read_body(response).await.ok()?;
        identity::from_profile(&serde_json::from_slice(&body).ok()?)
    }
}

fn classify(status: StatusCode, retry_after: Option<String>, body: &[u8]) -> UsageResponse {
    match status {
        StatusCode::OK => match serde_json::from_slice::<Value>(body) {
            Ok(value) if value.is_object() => UsageResponse::Usage(value),
            _ => UsageResponse::Invalid("the usage response is not a JSON object".to_owned()),
        },
        StatusCode::UNAUTHORIZED => UsageResponse::Unauthorized,
        StatusCode::FORBIDDEN => {
            let text = String::from_utf8_lossy(body);
            if text.contains("user:profile") {
                UsageResponse::MissingScope
            } else if text.contains("OAuth token has been revoked") {
                UsageResponse::Revoked
            } else {
                UsageResponse::Failed {
                    status: status.as_u16(),
                    message: error_message(body),
                }
            }
        }
        StatusCode::TOO_MANY_REQUESTS => UsageResponse::RateLimited { retry_after },
        _ => UsageResponse::Failed {
            status: status.as_u16(),
            message: error_message(body),
        },
    }
}

/// Anthropic's `{ "error": { "message": … } }`, shortened to one line.
fn error_message(body: &[u8]) -> Option<String> {
    let value: Value = serde_json::from_slice(body).ok()?;
    let message = value.get("error")?.get("message")?.as_str()?;
    let line = message.split_whitespace().collect::<Vec<_>>().join(" ");
    if line.is_empty() {
        return None;
    }
    Some(match line.char_indices().nth(200) {
        Some((cut, _)) => format!("{}…", &line[..cut]),
        None => line,
    })
}

async fn read_body(mut response: Response) -> Result<Vec<u8>, String> {
    let mut body = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|error| transport_error(&error))?
    {
        if body.len() + chunk.len() > MAX_BODY_BYTES {
            return Err("the response is larger than 1 MiB".to_owned());
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

/// A failure with no answer: no connection, a timeout, or a broken response.
fn transport_error(error: &reqwest::Error) -> String {
    if error.is_timeout() {
        return format!("no answer within {} seconds", TIMEOUT.as_secs());
    }
    // The innermost cause says the most ("connection refused", "dns error: …"). The URL carries
    // no secret, but it adds nothing for the user either.
    let mut cause: &dyn std::error::Error = error;
    while let Some(source) = cause.source() {
        cause = source;
    }
    cause.to_string()
}

/// `claude-code/<version>`, the User-Agent Claude Code sends.
pub(crate) fn user_agent(claude_code_version: Option<&str>) -> String {
    format!(
        "claude-code/{}",
        claude_code_version.unwrap_or(FALLBACK_CLAUDE_CODE_VERSION)
    )
}

/// The installed Claude Code's version, from `claude --version`, which prints for example
/// `2.1.286 (Claude Code)`.
pub(crate) fn detect_claude_code_version() -> Option<String> {
    let mut command = Command::new("claude");
    command.arg("--version");
    let finished = process::run(command, VERSION_TIMEOUT).ok()?;
    if !finished.status.success() {
        return None;
    }
    parse_version(&String::from_utf8_lossy(&finished.stdout))
}

/// The first word of `claude --version`, if it looks like a version number. Anything else would
/// not be safe to put in a header.
fn parse_version(output: &str) -> Option<String> {
    let word = output.split_whitespace().next()?;
    let looks_like_version = word.starts_with(|c: char| c.is_ascii_digit())
        && word
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '+'));
    looks_like_version.then(|| word.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn the_user_agent_names_claude_code() {
        assert_eq!(user_agent(Some("2.1.286")), "claude-code/2.1.286");
        assert_eq!(user_agent(None), "claude-code/2.1.0");
    }

    #[test]
    fn versions_are_read_from_claude_version_output() {
        assert_eq!(
            parse_version("2.1.286 (Claude Code)\n").as_deref(),
            Some("2.1.286")
        );
        assert_eq!(
            parse_version("2.2.0-beta.1\n").as_deref(),
            Some("2.2.0-beta.1")
        );
        assert_eq!(parse_version("claude: command not found"), None);
        assert_eq!(
            parse_version("2.1.286\r\nX-Injected: yes"),
            Some("2.1.286".to_owned())
        );
        assert_eq!(parse_version("2.1\u{7f}"), None);
        assert_eq!(parse_version(""), None);
    }

    #[test]
    fn statuses_are_classified() {
        assert!(matches!(
            classify(StatusCode::OK, None, br#"{"five_hour":null}"#),
            UsageResponse::Usage(_)
        ));
        assert!(matches!(
            classify(StatusCode::OK, None, b"<html>"),
            UsageResponse::Invalid(_)
        ));
        assert!(matches!(
            classify(StatusCode::UNAUTHORIZED, None, b""),
            UsageResponse::Unauthorized
        ));
        let scope = json!({ "type": "error", "error": { "type": "permission_error",
            "message": "OAuth token does not meet scope requirement user:profile" } });
        assert!(matches!(
            classify(StatusCode::FORBIDDEN, None, scope.to_string().as_bytes()),
            UsageResponse::MissingScope
        ));
        let revoked = json!({ "error": { "message": "OAuth token has been revoked" } });
        assert!(matches!(
            classify(StatusCode::FORBIDDEN, None, revoked.to_string().as_bytes()),
            UsageResponse::Revoked
        ));
        assert!(matches!(
            classify(StatusCode::TOO_MANY_REQUESTS, Some("120".into()), b""),
            UsageResponse::RateLimited { retry_after: Some(value) } if value == "120"
        ));
        let overloaded =
            json!({ "error": { "type": "overloaded_error", "message": "Overloaded" } });
        assert!(matches!(
            classify(StatusCode::SERVICE_UNAVAILABLE, None, overloaded.to_string().as_bytes()),
            UsageResponse::Failed { status: 503, message: Some(message) } if message == "Overloaded"
        ));
    }

    #[test]
    fn error_messages_are_one_short_line() {
        let long = json!({ "error": { "message": format!("a\nb {}", "x".repeat(300)) } });
        let message = error_message(long.to_string().as_bytes()).expect("message");

        assert!(message.starts_with("a b "));
        assert!(message.ends_with('…'));
        assert_eq!(message.chars().count(), 201);
        assert_eq!(error_message(b"not json"), None);
        assert_eq!(error_message(br#"{"error":{"message":"  "}}"#), None);
    }
}
