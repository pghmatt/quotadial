//! The daemon's poller: while the daemon runs, it is the only thing that asks providers for usage,
//! and every consumer (the CLI, the local API, the MCP server, later the tray app) reads what it
//! last found.
//!
//! Each account is asked no more often than its provider's floor (5 minutes for Anthropic, 2 for
//! ChatGPT), and providers enforce the floor themselves too, across every process, through the
//! shared gate. The interval adapts to activity, as the PRD asks: an account whose numbers moved in
//! the last 10 minutes is asked at its provider's floor, and a still one every 10 minutes. A change
//! in usage is the activity, so no process watching is needed. The interval counts from when an
//! account's answer came back, never from before its request went out, so the next poll cannot
//! come before the gate allows a request. Accounts are rediscovered every minute, so a new sign-in
//! appears and a removed one goes.

use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, PoisonError, RwLock};
use std::time::Duration;

use chrono::Utc;
use quotadial_core::{Account, Provider, ProviderError, Snapshot, StatusReport};
use tokio::sync::watch;
use tokio::time::Instant;

use crate::History;

/// Numbers that moved this recently keep an account at its provider's floor.
pub const ACTIVE_FOR: Duration = Duration::from_secs(10 * 60);
/// How often a still account is asked, unless its provider's floor is longer.
pub const IDLE_EVERY: Duration = Duration::from_secs(10 * 60);
/// How often accounts are rediscovered.
pub const DISCOVER_EVERY: Duration = Duration::from_secs(60);
/// How often history older than its keeping time is deleted.
const PRUNE_EVERY: Duration = Duration::from_secs(60 * 60);

/// What the poller last found, shared with every consumer.
#[derive(Debug, Clone, Default)]
pub struct Latest(Arc<Shared>);

#[derive(Debug, Default)]
struct Shared {
    providers: RwLock<Vec<ProviderState>>,
    /// Told whenever the report changes, so a consumer such as the tray app need not ask.
    changed: watch::Sender<()>,
}

#[derive(Debug, Clone)]
struct ProviderState {
    /// Why the provider's accounts could not be discovered, or what its discovery could not look
    /// at.
    problems: Vec<ProviderError>,
    accounts: Vec<(Account, Result<Snapshot, ProviderError>)>,
}

impl Latest {
    /// The report every consumer shows: the same shape as `quotadial status --json`.
    #[must_use]
    pub fn report(&self) -> StatusReport {
        let providers = self
            .0
            .providers
            .read()
            .unwrap_or_else(PoisonError::into_inner);
        let mut report = StatusReport::default();
        for provider in providers.iter() {
            report.errors.extend(provider.problems.iter().cloned());
            for (_, result) in &provider.accounts {
                match result {
                    Ok(snapshot) => report.snapshots.push(snapshot.clone()),
                    Err(error) => report.errors.push(error.clone()),
                }
            }
        }
        report.normalized()
    }

    /// A receiver whose `changed()` completes each time the report may have changed. It reports
    /// an error once the poller and every other holder of this `Latest` are gone.
    #[must_use]
    pub fn subscribe(&self) -> watch::Receiver<()> {
        self.0.changed.subscribe()
    }

    fn update(&self, change: impl FnOnce(&mut Vec<ProviderState>)) {
        change(
            &mut self
                .0
                .providers
                .write()
                .unwrap_or_else(PoisonError::into_inner),
        );
        self.0.changed.send_replace(());
    }
}

/// An account the poller asks about.
#[derive(Debug)]
struct Tracked {
    provider: usize,
    account: Account,
    due: Instant,
    /// When its numbers last moved.
    moved_at: Instant,
    last: Option<Snapshot>,
}

/// The daemon's poller.
pub struct Poller {
    providers: Vec<Arc<dyn Provider>>,
    history: Arc<History>,
    latest: Latest,
    tracked: HashMap<(usize, String, Option<String>), Tracked>,
    discover_at: Instant,
    prune_at: Instant,
}

impl std::fmt::Debug for Poller {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Poller")
            .field("providers", &self.providers.len())
            .field("tracked", &self.tracked.len())
            .finish_non_exhaustive()
    }
}

