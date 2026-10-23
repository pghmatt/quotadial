//! The one sentence QuotaDial shows when it has no numbers for a Claude account, naming the fix.

use chrono::{DateTime, SecondsFormat, Utc};

use crate::login::LoginSource;

pub(crate) const SIGNED_OUT: &str =
    "Claude Code is not signed in on this machine any more. Run `claude` to sign in.";

pub(crate) const DIFFERENT_ACCOUNT: &str = "Claude Code on this machine is now signed in to a \
     different account, so this account's usage cannot be read.";

/// The token's source holds only MCP server sign-ins.
pub(crate) fn no_claude_login(source: &LoginSource) -> String {
    format!(
        "The sign-in in {} holds no Claude account, only MCP server sign-ins. Run `claude` and \
         sign in with /login.",
        source.location()
    )
}

pub(crate) fn expired(at: DateTime<Utc>) -> String {
    format!(
        "Claude Code's sign-in expired at {}. Run `claude` once to refresh it.",
        at.to_rfc3339_opts(SecondsFormat::Secs, true)
    )
}

/// Anthropic answered 401.
pub(crate) fn rejected(source: &LoginSource) -> String {
    match source {
        LoginSource::Environment(_) => "Anthropic rejected the token in CLAUDE_CODE_OAUTH_TOKEN. \
             Replace it, or unset it to use Claude Code's own sign-in."
            .to_owned(),
        _ => "Anthropic rejected Claude Code's stored sign-in. Run `claude` to sign in again."
            .to_owned(),
    }
}

/// The token lacks the `user:profile` scope the usage endpoint requires.
pub(crate) fn missing_scope(source: &LoginSource) -> String {
    match source {
        LoginSource::Environment(_) => "The token in CLAUDE_CODE_OAUTH_TOKEN cannot read usage \
             because it lacks the user:profile scope. Unset it to use Claude Code's own sign-in."
            .to_owned(),
        _ => "Claude Code's sign-in cannot read usage because it lacks the user:profile scope. \
             Run `claude` and sign in again with /login."
            .to_owned(),
    }
}

/// Anthropic answered 403 because the token was revoked.
pub(crate) fn revoked(source: &LoginSource) -> String {
    match source {
        LoginSource::Environment(_) => "Anthropic says the token in CLAUDE_CODE_OAUTH_TOKEN was \
             revoked. Replace it, or unset it to use Claude Code's own sign-in."
            .to_owned(),
        _ => "Anthropic says Claude Code's sign-in was revoked. Run `claude` to sign in again."
            .to_owned(),
    }
}

/// The response had no limits at all. The PRD: Free accounts report nothing since August 2026,
/// and Enterprise returns no usage.
pub(crate) fn no_limits(plan: Option<&str>) -> &'static str {
    match plan {
        Some(plan) if plan.contains("free") => {
            "Claude Free plans have reported no usage limits since August 2026. Upgrade to Pro or \
             Max on claude.ai to track them."
        }
        Some(plan) if plan.contains("enterprise") => {
            "Claude Enterprise accounts report no usage limits to Claude Code's usage endpoint."
        }
        _ => "Anthropic reported no usage limits for this account.",
    }
}

/// Anthropic answered with a status QuotaDial cannot use.
pub(crate) fn failed(status: u16, message: Option<&str>) -> String {
    match message {
        Some(message) => format!(
            "Anthropic's usage endpoint answered HTTP {status}: {}.",
            message.trim_end_matches('.')
        ),
        None => format!("Anthropic's usage endpoint answered HTTP {status}."),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::secret::Secret;

    fn keychain() -> LoginSource {
        LoginSource::Keychain {
            service: "Claude Code-credentials".into(),
            account: Some("example".into()),
        }
    }

    fn environment() -> LoginSource {
        LoginSource::Environment(Secret::new("sk-ant-oat01-QUOTADIAL-TEST-TOKEN"))
    }

    #[test]
    fn every_sentence_names_a_fix() {
        let sentences = [
            SIGNED_OUT.to_owned(),
            no_claude_login(&keychain()),
            expired(DateTime::<Utc>::UNIX_EPOCH),
            rejected(&keychain()),
            rejected(&environment()),
            missing_scope(&keychain()),
            missing_scope(&environment()),
            revoked(&keychain()),
            revoked(&environment()),
            no_limits(Some("free")).to_owned(),
        ];
        for sentence in sentences {
            assert!(
                ["Run `claude`", "Replace it", "Unset it", "Upgrade"]
                    .iter()
                    .any(|fix| sentence.contains(fix)),
                "{sentence}"
            );
            assert!(!sentence.contains("sk-ant-"), "{sentence}");
        }
    }

    #[test]
    fn the_source_is_named_where_it_matters() {
        assert_eq!(
            no_claude_login(&keychain()),
            "The sign-in in macOS Keychain item \"Claude Code-credentials\" holds no Claude \
             account, only MCP server sign-ins. Run `claude` and sign in with /login."
        );
        assert!(
            no_claude_login(&LoginSource::File(PathBuf::from("/x/.credentials.json")))
                .starts_with("The sign-in in /x/.credentials.json holds")
        );
    }

    #[test]
    fn plans_without_limits_get_their_own_reason() {
        assert!(no_limits(Some("free")).starts_with("Claude Free"));
        assert!(no_limits(Some("enterprise")).starts_with("Claude Enterprise"));
        assert_eq!(
            no_limits(None),
            "Anthropic reported no usage limits for this account."
        );
    }

    #[test]
    fn failures_quote_anthropics_message_once() {
        assert_eq!(
            failed(503, Some("Overloaded.")),
            "Anthropic's usage endpoint answered HTTP 503: Overloaded."
        );
        assert_eq!(
            failed(500, None),
            "Anthropic's usage endpoint answered HTTP 500."
        );
    }
}
