//! Runs the real `quotadial` and `qd` binaries in an empty home directory, as a release builds them
//! or as the example build does (`cargo test --workspace --features fake`, D-117), whose fake
//! provider shows its made-up accounts in any home.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdout, Command, Output, Stdio};
use std::time::Duration;

use quotadial_core::licence::TEST_SEED;
use quotadial_core::{Licence, Plan};
use serde_json::{Value, json};

/// Whether these tests run the example build.
const EXAMPLE: bool = cfg!(feature = "fake");

/// How the daemon and `watch` say whom QuotaDial asks and how often, and whom it asks no one about
/// (P-021).
const ASKS: &str = if EXAMPLE {
    "It asks no one about Example."
} else {
    "It asks OpenAI at most every 2 min, and no one about Claude, Gemini CLI or Antigravity."
};

/// The providers each build asks, by id and as each names itself (P-020): a release has Gemini CLI
/// since D-124, and Antigravity since D-126.
const PROVIDERS: &[(&str, &str)] = if EXAMPLE {
    &[("fake", "Example")]
} else {
    &[
        ("claude", "Claude"),
        ("codex", "Codex"),
        ("gemini", "Gemini CLI"),
        ("antigravity", "Antigravity"),
    ]
};

/// [`PROVIDERS`] as a report lists them.
fn providers() -> Value {
    PROVIDERS
        .iter()
        .map(|(id, name)| json!({ "id": id, "name": name }))
        .collect()
}

/// `parts` in the home folder, as QuotaDial writes a place there: `~` and the system's own
/// separator, so `~/data/history.db` on macOS and Linux and `~\data\history.db` on Windows (D-129).
fn in_home(parts: &[&str]) -> String {
    Path::new("~")
        .join(parts.iter().collect::<PathBuf>())
        .display()
        .to_string()
}

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
        .env("GEMINI_CLI_HOME", home)
        .env(quotadial_core::copy::DATA_DIR_VARIABLE, home.join("data"))
        .env_remove("CLAUDE_CODE_OAUTH_TOKEN")
        .env_remove("ANTHROPIC_API_KEY")
        .env_remove("OPENAI_API_KEY")
        .env_remove("QUOTADIAL_FAKE_ACCOUNTS");
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

/// Adds a QuotaDial Pro key, signed with the test key a debug build trusts, with `licence add`.
fn add_pro_key(home: &Path) {
    let key = Licence {
        id: "cli-test".into(),
        plan: Plan::Pro,
        issued: "2026-10-24".parse().expect("a date"),
        until: None,
    }
    .sign(&TEST_SEED);
    let added = run(
        env!("CARGO_BIN_EXE_quotadial"),
        &["licence", "add", &key],
        home,
    );
    assert!(added.status.success(), "{:?}", added);
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("UTF-8")
}

/// Holds `report`, as `status --json` prints it, to what an empty home shows: nothing, or in the
/// example build its two made-up accounts, each called Example with a plan and no address, every
/// window an example with its age and its reset.
fn assert_empty_home_report(report: &Value) {
    if !EXAMPLE {
        assert_eq!(
            report,
            &json!({ "snapshots": [], "errors": [], "providers": providers() })
        );
        return;
    }
    assert_eq!(report["errors"], json!([]), "{report}");
    assert_eq!(report["providers"], providers(), "{report}");
    let snapshots = report["snapshots"].as_array().expect("snapshots");
    let plans: Vec<&str> = snapshots
        .iter()
        .map(|snapshot| snapshot["account"]["plan"].as_str().expect("a plan"))
        .collect();
    assert_eq!(plans, ["Claude Max 5x", "ChatGPT Pro"], "{report}");
    for snapshot in snapshots {
        let account = &snapshot["account"];
        assert_eq!(account["provider"], "fake");
        assert_eq!(account["display_name"], "Example");
        assert!(account.get("sign_in").is_none(), "{account}");
        assert_eq!(snapshot["status"], "ok");
        let windows = snapshot["windows"].as_array().expect("windows");
        assert!(!windows.is_empty(), "{snapshot}");
        for window in windows {
            assert_eq!(window["source"], "example", "{window}");
            assert!(window["fetched_at"].is_string(), "{window}");
            assert!(window["resets_at"].is_string(), "{window}");
        }
    }
    assert!(!report.to_string().contains('@'), "no address: {report}");
}

#[test]
fn the_binary_is_the_build_these_tests_expect() {
    assert_eq!(
        EXAMPLE,
        quotadial_core::copy::EXAMPLE,
        "test the example build with `--features fake`, which turns it on in the CLI, the provider \
         list and core together"
    );
}

