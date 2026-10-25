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
//! appears and a removed one goes. Every answer goes to the history, and each window of an `ok`
//! answer gets its pace from it (P-016). The alert rules are checked after every round of polls and
//! every minute between, for a time before a reset (D-093). Every 10 minutes the CLIs' logs are
//! read for the week's API-equivalent cost (D-097).
//!
//! What QuotaDial Pro adds is asked of the copy's licence at every round (D-100): the accounts each
//! provider shows, the history kept, the kinds of rule checked, the webhooks sent to, and the cost
//! scan. A licence added, removed or ended counts within a minute, or at once when a consumer in
//! the same process nudges the poller, as the app's Settings does.

use std::collections::{BTreeMap, HashMap};
use std::future::Future;
use std::path::PathBuf;
use std::sync::{Arc, PoisonError, RwLock};
use std::time::Duration;

use chrono::{DateTime, Utc};
use quotadial_core::{
    Account, Cost, LeftOut, ProFeature, Provider, ProviderError, ProviderName, Snapshot, Status,
    StatusPage, StatusReport, Tier,
};
use quotadial_cost::CostScan;
use tokio::sync::{Notify, watch};
use tokio::time::Instant;

use crate::{
    Deliverer, History, HistoryError, KEEP_DAYS, Licensing, STATUS_EVERY, StatusReader, Webhook,
    deliver_due, licence,
};

/// Numbers that moved this recently keep an account at its provider's floor.
pub const ACTIVE_FOR: Duration = Duration::from_secs(10 * 60);
/// How often a still account is asked, unless its provider's floor is longer.
pub const IDLE_EVERY: Duration = Duration::from_secs(10 * 60);
/// How often accounts are rediscovered.
pub const DISCOVER_EVERY: Duration = Duration::from_secs(60);
/// How often history older than its keeping time is deleted.
const PRUNE_EVERY: Duration = Duration::from_secs(60 * 60);
/// How soon an account is asked again when its numbers could not be refreshed, as while the
/// network is down, so they are current soon after it is back. The provider's floor still decides
/// whether a request goes out; until it allows one, a provider replays its last answer.
pub const RETRY_UNREACHED: Duration = Duration::from_secs(60);
/// How much further the wall clock moves than the poller's own between two rounds before the
/// poller takes it that the machine slept, or the clock jumped ahead, and asks everything again
/// at once. On macOS and Linux the poller's clock stops while the machine sleeps.
pub const WOKE_AFTER: Duration = Duration::from_secs(2 * 60);

/// Where the poller reads the wall clock, which a test moves on alone to stand for a machine that
/// slept.
type WallClock = Arc<dyn Fn() -> DateTime<Utc> + Send + Sync>;
/// How often the alert rules are checked between polls, for a time before a reset.
pub const ALERTS_EVERY: Duration = Duration::from_secs(60);
/// How often the CLIs' logs are read for the week's cost. Each reading reads only what the logs
/// gained since the last.
pub const COSTS_EVERY: Duration = Duration::from_secs(10 * 60);

/// What the poller last found, shared with every consumer.
#[derive(Debug, Clone, Default)]
pub struct Latest(Arc<Shared>);

#[derive(Debug, Default)]
struct Shared {
    providers: RwLock<Vec<ProviderState>>,
    /// What the providers' status pages last said (D-096).
    status_pages: RwLock<Vec<StatusPage>>,
    /// The week's cost of each provider, from the last scan of the logs (D-097).
    costs: RwLock<Costs>,
    /// What the copy leaves out because it is QuotaDial Free (D-100).
    left_out: RwLock<Vec<LeftOut>>,
    /// What is wrong with QuotaDial itself, each in one line (D-107).
    problems: RwLock<BTreeMap<Problem, String>>,
    /// Woken when the licence may have changed, so the poller asks again at once.
    nudged: Notify,
    /// Told whenever the report changes, so a consumer such as the tray app need not ask.
    changed: watch::Sender<()>,
    /// The newest firing of an alert rule to tell, so the tray app can tell it.
    fired: watch::Sender<i64>,
}

/// Something wrong with QuotaDial itself, which the report says in one line while it lasts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Problem {
    /// The history could not be read, so a new one was started (#52). Said until the daemon stops.
    Recovered,
    /// The history could not be saved, as on a full disk, or SQLite found it damaged while the
    /// daemon ran. Said until a save works again.
    Saving,
}

/// The week's costs, and the generation of scans that may still keep what they find. Clearing the
/// costs, as when the copy becomes QuotaDial Free, starts a new generation, so a scan that was
/// already reading keeps nothing (#56). Both happen under the one lock, so neither can slip between
/// the other's check and change.
#[derive(Debug, Default)]
struct Costs {
    generation: u64,
    kept: Vec<Cost>,
}

