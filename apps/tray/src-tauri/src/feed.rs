//! Where the app's numbers come from. With no daemon running, the app runs it, through the same
//! [`Host`] as `quotadial daemon`: one poller for the app, the CLI and the MCP server, with the
//! lock and the API. With a standalone daemon already running, the app follows it through the
//! local API, as the CLI does, and takes over when it stops.

use std::sync::Arc;
use std::time::Duration;

use quotadial_api::{DEFAULT_PORT, Host, HostError, client};
use quotadial_core::{Provider, StatusReport};
use quotadial_daemon::DataDir;
use tokio::sync::watch;

/// What the app learns.
#[derive(Debug, Clone, PartialEq)]
pub enum Update {
    /// New numbers.
    Report(StatusReport),
    /// No numbers can be had right now: why, naming the fix.
    Problem(String),
    /// Where the numbers come from now.
    Role(Role),
}

/// Whether the app runs the daemon or follows one already running.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// The app runs the daemon, with its API on this port.
    Running { port: u16 },
    /// The app follows a daemon started with `quotadial daemon`.
    Following,
}

/// How often things are tried again.
#[derive(Debug, Clone, Copy)]
pub struct Timing {
    /// How often a standalone daemon is asked, while the app follows it.
    pub follow_every: Duration,
    /// How long after a failed start the app tries again.
    pub retry_after: Duration,
}

impl Default for Timing {
    fn default() -> Self {
        Self {
            follow_every: Duration::from_secs(15),
            retry_after: Duration::from_secs(60),
        }
    }
}

/// Runs until `quit` turns true, sending every [`Update`] to `send`. When it returns, the daemon it
/// ran, if any, has stopped and released its lock.
pub async fn run(
    dir: DataDir,
    providers: impl Fn() -> Vec<Arc<dyn Provider>>,
    send: impl Fn(Update),
    timing: Timing,
    mut quit: watch::Receiver<bool>,
) {
    let mut port = DEFAULT_PORT;
    loop {
        if *quit.borrow() {
            return;
        }
        match Host::start(dir.clone(), providers(), port).await {
            Ok(host) => {
                if host_until_quit(host, &send, &mut quit).await {
                    return;
                }
                // The API stopped on its own. Start again.
            }
            Err(HostError::AlreadyRunning) => {
                if follow(&dir, &send, timing, &mut quit).await {
                    return;
                }
                // The standalone daemon stopped. Take over.
            }
            // Something else has the usual port; any free one will do, since consumers read the
            // port from `daemon.json`.
            Err(HostError::Listen { .. }) if port != 0 => port = 0,
            Err(error) => {
                send(Update::Problem(format!(
                    "QuotaDial could not start: {error}. It tries again in a minute."
                )));
                if wait(timing.retry_after, &mut quit).await {
                    return;
                }
            }
        }
    }
}

/// Runs `host`, sending its reports, until `quit`. Returns true when it stopped because of `quit`.
async fn host_until_quit(
    host: Host,
    send: &impl Fn(Update),
    quit: &mut watch::Receiver<bool>,
) -> bool {
    send(Update::Role(Role::Running { port: host.port() }));
    let latest = host.latest();
    let mut changes = latest.subscribe();
    send(Update::Report(latest.report()));
    let forward = async {
        while changes.changed().await.is_ok() {
            send(Update::Report(latest.report()));
        }
    };
    let quitting = {
        let mut quit = quit.clone();
        async move {
            let _ = quit.wait_for(|quit| *quit).await;
        }
    };
    tokio::select! {
        _ = host.run(quitting) => {}
        () = forward => {}
    }
    *quit.borrow()
}

/// Follows a standalone daemon until it stops (false) or `quit` (true).
async fn follow(
    dir: &DataDir,
    send: &impl Fn(Update),
    timing: Timing,
    quit: &mut watch::Receiver<bool>,
) -> bool {
    send(Update::Role(Role::Following));
    loop {
        match client::status(dir).await {
            Ok(Some(report)) => send(Update::Report(report)),
            Ok(None) => return false,
            Err(unanswered) => send(Update::Problem(unanswered.to_string())),
        }
        if wait(timing.follow_every, quit).await {
            return true;
        }
    }
}

