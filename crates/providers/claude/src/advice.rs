//! What QuotaDial says of a Claude account, in #52's plain words (D-145): what it does with the
//! sign-in, which is nothing, carrying the exact line to add to Claude Code's settings and where it
//! keeps the numbers (the manager's rules 7 and 8 on #73, P-020's rule 2), and the one line under an
//! account whose numbers are not current. Claude Code reads its settings when it starts, so each
//! says to start it again, and the command must be on the `PATH` Claude Code starts with.
//!
//! A time is written as a person reads it, in UTC (D-142), never as "3 min ago", so a line stays
//! the same while the numbers it is about do, and the daemon's history, which keeps a snapshot only
//! when its time, status or line changes, keeps one for it rather than one a minute.

use chrono::{DateTime, Utc};
use quotadial_core::text;

use crate::environment::Environment;
use crate::folders::Folder;
use crate::{ID, command};

/// The command Claude Code's settings name: `quotadial claude status-line`.
pub(crate) fn command() -> String {
    format!("quotadial {ID} {}", command::NAME)
}

/// The block to add to Claude Code's settings, as its docs write one ("Customize your status
/// line"). QuotaDial never writes it.
pub(crate) fn block() -> String {
    format!(
        r#""statusLine": {{ "type": "command", "command": "{}" }}"#,
        command()
    )
}

/// The settings file the block goes in: `settings.json` in the folder, as people write it.
pub(crate) fn settings(folder: &Folder, env: &Environment) -> String {
    match folder.path(env) {
        Some(path) => env.tilde(&path.join("settings.json")),
        None => "settings.json in Claude Code's folder".to_owned(),
    }
}

/// What QuotaDial does with a Claude account's sign-in: nothing. It names `file`, the folder's file
/// in QuotaDial's data folder, the block that hands it the numbers, where the block goes, and that
/// Claude Code must start again with the command on its `PATH`.
pub(crate) fn sign_in_use(folder: &Folder, file: &str, env: &Environment) -> String {
    format!(
        "QuotaDial never reads Claude Code's sign-in. It keeps only the numbers Claude Code's \
         status line hands it, in {file} in QuotaDial's data folder, once `{}` is in {} and Claude \
         Code has started again with `quotadial` on its PATH.",
        block(),
        settings(folder, env)
    )
}

/// Before any numbers, as for a person whose numbers came from Claude Code's sign-in before (the
/// manager's rule 9): why there are none, what to add, as a new status line or in front of one
/// already there, to start Claude Code again, and, in the docs' words, which sign-ins Claude Code
/// hands numbers for (rule 5). One sentence.
pub(crate) fn waiting(folder: &Folder, env: &Environment) -> String {
    format!(
        "QuotaDial reads Claude's limits only from what Claude Code's status line hands it, and \
         none has come yet: add `{}` to {}, or put `{}` and a space in front of the command \
         already there, then start Claude Code again; Claude Code hands them over only for \
         claude.ai Pro and Max subscribers, and only after the first API response in the session.",
        block(),
        settings(folder, env),
        command()
    )
}

/// Numbers handed over longer ago than two polls (#52, D-126).
pub(crate) fn stale(handed_over: DateTime<Utc>) -> String {
    format!(
        "Claude Code last handed over its numbers on {}: they update when it runs again, and a \
         window that has reset since is left out until then.",
        text::date_and_time(handed_over)
    )
}

/// Numbers whose every window has reset since they were handed over (the manager's rule 3): when
/// Claude Code last handed them over, and that its next run brings them back.
pub(crate) fn nothing(handed_over: DateTime<Utc>) -> String {
    format!(
        "Claude Code last handed over its numbers on {}, and each window has reset since: its next \
         run brings them back.",
        text::date_and_time(handed_over)
    )
}

/// The folder the account was found in is now signed in to another account.
pub(crate) fn another_account(folder: &Folder, env: &Environment) -> String {
    let place = folder
        .path(env)
        .map_or_else(|| "its folder".to_owned(), |path| env.tilde(&path));
    format!(
        "Claude Code in {place} is now signed in to another account, which QuotaDial shows in this \
         one's place at its next look."
    )
}

/// No Claude Code folder on the machine names the account any more.
pub(crate) const NOT_FOUND: &str = "QuotaDial no longer finds this account in any Claude Code \
                                    folder on this machine, and shows it again once Claude Code is \
                                    signed in to it.";

/// An `extra_config_dirs` entry was left out because it is not a full path. Like every warning, it
/// leaves the provider out: the report shows it under the provider's own name (D-128).
pub(crate) fn skipped_dir(dir: &std::path::Path) -> String {
    format!(
        "the extra_config_dirs entry \"{}\" in config.json is not a full path, so QuotaDial left \
         it out. Write it in full, or start it with ~.",
        dir.display()
    )
}

