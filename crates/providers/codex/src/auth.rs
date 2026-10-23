//! The Codex CLI's sign-in, read from `auth.json`.
//!
//! Ported from CodexBar's `Providers/Codex/CodexOAuth/CodexOAuthCredentials.swift`. The file holds
//! `auth_mode`, `OPENAI_API_KEY`, `last_refresh` and `tokens { access_token, refresh_token,
//! id_token, account_id }`, and camelCase spellings of the token fields are accepted too.
//! QuotaDial reads the access token and the ChatGPT account id, and from the JWT claims of the
//! access and ID tokens the user id, the email, the plan and when the access token expires. The
//! claims are decoded, not verified: they only name the account, and ChatGPT checks the token
//! itself. The refresh token is skipped by the parser without being copied, text holding a token
//! is wiped from memory once used, and the file is never written, because refreshing is the Codex
//! CLI's job.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, Utc};
use quotadial_core::Secret;
use serde::Deserialize;
use serde_json::{Map, Value};
use zeroize::Zeroizing;

/// The JWT claim OpenAI keeps the ChatGPT account, user and plan in.
const AUTH_CLAIM: &str = "https://api.openai.com/auth";
/// The access token's JWT claim with the user's profile, including the email.
const PROFILE_CLAIM: &str = "https://api.openai.com/profile";

/// What `auth.json` says about the sign-in.
#[derive(Debug)]
pub(crate) struct Auth {
    pub(crate) access_token: Secret,
    /// The ChatGPT account (workspace) the token acts for, sent as `ChatGPT-Account-Id`.
    pub(crate) account_id: Option<String>,
    /// The ChatGPT user.
    pub(crate) user_id: Option<String>,
    pub(crate) email: Option<String>,
    /// The ChatGPT plan, such as `plus` or `pro`.
    pub(crate) plan: Option<String>,
    /// When the access token expires, from its `exp` claim.
    pub(crate) expires_at: Option<DateTime<Utc>>,
}

/// Why `auth.json` holds no usable ChatGPT sign-in. No variant carries the file's contents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ParseError {
    NotJson,
    /// The Codex CLI uses an OpenAI API key, which has no ChatGPT plan limits.
    ApiKey,
    NoAccessToken,
}

impl ParseError {
    pub(crate) fn describe(self) -> &'static str {
        match self {
            Self::NotJson => "the file is not valid JSON; run `codex login` to sign in again",
            Self::ApiKey => "the Codex CLI is signed in with an API key",
            Self::NoAccessToken => {
                "the file holds no access token; run `codex login` to sign in again"
            }
        }
    }
}

/// The parts of `auth.json` that QuotaDial reads. Fields not named here, such as the refresh
/// token, are skipped without being copied.
#[derive(Deserialize)]
struct AuthFile {
    auth_mode: Option<String>,
    #[serde(rename = "OPENAI_API_KEY")]
    api_key: Option<Zeroizing<String>>,
    tokens: Option<Tokens>,
}

#[derive(Deserialize)]
struct Tokens {
    #[serde(alias = "accessToken")]
    access_token: Option<Zeroizing<String>>,
    #[serde(alias = "idToken")]
    id_token: Option<Zeroizing<String>>,
    #[serde(alias = "accountId")]
    account_id: Option<String>,
}

