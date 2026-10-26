//! GitHub Copilot for QuotaDial: the month's AI credits, premium requests on an annual plan, and
//! Copilot Free's chat messages and code completions, from GitHub's own usage endpoint, with the
//! sign-in GitHub CLI or the Copilot CLI already keeps (D-127; the manager's Copilot rules on #73).
//!
//! - **Sign-ins**, never written, refreshed, moved or copied; QuotaDial runs no device flow and no
//!   login of its own.
//!   - GitHub CLI's, read as a file (`hosts`): the `github.com` entry of `hosts.yml` in its config
//!     folder (`environment`), when GitHub CLI keeps its token there rather than in the system's
//!     keychain. GitHub CLI's keychain item is never read.
//!   - On macOS, the Copilot CLI's Keychain item, `copilot-cli` (`keychain`): found by its
//!     attributes alone, and its secret read only when a request goes out, through
//!     `/usr/bin/security`, the consent path the Claude provider takes (P-007, D-022). If macOS asks,
//!     the prompt is its own and the account's sentence has said so first, and a no is kept until
//!     the item changes or QuotaDial starts again, so QuotaDial never asks again in the background.
//!   - When both name one GitHub account, GitHub CLI's file is used, which macOS never asks about.
//!     The Copilot CLI's account comes first, as the one the Copilot CLI itself uses, so QuotaDial
//!     Free, which shows one account per provider (D-100), shows it.
//! - **Requests** (`api`): `GET https://api.github.com/copilot_internal/user`, at most once every 5
//!   minutes per account through `quotadial-poll`, the floor saved before each request; a 403 or 429
//!   that says when to come back, by `Retry-After` or `x-ratelimit-reset`, holds requests until
//!   then. GitHub's terms forbid excessively frequent requests; one every five minutes is not that.
//!   No request goes out without a sign-in, and the token goes nowhere but GitHub.
//! - **Windows** (`usage`): the month's AI credits as a `credits` window, premium requests on an
//!   annual plan, Copilot Free's chat messages and code completions; what is unlimited, and an
//!   allowance GitHub sets none of, said in words and never as a share (`advice`).
//!
//! Ported in part from CodexBar (`Sources/CodexBarCore/` in `../codexbar-ref`); each module names
//! the files it follows. CodexBar's own device-flow login, the token store it keeps for it and the
//! budgets it reads with github.com's browser cookies are not ported: QuotaDial asks for no sign-in
//! of its own and reads no browser's cookies.

mod advice;
mod api;
mod environment;
mod hosts;
mod keychain;
mod process;
mod usage;

#[cfg(test)]
mod provider_tests;

use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock, PoisonError};
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, SecondsFormat, TimeDelta, Utc};
use quotadial_core::{Account, AccountLimit, Error, Provider, Result, Secret, Snapshot, Status};
use quotadial_poll::{AccountState, Clock, Decision, Policy, StateStore, SystemClock};

use crate::api::{Api, UsageError, UserResponse};
use crate::environment::Environment;
use crate::keychain::{Holder, Keychain, ReadError};
use crate::usage::Reading;

/// The provider id used in [`Account::provider`].
pub const ID: &str = "copilot";

/// What the provider calls itself (P-020): the product, as Claude and Codex are named without their
/// companies. The limits belong to the GitHub account's Copilot plan, which every Copilot client
/// shares; GitHub CLI and the Copilot CLI are only where QuotaDial finds the sign-in.
pub const NAME: &str = "Copilot";

/// At most one request per account every five minutes: GitHub's terms forbid excessively frequent
/// requests, and a monthly allowance moves slowly.
pub const MIN_POLL_INTERVAL: Duration = Duration::from_secs(5 * 60);

/// Whom QuotaDial asks (P-021), in `docs/PRIVACY.md`'s words.
const COMPANY: &str = "GitHub";

