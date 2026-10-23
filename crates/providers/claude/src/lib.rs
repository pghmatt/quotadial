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

use std::fmt;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, TimeDelta, Utc};
use quotadial_core::{Account, Error, Provider, Result, Secret, Snapshot, Status};
use quotadial_poll::{AccountState, Clock, Decision, Policy, StateStore, SystemClock};

use crate::api::{Api, UsageError, UsageResponse};
use crate::credentials::{Credentials, ParseError};
use crate::environment::Environment;
use crate::identity::Identity;
use crate::keychain::Keychain;
use crate::login::{Login, LoginSource};
use crate::usage::Reading;

/// The provider id used in [`Account::provider`].
pub const ID: &str = "claude";

/// Anthropic rate-limits its usage endpoint aggressively, so QuotaDial polls it at most once every
/// five minutes per account.
pub const MIN_POLL_INTERVAL: Duration = Duration::from_secs(5 * 60);

/// The floor between requests, enforced across every QuotaDial process, and how messages name
/// Anthropic and Claude Code.
const POLICY: Policy = Policy {
    service: "Anthropic",
    cli: "Claude Code",
    floor: TimeDelta::minutes(5),
};

/// Reads Claude usage with the login Claude Code already stores.
pub struct ClaudeProvider {
    env: Environment,
    keychain: Arc<dyn Keychain>,
    clock: Arc<dyn Clock>,
    state_dir: Option<PathBuf>,
    base_url: String,
    claude_code_version: OnceLock<Option<String>>,
    request_timeout: Duration,
    api: OnceLock<std::result::Result<Api, String>>,
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
            claude_code_version: OnceLock::new(),
            request_timeout: api::TIMEOUT,
            api: OnceLock::new(),
        }
    }

    fn locate(&self) -> Result<Option<Login>> {
        login::locate(&self.env, self.keychain.as_ref())
    }

    /// The HTTP client, built on first use. Claude Code's version is detected only then, because
    /// it is needed only for a request.
    fn api(&self) -> Result<&Api> {
        self.api
            .get_or_init(|| {
                let version = self
                    .claude_code_version
                    .get_or_init(api::detect_claude_code_version);
                Api::new(
                    &self.base_url,
                    api::user_agent(version.as_deref()),
                    self.request_timeout,
                )
            })
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
                .map_err(|error| credentials_error(&login.source, error.to_string()))?;
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
                Err(error) => return Err(credentials_error(&login.source, error.to_string())),
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
                    "Could not reach Anthropic: {reason}."
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
                if login.identity.is_none() {
                    account = match known_account(login, state, &account) {
                        Some(known) => known,
                        None => match api.profile(&credentials.access_token).await {
                            Some(identity) => named(account, &identity),
                            None => account,
                        },
                    };
                }
                let snapshot = snapshot_from(account, usage::read(&body, now));
                state.record_usage(&snapshot);
                snapshot
            }
            UsageResponse::Unauthorized => {
                auth_failure(state, account, advice::rejected(&login.source))
            }
            UsageResponse::MissingScope => {
                auth_failure(state, account, advice::missing_scope(&login.source))
            }
            UsageResponse::Revoked => auth_failure(state, account, advice::revoked(&login.source)),
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
        Ok(self
            .locate()?
            .map(|login| login.account())
            .into_iter()
            .collect())
    }

    async fn fetch(&self, account: &Account) -> Result<Snapshot> {
        let store = self.store()?;
        let key = quotadial_poll::account_key(account);
        // Everything below runs under the account's lock, so the sign-in found, the token read
        // and the answer saved belong to one moment, and two processes send one request.
        let _lock = store.lock_account(&key).await?;

        let Some(login) = self.locate()? else {
            return Ok(Snapshot::auth_required(account.clone(), advice::SIGNED_OUT));
        };
        if !same_account(&login.account(), account) {
            return Ok(Snapshot::auth_required(
                account.clone(),
                advice::DIFFERENT_ACCOUNT,
            ));
        }

        let now = self.clock.now();
        let (mut state, damaged) = store.load(&key, now).map_err(|error| store.error(&error))?;
        // A damaged file is saved again at once, so the hold it now records cannot restart.
        let mut changed = damaged;
        changed |= state.reanchor(now);
        if login.identity.is_none() && !matches!(login.source, LoginSource::Environment(_)) {
            changed |= state.forget_numbers_after_new_sign_in(&POLICY, login.generation.as_deref());
        }
        if changed {
            store
                .save(&key, &state)
                .map_err(|error| store.error(&error))?;
        }
        match quotadial_poll::decide(&POLICY, &state, account, login.generation.as_deref(), now) {
            Decision::Request => {}
            Decision::Replay(snapshot) => return Ok(*snapshot),
            Decision::Wait(reason) => {
                return Err(Error::Request {
                    provider: ID,
                    reason,
                });
            }
        }

        let credentials = match self.read_credentials(&login).await? {
            Stored::Usable(credentials) => credentials,
            Stored::SignedOut => {
                return Ok(Snapshot::auth_required(account.clone(), advice::SIGNED_OUT));
            }
            Stored::NoClaudeLogin => {
                return Ok(Snapshot::auth_required(
                    account.clone(),
                    advice::no_claude_login(&login.source),
                ));
            }
        };
        // Claude Code may have switched accounts while the token was read. A token read during a
        // switch must not be counted as this account's.
        match self.locate()? {
            Some(current) if same_account(&current.account(), account) => {}
            Some(_) => {
                return Ok(Snapshot::auth_required(
                    account.clone(),
                    advice::DIFFERENT_ACCOUNT,
                ));
            }
            None => return Ok(Snapshot::auth_required(account.clone(), advice::SIGNED_OUT)),
        }

        let mut account = account.clone();
        if let Some(plan) = credentials.plan() {
            account.plan = Some(plan);
        }
        // Checked locally, so a dead token costs no request. Refreshing it is Claude Code's job.
        if let Some(expired_at) = credentials
            .expires_at
            .filter(|_| credentials.is_expired(now))
        {
            return Ok(Snapshot::auth_required(
                account,
                advice::expired(expired_at),
            ));
        }
        if !credentials.can_read_usage() {
            return Ok(Snapshot::auth_required(
                account,
                advice::missing_scope(&login.source),
            ));
        }

        let api = self.api()?;
        // Save the floor before sending, so it holds even if QuotaDial stops mid-request. If the
        // cache cannot be written, no request is sent.
        let before = state.clone();
        state.start_request(&POLICY, now, login.generation.clone());
        store
            .save(&key, &state)
            .map_err(|error| store.error(&error))?;

        match Self::request(api, &login, &credentials, account, &mut state, now).await {
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
        // Claude Code's session logs under `~/.claude/projects` hold token counts, not limit
        // percentages, and the PRD's other fallback, the `anthropic-ratelimit-unified-*` headers,
        // needs a Messages API request that spends the user's usage. Stale numbers come from
        // QuotaDial's own cache inside `fetch` instead.
        None
    }

    fn min_poll_interval(&self) -> Duration {
        MIN_POLL_INTERVAL
    }
}

