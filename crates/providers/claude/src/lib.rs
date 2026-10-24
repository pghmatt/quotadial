//! Claude usage provider for QuotaDial.
//!
//! Reads the sign-in Claude Code already stores, asks Anthropic's OAuth usage endpoint for the
//! account's limits, and maps the answer to the normalized model. It never writes, refreshes or
//! copies Claude Code's credentials, sends the token nowhere but Anthropic, and asks Anthropic at
//! most once every 5 minutes per account, across every QuotaDial process.
//!
//! Ported from CodexBar (`Sources/CodexBarCore/` in `../codexbar-ref`). Each module names the
//! files it follows.

mod advice;
mod api;
mod credentials;
mod environment;
mod identity;
mod keychain;
mod login;
mod plan;
mod process;
mod usage;

#[cfg(test)]
mod provider_tests;

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, PoisonError};
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, TimeDelta, Utc};
use quotadial_core::{Account, Error, Provider, Result, Secret, Snapshot, Status};
use quotadial_poll::{AccountState, Clock, Decision, Policy, StateStore, SystemClock};
use serde::{Deserialize, Serialize};

use crate::api::{Api, UsageError, UsageResponse};
use crate::credentials::{Credentials, ParseError};
use crate::environment::Environment;
use crate::identity::Identity;
use crate::keychain::Keychain;
use crate::login::{Folder, Login, LoginSource};
use crate::usage::Reading;

/// The provider id used in [`Account::provider`].
pub const ID: &str = "claude";

/// Anthropic rate-limits its usage endpoint aggressively, so QuotaDial polls it at most once every
/// five minutes per account.
pub const MIN_POLL_INTERVAL: Duration = Duration::from_secs(5 * 60);

/// How long recording an item as not an account waits for another QuotaDial process doing the same.
const NOT_ACCOUNTS_WAIT: Duration = Duration::from_secs(5);

/// The floor between requests, enforced across every QuotaDial process, and how messages name
/// Anthropic and Claude Code.
const POLICY: Policy = Policy {
    service: "Anthropic",
    cli: "Claude Code",
    floor: TimeDelta::minutes(5),
};

/// Reads Claude usage with the sign-ins Claude Code already stores: every one on this machine, not
/// only the one this shell would use (#34).
pub struct ClaudeProvider {
    env: Environment,
    keychain: Arc<dyn Keychain>,
    clock: Arc<dyn Clock>,
    state_dir: Option<PathBuf>,
    base_url: String,
    request_timeout: Duration,
    api: OnceLock<std::result::Result<Api, String>>,
    /// More config folders to look in, from QuotaDial's settings.
    extra_dirs: Vec<PathBuf>,
    /// At most this many accounts, this shell's own first; all when `None`.
    max_accounts: Option<usize>,
    /// The sign-ins the last discovery found, so a fetch looks up only its own account's.
    logins: Mutex<Vec<Login>>,
    /// What the last discovery could not look at.
    warnings: Mutex<Vec<String>>,
}

