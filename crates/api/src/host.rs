//! Running the daemon: the one poller, its history and the local API, holding the daemon lock.
//! `quotadial daemon` and the tray app both run it from here, so they start and stop the same way.
//!
//! Starting claims `daemon.lock`, deletes a `daemon.json` that a crashed daemon left, opens the
//! history, loads or makes the token, listens on `127.0.0.1` and then announces its port in
//! `daemon.json`. Stopping abandons a poll in progress, lets the API finish the requests it has,
//! deletes `daemon.json` and releases the lock last, so a consumer that then finds no daemon may
//! fetch for itself without two pollers ever running at once.

use std::fmt;
use std::future::Future;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;

use chrono::Utc;
use quotadial_core::Provider;
use quotadial_cost::CostScan;
use quotadial_daemon::{
    Config, DataDir, History, HistoryError, Instance, InstanceError, Latest, Licensing, Poller,
    Problem, Recovered, STATUS_PAGES,
};
use tokio::net::TcpListener;
use tokio::sync::watch;

use crate::{Runtime, Token, bind, serve};

/// A started daemon, ready to run.
#[derive(Debug)]
pub struct Host {
    dir: DataDir,
    instance: Instance,
    listener: TcpListener,
    port: u16,
    poller: Poller,
    history: Arc<History>,
    token: Token,
    /// What the copy may do, from the licence key in `dir` (D-100).
    licensing: Licensing,
    /// A damaged history moved aside as the daemon started, and a new one begun (#52).
    recovered: Option<Recovered>,
}

/// Why the daemon could not start.
#[derive(Debug)]
pub enum HostError {
    /// Another daemon, or the tray app, already runs.
    AlreadyRunning,
    /// The data directory or its lock could not be used.
    DataDir { path: PathBuf, error: io::Error },
    /// The history could not be opened.
    History { path: PathBuf, error: HistoryError },
    /// The token could not be read or made.
    Token { path: PathBuf, error: io::Error },
    /// Nothing can listen on the port.
    Listen { port: u16, error: io::Error },
    /// `daemon.json` could not be written.
    Announce { path: PathBuf, error: io::Error },
}

impl fmt::Display for HostError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let writable = "check that you can write to it";
        match self {
            Self::AlreadyRunning => formatter.write_str("a QuotaDial daemon is already running"),
            Self::DataDir { path, error } => {
                write!(
                    formatter,
                    "could not use {}: {error}; {writable}",
                    path.display()
                )
            }
            Self::History { path, error } => write!(
                formatter,
                "could not open the history at {}: {error}",
                path.display()
            ),
            Self::Token { path, error } => write!(
                formatter,
                "could not create the API token at {}: {error}; {writable}",
                path.display()
            ),
            Self::Listen { port, error } => write!(
                formatter,
                "could not listen on 127.0.0.1:{port}: {error}; another program may be using the \
                 port"
            ),
            Self::Announce { path, error } => {
                write!(
                    formatter,
                    "could not write {}: {error}; {writable}",
                    path.display()
                )
            }
        }
    }
}

impl std::error::Error for HostError {}

