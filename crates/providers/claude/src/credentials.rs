//! Claude Code's stored sign-in: the `claudeAiOauth` object of its credentials JSON, as kept in the
//! macOS Keychain item or in `.credentials.json`.
//!
//! Ported from CodexBar's `Providers/Claude/ClaudeOAuth/ClaudeOAuthCredentialModels.swift`. Only
//! what QuotaDial needs is kept. The refresh token is never read: refreshing is Claude Code's job.

use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::plan;
use crate::secret::Secret;

/// The scope Anthropic's usage endpoint requires.
const PROFILE_SCOPE: &str = "user:profile";

/// A token QuotaDial can send to Anthropic, with what Claude Code recorded about it.
#[derive(Debug)]
pub(crate) struct Credentials {
    pub(crate) access_token: Secret,
    pub(crate) expires_at: Option<DateTime<Utc>>,
    /// `None` when unknown, as for a token from `CLAUDE_CODE_OAUTH_TOKEN`.
    pub(crate) scopes: Option<Vec<String>>,
    subscription_type: Option<String>,
    rate_limit_tier: Option<String>,
}

/// Why a stored sign-in cannot be used. No variant carries any part of the payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ParseError {
    /// The payload is not a JSON object.
    NotJson,
    /// The payload holds no Claude sign-in: no `claudeAiOauth`, perhaps only MCP server sign-ins.
    NoClaudeLogin,
    /// `claudeAiOauth` has no access token.
    NoAccessToken,
}

impl ParseError {
    pub(crate) fn describe(self) -> &'static str {
        match self {
            Self::NotJson => "it is not valid JSON",
            Self::NoClaudeLogin => "it holds no Claude sign-in",
            Self::NoAccessToken => "its claudeAiOauth object has no access token",
        }
    }
}

impl Credentials {
    /// A token from `CLAUDE_CODE_OAUTH_TOKEN`, whose expiry and scopes are unknown.
    pub(crate) fn from_env_token(token: Secret) -> Self {
        Self {
            access_token: token,
            expires_at: None,
            scopes: None,
            subscription_type: None,
            rate_limit_tier: None,
        }
    }

    /// Parses Claude Code's credentials JSON. `security -w` prints a payload with bytes it cannot
    /// show as text in hex instead, so hex-encoded JSON is accepted too.
    pub(crate) fn parse(payload: &str) -> Result<Self, ParseError> {
        let root = json_object(payload.trim()).ok_or(ParseError::NotJson)?;
        let oauth = root
            .get("claudeAiOauth")
            .filter(|oauth| oauth.is_object())
            .ok_or(ParseError::NoClaudeLogin)?;
        let access_token = text(oauth, "accessToken").ok_or(ParseError::NoAccessToken)?;
        Ok(Self {
            access_token: Secret::new(access_token),
            expires_at: oauth.get("expiresAt").and_then(timestamp_millis),
            scopes: oauth.get("scopes").and_then(Value::as_array).map(|scopes| {
                scopes
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect()
            }),
            subscription_type: text(oauth, "subscriptionType"),
            rate_limit_tier: text(oauth, "rateLimitTier"),
        })
    }

    /// Whether the token is past the expiry Claude Code recorded. An unknown expiry is left to
    /// Anthropic to judge.
    pub(crate) fn is_expired(&self, now: DateTime<Utc>) -> bool {
        self.expires_at.is_some_and(|expires_at| expires_at <= now)
    }

    /// False only when the token's scopes are known and lack `user:profile`.
    pub(crate) fn can_read_usage(&self) -> bool {
        self.scopes
            .as_ref()
            .is_none_or(|scopes| scopes.iter().any(|scope| scope == PROFILE_SCOPE))
    }

    /// The plan Claude Code recorded for this sign-in.
    pub(crate) fn plan(&self) -> Option<String> {
        self.subscription_type
            .as_deref()
            .and_then(plan::from_subscription_type)
            .or_else(|| {
                self.rate_limit_tier
                    .as_deref()
                    .and_then(plan::from_tier)
                    .map(str::to_owned)
            })
    }
}

fn json_object(payload: &str) -> Option<Value> {
    serde_json::from_str::<Value>(payload)
        .ok()
        .or_else(|| serde_json::from_slice::<Value>(&hex_decode(payload)?).ok())
        .filter(Value::is_object)
}

fn hex_decode(text: &str) -> Option<Vec<u8>> {
    if text.is_empty()
        || !text.len().is_multiple_of(2)
        || !text.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return None;
    }
    text.as_bytes()
        .chunks(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).ok()?, 16).ok())
        .collect()
}

fn text(object: &Value, key: &str) -> Option<String> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