/// Waits for `duration`, or until `quit`. Returns true for `quit`.
async fn wait(duration: Duration, quit: &mut watch::Receiver<bool>) -> bool {
    tokio::select! {
        () = tokio::time::sleep(duration) => {}
        // The app going away, and with it the sender, also means quit.
        _ = quit.wait_for(|quit| *quit) => return true,
    }
    *quit.borrow()
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use async_trait::async_trait;
    use chrono::{DateTime, Utc};
    use quotadial_api::Runtime;
    use quotadial_core::{Account, Result, Snapshot, Source, Status, Window, WindowKind};
    use quotadial_daemon::Instance;

    use super::*;

    /// A provider with one account whose session window is the given share used.
    struct Fixed(f64);

    #[async_trait]
    impl Provider for Fixed {
        fn id(&self) -> &'static str {
            "claude"
        }

        fn discover_accounts(&self) -> Result<Vec<Account>> {
            Ok(vec![Account {
                provider: "claude".into(),
                account_id: "user-1".into(),
                display_name: "user@example.com".into(),
                plan: None,
                org: None,
            }])
        }

        async fn fetch(&self, account: &Account) -> Result<Snapshot> {
            let at = DateTime::parse_from_rfc3339("2099-01-01T00:00:00Z")
                .expect("valid timestamp")
                .with_timezone(&Utc);
            Ok(Snapshot {
                account: account.clone(),
                windows: vec![Window {
                    kind: WindowKind::Session,
                    label: "Session".into(),
                    used_pct: self.0,
                    resets_at: None,
                    window_seconds: 18_000,
                    scope_model: None,
                    source: Source::Api,
                    fetched_at: at,
                    unconfirmed: false,
                }],
                extra_usage: None,
                status: Status::Ok,
                error: None,
            })
        }

        fn fallback(&self, _account: &Account) -> Option<Snapshot> {
            None
        }

        fn min_poll_interval(&self) -> Duration {
            Duration::from_secs(300)
        }
    }

    fn fixed(used_pct: f64) -> impl Fn() -> Vec<Arc<dyn Provider>> {
        move || vec![Arc::new(Fixed(used_pct)) as Arc<dyn Provider>]
    }

    fn quick() -> Timing {
        Timing {
            follow_every: Duration::from_millis(50),
            retry_after: Duration::from_millis(50),
        }
    }

    /// The updates the app received, shared with the feed.
    #[derive(Clone, Default)]
    struct Received(Arc<Mutex<Vec<Update>>>);

    impl Received {
        fn sender(&self) -> impl Fn(Update) + use<> {
            let received = self.clone();
            move |update| received.0.lock().expect("lock").push(update)
        }

        /// The share used in the last report, if any.
        fn last_used_pct(&self) -> Option<f64> {
            self.0
                .lock()
                .expect("lock")
                .iter()
                .rev()
                .find_map(|update| match update {
                    Update::Report(report) => report
                        .snapshots
                        .first()
                        .and_then(|snapshot| snapshot.windows.first())
                        .map(|window| window.used_pct),
                    Update::Problem(_) | Update::Role(_) => None,
                })
        }

        fn roles(&self) -> Vec<Role> {
            self.0
                .lock()
                .expect("lock")
                .iter()
                .filter_map(|update| match update {
                    Update::Role(role) => Some(*role),
                    _ => None,
                })
                .collect()
        }

        async fn until(&self, used_pct: f64) {
            for _ in 0..200 {
                if self.last_used_pct() == Some(used_pct) {
                    return;
                }
                tokio::time::sleep(Duration::from_millis(25)).await;
            }
            panic!("never saw {used_pct}%: {:?}", self.last_used_pct());
        }
    }

    fn data_dir() -> (tempfile::TempDir, DataDir) {
        let temp = tempfile::tempdir().expect("temp dir");
        let dir = DataDir::new(temp.path().join("quotadial"));
        (temp, dir)
    }

    #[tokio::test]
    async fn with_no_daemon_running_the_app_runs_it_and_stops_it_cleanly() {
        let (_temp, dir) = data_dir();
        let received = Received::default();
        let (quit, quitting) = watch::channel(false);
        let app = tokio::spawn(run(
            dir.clone(),
            fixed(40.0),
            received.sender(),
            quick(),
            quitting,
        ));

        received.until(40.0).await;
        assert!(received.roles().contains(&Role::Running {
            port: Runtime::read(&dir).expect("announced").port
        }));
        assert!(Instance::is_running(&dir), "the app holds the daemon lock");
        assert!(Runtime::read(&dir).is_some(), "and says where its API is");
        assert!(
            client::status(&dir).await.expect("answers").is_some(),
            "the CLI reads the app's numbers"
        );

        quit.send(true).expect("running");
        app.await.expect("the feed stops");
        assert!(!Instance::is_running(&dir), "the lock is released");
        assert_eq!(Runtime::read(&dir), None);
    }

    #[tokio::test]
    async fn with_a_daemon_running_the_app_follows_it_then_takes_over() {
        let (_temp, dir) = data_dir();
        let standalone = Host::start(dir.clone(), fixed(40.0)(), 0)
            .await
            .expect("the standalone daemon starts");
        let (stop_standalone, standalone_stopped) = tokio::sync::oneshot::channel::<()>();
        let standalone = tokio::spawn(standalone.run(async {
            let _ = standalone_stopped.await;
        }));
        let received = Received::default();
        let (quit, quitting) = watch::channel(false);
        let app = tokio::spawn(run(
            dir.clone(),
            fixed(75.0),
            received.sender(),
            quick(),
            quitting,
        ));

        received.until(40.0).await;
        assert_eq!(received.roles(), [Role::Following]);

        stop_standalone.send(()).expect("running");
        standalone.await.expect("joins").expect("stops cleanly");
        received.until(75.0).await;
        assert!(Instance::is_running(&dir), "the app took over the lock");

        quit.send(true).expect("running");
        app.await.expect("the feed stops");
        assert!(!Instance::is_running(&dir));
    }

    #[tokio::test]
    async fn a_taken_port_moves_the_app_to_a_free_one() {
        let (_temp, dir) = data_dir();
        let taken = std::net::TcpListener::bind(("127.0.0.1", DEFAULT_PORT));
        let received = Received::default();
        let (quit, quitting) = watch::channel(false);
        let app = tokio::spawn(run(
            dir.clone(),
            fixed(40.0),
            received.sender(),
            quick(),
            quitting,
        ));

        received.until(40.0).await;
        let port = Runtime::read(&dir).expect("announced").port;
        if taken.is_ok() {
            assert_ne!(port, DEFAULT_PORT);
        }

        quit.send(true).expect("running");
        app.await.expect("the feed stops");
    }
}