/// The floor between requests, enforced across every QuotaDial process.
const FLOOR: TimeDelta = TimeDelta::minutes(5);

/// The account id of a Copilot CLI sign-in whose Keychain item names no GitHub login.
const UNNAMED: &str = "copilot-cli";

/// Where the Copilot CLI's sign-in is, as people read it.
const KEYCHAIN_PLACE: &str = "the macOS Keychain item \"copilot-cli\"";

/// Where a sign-in is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Source {
    /// GitHub CLI's `hosts.yml`, read as a file.
    GhFile(PathBuf),
    /// The Copilot CLI's macOS Keychain item, by its account attribute.
    Keychain(Option<String>),
}

impl Source {
    /// The floor, and how the poll gate's messages name GitHub and the CLI whose sign-in it is.
    fn policy(&self) -> Policy {
        Policy {
            service: COMPANY,
            cli: match self {
                Self::GhFile(_) => "GitHub CLI",
                Self::Keychain(_) => "The Copilot CLI",
            },
            floor: FLOOR,
        }
    }
}

/// A sign-in, found without reading a Keychain secret.
#[derive(Debug, Clone)]
struct SignIn {
    source: Source,
    /// The GitHub login it is for, as written, when it names one.
    login: Option<String>,
    /// Changes whenever the CLI writes it again: the file's or the item's modification time.
    generation: Option<String>,
    /// Where it is, as people read it, with `~` for the home folder.
    place: String,
}

impl SignIn {
    /// The account this sign-in is for: the GitHub login, whatever its case, as GitHub's are.
    fn account(&self) -> Account {
        let (account_id, display_name) = match &self.login {
            Some(login) => (login.to_lowercase(), login.clone()),
            None => (UNNAMED.to_owned(), String::new()),
        };
        Account {
            provider: ID.to_owned(),
            provider_name: NAME.to_owned(),
            account_id,
            display_name,
            plan: None,
            org: None,
            sign_in: Some(self.place.clone()),
            sign_in_use: advice::sign_in_use(&self.source, &self.place),
        }
    }

    /// Whether `other` names the same GitHub account.
    fn same_login(&self, other: &SignIn) -> bool {
        matches!(
            (&self.login, &other.login),
            (Some(login), Some(other)) if login.eq_ignore_ascii_case(other)
        )
    }
}

/// Every sign-in found now, one per GitHub account, the Copilot CLI's first, without reading a
/// Keychain secret; and what could not be looked at.
fn locate(env: &Environment, keychain: &dyn Keychain) -> (Vec<SignIn>, Vec<String>) {
    let mut found = Vec::new();
    let mut warnings = Vec::new();
    match keychain.find(keychain::SERVICE) {
        Ok(Some(item)) => {
            let login = match keychain::holder(item.account.as_deref()) {
                Holder::Login(login) => Some(Some(login)),
                Holder::Unnamed => Some(None),
                // An account on another host, whose usage QuotaDial never asks for.
                Holder::Elsewhere => None,
            };
            if let Some(login) = login {
                found.push(SignIn {
                    source: Source::Keychain(item.account),
                    login,
                    generation: item.modified,
                    place: KEYCHAIN_PLACE.to_owned(),
                });
            }
        }
        Ok(None) => {}
        Err(reason) => warnings.push(advice::keychain_not_searched(&reason)),
    }
    if let Some(path) = env.hosts_file() {
        let place = quotadial_core::text::tilde(&path, env.home.as_deref());
        match hosts::read(&path) {
            // Only a sign-in whose token is in the file: GitHub CLI's keychain is never read.
            Ok(Some(hosts::GhSignIn {
                user: Some(user),
                token: Some(_),
            })) => {
                let gh = SignIn {
                    source: Source::GhFile(path.clone()),
                    login: Some(user),
                    generation: written(&path),
                    place,
                };
                // The same account in the Copilot CLI's item: the file, which macOS never asks
                // about, takes its place.
                match found.iter_mut().find(|sign_in| sign_in.same_login(&gh)) {
                    Some(same) => *same = gh,
                    None => found.push(gh),
                }
            }
            Ok(_) => {}
            Err(reason) => warnings.push(advice::gh_unreadable(&place, &reason)),
        }
    }
    (found, warnings)
}

