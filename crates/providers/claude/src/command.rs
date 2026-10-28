//! `quotadial claude status-line`, the command Claude Code's status line runs (D-145): Claude Code
//! runs it "with JSON session data on stdin" when a session starts, and again as it goes on, such as
//! when a new assistant message arrives or a window it handed over last reaches its reset, at most
//! once every 300 ms, and shows what it prints (its docs, "Customize your status line").
//!
//! So the command keeps the two windows (`handover`) in its folder's file (`store`) first, then runs
//! the status line the user already had, when one is named after it, through
//! `quotadial_status_line::run`, which Antigravity's command shares (D-126). Nothing of its own is
//! ever printed: Claude Code shows no status line when the command prints nothing, and a person who
//! wants one chains theirs. When QuotaDial cannot keep its numbers, as with no data folder or a full
//! disk, a command after it still runs, its output and exit status unchanged.

use std::path::Path;
use std::process::ExitCode;

use chrono::{DateTime, Utc};
use quotadial_core::Invocation;

use crate::{handover, store};

/// The command's name: `quotadial claude status-line`.
pub(crate) const NAME: &str = "status-line";

/// What `quotadial claude --help` and the command's manual page say it is for.
pub(crate) const ABOUT: &str = "Keeps the numbers Claude Code's status line hands it on standard \
                                input, then runs the status line command named after it, if any, \
                                with the same input.";

/// Runs the command at `now`, keeping what it is handed in `file`, the file in QuotaDial's data
/// folder for the Claude Code folder it runs for, or nowhere when there is no data folder.
pub(crate) fn run(file: Option<&Path>, now: DateTime<Utc>, invocation: Invocation<'_>) -> ExitCode {
    quotadial_status_line::run(invocation, |payload| keep(file, payload, now))
}