#[cfg(not(feature = "fake"))]
#[test]
fn status_says_no_accounts_found() {
    let home = empty_home("status");
    let output = run(env!("CARGO_BIN_EXE_quotadial"), &["status"], &home);

    assert!(output.status.success(), "exit status {:?}", output.status);
    assert_eq!(stdout(&output), "no accounts found\n");
}

/// Gemini CLI signed in with Google in a made-up home: one account, which says why there are no
/// numbers and where QuotaDial saw the sign-in, in `status`, `status --json` and `doctor` alike
/// (D-118). Every release has it (D-124); the example build has the fake provider alone.
#[cfg(not(feature = "fake"))]
#[test]
fn status_shows_gemini_cli_signed_in_and_says_why_there_are_no_numbers() {
    let home = empty_home("gemini");
    let gemini = home.join(".gemini");
    std::fs::create_dir_all(&gemini).expect("makes .gemini");
    std::fs::write(gemini.join("oauth_creds.json"), "{}").expect("signs in");
    std::fs::write(
        gemini.join("settings.json"),
        r#"{ "security": { "auth": { "selectedType": "oauth-personal" } } }"#,
    )
    .expect("chooses Google");
    let sign_in = in_home(&[".gemini", "oauth_creds.json"]);
    let why = "Google's terms do not let other tools use Gemini CLI's sign-in, so QuotaDial shows \
               no numbers for it. In Gemini CLI, `/model` shows each model's share used and when \
               it resets.";

    let text = run(env!("CARGO_BIN_EXE_quotadial"), &["status"], &home);
    assert!(text.status.success(), "exit status {:?}", text.status);
    // A copy with no licence offers the week's cost only where a provider keeps logs to read it
    // from, and Gemini CLI keeps none QuotaDial reads (D-122, D-125).
    assert_eq!(
        stdout(&text),
        format!("Gemini CLI Google account: unsupported\n  {why}\n")
    );

    let json = run(
        env!("CARGO_BIN_EXE_quotadial"),
        &["status", "--json"],
        &home,
    );
    let report: Value = serde_json::from_str(&stdout(&json)).expect("valid JSON");
    assert_eq!(
        report,
        json!({
            "snapshots": [{
                "account": {
                    "provider": "gemini",
                    "provider_name": "Gemini CLI",
                    "account_id": "google",
                    "display_name": "Google account",
                    "sign_in": sign_in,
                    "sign_in_use": format!(
                        "QuotaDial only checks that Gemini CLI's sign-in is at {sign_in}. It never \
                         opens it."
                    )
                },
                "windows": [],
                "status": "unsupported",
                "error": why
            }],
            "errors": [],
            "providers": providers()
        })
    );

    let doctor = stdout(&run(env!("CARGO_BIN_EXE_quotadial"), &["doctor"], &home));
    assert!(
        doctor.contains(&format!(
            "Accounts (1):\n  Gemini CLI Google account: unsupported\n    sign-in: QuotaDial only \
             checks that Gemini CLI's sign-in is at {sign_in}. It never opens it.\n    {why}\n"
        )),
        "{doctor}"
    );
}

/// The block Antigravity CLI's settings take to run QuotaDial's command (D-126).
#[cfg(not(feature = "fake"))]
const ANTIGRAVITY_BLOCK: &str = r#""statusLine": { "type": "command", "command": "quotadial antigravity status-line", "stack_with_default": true }"#;

/// Runs `quotadial antigravity status-line` with `args` after it, as Antigravity CLI's status line
/// does, with `payload` on standard input.
#[cfg(not(feature = "fake"))]
fn status_line(home: &Path, args: &[&str], payload: &[u8]) -> Output {
    let mut words = vec!["antigravity", "status-line"];
    words.extend_from_slice(args);
    let mut child = command(env!("CARGO_BIN_EXE_quotadial"), &words, home)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the command starts");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(payload)
        .expect("hands over");
    child.wait_with_output().expect("the command ends")
}

