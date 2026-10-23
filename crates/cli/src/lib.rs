//! QuotaDial command-line interface, shared by the `quotadial` and `qd` binaries.

// Printing results is this crate's job.
#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::io::{self, IsTerminal, Write};
use std::process::ExitCode;
use std::time::Duration;

use chrono::{DateTime, SecondsFormat, TimeDelta, Utc};
use clap::{Parser, Subcommand};
use quotadial_api::client::Unanswered;
use quotadial_api::{DEFAULT_PORT, Host, HostError};
use quotadial_core::{Provider, StatusReport, collect, text};
use quotadial_daemon::DataDir;

/// Track Claude and ChatGPT/Codex usage limits.
#[derive(Debug, Parser)]
#[command(name = "quotadial", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Show current usage for every account this machine is signed in to.
    Status {
        /// Print JSON instead of text.
        #[arg(long)]
        json: bool,
    },
    /// Keep showing usage, refreshing every minute. No provider is asked more often than its own
    /// limit, so most refreshes show cached numbers with their age.
    Watch {
        /// Print one line of JSON per refresh instead of text.
        #[arg(long)]
        json: bool,
    },
    /// Run the daemon until Ctrl-C: the one poller every consumer shares, with history and the
    /// local API. While it runs, `status` and `watch` ask it instead of polling.
    Daemon {
        /// The port for the local API, on 127.0.0.1. 0 lets the system choose a free one.
        #[arg(long, default_value_t = DEFAULT_PORT)]
        port: u16,
    },
    /// Run the MCP server named quotadial on standard input and output, for agents such as Claude
    /// Code and Codex. Like `status`, it asks the daemon while one runs.
    Mcp,
}

/// How often `watch` refreshes. A fetch inside a provider's floor replays its cache, so no provider
/// is asked more often than its own limit (5 minutes for Anthropic).
const WATCH_EVERY: Duration = Duration::from_secs(60);

/// Runs the CLI and returns the process exit code.
#[must_use]
pub fn run() -> ExitCode {
    match Cli::parse().command {
        Command::Status { json } => status(json),
        Command::Watch { json } => watch(json),
        Command::Daemon { port } => daemon(port),
        Command::Mcp => mcp(),
    }
}

fn runtime() -> Option<tokio::runtime::Runtime> {
    match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => Some(runtime),
        Err(error) => {
            eprintln!("quotadial: could not start the async runtime: {error}");
            None
        }
    }
}

/// Where a report came from. Only the daemon keeps the history a window's pace is fitted to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Origin {
    Daemon,
    Direct,
}

/// What the running daemon last found or, when no daemon is running, a direct fetch, which still
/// keeps every provider's floor through the shared gate. While a daemon runs it is the only thing
/// that polls, so one that does not answer is an error, not a reason to poll around it.
async fn report<P: AsRef<dyn Provider>>(
    providers: &[P],
    dir: Option<&DataDir>,
) -> Result<(StatusReport, Origin), Unanswered> {
    if let Some(dir) = dir
        && let Some(report) = quotadial_api::client::status(dir).await?
    {
        return Ok((report, Origin::Daemon));
    }
    Ok((collect(providers).await, Origin::Direct))
}

fn status(json: bool) -> ExitCode {
    let Some(runtime) = runtime() else {
        return ExitCode::FAILURE;
    };
    let providers = quotadial_providers::all();
    let (report, origin) = match runtime.block_on(report(&providers, DataDir::from_env().as_ref()))
    {
        Ok(shown) => shown,
        Err(error) => {
            eprintln!("quotadial: {error}");
            return ExitCode::FAILURE;
        }
    };

    let mut stdout = io::stdout().lock();
    let written = if json {
        write_json(&report, &mut stdout)
    } else {
        write_text(&report, origin, Utc::now(), &mut stdout)
    };
    match written {
        Ok(()) if report.errors.is_empty() => ExitCode::SUCCESS,
        Ok(()) => ExitCode::FAILURE,
        Err(error) => write_failed(&error),
    }
}