#[derive(Debug, Clone)]
struct ProviderState {
    /// The provider's id and its own name, which the report lists (P-020).
    provider: ProviderName,
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
            report.providers.push(provider.provider.clone());
            report.errors.extend(provider.problems.iter().cloned());
            for (_, result) in &provider.accounts {
                match result {
                    Ok(snapshot) => report.snapshots.push(snapshot.clone()),
                    Err(error) => report.errors.push(error.clone()),
                }
            }
        }
        report.status_pages = self
            .0
            .status_pages
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        report.costs = self
            .0
            .costs
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .kept
            .clone();
        report.left_out = self
            .0
            .left_out
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        report.problems = self
            .0
            .problems
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .values()
            .cloned()
            .collect();
        report.normalized()
    }

    /// Tells the poller the licence may have changed, as when the app's Settings adds or removes
    /// a key, so it asks again now rather than within a minute.
    pub fn nudge(&self) {
        self.0.nudged.notify_one();
    }

    /// Says `line` in the report's problems for `problem`, or nothing more of it with `None`, and
    /// tells consumers when that changed.
    pub fn set_problem(&self, problem: Problem, line: Option<String>) {
        let mut problems = self
            .0
            .problems
            .write()
            .unwrap_or_else(PoisonError::into_inner);
        let changed = match line {
            Some(line) => problems.insert(problem, line.clone()).as_ref() != Some(&line),
            None => problems.remove(&problem).is_some(),
        };
        drop(problems);
        if changed {
            self.0.changed.send_replace(());
        }
    }

    /// Keeps `left_out` as what the copy leaves out, and tells consumers when it changed.
    fn set_left_out(&self, left_out: Vec<LeftOut>) {
        let mut kept = self
            .0
            .left_out
            .write()
            .unwrap_or_else(PoisonError::into_inner);
        if *kept != left_out {
            *kept = left_out;
            drop(kept);
            self.0.changed.send_replace(());
        }
    }

    /// Keeps `costs` as the week's, and tells consumers.
    /// The generation a scan beginning now belongs to.
    fn costs_generation(&self) -> u64 {
        self.0
            .costs
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .generation
    }

    /// Keeps `costs`, found by a scan that began in `generation`, and tells consumers, unless the
    /// costs were cleared since the scan began. Returns whether they were kept.
    fn set_costs(&self, generation: u64, costs: Vec<Cost>) -> bool {
        let mut kept = self.0.costs.write().unwrap_or_else(PoisonError::into_inner);
        if kept.generation != generation {
            return false;
        }
        kept.kept = costs;
        drop(kept);
        self.0.changed.send_replace(());
        true
    }

    /// Clears the costs and starts a new generation, so a scan already reading keeps nothing, and
    /// tells consumers when there were costs to clear.
    fn clear_costs(&self) {
        let mut kept = self.0.costs.write().unwrap_or_else(PoisonError::into_inner);
        kept.generation += 1;
        let had = !kept.kept.is_empty();
        kept.kept.clear();
        drop(kept);
        if had {
            self.0.changed.send_replace(());
        }
    }

    /// Keeps `page` as its provider's status, and tells consumers.
    fn set_status_page(&self, page: StatusPage) {
        {
            let mut pages = self
                .0
                .status_pages
                .write()
                .unwrap_or_else(PoisonError::into_inner);
            pages.retain(|known| known.provider != page.provider);
            pages.push(page);
            pages.sort_by(|a, b| a.provider.cmp(&b.provider));
        }
        self.0.changed.send_replace(());
    }

    /// A receiver whose `changed()` completes each time the report may have changed. It reports
    /// an error once the poller and every other holder of this `Latest` are gone.
    #[must_use]
    pub fn subscribe(&self) -> watch::Receiver<()> {
        self.0.changed.subscribe()
    }

    /// A receiver whose `changed()` completes each time an alert rule has fired, holding the
    /// newest firing's id. The firings themselves are in the history.
    #[must_use]
    pub fn subscribe_firings(&self) -> watch::Receiver<i64> {
        self.0.fired.subscribe()
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

/// The status pages a poller reads, and its reader.
#[derive(Clone)]
struct StatusPages {
    reader: StatusReader,
    /// Each page, as `(provider, url)`.
    pages: Arc<[(String, String)]>,
}

/// The daemon's poller.
pub struct Poller {
    providers: Vec<Arc<dyn Provider>>,
    history: Arc<History>,
    latest: Latest,
    /// What the copy may do, asked at every round (D-100).
    licensing: Licensing,
    /// The tier at the last round, so a change is acted on at once.
    tier: Option<Tier>,
    tracked: HashMap<(usize, String, Option<String>), Tracked>,
    discover_at: Instant,
    prune_at: Instant,
    alerts_at: Instant,
    /// Where told alerts are also sent (D-094).
    webhooks: Arc<[Webhook]>,
    /// The status pages to read, when they are read (D-096).
    status: Option<StatusPages>,
    status_at: Instant,
    /// Held while the status pages are being read.
    reading: Arc<tokio::sync::Mutex<()>>,
    /// The cost scan, when the logs are read (D-097).
    costs: Option<Arc<CostScan>>,
    costs_at: Instant,
    /// Held while the logs are being read, so a long first reading is never begun twice.
    scanning: Arc<tokio::sync::Mutex<()>>,
    deliverer: Option<Deliverer>,
    /// Held while alerts are being sent, so a slow webhook never holds up polling and two rounds
    /// never send the same alert at once.
    delivering: Arc<tokio::sync::Mutex<()>>,
    wall: WallClock,
    /// When the last round began, by the poller's clock and by the wall clock.
    last_round: Option<(Instant, DateTime<Utc>)>,
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
    /// A poller for `providers`, storing history in `history`, doing what `licensing` allows. What
    /// history already holds is shown at once, marked as coming from the cache, until the first
    /// poll replaces it.
    #[must_use]
    pub fn new(
        providers: Vec<Arc<dyn Provider>>,
        history: Arc<History>,
        licensing: Licensing,
    ) -> Self {
        let latest = Latest::default();
        *latest
            .0
            .status_pages
            .write()
            .unwrap_or_else(PoisonError::into_inner) = history.status_pages().unwrap_or_default();
        let stored = history.latest().unwrap_or_default();
        latest.update(|states| {
            *states = providers
                .iter()
                .map(|provider| ProviderState {
                    provider: ProviderName::of(provider.as_ref()),
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
            licensing,
            tier: None,
            tracked: HashMap::new(),
            discover_at: now,
            prune_at: now,
            alerts_at: now + ALERTS_EVERY,
            webhooks: Arc::new([]),
            deliverer: None,
            delivering: Arc::default(),
            status: None,
            status_at: now,
            reading: Arc::default(),
            costs: None,
            costs_at: now,
            scanning: Arc::default(),
            wall: Arc::new(Utc::now),
            last_round: None,
        }
    }

    /// The poller, reading the wall clock from `wall`, which tests move on alone.
    #[cfg(test)]
    fn with_wall(mut self, wall: WallClock) -> Self {
        self.wall = wall;
        self
    }

    /// The poller, also reading the logs of the providers it has accounts with into `scan` every
    /// [`COSTS_EVERY`], for the week's cost (D-097).
    #[must_use]
    pub fn with_costs(mut self, scan: CostScan) -> Self {
        self.costs = Some(Arc::new(scan));
        self
    }

    /// The poller, also reading each provider's status page in `pages`, as `(provider, url)`,
    /// every [`STATUS_EVERY`] while it has an account with that provider (D-096).
    #[must_use]
    pub fn with_status_pages(mut self, pages: Vec<(String, String)>) -> Self {
        self.status = StatusReader::new().ok().map(|reader| StatusPages {
            reader,
            pages: pages.into(),
        });
        self
    }

    /// The poller, also sending told alerts to `webhooks`.
    #[must_use]
    pub fn with_webhooks(mut self, webhooks: Vec<Webhook>) -> Self {
        self.deliverer = (!webhooks.is_empty())
            .then(Deliverer::new)
            .and_then(Result::ok);
        self.webhooks = webhooks.into();
        self
    }

    /// What the poller last found, to hand to consumers.
    #[must_use]
    pub fn latest(&self) -> Latest {
        self.latest.clone()
    }

    /// Whether the poller reads the logs for the week's cost.
    #[must_use]
    pub fn scans_costs(&self) -> bool {
        self.costs.is_some()
    }

    /// Polls until `shutdown` completes. A poll in progress then is abandoned: providers save the
    /// floor before they send a request, so it holds for whoever asks next.
    pub async fn run(mut self, shutdown: impl Future<Output = ()>) {
        tokio::pin!(shutdown);
        let latest = self.latest.clone();
        loop {
            let wake = tokio::select! {
                wake = self.poll_due() => wake,
                () = &mut shutdown => return,
            };
            tokio::select! {
                () = tokio::time::sleep_until(wake) => {}
                () = latest.0.nudged.notified() => {}
                () = &mut shutdown => return,
            }
        }
    }

    /// Does whatever is due now: rediscovers accounts, asks every account that is due, and prunes
    /// old history. Returns when something is next due.
    pub async fn poll_due(&mut self) -> Instant {
        let now = Instant::now();
        let wall = (self.wall)();
        if let Some((then, then_wall)) = self.last_round {
            let unseen = (wall - then_wall)
                .to_std()
                .unwrap_or_default()
                .saturating_sub(now.duration_since(then));
            if unseen >= WOKE_AFTER {
                // The machine slept, or the clock jumped ahead: what is on show is old. Everything
                // is asked again now; each provider's floor still holds, since it counts in wall
                // time.
                for tracked in self.tracked.values_mut() {
                    tracked.due = now;
                }
                self.discover_at = now;
                self.alerts_at = now;
                self.status_at = now;
                self.costs_at = now;
            }
        }
        self.last_round = Some((now, wall));
        let tier = self.licensing.tier();
        if self.tier != Some(tier) {
            // A licence added, removed or ended: the accounts shown, the cost scan, the history
            // kept and what the copy leaves out change now, not at their next round (#56).
            self.tier = Some(tier);
            self.discover_at = now;
            self.costs_at = now;
            self.prune_at = now;
        }
        if now >= self.prune_at {
            // QuotaDial Free keeps 30 days whatever the setting says. A failed prune is retried
            // next time; history is not worth stopping for.
            let history = Arc::clone(&self.history);
            let days = if tier.allows(ProFeature::LongHistory) {
                history.keep_days()
            } else {
                history.keep_days().min(KEEP_DAYS)
            };
            let latest = self.latest.clone();
            let _ = tokio::task::spawn_blocking(move || {
                if let Err(error) = history.prune_to(Utc::now(), days) {
                    not_saved(&latest, &history, &error);
                }
            })
            .await;
            self.prune_at = now + PRUNE_EVERY;
        }
        if now >= self.discover_at {
            self.discover(now).await;
            self.discover_at = now + DISCOVER_EVERY;
            // The logs are read only for providers with accounts, so with none there is no cost
            // to leave out.
            self.latest.set_left_out(licence::left_out(
                tier,
                &self.providers,
                self.costs.is_some() && !self.tracked.is_empty(),
            ));
        }
        let mut due: Vec<_> = self
            .tracked
            .iter()
            .filter(|(_, tracked)| tracked.due <= now)
            .map(|(key, _)| key.clone())
            .collect();
        due.sort();
        let polled = !due.is_empty();
        for key in due {
            self.poll(&key, now).await;
        }
        if polled || now >= self.alerts_at {
            self.check_alerts(tier).await;
            self.alerts_at = Instant::now() + ALERTS_EVERY;
        }
        if self.status.is_some() && now >= self.status_at {
            self.read_status_pages(tier);
            self.status_at = now + STATUS_EVERY;
        }
        if self.costs.is_some() && now >= self.costs_at {
            self.scan_costs(tier);
            self.costs_at = now + COSTS_EVERY;
        }
        let status_at = self.status.as_ref().map(|_| self.status_at);
        let costs_at = self.costs.as_ref().map(|_| self.costs_at);
        self.tracked
            .values()
            .map(|tracked| tracked.due)
            .chain([self.discover_at, self.prune_at, self.alerts_at])
            .chain(status_at)
            .chain(costs_at)
            .min()
            .unwrap_or(self.discover_at)
    }

    /// Reads what the logs of the providers the poller has accounts with gained, in its own task
    /// off the async runtime, unless a scan is still reading, and keeps the week's costs. A scan
    /// that fails keeps the last costs, which carry their age. On QuotaDial Free the logs are not
    /// read, no costs are shown, and a scan that began on Pro keeps nothing of what it finds.
    fn scan_costs(&self, tier: Tier) {
        let Some(scan) = self.costs.clone() else {
            return;
        };
        if !tier.allows(ProFeature::CostScan) {
            self.latest.clear_costs();
            return;
        }
        let Ok(scanning) = Arc::clone(&self.scanning).try_lock_owned() else {
            return;
        };
        let providers: Vec<Arc<dyn Provider>> = self
            .providers
            .iter()
            .enumerate()
            .filter(|(index, _)| {
                self.tracked
                    .values()
                    .any(|tracked| tracked.provider == *index)
            })
            .map(|(_, provider)| Arc::clone(provider))
            .collect();
        let latest = self.latest.clone();
        let generation = latest.costs_generation();
        tokio::spawn(async move {
            let found = tokio::task::spawn_blocking(move || {
                let folders: Vec<(String, Vec<PathBuf>)> = providers
                    .iter()
                    .map(|provider| (provider.id().to_owned(), provider.log_folders()))
                    .collect();
                scan.scan(&folders, Utc::now())
            })
            .await;
            if let Ok(Ok(costs)) = found {
                latest.set_costs(generation, costs);
            }
            drop(scanning);
        });
    }

    /// Reads the status pages of the providers the poller has accounts with, in their own task,
    /// unless a round is still reading. Each page read is kept, in the history too, and the alert
    /// rules are checked against it.
    fn read_status_pages(&self, tier: Tier) {
        let Some(StatusPages { reader, pages }) = self.status.clone() else {
            return;
        };
        let Ok(reading) = Arc::clone(&self.reading).try_lock_owned() else {
            return;
        };
        let wanted: Vec<(String, String)> = pages
            .iter()
            .filter(|(provider, _)| {
                self.tracked
                    .values()
                    .any(|tracked| self.providers[tracked.provider].id() == provider.as_str())
            })
            .cloned()
            .collect();
        let (history, latest) = (Arc::clone(&self.history), self.latest.clone());
        tokio::spawn(async move {
            for (provider, url) in wanted {
                let before = latest
                    .report()
                    .status_pages
                    .into_iter()
                    .find(|page| page.provider == provider);
                if let Some(page) = reader.read(&provider, &url, before.as_ref()).await {
                    let stored = (Arc::clone(&history), page.clone(), latest.clone());
                    let _ = tokio::task::spawn_blocking(move || {
                        if let Err(error) = stored.0.record_status_page(&stored.1) {
                            not_saved(&stored.2, &stored.0, &error);
                        }
                    })
                    .await;
                    latest.set_status_page(page);
                }
            }
            check_alerts(&history, &latest, tier).await;
            drop(reading);
        });
    }

    /// Checks the alert rules against what was last found, and tells consumers of any firing.
    async fn check_alerts(&self, tier: Tier) {
        check_alerts(&self.history, &self.latest, tier).await;
        self.deliver(tier);
    }

    /// Sends what is due to the webhooks, in its own task, unless a round is still sending. On
    /// QuotaDial Free alerts go only to the system's notifications.
    fn deliver(&self, tier: Tier) {
        let Some(deliverer) = self.deliverer.clone() else {
            return;
        };
        if !tier.allows(ProFeature::AlertDelivery) {
            return;
        }
        let Ok(sending) = Arc::clone(&self.delivering).try_lock_owned() else {
            return;
        };
        let (history, webhooks) = (Arc::clone(&self.history), Arc::clone(&self.webhooks));
        tokio::spawn(async move {
            deliver_due(&history, &deliverer, &webhooks, Utc::now()).await;
            drop(sending);
        });
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
        let mut unreached = false;
        // What an older QuotaDial left out of a snapshot it kept is taken from the account found
        // (P-020).
        let result = match provider.fetch(&tracked.account).await {
            Ok(snapshot) => Ok(snapshot.completed_from(&tracked.account).without_secrets()),
            Err(error) => {
                unreached = matches!(error, quotadial_core::Error::Request { .. });
                provider
                    .fallback(&tracked.account)
                    .map(|snapshot| snapshot.completed_from(&tracked.account).without_secrets())
                    .ok_or_else(|| ProviderError::new(provider.id(), &error))
            }
        };
        // The gate counts the floor from when the request went out: after the accounts asked
        // before this one in the round, and after finding this one's sign-in. Counting from `now`,
        // when the round began, would bring the next poll a little before the floor allows, and it
        // would only replay the cache.
        let answered = Instant::now();
        let result = match result {
            Ok(snapshot) => {
                if tracked
                    .last
                    .as_ref()
                    .is_none_or(|last| numbers_moved(last, &snapshot))
                {
                    tracked.moved_at = now;
                }
                tracked.last = Some(snapshot.clone());
                let active = now.duration_since(tracked.moved_at) < ACTIVE_FOR;
                tracked.due = answered
                    + if unreached || snapshot.status == Status::Stale {
                        RETRY_UNREACHED
                    } else if active {
                        floor
                    } else {
                        IDLE_EVERY.max(floor)
                    };
                // Written off the async runtime, so a slow disk never holds up the API. A failed
                // write loses one history point and says why in the report; the numbers are still
                // served. Each window's pace is then fitted to what history holds, this reading
                // included.
                let (history, latest) = (Arc::clone(&self.history), self.latest.clone());
                let unpaced = snapshot.clone();
                Ok(tokio::task::spawn_blocking(move || {
                    match history.record(&snapshot, Utc::now()) {
                        Ok(_) => latest.set_problem(Problem::Saving, None),
                        Err(error) => not_saved(&latest, &history, &error),
                    }
                    history.with_pace(snapshot)
                })
                .await
                .unwrap_or(unpaced))
            }
            Err(error) => {
                tracked.due = answered + if unreached { RETRY_UNREACHED } else { floor };
                Err(error)
            }
        };
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

/// Checks the alert rules against what `latest` last found, on a copy with `tier`, and tells its
/// consumers of any firing. A failed check is tried again within a minute; a firing is recorded
/// once whenever it is.
async fn check_alerts(history: &Arc<History>, latest: &Latest, tier: Tier) {
    let (history, report, told) = (Arc::clone(history), latest.report(), latest.clone());
    let fired = tokio::task::spawn_blocking(move || {
        history
            .check(&report, Utc::now(), tier)
            .inspect_err(|error| not_saved(&told, &history, error))
    })
    .await
    .ok()
    .and_then(Result::ok)
    .and_then(|fired| fired.iter().map(|firing| firing.id).max());
    if let Some(newest) = fired {
        latest.0.fired.send_replace(newest);
    }
}

/// Says in the report why `history` could not be saved, until a save works again. A history SQLite
/// finds damaged is noted beside it, so the next daemon to start moves it aside and starts a new
/// one (#52).
fn not_saved(latest: &Latest, history: &History, error: &HistoryError) {
    let line = if error.is_damaged() {
        history.note_damage();
        "QuotaDial's history is damaged. Quit QuotaDial and open it again, or restart \
         `quotadial daemon`, to start a new one with your alert rules; the numbers still update \
         meanwhile."
            .to_owned()
    } else {
        format!(
            "QuotaDial could not save its history ({error}). The numbers still update, and it \
             saves again once it can, as when the disk has room."
        )
    };
    latest.set_problem(Problem::Saving, Some(line));
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
    use quotadial_core::{Error, Pace, Result, Source, Status, Window, WindowKind};

    use super::*;

    /// A provider whose answers the test sets.
    struct Scripted {
        id: &'static str,
        /// What it calls itself: its id written as a name, as its accounts give it.
        name: &'static str,
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
        /// Where its CLI's logs are.
        logs: Mutex<Vec<PathBuf>>,
        /// How many accounts its limit held back.
        held: AtomicUsize,
        /// Whether its answers are its last numbers again, as a provider gives while it cannot
        /// reach its usage endpoint.
        stale: std::sync::atomic::AtomicBool,
        /// Whether its answers are ones an older QuotaDial kept, whose accounts name no provider
        /// and say nothing of the sign-in.
        older: std::sync::atomic::AtomicBool,
    }

    impl Scripted {
        fn new(id: &'static str, floor_minutes: u64) -> Self {
            Self {
                id,
                name: Box::leak(named(id).into_boxed_str()),
                floor: Duration::from_secs(floor_minutes * 60),
                accounts: Mutex::new(Ok(vec![account(id, "user-1")])),
                used_pct: Mutex::new(Ok(10.0)),
                fallback: None,
                fetches: AtomicUsize::new(0),
                fetched: Mutex::default(),
                warnings: Mutex::default(),
                delay: Duration::ZERO,
                logs: Mutex::default(),
                held: AtomicUsize::new(0),
                stale: std::sync::atomic::AtomicBool::new(false),
                older: std::sync::atomic::AtomicBool::new(false),
            }
        }

        /// `snapshot` as an older QuotaDial kept it, when the test says so.
        fn as_kept(&self, mut snapshot: Snapshot) -> Snapshot {
            if self.older.load(Ordering::SeqCst) {
                snapshot.account.provider_name = quotadial_core::UNNAMED_PROVIDER.to_owned();
                snapshot.account.sign_in_use = quotadial_core::UNSAID_SIGN_IN_USE.to_owned();
            }
            snapshot
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

        fn name(&self) -> &'static str {
            self.name
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

        fn held_back(&self) -> usize {
            self.held.load(Ordering::SeqCst)
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
            let mut answer = snapshot(account, used_pct, count, Source::Api);
            if self.stale.load(Ordering::SeqCst) {
                answer.status = Status::Stale;
            }
            Ok(self.as_kept(answer))
        }

        fn fallback(&self, account: &Account) -> Option<Snapshot> {
            self.fallback
                .map(|used_pct| self.as_kept(snapshot(account, used_pct, 0, Source::LocalLog)))
        }

        fn min_poll_interval(&self) -> Duration {
            self.floor
        }

        fn log_folders(&self) -> Vec<PathBuf> {
            self.logs.lock().expect("lock").clone()
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

        fn name(&self) -> &'static str {
            "Claude"
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

    /// An account of `provider`, named as such a provider names itself: `Claude` for `claude`.
    fn account(provider: &str, id: &str) -> Account {
        Account {
            provider: provider.into(),
            provider_name: named(provider),
            account_id: id.into(),
            display_name: "user@example.com".into(),
            plan: None,
            org: None,
            sign_in: None,
            sign_in_use: "QuotaDial reads this test's sign-in.".into(),
        }
    }

    /// `id` written as a name, `Claude` for `claude`, as a test provider calls itself.
    fn named(id: &str) -> String {
        let mut letters = id.chars();
        letters.next().map_or_else(String::new, |first| {
            first.to_uppercase().chain(letters).collect()
        })
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
                pace: None,
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

    /// A poller on QuotaDial Free, which polls exactly as Pro does.
    fn poller(providers: Vec<Arc<Scripted>>, history: Arc<History>) -> Poller {
        poller_with(providers, history, Licensing::default())
    }

    fn poller_with(
        providers: Vec<Arc<Scripted>>,
        history: Arc<History>,
        licensing: Licensing,
    ) -> Poller {
        Poller::new(
            providers
                .into_iter()
                .map(|provider| provider as Arc<dyn Provider>)
                .collect(),
            history,
            licensing,
        )
    }

    /// Waits, in real time, until `done`, failing after `seconds`.
    async fn within(seconds: u64, done: impl Fn() -> bool) {
        let deadline = std::time::Instant::now() + Duration::from_secs(seconds);
        while !done() {
            assert!(
                std::time::Instant::now() < deadline,
                "not within {seconds} s"
            );
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }

    /// Waits up to 10 seconds for the poller's costs to be other than empty.
    async fn costs_of(poller: &Poller) -> Vec<Cost> {
        for _ in 0..200 {
            let costs = poller.latest().report().costs;
            if !costs.is_empty() {
                return costs;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        Vec::new()
    }

    /// A Codex session log written an hour ago in `dir`, with one request.
    fn codex_log(dir: &std::path::Path) -> PathBuf {
        let written = (Utc::now() - TimeDelta::hours(1)).to_rfc3339();
        let rollout = dir.join("codex/sessions/2026/10/23/rollout-2026-10-23T09-00-00-a.jsonl");
        std::fs::create_dir_all(rollout.parent().expect("a folder")).expect("makes it");
        std::fs::write(
            &rollout,
            format!(
                "{{\"timestamp\":\"{written}\",\"type\":\"turn_context\",\"payload\":{{\"model\":\"gpt-6-astra\"}}}}\n\
                 {{\"timestamp\":\"{written}\",\"type\":\"token_usage_record\",\"payload\":{{\"response_id\":\"r1\",\"usage\":{{\"input_tokens\":100000,\"cached_input_tokens\":0,\"output_tokens\":1000}}}}}}\n"
            ),
        )
        .expect("writes");
        dir.join("codex/sessions")
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
    async fn a_snapshot_an_older_quotadial_kept_is_completed_from_the_account_found() {
        let (_dir, history) = history();
        let codex = Arc::new(Scripted::new("codex", 2));
        codex.older.store(true, Ordering::SeqCst);
        let mut poller = poller(vec![Arc::clone(&codex)], history);

        poller.poll_due().await;

        let report = poller.latest().report();
        assert_eq!(report.snapshots.len(), 1);
        let account = &report.snapshots[0].account;
        assert_eq!(account.provider_name, "Codex");
        assert_eq!(account.sign_in_use, "QuotaDial reads this test's sign-in.");
    }

    #[tokio::test(start_paused = true)]
    async fn the_report_lists_each_provider_by_its_id_and_its_own_name() {
        let (_dir, history) = history();
        let claude = Arc::new(Scripted::new("claude", 5));
        let codex = Arc::new(Scripted::new("codex", 2));
        let poller = poller(vec![claude, codex], history);

        let listed = |report: StatusReport| {
            report
                .providers
                .into_iter()
                .map(|provider| (provider.id, provider.name))
                .collect::<Vec<_>>()
        };
        let expected = vec![
            ("claude".to_owned(), "Claude".to_owned()),
            ("codex".to_owned(), "Codex".to_owned()),
        ];
        assert_eq!(
            listed(poller.latest().report()),
            expected,
            "before any poll, with no account found yet"
        );
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
        let mut poller = Poller::new(
            vec![Arc::clone(&claude) as Arc<dyn Provider>],
            history,
            Licensing::default(),
        );

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
        assert_eq!(
            poller.latest().report(),
            StatusReport {
                providers: vec![ProviderName {
                    id: "claude".into(),
                    name: "Claude".into(),
                }],
                ..StatusReport::default()
            },
            "nothing but the provider asked"
        );
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
    async fn a_window_gets_its_pace_once_history_has_enough_to_go_on() {
        let (_dir, history) = history();
        let codex = Arc::new(Scripted::new("codex", 2));
        let mut poller = poller(vec![Arc::clone(&codex)], history);
        let pace = |poller: &Poller| {
            poller.latest().report().snapshots[0].windows[0]
                .pace
                .clone()
        };

        // A point more with each fetch, and the fetch time moves on a minute: 60 points an hour.
        while codex.fetches() < 31 {
            codex.set_used(Ok(10.0 + codex.fetches() as f64));
            let wake = poller.poll_due().await;
            if codex.fetches() < 31 {
                assert_eq!(pace(&poller), None, "under half an hour of readings");
            }
            tokio::time::sleep_until(wake).await;
        }

        // 40% at minute 30, so 60 points to go take an hour, and the window has no reset time.
        assert_eq!(
            pace(&poller),
            Some(Pace {
                pct_per_hour: 60.0,
                runs_out_at: Some(base() + TimeDelta::minutes(90)),
            })
        );
    }

    // Real time, since the scan reads files off the async runtime.
    #[tokio::test]
    async fn the_logs_of_providers_with_accounts_are_read_for_the_week_s_cost() {
        let (dir, history) = history();
        let sessions = codex_log(dir.path());
        let codex = Arc::new(Scripted::new("codex", 2));
        *codex.logs.lock().expect("lock") = vec![sessions.clone()];
        // Signed out: its logs are not read.
        let claude = Arc::new(Scripted::new("claude", 5));
        claude.set_accounts(Ok(Vec::new()));
        *claude.logs.lock().expect("lock") = vec![sessions];
        let scan = CostScan::open(&dir.path().join("costs.db")).expect("opens");
        let pro = crate::test_support::pro(dir.path());
        let mut poller =
            poller_with(vec![Arc::clone(&codex), claude], history, pro).with_costs(scan);

        poller.poll_due().await;
        let costs = costs_of(&poller).await;

        assert_eq!(costs.len(), 1, "{costs:?}");
        assert_eq!(costs[0].provider, "codex");
        assert_eq!(costs[0].requests, 1);
        assert_eq!(costs[0].models[0].model, "gpt-6-astra");
        assert!(costs[0].usd > 0.0);
        assert!(poller.latest().report().left_out.is_empty());
    }

    #[tokio::test]
    async fn on_free_the_logs_wait_and_a_key_added_reads_them_at_once() {
        let (dir, history) = history();
        let codex = Arc::new(Scripted::new("codex", 2));
        *codex.logs.lock().expect("lock") = vec![codex_log(dir.path())];
        let scan = CostScan::open(&dir.path().join("costs.db")).expect("opens");
        let data = crate::DataDir::new(dir.path().to_path_buf());
        let mut poller = poller_with(
            vec![Arc::clone(&codex)],
            history,
            Licensing::new(Some(&data)),
        )
        .with_costs(scan);

        poller.poll_due().await;
        tokio::time::sleep(Duration::from_millis(300)).await;
        let free = poller.latest().report();
        assert!(free.costs.is_empty(), "QuotaDial Free reads no logs");
        assert_eq!(
            free.left_out,
            [LeftOut::feature(ProFeature::CostScan)],
            "and says so where the cost would be"
        );

        // The next round reads them; how soon a running poller has that round is
        // `a_nudge_wakes_a_running_poller_at_once`.
        crate::test_support::pro(dir.path());
        poller.poll_due().await;

        assert_eq!(
            costs_of(&poller).await.len(),
            1,
            "read at once, with no restart"
        );
        assert!(poller.latest().report().left_out.is_empty());
    }

    #[tokio::test]
    #[expect(
        clippy::await_holding_lock,
        reason = "the lock is held on purpose, to keep the scan reading on its blocking thread"
    )]
    async fn a_scan_that_began_on_pro_keeps_nothing_once_the_key_lapses() {
        let (dir, history) = history();
        let codex = Arc::new(Scripted::new("codex", 2));
        *codex.logs.lock().expect("lock") = vec![codex_log(dir.path())];
        let scan = CostScan::open(&dir.path().join("costs.db")).expect("opens");
        let data = crate::DataDir::new(dir.path().to_path_buf());
        crate::test_support::pro(dir.path());
        let mut poller = poller_with(
            vec![Arc::clone(&codex)],
            history,
            Licensing::new(Some(&data)),
        )
        .with_costs(scan);

        // A scan first asks the provider where its logs are, so holding them keeps it reading.
        let held = codex.logs.lock().expect("lock");
        poller.poll_due().await;
        crate::licence::remove(&data).expect("removes the key");
        poller.poll_due().await;
        drop(held);
        // The scan has finished once it lets go of the lock it holds while reading.
        drop(Arc::clone(&poller.scanning).lock_owned().await);

        let report = poller.latest().report();
        assert!(
            report.costs.is_empty(),
            "a scan that began on Pro keeps nothing on Free: {:?}",
            report.costs
        );
        assert_eq!(report.left_out, [LeftOut::feature(ProFeature::CostScan)]);
    }

    #[tokio::test]
    async fn a_lapsed_key_keeps_free_s_30_days_at_once() {
        let dir = tempfile::tempdir().expect("temp dir");
        let history = Arc::new(
            History::open(&dir.path().join("history.db"))
                .expect("opens")
                .keeping(90),
        );
        let codex = Arc::new(Scripted::new("codex", 2));
        // An account the provider no longer has, so only this old reading is its.
        let account = account("codex", "gone");
        let mut old = snapshot(&account, 40.0, 0, Source::Api);
        old.windows[0].fetched_at = Utc::now() - TimeDelta::days(60);
        history
            .record(&old, old.windows[0].fetched_at)
            .expect("records");
        let window = old.windows[0].clone();
        let data = crate::DataDir::new(dir.path().to_path_buf());
        crate::test_support::pro(dir.path());
        let mut poller = poller_with(
            vec![Arc::clone(&codex)],
            Arc::clone(&history),
            Licensing::new(Some(&data)),
        );

        poller.poll_due().await;
        let kept = history.readings(&account, &window, None).expect("reads");
        assert_eq!(kept.len(), 1, "Pro keeps 90 days");

        crate::licence::remove(&data).expect("removes the key");
        poller.poll_due().await;

        let kept = history.readings(&account, &window, None).expect("reads");
        assert!(
            kept.is_empty(),
            "Free keeps 30 days from the moment the key goes, not from the next hourly prune"
        );
    }

    // Real time: within the seconds this waits, nothing but the nudge can wake the poller, whose
    // next round is a minute away.
    #[tokio::test]
    async fn a_nudge_wakes_a_running_poller_at_once() {
        let (dir, history) = history();
        let codex = Arc::new(Scripted::new("codex", 2));
        *codex.logs.lock().expect("lock") = vec![codex_log(dir.path())];
        let scan = CostScan::open(&dir.path().join("costs.db")).expect("opens");
        let data = crate::DataDir::new(dir.path().to_path_buf());
        let poller = poller_with(
            vec![Arc::clone(&codex)],
            history,
            Licensing::new(Some(&data)),
        )
        .with_costs(scan);
        let latest = poller.latest();
        let (stop, stopped) = tokio::sync::oneshot::channel::<()>();
        let running = tokio::spawn(poller.run(async {
            let _ = stopped.await;
        }));
        within(5, || !latest.report().left_out.is_empty()).await;

        crate::test_support::pro(dir.path());
        latest.nudge();

        within(5, || !latest.report().costs.is_empty()).await;
        let _ = stop.send(());
        running.await.expect("stops");
    }

    /// A wall clock the test moves on alone, and the poller reading it.
    fn wall_clock() -> (Arc<Mutex<DateTime<Utc>>>, WallClock) {
        let wall = Arc::new(Mutex::new(base()));
        let read = Arc::clone(&wall);
        (wall, Arc::new(move || *read.lock().expect("lock")))
    }

    #[tokio::test(start_paused = true)]
    async fn after_the_machine_sleeps_everything_is_asked_again_at_once() {
        let (_dir, history) = history();
        let codex = Arc::new(Scripted::new("codex", 2));
        let (wall, clock) = wall_clock();
        let mut poller = poller(vec![Arc::clone(&codex)], history).with_wall(clock);

        poller.poll_due().await;
        assert_eq!(codex.fetches(), 1);
        // A minute by the poller's clock, which stood still while the machine slept for hours.
        tokio::time::advance(Duration::from_secs(60)).await;
        *wall.lock().expect("lock") += TimeDelta::hours(8) + TimeDelta::minutes(1);
        poller.poll_due().await;

        assert_eq!(
            codex.fetches(),
            2,
            "asked again on waking, not at its next turn"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn a_clock_that_goes_back_is_not_taken_for_a_wake() {
        let (_dir, history) = history();
        let codex = Arc::new(Scripted::new("codex", 2));
        let (wall, clock) = wall_clock();
        let mut poller = poller(vec![Arc::clone(&codex)], history).with_wall(clock);

        poller.poll_due().await;
        tokio::time::advance(Duration::from_secs(60)).await;
        *wall.lock().expect("lock") -= TimeDelta::hours(3);
        poller.poll_due().await;

        assert_eq!(codex.fetches(), 1, "not due for another minute");
        tokio::time::advance(Duration::from_secs(61)).await;
        poller.poll_due().await;
        assert_eq!(codex.fetches(), 2, "and asked then, as usual");
    }

    #[tokio::test(start_paused = true)]
    async fn an_account_that_could_not_be_reached_is_asked_again_within_a_minute() {
        let (_dir, history) = history();
        let codex = Arc::new(Scripted::new("codex", 2));
        codex.set_used(Err("network is unreachable".into()));
        let mut poller = poller(vec![Arc::clone(&codex)], history);

        poller.poll_due().await;
        tokio::time::advance(RETRY_UNREACHED + Duration::from_secs(1)).await;
        poller.poll_due().await;
        assert_eq!(
            codex.fetches(),
            2,
            "within a minute, not at the 2-minute floor"
        );

        // The network is back.
        codex.set_used(Ok(20.0));
        tokio::time::advance(RETRY_UNREACHED + Duration::from_secs(1)).await;
        poller.poll_due().await;
        assert_eq!(codex.fetches(), 3);
        let report = poller.latest().report();
        assert_eq!(report.snapshots[0].windows[0].used_pct, 20.0);
        assert!(report.errors.is_empty());

        tokio::time::advance(RETRY_UNREACHED + Duration::from_secs(1)).await;
        poller.poll_due().await;
        assert_eq!(codex.fetches(), 3, "answered: back to its floor");
    }

    #[tokio::test(start_paused = true)]
    async fn stale_numbers_are_asked_for_again_within_a_minute() {
        let (_dir, history) = history();
        let codex = Arc::new(Scripted::new("codex", 2));
        codex.stale.store(true, Ordering::SeqCst);
        let mut poller = poller(vec![Arc::clone(&codex)], history);

        poller.poll_due().await;
        tokio::time::advance(RETRY_UNREACHED + Duration::from_secs(1)).await;
        poller.poll_due().await;

        assert_eq!(codex.fetches(), 2);
    }

    #[tokio::test(start_paused = true)]
    async fn a_history_that_cannot_be_saved_says_so_until_it_saves_again() {
        let (_dir, history) = history();
        let codex = Arc::new(Scripted::new("codex", 2));
        let mut poller = poller(vec![Arc::clone(&codex)], Arc::clone(&history));
        // The first round also prunes; the next saves only its reading.
        poller.poll_due().await;
        assert!(poller.latest().report().problems.is_empty());
        // Reads work and writes fail, as on a full disk.
        history
            .lock()
            .pragma_update(None, "query_only", true)
            .expect("read-only");
        tokio::time::advance(Duration::from_secs(2 * 60 + 1)).await;

        poller.poll_due().await;

        let report = poller.latest().report();
        assert_eq!(report.problems.len(), 1, "{:?}", report.problems);
        assert!(
            report.problems[0].starts_with("QuotaDial could not save its history ("),
            "{}",
            report.problems[0]
        );
        assert!(report.problems[0].ends_with(
            "The numbers still update, and it saves again once it can, as when the disk has room."
        ));
        assert_eq!(report.snapshots.len(), 1, "the numbers are still served");

        history
            .lock()
            .pragma_update(None, "query_only", false)
            .expect("writable");
        tokio::time::advance(Duration::from_secs(2 * 60 + 1)).await;
        poller.poll_due().await;
        assert!(
            poller.latest().report().problems.is_empty(),
            "said no more once a save works"
        );
    }

    #[test]
    fn damage_seen_while_the_daemon_runs_is_noted_for_the_next_start() {
        let (dir, history) = history();
        let latest = Latest::default();
        let damaged = HistoryError::from(rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CORRUPT),
            None,
        ));

        not_saved(&latest, &history, &damaged);

        assert!(History::damage_noted(&dir.path().join("history.db")));
        assert_eq!(
            latest.report().problems,
            [
                "QuotaDial's history is damaged. Quit QuotaDial and open it again, or restart \
              `quotadial daemon`, to start a new one with your alert rules; the numbers still \
              update meanwhile."
            ]
        );
    }

    #[tokio::test(start_paused = true)]
    async fn alert_rules_are_checked_after_each_round_and_consumers_told() {
        let (_dir, history) = history();
        let codex = Arc::new(Scripted::new("codex", 2));
        codex.set_used(Ok(95.0));
        let mut poller = poller(vec![Arc::clone(&codex)], Arc::clone(&history));
        let mut fired = poller.latest().subscribe_firings();

        poller.poll_due().await;

        assert!(
            fired.has_changed().expect("open"),
            "a new history's 90% rule"
        );
        let newest = *fired.borrow_and_update();
        let firings = history.firings(0, DateTime::<Utc>::MIN_UTC).expect("reads");
        assert_eq!(firings.len(), 1);
        assert_eq!(firings[0].id, newest);
        assert_eq!(firings[0].title, "Codex Session passed 90%");

        tokio::time::advance(Duration::from_secs(2 * 60)).await;
        poller.poll_due().await;
        assert!(!fired.has_changed().expect("open"), "told once");
    }

    // Real time, since the webhook is reached over a real socket.
    #[tokio::test]
    async fn a_told_alert_goes_to_the_webhooks_too() {
        let (_dir, history) = history();
        let codex = Arc::new(Scripted::new("codex", 2));
        codex.set_used(Ok(95.0));
        let (address, bodies) = crate::test_support::receiver("200 OK").await;
        let webhook =
            Webhook::new(&address, crate::WebhookKind::Json).expect("a webhook on this machine");
        let pro = crate::test_support::pro(_dir.path());
        let mut poller =
            poller_with(vec![Arc::clone(&codex)], history, pro).with_webhooks(vec![webhook]);

        poller.poll_due().await;

        let sent = async {
            while bodies.lock().expect("lock").is_empty() {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        };
        tokio::time::timeout(Duration::from_secs(5), sent)
            .await
            .expect("the webhook got the new history's 90% alert");
        assert!(bodies.lock().expect("lock")[0].contains("Codex Session passed 90%"));
    }

    #[tokio::test(start_paused = true)]
    async fn on_free_the_accounts_a_provider_held_back_are_named_and_pro_names_none() {
        let (dir, history) = history();
        let claude = Arc::new(Scripted::new("claude", 5));
        claude.held.store(8, Ordering::SeqCst);
        let codex = Arc::new(Scripted::new("codex", 2));
        let data = crate::DataDir::new(dir.path().to_path_buf());
        let mut poller = poller_with(
            vec![Arc::clone(&claude), codex],
            history,
            Licensing::new(Some(&data)),
        );

        poller.poll_due().await;
        assert_eq!(
            poller.latest().report().left_out,
            [LeftOut::accounts("claude", "Claude", 8)]
        );
        assert_eq!(
            LeftOut::accounts("claude", "Claude", 8).text,
            "8 more Claude accounts are on this machine. Showing every account is part of \
             QuotaDial Pro: quotadial.com/pro"
        );
        assert_eq!(
            LeftOut::accounts("claude", "Claude", 1).text,
            "1 more Claude account is on this machine. Showing every account is part of \
             QuotaDial Pro: quotadial.com/pro"
        );

        crate::test_support::pro(dir.path());
        claude.held.store(0, Ordering::SeqCst);
        poller.poll_due().await;
        assert!(poller.latest().report().left_out.is_empty());
    }

    #[tokio::test]
    async fn on_free_a_told_alert_goes_only_to_the_notifications() {
        let (_dir, history) = history();
        let codex = Arc::new(Scripted::new("codex", 2));
        codex.set_used(Ok(95.0));
        let (address, bodies) = crate::test_support::receiver("200 OK").await;
        let webhook =
            Webhook::new(&address, crate::WebhookKind::Json).expect("a webhook on this machine");
        let mut poller =
            poller(vec![Arc::clone(&codex)], Arc::clone(&history)).with_webhooks(vec![webhook]);
        let mut fired = poller.latest().subscribe_firings();

        poller.poll_due().await;
        tokio::time::timeout(Duration::from_secs(5), fired.changed())
            .await
            .expect("the 90% alert is told to the app")
            .expect("told");
        tokio::time::sleep(Duration::from_millis(500)).await;

        assert!(
            bodies.lock().expect("lock").is_empty(),
            "QuotaDial Free sends nothing to webhooks"
        );
    }

    // Real time, since the status page is read over a real socket.
    #[tokio::test]
    async fn the_status_page_of_a_provider_with_accounts_is_read_and_kept() {
        let (_dir, history) = history();
        let page = crate::test_support::status_page(
            r#"{"status": {"indicator": "minor", "description": "Partial System Degradation"}}"#,
        )
        .await;
        // Claude's page answers too, but the poller has no Claude account, so it is not read.
        let claude_page = crate::test_support::status_page(
            r#"{"status": {"indicator": "none", "description": "All Systems Operational"}}"#,
        )
        .await;
        let codex = Arc::new(Scripted::new("codex", 2));
        let mut poller =
            poller(vec![Arc::clone(&codex)], Arc::clone(&history)).with_status_pages(vec![
                ("codex".into(), page.clone()),
                ("claude".into(), claude_page),
            ]);

        poller.poll_due().await;

        let read = async {
            loop {
                let pages = poller.latest().report().status_pages;
                if !pages.is_empty() {
                    return pages;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        };
        let pages = tokio::time::timeout(Duration::from_secs(5), read)
            .await
            .expect("the page is read");
        assert_eq!(
            pages.len(),
            1,
            "no account with Claude, so its page is not read"
        );
        assert_eq!(pages[0].provider, "codex");
        assert_eq!(pages[0].indicator, quotadial_core::Indicator::Minor);
        assert_eq!(pages[0].url, page);
        assert!(pages[0].since.is_some());
        assert_eq!(
            history.status_pages().expect("reads"),
            pages,
            "kept for the next start, before it is shown"
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