/// Antigravity in a made-up home, through the command its CLI's status line runs (D-126): with
/// only Antigravity CLI's folder there, an account says what to add; the command keeps a handover,
/// printing nothing and ending with 0; then `status`, `status --json` and `doctor` show its window
/// from the status line, and the file holds nothing else of what it was handed. Every release has
/// it; the example build has the fake provider alone.
#[cfg(not(feature = "fake"))]
#[test]
fn antigravity_s_status_line_hands_its_numbers_to_status_json_and_doctor() {
    let home = empty_home("antigravity");
    std::fs::create_dir_all(home.join(".gemini").join("antigravity-cli"))
        .expect("makes its folder");
    let settings = "~/.gemini/antigravity-cli/settings.json";
    let waiting = format!(
        "No numbers yet: add `{ANTIGRAVITY_BLOCK}` to {settings}, or put `quotadial antigravity \
         status-line` in front of the status line command already there, and they come the next \
         time Antigravity CLI runs."
    );
    let sign_in_use = format!(
        "QuotaDial never reads Antigravity's sign-in. It keeps only the numbers Antigravity CLI's \
         status line hands it once `{ANTIGRAVITY_BLOCK}` is in {settings}."
    );

    let before = run(env!("CARGO_BIN_EXE_quotadial"), &["status"], &home);
    assert!(before.status.success(), "exit status {:?}", before.status);
    assert_eq!(
        stdout(&before),
        format!("Antigravity: unsupported\n  {waiting}\n")
    );

    let payload = json!({
        "cwd": "/home/someone/a-secret-project",
        "conversation_title": "A private conversation",
        "quota": { "gemini-weekly": { "remaining_fraction": 0.25, "reset_in_seconds": 90_000 } },
        "plan_tier": "Pro",
        "email": "someone@example.com"
    });
    let handed = status_line(&home, &[], payload.to_string().as_bytes());
    assert!(handed.status.success(), "exit status {:?}", handed.status);
    assert!(
        handed.stdout.is_empty() && handed.stderr.is_empty(),
        "{handed:?}"
    );
    let kept = std::fs::read_to_string(home.join("data").join("antigravity.json")).expect("kept");
    assert!(
        !kept.contains("secret") && !kept.contains("private"),
        "{kept}"
    );

    let text = run(env!("CARGO_BIN_EXE_quotadial"), &["status"], &home);
    assert!(text.status.success(), "exit status {:?}", text.status);
    assert_eq!(
        stdout(&text),
        "Antigravity someone@example.com (Pro): ok\n  Weekly (Gemini): 75% used, resets in 1 d, \
         from the CLI's status line just now\nPace needs the daemon: with the QuotaDial app or \
         `quotadial daemon` running, each window also shows how fast it fills.\n"
    );

    let json = run(
        env!("CARGO_BIN_EXE_quotadial"),
        &["status", "--json"],
        &home,
    );
    let report: Value = serde_json::from_str(&stdout(&json)).expect("valid JSON");
    let snapshot = &report["snapshots"][0];
    assert_eq!(
        snapshot["account"],
        json!({
            "provider": "antigravity",
            "provider_name": "Antigravity",
            "account_id": "someone@example.com",
            "display_name": "someone@example.com",
            "plan": "Pro",
            "sign_in_use": sign_in_use
        })
    );
    assert_eq!(snapshot["status"], "ok");
    let window = &snapshot["windows"][0];
    assert_eq!(window["label"], "Weekly (Gemini)");
    assert_eq!(window["kind"], "weekly");
    assert_eq!(window["used_pct"], 75.0);
    assert_eq!(window["source"], "status_line");
    assert_eq!(window["scope_model"], "gemini");
    assert_eq!(report["errors"], json!([]));
    assert_eq!(report["providers"], providers());

    let doctor = stdout(&run(env!("CARGO_BIN_EXE_quotadial"), &["doctor"], &home));
    assert!(
        doctor.contains(&format!(
            "Accounts (1):\n  Antigravity s***@e***.com (Pro): ok, fetched just now from the CLI's \
             status line\n    sign-in: {sign_in_use}\n"
        )),
        "{doctor}"
    );
}

/// A status line the user already had, run by the real binary after QuotaDial keeps its numbers:
/// it gets the same input, and its output and exit status come back unchanged (D-126).
#[cfg(all(unix, not(feature = "fake")))]
#[test]
fn antigravity_s_status_line_runs_the_one_already_there_unchanged() {
    let home = empty_home("antigravity-chained");
    let payload = json!({ "quota": { "3p-5h": { "remaining_fraction": 0.5 } } }).to_string();

    let ran = status_line(
        &home,
        &["/bin/sh", "-c", "cat; printf ' and its own line'; exit 3"],
        payload.as_bytes(),
    );

    assert_eq!(ran.status.code(), Some(3));
    assert_eq!(
        String::from_utf8(ran.stdout).expect("UTF-8"),
        format!("{payload} and its own line")
    );
    assert!(ran.stderr.is_empty());
    assert!(home.join("data").join("antigravity.json").exists());
}