/// When the file was last written, which changes whenever GitHub CLI signs in again.
fn written(path: &Path) -> Option<String> {
    let modified = std::fs::metadata(path).ok()?.modified().ok()?;
    Some(DateTime::<Utc>::from(modified).to_rfc3339_opts(SecondsFormat::Nanos, true))
}

/// Reads GitHub Copilot's limits with the sign-in GitHub CLI or the Copilot CLI already keeps.
pub struct CopilotProvider {
    env: Environment,
    keychain: Arc<dyn Keychain>,
    clock: Arc<dyn Clock>,
    state_dir: Option<PathBuf>,
    base_url: String,
    request_timeout: Duration,
    api: OnceLock<std::result::Result<Api, String>>,
    /// At most this many accounts, the Copilot CLI's first, asked at each discovery.
    max_accounts: AccountLimit,
    /// How many more accounts the last discovery found than `max_accounts` let it show.
    held_back: AtomicUsize,
    /// What the last discovery could not look at.
    warnings: Mutex<Vec<String>>,
    /// The Keychain items the person told macOS not to give QuotaDial, by their account attribute
    /// and their modification time then: none is read again until it changes or QuotaDial starts
    /// again.
    refused: Mutex<Vec<(Option<String>, Option<String>)>>,
}

impl CopilotProvider {
    /// Creates the provider for this process's environment. Nothing is read until it is asked.
    #[must_use]
    pub fn new() -> Self {
        Self {
            env: Environment::from_process(),
            keychain: Arc::from(keychain::system()),
            clock: Arc::new(SystemClock),
            state_dir: StateStore::default_dir(ID),
            base_url: api::DEFAULT_BASE_URL.to_owned(),
            request_timeout: api::TIMEOUT,
            api: OnceLock::new(),
            max_accounts: AccountLimit::none(),
            held_back: AtomicUsize::new(0),
            warnings: Mutex::default(),
            refused: Mutex::default(),
        }
    }

    /// Shows at most as many accounts as `limit` says at each discovery, the Copilot CLI's first:
    /// the `max_accounts` setting, and one on QuotaDial Free.
    #[must_use]
    pub fn with_max_accounts(mut self, limit: AccountLimit) -> Self {
        self.max_accounts = limit;
        self
    }

    /// The HTTP client, built on first use.
    fn api(&self) -> Result<&Api> {
        self.api
            .get_or_init(|| Api::new(&self.base_url, self.request_timeout))
            .as_ref()
            .map_err(|reason| Error::Request {
                provider: ID,
                reason: reason.clone(),
            })
    }

    fn store(&self) -> Result<StateStore> {
        self.state_dir
            .clone()
            .map(|dir| StateStore::new(ID, dir))
            .ok_or_else(|| Error::Cache {
                provider: ID,
                location: "the user's cache directory".to_owned(),
                reason: "the operating system did not name one".to_owned(),
            })
    }

    /// Every sign-in found now, off the async runtime: on macOS that runs `/usr/bin/security`.
    async fn locate_now(&self) -> Result<Vec<SignIn>> {
        let (env, keychain) = (self.env.clone(), Arc::clone(&self.keychain));
        let (found, _) = tokio::task::spawn_blocking(move || locate(&env, keychain.as_ref()))
            .await
            .map_err(|_| stopped())?;
        Ok(found)
    }