impl Poller {
    /// A poller for `providers`, storing history in `history`. What history already holds is shown
    /// at once, marked as coming from the cache, until the first poll replaces it.
    #[must_use]
    pub fn new(providers: Vec<Arc<dyn Provider>>, history: Arc<History>) -> Self {
        let latest = Latest::default();
        let stored = history.latest().unwrap_or_default();
        latest.update(|states| {
            *states = providers
                .iter()
                .map(|provider| ProviderState {
                    problems: Vec::new(),
                    accounts: stored
                        .iter()
                        .filter(|snapshot| snapshot.account.provider == provider.id())
                        .map(|snapshot| {
                            (
                                snapshot.account.clone(),
                                Ok(snapshot.clone().served_from_cache()),
                            )
                        })
                        .collect(),
                })
                .collect();
        });
        let now = Instant::now();
        Self {
            providers,
            history,
            latest,
            tracked: HashMap::new(),
            discover_at: now,
            prune_at: now,
        }
    }

    /// What the poller last found, to hand to consumers.
    #[must_use]
    pub fn latest(&self) -> Latest {
        self.latest.clone()
    }

    /// Polls until `shutdown` completes. A poll in progress then is abandoned: providers save the
    /// floor before they send a request, so it holds for whoever asks next.
    pub async fn run(mut self, shutdown: impl Future<Output = ()>) {
        tokio::pin!(shutdown);
        loop {
            let wake = tokio::select! {
                wake = self.poll_due() => wake,
                () = &mut shutdown => return,
            };
            tokio::select! {
                () = tokio::time::sleep_until(wake) => {}
                () = &mut shutdown => return,
            }
        }
    }

    /// Does whatever is due now: rediscovers accounts, asks every account that is due, and prunes
    /// old history. Returns when something is next due.
    pub async fn poll_due(&mut self) -> Instant {
        let now = Instant::now();
        if now >= self.prune_at {
            // A failed prune is retried next time; history is not worth stopping for.
            let history = Arc::clone(&self.history);
            let _ = tokio::task::spawn_blocking(move || history.prune(Utc::now())).await;
            self.prune_at = now + PRUNE_EVERY;
        }
        if now >= self.discover_at {
            self.discover(now).await;
            self.discover_at = now + DISCOVER_EVERY;
        }
        let mut due: Vec<_> = self
            .tracked
            .iter()
            .filter(|(_, tracked)| tracked.due <= now)
            .map(|(key, _)| key.clone())
            .collect();
        due.sort();
        for key in due {
            self.poll(&key, now).await;
        }
        self.tracked
            .values()
            .map(|tracked| tracked.due)
            .chain([self.discover_at, self.prune_at])
            .min()
            .unwrap_or(self.discover_at)
    }

    /// Finds every provider's accounts. A new account is due at once; one that has gone is
    /// forgotten. A lookup that fails, such as a keychain that does not answer in time, keeps the
    /// accounts found last: they are still asked about, and the error shows beside them until a
    /// lookup works again.
    async fn discover(&mut self, now: Instant) {
        for (index, provider) in self.providers.iter().enumerate() {
            let blocking = Arc::clone(provider);
            // Discovery may run `/usr/bin/security`, so it runs off the async runtime.
            let found = tokio::task::spawn_blocking(move || blocking.discover_accounts())
                .await
                .unwrap_or_else(|_| {
                    Err(quotadial_core::Error::Credentials {
                        provider: provider.id(),
                        location: "the sign-in".to_owned(),
                        reason: "looking it up stopped unexpectedly; QuotaDial tries again in a \
                                 minute"
                            .to_owned(),
                    })
                });
            let accounts = match found {
                Ok(accounts) => accounts,
                Err(error) => {
                    let error = ProviderError::new(provider.id(), &error);
                    self.latest
                        .update(|states| states[index].problems = vec![error]);
                    continue;
                }
            };
            let warnings: Vec<ProviderError> = provider
                .discovery_warnings()
                .iter()
                .map(|warning| ProviderError::warning(provider.id(), warning))
                .collect();
            self.tracked.retain(|key, _| {
                key.0 != index
                    || accounts.iter().any(|account| {
                        (index, account.account_id.clone(), account.org.clone()) == *key
                    })
            });
            for account in &accounts {
                self.tracked
                    .entry((index, account.account_id.clone(), account.org.clone()))
                    // Its name, plan or sign-in may have changed since it was found.
                    .and_modify(|tracked| tracked.account = account.clone())
                    .or_insert_with(|| Tracked {
                        provider: index,
                        account: account.clone(),
                        due: now,
                        moved_at: now,
                        last: None,
                    });
            }
            self.latest.update(|states| {
                let state = &mut states[index];
                state.problems = warnings;
                state
                    .accounts
                    .retain(|(known, _)| accounts.iter().any(|account| account.same_as(known)));
                for account in &accounts {
                    if !state
                        .accounts
                        .iter()
                        .any(|(known, _)| account.same_as(known))
                    {
                        // Shown as soon as it is found; its first poll follows at once.
                        state.accounts.push((
                            account.clone(),
                            Err(ProviderError {
                                provider: provider.id().to_owned(),
                                message: "QuotaDial has not asked about this account yet."
                                    .to_owned(),
                            }),
                        ));
                    }
                }
            });
        }
    }