/// The example build's accounts, by `status`: each called Example with its plan, every window
/// saying it is an example, and nothing that reads as an address.
#[cfg(feature = "fake")]
#[test]
fn example_status_names_each_example_and_where_its_numbers_came_from() {
    let home = empty_home("example-status");
    let output = run(env!("CARGO_BIN_EXE_quotadial"), &["status"], &home);

    assert!(output.status.success(), "exit status {:?}", output.status);
    let text = stdout(&output);
    let accounts: Vec<&str> = text
        .lines()
        .filter(|line| !line.starts_with(' '))
        .take(2)
        .collect();
    assert_eq!(
        accounts,
        ["Example (Claude Max 5x): ok", "Example (ChatGPT Pro): ok"],
        "{text}"
    );
    let windows: Vec<&str> = text.lines().filter(|line| line.starts_with("  ")).collect();
    assert_eq!(windows.len(), 5, "{text}");
    for window in windows {
        assert!(
            window.ends_with(", from the example build just now"),
            "{window}"
        );
    }
    assert!(text.starts_with("Example (Claude Max 5x): ok\n  Weekly: 91% used"));
    assert!(!text.contains('@'), "{text}");
}

#[test]
fn status_json_reports_what_an_empty_home_holds() {
    let home = empty_home("status-json");
    let output = run(
        env!("CARGO_BIN_EXE_quotadial"),
        &["status", "--json"],
        &home,
    );

    assert!(output.status.success(), "exit status {:?}", output.status);
    let value: Value = serde_json::from_str(&stdout(&output)).expect("valid JSON");
    assert_empty_home_report(&value);
}

#[test]
fn qd_is_an_alias_for_quotadial() {
    let home = empty_home("qd");
    let output = run(env!("CARGO_BIN_EXE_qd"), &["status"], &home);
    let full = run(env!("CARGO_BIN_EXE_quotadial"), &["status"], &home);

    assert!(output.status.success(), "exit status {:?}", output.status);
    assert_eq!(stdout(&output), stdout(&full));
    if !EXAMPLE {
        assert_eq!(stdout(&output), "no accounts found\n");
    }
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
    assert_empty_home_report(&value);
}

#[test]
fn watch_prints_the_status_and_when_it_refreshes() {
    let home = empty_home("watch");
    let output = first_refresh(&["watch"], &home, |line| line.contains("Ctrl-C"));

    let first = if EXAMPLE {
        "Example (Claude Max 5x): ok\n"
    } else {
        "no accounts found\n\nUpdated "
    };
    assert!(output.starts_with(first), "{output}");
    assert!(
        output.ends_with(&format!(
            "Refreshes every minute. {ASKS} Press Ctrl-C to stop.\n"
        )),
        "{output}"
    );
    assert!(!output.contains('\x1b'), "no escape codes when piped");
}

/// Starts `daemon --port 0` and returns it with its output after the two lines it starts with.
fn start_daemon(home: &Path) -> (Daemon, BufReader<ChildStdout>, [String; 3]) {
    let mut daemon = command(
        env!("CARGO_BIN_EXE_quotadial"),
        &["daemon", "--port", "0"],
        home,
    )
    .stdout(Stdio::piped())
    .spawn()
    .expect("the daemon starts");
    let mut reader = BufReader::new(daemon.stdout.take().expect("stdout"));
    let daemon = Daemon(Some(daemon));
    let mut lines = [String::new(), String::new(), String::new()];
    // The third is QuotaDial Free's line for the week's cost, which only a copy whose providers
    // keep logs prints: a release, never the example build, whose provider keeps none (D-125).
    let printed = if EXAMPLE { 2 } else { 3 };
    for line in lines.iter_mut().take(printed) {
        reader.read_line(line).expect("reads");
    }
    (daemon, reader, lines)
}

/// A daemon a test started, stopped if the test fails before it stops it itself, so a failed
/// test leaves no daemon running and holding the test run's output open.
struct Daemon(Option<Child>);

impl Drop for Daemon {
    fn drop(&mut self) {
        if let Some(mut daemon) = self.0.take() {
            let _ = daemon.kill();
            let _ = daemon.wait();
        }
    }
}

/// Stops the daemon as Ctrl-C or a service manager would, where the platform allows, and returns
/// whether it exited cleanly and what it printed last.
fn stop_daemon(
    mut daemon: Daemon,
    mut reader: BufReader<ChildStdout>,
    signal: &str,
) -> (bool, String) {
    let mut daemon = daemon.0.take().expect("a running daemon");
    if cfg!(unix) {
        let sent = Command::new("kill")
            .args([signal, &daemon.id().to_string()])
            .status()
            .expect("kill runs");
        assert!(sent.success());
    } else {
        daemon.kill().expect("stops");
    }
    let mut rest = String::new();
    reader.read_to_string(&mut rest).expect("reads");
    (daemon.wait().expect("exits").success(), rest)
}

