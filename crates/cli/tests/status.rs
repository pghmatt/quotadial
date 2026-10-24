//! Runs the real `quotadial` and `qd` binaries in an empty home directory.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdout, Command, Output, Stdio};
use std::time::Duration;

use quotadial_core::licence::TEST_SEED;
use quotadial_core::{Licence, Plan};
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
        .env("QUOTADIAL_DATA_DIR", home.join("data"))
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

/// Starts `daemon --port 0` and returns it with its output after the two lines it starts with.
fn start_daemon(home: &Path) -> (Child, BufReader<ChildStdout>, [String; 3]) {
    let mut daemon = command(
        env!("CARGO_BIN_EXE_quotadial"),
        &["daemon", "--port", "0"],
        home,
    )
    .stdout(Stdio::piped())
    .spawn()
    .expect("the daemon starts");
    let mut reader = BufReader::new(daemon.stdout.take().expect("stdout"));
    let mut lines = [String::new(), String::new(), String::new()];
    for line in &mut lines {
        reader.read_line(line).expect("reads");
    }
    (daemon, reader, lines)
}

/// Stops the daemon as Ctrl-C or a service manager would, where the platform allows, and returns
/// whether it exited cleanly and what it printed last.
fn stop_daemon(
    mut daemon: Child,
    mut reader: BufReader<ChildStdout>,
    signal: &str,
) -> (bool, String) {
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

    assert!(
        line.starts_with(
            "QuotaDial daemon running. It asks claude at most every 5 min and codex at most every 2 \
             min, keeps history in "
        ),
        "{line}"
    );
    assert_eq!(
        costs, "The week's cost at API prices is part of QuotaDial Pro: quotadial.com/pro\n",
        "QuotaDial Free says what it leaves out"
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

    assert!(
        line.starts_with(&format!(
            "Its API is at http://127.0.0.1:{port}, for requests with the token in "
        )),
        "{line}"
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
    let (status, body) = get(port, "/v1/status", Some(&token));
    assert_eq!(status, "HTTP/1.1 200 OK");
    let report: Value = serde_json::from_str(&body).expect("valid JSON");
    assert_eq!(report, json!({ "snapshots": [], "errors": [] }));
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
    assert_eq!(headroom, json!({ "accounts": [], "errors": [] }));

    // `status` asks the daemon: given a token the daemon does not know, it says so rather than
    // fetching for itself, which in this empty home would have printed `no accounts found`.
    std::fs::write(data.join("token"), "0".repeat(64)).expect("writes another token");
    let refused = run(env!("CARGO_BIN_EXE_quotadial"), &["status"], &home);
    assert!(!refused.status.success());
    assert_eq!(stdout(&refused), "");
    let stderr = String::from_utf8(refused.stderr).expect("UTF-8");
    assert!(stderr.contains("refused the token"), "{stderr}");
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
    assert_eq!(value, json!({ "snapshots": [], "errors": [] }));

    let (clean, rest) = stop_daemon(daemon, reader, "-TERM");
    if cfg!(unix) {
        assert!(clean, "a service manager's stop is a clean stop");
        assert_eq!(rest, "QuotaDial daemon stopped.\n");
        assert!(!data.join("daemon.json").exists());
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
    assert_eq!(
        replies[2]["result"]["structuredContent"],
        json!({ "snapshots": [], "errors": [] })
    );
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