impl ClaudeProvider {
    /// Creates the provider for this process's environment.
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
            extra_dirs: Vec::new(),
            max_accounts: None,
            logins: Mutex::default(),
            warnings: Mutex::default(),
        }
    }

    /// Also looks for sign-ins in `dirs`, the `extra_config_dirs` setting. A folder ending in `*`
    /// adds every folder inside it.
    #[must_use]
    pub fn with_extra_config_dirs(mut self, dirs: Vec<PathBuf>) -> Self {
        self.extra_dirs = dirs;
        self
    }

    /// Shows at most `max` accounts, this shell's own first (the `max_accounts` setting).
    #[must_use]
    pub fn with_max_accounts(mut self, max: Option<usize>) -> Self {
        self.max_accounts = max;
        self
    }

    fn locate_all(&self) -> Result<Vec<Login>> {
        let not_accounts = self
            .state_dir
            .as_deref()
            .map(NotAccounts::load)
            .unwrap_or_default();
        let found = login::locate_all(&self.env, self.keychain.as_ref(), &self.extra_dirs)?;
        let mut logins = found.logins;
        logins.retain(|login| !not_accounts.excludes(login));
        if let Some(max) = self.max_accounts {
            self.keep_subscriptions(&mut logins, max);
        }
        self.logins
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone_from(&logins);
        *self.warnings.lock().unwrap_or_else(PoisonError::into_inner) = found
            .unlisted
            .iter()
            .map(|reason| advice::unlisted(reason))
            .chain(found.skipped.iter().map(|dir| advice::skipped_dir(dir)))
            .collect();
        Ok(logins)
    }

    /// Keeps the sign-ins of the first `max` subscriptions, this shell's own first (the
    /// `max_accounts` setting). A sign-in that does not say whose it is counts as the account an
    /// earlier answer named for its token, so two sign-ins of one subscription count once (#39).
    /// One not named yet counts as its own until it is.
    fn keep_subscriptions(&self, logins: &mut Vec<Login>, max: usize) {
        let store = self.store().ok();
        let now = self.clock.now();
        let mut counted: Vec<Account> = Vec::new();
        logins.retain(|login| {
            let account = login.account();
            let subscription = login
                .identity
                .is_none()
                .then_some(store.as_ref())
                .flatten()
                .and_then(|store| store.load(&quotadial_poll::account_key(&account), now).ok())
                .and_then(|(state, _)| known_account(login, &state, &account))
                .unwrap_or(account);
            if counted.iter().any(|known| known.same_as(&subscription)) {
                // Another sign-in of a subscription already kept: it adds no row.
                return true;
            }
            let kept = counted.len() < max;
            if kept {
                counted.push(subscription);
            }
            kept
        });
    }

    /// Remembers that `login`, a Keychain item whose folder is unknown, holds no Claude account,
    /// so discovery leaves it out until Claude Code rewrites it. Other QuotaDial processes may be
    /// recording items too, so the record is read and rewritten under a lock. A record that cannot
    /// be kept only means the item is asked about again.
    async fn not_an_account(&self, login: &Login) {
        let (Some(dir), LoginSource::Keychain { service, .. }) =
            (self.state_dir.clone(), &login.source)
        else {
            return;
        };
        if login.folder != Folder::Unknown || login.generation.is_none() {
            return;
        }
        let (service, generation) = (service.clone(), login.generation.clone());
        let _ = tokio::task::spawn_blocking(move || -> std::io::Result<()> {
            let _lock =
                StateStore::new(ID, dir.clone()).lock(NotAccounts::LOCK, NOT_ACCOUNTS_WAIT)?;
            let mut not_accounts = NotAccounts::load(&dir);
            not_accounts.0.insert(service, generation);
            not_accounts.save(&dir)
        })
        .await;
    }

    /// The sign-in for `account` as it is now, or the sentence to show when it is gone. Only that
    /// sign-in is looked up again, the one discovery found for the account, so a fetch costs one
    /// attribute lookup rather than a search of every folder.
    async fn login_for(&self, account: &Account) -> Result<std::result::Result<Login, String>> {
        let found = self
            .logins
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .iter()
            .find(|login| login.account().same_as(account))
            .cloned();
        let Some(found) = found else {
            // Not among the sign-ins found last, as when this process has not looked yet.
            let (env, keychain, extra) = (
                self.env.clone(),
                Arc::clone(&self.keychain),
                self.extra_dirs.clone(),
            );
            let found = tokio::task::spawn_blocking(move || {
                login::locate_all(&env, keychain.as_ref(), &extra)
            })
            .await
            .map_err(|_| stopped())??;
            return Ok(found
                .logins
                .into_iter()
                .find(|login| login.account().same_as(account))
                .ok_or_else(|| advice::not_found(account)));
        };
        Ok(match self.relocate(&found).await? {
            None => Err(advice::signed_out(&found)),
            Some(now) if !now.account().same_as(account) => Err(advice::different_account(&found)),
            Some(now) => Ok(now),
        })
    }

    /// `login` looked up again on its own, off the async runtime: on macOS that runs
    /// `/usr/bin/security`.
    async fn relocate(&self, login: &Login) -> Result<Option<Login>> {
        let (login, keychain) = (login.clone(), Arc::clone(&self.keychain));
        tokio::task::spawn_blocking(move || login.relocate(keychain.as_ref()))
            .await
            .map_err(|_| stopped())?
    }

    /// The account's saved state, put right first: a damaged file is saved again at once, so the
    /// hold it records cannot restart, and a clock that moved back is re-anchored. For `unnamed`, a
    /// sign-in that does not say whose it is, numbers from an earlier token are forgotten, since
    /// they may be another account's.
    fn prepare(
        store: &StateStore,
        key: &str,
        unnamed: Option<&Login>,
        now: DateTime<Utc>,
    ) -> Result<AccountState> {
        let (mut state, damaged) = store.load(key, now).map_err(|error| store.error(&error))?;
        let mut changed = damaged;
        changed |= state.reanchor(now);
        if let Some(login) = unnamed.filter(|login| {
            login.identity.is_none() && !matches!(login.source, LoginSource::Environment(_))
        }) {
            changed |= state.forget_numbers_after_new_sign_in(&POLICY, login.generation.as_deref());
        }
        if changed {
            store
                .save(key, &state)
                .map_err(|error| store.error(&error))?;
        }
        Ok(state)
    }

    /// Fetches for `named`, the account an earlier answer named for `login`'s token, under that
    /// account's own lock and floor. `key` is the sign-in's own, already locked.
    async fn fetch_as(
        &self,
        store: &StateStore,
        key: &str,
        login: &Login,
        named: Account,
        now: DateTime<Utc>,
    ) -> Result<Snapshot> {
        let named_key = quotadial_poll::account_key(&named);
        // Locked after the sign-in's own key, never before it, so two processes cannot wait on
        // each other.
        let _lock = if named_key == key {
            None
        } else {
            Some(store.lock_account(&named_key).await?)
        };
        let mut state = Self::prepare(store, &named_key, None, now)?;
        match quotadial_poll::decide(&POLICY, &state, &named, login.generation.as_deref(), now) {
            Decision::Request => {}
            Decision::Replay(snapshot) => return Ok(*snapshot),
            Decision::Wait(reason) => {
                return Err(Error::Request {
                    provider: ID,
                    reason,
                });
            }
        }
        let (credentials, named) = match self.credentials(login, &state, named, now).await? {
            Ok(usable) => usable,
            Err(snapshot) => return Ok(snapshot),
        };
        self.send(store, &named_key, &mut state, login, &credentials, named)
            .await
    }

    /// Fetches for `named`, the account the profile has just named for `login`'s token, under that
    /// account's own lock and floor. `key` is the sign-in's own, already locked.
    async fn request_as(
        &self,
        store: &StateStore,
        key: &str,
        login: &Login,
        credentials: &Credentials,
        named: Account,
        now: DateTime<Utc>,
    ) -> Result<Snapshot> {
        let named_key = quotadial_poll::account_key(&named);
        if named_key == key {
            let mut state = Self::prepare(store, key, None, now)?;
            return self
                .send(store, key, &mut state, login, credentials, named)
                .await;
        }
        // Locked after the sign-in's own key, as in `fetch_as`.
        let _lock = store.lock_account(&named_key).await?;
        let mut named_state = Self::prepare(store, &named_key, None, now)?;
        match quotadial_poll::decide(
            &POLICY,
            &named_state,
            &named,
            login.generation.as_deref(),
            now,
        ) {
            Decision::Request => {
                self.send(
                    store,
                    &named_key,
                    &mut named_state,
                    login,
                    credentials,
                    named,
                )
                .await
            }
            Decision::Replay(snapshot) => Ok(*snapshot),
            Decision::Wait(reason) => Err(Error::Request {
                provider: ID,
                reason,
            }),
        }
    }

    /// Reads `login`'s token and checks, without asking Anthropic, that it can be used: the
    /// credentials and `account` with the plan they name, or the snapshot to show instead.
    async fn credentials(
        &self,
        login: &Login,
        state: &AccountState,
        mut account: Account,
        now: DateTime<Utc>,
    ) -> Result<std::result::Result<(Credentials, Account), Snapshot>> {
        let credentials = match self.read_credentials(login).await? {
            Stored::Usable(credentials) => credentials,
            Stored::SignedOut => {
                return Ok(Err(needs_sign_in(
                    state,
                    account,
                    &advice::signed_out(login),
                )));
            }
            Stored::NoClaudeLogin => {
                self.not_an_account(login).await;
                return Ok(Err(needs_sign_in(
                    state,
                    account,
                    &advice::no_claude_login(login),
                )));
            }
        };
        // Claude Code may have rewritten this sign-in while its token was read, perhaps for
        // another account. A token read during a change is not used.
        match self.relocate(login).await? {
            None => {
                return Ok(Err(needs_sign_in(
                    state,
                    account,
                    &advice::signed_out(login),
                )));
            }
            Some(again) if !again.account().same_as(&login.account()) => {
                return Ok(Err(needs_sign_in(
                    state,
                    account,
                    &advice::different_account(login),
                )));
            }
            Some(again) if again.generation != login.generation => {
                return Ok(Err(changed_while_read(state, account, login)));
            }
            Some(_) => {}
        }
        if let Some(plan) = credentials.plan() {
            account.plan = Some(plan);
        }
        // Checked locally, so a dead token costs no request. Refreshing it is Claude Code's job.
        if let Some(expired_at) = credentials
            .expires_at
            .filter(|_| credentials.is_expired(now))
        {
            return Ok(Err(needs_sign_in(
                state,
                account,
                &advice::expired(login, expired_at),
            )));
        }
        if !credentials.can_read_usage() {
            return Ok(Err(needs_sign_in(
                state,
                account,
                &advice::missing_scope(login),
            )));
        }
        Ok(Ok((credentials, account)))
    }

    /// Sends the usage request for `account` under `key`'s floor and saves the answer.
    async fn send(
        &self,
        store: &StateStore,
        key: &str,
        state: &mut AccountState,
        login: &Login,
        credentials: &Credentials,
        account: Account,
    ) -> Result<Snapshot> {
        let api = self.api()?;
        let now = self.clock.now();
        // Save the floor before sending, so it holds even if QuotaDial stops mid-request. If the
        // cache cannot be written, no request is sent.
        let before = state.clone();
        state.start_request(&POLICY, now, login.generation.clone());
        store
            .save(key, state)
            .map_err(|error| store.error(&error))?;

        match Self::request(api, login, credentials, account, state, now).await {
            Err(RequestFailure::NotSent(message)) => {
                // Nothing left this machine, so the floor is put back as it was. If this save
                // fails, the floor simply holds for 5 minutes.
                let _ = store.save(key, &before);
                quotadial_poll::unanswered(&before, &message).map_err(|reason| Error::Request {
                    provider: ID,
                    reason,
                })
            }
            answered => {
                store.save_answer(key, state)?;
                answered.map_err(RequestFailure::into_error)
            }
        }
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

    /// Reads the token for `login`: from the environment, the Keychain (where macOS may ask for
    /// permission) or the credentials file.
    async fn read_credentials(&self, login: &Login) -> Result<Stored> {
        let payload = match &login.source {
            LoginSource::Environment(token) => {
                return Ok(Stored::Usable(Credentials::from_env_token(token.clone())));
            }
            LoginSource::Keychain { service, account } => {
                let keychain = Arc::clone(&self.keychain);
                let (service, account) = (service.clone(), account.clone());
                let read = tokio::task::spawn_blocking(move || {
                    keychain.read(&service, account.as_deref())
                })
                .await
                .map_err(|_| {
                    credentials_error(
                        &login.source,
                        "reading it stopped unexpectedly; try again".to_owned(),
                    )
                })?;
                match read {
                    Ok(Some(secret)) => secret,
                    Ok(None) => return Ok(Stored::SignedOut),
                    Err(reason) => return Err(credentials_error(&login.source, reason)),
                }
            }
            LoginSource::File(path) => match std::fs::read_to_string(path) {
                Ok(text) => Secret::new(text),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    return Ok(Stored::SignedOut);
                }
                Err(error) => return Err(credentials_error(&login.source, unreadable(&error))),
            },
        };
        match Credentials::parse(payload.expose()) {
            Ok(credentials) => Ok(Stored::Usable(credentials)),
            Err(ParseError::NoClaudeLogin) => Ok(Stored::NoClaudeLogin),
            Err(error) => Err(credentials_error(
                &login.source,
                error.describe().to_owned(),
            )),
        }
    }

    /// Sends the usage request and records the answer. `state` already says a request was sent.
    async fn request(
        api: &Api,
        login: &Login,
        credentials: &Credentials,
        mut account: Account,
        state: &mut AccountState,
        now: DateTime<Utc>,
    ) -> std::result::Result<Snapshot, RequestFailure> {
        let request_error = |reason| Error::Request {
            provider: ID,
            reason,
        };
        let response = match api.usage(&credentials.access_token).await {
            Ok(response) => response,
            Err(UsageError::NotSent(reason)) => {
                return Err(RequestFailure::NotSent(format!(
                    "Could not reach Anthropic: {reason}. Check the internet connection."
                )));
            }
            // Anthropic may have counted it, so it is recorded like any answer QuotaDial cannot use.
            Err(UsageError::NoAnswer(reason)) => {
                return state
                    .record_failure(&POLICY, format!("Anthropic did not answer: {reason}."), now)
                    .map_err(|reason| RequestFailure::Error(request_error(reason)));
            }
        };

        let snapshot = match response {
            UsageResponse::Usage(body) => {
                // Still the stand-in for an account Anthropic has not named: the answer names it
                // if it can.
                if login.identity.is_none() && account.same_as(&login.account()) {
                    account = match known_account(login, state, &account) {
                        Some(known) => known,
                        None => match api.profile(&credentials.access_token).await {
                            Some(identity) => named(account, &identity),
                            None => account,
                        },
                    };
                }
                state.record_usage(snapshot_from(account, usage::read(&body, now)))
            }
            UsageResponse::Unauthorized => auth_failure(state, account, advice::rejected(login)),
            UsageResponse::MissingScope => {
                auth_failure(state, account, advice::missing_scope(login))
            }
            UsageResponse::Revoked => auth_failure(state, account, advice::revoked(login)),
            UsageResponse::RateLimited { retry_after } => {
                let retry_after = retry_after
                    .as_deref()
                    .and_then(|value| quotadial_poll::retry_after(value, now));
                state.record_rate_limit(&POLICY, &account, retry_after, now)
            }
            UsageResponse::Failed { status, message } => state
                .record_failure(&POLICY, advice::failed(status, message.as_deref()), now)
                .map_err(|reason| RequestFailure::Error(request_error(reason)))?,
            UsageResponse::Invalid(reason) => state
                .record_failure(
                    &POLICY,
                    format!("Anthropic's answer could not be read: {reason}."),
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

impl Default for ClaudeProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for ClaudeProvider {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ClaudeProvider")
            .field("state_dir", &self.state_dir)
            .field("base_url", &self.base_url)
            .finish_non_exhaustive()
    }
}

/// Keychain items whose folder is unknown and which hold only MCP server sign-ins: not accounts
/// (#34). Kept in the provider's state folder by service, with the modification time each item
/// had, so discovery leaves them out until Claude Code rewrites them.
#[derive(Debug, Default, Serialize, Deserialize)]
struct NotAccounts(BTreeMap<String, Option<String>>);

impl NotAccounts {
    const FILE: &'static str = "not-accounts.json";
    /// The lock taken while the record is read and rewritten.
    const LOCK: &'static str = "not-accounts";

    fn load(dir: &Path) -> Self {
        std::fs::read(dir.join(Self::FILE))
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    /// Saves privately, through a temporary file.
    fn save(&self, dir: &Path) -> std::io::Result<()> {
        use std::io::Write;

        std::fs::create_dir_all(dir)?;
        let temporary = dir.join(format!("{}.tmp", Self::FILE));
        let mut options = std::fs::OpenOptions::new();
        options.create(true).truncate(true).write(true);
        #[cfg(unix)]
        std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
        let mut file = options.open(&temporary)?;
        file.write_all(&serde_json::to_vec(self).map_err(std::io::Error::other)?)?;
        drop(file);
        std::fs::rename(temporary, dir.join(Self::FILE))
    }

    /// Whether `login` is an item recorded as not an account, unchanged since. An item with no
    /// modification time cannot be told apart from a rewritten one, so it is never left out (#39).
    fn excludes(&self, login: &Login) -> bool {
        match &login.source {
            LoginSource::Keychain { service, .. }
                if login.folder == Folder::Unknown && login.generation.is_some() =>
            {
                self.0
                    .get(service)
                    .is_some_and(|generation| *generation == login.generation)
            }
            _ => false,
        }
    }
}

/// What reading a sign-in's token found.
enum Stored {
    Usable(Credentials),
    /// The sign-in disappeared after it was found.
    SignedOut,
    /// The sign-in holds only MCP server sign-ins.
    NoClaudeLogin,
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
impl Provider for ClaudeProvider {
    fn id(&self) -> &'static str {
        ID
    }

    fn discover_accounts(&self) -> Result<Vec<Account>> {
        Ok(self.locate_all()?.iter().map(Login::account).collect())
    }

    fn discovery_warnings(&self) -> Vec<String> {
        self.warnings
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    async fn fetch(&self, account: &Account) -> Result<Snapshot> {
        let store = self.store()?;
        let key = quotadial_poll::account_key(account);
        // Everything below runs under the account's lock, so the sign-in found, the token read
        // and the answer saved belong to one moment, and two processes send one request.
        let _lock = store.lock_account(&key).await?;

        let login = match self.login_for(account).await? {
            Ok(login) => login,
            Err(reason) => {
                let (state, _) = store
                    .load(&key, self.clock.now())
                    .map_err(|error| store.error(&error))?;
                return Ok(needs_sign_in(&state, account.clone(), &reason));
            }
        };
        // Named by where its sign-in was found this time, which may have moved (P-015).
        let account = Account {
            sign_in: Some(login.place.clone()),
            ..account.clone()
        };
        let now = self.clock.now();
        let mut state = Self::prepare(&store, &key, Some(&login), now)?;

        // A sign-in that does not say whose it is, such as an environment token, may reach an
        // account that another sign-in reaches too. Once an earlier answer named its account, that
        // account's floor applies, so two sign-ins never send two requests inside it (#34).
        if login.identity.is_none()
            && let Some(named) = known_account(&login, &state, &account)
        {
            return self.fetch_as(&store, &key, &login, named, now).await;
        }
        match quotadial_poll::decide(&POLICY, &state, &account, login.generation.as_deref(), now) {
            Decision::Request => {}
            Decision::Replay(snapshot) => return Ok(*snapshot),
            Decision::Wait(reason) => {
                return Err(Error::Request {
                    provider: ID,
                    reason,
                });
            }
        }
        let (credentials, account) = match self.credentials(&login, &state, account, now).await? {
            Ok(usable) => usable,
            Err(snapshot) => return Ok(snapshot),
        };
        if login.identity.is_none() {
            // Asked before the usage request, so that request counts against the floor of the
            // account the token belongs to, whichever of its sign-ins asks first.
            if let Some(identity) = self.api()?.profile(&credentials.access_token).await {
                let named = named(account, &identity);
                // Remembered before the request, whatever its answer, so the next fetch goes
                // straight to the named account's floor without reading the token or asking the
                // profile again (#39). A record that cannot be kept only means the profile is asked
                // again next time.
                state.named = Some(named.clone());
                state.named_generation.clone_from(&login.generation);
                state.login_generation.clone_from(&login.generation);
                let _ = store.save(&key, &state);
                return self
                    .request_as(&store, &key, &login, &credentials, named, now)
                    .await;
            }
        }
        self.send(&store, &key, &mut state, &login, &credentials, account)
            .await
    }

    fn fallback(&self, _account: &Account) -> Option<Snapshot> {
        // Claude Code's session logs under `~/.claude/projects` hold token counts, not limit
        // percentages, and the PRD's other fallback, the `anthropic-ratelimit-unified-*` headers,
        // needs a Messages API request that spends the user's usage. Stale numbers come from
        // QuotaDial's own cache inside `fetch` instead.
        None
    }

    fn min_poll_interval(&self) -> Duration {
        MIN_POLL_INTERVAL
    }

    /// `projects` in each folder sign-ins are looked for in: this shell's, the default and every
    /// candidate, as Claude Code writes a session's log beside the sign-in it used.
    fn log_folders(&self) -> Vec<PathBuf> {
        let mut folders: Vec<PathBuf> = Vec::new();
        let candidates = login::candidate_dirs(&self.env, &self.extra_dirs).dirs;
        for dir in self
            .env
            .config_dir()
            .into_iter()
            .chain(self.env.default_config_dir())
            .chain(candidates.into_iter().map(PathBuf::from))
        {
            let projects = dir.join("projects");
            if !folders.contains(&projects) {
                folders.push(projects);
            }
        }
        folders
    }
}

/// Why a credentials file could not be read, and the fix.
fn unreadable(error: &std::io::Error) -> String {
    format!("{error}; check that you can read it")
}

fn credentials_error(source: &LoginSource, reason: String) -> Error {
    Error::Credentials {
        provider: ID,
        location: source.location(),
        reason,
    }
}

/// The account Anthropic named for this sign-in's token earlier, so the profile is asked once per
/// token rather than on every request: the one the profile named (`named`), or else the one an
/// earlier answer was recorded for. An environment token's state is kept per token (its account id
/// is a hash of the token), so it cannot name another token's account. A Keychain or file sign-in
/// counts only while its token is the one that was named.
fn known_account(login: &Login, state: &AccountState, stand_in: &Account) -> Option<Account> {
    let same_token = |generation: &Option<String>| {
        matches!(login.source, LoginSource::Environment(_))
            || (login.generation.is_some() && login.generation == *generation)
    };
    let known = state
        .named
        .as_ref()
        .filter(|_| same_token(&state.named_generation))
        .or_else(|| {
            state
                .last_good
                .as_ref()
                .filter(|_| same_token(&state.last_good_generation))
                .map(|snapshot| &snapshot.account)
        })?;
    (known.account_id != stand_in.account_id).then(|| Account {
        plan: stand_in.plan.clone().or_else(|| known.plan.clone()),
        sign_in: stand_in.sign_in.clone(),
        ..known.clone()
    })
}

/// `account` as Anthropic's profile names it, keeping the plan already known and where its
/// sign-in is.
fn named(account: Account, identity: &Identity) -> Account {
    let named = login::account_for(identity);
    Account {
        plan: account.plan.or(named.plan.clone()),
        sign_in: account.sign_in,
        ..named
    }
}

/// Shown when a sign-in changed while its token was read: its last numbers from the cache, if
/// any, until the next fetch reads it again.
fn changed_while_read(state: &AccountState, account: Account, login: &Login) -> Snapshot {
    let mut snapshot = Snapshot {
        account,
        windows: Vec::new(),
        extra_usage: None,
        status: Status::Stale,
        error: Some(format!(
            "The Claude Code sign-in in {} changed while QuotaDial read it. QuotaDial reads it \
             again at the next check.",
            login.place
        )),
    };
    if let Some(last) = &state.last_good {
        snapshot.windows = last.clone().served_from_cache().windows;
        snapshot.extra_usage.clone_from(&last.extra_usage);
    }
    snapshot
}

/// The error for a lookup whose task stopped before it finished.
fn stopped() -> Error {
    Error::Credentials {
        provider: ID,
        location: "Claude Code's sign-in".to_owned(),
        reason: "looking it up stopped unexpectedly; try again".to_owned(),
    }
}

fn auth_failure(state: &mut AccountState, account: Account, reason: String) -> Snapshot {
    let snapshot = needs_sign_in(state, account, &reason);
    state.record_auth_failure(&snapshot);
    snapshot
}

/// A sign-in that needs the user, keeping the account's last good numbers on show, marked as coming
/// from the cache with their age, so its row does not empty while it waits (#34).
fn needs_sign_in(state: &AccountState, account: Account, reason: &str) -> Snapshot {
    let mut snapshot = Snapshot::auth_required(account, reason);
    if let Some(last) = &state.last_good {
        snapshot.windows = last.clone().served_from_cache().windows;
        snapshot.extra_usage.clone_from(&last.extra_usage);
    }
    snapshot
}

/// The snapshot for a usage response. One with no limits at all is `unsupported`, with a reason,
/// instead of a misleading 0%.
fn snapshot_from(account: Account, reading: Reading) -> Snapshot {
    let spending = reading
        .extra_usage
        .as_ref()
        .is_some_and(|extra| extra.enabled);
    if reading.windows.is_empty() && !spending {
        let reason = advice::no_limits(account.plan.as_deref());
        return Snapshot {
            extra_usage: reading.extra_usage,
            ..Snapshot::unsupported(account, reason)
        };
    }
    Snapshot {
        account,
        windows: reading.windows,
        extra_usage: reading.extra_usage,
        status: Status::Ok,
        error: None,
    }
}

#[cfg(test)]
impl ClaudeProvider {
    /// A provider with every outside dependency replaced: no real environment, keychain, clock,
    /// cache directory, Anthropic or `claude --version`.
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
            extra_dirs: Vec::new(),
            max_accounts: None,
            logins: Mutex::default(),
            warnings: Mutex::default(),
        }
    }

    /// The same, with requests giving up after `timeout`.
    pub(crate) fn with_request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }
}