/// `GET /v1/status` once the daemon has read what an empty home holds: at once, or in the example
/// build once its first round has both examples, which takes well under 10 seconds.
fn polled_status(port: u16, token: &str) -> Value {
    let expected = if EXAMPLE { 2 } else { 0 };
    for _ in 0..100 {
        let (status, body) = get(port, "/v1/status", Some(token));
        assert_eq!(status, "HTTP/1.1 200 OK");
        let report: Value = serde_json::from_str(&body).expect("valid JSON");
        if report["snapshots"].as_array().map(Vec::len) == Some(expected) {
            return report;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    panic!("the daemon did not show {expected} accounts within 10 seconds");
}

/// A plain HTTP/1.1 GET, returning the status line and the body.
fn get(port: u16, path: &str, token: Option<&str>) -> (String, String) {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connects");
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .expect("sets a timeout");
    let authorization = token
        .map(|token| format!("Authorization: Bearer {token}\r\n"))
        .unwrap_or_default();
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n{authorization}Connection: close\r\n\r\n"
    )
    .expect("sends");
    let mut response = String::new();
    stream.read_to_string(&mut response).expect("reads");
    let (head, body) = response.split_once("\r\n\r\n").expect("a head and a body");
    (
        head.lines().next().expect("a status line").to_owned(),
        body.to_owned(),
    )
}

#[test]
fn only_one_daemon_runs_and_it_keeps_history_in_the_data_directory() {
    let home = empty_home("daemon");
    let (first, reader, [line, _, costs]) = start_daemon(&home);

    let second = run(
        env!("CARGO_BIN_EXE_quotadial"),
        &["daemon", "--port", "0"],
        &home,
    );
    let (clean, rest) = stop_daemon(first, reader, "-INT");

    // The data folder is in the home folder, which is written `~` (D-129).
    assert_eq!(
        line,
        format!(
            "QuotaDial daemon running. {ASKS} It keeps history in {}, and is the only thing that \
             polls while it runs.\n",
            in_home(&["data", "history.db"])
        )
    );
    assert_eq!(
        costs,
        if EXAMPLE {
            ""
        } else {
            "The week's cost at API prices is part of QuotaDial Pro: quotadial.com/pro\n"
        },
        "QuotaDial Free says what it leaves out, where a provider keeps logs to read it from"
    );
    assert!(home.join("data").join("history.db").exists());
    assert!(home.join("data").join("costs.db").exists());
    assert!(!second.status.success());
    assert_eq!(
        String::from_utf8(second.stderr).expect("UTF-8"),
        "quotadial: a QuotaDial daemon is already running.\n"
    );
    if cfg!(unix) {
        assert!(clean, "Ctrl-C stops the daemon cleanly");
        assert_eq!(rest, "QuotaDial daemon stopped.\n");
        assert!(!home.join("data").join("daemon.json").exists());
    }
}

#[test]
fn the_daemon_serves_its_api_on_127_0_0_1_to_requests_with_its_token() {
    let home = empty_home("daemon-api");
    let (daemon, reader, [_, line, _]) = start_daemon(&home);
    let data = home.join("data");
    let runtime: Value = serde_json::from_slice(
        &std::fs::read(data.join("daemon.json")).expect("the daemon says where it listens"),
    )
    .expect("valid JSON");
    let port = u16::try_from(runtime["port"].as_u64().expect("a port")).expect("a port");
    let token = std::fs::read_to_string(data.join("token")).expect("the token");

    let token_file = in_home(&["data", "token"]);
    assert_eq!(
        line,
        format!(
            "Its API is at http://127.0.0.1:{port}, for requests with the token in {token_file}. \
             Press Ctrl-C to stop.\n"
        ),
        "the home folder written ~ (D-129)"
    );
    assert!(!line.contains(&token), "the token itself is never printed");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for file in ["token", "daemon.json"] {
            let mode = std::fs::metadata(data.join(file))
                .expect("metadata")
                .permissions()
                .mode();
            assert_eq!(mode & 0o777, 0o600, "{file}");
        }
    }

    let (status, body) = get(port, "/v1/status", None);
    assert_eq!(status, "HTTP/1.1 401 Unauthorized");
    assert!(!body.contains("snapshots"));
    let report = polled_status(port, &token);
    // QuotaDial Free says what it leaves out; the accounts are what an empty home holds.
    let accounts = json!({
        "snapshots": report["snapshots"],
        "errors": report["errors"],
        "providers": report["providers"],
    });
    assert_empty_home_report(&accounts);
    let (status, body) = get(port, "/v1/headroom", Some(&token));
    assert_eq!(
        status, "HTTP/1.1 403 Forbidden",
        "headroom is QuotaDial Pro's"
    );
    let refused: Value = serde_json::from_str(&body).expect("valid JSON");
    assert_eq!(refused["pro_feature"], "headroom");
    // A key added while the daemon runs counts at the next request, with no restart.
    add_pro_key(&home);
    let (status, body) = get(port, "/v1/headroom", Some(&token));
    assert_eq!(status, "HTTP/1.1 200 OK");
    let headroom: Value = serde_json::from_str(&body).expect("valid JSON");
    if EXAMPLE {
        assert_eq!(headroom["errors"], json!([]));
        let ranked = headroom["accounts"].as_array().expect("accounts");
        assert_eq!(ranked.len(), 2, "both examples are ranked: {headroom}");
        assert!(
            ranked
                .iter()
                .all(|ranked| ranked["account"]["provider"] == "fake")
        );
    } else {
        assert_eq!(headroom, json!({ "accounts": [], "errors": [] }));
    }

    // `status` asks the daemon: given a token the daemon does not know, it says so rather than
    // fetching for itself, which in this empty home would have printed `no accounts found`.
    std::fs::write(data.join("token"), "0".repeat(64)).expect("writes another token");
    let refused = run(env!("CARGO_BIN_EXE_quotadial"), &["status"], &home);
    assert!(!refused.status.success());
    assert_eq!(stdout(&refused), "");
    assert_eq!(
        String::from_utf8(refused.stderr).expect("UTF-8"),
        format!(
            "quotadial: the QuotaDial daemon is running, but its API at \
             http://127.0.0.1:{port}/v1/status refused the token at {token_file}; restart it with \
             `quotadial daemon`.\n"
        )
    );
    std::fs::write(data.join("token"), &token).expect("restores the token");
    let answered = run(
        env!("CARGO_BIN_EXE_quotadial"),
        &["status", "--json"],
        &home,
    );
    assert!(
        answered.status.success(),
        "exit status {:?}",
        answered.status
    );
    let value: Value = serde_json::from_str(&stdout(&answered)).expect("valid JSON");
    assert_empty_home_report(&value);

    let (clean, rest) = stop_daemon(daemon, reader, "-TERM");
    if cfg!(unix) {
        assert!(clean, "a service manager's stop is a clean stop");
        assert_eq!(rest, "QuotaDial daemon stopped.\n");
        assert!(!data.join("daemon.json").exists());
    }
}

