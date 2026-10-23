//! The one sentence QuotaDial shows when it has no numbers for a Claude account, naming the fix.
//! A sign-in kept for another config folder gets the command for that folder (#34).

use chrono::{DateTime, SecondsFormat, Utc};
use quotadial_core::Account;

use crate::login::{Login, LoginSource};

/// The sign-in where the account was found is gone.
pub(crate) fn signed_out(login: &Login) -> String {
    format!(
        "QuotaDial no longer finds the Claude Code sign-in in {}. Run {} to sign in again.",
        login.place,
        login.sign_in_command()
    )
}

/// The sign-in where the account was found now belongs to someone else.
pub(crate) fn different_account(login: &Login) -> String {
    format!(
        "The Claude Code sign-in in {} is now for another account, so this one's usage cannot be \
         read there. To keep tracking it, sign in to it again in another folder, or run {} and \
         sign in to it with /login.",
        login.place,
        login.sign_in_command()
    )
}

/// No sign-in for the account was found, and nothing is known about where it was.
pub(crate) fn not_found(account: &Account) -> String {
    match &account.sign_in {
        Some(place) => format!(
            "QuotaDial no longer finds the Claude Code sign-in in {place}. Run `claude` with the \
             CLAUDE_CONFIG_DIR it was made with, and sign in again."
        ),
        None => "QuotaDial no longer finds this account's Claude Code sign-in on this machine. \
                 Run `claude` with the CLAUDE_CONFIG_DIR it was made with, and sign in again."
            .to_owned(),
    }
}

/// The token's source holds only MCP server sign-ins.
pub(crate) fn no_claude_login(login: &Login) -> String {
    format!(
        "The sign-in in {} holds no Claude account, only MCP server sign-ins. Run {} and sign in \
         with /login.",
        login.place,
        login.sign_in_command()
    )
}

pub(crate) fn expired(login: &Login, at: DateTime<Utc>) -> String {
    format!(
        "This Claude Code sign-in expired at {}. Run {} once to refresh it.",
        at.to_rfc3339_opts(SecondsFormat::Secs, true),
        login.sign_in_command()
    )
}

/// Anthropic answered 401.
pub(crate) fn rejected(login: &Login) -> String {
    match login.source {
        LoginSource::Environment(_) => "Anthropic rejected the token in CLAUDE_CODE_OAUTH_TOKEN. \
             Replace it, or unset it to use Claude Code's own sign-in."
            .to_owned(),
        _ => format!(
            "Anthropic rejected this Claude Code sign-in. Run {} to sign in again.",
            login.sign_in_command()
        ),
    }
}

/// The token lacks the `user:profile` scope the usage endpoint requires.
pub(crate) fn missing_scope(login: &Login) -> String {
    match login.source {
        LoginSource::Environment(_) => "The token in CLAUDE_CODE_OAUTH_TOKEN cannot read usage \
             because it lacks the user:profile scope. Unset it to use Claude Code's own sign-in."
            .to_owned(),
        _ => format!(
            "This Claude Code sign-in cannot read usage because it lacks the user:profile scope. \
             Run {} and sign in again with /login.",
            login.sign_in_command()
        ),
    }
}