    /// The sign-in for `account` as it is now, or the sentence to show when it is gone or holds
    /// another account. No secret is read.
    async fn sign_in_for(&self, account: &Account) -> Result<std::result::Result<SignIn, String>> {
        let found = self.locate_now().await?;
        if let Some(sign_in) = found
            .iter()
            .find(|sign_in| sign_in.account().same_as(account))
        {
            return Ok(Ok(sign_in.clone()));
        }
        let place = account.sign_in.as_deref();
        Ok(Err(
            match found
                .iter()
                .find(|sign_in| Some(sign_in.place.as_str()) == place)
            {
                Some(other) => advice::different_account(&other.source, &other.place),
                None => match place {
                    Some(KEYCHAIN_PLACE) => {
                        advice::signed_out(&Source::Keychain(None), KEYCHAIN_PLACE)
                    }
                    Some(place) => advice::signed_out(&Source::GhFile(PathBuf::from(place)), place),
                    None => advice::NOT_FOUND.to_owned(),
                },
            },
        ))
    }

    /// The account's saved state, put right first: a damaged file is saved again at once, so the
    /// hold it records cannot restart, and a clock that moved back is re-anchored. For a sign-in
    /// that names no account, numbers from an earlier token are forgotten, since they may be
    /// another account's.
    fn prepare(
        store: &StateStore,
        key: &str,
        sign_in: &SignIn,
        policy: &Policy,
        now: DateTime<Utc>,
    ) -> Result<AccountState> {
        let (mut state, damaged) = store.load(key, now).map_err(|error| store.error(&error))?;
        let mut changed = damaged;
        changed |= state.reanchor(now);
        if sign_in.login.is_none() {
            changed |=
                state.forget_numbers_after_new_sign_in(policy, sign_in.generation.as_deref());
        }
        if changed {
            store
                .save(key, &state)
                .map_err(|error| store.error(&error))?;
        }
        Ok(state)
    }

    /// Reads `sign_in`'s token, as late as it can be read: the file again, or the Keychain item's
    /// secret, where macOS may ask. A refused item is not asked for again while it is unchanged.
    async fn token(&self, sign_in: &SignIn) -> Result<std::result::Result<Secret, Unusable>> {
        match &sign_in.source {
            Source::GhFile(path) => {
                let read = hosts::read(path).map_err(|reason| Error::Credentials {
                    provider: ID,
                    location: sign_in.place.clone(),
                    reason,
                })?;
                let same = |user: &str| {
                    sign_in
                        .login
                        .as_deref()
                        .is_some_and(|login| login.eq_ignore_ascii_case(user))
                };
                Ok(match read {
                    Some(hosts::GhSignIn {
                        user: Some(user), ..
                    }) if !same(&user) => Err(Unusable::AnotherAccount),
                    Some(hosts::GhSignIn {
                        user: Some(_),
                        token: Some(token),
                    }) => Ok(token),
                    // No file, or no token in it now: GitHub CLI signed out, or keeps it elsewhere.
                    _ => Err(Unusable::SignedOut),
                })
            }
            Source::Keychain(account) => {
                let marked = (account.clone(), sign_in.generation.clone());
                if self
                    .refused
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .contains(&marked)
                {
                    return Ok(Err(Unusable::Refused));
                }
                let (keychain, service_account) = (Arc::clone(&self.keychain), account.clone());
                let read = tokio::task::spawn_blocking(move || {
                    keychain.read(keychain::SERVICE, service_account.as_deref())
                })
                .await
                .map_err(|_| Error::Credentials {
                    provider: ID,
                    location: KEYCHAIN_PLACE.to_owned(),
                    reason: "reading it stopped unexpectedly; try again".to_owned(),
                })?;
                let token = match read {
                    Ok(Some(token)) => token,
                    Ok(None) => return Ok(Err(Unusable::SignedOut)),
                    Err(ReadError::Refused) => {
                        self.refused
                            .lock()
                            .unwrap_or_else(PoisonError::into_inner)
                            .push(marked);
                        return Ok(Err(Unusable::Refused));
                    }
                    Err(ReadError::Failed(reason)) => {
                        return Err(Error::Credentials {
                            provider: ID,
                            location: KEYCHAIN_PLACE.to_owned(),
                            reason,
                        });
                    }
                };
                // The Copilot CLI may have written the item again while it was read, perhaps for
                // another account. A token read during a change is not used.
                let now = self.locate_now().await?;
                let unchanged = now.iter().any(|again| {
                    again.source == sign_in.source
                        && again.login == sign_in.login
                        && again.generation == sign_in.generation
                });
                Ok(if unchanged {
                    Ok(token)
                } else {
                    Err(Unusable::Changed)
                })
            }
        }
    }