impl Auth {
    pub(crate) fn parse(contents: &str) -> Result<Self, ParseError> {
        let file: AuthFile = serde_json::from_str(contents).map_err(|_| ParseError::NotJson)?;
        let api_key_mode = file
            .auth_mode
            .as_deref()
            .is_some_and(|mode| mode.eq_ignore_ascii_case("apikey"));
        let tokens = file.tokens.as_ref();
        let access_token = tokens
            .and_then(|tokens| tokens.access_token.as_deref())
            .map(|token| token.trim())
            .filter(|token| !token.is_empty() && !api_key_mode);
        let Some(access_token) = access_token else {
            let has_api_key = file
                .api_key
                .as_deref()
                .is_some_and(|key| !key.trim().is_empty());
            return Err(if api_key_mode || has_api_key {
                ParseError::ApiKey
            } else {
                ParseError::NoAccessToken
            });
        };

        let access = claims(access_token);
        let id = tokens
            .and_then(|tokens| tokens.id_token.as_deref())
            .and_then(|token| claims(token));
        // The access token is refreshed most often, so its claims come first.
        let claimed = |pick: fn(&Map<String, Value>) -> Option<String>| {
            access
                .as_ref()
                .and_then(pick)
                .or_else(|| id.as_ref().and_then(pick))
        };
        Ok(Self {
            account_id: tokens
                .and_then(|tokens| tokens.account_id.as_deref())
                .map(str::trim)
                .filter(|id| !id.is_empty())
                .map(str::to_owned)
                .or_else(|| claimed(account_id_claim)),
            user_id: claimed(user_id_claim),
            email: id
                .as_ref()
                .and_then(|claims| text(claims, "email"))
                .or_else(|| access.as_ref().and_then(profile_email)),
            plan: claimed(|claims| auth_text(claims, "chatgpt_plan_type")),
            expires_at: access
                .as_ref()
                .and_then(|claims| claims.get("exp")?.as_i64())
                .and_then(|exp| DateTime::from_timestamp(exp, 0)),
            access_token: Secret::new(access_token),
        })
    }

    /// Whether the access token has expired at `now`. A token without an expiry counts as not
    /// expired: ChatGPT decides.
    pub(crate) fn is_expired(&self, now: DateTime<Utc>) -> bool {
        self.expires_at.is_some_and(|expires_at| expires_at <= now)
    }
}

/// A JWT's claims, or `None` for an opaque token.
fn claims(token: &str) -> Option<Map<String, Value>> {
    let mut parts = token.split('.');
    let (Some(_header), Some(payload), Some(_signature), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return None;
    };
    let bytes = URL_SAFE_NO_PAD.decode(payload.trim_end_matches('=')).ok()?;
    serde_json::from_slice(&bytes).ok()
}

fn account_id_claim(claims: &Map<String, Value>) -> Option<String> {
    text(claims, "chatgpt_account_id").or_else(|| auth_text(claims, "chatgpt_account_id"))
}

fn user_id_claim(claims: &Map<String, Value>) -> Option<String> {
    auth_text(claims, "chatgpt_user_id").or_else(|| auth_text(claims, "user_id"))
}

fn profile_email(claims: &Map<String, Value>) -> Option<String> {
    claims
        .get(PROFILE_CLAIM)?
        .as_object()
        .and_then(|profile| text(profile, "email"))
}

fn auth_text(claims: &Map<String, Value>, key: &str) -> Option<String> {
    claims
        .get(AUTH_CLAIM)?
        .as_object()
        .and_then(|auth| text(auth, key))
}