/// Refreshes forever, until stopped with Ctrl-C or its reader goes away.
fn watch(json: bool) -> ExitCode {
    let Some(runtime) = runtime() else {
        return ExitCode::FAILURE;
    };
    let providers = quotadial_providers::all();
    let dir = DataDir::from_env();
    // The Windows console shows escape codes as text unless a program turns them on, which needs
    // unsafe code, so on Windows each refresh simply follows the last.
    let redraw = !json && io::stdout().is_terminal() && !cfg!(windows);
    runtime.block_on(async {
        loop {
            let report = report(&providers, dir.as_ref()).await;
            let mut stdout = io::stdout().lock();
            let written = match &report {
                Ok((report, _)) if json => write_json_line(report, &mut stdout),
                // The JSON lines stay reports; why a refresh has none goes to standard error.
                Err(error) if json => {
                    eprintln!("quotadial: {error}");
                    Ok(())
                }
                _ => write_frame(
                    report.as_ref().map(|(report, origin)| (report, *origin)),
                    &providers,
                    Utc::now(),
                    redraw,
                    &mut stdout,
                ),
            };
            if let Err(error) = written.and_then(|()| stdout.flush()) {
                return write_failed(&error);
            }
            drop(stdout);
            tokio::time::sleep(WATCH_EVERY).await;
        }
    })
}

