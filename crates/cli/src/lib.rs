//! QuotaDial command-line interface, shared by the `quotadial` and `qd` binaries.

// Printing results is this crate's job.
#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::io::{self, IsTerminal, Write};
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, SecondsFormat, TimeDelta, Utc};
use clap::{Parser, Subcommand};
use quotadial_api::client::Unanswered;
use quotadial_api::{DEFAULT_PORT, Runtime, Token};
use quotadial_core::{ExtraUsage, Provider, Source, Status, StatusReport, collect};
use quotadial_daemon::{DataDir, History, Instance, InstanceError, Poller};
use quotadial_provider_claude::ClaudeProvider;
use quotadial_provider_codex::CodexProvider;

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

/// Every provider QuotaDial ships, in display order.
fn providers() -> Vec<Box<dyn Provider>> {
    vec![
        Box::new(ClaudeProvider::new()),
        Box::new(CodexProvider::new()),
    ]
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

/// What the running daemon last found or, when no daemon is running, a direct fetch, which still
/// keeps every provider's floor through the shared gate. While a daemon runs it is the only thing
/// that polls, so one that does not answer is an error, not a reason to poll around it.
async fn report<P: AsRef<dyn Provider>>(
    providers: &[P],
    dir: Option<&DataDir>,
) -> Result<StatusReport, Unanswered> {
    if let Some(dir) = dir
        && let Some(report) = quotadial_api::client::status(dir).await?
    {
        return Ok(report);
    }
    Ok(collect(providers).await)
}

fn status(json: bool) -> ExitCode {
    let Some(runtime) = runtime() else {
        return ExitCode::FAILURE;
    };
    let report = match runtime.block_on(report(&providers(), DataDir::from_env().as_ref())) {
        Ok(report) => report,
        Err(error) => {
            eprintln!("quotadial: {error}");
            return ExitCode::FAILURE;
        }
    };

    let mut stdout = io::stdout().lock();
    let written = if json {
        write_json(&report, &mut stdout)
    } else {
        write_text(&report, Utc::now(), &mut stdout)
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
    let providers = providers();
    let dir = DataDir::from_env();
    // The Windows console shows escape codes as text unless a program turns them on, which needs
    // unsafe code, so on Windows each refresh simply follows the last.
    let redraw = !json && io::stdout().is_terminal() && !cfg!(windows);
    runtime.block_on(async {
        loop {
            let report = report(&providers, dir.as_ref()).await;
            let mut stdout = io::stdout().lock();
            let written = match &report {
                Ok(report) if json => write_json_line(report, &mut stdout),
                // The JSON lines stay reports; why a refresh has none goes to standard error.
                Err(error) if json => {
                    eprintln!("quotadial: {error}");
                    Ok(())
                }
                _ => write_frame(report.as_ref(), &providers, Utc::now(), redraw, &mut stdout),
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
    let providers = providers();
    let ids: Vec<&str> = providers.iter().map(|provider| provider.id()).collect();
    let dir = DataDir::from_env();
    let served = runtime.block_on(quotadial_api::mcp::serve(
        tokio::io::BufReader::new(tokio::io::stdin()),
        tokio::io::stdout(),
        &ids,
        async || report(&providers, dir.as_ref()).await,
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
    let _instance = match Instance::acquire(&dir) {
        Ok(instance) => instance,
        Err(error @ InstanceError::AlreadyRunning) => {
            eprintln!("quotadial: {error}.");
            return ExitCode::FAILURE;
        }
        Err(error) => {
            eprintln!(
                "quotadial: could not use {}: {error}; check that you can write to it.",
                dir.path().display()
            );
            return ExitCode::FAILURE;
        }
    };
    // A daemon that crashed left its port behind, and another program may hold that port now.
    Runtime::remove(&dir);
    let history = match History::open(&dir.history()) {
        Ok(history) => Arc::new(history),
        Err(error) => {
            eprintln!(
                "quotadial: could not open the history at {}: {error}",
                dir.history().display()
            );
            return ExitCode::FAILURE;
        }
    };
    let token = match Token::load_or_create(&dir) {
        Ok(token) => token,
        Err(error) => {
            eprintln!(
                "quotadial: could not create the API token at {}: {error}; check that you can \
                 write to it.",
                dir.token().display()
            );
            return ExitCode::FAILURE;
        }
    };
    let Some(runtime) = runtime() else {
        return ExitCode::FAILURE;
    };
    let providers: Vec<Arc<dyn Provider>> = providers().into_iter().map(Arc::from).collect();
    let code = runtime.block_on(serve_daemon(&dir, providers, history, token, port));
    Runtime::remove(&dir);
    code
}

/// Listens on `port`, says where, then polls and serves until stopped.
async fn serve_daemon(
    dir: &DataDir,
    providers: Vec<Arc<dyn Provider>>,
    history: Arc<History>,
    token: Token,
    port: u16,
) -> ExitCode {
    let listener = match quotadial_api::bind(port).await {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!(
                "quotadial: could not listen on 127.0.0.1:{port}: {error}; another program may \
                 be using the port, so choose another with --port."
            );
            return ExitCode::FAILURE;
        }
    };
    let announced = listener.local_addr().and_then(|address| {
        let runtime = Runtime {
            pid: std::process::id(),
            port: address.port(),
            started_at: Utc::now(),
        };
        runtime.write(dir).map(|()| runtime)
    });
    let port = match announced {
        Ok(runtime) => runtime.port,
        Err(error) => {
            eprintln!(
                "quotadial: could not write {}: {error}; check that you can write to it.",
                dir.runtime().display()
            );
            return ExitCode::FAILURE;
        }
    };
    say(format_args!(
        "QuotaDial daemon running. It asks {}, keeps history in {}, and is the only thing that \
         polls while it runs.",
        limits(&providers),
        dir.history().display()
    ));
    say(format_args!(
        "Its API is at http://127.0.0.1:{port}, for requests with the token in {}. Press Ctrl-C \
         to stop.",
        dir.token().display()
    ));
    let poller = Poller::new(providers, history);
    let latest = poller.latest();
    let (stop, stopped) = tokio::sync::watch::channel(false);
    let until_stopped = |mut stopped: tokio::sync::watch::Receiver<bool>| async move {
        // A closed channel also means stop.
        let _ = stopped.wait_for(|stop| *stop).await;
    };
    let serve = async {
        let served =
            quotadial_api::serve(listener, latest, token, until_stopped(stopped.clone())).await;
        // A daemon whose API has stopped serves no one, so it stops polling too, and consumers
        // fetch for themselves again.
        let _ = stop.send(true);
        served
    };
    let signalled = async {
        tokio::select! {
            () = shutdown_signal() => {
                let _ = stop.send(true);
            }
            () = until_stopped(stopped.clone()) => {}
        }
    };
    let (served, (), ()) =
        tokio::join!(serve, poller.run(until_stopped(stopped.clone())), signalled);
    match served {
        Ok(()) => {
            say(format_args!("QuotaDial daemon stopped."));
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("quotadial: the API stopped: {error}");
            ExitCode::FAILURE
        }
    }
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
                duration(
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
fn write_frame(
    report: Result<&StatusReport, &Unanswered>,
    providers: &[Box<dyn Provider>],
    now: DateTime<Utc>,
    redraw: bool,
    out: &mut impl Write,
) -> io::Result<()> {
    if redraw {
        // Clear the screen and move to its top-left corner.
        write!(out, "\x1b[2J\x1b[H")?;
    }
    match report {
        Ok(report) => write_text(report, now, out)?,
        Err(error) => writeln!(out, "error: {error}")?,
    }
    writeln!(
        out,
        "\nUpdated {}. Refreshes every minute and asks {}. Press Ctrl-C to stop.",
        timestamp(now),
        limits(providers)
    )
}

/// The report as text, with every time relative to `now`.
fn write_text(report: &StatusReport, now: DateTime<Utc>, out: &mut impl Write) -> io::Result<()> {
    if report.snapshots.is_empty() && report.errors.is_empty() {
        return writeln!(out, "no accounts found");
    }
    for snapshot in &report.snapshots {
        let account = &snapshot.account;
        write!(out, "{} {}", account.provider, account.display_name)?;
        if let Some(plan) = &account.plan {
            write!(out, " ({plan})")?;
        }
        writeln!(out, ": {}", status_label(snapshot.status))?;
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
                resets(window.resets_at, now),
                source_label(window.source),
                ago(window.fetched_at, now),
            )?;
        }
        if let Some(extra) = &snapshot.extra_usage {
            writeln!(out, "  Extra usage: {}", extra_usage_text(extra))?;
        }
        if let Some(error) = &snapshot.error {
            writeln!(out, "  {error}")?;
        }
    }
    for error in &report.errors {
        writeln!(out, "error: {}", error.message)?;
    }
    Ok(())
}

fn timestamp(at: DateTime<Utc>) -> String {
    at.to_rfc3339_opts(SecondsFormat::Secs, true)
}

/// For example `resets in 2 h 10 min`, or `reset 3 min ago` when the numbers predate a reset.
fn resets(resets_at: Option<DateTime<Utc>>, now: DateTime<Utc>) -> String {
    match resets_at {
        None => "no reset time reported".to_owned(),
        Some(at) if at > now => format!("resets in {}", duration(at - now)),
        Some(at) => format!("reset {} ago", duration(now - at)),
    }
}

/// For example `4 min ago`, or `just now`.
fn ago(at: DateTime<Utc>, now: DateTime<Utc>) -> String {
    if now - at < TimeDelta::minutes(1) {
        return "just now".to_owned();
    }
    format!("{} ago", duration(now - at))
}

/// For example `under a minute`, `45 min`, `2 h 10 min` or `3 d 4 h`, rounded down.
fn duration(length: TimeDelta) -> String {
    let minutes = length.num_minutes();
    let (days, hours) = (minutes / (24 * 60), minutes / 60 % 24);
    match (days, hours, minutes % 60) {
        _ if minutes < 1 => "under a minute".to_owned(),
        (0, 0, minutes) => format!("{minutes} min"),
        (0, hours, 0) => format!("{hours} h"),
        (0, hours, minutes) => format!("{hours} h {minutes} min"),
        (days, 0, _) => format!("{days} d"),
        (days, hours, _) => format!("{days} d {hours} h"),
    }
}

/// For example `on, 12.50 of 50.00 USD used (25%)`, `on, 1056.44 credits left`, or `off`.
fn extra_usage_text(extra: &ExtraUsage) -> String {
    if !extra.enabled {
        return "off".to_owned();
    }
    let mut text = "on".to_owned();
    if let (Some(used), Some(limit)) = (extra.used, extra.limit) {
        let currency = extra.currency.as_deref().unwrap_or("USD");
        text.push_str(&format!(", {used:.2} of {limit:.2} {currency} used"));
    }
    if let Some(used_pct) = extra.used_pct {
        text.push_str(&format!(" ({used_pct:.0}%)"));
    }
    if let Some(balance) = extra.balance {
        text.push_str(&format!(", {balance:.2} credits left"));
    }
    text
}

fn status_label(status: Status) -> &'static str {
    match status {
        Status::Ok => "ok",
        Status::Stale => "stale",
        Status::AuthRequired => "sign-in required",
        Status::RateLimited => "rate limited",
        Status::Unsupported => "unsupported",
    }
}

fn source_label(source: Source) -> &'static str {
    match source {
        Source::Api => "the API",
        Source::LocalLog => "local logs",
        Source::Cache => "cache",
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    use async_trait::async_trait;
    use chrono::{DateTime, Utc};
    use quotadial_core::{Account, Error, Result, Snapshot, Window, WindowKind};
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
            write_text(report, now(), &mut out).expect("writes");
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
    fn durations_read_naturally_and_round_down() {
        assert_eq!(duration(TimeDelta::seconds(59)), "under a minute");
        assert_eq!(duration(TimeDelta::minutes(45)), "45 min");
        assert_eq!(duration(TimeDelta::hours(2)), "2 h");
        assert_eq!(
            duration(TimeDelta::minutes(130) + TimeDelta::seconds(59)),
            "2 h 10 min"
        );
        assert_eq!(duration(TimeDelta::days(3)), "3 d");
        assert_eq!(
            duration(TimeDelta::days(3) + TimeDelta::hours(4) + TimeDelta::minutes(5)),
            "3 d 4 h"
        );
    }

    #[test]
    fn reset_times_and_ages_are_relative_to_now() {
        assert_eq!(
            resets(Some(ts("2099-01-01T05:00:00Z")), now()),
            "resets in 4 h 56 min"
        );
        assert_eq!(
            resets(Some(ts("2099-01-01T00:01:00Z")), now()),
            "reset 3 min ago",
            "numbers from before a reset say so"
        );
        assert_eq!(resets(None, now()), "no reset time reported");
        assert_eq!(ago(ts("2099-01-01T00:00:00Z"), now()), "4 min ago");
        assert_eq!(ago(ts("2099-01-01T00:03:30Z"), now()), "just now");
        assert_eq!(
            ago(ts("2099-01-01T00:09:00Z"), now()),
            "just now",
            "a clock that moved back"
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
            write_frame(Ok(&report), &providers, now(), redraw, &mut out).expect("writes");
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

    #[test]
    fn extra_usage_that_is_off_says_so() {
        let extra = ExtraUsage {
            enabled: false,
            used_pct: None,
            used: None,
            limit: None,
            currency: None,
            balance: None,
        };

        assert_eq!(extra_usage_text(&extra), "off");
    }

    #[test]
    fn a_credit_balance_is_shown() {
        let extra = ExtraUsage {
            enabled: true,
            used_pct: None,
            used: None,
            limit: None,
            currency: None,
            balance: Some(1_056.436_375),
        };

        assert_eq!(extra_usage_text(&extra), "on, 1056.44 credits left");
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

        let shown = report(&[Arc::clone(&cli) as Arc<dyn Provider>], Some(&dir))
            .await
            .expect("the daemon answers");

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

        let shown = report(&providers, Some(&dir)).await.expect("fetches");
        assert_eq!(shown.snapshots, vec![snapshot(account(), Source::Api)]);
        assert_eq!(cli.fetches(), 1);

        report(&providers, None).await.expect("fetches");
        assert_eq!(cli.fetches(), 2, "with no data directory at all");
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
