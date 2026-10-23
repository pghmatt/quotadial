//! QuotaDial command-line interface, shared by the `quotadial` and `qd` binaries.

// Printing results is this crate's job.
#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::io::{self, IsTerminal, Write};
use std::process::ExitCode;
use std::time::Duration;

use chrono::{DateTime, SecondsFormat, TimeDelta, Utc};
use clap::{Parser, Subcommand};
use quotadial_core::{ExtraUsage, Provider, Snapshot, Source, Status, Window, redact_secrets};
use quotadial_provider_claude::ClaudeProvider;
use quotadial_provider_codex::CodexProvider;
use serde::Serialize;

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
}

/// How often `watch` refreshes. A fetch inside a provider's floor replays its cache, so no provider
/// is asked more often than its own limit (5 minutes for Anthropic).
const WATCH_EVERY: Duration = Duration::from_secs(60);

/// What `quotadial status` reports: a snapshot per account, plus any provider that failed outright.
#[derive(Debug, Default, Serialize)]
struct StatusReport {
    snapshots: Vec<Snapshot>,
    errors: Vec<ProviderError>,
}

/// A provider that produced no snapshot, for example because its credentials were unreadable.
#[derive(Debug, Serialize)]
struct ProviderError {
    provider: &'static str,
    message: String,
}

impl ProviderError {
    fn new(provider: &dyn Provider, error: &quotadial_core::Error) -> Self {
        Self {
            provider: provider.id(),
            message: redact_secrets(&error.to_string()).into_owned(),
        }
    }
}

/// Runs the CLI and returns the process exit code.
#[must_use]
pub fn run() -> ExitCode {
    match Cli::parse().command {
        Command::Status { json } => status(json),
        Command::Watch { json } => watch(json),
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

fn status(json: bool) -> ExitCode {
    let Some(runtime) = runtime() else {
        return ExitCode::FAILURE;
    };
    let report = runtime.block_on(collect(&providers()));

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
    // The Windows console shows escape codes as text unless a program turns them on, which needs
    // unsafe code, so on Windows each refresh simply follows the last.
    let redraw = !json && io::stdout().is_terminal() && !cfg!(windows);
    runtime.block_on(async {
        loop {
            let report = collect(&providers).await;
            let mut stdout = io::stdout().lock();
            let written = if json {
                write_json_line(&report, &mut stdout)
            } else {
                write_frame(&report, &providers, Utc::now(), redraw, &mut stdout)
            };
            if let Err(error) = written.and_then(|()| stdout.flush()) {
                return write_failed(&error);
            }
            drop(stdout);
            tokio::time::sleep(WATCH_EVERY).await;
        }
    })
}

fn write_failed(error: &io::Error) -> ExitCode {
    // The reader went away, as in `quotadial status | head`. Nothing is left to say.
    if error.kind() == io::ErrorKind::BrokenPipe {
        return ExitCode::SUCCESS;
    }
    eprintln!("quotadial: could not write the status: {error}");
    ExitCode::FAILURE
}

/// Asks every provider for its accounts, then fetches each account, using the provider's local
/// fallback when the fetch fails. Error text is checked for tokens here, whichever provider wrote
/// it, so none reaches the output.
async fn collect(providers: &[Box<dyn Provider>]) -> StatusReport {
    let mut report = StatusReport::default();
    for provider in providers {
        let accounts = match provider.discover_accounts() {
            Ok(accounts) => accounts,
            Err(error) => {
                report
                    .errors
                    .push(ProviderError::new(provider.as_ref(), &error));
                continue;
            }
        };
        for account in accounts {
            match provider.fetch(&account).await {
                Ok(snapshot) => report.snapshots.push(without_secrets(snapshot)),
                Err(error) => match provider.fallback(&account) {
                    Some(snapshot) => report.snapshots.push(without_secrets(snapshot)),
                    None => report
                        .errors
                        .push(ProviderError::new(provider.as_ref(), &error)),
                },
            }
        }
    }
    report
}

/// `snapshot` with any token blanked out of its error text.
fn without_secrets(mut snapshot: Snapshot) -> Snapshot {
    snapshot.error = snapshot
        .error
        .map(|error| redact_secrets(&error).into_owned());
    snapshot
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

/// One `watch` refresh: the status, then when it was updated and how often each provider is
/// asked. `redraw` clears the terminal first.
fn write_frame(
    report: &StatusReport,
    providers: &[Box<dyn Provider>],
    now: DateTime<Utc>,
    redraw: bool,
    out: &mut impl Write,
) -> io::Result<()> {
    if redraw {
        // Clear the screen and move to its top-left corner.
        write!(out, "\x1b[2J\x1b[H")?;
    }
    write_text(report, now, out)?;
    let limits: Vec<String> = providers
        .iter()
        .map(|provider| {
            format!(
                "{} at most every {}",
                provider.id(),
                duration(
                    TimeDelta::from_std(provider.min_poll_interval()).unwrap_or(TimeDelta::MAX)
                )
            )
        })
        .collect();
    writeln!(
        out,
        "\nUpdated {}. Refreshes every minute and asks {}. Press Ctrl-C to stop.",
        timestamp(now),
        limits.join(" and ")
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
        for window in tightest_first(&snapshot.windows) {
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

/// The windows closest to their limits first, and of those, the one that resets soonest.
fn tightest_first(windows: &[Window]) -> Vec<&Window> {
    let mut sorted: Vec<&Window> = windows.iter().collect();
    sorted.sort_by(|a, b| {
        b.used_pct.total_cmp(&a.used_pct).then_with(|| {
            let reset = |window: &Window| window.resets_at.unwrap_or(DateTime::<Utc>::MAX_UTC);
            reset(a).cmp(&reset(b))
        })
    });
    sorted
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

/// For example `on, 12.50 of 50.00 USD used (25%)`, or `off`.
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
    use std::time::Duration;

    use async_trait::async_trait;
    use chrono::{DateTime, Utc};
    use quotadial_core::{Account, Error, Result, Window, WindowKind};
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
    fn the_tightest_window_comes_first() {
        let window = |label: &str, used_pct, resets_at: Option<&str>| Window {
            kind: WindowKind::Weekly,
            label: label.into(),
            used_pct,
            resets_at: resets_at.map(ts),
            window_seconds: 604_800,
            scope_model: None,
            source: Source::Api,
            fetched_at: ts("2099-01-01T00:00:00Z"),
            unconfirmed: false,
        };
        let windows = [
            window("Session", 10.0, Some("2099-01-01T05:00:00Z")),
            window("Weekly", 70.0, Some("2099-01-07T00:00:00Z")),
            window("Weekly (Opus)", 70.0, Some("2099-01-03T00:00:00Z")),
            window("Weekly (unknown reset)", 70.0, None),
        ];

        let order: Vec<&str> = tightest_first(&windows)
            .into_iter()
            .map(|window| window.label.as_str())
            .collect();

        assert_eq!(
            order,
            [
                "Weekly (Opus)",
                "Weekly",
                "Weekly (unknown reset)",
                "Session"
            ]
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
            write_frame(&report, &providers, now(), redraw, &mut out).expect("writes");
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
        };

        assert_eq!(extra_usage_text(&extra), "off");
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
}