    /// Asks about one account and schedules its next poll.
    async fn poll(&mut self, key: &(usize, String, Option<String>), now: Instant) {
        let Some(tracked) = self.tracked.get_mut(key) else {
            return;
        };
        let provider = Arc::clone(&self.providers[tracked.provider]);
        let floor = provider.min_poll_interval();
        let result = match provider.fetch(&tracked.account).await {
            Ok(snapshot) => Ok(snapshot.without_secrets()),
            Err(error) => provider
                .fallback(&tracked.account)
                .map(Snapshot::without_secrets)
                .ok_or_else(|| ProviderError::new(provider.id(), &error)),
        };
        // The gate counts the floor from when the request went out: after the accounts asked
        // before this one in the round, and after finding this one's sign-in. Counting from `now`,
        // when the round began, would bring the next poll a little before the floor allows, and it
        // would only replay the cache.
        let answered = Instant::now();
        match &result {
            Ok(snapshot) => {
                if tracked
                    .last
                    .as_ref()
                    .is_none_or(|last| numbers_moved(last, snapshot))
                {
                    tracked.moved_at = now;
                }
                tracked.last = Some(snapshot.clone());
                let active = now.duration_since(tracked.moved_at) < ACTIVE_FOR;
                tracked.due = answered + if active { floor } else { IDLE_EVERY.max(floor) };
                // Written off the async runtime, so a slow disk never holds up the API. A failed
                // write loses one history point; the numbers are still served.
                let (history, stored) = (Arc::clone(&self.history), snapshot.clone());
                let _ =
                    tokio::task::spawn_blocking(move || history.record(&stored, Utc::now())).await;
            }
            Err(_) => tracked.due = answered + floor,
        }
        let (index, account) = (tracked.provider, tracked.account.clone());
        self.latest.update(|states| {
            let state = &mut states[index];
            match state
                .accounts
                .iter_mut()
                .find(|(known, _)| known.same_as(&account))
            {
                Some(entry) => entry.1 = result,
                None => state.accounts.push((account, result)),
            }
        });
    }
}