    /// Sends the usage request and records the answer. `state` already says a request was sent.
    async fn request(
        api: &Api,
        token: &Secret,
        sign_in: &SignIn,
        mut account: Account,
        state: &mut AccountState,
        policy: &Policy,
        now: DateTime<Utc>,
    ) -> std::result::Result<Snapshot, RequestFailure> {
        let request_error = |reason| Error::Request {
            provider: ID,
            reason,
        };
        let response = match api.user(token).await {
            Ok(response) => response,
            Err(UsageError::NotSent(reason)) => {
                return Err(RequestFailure::NotSent(format!(
                    "Could not reach GitHub: {reason}. Check the internet connection."
                )));
            }
            // GitHub may have counted it, so it is recorded like any answer QuotaDial cannot use.
            Err(UsageError::NoAnswer(reason)) => {
                return state
                    .record_failure(policy, format!("GitHub did not answer: {reason}."), now)
                    .map_err(|reason| RequestFailure::Error(request_error(reason)));
            }
        };

        let snapshot = match response {
            UserResponse::Usage(body) => {
                let reading = usage::read(&body, now);
                let foreign = matches!(
                    (&reading.login, &sign_in.login),
                    (Some(answered), Some(login)) if !answered.eq_ignore_ascii_case(login)
                );
                if foreign {
                    return state
                        .record_failure(policy, advice::ANOTHER_ACCOUNT.to_owned(), now)
                        .map_err(|reason| {
                            RequestFailure::Error(Error::InvalidResponse {
                                provider: ID,
                                reason,
                            })
                        });
                }
                if reading.plan.is_some() {
                    account.plan.clone_from(&reading.plan);
                }
                state.record_usage(snapshot_from(account, reading))
            }
            UserResponse::Unauthorized => {
                let snapshot = needs_sign_in(state, account, &advice::rejected(&sign_in.source));
                state.record_auth_failure(&snapshot);
                snapshot
            }
            UserResponse::RateLimited { retry_after, reset } => {
                let until = api::limited_until(retry_after.as_deref(), reset.as_deref(), now);
                state.record_rate_limit(policy, &account, until, now)
            }
            UserResponse::Failed { status, message } => state
                .record_failure(policy, advice::failed(status, message.as_deref()), now)
                .map_err(|reason| RequestFailure::Error(request_error(reason)))?,
            UserResponse::Invalid(reason) => state
                .record_failure(
                    policy,
                    format!("GitHub's answer could not be read: {reason}."),
                    now,
                )
                .map_err(|reason| {
                    RequestFailure::Error(Error::InvalidResponse {
                        provider: ID,
                        reason,
                    })
                })?,
        };
        Ok(snapshot)
    }
}

impl Default for CopilotProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for CopilotProvider {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // No folder: its path has the user's name in it (#52).
        formatter
            .debug_struct("CopilotProvider")
            .field("base_url", &self.base_url)
            .finish_non_exhaustive()
    }
}

/// Why a sign-in found gave no token to send.
enum Unusable {
    /// It is gone.
    SignedOut,
    /// It is now another GitHub account's.
    AnotherAccount,
    /// The person told macOS not to give it to QuotaDial.
    Refused,
    /// It changed while it was read.
    Changed,
}

