//! Runs the real `quotadial` and `qd` binaries in an empty home directory.

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use serde_json::{Value, json};

/// An empty directory used as the home directory, so no stored login on the machine is found.
fn empty_home(test: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("quotadial-{test}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

/// The binary with every place it could find a real sign-in pointed at `home`.
fn command(binary: &str, args: &[&str], home: &Path) -> Command {
    let mut command = Command::new(binary);
    command
        .args(args)
        .env("HOME", home)
        .env("USERPROFILE", home)
        .env("CLAUDE_CONFIG_DIR", home.join(".claude"))
        .env("CODEX_HOME", home.join(".codex"))
        .env_remove("CLAUDE_CODE_OAUTH_TOKEN")
        .env_remove("ANTHROPIC_API_KEY")
        .env_remove("OPENAI_API_KEY");
    command
}

fn run(binary: &str, args: &[&str], home: &Path) -> Output {
    command(binary, args, home).output().expect("binary runs")
}

/// Starts `watch`, reads its output until a line satisfies `done`, then stops it.
fn first_refresh(args: &[&str], home: &Path, done: impl Fn(&str) -> bool) -> String {
    let mut child = command(env!("CARGO_BIN_EXE_quotadial"), args, home)
        .stdout(Stdio::piped())
        .spawn()
        .expect("watch starts");
    let mut reader = BufReader::new(child.stdout.take().expect("stdout"));
    let mut output = String::new();
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).expect("reads") == 0 {
            break;
        }
        output.push_str(&line);
        if done(&line) {
            break;
        }
    }
    child.kill().expect("stops");
    child.wait().expect("exits");
    output
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("UTF-8")
}

#[test]
fn status_says_no_accounts_found() {
    let home = empty_home("status");
    let output = run(env!("CARGO_BIN_EXE_quotadial"), &["status"], &home);

    assert!(output.status.success(), "exit status {:?}", output.status);
    assert_eq!(stdout(&output), "no accounts found\n");
}

#[test]
fn status_json_reports_no_snapshots() {
    let home = empty_home("status-json");
    let output = run(
        env!("CARGO_BIN_EXE_quotadial"),
        &["status", "--json"],
        &home,
    );

    assert!(output.status.success(), "exit status {:?}", output.status);
    let value: Value = serde_json::from_str(&stdout(&output)).expect("valid JSON");
    assert_eq!(value, json!({ "snapshots": [], "errors": [] }));
}

#[test]
fn qd_is_an_alias_for_quotadial() {
    let home = empty_home("qd");
    let output = run(env!("CARGO_BIN_EXE_qd"), &["status"], &home);

    assert!(output.status.success(), "exit status {:?}", output.status);
    assert_eq!(stdout(&output), "no accounts found\n");
}

#[test]
fn version_names_the_product() {
    let home = empty_home("version");
    let output = run(env!("CARGO_BIN_EXE_quotadial"), &["--version"], &home);

    assert!(output.status.success(), "exit status {:?}", output.status);
    assert_eq!(
        stdout(&output),
        format!("quotadial {}\n", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn watch_json_prints_one_report_per_refresh() {
    let home = empty_home("watch-json");
    let output = first_refresh(&["watch", "--json"], &home, |_| true);

    let value: Value = serde_json::from_str(&output).expect("valid JSON");
    assert_eq!(value, json!({ "snapshots": [], "errors": [] }));
}

#[test]
fn watch_prints_the_status_and_when_it_refreshes() {
    let home = empty_home("watch");
    let output = first_refresh(&["watch"], &home, |line| line.contains("Ctrl-C"));

    assert!(
        output.starts_with("no accounts found\n\nUpdated "),
        "{output}"
    );
    assert!(
        output.ends_with(
            "Refreshes every minute and asks claude at most every 5 min and codex at most every \
             2 min. Press Ctrl-C to stop.\n"
        ),
        "{output}"
    );
    assert!(!output.contains('\x1b'), "no escape codes when piped");
}