/// Whether any window's share used differs between two snapshots of one account, or a window came
/// or went.
fn numbers_moved(before: &Snapshot, after: &Snapshot) -> bool {
    let key = |snapshot: &Snapshot| {
        let mut windows: Vec<(String, Option<String>, u64)> = snapshot
            .windows
            .iter()
            .map(|window| {
                (
                    window.label.clone(),
                    window.scope_model.clone(),
                    // Hundredths of a percent, so a float rounding never counts as a move.
                    (window.used_pct * 100.0).round().max(0.0) as u64,
                )
            })
            .collect();
        windows.sort();
        windows
    };
    key(before) != key(after) || before.status != after.status
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use async_trait::async_trait;
    use chrono::{DateTime, TimeDelta};
    use quotadial_core::{Error, Result, Source, Status, Window, WindowKind};

    use super::*;

    /// A provider whose answers the test sets.
    struct Scripted {
        id: &'static str,
        floor: Duration,
        accounts: Mutex<std::result::Result<Vec<Account>, String>>,
        used_pct: Mutex<std::result::Result<f64, String>>,
        fallback: Option<f64>,
        fetches: AtomicUsize,
        /// Every account asked about, in order.
        fetched: Mutex<Vec<Account>>,
        /// What discovery says it could not look at.
        warnings: Mutex<Vec<String>>,
        /// How long each fetch takes.
        delay: Duration,
    }

    impl Scripted {
        fn new(id: &'static str, floor_minutes: u64) -> Self {
            Self {
                id,
                floor: Duration::from_secs(floor_minutes * 60),
                accounts: Mutex::new(Ok(vec![account(id, "user-1")])),
                used_pct: Mutex::new(Ok(10.0)),
                fallback: None,
                fetches: AtomicUsize::new(0),
                fetched: Mutex::default(),
                warnings: Mutex::default(),
                delay: Duration::ZERO,
            }
        }

        fn fetches(&self) -> usize {
            self.fetches.load(Ordering::SeqCst)
        }

        fn set_used(&self, used_pct: std::result::Result<f64, String>) {
            *self.used_pct.lock().expect("lock") = used_pct;
        }

        fn set_accounts(&self, accounts: std::result::Result<Vec<Account>, String>) {
            *self.accounts.lock().expect("lock") = accounts;
        }
    }

    #[async_trait]
    impl Provider for Scripted {
        fn id(&self) -> &'static str {
            self.id
        }

        fn discover_accounts(&self) -> Result<Vec<Account>> {
            self.accounts
                .lock()
                .expect("lock")
                .clone()
                .map_err(|reason| Error::Credentials {
                    provider: self.id,
                    location: "the sign-in".into(),
                    reason,
                })
        }

        fn discovery_warnings(&self) -> Vec<String> {
            self.warnings.lock().expect("lock").clone()
        }

        async fn fetch(&self, account: &Account) -> Result<Snapshot> {
            let count = self.fetches.fetch_add(1, Ordering::SeqCst);
            self.fetched.lock().expect("lock").push(account.clone());
            tokio::time::sleep(self.delay).await;
            let used_pct = self
                .used_pct
                .lock()
                .expect("lock")
                .clone()
                .map_err(|reason| Error::Request {
                    provider: self.id,
                    reason,
                })?;
            Ok(snapshot(account, used_pct, count, Source::Api))
        }

        fn fallback(&self, account: &Account) -> Option<Snapshot> {
            self.fallback
                .map(|used_pct| snapshot(account, used_pct, 0, Source::LocalLog))
        }

        fn min_poll_interval(&self) -> Duration {
            self.floor
        }
    }

    /// A provider that keeps its floor itself, as the real ones do through the shared gate: it
    /// counts the floor from when it sends a request, which is after it has found the sign-in, and
    /// a fetch sooner than that replays its last answer from the cache. Finding the sign-in takes
    /// `lookups[0]` the first time and `lookups[1]` after, as a first `security` call is slower.
    struct Gated {
        floor: Duration,
        lookups: [Duration; 2],
        state: Mutex<GatedState>,
    }

    #[derive(Default)]
    struct GatedState {
        requests: Vec<Instant>,
        last: Option<Snapshot>,
    }

    impl Gated {
        fn requests(&self) -> Vec<Instant> {
            self.state.lock().expect("lock").requests.clone()
        }
    }

    #[async_trait]
    impl Provider for Gated {
        fn id(&self) -> &'static str {
            "claude"
        }

        fn discover_accounts(&self) -> Result<Vec<Account>> {
            Ok(vec![account("claude", "user-1")])
        }

        async fn fetch(&self, account: &Account) -> Result<Snapshot> {
            let first = self.state.lock().expect("lock").requests.is_empty();
            tokio::time::sleep(self.lookups[usize::from(!first)]).await;
            let mut state = self.state.lock().expect("lock");
            let now = Instant::now();
            if let (Some(requested_at), Some(last)) = (state.requests.last(), &state.last)
                && now < *requested_at + self.floor
            {
                return Ok(last.clone().served_from_cache());
            }
            state.requests.push(now);
            // The numbers move with every request, so the account stays at the floor.
            let count = state.requests.len();
            let snapshot = snapshot(account, count as f64, count, Source::Api);
            state.last = Some(snapshot.clone());
            Ok(snapshot)
        }

        fn fallback(&self, _account: &Account) -> Option<Snapshot> {
            None
        }

        fn min_poll_interval(&self) -> Duration {
            self.floor
        }
    }

    fn account(provider: &str, id: &str) -> Account {
        Account {
            provider: provider.into(),
            account_id: id.into(),
            display_name: "user@example.com".into(),
            plan: None,
            org: None,
            sign_in: None,
        }
    }

    fn base() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2099-01-01T00:00:00Z")
            .expect("valid timestamp")
            .with_timezone(&Utc)
    }

    /// A snapshot whose fetch time moves on with every fetch, as real ones do.
    fn snapshot(account: &Account, used_pct: f64, fetch: usize, source: Source) -> Snapshot {
        Snapshot {
            account: account.clone(),
            windows: vec![Window {
                kind: WindowKind::Session,
                label: "Session".into(),
                used_pct,
                resets_at: None,
                window_seconds: 18_000,
                scope_model: None,
                source,
                fetched_at: base() + TimeDelta::minutes(i64::try_from(fetch).expect("small")),
                unconfirmed: false,
            }],
            extra_usage: None,
            status: Status::Ok,
            error: None,
        }
    }

    fn history() -> (tempfile::TempDir, Arc<History>) {
        let dir = tempfile::tempdir().expect("temp dir");
        let history = History::open(&dir.path().join("history.db")).expect("opens");
        (dir, Arc::new(history))
    }

    fn poller(providers: Vec<Arc<Scripted>>, history: Arc<History>) -> Poller {
        Poller::new(
            providers
                .into_iter()
                .map(|provider| provider as Arc<dyn Provider>)
                .collect(),
            history,
        )
    }

    /// Runs the poller for `minutes`, waking whenever it asks to, and returns the minute of every
    /// pass that fetched.
    async fn run_for(poller: &mut Poller, provider: &Scripted, minutes: u64) -> Vec<u64> {
        let start = Instant::now();
        let mut fetched_at = Vec::new();
        loop {
            let before = provider.fetches();
            let wake = poller.poll_due().await;
            if provider.fetches() > before {
                fetched_at.push(Instant::now().duration_since(start).as_secs() / 60);
            }
            if wake.duration_since(start) > Duration::from_secs(minutes * 60) {
                return fetched_at;
            }
            tokio::time::sleep_until(wake).await;
        }
    }

    #[tokio::test(start_paused = true)]
    async fn moving_numbers_are_asked_about_at_the_providers_floor() {
        let (_dir, history) = history();
        let codex = Arc::new(Scripted::new("codex", 2));
        let mut poller = poller(vec![Arc::clone(&codex)], history);

        let mut used = 10.0;
        let start = Instant::now();
        let mut minutes = Vec::new();
        while Instant::now().duration_since(start) < Duration::from_secs(9 * 60) {
            codex.set_used(Ok(used));
            used += 1.0;
            let before = codex.fetches();
            let wake = poller.poll_due().await;
            if codex.fetches() > before {
                minutes.push(Instant::now().duration_since(start).as_secs() / 60);
            }
            tokio::time::sleep_until(wake).await;
        }

        assert_eq!(minutes, [0, 2, 4, 6, 8]);
    }

    #[tokio::test(start_paused = true)]
    async fn each_poll_comes_once_the_floor_since_the_request_has_passed() {
        let (_dir, history) = history();
        let claude = Arc::new(Gated {
            floor: Duration::from_secs(5 * 60),
            lookups: [Duration::from_secs(2), Duration::from_secs(1)],
            state: Mutex::default(),
        });
        let mut poller = Poller::new(vec![Arc::clone(&claude) as Arc<dyn Provider>], history);

        let start = Instant::now();
        loop {
            let wake = poller.poll_due().await;
            if wake.duration_since(start) > Duration::from_secs(16 * 60) {
                break;
            }
            tokio::time::sleep_until(wake).await;
        }

        let minutes: Vec<u64> = claude
            .requests()
            .iter()
            .map(|at| at.duration_since(start).as_secs() / 60)
            .collect();
        assert_eq!(
            minutes,
            [0, 5, 10, 15],
            "every poll sends a request; none comes early and replays the cache"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn still_numbers_slow_to_every_ten_minutes() {
        let (_dir, history) = history();
        let codex = Arc::new(Scripted::new("codex", 2));
        let mut poller = poller(vec![Arc::clone(&codex)], history);

        let minutes = run_for(&mut poller, &codex, 35).await;

        // At the floor for the first 10 minutes after the last move, then every 10 minutes.
        assert_eq!(minutes, [0, 2, 4, 6, 8, 10, 20, 30]);
    }

    #[tokio::test(start_paused = true)]
    async fn the_report_has_every_account_and_every_failure() {
        let (_dir, history) = history();
        let claude = Arc::new(Scripted::new("claude", 5));
        let codex = Arc::new(Scripted::new("codex", 2));
        codex.set_used(Err("Could not reach ChatGPT.".into()));
        let mut poller = poller(vec![Arc::clone(&claude), Arc::clone(&codex)], history);

        poller.poll_due().await;
        let report = poller.latest().report();

        assert_eq!(report.snapshots.len(), 1);
        assert_eq!(report.snapshots[0].account.provider, "claude");
        assert_eq!(
            report.errors,
            vec![ProviderError {
                provider: "codex".into(),
                message: "codex: request failed: Could not reach ChatGPT.".into(),
            }]
        );
    }

    #[tokio::test(start_paused = true)]
    async fn a_failed_fetch_falls_back_to_the_providers_local_numbers() {
        let (_dir, history) = history();
        let mut codex = Scripted::new("codex", 2);
        codex.fallback = Some(42.0);
        codex.set_used(Err("Could not reach ChatGPT.".into()));
        let codex = Arc::new(codex);
        let mut poller = poller(vec![Arc::clone(&codex)], history);

        poller.poll_due().await;
        let report = poller.latest().report();

        assert!(report.errors.is_empty());
        assert_eq!(report.snapshots[0].windows[0].source, Source::LocalLog);
        assert_eq!(report.snapshots[0].windows[0].used_pct, 42.0);
    }

    #[tokio::test(start_paused = true)]
    async fn sign_ins_that_appear_fail_or_go_are_followed() {
        let (_dir, history) = history();
        let claude = Arc::new(Scripted::new("claude", 5));
        let mut poller = poller(vec![Arc::clone(&claude)], history);
        poller.poll_due().await;

        claude.set_accounts(Ok(vec![
            account("claude", "user-1"),
            account("claude", "user-2"),
        ]));
        tokio::time::advance(DISCOVER_EVERY).await;
        poller.poll_due().await;
        assert_eq!(
            poller.latest().report().snapshots.len(),
            2,
            "a new sign-in is asked at once"
        );

        claude.set_accounts(Err("permission denied; check that you can read it".into()));
        tokio::time::advance(DISCOVER_EVERY).await;
        poller.poll_due().await;
        let report = poller.latest().report();
        assert_eq!(
            report.snapshots.len(),
            2,
            "a failed lookup keeps the accounts found last"
        );
        assert_eq!(
            report.errors[0].message,
            "claude: could not read credentials from the sign-in: permission denied; check that \
             you can read it"
        );

        claude.set_accounts(Ok(Vec::new()));
        tokio::time::advance(DISCOVER_EVERY).await;
        poller.poll_due().await;
        assert_eq!(poller.latest().report(), StatusReport::default());
    }

    #[tokio::test(start_paused = true)]
    async fn what_discovery_could_not_look_at_shows_beside_the_accounts_it_found() {
        let (_dir, history) = history();
        let claude = Arc::new(Scripted::new("claude", 5));
        *claude.warnings.lock().expect("lock") =
            vec!["claude: could not list the Claude Code items in the macOS Keychain.".into()];
        let mut poller = poller(vec![Arc::clone(&claude)], history);

        poller.poll_due().await;
        let report = poller.latest().report();
        assert_eq!(report.snapshots.len(), 1, "the accounts found still show");
        assert_eq!(
            report.errors,
            [ProviderError {
                provider: "claude".into(),
                message: "claude: could not list the Claude Code items in the macOS Keychain."
                    .into(),
            }]
        );

        claude.warnings.lock().expect("lock").clear();
        tokio::time::advance(DISCOVER_EVERY).await;
        poller.poll_due().await;
        assert!(
            poller.latest().report().errors.is_empty(),
            "gone once a discovery looks everywhere"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn a_changed_name_or_sign_in_reaches_the_next_fetch() {
        let (_dir, history) = history();
        let claude = Arc::new(Scripted::new("claude", 5));
        let mut poller = poller(vec![Arc::clone(&claude)], history);
        poller.poll_due().await;

        let mut moved = account("claude", "user-1");
        moved.sign_in = Some("the macOS Keychain item \"Claude Code-credentials-0000000a\"".into());
        claude.set_accounts(Ok(vec![moved.clone()]));
        tokio::time::advance(Duration::from_secs(5 * 60)).await;
        poller.poll_due().await;

        assert_eq!(
            claude.fetched.lock().expect("lock").last(),
            Some(&moved),
            "the account as last found, not as first found"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn history_gets_new_numbers_only() {
        let (_dir, history) = history();
        let codex = Arc::new(Scripted::new("codex", 2));
        let mut poller = poller(vec![Arc::clone(&codex)], Arc::clone(&history));

        run_for(&mut poller, &codex, 5).await;

        // Three fetches, each with a new fetch time.
        assert_eq!(
            history.count(&account("codex", "user-1")).expect("counts"),
            3
        );
    }

    #[tokio::test(start_paused = true)]
    async fn a_restarted_daemon_shows_the_stored_numbers_as_cache_at_once() {
        let (_dir, history) = history();
        let stored = snapshot(&account("claude", "user-1"), 33.0, 0, Source::Api);
        history.record(&stored, Utc::now()).expect("records");

        let poller = poller(vec![Arc::new(Scripted::new("claude", 5))], history);
        let report = poller.latest().report();

        assert_eq!(report.snapshots.len(), 1);
        assert_eq!(report.snapshots[0].windows[0].used_pct, 33.0);
        assert_eq!(report.snapshots[0].windows[0].source, Source::Cache);
    }

    #[tokio::test(start_paused = true)]
    async fn consumers_are_told_when_the_report_changes() {
        let (_dir, history) = history();
        let codex = Arc::new(Scripted::new("codex", 2));
        let mut poller = poller(vec![Arc::clone(&codex)], history);
        let mut changes = poller.latest().subscribe();

        assert!(!changes.has_changed().expect("open"), "nothing new yet");
        poller.poll_due().await;

        assert!(changes.has_changed().expect("open"));
        changes.mark_unchanged();
        assert_eq!(poller.latest().report().snapshots.len(), 1);
    }

    #[tokio::test(start_paused = true)]
    async fn stopping_abandons_a_poll_in_progress() {
        let (_dir, history) = history();
        let mut slow = Scripted::new("claude", 5);
        slow.delay = Duration::from_secs(3600);
        let slow = Arc::new(slow);
        let poller = poller(vec![Arc::clone(&slow)], history);
        let started = Instant::now();

        poller.run(tokio::time::sleep(Duration::from_secs(1))).await;

        assert_eq!(slow.fetches(), 1, "the fetch had started");
        assert!(
            Instant::now().duration_since(started) < Duration::from_secs(2),
            "it did not wait for the fetch to finish"
        );
    }

    #[test]
    fn a_move_is_a_changed_share_or_status() {
        let account = account("claude", "user-1");
        let first = snapshot(&account, 10.0, 0, Source::Api);
        let same_later = snapshot(&account, 10.0, 5, Source::Api);
        let higher = snapshot(&account, 10.5, 5, Source::Api);
        let mut limited = same_later.clone();
        limited.status = Status::RateLimited;

        assert!(!numbers_moved(&first, &same_later));
        assert!(numbers_moved(&first, &higher));
        assert!(numbers_moved(&first, &limited));
    }
}
