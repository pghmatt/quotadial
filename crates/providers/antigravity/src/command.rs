//! `quotadial antigravity status-line`, the command Antigravity CLI's status line runs: Antigravity
//! CLI pipes its session's state to it as JSON whenever its agent's state changes, at most every
//! 300 ms, stops a run that takes longer than 5 seconds, and turns its status line off for the
//! session after 30 failed runs in a row (its docs, "Status line customization").
//!
//! So the command keeps what QuotaDial needs (`handover`) in its file (`store`) first, so a slow or
//! failing status line never costs the handover, then runs the status line the user already had,
//! when one is named after it (`chain`). Nothing of its own is ever printed: with no command named
//! it prints nothing and ends with 0, and when QuotaDial cannot keep its numbers, as with no data
//! folder or a full disk, the named command still runs and its output and exit status come back
//! unchanged (the manager on #73). The user's status line is never broken by QuotaDial's problem.

use std::path::Path;
use std::process::ExitCode;

use chrono::{DateTime, Utc};
use quotadial_core::Invocation;

use crate::{chain, handover, store};

/// The command's name: `quotadial antigravity status-line`.
pub(crate) const NAME: &str = "status-line";

/// What `quotadial antigravity --help` and the command's manual page say it is for.
pub(crate) const ABOUT: &str = "Keeps the numbers Antigravity CLI's status line hands it on \
                                standard input, then runs the status line command named after \
                                it, if any, with the same input.";

/// Runs the command at `now`, keeping what it is handed in `file`, QuotaDial's file in its data
/// folder, or nowhere when there is none.
pub(crate) fn run(file: Option<&Path>, now: DateTime<Utc>, invocation: Invocation<'_>) -> ExitCode {
    let Invocation {
        args,
        stdin,
        stdout,
        stderr,
    } = invocation;
    let mut payload = Vec::new();
    // All of it, whatever it holds: a command named after this one gets the same bytes.
    let _ = stdin.read_to_end(&mut payload);
    keep(file, &payload, now);
    let Some((program, args)) = args.split_first() else {
        return ExitCode::SUCCESS;
    };
    let ran = chain::run(program, args, &payload, chain::WAIT);
    let _ = stdout.write_all(&ran.stdout).and_then(|()| stdout.flush());
    let _ = stderr.write_all(&ran.stderr).and_then(|()| stderr.flush());
    ExitCode::from(ran.code)
}