#[cfg(test)]
mod tests {
    use std::path::{MAIN_SEPARATOR_STR, Path, PathBuf};

    use quotadial_core::{Account, Diagnostics, Snapshot, Status, StatusReport};

    use super::*;

    fn at(rfc3339: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(rfc3339)
            .expect("a time")
            .with_timezone(&Utc)
    }

    fn env_in(home: &Path) -> Environment {
        Environment {
            home: Some(home.to_path_buf()),
            ..Environment::default()
        }
    }

    fn home() -> PathBuf {
        [MAIN_SEPARATOR_STR, "Users", "example"].iter().collect()
    }

    /// `~` and the rest as `text::tilde` writes them, with the system's own separator (D-129).
    fn in_home(parts: &[&str]) -> String {
        let mut path = PathBuf::from("~");
        for part in parts {
            path.push(part);
        }
        path.display().to_string()
    }

    #[test]
    fn the_block_is_the_one_claude_code_s_settings_take() {
        let block = block();

        assert_eq!(
            block,
            r#""statusLine": { "type": "command", "command": "quotadial claude status-line" }"#
        );
        let parsed: serde_json::Value =
            serde_json::from_str(&format!("{{ {block} }}")).expect("JSON");
        assert_eq!(parsed["statusLine"]["type"], "command");
        assert_eq!(
            parsed["statusLine"]["command"],
            "quotadial claude status-line"
        );
        let env = env_in(&home());
        assert_eq!(
            settings(&Folder::Default, &env),
            in_home(&[".claude", "settings.json"])
        );
        let work = home().join(".claude-work").display().to_string();
        assert_eq!(
            settings(&Folder::Named(work), &env),
            in_home(&[".claude-work", "settings.json"])
        );
    }

    #[test]
    fn the_sign_in_sentence_names_the_file_the_block_where_it_goes_a_restart_and_the_path() {
        let env = env_in(&home());

        let said = sign_in_use(&Folder::Default, "claude.json", &env);

        assert_eq!(
            said,
            format!(
                "QuotaDial never reads Claude Code's sign-in. It keeps only the numbers Claude \
                 Code's status line hands it, in claude.json in QuotaDial's data folder, once \
                 `\"statusLine\": {{ \"type\": \"command\", \"command\": \"quotadial claude \
                 status-line\" }}` is in {} and Claude Code has started again with `quotadial` on \
                 its PATH.",
                in_home(&[".claude", "settings.json"])
            )
        );
    }

    #[test]
    fn the_first_line_says_why_it_is_empty_what_to_add_and_which_sign_ins_get_numbers() {
        let env = env_in(&home());

        let said = waiting(&Folder::Default, &env);

        assert!(
            said.starts_with(
                "QuotaDial reads Claude's limits only from what Claude Code's status line hands \
                 it, and none has come yet: add `"
            ),
            "{said}"
        );
        assert!(said.contains(&format!("`{}`", block())), "{said}");
        assert!(said.contains(&settings(&Folder::Default, &env)), "{said}");
        assert!(
            said.contains("`quotadial claude status-line` and a space in front of"),
            "{said}"
        );
        assert!(said.contains("start Claude Code again"), "{said}");
        // In the docs' words ("Customize your status line"): `rate_limits` "appears only for
        // claude.ai Pro and Max subscribers ... and only after the first API response in the
        // session" (the manager's rule 5).
        assert!(
            said.ends_with(
                "only for claude.ai Pro and Max subscribers, and only after the first API \
                 response in the session."
            ),
            "{said}"
        );
    }

    #[test]
    fn a_time_is_written_as_itself_so_a_line_stays_the_same() {
        let handed_over = at("2026-10-28T17:30:00.5Z");

        assert_eq!(
            stale(handed_over),
            "Claude Code last handed over its numbers on 28 Oct 2026 at 17:30 UTC: they update \
             when it runs again, and a window that has reset since is left out until then."
        );
        assert_eq!(
            nothing(handed_over),
            "Claude Code last handed over its numbers on 28 Oct 2026 at 17:30 UTC, and each window \
             has reset since: its next run brings them back."
        );
    }

    /// Every line, for the default folder and a named one.
    fn every_line() -> Vec<String> {
        let env = env_in(&home());
        let work = Folder::Named(home().join("My Claude").display().to_string());
        let handed_over = at("2026-10-28T17:30:00Z");
        let mut lines = vec![
            stale(handed_over),
            nothing(handed_over),
            NOT_FOUND.to_owned(),
            skipped_dir(Path::new("claude-work")),
        ];
        for folder in [Folder::Default, work] {
            lines.push(sign_in_use(&folder, &folder.file_name(), &env));
            lines.push(waiting(&folder, &env));
            lines.push(another_account(&folder, &env));
        }
        lines
    }