impl Host {
    /// Claims `dir` for this daemon and listens on `port` (0 lets the system choose). What the
    /// history holds is in [`Host::latest`] at once, marked as coming from the cache, until the
    /// first poll replaces it.
    ///
    /// # Errors
    ///
    /// [`HostError::AlreadyRunning`] while another daemon runs, or whichever step failed.
    pub async fn start(
        dir: DataDir,
        providers: Vec<Arc<dyn Provider>>,
        port: u16,
    ) -> Result<Self, HostError> {
        let instance = Instance::acquire(&dir).map_err(|error| match error {
            InstanceError::AlreadyRunning => HostError::AlreadyRunning,
            InstanceError::Io(error) => HostError::DataDir {
                path: dir.path().to_owned(),
                error,
            },
        })?;
        // A daemon that did not stop cleanly left its `daemon.json`, and one that saw its history
        // damaged noted it: only then is every page of the history read first, which takes a
        // moment on a year of it. A damaged history is moved aside and a new one started (#52).
        let unclean = Runtime::read(&dir).is_some();
        // A daemon that crashed left its port behind, and another program may hold that port now.
        Runtime::remove(&dir);
        let config = Config::load(&dir);
        let check = unclean || History::damage_noted(&dir.history());
        let (history, recovered) = History::open_or_recover(&dir.history(), check, Utc::now())
            .map_err(|error| HostError::History {
                path: dir.history(),
                error,
            })?;
        let history = history.keeping(config.history_days);
        let token = Token::load_or_create(&dir).map_err(|error| HostError::Token {
            path: dir.token(),
            error,
        })?;
        let listener = bind(port)
            .await
            .map_err(|error| HostError::Listen { port, error })?;
        let announce = |error| HostError::Announce {
            path: dir.runtime(),
            error,
        };
        let port = listener.local_addr().map_err(announce)?.port();
        Runtime {
            pid: std::process::id(),
            port,
            started_at: Utc::now(),
        }
        .write(&dir)
        .map_err(announce)?;
        let history = Arc::new(history);
        let licensing = Licensing::new(Some(&dir));
        let mut poller = Poller::new(providers, Arc::clone(&history), licensing.clone())
            .with_webhooks(config.webhooks);
        if let Some(recovered) = &recovered {
            poller.latest().set_problem(
                Problem::Recovered,
                Some(recovered.line(std::env::home_dir().as_deref())),
            );
        }
        if config.status_pages {
            poller = poller.with_status_pages(
                STATUS_PAGES
                    .iter()
                    .map(|(provider, url)| ((*provider).to_owned(), (*url).to_owned()))
                    .collect(),
            );
        }
        // The cost scan's database is a cache: one that cannot be opened means no costs, not no
        // daemon. On QuotaDial Free the poller leaves the logs unread until a licence is added.
        if config.cost_scan
            && let Ok(scan) = CostScan::open(&dir.costs())
        {
            poller = poller.with_costs(scan);
        }
        Ok(Self {
            dir,
            instance,
            listener,
            port,
            poller,
            history,
            token,
            licensing,
            recovered,
        })
    }

    /// The history the daemon keeps, for a consumer in the same process, such as the tray app, to
    /// read the alert rules and their firings from the same file.
    #[must_use]
    pub fn history(&self) -> Arc<History> {
        Arc::clone(&self.history)
    }

    /// The damaged history moved aside as the daemon started, if one was, and where.
    #[must_use]
    pub fn recovered(&self) -> Option<&Recovered> {
        self.recovered.as_ref()
    }

    /// The port the API listens on, at `127.0.0.1`.
    #[must_use]
    pub fn port(&self) -> u16 {
        self.port
    }

    /// The data directory the daemon claimed.
    #[must_use]
    pub fn dir(&self) -> &DataDir {
        &self.dir
    }

    /// Whether the daemon reads the CLIs' logs for the week's cost (D-097): the setting is on and
    /// `costs.db` opened. On QuotaDial Free the logs wait for a licence.
    #[must_use]
    pub fn scans_costs(&self) -> bool {
        self.poller.scans_costs()
    }

    /// What the copy may do, from the licence key in its data folder.
    #[must_use]
    pub fn licensing(&self) -> &Licensing {
        &self.licensing
    }

    /// What the poller last found, for consumers in the same process, such as the tray app.
    #[must_use]
    pub fn latest(&self) -> Latest {
        self.poller.latest()
    }

    /// Polls and serves until `shutdown` completes or the API stops, then stops in the order the
    /// module describes.
    ///
    /// # Errors
    ///
    /// When the API stopped on its own.
    pub async fn run(self, shutdown: impl Future<Output = ()>) -> io::Result<()> {
        let Self {
            dir,
            instance,
            listener,
            poller,
            history,
            token,
            licensing,
            ..
        } = self;
        let latest = poller.latest();
        let (stop, stopped) = watch::channel(false);
        let until_stopped = |mut stopped: watch::Receiver<bool>| async move {
            // A closed channel also means stop.
            let _ = stopped.wait_for(|stop| *stop).await;
        };
        let serving = async {
            let served = serve(
                listener,
                latest,
                history,
                token,
                licensing,
                until_stopped(stopped.clone()),
            )
            .await;
            // A daemon whose API has stopped serves no one, so it stops polling too, and consumers
            // fetch for themselves again.
            let _ = stop.send(true);
            served
        };
        let signalled = async {
            tokio::select! {
                () = shutdown => {
                    let _ = stop.send(true);
                }
                () = until_stopped(stopped.clone()) => {}
            }
        };
        let (served, (), ()) = tokio::join!(
            serving,
            poller.run(until_stopped(stopped.clone())),
            signalled
        );
        Runtime::remove(&dir);
        // The lock goes last: until now this process could still have been polling.
        drop(instance);
        served
    }
}