fn credentials_error(source: &LoginSource, reason: String) -> Error {
    Error::Credentials {
        provider: ID,
        location: source.location(),
        reason,
    }
}

/// The same account, as `Account` identity is defined: provider, organization and account id.
fn same_account(a: &Account, b: &Account) -> bool {
    a.provider == b.provider && a.account_id == b.account_id && a.org == b.org
}

/// The account Anthropic named for this environment token on an earlier request. The state is
/// kept per token (its account id is a hash of the token), so it cannot name another token's
/// account. Other sign-ins without a known account are always asked about again.
fn known_account(login: &Login, state: &AccountState, stand_in: &Account) -> Option<Account> {
    if !matches!(login.source, LoginSource::Environment(_)) {
        return None;
    }
    let known = &state.last_good.as_ref()?.account;
    (known.account_id != stand_in.account_id).then(|| Account {
        plan: stand_in.plan.clone().or_else(|| known.plan.clone()),
        ..known.clone()
    })
}

/// `account` as Anthropic's profile names it, keeping the plan already known.
fn named(account: Account, identity: &Identity) -> Account {
    let named = login::account_for(identity);
    Account {
        plan: account.plan.or(named.plan.clone()),
        ..named
    }
}

fn auth_failure(state: &mut AccountState, account: Account, reason: String) -> Snapshot {
    let snapshot = Snapshot::auth_required(account, reason);
    state.record_auth_failure(&snapshot);
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
            claude_code_version: OnceLock::from(Some("2.1.286".to_owned())),
            request_timeout: api::TIMEOUT,
            api: OnceLock::new(),
        }
    }

    /// The same, with requests giving up after `timeout`.
    pub(crate) fn with_request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }
}