/// Keeps the windows `payload` hands over in `file`, when it hands over one with a share. What goes
/// wrong is dropped, never printed, so it can never reach the status line.
fn keep(file: Option<&Path>, payload: &[u8], now: DateTime<Utc>) {
    if let (Some(file), Some(handover)) = (file, handover::from_status_line(payload, now)) {
        let _ = store::write(file, &handover::to_file(&handover));
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{Duration, Instant};

    use serde_json::{Value, json};

    use super::*;

    fn now() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-10-28T17:30:00Z")
            .expect("a time")
            .with_timezone(&Utc)
    }

    /// Runs the command with `args`, `input` on standard input, keeping its numbers in `file`, and
    /// gives its exit status and what it printed.
    fn invoke(file: Option<&Path>, args: &[&str], input: &[u8]) -> (ExitCode, Vec<u8>, Vec<u8>) {
        let args: Vec<String> = args.iter().map(|arg| (*arg).to_owned()).collect();
        let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
        let code = run(
            file,
            now(),
            Invocation {
                args: &args,
                stdin: &mut &input[..],
                stdout: &mut stdout,
                stderr: &mut stderr,
            },
        );
        (code, stdout, stderr)
    }

    /// A data folder made up by the test, and QuotaDial's file in it.
    fn data_folder() -> (tempfile::TempDir, PathBuf) {
        let temp = tempfile::tempdir().expect("temp dir");
        let file = temp.path().join("quotadial").join(store::DEFAULT_FILE);
        (temp, file)
    }

    /// A folder nothing can be written in: a file stands where its parent should be.
    fn unwritable() -> (tempfile::TempDir, PathBuf) {
        let temp = tempfile::tempdir().expect("temp dir");
        fs::write(temp.path().join("quotadial"), "a file, not a folder").expect("writes");
        let file = temp.path().join("quotadial").join(store::DEFAULT_FILE);
        (temp, file)
    }

    /// What Claude Code's docs say its status line hands over, every field there, each value made up
    /// and marked, so a test can tell whether any of it was kept.
    fn every_field() -> Value {
        json!({
            "cwd": "/not-kept/cwd",
            "session_id": "not-kept-session",
            "session_name": "Not kept: a session's name",
            "prompt_id": "not-kept-prompt",
            "transcript_path": "/not-kept/transcript.jsonl",
            "model": { "id": "not-kept-model", "display_name": "Not kept model" },
            "workspace": {
                "current_dir": "/not-kept/current",
                "project_dir": "/not-kept/project",
                "added_dirs": ["/not-kept/added"],
                "git_worktree": "not-kept-worktree",
                "repo": { "host": "not-kept.example.com", "owner": "not-kept-owner", "name": "not-kept-repo" }
            },
            "version": "9.8.7",
            "output_style": { "name": "not-kept-style" },
            "cost": {
                "total_cost_usd": 424.242,
                "total_duration_ms": 434_343,
                "total_api_duration_ms": 444_444,
                "total_lines_added": 454_545,
                "total_lines_removed": 464_646
            },
            "context_window": {
                "total_input_tokens": 474_747,
                "total_output_tokens": 484_848,
                "context_window_size": 4_949_494,
                "used_percentage": 50.5050,
                "remaining_percentage": 49.4949,
                "current_usage": {
                    "input_tokens": 515_151,
                    "output_tokens": 525_252,
                    "cache_creation_input_tokens": 535_353,
                    "cache_read_input_tokens": 545_454
                }
            },
            "exceeds_200k_tokens": true,
            "prompt_cache": {
                "warm": true,
                "ttl": "not-kept-ttl",
                "expires_at": 1_555_555_555,
                "requests": 565_656,
                "hit_ratio": 0.575_757
            },
            "fast_mode": true,
            "effort": { "level": "not-kept-effort" },
            "thinking": { "enabled": true },
            "rate_limits": {
                "five_hour": {
                    "used_percentage": 23.5,
                    "resets_at": 1_761_690_600,
                    "window": "not-kept-window"
                },
                "seven_day": { "used_percentage": 41.2, "resets_at": 1_762_097_400 },
                "spend_limit": {
                    "used_percentage": 58.5858,
                    "resets_at": 1_595_959_595,
                    "used_usd": 606.0606,
                    "limit_usd": 616_161,
                    "period": "not-kept-period"
                }
            },
            "vim": { "mode": "VISUAL LINE" },
            "agent": { "name": "not-kept-agent" },
            "pr": { "number": 626_262, "url": "https://not-kept.example.com/pull", "review_state": "not-kept-state" },
            "worktree": {
                "name": "not-kept-worktree-name",
                "path": "/not-kept/worktree",
                "branch": "not-kept-branch",
                "original_cwd": "/not-kept/original",
                "original_branch": "not-kept-original-branch"
            }
        })
    }

    fn keys(value: &Value) -> Vec<&str> {
        let mut keys: Vec<&str> = value
            .as_object()
            .expect("an object")
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();
        keys
    }

    /// The test rule 5 asks for, the twin of D-126's (the manager on #73, D-145): the file holds the
    /// five-hour and the weekly window's share used and reset, and the time, and nothing else, even
    /// when what Claude Code hands over carries the session's folder, transcript, model, workspace,
    /// context, cost and a spend limit.
    #[test]
    fn the_file_holds_only_the_fields_rule_5_keeps() {
        let (_temp, file) = data_folder();

        let (code, stdout, stderr) = invoke(Some(&file), &[], every_field().to_string().as_bytes());

        assert_eq!(code, ExitCode::SUCCESS);
        assert!(stdout.is_empty() && stderr.is_empty());
        let text = fs::read_to_string(&file).expect("the file is written");
        let kept: Value = serde_json::from_str(&text).expect("JSON");
        assert_eq!(keys(&kept), ["rate_limits", "written_at"]);
        assert_eq!(kept["written_at"], "2026-10-28T17:30:00Z");
        assert_eq!(keys(&kept["rate_limits"]), ["five_hour", "seven_day"]);
        assert_eq!(
            kept["rate_limits"],
            json!({
                "five_hour": { "used_percentage": 23.5, "resets_at": 1_761_690_600 },
                "seven_day": { "used_percentage": 41.2, "resets_at": 1_762_097_400 }
            })
        );
        for never in [
            "not-kept", "Not kept", "not_kept", "9.8.7", "4242", "4343", "4444", "4545", "4646",
            "4747", "4848", "4949", "5050", "5151", "5252", "5353", "5454", "5555", "5656", "5757",
            "5858", "5959", "6060", "6161", "6262", "VISUAL", "spend", "window\"",
        ] {
            assert!(!text.contains(never), "{never} reached the file:\n{text}");
        }
    }

    #[cfg(unix)]
    #[test]
    fn the_file_is_readable_only_by_the_user() {
        use std::os::unix::fs::PermissionsExt;

        let (_temp, file) = data_folder();

        invoke(Some(&file), &[], every_field().to_string().as_bytes());

        let mode = fs::metadata(&file).expect("metadata").permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    #[test]
    fn with_no_command_after_it_nothing_is_printed_and_it_ends_with_0() {
        let (_temp, file) = data_folder();
        for input in [
            every_field().to_string().into_bytes(),
            b"not JSON".to_vec(),
            Vec::new(),
        ] {
            let (code, stdout, stderr) = invoke(Some(&file), &[], &input);

            assert_eq!(code, ExitCode::SUCCESS);
            assert!(stdout.is_empty(), "{}", String::from_utf8_lossy(&stdout));
            assert!(stderr.is_empty(), "{}", String::from_utf8_lossy(&stderr));
        }
    }

    #[test]
    fn it_ends_with_0_at_once_when_its_folder_cannot_be_written_or_the_input_is_not_json() {
        let (_temp, blocked) = unwritable();
        let (_other, file) = data_folder();
        let started = Instant::now();

        for (file, input) in [
            (
                Some(blocked.as_path()),
                every_field().to_string().into_bytes(),
            ),
            (None, every_field().to_string().into_bytes()),
            (Some(file.as_path()), b"{ not JSON".to_vec()),
            (Some(file.as_path()), b"\xff\xfe".to_vec()),
        ] {
            let (code, stdout, stderr) = invoke(file, &[], &input);

            assert_eq!(code, ExitCode::SUCCESS);
            assert!(stdout.is_empty() && stderr.is_empty());
        }
        assert!(
            started.elapsed() < Duration::from_secs(1),
            "{:?}",
            started.elapsed()
        );
        assert!(!blocked.exists());
        assert!(!file.exists(), "nothing to keep");
    }

    #[test]
    fn a_payload_with_no_rate_limits_leaves_the_file_as_it_was() {
        let (_temp, file) = data_folder();
        invoke(Some(&file), &[], every_field().to_string().as_bytes());
        let before = fs::read(&file).expect("written");

        // As a sign-in with an API key hands it over, and before a session's first answer.
        let mut no_rate_limits = every_field();
        no_rate_limits
            .as_object_mut()
            .expect("an object")
            .remove("rate_limits");
        invoke(Some(&file), &[], no_rate_limits.to_string().as_bytes());
        invoke(
            Some(&file),
            &[],
            br#"{ "rate_limits": { "spend_limit": { "used_percentage": 10 } } }"#,
        );

        assert_eq!(fs::read(&file).expect("still there"), before);
    }

    #[test]
    fn a_command_after_it_that_cannot_be_found_ends_as_a_shell_says_and_the_numbers_are_kept() {
        let (_temp, file) = data_folder();

        let (code, stdout, stderr) = invoke(
            Some(&file),
            &["quotadial-test-no-such-program-0a1b2c", "--flag"],
            every_field().to_string().as_bytes(),
        );

        assert_eq!(code, ExitCode::from(127));
        assert!(stdout.is_empty());
        assert!(
            String::from_utf8_lossy(&stderr)
                .starts_with("quotadial: quotadial-test-no-such-program")
        );
        assert!(file.exists());
    }

    #[cfg(unix)]
    #[test]
    fn the_file_is_written_before_the_command_after_it_runs() {
        let (_temp, file) = data_folder();
        let path = file.to_str().expect("UTF-8");

        // The command prints QuotaDial's file as it finds it.
        let (code, stdout, _) = invoke(
            Some(&file),
            &["/bin/sh", "-c", "cat \"$0\"", path],
            every_field().to_string().as_bytes(),
        );

        assert_eq!(code, ExitCode::SUCCESS);
        assert_eq!(stdout, fs::read(&file).expect("written"));
        assert!(String::from_utf8_lossy(&stdout).contains("\"written_at\""));
    }

    #[cfg(unix)]
    #[test]
    fn the_command_after_it_gets_the_same_input_and_its_output_and_status_come_back_unchanged() {
        let (_temp, file) = data_folder();
        let input = every_field().to_string().into_bytes();

        let (code, stdout, stderr) = invoke(
            Some(&file),
            &[
                "/bin/sh",
                "-c",
                "cat; printf ' and its own'; printf 'its warning' >&2; exit 4",
            ],
            &input,
        );

        let mut expected = input.clone();
        expected.extend_from_slice(b" and its own");
        assert_eq!(stdout, expected);
        assert_eq!(stderr, b"its warning");
        assert_eq!(code, ExitCode::from(4));
        assert!(file.exists(), "and the numbers are kept");
    }

    #[cfg(unix)]
    #[test]
    fn when_quotadial_cannot_keep_its_numbers_the_command_after_it_runs_unchanged() {
        let (_temp, blocked) = unwritable();
        let input = every_field().to_string().into_bytes();
        let command = ["/bin/sh", "-c", "cat; printf 'its warning' >&2; exit 4"];

        for file in [Some(blocked.as_path()), None] {
            let (code, stdout, stderr) = invoke(file, &command, &input);

            assert_eq!(stdout, input, "only the command's own output");
            assert_eq!(stderr, b"its warning", "and nothing of QuotaDial's");
            assert_eq!(code, ExitCode::from(4));
        }
        assert!(!blocked.exists());
    }
}