#[test]
fn a_history_that_cannot_be_opened_is_named_with_the_home_folder_written_with_a_tilde() {
    // #52 (D-129): `alerts` opens the history itself, here a folder in its place, in the data
    // folder in the home folder and in one beside it, in a folder whose name starts as the home
    // folder's does.
    let home = empty_home("unopened-history");
    let beside = PathBuf::from(format!("{}a", home.display()));
    for data in [home.join("data"), beside.join("data")] {
        std::fs::create_dir_all(data.join("history.db")).expect("a folder in the history's place");
    }

    let in_home_output = run(env!("CARGO_BIN_EXE_quotadial"), &["alerts"], &home);
    let elsewhere = command(env!("CARGO_BIN_EXE_quotadial"), &["alerts"], &home)
        .env(quotadial_core::copy::DATA_DIR_VARIABLE, beside.join("data"))
        .output()
        .expect("binary runs");

    for (output, place) in [
        (in_home_output, in_home(&["data", "history.db"])),
        (
            elsewhere,
            beside.join("data").join("history.db").display().to_string(),
        ),
    ] {
        assert!(!output.status.success());
        let said = String::from_utf8(output.stderr).expect("UTF-8");
        assert!(
            said.starts_with(&format!(
                "quotadial: could not open the history at {place}: "
            )),
            "{said}"
        );
    }
    for folder in [home, beside] {
        std::fs::remove_dir_all(folder).expect("cleans up");
    }
}