/// A non-blank string field.
fn text(map: &Map<String, Value>, key: &str) -> Option<String> {
    map.get(key)?
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

#[cfg(test)]
pub(crate) mod tests {
    use serde_json::json;

    use super::*;

    /// A JWT with `claims` and a made-up signature. Never a real token.
    pub(crate) fn jwt(claims: &Value) -> String {
        format!(
            "{}.{}.{}",
            URL_SAFE_NO_PAD.encode(r#"{"alg":"RS256","typ":"JWT"}"#),
            URL_SAFE_NO_PAD.encode(claims.to_string()),
            URL_SAFE_NO_PAD.encode("quotadial-test-signature")
        )
    }

    /// The made-up ChatGPT user the test sign-ins belong to.
    pub(crate) const USER_ID: &str = "user-000000000000000000000001";

    /// `auth.json` as the Codex CLI 0.163.0 writes it for a ChatGPT sign-in, with made-up values.
    pub(crate) fn auth_json(plan: &str, expires: i64) -> String {
        auth_json_for(USER_ID, plan, expires)
    }

    /// The same, for another user.
    pub(crate) fn auth_json_for(user_id: &str, plan: &str, expires: i64) -> String {
        let access = jwt(&json!({
            "exp": expires,
            "https://api.openai.com/auth": {
                "chatgpt_account_id": "00000000-0000-0000-0000-000000000002",
                "chatgpt_plan_type": plan,
                "chatgpt_user_id": user_id
            },
            "https://api.openai.com/profile": { "email": "user@example.com" }
        }));
        let id = jwt(&json!({
            "email": "user@example.com",
            "exp": 1,
            "https://api.openai.com/auth": {
                "chatgpt_account_id": "00000000-0000-0000-0000-000000000002",
                "chatgpt_plan_type": plan,
                "chatgpt_user_id": user_id,
                "user_id": user_id
            }
        }));
        json!({
            "auth_mode": "chatgpt",
            "OPENAI_API_KEY": null,
            "tokens": {
                "id_token": id,
                "access_token": access,
                "refresh_token": "rt.quotadial-test-refresh-token",
                "account_id": "00000000-0000-0000-0000-000000000002"
            },
            "last_refresh": "2099-01-01T00:00:00.000000Z"
        })
        .to_string()
    }

    #[test]
    fn reads_the_sign_in_the_codex_cli_writes() {
        let auth = Auth::parse(&auth_json("pro", 4_102_444_800)).expect("parses");

        assert_eq!(
            auth.account_id.as_deref(),
            Some("00000000-0000-0000-0000-000000000002")
        );
        assert_eq!(auth.user_id.as_deref(), Some(USER_ID));
        assert_eq!(auth.email.as_deref(), Some("user@example.com"));
        assert_eq!(auth.plan.as_deref(), Some("pro"));
        assert_eq!(auth.expires_at, DateTime::from_timestamp(4_102_444_800, 0));
        assert!(auth.access_token.expose().starts_with("eyJ"));
        assert!(
            !format!("{auth:?}").contains("eyJ"),
            "Debug hides the token"
        );
    }

    #[test]
    fn the_account_comes_from_the_claims_when_the_file_does_not_name_it() {
        let access = jwt(&json!({
            "https://api.openai.com/auth": {
                "chatgpt_account_id": "00000000-0000-0000-0000-000000000003",
                "chatgpt_plan_type": "plus"
            }
        }));
        let text = json!({ "tokens": { "accessToken": access } }).to_string();

        let auth = Auth::parse(&text).expect("parses");

        assert_eq!(
            auth.account_id.as_deref(),
            Some("00000000-0000-0000-0000-000000000003")
        );
        assert_eq!(auth.plan.as_deref(), Some("plus"));
        assert_eq!(auth.email, None);
        assert_eq!(auth.expires_at, None);
    }

    #[test]
    fn an_opaque_access_token_is_still_used() {
        let text = json!({ "tokens": { "access_token": "opaque-token", "account_id": "a" } });

        let auth = Auth::parse(&text.to_string()).expect("parses");

        assert_eq!(auth.access_token.expose(), "opaque-token");
        assert_eq!(auth.account_id.as_deref(), Some("a"));
        assert_eq!(auth.user_id, None);
    }

    #[test]
    fn files_without_a_chatgpt_sign_in_say_why_without_their_contents() {
        let api_key = json!({ "auth_mode": "apikey", "OPENAI_API_KEY": "sk-proj-not-real",
            "tokens": { "access_token": "leftover" } });
        assert_eq!(
            Auth::parse(&api_key.to_string()).map(|_| ()),
            Err(ParseError::ApiKey)
        );
        let key_only = json!({ "OPENAI_API_KEY": "sk-proj-not-real", "tokens": null });
        assert_eq!(
            Auth::parse(&key_only.to_string()).map(|_| ()),
            Err(ParseError::ApiKey)
        );
        assert_eq!(
            Auth::parse(r#"{"tokens":{"access_token":"  "}}"#).map(|_| ()),
            Err(ParseError::NoAccessToken)
        );
        assert_eq!(
            Auth::parse("{ not json").map(|_| ()),
            Err(ParseError::NotJson)
        );
        assert!(!ParseError::ApiKey.describe().contains("sk-"));
    }

    #[test]
    fn expiry_is_judged_against_now() {
        let auth = Auth::parse(&auth_json("plus", 4_102_444_800)).expect("parses");
        let expires_at = auth.expires_at.expect("expiry");

        assert!(!auth.is_expired(expires_at - chrono::TimeDelta::seconds(1)));
        assert!(auth.is_expired(expires_at));
    }
}