    #[test]
    fn every_line_is_in_plain_words_and_each_line_under_an_account_is_one_sentence() {
        for said in every_line() {
            assert!(!said.contains('\n') && said.ends_with('.'), "{said}");
            assert!(
                !said.contains('\u{2014}') && !said.contains('\u{2013}'),
                "{said}"
            );
            // #52's plain words: never "quota", "auth" or "credential" outside a name.
            assert_eq!(text::unplain_words(&said), Vec::<String>::new(), "{said}");
        }
        // The line a card shows in place of its windows is one sentence (the manager's rule 9):
        // outside its code, no stop ends one sentence and starts another.
        let env = env_in(&home());
        for said in [
            waiting(&Folder::Default, &env),
            stale(at("2026-10-28T17:30:00Z")),
            nothing(at("2026-10-28T17:30:00Z")),
            another_account(&Folder::Default, &env),
            NOT_FOUND.to_owned(),
        ] {
            let prose: String = said.split('`').step_by(2).collect();
            assert!(!prose.trim_end_matches('.').contains(". "), "{said}");
        }
    }

    /// The window check's copies hold Claude's sentences as this provider writes them, each whole on
    /// its line, so the check covers the very line a person on 0.2.0 sees on a Claude card after
    /// updating (the manager's rule 9): for Claude Code's default folder and for `~/.claude-work`,
    /// the home folder being `/Users/someone`, as a Mac writes them.
    #[cfg(unix)]
    #[test]
    fn the_window_check_shows_claude_s_lines_as_the_provider_writes_them() {
        const WINDOW: &str = include_str!("../../../../apps/tray/scripts/window-fixtures.mjs");
        // The website's copy, which its pictures of the app are taken from.
        const SITE: &str = include_str!("../../../../apps/tray/scripts/site-fixtures.mjs");
        let home = Path::new("/Users/someone");
        let env = env_in(home);
        let work = Folder::Named(home.join(".claude-work").display().to_string());
        for (name, said) in [
            (
                "CLAUDE_SIGN_IN",
                sign_in_use(&Folder::Default, "claude.json", &env),
            ),
            (
                "CLAUDE_WORK_SIGN_IN",
                sign_in_use(&work, &work.file_name(), &env),
            ),
            ("CLAUDE_WAITING", waiting(&work, &env)),
        ] {
            let written = format!(
                "const {name} = {};",
                serde_json::to_string(&said).expect("JSON")
            );
            assert!(
                WINDOW.contains(&written),
                "window-fixtures.mjs has no {written}"
            );
        }
        let site = format!(
            "const CLAUDE_SIGN_IN = {};",
            serde_json::to_string(&sign_in_use(&Folder::Default, "claude.json", &env))
                .expect("JSON")
        );
        assert!(SITE.contains(&site), "site-fixtures.mjs has no {site}");
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
                    status: Status::Unsupported,
                    error: Some(line.to_owned()),
                }],
                ..StatusReport::default()
            },
            settings: Vec::new(),
        }
        .text(DateTime::<Utc>::UNIX_EPOCH, Some(home))
    }

    #[test]
    fn doctor_shows_the_line_to_add_and_the_file_as_written_for_a_user_called_claude_or_codex() {
        // Doctor hides the user's name only in a place outside the home folder (D-134): for a user
        // called `claude`, the block's `quotadial claude status-line` and `~/.claude` stay as they
        // are written, in the account's sentence and in its line (the manager's rule 8).
        for name in ["ann", "claude", "codex"] {
            let home: PathBuf = [MAIN_SEPARATOR_STR, "Users", name].iter().collect();
            let env = env_in(&home);
            for folder in [
                Folder::Default,
                Folder::Named(home.join(".claude-work").display().to_string()),
                Folder::Named(home.join("My Claude").display().to_string()),
            ] {
                let account = Account {
                    provider: ID.into(),
                    provider_name: crate::NAME.into(),
                    account_id: "00000000-0000-0000-0000-000000000001".into(),
                    display_name: String::new(),
                    plan: None,
                    org: None,
                    sign_in: None,
                    sign_in_use: sign_in_use(&folder, &folder.file_name(), &env),
                };
                let line = waiting(&folder, &env);

                let text = doctor(account.clone(), &line, &home);

                assert!(
                    text.contains(&format!("\n    sign-in: {}\n", account.sign_in_use)),
                    "{name}: {}\n{text}",
                    account.sign_in_use
                );
                assert!(
                    text.contains(&format!("\n    {line}\n")),
                    "{name}: {line}\n{text}"
                );
            }
        }
    }
}
