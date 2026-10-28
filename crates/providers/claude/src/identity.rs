//! Who a Claude Code folder is signed in to.
//!
//! Claude Code writes the signed-in account into the `oauthAccount` object of its account config
//! (`.claude.json`) when it signs in: the account's id, address, organisation and plan. That file is
//! Claude Code's own record of the account, not its sign-in, and reading it sends nothing (D-024,
//! D-145). Nothing here asks Anthropic who an account is: the profile request is gone with the rest
//! of the sign-in reading (the manager's rule 4 on #73).

use std::path::Path;

use serde::Deserialize;
use serde_json::Value;

use crate::plan;

/// The account a Claude Code folder is signed in to, as its account file names it.
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
/// yields `None`: the folder names no account.
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

    proptest::proptest! {
        /// Claude Code's account file is text from outside: no `oauthAccount`, of any shape, makes
        /// its reader panic, and an account it names always has an id.
        #[test]
        fn no_account_file_makes_the_reader_panic(account in quotadial_core::testing::json()) {
            let file = write_config(&json!({ "numStartups": 1, "oauthAccount": account }));
            if let Some(identity) = from_account_config(file.path()) {
                proptest::prop_assert!(!identity.account_uuid.trim().is_empty());
            }
            let whole = write_config(&account);
            let _ = from_account_config(whole.path());
        }
    }
}
