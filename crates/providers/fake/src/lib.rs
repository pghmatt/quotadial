//! QuotaDial's example provider, `fake` (D-117): made-up accounts for pictures of the app, such as
//! the README's recording, so that no real account is ever shown. Only the example build has it:
//! the `fake` feature of `quotadial-providers`, which no release turns on, puts it in place of
//! Claude and Codex.
//!
//! It reads no sign-in, no Keychain and no network, and it keeps nothing. Its accounts are built in
//! (`fixtures/fake/example.json`), or come from a JSON file of the same form that
//! `QUOTADIAL_FAKE_ACCOUNTS` names, read once when the provider is made. Each is called "Example",
//! with a plan and no address. Each window fills at its own pace from the first whole minute after
//! the provider is made, and starts again at each reset, so the daemon's history of it gives it a
//! pace, as a real account's does. Every window carries the time it was read and says it is an example
//! (`Source::Example`).

mod examples;

use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use quotadial_core::{Account, Error, Provider, Result, Snapshot, Status};

use crate::examples::Examples;

/// The provider id used in [`Account::provider`].
pub const ID: &str = "fake";

/// What the provider calls itself, so no picture of it is taken for a real provider (P-020).
pub const NAME: &str = "Example";

/// What QuotaDial does with an example account's sign-in: there is none (P-020).
pub const SIGN_IN_USE: &str = "An example account, made up by QuotaDial's example build for \
                               pictures of the app. It has no sign-in, and QuotaDial asks no one \
                               for its numbers.";

/// The environment variable naming a JSON file of example accounts to show in place of the
/// built-in ones, in the form of `fixtures/fake/example.json`.
pub const ACCOUNTS_VARIABLE: &str = "QUOTADIAL_FAKE_ACCOUNTS";

/// How often the daemon may read an example account. Nothing is asked, so a minute keeps its
/// numbers moving without filling the history.
pub const MIN_POLL_INTERVAL: Duration = Duration::from_secs(60);

/// What every example account is called, so no picture of one is mistaken for a real account.
pub const DISPLAY_NAME: &str = "Example";

/// Where the time is read, which tests move on by hand.
type Clock = Arc<dyn Fn() -> DateTime<Utc> + Send + Sync>;

/// Made-up accounts for pictures of the app.
pub struct FakeProvider {
    /// The examples, or why the file `QUOTADIAL_FAKE_ACCOUNTS` names cannot be used.
    examples: std::result::Result<Examples, String>,
    /// When its examples start, the first whole minute after it was made: every window fills from
    /// then.
    start: DateTime<Utc>,
    clock: Clock,
}

impl FakeProvider {
    /// The built-in examples, or those in the file `QUOTADIAL_FAKE_ACCOUNTS` names when it is set,
    /// filling from now.
    #[must_use]
    pub fn new() -> Self {
        let file = std::env::var_os(ACCOUNTS_VARIABLE)
            .filter(|path| !path.is_empty())
            .map(PathBuf::from);
        Self::from_file(file.as_deref(), Arc::new(Utc::now))
    }

    /// The examples in `file`, or the built-in ones without one, filling from what `clock` says
    /// now.
    fn from_file(file: Option<&Path>, clock: Clock) -> Self {
        let examples = match file {
            Some(path) => examples::read(path),
            None => examples::parse(examples::BUILT_IN),
        };
        Self {
            examples,
            start: examples::next_minute(clock()),
            clock,
        }
    }

    fn examples(&self) -> Result<&Examples> {
        self.examples
            .as_ref()
            .map_err(|reason| Error::InvalidResponse {
                provider: ID,
                reason: reason.clone(),
            })
    }
}

impl Default for FakeProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for FakeProvider {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FakeProvider")
            .field("start", &self.start)
            .finish_non_exhaustive()
    }
}

/// The id of the example account at `index`, counted from 0: `example-1` for the first.
fn account_id(index: usize) -> String {
    format!("example-{}", index + 1)
}

