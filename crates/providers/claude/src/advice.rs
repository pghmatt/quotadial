//! The one sentence QuotaDial shows when it has no numbers for a Claude account, naming the fix.
//! A sign-in kept for another config folder gets the command for that folder (#34).

use chrono::{DateTime, Utc};
use quotadial_core::{Account, text};

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

/// The Keychain could not be listed, so its items were looked up folder by folder. Like every
/// warning, it leaves the provider out: the report shows it under the provider's own name (D-128).
pub(crate) fn unlisted(reason: &str) -> String {
    format!(
        "could not list the Claude Code items in the macOS Keychain: {reason}. Each folder's item \
         was looked up by name instead, so a Keychain item whose folder QuotaDial does not know is \
         left out until the listing works."
    )
}

/// An `extra_config_dirs` entry was left out because it is not a full path.
pub(crate) fn skipped_dir(dir: &std::path::Path) -> String {
    format!(
        "the extra_config_dirs entry \"{}\" in config.json is not a full path, so QuotaDial left \
         it out. Write it in full, or start it with ~.",
        dir.display()
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

/// The sign-in's token expired at `at`, written as a person reads it (D-142), and Claude Code
/// refreshes it when it next runs.
pub(crate) fn expired(login: &Login, at: DateTime<Utc>) -> String {
    format!(
        "This Claude Code sign-in expired on {}. Run {} once to refresh it.",
        text::date_and_time(at),
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
    use std::path::{MAIN_SEPARATOR, MAIN_SEPARATOR_STR, Path, PathBuf};

    use quotadial_core::{Diagnostics, Secret, Snapshot, Status, StatusReport};

    use super::*;
    use crate::environment::Environment;
    use crate::login::{Folder, claude_in};

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
        // PowerShell on Windows, where `~` would not be expanded, so the folder is written from
        // `$HOME` there (D-131).
        let command = if cfg!(windows) {
            format!("`$env:CLAUDE_CONFIG_DIR = \"$HOME{MAIN_SEPARATOR}.claude-work\"; claude`")
        } else {
            "`CLAUDE_CONFIG_DIR=~/'.claude-work' claude`".to_owned()
        };
        assert_eq!(
            rejected(&work),
            format!("Anthropic rejected this Claude Code sign-in. Run {command} to sign in again.")
        );
        // The time as a person reads it, its seconds dropped (D-142).
        let at: DateTime<Utc> = "2026-10-26T09:12:47Z".parse().expect("a time");
        assert_eq!(
            expired(&work, at),
            format!(
                "This Claude Code sign-in expired on 26 Oct 2026 at 09:12 UTC. Run {command} once \
                 to refresh it."
            )
        );
        // The folder as `text::tilde` writes it: `~` and the system's own separator (D-129).
        let folder = Path::new("~").join(".claude-work");
        assert_eq!(
            signed_out(&work),
            format!(
                "QuotaDial no longer finds the Claude Code sign-in in the macOS Keychain item \
                 \"Claude Code-credentials-1df4ed49\" (CLAUDE_CONFIG_DIR {}). Run {command} to \
                 sign in again.",
                folder.display()
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

    /// Doctor's text for `account` with `line` under it, the home folder being `home`.
    fn doctor(account: Account, line: &str, home: &Path) -> String {
        Diagnostics {
            version: "0.3.0".into(),
            system: "macOS 26.1 (aarch64)".into(),
            licence: "QuotaDial Free, with no licence key.".into(),
            data_folder: home.join("quotadial").display().to_string(),
            daemon: "not running".into(),
            numbers_from: "a direct fetch".into(),
            report: StatusReport {
                snapshots: vec![Snapshot {
                    account,
                    windows: Vec::new(),
                    extra_usage: None,
                    status: Status::AuthRequired,
                    error: Some(line.to_owned()),
                }],
                ..StatusReport::default()
            },
            settings: Vec::new(),
        }
        .text(DateTime::<Utc>::UNIX_EPOCH, Some(home))
    }

    #[test]
    fn doctor_shows_the_fix_command_as_written_in_both_shells() {
        // Doctor writes the home folder `~` over its whole text (D-129), which turned the command
        // `CLAUDE_CONFIG_DIR='/Users/ann/My Claude' claude`, which named the user, into
        // `CLAUDE_CONFIG_DIR='~/My Claude' claude`, whose quoted `~` no shell expands (D-131). The
        // sentence as this system writes it, and each shell's command, come through doctor as
        // written.
        let home: PathBuf = [MAIN_SEPARATOR_STR, "Users", "ann"].iter().collect();
        let env = Environment {
            home: Some(home.clone()),
            ..Environment::default()
        };
        let outside: PathBuf = [MAIN_SEPARATOR_STR, "Volumes", "Data", "O'Brien's Claude"]
            .iter()
            .collect();
        for dir in [
            home.join("My Claude"),
            home.join("it's \"$HOME\" `Claude`"),
            outside,
        ] {
            let login = Login::new(
                &env,
                LoginSource::File(dir.join(".credentials.json")),
                None,
                None,
                Folder::Named(dir.clone()),
            );
            for line in [
                signed_out(&login),
                claude_in(&dir, Some(&home), false),
                claude_in(&dir, Some(&home), true),
            ] {
                let text = doctor(login.account(), &line, &home);
                assert!(text.contains(&format!("\n    {line}\n")), "{line}\n{text}");
            }
        }
    }

    #[test]
    fn doctor_shows_the_command_and_places_as_written_for_a_user_called_claude_or_codex() {
        // Doctor hid every whole word that was the user's name, so for a user called `claude` it
        // wrote the command as `CLAUDE_CONFIG_DIR=~/'.[user]-work' [user]` and `~/.claude` as
        // `~/.[user]`: a command that fails and a folder that is not there (D-134). In the home
        // folder and in a command the words stay, whatever the user is called, and the name is
        // still hidden in a place outside the home folder, the command's own words around it kept.
        for name in ["ann", "claude", "codex"] {
            let home: PathBuf = [MAIN_SEPARATOR_STR, "Users", name].iter().collect();
            let env = Environment {
                home: Some(home.clone()),
                ..Environment::default()
            };
            let login_for = |dir: &Path| {
                Login::new(
                    &env,
                    LoginSource::File(dir.join(".credentials.json")),
                    None,
                    None,
                    Folder::Named(dir.to_path_buf()),
                )
            };
            for dir in [
                home.join(".claude"),
                home.join(".claude-work"),
                home.join(".codex"),
                home.join("My Claude"),
            ] {
                let login = login_for(&dir);
                for line in [
                    signed_out(&login),
                    claude_in(&dir, Some(&home), false),
                    claude_in(&dir, Some(&home), true),
                ] {
                    let text = doctor(login.account(), &line, &home);
                    assert!(
                        text.contains(&format!("\n    {line}\n")),
                        "{name}: {line}\n{text}"
                    );
                    // The account's own sentence, which names where its sign-in is.
                    let sign_in = login.account().sign_in_use;
                    assert!(
                        text.contains(&format!("\n    sign-in: {sign_in}\n")),
                        "{name}: {sign_in}\n{text}"
                    );
                }
            }
            let outside = |who: &str| -> PathBuf {
                [MAIN_SEPARATOR_STR, "Volumes", "Data", who, "My Claude"]
                    .iter()
                    .collect()
            };
            let (shown, hidden) = (outside(name), outside("[user]"));
            for windows in [false, true] {
                let line = claude_in(&shown, Some(&home), windows);
                let hidden_line = claude_in(&hidden, Some(&home), windows);
                let text = doctor(login_for(&shown).account(), &line, &home);
                assert!(
                    text.contains(&format!("\n    {hidden_line}\n")),
                    "{name}: {hidden_line}\n{text}"
                );
                assert!(!text.contains(&line), "{name}: {line}\n{text}");
            }
        }
    }

    #[test]
    fn the_source_is_named_where_it_matters() {
        assert_eq!(
            no_claude_login(&default_keychain()),
            "The sign-in in the macOS Keychain item \"Claude Code-credentials\" holds no Claude \
             account, only MCP server sign-ins. Run `claude` and sign in with /login."
        );
        let file = Path::new("~").join(".claude/.credentials.json");
        assert!(
            no_claude_login(&login(
                LoginSource::File(PathBuf::from("/Users/example/.claude/.credentials.json")),
                Folder::Default
            ))
            .starts_with(&format!("The sign-in in {} holds", file.display()))
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
    fn what_discovery_could_not_look_at_is_said_in_plain_words_without_the_providers_id() {
        let warnings = [
            unlisted("/usr/bin/security exited with status 36"),
            skipped_dir(std::path::Path::new("claude-work")),
        ];
        for warning in warnings {
            // The report names the provider, by its own name (D-128).
            assert!(!warning.starts_with(crate::ID), "{warning}");
            // #52's plain words: never "quota", "auth" or "credential" outside a name.
            assert_eq!(
                quotadial_core::text::unplain_words(&warning),
                Vec::<String>::new(),
                "{warning}"
            );
        }
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