/// Anthropic answered 403 because the token was revoked.
pub(crate) fn revoked(login: &Login) -> String {
    match login.source {
        LoginSource::Environment(_) => "Anthropic says the token in CLAUDE_CODE_OAUTH_TOKEN was \
             revoked. Replace it, or unset it to use Claude Code's own sign-in."
            .to_owned(),
        _ => format!(
            "Anthropic says this Claude Code sign-in was revoked. Run {} to sign in again.",
            login.sign_in_command()
        ),
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

    use quotadial_core::Secret;

    use super::*;
    use crate::environment::Environment;
    use crate::login::Folder;

    fn env() -> Environment {
        Environment {
            home: Some(PathBuf::from("/Users/example")),
            ..Environment::default()
        }
    }

    fn login(source: LoginSource, folder: Folder) -> Login {
        Login::new(&env(), source, None, None, folder)
    }

    fn keychain(service: &str, folder: Folder) -> Login {
        login(
            LoginSource::Keychain {
                service: service.into(),
                account: Some("example".into()),
            },
            folder,
        )
    }

    fn default_keychain() -> Login {
        keychain("Claude Code-credentials", Folder::Default)
    }

    fn environment() -> Login {
        login(
            LoginSource::Environment(Secret::new("sk-ant-oat01-QUOTADIAL-TEST-TOKEN")),
            Folder::Default,
        )
    }

    #[test]
    fn every_sentence_names_a_fix() {
        let sentences = [
            signed_out(&default_keychain()),
            different_account(&default_keychain()),
            not_found(&default_keychain().account()),
            no_claude_login(&default_keychain()),
            expired(&default_keychain(), DateTime::<Utc>::UNIX_EPOCH),
            rejected(&default_keychain()),
            rejected(&environment()),
            missing_scope(&default_keychain()),
            missing_scope(&environment()),
            revoked(&default_keychain()),
            revoked(&environment()),
            no_limits(Some("free")).to_owned(),
        ];
        for sentence in sentences {
            assert!(
                [
                    "Run `claude`",
                    "run `claude`",
                    "Replace it",
                    "Unset it",
                    "Upgrade"
                ]
                .iter()
                .any(|fix| sentence.contains(fix)),
                "{sentence}"
            );
            assert!(!sentence.contains("sk-ant-"), "{sentence}");
        }
    }

    #[test]
    fn a_sign_in_from_another_folder_gets_that_folders_command() {
        let work = keychain(
            "Claude Code-credentials-1df4ed49",
            Folder::Named(PathBuf::from("/Users/example/.claude-work")),
        );
        // PowerShell on Windows, where `~` would not be expanded.
        let command = if cfg!(windows) {
            "`$env:CLAUDE_CONFIG_DIR = '/Users/example/.claude-work'; claude`"
        } else {
            "`CLAUDE_CONFIG_DIR=~/.claude-work claude`"
        };
        assert_eq!(
            rejected(&work),
            format!("Anthropic rejected this Claude Code sign-in. Run {command} to sign in again.")
        );
        assert_eq!(
            expired(&work, DateTime::<Utc>::UNIX_EPOCH),
            format!(
                "This Claude Code sign-in expired at 1970-01-01T00:00:00Z. Run {command} once to \
                 refresh it."
            )
        );
        assert_eq!(
            signed_out(&work),
            format!(
                "QuotaDial no longer finds the Claude Code sign-in in the macOS Keychain item \
                 \"Claude Code-credentials-1df4ed49\" (CLAUDE_CONFIG_DIR ~/.claude-work). Run \
                 {command} to sign in again."
            )
        );
        assert!(different_account(&work).contains(&format!("run {command} and sign in")));

        let unknown = keychain("Claude Code-credentials-67f74348", Folder::Unknown);
        assert_eq!(
            revoked(&unknown),
            "Anthropic says this Claude Code sign-in was revoked. Run `claude` with the \
             CLAUDE_CONFIG_DIR that made the Keychain item \"Claude Code-credentials-67f74348\" to \
             sign in again."
        );
    }

    #[test]
    fn the_source_is_named_where_it_matters() {
        assert_eq!(
            no_claude_login(&default_keychain()),
            "The sign-in in the macOS Keychain item \"Claude Code-credentials\" holds no Claude \
             account, only MCP server sign-ins. Run `claude` and sign in with /login."
        );
        assert!(
            no_claude_login(&login(
                LoginSource::File(PathBuf::from("/Users/example/.claude/.credentials.json")),
                Folder::Default
            ))
            .starts_with("The sign-in in ~/.claude/.credentials.json holds")
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