/// Why a request produced no snapshot.
enum RequestFailure {
    /// The request never left this machine, so it does not count against the floor.
    NotSent(String),
    Error(Error),
}

impl RequestFailure {
    fn into_error(self) -> Error {
        match self {
            Self::NotSent(reason) => Error::Request {
                provider: ID,
                reason,
            },
            Self::Error(error) => error,
        }
    }
}

#[async_trait]
impl Provider for CopilotProvider {
    fn id(&self) -> &'static str {
        ID
    }

    fn name(&self) -> &'static str {
        NAME
    }

    fn asks(&self) -> Option<&'static str> {
        Some(COMPANY)
    }

    /// The accounts GitHub CLI's file and the Copilot CLI's Keychain item name, found without
    /// reading a Keychain secret, so macOS never asks here.
    fn discover_accounts(&self) -> Result<Vec<Account>> {
        let (mut found, warnings) = locate(&self.env, self.keychain.as_ref());
        let held_back = match self.max_accounts.get() {
            Some(max) if found.len() > max => {
                let held_back = found.len() - max;
                found.truncate(max);
                held_back
            }
            _ => 0,
        };
        self.held_back.store(held_back, Ordering::Relaxed);
        *self.warnings.lock().unwrap_or_else(PoisonError::into_inner) = warnings;
        Ok(found.iter().map(SignIn::account).collect())
    }

    fn held_back(&self) -> usize {
        self.held_back.load(Ordering::Relaxed)
    }

    fn discovery_warnings(&self) -> Vec<String> {
        self.warnings
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    async fn fetch(&self, account: &Account) -> Result<Snapshot> {
        if account.provider != ID {
            return Err(Error::InvalidResponse {
                provider: ID,
                reason: format!(
                    "an account of {} is not one of Copilot's",
                    quotadial_core::redact_secrets(&account.provider)
                ),
            });
        }
        let store = self.store()?;
        let key = quotadial_poll::account_key(account);
        // Everything below runs under the account's lock, so the sign-in found, the token read and
        // the answer saved belong to one moment, and two processes send one request.
        let _lock = store.lock_account(&key).await?;

        let now = self.clock.now();
        let sign_in = match self.sign_in_for(account).await? {
            Ok(sign_in) => sign_in,
            Err(reason) => {
                let (state, _) = store.load(&key, now).map_err(|error| store.error(&error))?;
                return Ok(needs_sign_in(&state, account.clone(), &reason));
            }
        };
        // Named by where its sign-in was found this time, which may have moved (P-015).
        let account = Account {
            sign_in: Some(sign_in.place.clone()),
            sign_in_use: advice::sign_in_use(&sign_in.source, &sign_in.place),
            ..account.clone()
        };
        let policy = sign_in.source.policy();
        let mut state = Self::prepare(&store, &key, &sign_in, &policy, now)?;
        match quotadial_poll::decide(
            &policy,
            &state,
            &account,
            sign_in.generation.as_deref(),
            now,
        ) {
            Decision::Request => {}
            Decision::Replay(snapshot) => return Ok(*snapshot),
            Decision::Wait(reason) => {
                return Err(Error::Request {
                    provider: ID,
                    reason,
                });
            }
        }

        // Only now is the token read, so a read macOS may ask about comes at most once a floor.
        let token = match self.token(&sign_in).await? {
            Ok(token) => token,
            Err(Unusable::SignedOut) => {
                let reason = advice::signed_out(&sign_in.source, &sign_in.place);
                return Ok(needs_sign_in(&state, account, &reason));
            }
            Err(Unusable::AnotherAccount) => {
                let reason = advice::different_account(&sign_in.source, &sign_in.place);
                return Ok(needs_sign_in(&state, account, &reason));
            }
            // Not a sign-in to fix but a no to keep, so no sign-in alert fires for it.
            Err(Unusable::Refused) => {
                return Ok(Snapshot::unsupported(
                    account,
                    advice::refused(&sign_in.place),
                ));
            }
            Err(Unusable::Changed) => return Ok(changed_while_read(&state, account, &sign_in)),
        };

        let api = self.api()?;
        // Save the floor before sending, so it holds even if QuotaDial stops mid-request. If the
        // cache cannot be written, no request is sent.
        let before = state.clone();
        state.start_request(&policy, now, sign_in.generation.clone());
        store
            .save(&key, &state)
            .map_err(|error| store.error(&error))?;

        match Self::request(api, &token, &sign_in, account, &mut state, &policy, now).await {
            Err(RequestFailure::NotSent(message)) => {
                // Nothing left this machine, so the floor is put back as it was. If this save
                // fails, the floor simply holds for 5 minutes.
                let _ = store.save(&key, &before);
                quotadial_poll::unanswered(&before, &message).map_err(|reason| Error::Request {
                    provider: ID,
                    reason,
                })
            }
            answered => {
                store.save_answer(&key, &state)?;
                answered.map_err(RequestFailure::into_error)
            }
        }
    }

    fn fallback(&self, _account: &Account) -> Option<Snapshot> {
        // Neither CLI keeps a record of the limits that could stand in; stale numbers come from
        // QuotaDial's own cache inside `fetch` instead.
        None
    }

    fn min_poll_interval(&self) -> Duration {
        MIN_POLL_INTERVAL
    }
}

