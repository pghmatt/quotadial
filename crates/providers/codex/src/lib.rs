//! ChatGPT and Codex usage provider for QuotaDial.
//!
//! Reads the sign-in the Codex CLI already stores in `auth.json`, asks ChatGPT's usage endpoint for
//! the account's limits and maps the answer to the normalized model, classifying windows by their
//! length. It never writes, refreshes or copies the Codex CLI's credentials, sends the token
//! nowhere but ChatGPT, and asks at most once every 2 minutes per account, across every QuotaDial
//! process. When ChatGPT cannot be asked, the newest rate-limit numbers in the Codex CLI's session
//! logs stand in, marked as coming from them.
//!
//! Ported from CodexBar (`Sources/CodexBarCore/` in `../codexbar-ref`). Each module names the
//! files it follows.

mod advice;
mod api;
mod auth;
mod environment;
mod sessions;
mod usage;

#[cfg(test)]
mod provider_tests;

use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, SecondsFormat, TimeDelta, Utc};
use quotadial_core::{Account, Error, Provider, Result, Secret, Snapshot, Status};
use quotadial_poll::{AccountState, Clock, Decision, Policy, StateStore, SystemClock};

use crate::api::{Api, UsageError, UsageResponse};
use crate::auth::{Auth, ParseError};
use crate::environment::Environment;
use crate::sessions::Owner;
use crate::usage::Reading;

/// The provider id used in [`Account::provider`].
pub const ID: &str = "codex";

/// What the provider calls itself (P-020).
pub const NAME: &str = "Codex";

/// The fastest adaptive poll interval in the PRD: every two minutes while a CLI is active.
/// QuotaDial never asks ChatGPT more often than this for one account.
pub const MIN_POLL_INTERVAL: Duration = Duration::from_secs(2 * 60);

/// The floor between requests, enforced across every QuotaDial process, and how messages name
/// ChatGPT and the Codex CLI.
const POLICY: Policy = Policy {
    service: "ChatGPT",
    cli: "The Codex CLI",
    floor: TimeDelta::minutes(2),
};

/// The account id for a Codex CLI signed in with an OpenAI API key, which has no ChatGPT account.
const API_KEY_ACCOUNT: &str = "api-key";

/// The account id for a ChatGPT sign-in whose tokens name no account or user.
const UNNAMED_ACCOUNT: &str = "unnamed";

/// Reads ChatGPT and Codex usage with the sign-in the Codex CLI already stores.
pub struct CodexProvider {
    env: Environment,
    clock: Arc<dyn Clock>,
    state_dir: Option<PathBuf>,
    base_url: String,
    request_timeout: Duration,
    api: OnceLock<std::result::Result<Api, String>>,
}