#[async_trait]
impl Provider for FakeProvider {
    fn id(&self) -> &'static str {
        ID
    }

    fn name(&self) -> &'static str {
        NAME
    }

    fn asks(&self) -> Option<&'static str> {
        None
    }

    /// Every example account, whatever the licence: they stand for the accounts a first run finds,
    /// one per provider, so none is held back.
    fn discover_accounts(&self) -> Result<Vec<Account>> {
        Ok(self
            .examples()?
            .accounts
            .iter()
            .enumerate()
            .map(|(index, example)| Account {
                provider: ID.to_owned(),
                provider_name: NAME.to_owned(),
                account_id: account_id(index),
                display_name: DISPLAY_NAME.to_owned(),
                plan: Some(example.plan.clone()),
                org: None,
                sign_in: None,
                sign_in_use: SIGN_IN_USE.to_owned(),
            })
            .collect())
    }

    async fn fetch(&self, account: &Account) -> Result<Snapshot> {
        let now = (self.clock)();
        let example = self
            .examples()?
            .accounts
            .iter()
            .enumerate()
            .find(|(index, _)| account.provider == ID && account.account_id == account_id(*index))
            .map(|(_, example)| example)
            .ok_or_else(|| Error::InvalidResponse {
                provider: ID,
                reason: format!(
                    "{} is not one of the example accounts",
                    quotadial_core::redact_secrets(&account.account_id)
                ),
            })?;
        Ok(Snapshot {
            account: account.clone(),
            windows: example
                .windows
                .iter()
                .map(|window| window.at(self.start, now))
                .collect(),
            extra_usage: None,
            status: Status::Ok,
            error: None,
        })
    }

    fn fallback(&self, _account: &Account) -> Option<Snapshot> {
        // The examples are always to hand; there is nothing to fall back on.
        None
    }

    fn min_poll_interval(&self) -> Duration {
        MIN_POLL_INTERVAL
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use chrono::TimeDelta;
    use quotadial_core::{Pace, Reading, Source, collect};

    use super::*;

    fn at(rfc3339: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(rfc3339)
            .expect("a time")
            .with_timezone(&Utc)
    }

    #[test]
    fn its_name_and_what_it_says_of_a_sign_in_keep_to_plain_words() {
        for words in [NAME, SIGN_IN_USE] {
            // #52's plain words: never "quota", "auth" or "credential" outside a name.
            assert_eq!(
                quotadial_core::text::unplain_words(words),
                Vec::<String>::new(),
                "{words}"
            );
        }
    }

    /// A clock that stays where the test puts it.
    fn clock(start: &str) -> (Clock, Arc<Mutex<DateTime<Utc>>>) {
        let now = Arc::new(Mutex::new(at(start)));
        let read = Arc::clone(&now);
        let clock: Clock = Arc::new(move || *read.lock().expect("the clock"));
        (clock, now)
    }

    fn built_in(start: &str) -> (FakeProvider, Arc<Mutex<DateTime<Utc>>>) {
        let (clock, now) = clock(start);
        (FakeProvider::from_file(None, clock), now)
    }

    #[test]
    fn its_accounts_are_called_example_with_a_plan_and_no_address() {
        let (provider, _) = built_in("2099-01-01T00:00:00Z");

        let accounts = provider.discover_accounts().expect("the built-in examples");

        assert_eq!(provider.id(), "fake");
        assert_eq!(provider.name(), "Example");
        let no_sign_in = "An example account, made up by QuotaDial's example build for pictures \
                          of the app. It has no sign-in, and QuotaDial asks no one for its \
                          numbers.";
        assert_eq!(
            accounts,
            [
                Account {
                    provider: "fake".into(),
                    provider_name: "Example".into(),
                    account_id: "example-1".into(),
                    display_name: "Example".into(),
                    plan: Some("Claude Max 5x".into()),
                    org: None,
                    sign_in: None,
                    sign_in_use: no_sign_in.into(),
                },
                Account {
                    provider: "fake".into(),
                    provider_name: "Example".into(),
                    account_id: "example-2".into(),
                    display_name: "Example".into(),
                    plan: Some("ChatGPT Pro".into()),
                    org: None,
                    sign_in: None,
                    sign_in_use: no_sign_in.into(),
                },
            ]
        );
        let json = serde_json::to_string(&accounts).expect("serializes");
        assert!(!json.contains('@'), "no address anywhere: {json}");
    }

    #[tokio::test]
    async fn every_window_says_it_is_an_example_and_when_it_was_read() {
        let (provider, now) = built_in("2099-01-01T00:00:00Z");
        let accounts = provider.discover_accounts().expect("examples");
        *now.lock().expect("the clock") = at("2099-01-01T00:30:00Z");

        for account in &accounts {
            let snapshot = provider.fetch(account).await.expect("a snapshot");
            assert_eq!(&snapshot.account, account);
            assert_eq!(snapshot.status, Status::Ok);
            assert_eq!(snapshot.error, None);
            assert!(!snapshot.windows.is_empty());
            for window in &snapshot.windows {
                assert_eq!(window.source, Source::Example, "{}", window.label);
                assert_eq!(window.fetched_at, at("2099-01-01T00:30:00Z"));
                assert!(window.resets_at.is_some());
            }
        }
        let session = provider
            .fetch(&accounts[0])
            .await
            .expect("a snapshot")
            .windows[2]
            .clone();
        assert_eq!(session.label, "Session");
        assert_eq!(
            session.used_pct, 42.3,
            "38% and half an hour at 8.5%/h, to a tenth"
        );
    }

    #[tokio::test]
    async fn the_first_reading_shows_each_window_as_the_examples_give_it() {
        let (provider, now) = built_in("2099-01-01T00:00:30Z");
        let account = provider.discover_accounts().expect("examples")[0].clone();
        *now.lock().expect("the clock") = at("2099-01-01T00:00:31Z");

        let session = provider.fetch(&account).await.expect("a snapshot").windows[2].clone();

        assert_eq!(session.used_pct, 38.0);
        assert_eq!(
            session.resets_at,
            Some(at("2099-01-01T02:11:00Z")),
            "2 h 10 min from the next whole minute"
        );
        assert_eq!(session.fetched_at, at("2099-01-01T00:00:31Z"));
        assert_eq!(
            quotadial_core::text::resets(session.resets_at, session.fetched_at),
            "resets in 2 h 10 min"
        );
    }

    #[tokio::test]
    async fn the_daemons_readings_of_a_window_give_it_its_pace() {
        let (provider, now) = built_in("2099-01-01T00:00:00Z");
        let account = provider.discover_accounts().expect("examples")[0].clone();
        let mut readings = Vec::new();
        for minutes in [0, 15, 30] {
            *now.lock().expect("the clock") =
                at("2099-01-01T00:00:00Z") + TimeDelta::minutes(minutes);
            let session = &provider.fetch(&account).await.expect("a snapshot").windows[2];
            readings.push(Reading {
                fetched_at: session.fetched_at,
                used_pct: session.used_pct,
            });
        }
        *now.lock().expect("the clock") = at("2099-01-01T00:45:00Z");
        let session = provider.fetch(&account).await.expect("a snapshot").windows[2].clone();

        let pace = Pace::of(&session, &readings).expect("4 readings over 45 minutes");

        assert!(
            (pace.pct_per_hour - 8.5).abs() < 0.1,
            "the example's 8.5%/h, read to a tenth: {pace:?}"
        );
        assert_eq!(pace.runs_out_at, None, "56% at its reset, so it lasts");
    }

    #[tokio::test]
    async fn an_account_that_is_not_an_example_is_an_error_naming_it() {
        let (provider, _) = built_in("2099-01-01T00:00:00Z");
        let mut stranger = provider.discover_accounts().expect("examples")[0].clone();
        stranger.account_id = "example-9".into();

        let error = provider.fetch(&stranger).await.expect_err("not an example");

        assert_eq!(
            error.to_string(),
            "unexpected response: example-9 is not one of the example accounts"
        );
    }

    #[tokio::test]
    async fn a_file_of_examples_replaces_the_built_in_ones() {
        let temp = tempfile::tempdir().expect("temp dir");
        let path = temp.path().join("examples.json");
        std::fs::write(
            &path,
            r#"{ "accounts": [{ "plan": "Team", "windows": [{ "kind": "daily", "label": "Daily",
                "window_seconds": 86400, "used_pct": 5, "resets_in_seconds": 3600 }] }] }"#,
        )
        .expect("writes");
        let (clock, _) = clock("2099-01-01T00:00:00Z");
        let provider = FakeProvider::from_file(Some(&path), clock);

        let accounts = provider.discover_accounts().expect("the file's examples");
        let snapshot = provider.fetch(&accounts[0]).await.expect("a snapshot");

        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].plan.as_deref(), Some("Team"));
        assert_eq!(snapshot.windows.len(), 1);
        assert_eq!(snapshot.windows[0].label, "Daily");
        assert_eq!(snapshot.windows[0].used_pct, 5.0);
        assert_eq!(
            snapshot.windows[0].resets_at,
            Some(at("2099-01-01T01:00:00Z"))
        );
    }

    #[tokio::test]
    async fn a_file_that_cannot_be_used_shows_why_and_no_account() {
        let temp = tempfile::tempdir().expect("temp dir");
        let path = temp.path().join("examples.json");
        std::fs::write(&path, r#"{ "accounts": "none" }"#).expect("writes");
        let (clock, _) = clock("2099-01-01T00:00:00Z");
        let provider: Arc<dyn Provider> = Arc::new(FakeProvider::from_file(Some(&path), clock));

        let report = collect(&[provider]).await;

        assert!(report.snapshots.is_empty());
        assert_eq!(report.errors.len(), 1);
        assert_eq!(report.errors[0].provider, "fake");
        let message = &report.errors[0].message;
        // A place in the home folder is written with `~`, as a temporary folder is on Windows.
        assert!(
            message.starts_with(&format!(
                "unexpected response: the example accounts in {} cannot be used: it is not \
                 the example accounts' JSON (invalid type: string \"none\"",
                quotadial_core::text::tilde(&path, std::env::home_dir().as_deref())
            )),
            "{message}"
        );
    }

    #[test]
    fn it_asks_no_one_so_it_has_no_logs_and_nothing_to_fall_back_on() {
        let (provider, _) = built_in("2099-01-01T00:00:00Z");
        let account = provider.discover_accounts().expect("examples")[0].clone();

        assert_eq!(provider.min_poll_interval(), Duration::from_secs(60));
        assert_eq!(provider.fallback(&account), None);
        assert!(provider.log_folders().is_empty());
        assert_eq!(provider.held_back(), 0);
        assert!(provider.discovery_warnings().is_empty());
    }
}