/// The snapshot for a usage answer. With no window, the account is `unsupported` with one sentence
/// saying why, never a misleading 0%; beside windows, what has no share is said after its sign-in
/// sentence, which every surface shows with the account.
fn snapshot_from(mut account: Account, reading: Reading) -> Snapshot {
    let unshared = advice::unshared(&reading.no_allowance, &reading.unlimited);
    if reading.windows.is_empty() {
        return Snapshot::unsupported(
            account,
            unshared.unwrap_or_else(|| advice::NO_LIMITS.to_owned()),
        );
    }
    if let Some(unshared) = unshared {
        account.sign_in_use = format!("{} {unshared}", account.sign_in_use);
    }
    Snapshot {
        account,
        windows: reading.windows,
        extra_usage: None,
        status: Status::Ok,
        error: None,
    }
}

/// A sign-in that needs the user, keeping the account's last good numbers on show, marked as coming
/// from the cache with their age, so its row does not empty while it waits.
fn needs_sign_in(state: &AccountState, account: Account, reason: &str) -> Snapshot {
    let mut snapshot = Snapshot::auth_required(account, reason);
    if let Some(last) = &state.last_good {
        snapshot.windows = last.clone().served_from_cache().windows;
    }
    snapshot
}

/// Shown when the Keychain item changed while its token was read: its last numbers from the cache,
/// if any, until the next fetch reads it again.
fn changed_while_read(state: &AccountState, account: Account, sign_in: &SignIn) -> Snapshot {
    let mut snapshot = Snapshot {
        account,
        windows: Vec::new(),
        extra_usage: None,
        status: Status::Stale,
        error: Some(advice::changed_while_read(&sign_in.place)),
    };
    if let Some(last) = &state.last_good {
        snapshot.windows = last.clone().served_from_cache().windows;
    }
    snapshot
}

/// The error for a lookup whose task stopped before it finished: the sign-in could not be read
/// "from this machine", as Claude's says (D-128).
fn stopped() -> Error {
    Error::Credentials {
        provider: ID,
        location: "this machine".to_owned(),
        reason: "looking it up stopped unexpectedly; try again".to_owned(),
    }
}