#[test]
fn mcp_answers_an_agent_on_standard_input_and_output() {
    let home = empty_home("mcp");
    let mut mcp = command(env!("CARGO_BIN_EXE_quotadial"), &["mcp"], &home)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the MCP server starts");
    let mut stdin = mcp.stdin.take().expect("stdin");
    for message in [
        json!({ "jsonrpc": "2.0", "id": 0, "method": "initialize", "params": {
            "protocolVersion": "2025-11-25",
            "capabilities": {},
            "clientInfo": { "name": "test", "version": "1.0" },
        } }),
        json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }),
        json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list" }),
        json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {
            "name": "get_usage", "arguments": {},
        } }),
        json!({ "jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {
            "name": "get_account_headroom",
        } }),
    ] {
        writeln!(stdin, "{message}").expect("sends");
    }
    // The agent hangs up, and the server ends.
    drop(stdin);
    let output = mcp.wait_with_output().expect("exits");

    assert!(output.status.success(), "exit status {:?}", output.status);
    assert_eq!(String::from_utf8(output.stderr.clone()).expect("UTF-8"), "");
    let replies: Vec<Value> = stdout(&output)
        .lines()
        .map(|line| serde_json::from_str(line).expect("one JSON message per line"))
        .collect();
    assert_eq!(replies.len(), 4, "the notification gets no reply");
    assert_eq!(replies[0]["result"]["serverInfo"]["name"], "quotadial");
    assert_eq!(replies[0]["result"]["protocolVersion"], "2025-11-25");
    let tools: Vec<&str> = replies[1]["result"]["tools"]
        .as_array()
        .expect("tools")
        .iter()
        .map(|tool| tool["name"].as_str().expect("a name"))
        .collect();
    assert_eq!(tools, ["get_usage", "get_account_headroom", "list_alerts"]);
    let usage = &replies[2]["result"]["structuredContent"];
    assert_empty_home_report(&json!({
        "snapshots": usage["snapshots"],
        "errors": usage["errors"],
        "providers": usage["providers"],
    }));
    assert_eq!(
        replies[3]["result"]["isError"], true,
        "headroom is QuotaDial Pro's"
    );
    assert_eq!(
        replies[3]["result"]["content"][0]["text"],
        "Headroom through the local API and MCP is part of QuotaDial Pro: quotadial.com/pro \
         get_usage gives every window, with its reset, on any copy."
    );
}

#[test]
fn alert_rules_are_added_listed_turned_off_and_removed_in_plain_words() {
    let home = empty_home("alerts");
    let quotadial = env!("CARGO_BIN_EXE_quotadial");

    let added = run(
        quotadial,
        &["alerts", "add", "warn", "me", "at", "80%"],
        &home,
    );
    assert!(added.status.success(), "{added:?}");
    assert_eq!(
        stdout(&added),
        "Added alert rule 3. QuotaDial reads it as: when any window reaches 80%.\n",
        "after the two rules every history starts with"
    );

    let listed = stdout(&run(quotadial, &["alerts"], &home));
    let expected = [
        "Alert rules:",
        "    1  on   warn me when a window passes 90%",
        "             read as: when any window reaches 90%",
        "    2  on   tell me when an account needs to sign in again",
        "             read as: when an account needs to sign in again",
        "    3  on   warn me at 80%",
        "             read as: when any window reaches 80%",
        "",
        "Nothing told in the last week.",
        "",
    ];
    assert_eq!(listed, expected.join("\n"));

    let unread = run(quotadial, &["alerts", "add", "make it stop"], &home);
    assert_eq!(unread.status.code(), Some(1));
    assert!(
        String::from_utf8(unread.stderr)
            .expect("UTF-8")
            .contains("Write it like one of these: \"warn me at 80%\"")
    );

    let off = run(quotadial, &["alerts", "off", "1"], &home);
    assert_eq!(
        stdout(&off),
        "Alert rule 1 is off: warn me when a window passes 90%.\n"
    );
    let removed = run(quotadial, &["alerts", "remove", "3"], &home);
    assert_eq!(stdout(&removed), "Removed alert rule 3.\n");
    let missing = run(quotadial, &["alerts", "remove", "3"], &home);
    assert_eq!(missing.status.code(), Some(1));

    let json: Value = serde_json::from_str(&stdout(&run(
        quotadial,
        &["alerts", "list", "--json"],
        &home,
    )))
    .expect("JSON");
    let rules = json["rules"].as_array().expect("rules");
    assert_eq!(rules.len(), 2);
    assert_eq!(rules[0]["enabled"], false);
    assert_eq!(rules[0]["kind"], "threshold");
    assert_eq!(rules[1]["kind"], "sign_in");
    assert_eq!(json["firings"], json!([]));
    std::fs::remove_dir_all(&home).expect("cleans up");
}