/// `expiresAt` is milliseconds since the Unix epoch.
fn timestamp_millis(value: &Value) -> Option<DateTime<Utc>> {
    let millis = value
        .as_f64()
        .or_else(|| value.as_str()?.trim().parse().ok())?;
    if !millis.is_finite() || millis <= 0.0 {
        return None;
    }
    // Saturates instead of wrapping for absurd values, which then fail the range check below.
    #[allow(clippy::cast_possible_truncation)]
    DateTime::from_timestamp_millis(millis as i64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const TOKEN: &str = "sk-ant-oat01-QUOTADIAL-TEST-TOKEN";

    fn payload(oauth: Value) -> String {
        json!({ "claudeAiOauth": oauth }).to_string()
    }

    fn ts(rfc3339: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(rfc3339)
            .expect("valid timestamp")
            .with_timezone(&Utc)
    }

    #[test]
    fn reads_the_fields_claude_code_stores() {
        let credentials = Credentials::parse(&payload(json!({
            "accessToken": TOKEN,
            "refreshToken": "sk-ant-ort01-QUOTADIAL-TEST-REFRESH",
            "expiresAt": 4_070_908_800_000_u64,
            "scopes": ["user:inference", "user:profile"],
            "subscriptionType": "max",
            "rateLimitTier": "default_claude_max_20x"
        })))
        .expect("parses");

        assert_eq!(credentials.access_token.expose(), TOKEN);
        assert_eq!(credentials.expires_at, Some(ts("2099-01-01T00:00:00Z")));
        assert!(credentials.can_read_usage());
        assert_eq!(credentials.plan().as_deref(), Some("max"));
        assert!(!credentials.is_expired(ts("2098-12-31T23:59:59Z")));
        assert!(credentials.is_expired(ts("2099-01-01T00:00:00Z")));
    }

    #[test]
    fn debug_output_redacts_the_token() {
        let credentials =
            Credentials::parse(&payload(json!({ "accessToken": TOKEN }))).expect("parses");

        assert!(!format!("{credentials:?}").contains(TOKEN));
    }

    #[test]
    fn a_payload_with_only_mcp_sign_ins_is_not_a_claude_login() {
        let payload = json!({ "mcpOAuth": { "server": { "accessToken": "x" } } }).to_string();

        assert_eq!(
            Credentials::parse(&payload).map(|_| ()),
            Err(ParseError::NoClaudeLogin)
        );
    }

    #[test]
    fn unusable_payloads_are_rejected_without_echoing_them() {
        assert_eq!(
            Credentials::parse(&format!("not json {TOKEN}")).map(|_| ()),
            Err(ParseError::NotJson)
        );
        assert_eq!(
            Credentials::parse(&payload(json!({ "accessToken": "  " }))).map(|_| ()),
            Err(ParseError::NoAccessToken)
        );
        assert_eq!(
            Credentials::parse("[1, 2]").map(|_| ()),
            Err(ParseError::NotJson)
        );
        for error in [
            ParseError::NotJson,
            ParseError::NoClaudeLogin,
            ParseError::NoAccessToken,
        ] {
            assert!(!error.describe().contains(TOKEN));
        }
    }

    #[test]
    fn hex_encoded_payloads_are_decoded() {
        let json = payload(json!({ "accessToken": TOKEN, "subscriptionType": "pro" }));
        let hex: String = json.bytes().map(|byte| format!("{byte:02x}")).collect();

        let credentials = Credentials::parse(&format!("{hex}\n")).expect("parses");

        assert_eq!(credentials.access_token.expose(), TOKEN);
        assert_eq!(credentials.plan().as_deref(), Some("pro"));
    }

    #[test]
    fn a_token_without_the_profile_scope_cannot_read_usage() {
        let credentials = Credentials::parse(&payload(json!({
            "accessToken": TOKEN,
            "scopes": ["user:inference"]
        })))
        .expect("parses");

        assert!(!credentials.can_read_usage());
    }

    #[test]
    fn an_environment_token_has_unknown_expiry_and_scopes() {
        let credentials = Credentials::from_env_token(Secret::new(TOKEN));

        assert!(!credentials.is_expired(ts("2099-01-01T00:00:00Z")));
        assert!(credentials.can_read_usage());
        assert_eq!(credentials.plan(), None);
    }

    #[test]
    fn the_plan_falls_back_to_the_rate_limit_tier() {
        let credentials = Credentials::parse(&payload(json!({
            "accessToken": TOKEN,
            "rateLimitTier": "default_claude_pro"
        })))
        .expect("parses");

        assert_eq!(credentials.plan().as_deref(), Some("pro"));
    }

    #[test]
    fn odd_expiry_values_are_treated_as_unknown() {
        for expires_at in [json!(null), json!(-5), json!("soon"), json!(true)] {
            let credentials = Credentials::parse(&payload(json!({
                "accessToken": TOKEN,
                "expiresAt": expires_at
            })))
            .expect("parses");
            assert_eq!(credentials.expires_at, None);
        }
        let as_text = Credentials::parse(&payload(json!({
            "accessToken": TOKEN,
            "expiresAt": "4070908800000"
        })))
        .expect("parses");
        assert_eq!(as_text.expires_at, Some(ts("2099-01-01T00:00:00Z")));
    }
}