impl CodexProvider {
    /// Creates the provider for this process's environment.
    #[must_use]
    pub fn new() -> Self {
        Self {
            env: Environment::from_process(),
            clock: Arc::new(SystemClock),
            state_dir: StateStore::default_dir(ID),
            base_url: api::DEFAULT_BASE_URL.to_owned(),
            request_timeout: api::TIMEOUT,
            api: OnceLock::new(),
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

    /// What `auth.json` holds right now.
    fn sign_in(&self) -> Result<SignIn> {
        let Some(path) = self.env.auth_file() else {
            return Ok(SignIn::Missing);
        };
        let text = match std::fs::read_to_string(&path) {
            Ok(text) => Secret::new(text),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(SignIn::Missing);
            }
            Err(error) => {
                return Err(credentials_error(
                    &path,
                    format!("{error}; check that you can read it"),
                ));
            }
        };
        match Auth::parse(text.expose()) {
            Ok(auth) => Ok(SignIn::ChatGpt {
                auth,
                written: written(&path),
            }),
            Err(ParseError::ApiKey) => Ok(SignIn::ApiKey),
            Err(error) => Err(credentials_error(&path, error.describe().to_owned())),
        }
    }

    /// Sends the usage request and records the answer. `state` already says a request was sent.
    async fn request(
        api: &Api,
        auth: &Auth,
        mut account: Account,
        state: &mut AccountState,
        now: DateTime<Utc>,
    ) -> std::result::Result<Snapshot, RequestFailure> {
        let request_error = |reason| Error::Request {
            provider: ID,
            reason,
        };
        let response = match api
            .usage(&auth.access_token, auth.account_id.as_deref())
            .await
        {
            Ok(response) => response,
            Err(UsageError::NotSent(reason)) => {
                return Err(RequestFailure::NotSent(format!(
                    "Could not reach ChatGPT: {reason}. Check the internet connection."
                )));
            }
            // ChatGPT may have counted it, so it is recorded like any answer QuotaDial cannot use.
            Err(UsageError::NoAnswer(reason)) => {
                return state
                    .record_failure(&POLICY, format!("ChatGPT did not answer: {reason}."), now)
                    .map_err(|reason| RequestFailure::Error(request_error(reason)));
            }
        };

        let snapshot = match response {
            UsageResponse::Usage(body) => {
                let reading = usage::read(&body, now);
                if let Some(reason) = foreign_answer(&reading, auth) {
                    return state
                        .record_failure(&POLICY, reason, now)
                        .map_err(|reason| {
                            RequestFailure::Error(Error::InvalidResponse {
                                provider: ID,
                                reason,
                            })
                        });
                }
                if let Some(plan) = &reading.plan {
                    account.plan = Some(plan.clone());
                }
                state.record_usage(snapshot_from(account, reading))
            }
            UsageResponse::Unauthorized => {
                let snapshot = Snapshot::auth_required(account, advice::REJECTED);
                state.record_auth_failure(&snapshot);
                snapshot
            }
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
                    format!("ChatGPT's answer could not be read: {reason}."),
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

impl Default for CodexProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for CodexProvider {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CodexProvider")
            .field("state_dir", &self.state_dir)
            .field("base_url", &self.base_url)
            .finish_non_exhaustive()
    }
}

/// What `auth.json` holds.
enum SignIn {
    /// No sign-in.
    Missing,
    /// An OpenAI API key, which has no ChatGPT plan limits.
    ApiKey,
    /// A ChatGPT sign-in, and when the Codex CLI last wrote the file: when it signed in or
    /// refreshed the token.
    ChatGpt {
        auth: Auth,
        written: Option<DateTime<Utc>>,
    },
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
impl Provider for CodexProvider {
    fn id(&self) -> &'static str {
        ID
    }

    fn name(&self) -> &'static str {
        NAME
    }

    fn discover_accounts(&self) -> Result<Vec<Account>> {
        // Where the sign-in was read, as every account carries it (P-015).
        let place = self
            .env
            .auth_file()
            .map(|path| quotadial_core::text::tilde(&path, self.env.home.as_deref()));
        let account = match self.sign_in()? {
            SignIn::Missing => return Ok(Vec::new()),
            SignIn::ApiKey => api_key_account(),
            SignIn::ChatGpt { auth, .. } => account_for(&auth),
        };
        Ok(vec![Account {
            sign_in_use: sign_in_use(place.as_deref()),
            sign_in: place,
            ..account
        }])
    }

    async fn fetch(&self, account: &Account) -> Result<Snapshot> {
        let store = self.store()?;
        let key = quotadial_poll::account_key(account);
        // Everything below runs under the account's lock, so the sign-in read and the answer saved
        // belong to one moment, and two processes send one request.
        let _lock = store.lock_account(&key).await?;

        let (auth, generation) = match self.sign_in()? {
            SignIn::Missing => {
                return Ok(Snapshot::auth_required(account.clone(), advice::SIGNED_OUT));
            }
            SignIn::ApiKey if account.account_id == API_KEY_ACCOUNT => {
                return Ok(Snapshot::unsupported(account.clone(), advice::API_KEY));
            }
            SignIn::ApiKey => {
                return Ok(Snapshot::auth_required(
                    account.clone(),
                    advice::DIFFERENT_ACCOUNT,
                ));
            }
            SignIn::ChatGpt { auth, written } => (auth, written),
        };
        // Changes whenever the Codex CLI rewrites the file, which allows one early retry after a
        // rejected sign-in.
        let generation = generation.map(|at| at.to_rfc3339_opts(SecondsFormat::Nanos, true));
        if !account_for(&auth).same_as(account) {
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
        if auth.account_id.is_none() {
            changed |= state.forget_numbers_after_new_sign_in(&POLICY, generation.as_deref());
        }
        if changed {
            store
                .save(&key, &state)
                .map_err(|error| store.error(&error))?;
        }
        match quotadial_poll::decide(&POLICY, &state, account, generation.as_deref(), now) {
            Decision::Request => {}
            Decision::Replay(snapshot) => return Ok(*snapshot),
            Decision::Wait(reason) => {
                return Err(Error::Request {
                    provider: ID,
                    reason,
                });
            }
        }

        let mut account = account.clone();
        if auth.plan.is_some() {
            account.plan.clone_from(&auth.plan);
        }
        // Checked locally, so a dead token costs no request. Refreshing it is the Codex CLI's job.
        if let Some(expired_at) = auth.expires_at.filter(|_| auth.is_expired(now)) {
            return Ok(Snapshot::auth_required(
                account,
                advice::expired(expired_at),
            ));
        }

        let api = self.api()?;
        // Save the floor before sending, so it holds even if QuotaDial stops mid-request. If the
        // cache cannot be written, no request is sent.
        let before = state.clone();
        state.start_request(&POLICY, now, generation);
        store
            .save(&key, &state)
            .map_err(|error| store.error(&error))?;

        match Self::request(api, &auth, account, &mut state, now).await {
            Err(RequestFailure::NotSent(message)) => {
                // Nothing left this machine, so the floor is put back as it was. If this save
                // fails, the floor simply holds for 2 minutes.
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

    fn fallback(&self, account: &Account) -> Option<Snapshot> {
        let SignIn::ChatGpt { auth, written } = self.sign_in().ok()? else {
            return None;
        };
        if !account_for(&auth).same_as(account) {
            return None;
        }
        let owner = Owner {
            account_id: auth.account_id.as_deref()?,
            user_id: auth.user_id.as_deref(),
            plan: auth.plan.as_deref(),
            since: written,
        };
        let reading = sessions::newest(&self.env.sessions_dir()?, owner, self.clock.now())?;
        if reading.windows.is_empty() {
            return None;
        }
        let mut account = account.clone();
        if reading.plan.is_some() {
            account.plan = reading.plan;
        }
        Some(Snapshot {
            account,
            windows: reading.windows,
            extra_usage: reading.extra_usage,
            status: Status::Stale,
            error: Some(advice::from_session_log(reading.at)),
        })
    }

    fn min_poll_interval(&self) -> Duration {
        MIN_POLL_INTERVAL
    }

    /// `sessions` and `archived_sessions` in the Codex CLI's folder.
    fn log_folders(&self) -> Vec<PathBuf> {
        self.env
            .codex_dir()
            .map(|dir| vec![dir.join("sessions"), dir.join("archived_sessions")])
            .unwrap_or_default()
    }
}

/// The account `auth.json` names: the ChatGPT user in a ChatGPT account (workspace), so one person
/// in two workspaces is two accounts whose usage never blends.
fn account_for(auth: &Auth) -> Account {
    let (account_id, org) = match (&auth.user_id, &auth.account_id) {
        (Some(user_id), workspace) => (user_id.clone(), workspace.clone()),
        (None, Some(workspace)) => (workspace.clone(), None),
        (None, None) => (UNNAMED_ACCOUNT.to_owned(), None),
    };
    Account {
        provider: ID.to_owned(),
        provider_name: NAME.to_owned(),
        account_id,
        display_name: auth
            .email
            .clone()
            .unwrap_or_else(|| "ChatGPT account".to_owned()),
        plan: auth.plan.clone(),
        org,
        sign_in: None,
        sign_in_use: sign_in_use(None),
    }
}

fn api_key_account() -> Account {
    Account {
        provider: ID.to_owned(),
        provider_name: NAME.to_owned(),
        account_id: API_KEY_ACCOUNT.to_owned(),
        display_name: "OpenAI API key".to_owned(),
        plan: None,
        org: None,
        sign_in: None,
        sign_in_use: sign_in_use(None),
    }
}

/// What QuotaDial does with the sign-in in `auth.json` at `place`, in the Codex CLI's words
/// (P-020): where it read it, or, with no place known, where the Codex CLI keeps it, as the popover
/// used to say.
fn sign_in_use(place: Option<&str>) -> String {
    place.map_or_else(
        || {
            "QuotaDial reads the sign-in the Codex CLI keeps in auth.json, in ~/.codex or the \
             folder CODEX_HOME names. It only reads the sign-in and never changes it. If it stops \
             working, run codex once."
                .to_owned()
        },
        quotadial_core::text::sign_in_read_from,
    )
}

/// Why an answer cannot be this sign-in's: ChatGPT named another user or workspace than the token
/// says. Its numbers are not used, so they can never be saved under the wrong account.
fn foreign_answer(reading: &Reading, auth: &Auth) -> Option<String> {
    let differs = |answered: &Option<String>, expected: &Option<String>| matches!((answered, expected), (Some(answered), Some(expected)) if answered != expected);
    (differs(&reading.user_id, &auth.user_id) || differs(&reading.account_id, &auth.account_id))
        .then(|| {
            "ChatGPT's answer names another account than the Codex CLI's sign-in, so it was not \
             used."
                .to_owned()
        })
}

/// When the file was last written.
fn written(path: &Path) -> Option<DateTime<Utc>> {
    let modified = std::fs::metadata(path).ok()?.modified().ok()?;
    Some(DateTime::<Utc>::from(modified))
}

fn credentials_error(path: &Path, reason: String) -> Error {
    Error::Credentials {
        provider: ID,
        location: path.display().to_string(),
        reason,
    }
}

/// The snapshot for a usage response. One with no limits at all is `unsupported`, with a reason,
/// instead of a misleading 0%.
fn snapshot_from(account: Account, reading: Reading) -> Snapshot {
    let spending = reading
        .extra_usage
        .as_ref()
        .is_some_and(|extra| extra.enabled);
    if reading.windows.is_empty() && !spending {
        return Snapshot {
            extra_usage: reading.extra_usage,
            ..Snapshot::unsupported(account, advice::NO_LIMITS)
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
impl CodexProvider {
    /// A provider with every outside dependency replaced: no real environment, clock, cache
    /// directory or ChatGPT.
    pub(crate) fn for_tests(
        env: Environment,
        clock: Arc<dyn Clock>,
        state_dir: PathBuf,
        base_url: &str,
    ) -> Self {
        Self {
            env,
            clock,
            state_dir: Some(state_dir),
            base_url: base_url.to_owned(),
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

#[cfg(test)]
mod sign_in_tests {
    use super::*;

    #[test]
    fn its_name_and_every_sentence_it_says_of_a_sign_in_keep_to_plain_words() {
        for words in [
            NAME.to_owned(),
            sign_in_use(None),
            sign_in_use(Some("~/.codex/auth.json")),
            sign_in_use(Some("~/work/codex/auth.json")),
        ] {
            // #52's plain words: never "quota", "auth" or "credential" outside a name.
            assert_eq!(
                quotadial_core::text::unplain_words(&words),
                Vec::<String>::new(),
                "{words}"
            );
        }
    }

    #[test]
    fn with_no_place_known_the_sentence_says_where_the_codex_cli_keeps_its_sign_in() {
        assert_eq!(
            sign_in_use(None),
            "QuotaDial reads the sign-in the Codex CLI keeps in auth.json, in ~/.codex or the \
             folder CODEX_HOME names. It only reads the sign-in and never changes it. If it stops \
             working, run codex once."
        );
    }
}