#[cfg(test)]
impl CopilotProvider {
    /// A provider with every outside dependency replaced: no real environment, keychain, clock,
    /// cache directory or GitHub.
    pub(crate) fn for_tests(
        env: Environment,
        keychain: Arc<dyn Keychain>,
        clock: Arc<dyn Clock>,
        state_dir: PathBuf,
        base_url: &str,
    ) -> Self {
        Self {
            env,
            keychain,
            clock,
            state_dir: Some(state_dir),
            base_url: base_url.to_owned(),
            request_timeout: api::TIMEOUT,
            api: OnceLock::new(),
            max_accounts: AccountLimit::none(),
            held_back: AtomicUsize::new(0),
            warnings: Mutex::default(),
            refused: Mutex::default(),
        }
    }

    /// The same, with requests giving up after `timeout`.
    pub(crate) fn with_request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn its_name_and_every_sentence_it_says_keep_to_plain_words() {
        let place = "~/.config/gh/hosts.yml";
        let mut said = vec![NAME.to_owned()];
        for source in [
            Source::GhFile(PathBuf::from("/Users/example/.config/gh/hosts.yml")),
            Source::Keychain(None),
        ] {
            said.push(advice::sign_in_use(&source, place));
            said.push(advice::sign_in_use(&source, KEYCHAIN_PLACE));
        }
        said.extend(advice::tests::every_sentence());
        for words in said {
            // #52's plain words: never "quota", "auth" or "credential" outside a name (P-020).
            assert_eq!(
                quotadial_core::text::unplain_words(&words),
                Vec::<String>::new(),
                "{words}"
            );
        }
        assert!(NAME.len() <= 20 && !NAME.ends_with('.'));
    }

    #[test]
    fn it_asks_github_at_most_every_five_minutes_and_keeps_no_logs() {
        let provider = CopilotProvider::new();

        assert_eq!((provider.id(), provider.name()), ("copilot", "Copilot"));
        assert_eq!(provider.asks(), Some("GitHub"));
        assert_eq!(provider.min_poll_interval(), Duration::from_secs(300));
        assert_eq!(Source::Keychain(None).policy().floor, TimeDelta::minutes(5));
        assert!(provider.log_folders().is_empty());
        assert!(provider.commands().is_empty());
        assert_eq!(
            format!("{provider:?}"),
            "CopilotProvider { base_url: \"https://api.github.com\", .. }"
        );
    }

    /// Each error the provider makes, as D-128 has them: the provider left out, since the report
    /// names it, the sign-in never called anything else, a place written as `text::tilde` writes
    /// it, and #52's plain words.
    #[test]
    fn every_error_it_makes_leaves_the_provider_out_in_plain_words() {
        let place = "~/.config/gh/hosts.yml";
        let errors = [
            stopped(),
            Error::Credentials {
                provider: ID,
                location: KEYCHAIN_PLACE.to_owned(),
                reason: "reading it stopped unexpectedly; try again".to_owned(),
            },
            Error::Credentials {
                provider: ID,
                location: place.to_owned(),
                reason: "it is not a plain file".to_owned(),
            },
            Error::Request {
                provider: ID,
                reason: advice::failed(404, Some("Not Found")),
            },
            Error::InvalidResponse {
                provider: ID,
                reason: advice::ANOTHER_ACCOUNT.to_owned(),
            },
        ];
        for error in errors {
            let text = error.to_string();
            assert!(!text.starts_with(ID), "{text}");
            assert_eq!(
                quotadial_core::text::unplain_words(&text),
                Vec::<String>::new(),
                "{text}"
            );
        }
        assert_eq!(
            stopped().to_string(),
            "could not read the sign-in from this machine: looking it up stopped unexpectedly; try \
             again"
        );
    }

    #[test]
    fn the_poll_gate_names_github_and_the_cli_whose_sign_in_it_is() {
        let gh = Source::GhFile(PathBuf::new()).policy();
        let cli = Source::Keychain(None).policy();

        assert_eq!((gh.service, gh.cli), ("GitHub", "GitHub CLI"));
        assert_eq!((cli.service, cli.cli), ("GitHub", "The Copilot CLI"));
    }
}