/// Keeps what `payload` hands over in `file`, when it hands over a share or an amount. What goes
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
        DateTime::parse_from_rfc3339("2026-10-25T22:30:00Z")
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
        let file = temp.path().join("quotadial").join(store::FILE);
        (temp, file)
    }

    /// A folder nothing can be written in: a file stands where its parent should be.
    fn unwritable() -> (tempfile::TempDir, PathBuf) {
        let temp = tempfile::tempdir().expect("temp dir");
        fs::write(temp.path().join("quotadial"), "a file, not a folder").expect("writes");
        let file = temp.path().join("quotadial").join(store::FILE);
        (temp, file)
    }

    /// What Antigravity CLI's docs say its status line hands over, every field there, each value
    /// made up and marked, so a test can tell whether any of it was kept.
    fn every_field() -> Value {
        json!({
            "cwd": "/not-kept/cwd",
            "session_id": "not-kept-session",
            "conversation_id": "not-kept-conversation",
            "conversation_title": "Not kept: a conversation's title",
            "transcript_path": "/not-kept/transcript.jsonl",
            "model": { "id": "not-kept-model", "display_name": "Not kept model", "effort": "max" },
            "workspace": { "current_dir": "/not-kept/current", "project_dir": "/not-kept/project" },
            "version": "9.8.7",
            "context_window": {
                "total_input_tokens": 424_242,
                "total_output_tokens": 434_343,
                "context_window_size": 4_545_454,
                "used_percentage": 46.4646,
                "remaining_percentage": 53.5354,
                "current_usage": { "input_tokens": 474_747, "output_tokens": 484_848 }
            },
            "exceeds_200k_tokens": true,
            "agent": { "name": "not-kept-agent" },
            "product": "antigravity",
            "quota": {
                "gemini-5h": {
                    "remaining_fraction": 0.75,
                    "reset_time": "2026-10-26T02:00:00Z",
                    "reset_in_seconds": 12_600,
                    "name": "not-kept-limit-name",
                    "window": "not-kept-window"
                },
                "gemini-weekly": { "remaining_fraction": 0.5, "reset_in_seconds": 300_000 },
                "3p-weekly": { "remaining_amount": 120, "disabled": true }
            },
            "agent_state": "not-kept-state",
            "vcs": { "type": "not-kept-vcs" },
            "cycle_mode": "not-kept-mode",
            "sandbox": { "enabled": true },
            "subagents": [{ "name": "not-kept-subagent", "role": "helper", "status": "running" }],
            "artifact_count": 494_949,
            "plan_tier": "Ultra",
            "email": "someone@example.com",
            "pending_input_count": 505_050,
            "tool_confirmation_pending": true,
            "task_count": 515_151,
            "terminal_width": 525,
            "vim": { "mode": "VISUAL LINE" },
            "cost": { "total_usd": 535.353, "subagent_usd": 545.454, "estimated": true }
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

    /// The one rule (b)(1) asks for (the manager on #73): the file holds each limit's share or
    /// amount and reset, the plan, the account and the time, and nothing else, even when what
    /// Antigravity CLI hands over carries the session's folder, transcript, title, model,
    /// workspace, context and cost.
    #[test]
    fn the_file_holds_only_the_fields_rule_b1_keeps() {
        let (_temp, file) = data_folder();

        let (code, stdout, stderr) = invoke(Some(&file), &[], every_field().to_string().as_bytes());

        assert_eq!(code, ExitCode::SUCCESS);
        assert!(stdout.is_empty() && stderr.is_empty());
        let text = fs::read_to_string(&file).expect("the file is written");
        let kept: Value = serde_json::from_str(&text).expect("JSON");
        assert_eq!(keys(&kept), ["email", "plan_tier", "quota", "written_at"]);
        assert_eq!(kept["email"], "someone@example.com");
        assert_eq!(kept["plan_tier"], "Ultra");
        assert_eq!(kept["written_at"], "2026-10-25T22:30:00Z");
        assert_eq!(
            keys(&kept["quota"]),
            ["3p-weekly", "gemini-5h", "gemini-weekly"]
        );
        for (id, limit) in kept["quota"].as_object().expect("limits") {
            for key in keys(limit) {
                assert!(
                    [
                        "remaining_fraction",
                        "remaining_amount",
                        "reset_time",
                        "reset_in_seconds",
                        "disabled"
                    ]
                    .contains(&key),
                    "{id} kept {key}"
                );
            }
        }
        for never in [
            "not-kept",
            "Not kept",
            "9.8.7",
            "4242",
            "4343",
            "4545",
            "4646",
            "5354",
            "4747",
            "4848",
            "4949",
            "5050",
            "5151",
            "525",
            "5353",
            "5454",
            "VISUAL",
            "antigravity",
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
    fn a_handover_with_no_share_or_amount_leaves_the_file_as_it_was() {
        let (_temp, file) = data_folder();
        invoke(Some(&file), &[], every_field().to_string().as_bytes());
        let before = fs::read(&file).expect("written");

        invoke(
            Some(&file),
            &[],
            br#"{ "quota": {}, "email": "other@example.com" }"#,
        );

        assert_eq!(fs::read(&file).expect("still there"), before);
    }

    #[test]
    fn a_token_in_what_is_kept_is_blanked_out_of_the_file_and_nothing_is_printed() {
        let (_temp, file) = data_folder();
        // Written in pieces, so no secret scanner takes this file for one holding a token.
        let token = concat!("ya29", ".MadeUpAccessTokenThatAntigravityNeverSends0123");
        let payload = json!({
            "plan_tier": format!("Pro {token}"),
            "quota": { format!("weekly {token}"): { "remaining_fraction": 0.5 } }
        });

        let (code, stdout, stderr) = invoke(Some(&file), &[], payload.to_string().as_bytes());

        let text = fs::read_to_string(&file).expect("written");
        assert!(!text.contains("MadeUpAccessToken"), "{text}");
        assert!(text.contains("Pro [redacted token]"), "{text}");
        assert_eq!(code, ExitCode::SUCCESS);
        assert!(stdout.is_empty() && stderr.is_empty());
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
