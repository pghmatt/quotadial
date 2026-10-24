//! Runs the real `quotadial` binary's `licence` command in an empty data folder. A debug build
//! trusts the test key, so these keys are signed with it.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use quotadial_core::licence::TEST_SEED;
use quotadial_core::{Licence, Plan};
use serde_json::{Value, json};

fn data_dir(test: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("quotadial-licence-{test}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

fn quotadial(args: &[&str], data: &Path, stdin: Option<&str>) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_quotadial"))
        .args(args)
        .env("QUOTADIAL_DATA_DIR", data)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("binary runs");
    let mut input = child.stdin.take().expect("stdin");
    input
        .write_all(stdin.unwrap_or_default().as_bytes())
        .expect("writes stdin");
    drop(input);
    child.wait_with_output().expect("exits")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn key(id: &str) -> String {
    Licence {
        id: id.into(),
        plan: Plan::Pro,
        issued: "2026-10-24".parse().expect("a date"),
        until: None,
    }
    .sign(&TEST_SEED)
}

#[test]
fn a_key_is_added_shown_and_removed() {
    let data = data_dir("cycle");

    let free = quotadial(&["licence"], &data, None);
    assert!(free.status.success());
    assert_eq!(text(&free.stdout), "QuotaDial Free, with no licence key.\n");

    let added = quotadial(&["licence", "add", &key("cli-1")], &data, None);
    assert!(added.status.success(), "{}", text(&added.stderr));
    assert_eq!(
        text(&added.stdout),
        "Added the licence key: QuotaDial Pro, licence cli-1.\n"
    );

    let shown = quotadial(&["license", "show", "--json"], &data, None);
    let json: Value = serde_json::from_slice(&shown.stdout).expect("JSON");
    assert_eq!(
        json,
        json!({
            "tier": "pro",
            "licence": {"id": "cli-1", "plan": "pro", "issued": "2026-10-24"},
            "text": "QuotaDial Pro, licence cli-1.",
        })
    );

    let removed = quotadial(&["licence", "remove"], &data, None);
    assert_eq!(
        text(&removed.stdout),
        "Removed the licence key: QuotaDial Free, with no licence key.\n"
    );
    let again = quotadial(&["licence", "remove"], &data, None);
    assert!(again.status.success());
    assert_eq!(text(&again.stdout), "There was no licence key to remove.\n");
}

#[test]
fn a_key_can_be_piped_in_wrapped_or_given_in_pieces() {
    let data = data_dir("piped");
    let key = key("cli-2");

    let piped = quotadial(
        &["licence", "add"],
        &data,
        Some(&format!("{}\n{}\n", &key[..60], &key[60..])),
    );
    assert!(piped.status.success(), "{}", text(&piped.stderr));
    assert!(text(&piped.stdout).contains("licence cli-2"));

    let pieces = quotadial(&["licence", "add", &key[..30], &key[30..]], &data, None);
    assert!(pieces.status.success(), "{}", text(&pieces.stderr));
}

#[test]
fn a_key_that_gives_nothing_is_refused_with_the_fix() {
    let data = data_dir("refused");

    let refused = quotadial(
        &["licence", "add", "80e15db5-c796-436b-850c-8f9c98a48abe"],
        &data,
        None,
    );
    assert_eq!(refused.status.code(), Some(1));
    assert_eq!(
        text(&refused.stderr),
        "quotadial: That isn't a QuotaDial licence key, which starts with QD1. Copy the whole key \
         from the email it came in.\n"
    );

    let cut = key("cli-3");
    let cut = quotadial(&["licence", "add", &cut[..cut.len() - 4]], &data, None);
    assert_eq!(cut.status.code(), Some(1));
    assert!(text(&cut.stderr).contains("doesn't check out"));
    assert!(!data.join("licence").exists());
}
