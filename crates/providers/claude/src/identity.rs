//! Who a Claude Code sign-in belongs to.
//!
//! Claude Code writes the signed-in account into the `oauthAccount` object of its account config
//! (`.claude.json`) when it signs in. Reading that file costs no request against Anthropic's
//! rate-limited API (decision D-024). `GET /api/oauth/profile` is the fallback for sign-ins with no
//! account config, such as `CLAUDE_CODE_OAUTH_TOKEN`. Claude Code 2.1.286 validates that response
//! as `{ account: { uuid, email }, organization: { uuid } }` and reads the plan name from
//! `organization.plan_display_name`; the other spellings accepted here are the ones CodexBar's
//! `OAuthProfileResponse` (`Providers/Claude/ClaudeOAuth/ClaudeOAuthUsageFetcher.swift`) accepts.

use std::path::Path;

use serde::Deserialize;
use serde_json::Value;

use crate::plan;

/// The account behind a sign-in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Identity {
    pub(crate) account_uuid: String,
    pub(crate) email: Option<String>,
    pub(crate) org_uuid: Option<String>,
    pub(crate) plan: Option<String>,
}

/// Only `oauthAccount` is decoded; serde skips the rest of what can be a large file.
#[derive(Deserialize)]
struct AccountConfig {
    #[serde(rename = "oauthAccount")]
    oauth_account: Option<Value>,
}

/// Reads `oauthAccount` from Claude Code's account config. A missing, unreadable or unexpected file
/// yields `None`, and the profile endpoint is asked instead.
pub(crate) fn from_account_config(path: &Path) -> Option<Identity> {
    let file = std::fs::File::open(path).ok()?;
    let config: AccountConfig = serde_json::from_reader(std::io::BufReader::new(file)).ok()?;
    let account = config.oauth_account?;
    Some(Identity {
        account_uuid: text(&account, &["accountUuid"])?,
        email: text(&account, &["emailAddress"]),
        org_uuid: text(&account, &["organizationUuid"]),
        plan: text(&account, &["organizationType"])
            .and_then(|kind| plan::from_organization_type(&kind))
            .or_else(|| {
                tier_plan(
                    &account,
                    &["organizationRateLimitTier", "userRateLimitTier"],
                )
            }),
    })
}

/// Decodes `GET /api/oauth/profile`.
pub(crate) fn from_profile(body: &Value) -> Option<Identity> {
    let account = body.get("account").filter(|value| value.is_object());
    let organization = body.get("organization").filter(|value| value.is_object());
    let email_keys = ["email", "email_address", "emailAddress"];
    Some(Identity {
        account_uuid: account
            .and_then(|account| text(account, &["uuid"]))
            .or_else(|| text(body, &["accountUuid", "account_uuid"]))?,
        email: account
            .and_then(|account| text(account, &email_keys))
            .or_else(|| text(body, &email_keys)),
        org_uuid: organization
            .and_then(|organization| text(organization, &["uuid"]))
            .or_else(|| text(body, &["organizationUuid", "organization_uuid"])),
        plan: organization.and_then(|organization| tier_plan(organization, &["plan_display_name"])),
    })
}

fn tier_plan(object: &Value, keys: &[&str]) -> Option<String> {
    text(object, keys)
        .as_deref()
        .and_then(plan::from_tier)
        .map(str::to_owned)
}

/// The first of `keys` that holds a non-blank string.
fn text(object: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        object
            .get(*key)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const PROFILE_FIXTURE: &str =
        include_str!("../../../../fixtures/claude/oauth-profile.synthetic.json");

    fn write_config(contents: &Value) -> tempfile::NamedTempFile {
        let file = tempfile::NamedTempFile::new().expect("temp file");
        std::fs::write(file.path(), contents.to_string()).expect("writes");
        file
    }

    #[test]
    fn reads_the_account_claude_code_recorded_at_sign_in() {
        let config = write_config(&json!({
            "numStartups": 42,
            "projects": { "/work": { "allowedTools": [] } },
            "oauthAccount": {
                "accountUuid": "00000000-0000-0000-0000-000000000001",
                "emailAddress": "user@example.com",
                "organizationUuid": "00000000-0000-0000-0000-000000000002",
                "organizationType": "claude_max",
                "organizationRateLimitTier": "default_claude_max_20x"
            }
        }));

        assert_eq!(
            from_account_config(config.path()),
            Some(Identity {
                account_uuid: "00000000-0000-0000-0000-000000000001".into(),
                email: Some("user@example.com".into()),
                org_uuid: Some("00000000-0000-0000-0000-000000000002".into()),
                plan: Some("max".into()),
            })
        );
    }

    #[test]
    fn the_plan_falls_back_to_the_rate_limit_tier() {
        let config = write_config(&json!({
            "oauthAccount": {
                "accountUuid": "00000000-0000-0000-0000-000000000001",
                "organizationRateLimitTier": "default_claude_pro"
            }
        }));

        let identity = from_account_config(config.path()).expect("identity");
        assert_eq!(identity.plan.as_deref(), Some("pro"));
        assert_eq!(identity.email, None);
        assert_eq!(identity.org_uuid, None);
    }

    #[test]
    fn no_usable_account_config_means_no_identity() {
        let no_account = write_config(&json!({ "numStartups": 1 }));
        let no_uuid =
            write_config(&json!({ "oauthAccount": { "emailAddress": "user@example.com" } }));
        let not_json = tempfile::NamedTempFile::new().expect("temp file");
        std::fs::write(not_json.path(), "{ not json").expect("writes");

        assert_eq!(from_account_config(no_account.path()), None);
        assert_eq!(from_account_config(no_uuid.path()), None);
        assert_eq!(from_account_config(not_json.path()), None);
        assert_eq!(
            from_account_config(Path::new("/nonexistent/quotadial/.claude.json")),
            None
        );
    }

    #[test]
    fn decodes_the_profile_fixture() {
        let body: Value = serde_json::from_str(PROFILE_FIXTURE).expect("fixture is valid JSON");

        assert_eq!(
            from_profile(&body),
            Some(Identity {
                account_uuid: "00000000-0000-0000-0000-000000000001".into(),
                email: Some("user@example.com".into()),
                org_uuid: Some("00000000-0000-0000-0000-000000000002".into()),
                plan: Some("max".into()),
            })
        );
    }

    #[test]
    fn decodes_flat_profile_spellings() {
        let body = json!({
            "account_uuid": "00000000-0000-0000-0000-000000000001",
            "email": "user@example.com",
            "organization_uuid": "00000000-0000-0000-0000-000000000002"
        });

        let identity = from_profile(&body).expect("identity");
        assert_eq!(
            identity.account_uuid,
            "00000000-0000-0000-0000-000000000001"
        );
        assert_eq!(identity.email.as_deref(), Some("user@example.com"));
        assert_eq!(
            identity.org_uuid.as_deref(),
            Some("00000000-0000-0000-0000-000000000002")
        );
        assert_eq!(from_profile(&json!({ "account": null })), None);
    }
}