/// Serves MCP on standard input and output until the agent closes them.
fn mcp() -> ExitCode {
    let Some(runtime) = runtime() else {
        return ExitCode::FAILURE;
    };
    let providers = quotadial_providers::all();
    let ids: Vec<&str> = providers.iter().map(|provider| provider.id()).collect();
    let dir = DataDir::from_env();
    let served = runtime.block_on(quotadial_api::mcp::serve(
        tokio::io::BufReader::new(tokio::io::stdin()),
        tokio::io::stdout(),
        &ids,
        async || {
            report(&providers, dir.as_ref())
                .await
                .map(|(report, _)| report)
        },
    ));
    match served {
        Ok(()) => ExitCode::SUCCESS,
        // The agent went away mid-answer. Nothing is left to say.
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("quotadial: the MCP server stopped: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Runs the daemon until Ctrl-C, or on Unix until it is told to terminate.
fn daemon(port: u16) -> ExitCode {
    let Some(dir) = DataDir::from_env() else {
        eprintln!(
            "quotadial: the operating system names no data directory; set QUOTADIAL_DATA_DIR."
        );
        return ExitCode::FAILURE;
    };
    let Some(runtime) = runtime() else {
        return ExitCode::FAILURE;
    };
    let providers = quotadial_providers::all();
    runtime.block_on(async {
        let limits = limits(&providers);
        let host = match Host::start(dir, providers, port).await {
            Ok(host) => host,
            Err(error @ HostError::Listen { .. }) => {
                eprintln!("quotadial: {error}, so choose another with --port.");
                return ExitCode::FAILURE;
            }
            Err(error) => {
                eprintln!("quotadial: {error}.");
                return ExitCode::FAILURE;
            }
        };
        say(format_args!(
            "QuotaDial daemon running. It asks {limits}, keeps history in {}, and is the only thing \
             that polls while it runs.",
            host.dir().history().display()
        ));
        say(format_args!(
            "Its API is at http://127.0.0.1:{}, for requests with the token in {}. Press Ctrl-C \
             to stop.",
            host.port(),
            host.dir().token().display()
        ));
        match host.run(shutdown_signal()).await {
            Ok(()) => {
                say(format_args!("QuotaDial daemon stopped."));
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("quotadial: the API stopped: {error}");
                ExitCode::FAILURE
            }
        }
    })
}

/// Prints a line from the daemon, which keeps running when nothing reads its output any more.
fn say(line: std::fmt::Arguments<'_>) {
    let _ = writeln!(io::stdout(), "{line}");
}

/// Completes on Ctrl-C or, on Unix, when the daemon is told to terminate, as a service manager
/// does. If neither can be caught, the daemon runs until it is stopped some other way.
async fn shutdown_signal() {
    let interrupt = async {
        if tokio::signal::ctrl_c().await.is_err() {
            std::future::pending::<()>().await;
        }
    };
    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut terminate) => {
                terminate.recv().await;
            }
            Err(_) => std::future::pending::<()>().await,
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        () = interrupt => {}
        () = terminate => {}
    }
}

/// For example `claude at most every 5 min and codex at most every 2 min`.
fn limits<P: AsRef<dyn Provider>>(providers: &[P]) -> String {
    providers
        .iter()
        .map(|provider| {
            let provider = provider.as_ref();
            format!(
                "{} at most every {}",
                provider.id(),
                text::duration(
                    TimeDelta::from_std(provider.min_poll_interval()).unwrap_or(TimeDelta::MAX)
                )
            )
        })
        .collect::<Vec<_>>()
        .join(" and ")
}

fn write_failed(error: &io::Error) -> ExitCode {
    // The reader went away, as in `quotadial status | head`. Nothing is left to say.
    if error.kind() == io::ErrorKind::BrokenPipe {
        return ExitCode::SUCCESS;
    }
    eprintln!("quotadial: could not write the status: {error}");
    ExitCode::FAILURE
}

fn write_json(report: &StatusReport, out: &mut impl Write) -> io::Result<()> {
    serde_json::to_writer_pretty(&mut *out, report)?;
    writeln!(out)
}

/// The same report as `status --json`, on one line, for `watch --json`.
fn write_json_line(report: &StatusReport, out: &mut impl Write) -> io::Result<()> {
    serde_json::to_writer(&mut *out, report)?;
    writeln!(out)
}

/// One `watch` refresh: the status, or why the running daemon gave none, then when it was updated
/// and how often each provider is asked. `redraw` clears the terminal first.
fn write_frame<P: AsRef<dyn Provider>>(
    report: Result<(&StatusReport, Origin), &Unanswered>,
    providers: &[P],
    now: DateTime<Utc>,
    redraw: bool,
    out: &mut impl Write,
) -> io::Result<()> {
    if redraw {
        // Clear the screen and move to its top-left corner.
        write!(out, "\x1b[2J\x1b[H")?;
    }
    match report {
        Ok((report, origin)) => write_text(report, origin, now, out)?,
        Err(error) => writeln!(out, "error: {error}")?,
    }
    writeln!(
        out,
        "\nUpdated {}. Refreshes every minute and asks {}. Press Ctrl-C to stop.",
        timestamp(now),
        limits(providers)
    )
}

/// The report as text, with every time relative to `now`. A report fetched directly has no pace,
/// which it says once.
fn write_text(
    report: &StatusReport,
    origin: Origin,
    now: DateTime<Utc>,
    out: &mut impl Write,
) -> io::Result<()> {
    if report.snapshots.is_empty() && report.errors.is_empty() {
        return writeln!(out, "no accounts found");
    }
    for snapshot in &report.snapshots {
        let account = &snapshot.account;
        write!(out, "{} {}", account.provider, account.display_name)?;
        if let Some(plan) = &account.plan {
            write!(out, " ({plan})")?;
        }
        writeln!(out, ": {}", text::status_label(snapshot.status))?;
        for window in snapshot.windows_tightest_first() {
            let unconfirmed = if window.unconfirmed {
                ", unconfirmed until the next fetch agrees"
            } else {
                ""
            };
            writeln!(
                out,
                "  {}: {:.0}% used{unconfirmed}, {}, from {} {}",
                window.label,
                window.used_pct,
                text::resets(window.resets_at, now),
                text::source_label(window.source),
                text::ago(window.fetched_at, now),
            )?;
            if let Some(pace) = &window.pace {
                writeln!(out, "    {}", text::pace(pace, window, now))?;
            }
        }
        if let Some(extra) = &snapshot.extra_usage {
            writeln!(out, "  Extra usage: {}", text::extra_usage(extra))?;
        }
        if let Some(error) = &snapshot.error {
            writeln!(out, "  {error}")?;
        }
    }
    for error in &report.errors {
        writeln!(out, "error: {}", error.message)?;
    }
    let windows = report
        .snapshots
        .iter()
        .any(|snapshot| !snapshot.windows.is_empty());
    if origin == Origin::Direct && windows {
        writeln!(
            out,
            "Pace needs the daemon: with the QuotaDial app or `quotadial daemon` running, each \
             window also shows how fast it fills."
        )?;
    }
    Ok(())
}

fn timestamp(at: DateTime<Utc>) -> String {
    at.to_rfc3339_opts(SecondsFormat::Secs, true)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    use async_trait::async_trait;
    use chrono::{DateTime, Utc};
    use quotadial_api::{Runtime, Token};
    use quotadial_core::{
        Account, Error, ExtraUsage, Result, Snapshot, Source, Status, Window, WindowKind,
    };
    use quotadial_daemon::{History, Instance, Poller};
    use serde_json::{Value, json};

    use super::*;

    /// A provider whose every answer is set by the test, to exercise `collect` without credentials.
    struct ScriptedProvider {
        discovery_fails: bool,
        fetch_fails: bool,
        fallback: Option<Snapshot>,
    }

    #[async_trait]
    impl Provider for ScriptedProvider {
        fn id(&self) -> &'static str {
            "scripted"
        }

        fn discover_accounts(&self) -> Result<Vec<Account>> {
            if self.discovery_fails {
                return Err(Error::Credentials {
                    provider: "scripted",
                    location: "/nonexistent/credentials.json".into(),
                    reason: "permission denied".into(),
                });
            }
            Ok(vec![account()])
        }

        async fn fetch(&self, account: &Account) -> Result<Snapshot> {
            if self.fetch_fails {
                return Err(Error::Request {
                    provider: "scripted",
                    reason: "connection refused".into(),
                });
            }
            Ok(snapshot(account.clone(), Source::Api))
        }

        fn fallback(&self, _account: &Account) -> Option<Snapshot> {
            self.fallback.clone()
        }

        fn min_poll_interval(&self) -> Duration {
            Duration::from_secs(300)
        }
    }

    /// A provider that counts its fetches, to show whether something polled.
    #[derive(Default)]
    struct CountingProvider(AtomicUsize);

    impl CountingProvider {
        fn fetches(&self) -> usize {
            self.0.load(Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl Provider for CountingProvider {
        fn id(&self) -> &'static str {
            "scripted"
        }

        fn discover_accounts(&self) -> Result<Vec<Account>> {
            Ok(vec![account()])
        }

        async fn fetch(&self, account: &Account) -> Result<Snapshot> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Ok(snapshot(account.clone(), Source::Api))
        }

        fn fallback(&self, _account: &Account) -> Option<Snapshot> {
            None
        }

        fn min_poll_interval(&self) -> Duration {
            Duration::from_secs(300)
        }
    }

    /// A provider whose discovery found its account but could not look everywhere.
    struct PartlyLookedProvider;

    #[async_trait]
    impl Provider for PartlyLookedProvider {
        fn id(&self) -> &'static str {
            "scripted"
        }

        fn discover_accounts(&self) -> Result<Vec<Account>> {
            Ok(vec![account()])
        }

        fn discovery_warnings(&self) -> Vec<String> {
            vec![format!(
                "scripted: could not list every sign-in, near the token {LEAKED_TOKEN}."
            )]
        }

        async fn fetch(&self, account: &Account) -> Result<Snapshot> {
            Ok(snapshot(account.clone(), Source::Api))
        }

        fn fallback(&self, _account: &Account) -> Option<Snapshot> {
            None
        }

        fn min_poll_interval(&self) -> Duration {
            Duration::from_secs(300)
        }
    }

    fn data_dir() -> (tempfile::TempDir, DataDir) {
        let temp = tempfile::tempdir().expect("temp dir");
        let dir = DataDir::new(temp.path().join("quotadial"));
        (temp, dir)
    }

    /// A fake Anthropic token, never a real one.
    const LEAKED_TOKEN: &str = "sk-ant-oat01-QUOTADIAL-TEST-TOKEN-0123456789";

    /// A provider that passes a token through in its error text, as a server's error message
    /// quoting the token might.
    struct LeakyProvider {
        fetch_fails: bool,
    }

    #[async_trait]
    impl Provider for LeakyProvider {
        fn id(&self) -> &'static str {
            "leaky"
        }

        fn discover_accounts(&self) -> Result<Vec<Account>> {
            Ok(vec![account()])
        }

        async fn fetch(&self, account: &Account) -> Result<Snapshot> {
            let message = format!("the server said: invalid token {LEAKED_TOKEN}");
            if self.fetch_fails {
                return Err(Error::Request {
                    provider: "leaky",
                    reason: message,
                });
            }
            Ok(Snapshot::auth_required(account.clone(), message))
        }

        fn fallback(&self, _account: &Account) -> Option<Snapshot> {
            None
        }

        fn min_poll_interval(&self) -> Duration {
            Duration::from_secs(300)
        }
    }

    fn ts(rfc3339: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(rfc3339)
            .expect("valid timestamp")
            .with_timezone(&Utc)
    }

    fn account() -> Account {
        Account {
            provider: "scripted".into(),
            account_id: "00000000-0000-0000-0000-000000000001".into(),
            display_name: "user@example.com".into(),
            plan: Some("max".into()),
            org: Some("00000000-0000-0000-0000-000000000002".into()),
            sign_in: None,
        }
    }

    fn snapshot(account: Account, source: Source) -> Snapshot {
        Snapshot {
            account,
            windows: vec![Window {
                kind: WindowKind::Session,
                label: "Session".into(),
                used_pct: 12.4,
                resets_at: Some(ts("2099-01-01T05:00:00Z")),
                window_seconds: 18_000,
                scope_model: None,
                source,
                fetched_at: ts("2099-01-01T00:00:00Z"),
                unconfirmed: false,
                pace: None,
            }],
            extra_usage: None,
            status: Status::Ok,
            error: None,
        }
    }

    /// Four minutes after the test snapshots were fetched.
    fn now() -> DateTime<Utc> {
        ts("2099-01-01T00:04:00Z")
    }

    fn render(report: &StatusReport, json: bool) -> String {
        let mut out = Vec::new();
        if json {
            write_json(report, &mut out).expect("writes");
        } else {
            write_text(report, Origin::Daemon, now(), &mut out).expect("writes");
        }
        String::from_utf8(out).expect("UTF-8")
    }

    #[test]
    fn empty_report_says_no_accounts_found() {
        assert_eq!(
            render(&StatusReport::default(), false),
            "no accounts found\n"
        );
    }

    #[test]
    fn empty_report_is_valid_json() {
        let value: Value =
            serde_json::from_str(&render(&StatusReport::default(), true)).expect("valid JSON");

        assert_eq!(value, json!({ "snapshots": [], "errors": [] }));
    }

    #[test]
    fn text_shows_each_number_with_its_source_and_fetch_time() {
        let report = StatusReport {
            snapshots: vec![snapshot(account(), Source::Api)],
            errors: Vec::new(),
        };

        assert_eq!(
            render(&report, false),
            "scripted user@example.com (max): ok\n  Session: 12% used, resets in 4 h 56 min, \
             from the API 4 min ago\n"
        );
    }

    #[test]
    fn text_shows_each_windows_pace_under_it() {
        let mut snapshot = snapshot(account(), Source::Api);
        snapshot.windows[0].pace = Some(quotadial_core::Pace {
            pct_per_hour: 30.0,
            runs_out_at: Some(ts("2099-01-01T02:59:00Z")),
        });
        let report = StatusReport {
            snapshots: vec![snapshot],
            errors: Vec::new(),
        };

        assert_eq!(
            render(&report, false),
            "scripted user@example.com (max): ok\n  Session: 12% used, resets in 4 h 56 min, \
             from the API 4 min ago\n    at 30%/h, runs out in 2 h 55 min, 2 h 1 min before it \
             resets\n"
        );
    }

    #[test]
    fn numbers_fetched_directly_say_once_that_pace_needs_the_daemon() {
        let report = StatusReport {
            snapshots: vec![
                snapshot(account(), Source::Api),
                snapshot(account(), Source::Api),
            ],
            errors: Vec::new(),
        };
        let text = |report: &StatusReport| {
            let mut out = Vec::new();
            write_text(report, Origin::Direct, now(), &mut out).expect("writes");
            String::from_utf8(out).expect("UTF-8")
        };

        let shown = text(&report);
        assert_eq!(shown.matches("Pace needs the daemon").count(), 1);
        assert!(shown.ends_with(
            "Pace needs the daemon: with the QuotaDial app or `quotadial daemon` running, each \
             window also shows how fast it fills.\n"
        ));

        let expired = StatusReport {
            snapshots: vec![Snapshot::auth_required(
                account(),
                "Run `claude` to sign in again.",
            )],
            errors: Vec::new(),
        };
        assert!(
            !text(&expired).contains("Pace"),
            "nothing to say with no windows to show"
        );
    }

    #[test]
    fn text_marks_an_unconfirmed_zero() {
        let mut snapshot = snapshot(account(), Source::Api);
        snapshot.windows[0].used_pct = 0.0;
        snapshot.windows[0].unconfirmed = true;
        let report = StatusReport {
            snapshots: vec![snapshot],
            errors: Vec::new(),
        };

        assert_eq!(
            render(&report, false),
            "scripted user@example.com (max): ok\n  Session: 0% used, unconfirmed until the next \
             fetch agrees, resets in 4 h 56 min, from the API 4 min ago\n"
        );
    }

    #[test]
    fn text_leaves_out_what_the_provider_did_not_report() {
        let mut snapshot = snapshot(account(), Source::Cache);
        snapshot.account.plan = None;
        snapshot.windows[0].resets_at = None;
        let report = StatusReport {
            snapshots: vec![snapshot],
            errors: Vec::new(),
        };

        assert_eq!(
            render(&report, false),
            "scripted user@example.com: ok\n  Session: 12% used, no reset time reported, from \
             cache 4 min ago\n"
        );
    }

    #[test]
    fn text_shows_extra_usage_and_the_reason_for_a_non_ok_status() {
        let mut snapshot = snapshot(account(), Source::Cache);
        snapshot.status = Status::RateLimited;
        snapshot.extra_usage = Some(ExtraUsage {
            enabled: true,
            used_pct: Some(25.0),
            used: Some(12.5),
            limit: Some(50.0),
            currency: Some("USD".into()),
            balance: None,
        });
        snapshot.error = Some("Anthropic is rate-limiting usage requests.".into());
        let report = StatusReport {
            snapshots: vec![snapshot],
            errors: Vec::new(),
        };

        assert_eq!(
            render(&report, false),
            "scripted user@example.com (max): rate limited\n  Session: 12% used, resets in 4 h \
             56 min, from cache 4 min ago\n  Extra usage: on, 12.50 of 50.00 USD used (25%)\n  \
             Anthropic is rate-limiting usage requests.\n"
        );
    }

    #[test]
    fn a_watch_refresh_says_when_it_was_updated_and_how_often_providers_are_asked() {
        let providers: Vec<Box<dyn Provider>> = vec![Box::new(ScriptedProvider {
            discovery_fails: false,
            fetch_fails: false,
            fallback: None,
        })];
        let report = StatusReport {
            snapshots: vec![snapshot(account(), Source::Cache)],
            errors: Vec::new(),
        };
        let frame = |redraw| {
            let mut out = Vec::new();
            write_frame(
                Ok((&report, Origin::Daemon)),
                &providers,
                now(),
                redraw,
                &mut out,
            )
            .expect("writes");
            String::from_utf8(out).expect("UTF-8")
        };

        let redrawn = frame(true);
        assert!(redrawn.starts_with("\x1b[2J\x1b[Hscripted user@example.com (max): ok\n"));
        assert!(redrawn.ends_with(
            "\nUpdated 2099-01-01T00:04:00Z. Refreshes every minute and asks scripted at most \
             every 5 min. Press Ctrl-C to stop.\n"
        ));
        assert!(!frame(false).contains('\x1b'), "no escape codes when piped");
    }

    #[test]
    fn watch_json_is_one_line_per_refresh_with_the_status_json() {
        let report = StatusReport {
            snapshots: vec![snapshot(account(), Source::Api)],
            errors: Vec::new(),
        };
        let mut out = Vec::new();
        write_json_line(&report, &mut out).expect("writes");
        let line = String::from_utf8(out).expect("UTF-8");

        assert_eq!(line.matches('\n').count(), 1);
        assert!(line.ends_with('\n'));
        let parsed: Value = serde_json::from_str(&line).expect("valid JSON");
        let pretty: Value = serde_json::from_str(&render(&report, true)).expect("valid JSON");
        assert_eq!(parsed, pretty);
    }

    #[tokio::test]
    async fn collects_a_snapshot_per_account() {
        let providers: Vec<Box<dyn Provider>> = vec![Box::new(ScriptedProvider {
            discovery_fails: false,
            fetch_fails: false,
            fallback: None,
        })];
        let report = collect(&providers).await;

        assert_eq!(report.snapshots, vec![snapshot(account(), Source::Api)]);
        assert!(report.errors.is_empty());
    }

    #[tokio::test]
    async fn what_discovery_could_not_look_at_shows_beside_the_accounts_found() {
        let providers: Vec<Box<dyn Provider>> = vec![Box::new(PartlyLookedProvider)];

        let report = collect(&providers).await;

        assert_eq!(report.snapshots, vec![snapshot(account(), Source::Api)]);
        assert_eq!(report.errors.len(), 1);
        assert_eq!(report.errors[0].provider, "scripted");
        assert_eq!(
            report.errors[0].message,
            format!(
                "scripted: could not list every sign-in, near the token {}.",
                quotadial_core::REDACTED
            )
        );
    }

    #[tokio::test]
    async fn failed_fetch_uses_the_local_fallback() {
        let fallback = snapshot(account(), Source::LocalLog);
        let providers: Vec<Box<dyn Provider>> = vec![Box::new(ScriptedProvider {
            discovery_fails: false,
            fetch_fails: true,
            fallback: Some(fallback.clone()),
        })];
        let report = collect(&providers).await;

        assert_eq!(report.snapshots, vec![fallback]);
        assert!(report.errors.is_empty());
    }

    #[tokio::test]
    async fn failed_fetch_without_fallback_is_reported() {
        let providers: Vec<Box<dyn Provider>> = vec![Box::new(ScriptedProvider {
            discovery_fails: false,
            fetch_fails: true,
            fallback: None,
        })];
        let report = collect(&providers).await;

        assert!(report.snapshots.is_empty());
        assert_eq!(report.errors.len(), 1);
        assert_eq!(report.errors[0].provider, "scripted");
        assert_eq!(
            report.errors[0].message,
            "scripted: request failed: connection refused"
        );
    }

    #[tokio::test]
    async fn a_token_in_provider_error_text_never_reaches_the_output() {
        let providers: Vec<Box<dyn Provider>> = vec![
            Box::new(LeakyProvider { fetch_fails: false }),
            Box::new(LeakyProvider { fetch_fails: true }),
        ];
        let report = collect(&providers).await;
        assert_eq!(report.snapshots.len(), 1);
        assert_eq!(report.errors.len(), 1);

        for json in [false, true] {
            let output = render(&report, json);
            assert!(!output.contains("sk-ant-"), "a token reached: {output}");
            assert_eq!(
                output.matches(quotadial_core::REDACTED).count(),
                2,
                "both messages keep their text around the blanked token: {output}"
            );
        }
    }

    #[tokio::test]
    async fn unreadable_credentials_are_reported_with_their_location() {
        let providers: Vec<Box<dyn Provider>> = vec![Box::new(ScriptedProvider {
            discovery_fails: true,
            fetch_fails: false,
            fallback: None,
        })];
        let report = collect(&providers).await;

        assert!(report.snapshots.is_empty());
        assert_eq!(
            render(&report, false),
            "error: scripted: could not read credentials from /nonexistent/credentials.json: \
             permission denied\n"
        );
    }
    #[tokio::test]
    async fn while_a_daemon_runs_the_cli_asks_it_instead_of_polling() {
        let (_temp, dir) = data_dir();
        let _instance = Instance::acquire(&dir).expect("holds the daemon lock");
        let token = Token::load_or_create(&dir).expect("makes a token");
        let history = Arc::new(History::open(&dir.history()).expect("opens history"));
        let polled_by_daemon = Arc::new(CountingProvider::default());
        let mut poller = Poller::new(
            vec![Arc::clone(&polled_by_daemon) as Arc<dyn Provider>],
            history,
        );
        poller.poll_due().await;
        let latest = poller.latest();
        let listener = quotadial_api::bind(0).await.expect("binds");
        Runtime {
            pid: std::process::id(),
            port: listener.local_addr().expect("address").port(),
            started_at: Utc::now(),
        }
        .write(&dir)
        .expect("writes the runtime file");
        let (stop, stopped) = tokio::sync::oneshot::channel::<()>();
        let server = tokio::spawn(quotadial_api::serve(
            listener,
            latest.clone(),
            token,
            async {
                let _ = stopped.await;
            },
        ));
        let cli = Arc::new(CountingProvider::default());

        let (shown, origin) = report(&[Arc::clone(&cli) as Arc<dyn Provider>], Some(&dir))
            .await
            .expect("the daemon answers");

        assert_eq!(origin, Origin::Daemon);
        assert_eq!(shown, latest.report());
        assert_eq!(shown.snapshots.len(), 1);
        assert_eq!(cli.fetches(), 0, "the CLI did not poll");
        assert_eq!(polled_by_daemon.fetches(), 1);
        stop.send(()).expect("the server is running");
        server.await.expect("joins").expect("stops cleanly");
    }

    #[tokio::test]
    async fn with_no_daemon_running_the_cli_fetches_directly() {
        let (_temp, dir) = data_dir();
        let cli = Arc::new(CountingProvider::default());
        let providers = [Arc::clone(&cli) as Arc<dyn Provider>];

        let (shown, origin) = report(&providers, Some(&dir)).await.expect("fetches");
        assert_eq!(origin, Origin::Direct);
        assert_eq!(shown.snapshots, vec![snapshot(account(), Source::Api)]);
        assert_eq!(cli.fetches(), 1);

        report(&providers, None).await.expect("fetches");
        assert_eq!(cli.fetches(), 2, "with no data directory at all");
    }

    #[tokio::test]
    async fn when_the_daemon_quits_mid_answer_the_cli_fetches_for_itself() {
        use tokio::io::AsyncReadExt;

        let (_temp, dir) = data_dir();
        let instance = Instance::acquire(&dir).expect("holds the daemon lock");
        Token::load_or_create(&dir).expect("makes a token");
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("binds");
        Runtime {
            pid: std::process::id(),
            port: listener.local_addr().expect("address").port(),
            started_at: Utc::now(),
        }
        .write(&dir)
        .expect("writes the runtime file");
        // The tray app takes the question and quits before answering; its lock goes a moment
        // after its connection, as a process's do.
        let quitting = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("accepts");
            let _ = stream.read(&mut [0_u8; 4096]).await;
            drop(stream);
            tokio::time::sleep(Duration::from_millis(200)).await;
            drop(instance);
        });
        let cli = Arc::new(CountingProvider::default());

        let (shown, _) = report(&[Arc::clone(&cli) as Arc<dyn Provider>], Some(&dir))
            .await
            .expect("no error reaches the user");

        assert_eq!(
            cli.fetches(),
            1,
            "the CLI fetched for itself once the daemon was gone"
        );
        assert_eq!(shown.snapshots, vec![snapshot(account(), Source::Api)]);
        quitting.await.expect("the daemon quit");
    }

    #[tokio::test]
    async fn a_running_daemon_that_does_not_answer_is_shown_instead_of_polling() {
        let (_temp, dir) = data_dir();
        let _instance = Instance::acquire(&dir).expect("holds the daemon lock");
        Token::load_or_create(&dir).expect("makes a token");
        let closed = std::net::TcpListener::bind("127.0.0.1:0").expect("binds");
        let port = closed.local_addr().expect("address").port();
        drop(closed);
        Runtime {
            pid: std::process::id(),
            port,
            started_at: Utc::now(),
        }
        .write(&dir)
        .expect("writes the runtime file");
        let cli = Arc::new(CountingProvider::default());

        let error = report(&[Arc::clone(&cli) as Arc<dyn Provider>], Some(&dir))
            .await
            .expect_err("the daemon does not answer");
        assert_eq!(cli.fetches(), 0, "the CLI did not poll around the daemon");

        let providers: Vec<Box<dyn Provider>> = vec![Box::new(CountingProvider::default())];
        let mut out = Vec::new();
        write_frame(Err(&error), &providers, now(), false, &mut out).expect("writes");
        assert_eq!(
            String::from_utf8(out).expect("UTF-8"),
            format!(
                "error: the QuotaDial daemon is running, but its API at \
                 http://127.0.0.1:{port}/v1/status could not be reached; restart it with \
                 `quotadial daemon`.\n\nUpdated 2099-01-01T00:04:00Z. Refreshes every minute and \
                 asks scripted at most every 5 min. Press Ctrl-C to stop.\n"
            )
        );
    }
}