#[test]
fn a_rule_that_waits_for_pro_says_so_in_json_too() {
    let quotadial = env!("CARGO_BIN_EXE_quotadial");
    let home = empty_home("alerts-waits");
    add_pro_key(&home);
    let added = run(
        quotadial,
        &["alerts", "add", "warn me about outages"],
        &home,
    );
    assert!(added.status.success(), "{added:?}");
    let removed = run(quotadial, &["licence", "remove"], &home);
    assert!(removed.status.success(), "{removed:?}");

    let text = stdout(&run(quotadial, &["alerts", "list"], &home));
    assert!(
        text.contains("warn me about outages (waits for Pro)"),
        "{text}"
    );
    let json: Value = serde_json::from_str(&stdout(&run(
        quotadial,
        &["alerts", "list", "--json"],
        &home,
    )))
    .expect("JSON");
    let rules = json["rules"].as_array().expect("rules");
    let (waiting, checked): (Vec<&Value>, Vec<&Value>) = rules
        .iter()
        .partition(|rule| rule["written"] == "warn me about outages");
    assert_eq!(waiting.len(), 1);
    assert_eq!(
        waiting[0]["waits"], true,
        "a script can tell, as the text does"
    );
    assert!(!checked.is_empty());
    assert!(
        checked.iter().all(|rule| rule.get("waits").is_none()),
        "a rule Free checks has no marker: {checked:?}"
    );
    std::fs::remove_dir_all(&home).expect("cleans up");
}

#[test]
fn doctor_prints_what_support_asks_for_with_nothing_secret_in_it() {
    let home = empty_home("doctor");
    let output = run(env!("CARGO_BIN_EXE_quotadial"), &["doctor"], &home);

    assert!(output.status.success(), "exit status {:?}", output.status);
    let text = stdout(&output);
    let first = text.lines().next().expect("a first line");
    assert!(
        first.starts_with(&format!("QuotaDial {} on ", env!("CARGO_PKG_VERSION"))),
        "{first}"
    );
    let accounts: &[&str] = if EXAMPLE {
        &[
            "Accounts (2):",
            "  Example (Claude Max 5x): ok, fetched just now from the example build",
            "  Example (ChatGPT Pro): ok, fetched just now from the example build",
        ]
    } else {
        &["Accounts (0):", "  none found"]
    };
    for line in [
        "Licence: QuotaDial Free, with no licence key.",
        "Daemon: not running",
        "Numbers: from a direct fetch",
        "Settings in effect:",
        "  history_days: 30",
        "  cost_scan: on",
    ]
    .iter()
    .chain(accounts)
    {
        assert!(
            text.lines().any(|shown| shown == *line),
            "{line:?} in\n{text}"
        );
    }
    let home_text = home.display().to_string();
    assert!(
        !text.contains(&home_text),
        "the home folder is written ~:\n{text}"
    );
    // As every message writes a place in the home folder: `~` and the system's own separator, and
    // the rest as the system writes it (D-129).
    let data_folder = format!("Data folder: {}", in_home(&["data"]));
    assert!(text.lines().any(|line| line == data_folder), "{text}");
}

/// Rule 3 of D-117: with no variable naming a data folder, the example build still keeps its own,
/// `quotadial-fake`, so it can never take a real copy's lock, read its token or write its history.
/// On Windows the data directory comes from the system rather than the home folder, so this runs on
/// macOS and Linux.
#[cfg(all(feature = "fake", not(windows)))]
#[test]
fn the_example_build_keeps_its_own_data_folder_when_no_variable_names_one() {
    let home = empty_home("example-folder");
    let own = if cfg!(target_os = "macos") {
        "Library/Application Support/quotadial-fake"
    } else {
        ".local/share/quotadial-fake"
    };
    let quotadial = |args: &[&str]| {
        command(env!("CARGO_BIN_EXE_quotadial"), args, &home)
            .env_remove(quotadial_core::copy::DATA_DIR_VARIABLE)
            .env_remove("QUOTADIAL_DATA_DIR")
            .env_remove("XDG_DATA_HOME")
            .output()
            .expect("binary runs")
    };

    let doctor = stdout(&quotadial(&["doctor"]));
    let added = quotadial(&["alerts", "add", "warn me at 80%"]);

    assert!(
        doctor
            .lines()
            .any(|line| line == format!("Data folder: ~/{own}")),
        "{doctor}"
    );
    assert!(added.status.success(), "{added:?}");
    assert!(home.join(own).join("history.db").exists());
    let real = own.trim_end_matches("-fake");
    assert!(!home.join(real).exists(), "nothing in a real copy's folder");
    std::fs::remove_dir_all(&home).expect("cleans up");
}
