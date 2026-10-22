//! Runs the real `quotadial` and `qd` binaries in an empty home directory.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::{Value, json};

/// An empty directory used as the home directory, so no stored login on the machine is found.
fn empty_home(test: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("quotadial-{test}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

fn run(binary: &str, args: &[&str], home: &Path) -> Output {
    Command::new(binary)
        .args(args)
        .env("HOME", home)
        .env("USERPROFILE", home)
        .env("CLAUDE_CONFIG_DIR", home.join(".claude"))
        .env("CODEX_HOME", home.join(".codex"))
        .env_remove("CLAUDE_CODE_OAUTH_TOKEN")
        .env_remove("ANTHROPIC_API_KEY")
        .env_remove("OPENAI_API_KEY")
        .output()
        .expect("binary runs")
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
