//! Antigravity for QuotaDial: each of an Antigravity account's limits, from the numbers Antigravity
//! CLI's status line hands to a command the user adds to its settings (D-119, D-126).
//!
//! Antigravity's terms forbid another tool to use its sign-in, so QuotaDial is never a reader of
//! Antigravity's sign-in or of its app's local server, and asks no one about it (P-021). Its CLI's
//! status line is documented for exactly this, handing each limit's share and reset to a command
//! the user names ("Status line customization", antigravity.google/docs/cli/statusline, read on
//! 2026-10-25), and QuotaDial never edits Antigravity's settings to add itself: `doctor`, the
//! account's sign-in panel and `docs/INSTALL.md` give the block to add.
//!
//! - `quotadial antigravity status-line` (`command`), through the provider's own commands (P-018):
//!   Antigravity CLI runs it with its session's state on standard input; it keeps only each limit's
//!   share or amount and reset, the plan, the account's address and its own time (`handover`), in
//!   `antigravity.json` in QuotaDial's data folder (`store`), then runs any status line command
//!   named after it, with the same input. The run, the chaining and the file written whole are
//!   `quotadial-status-line`'s, lifted from this crate so Claude's command shares them (D-145).
//! - Discovery looks at two things only: that file, and whether Antigravity CLI's folder
//!   ([`FOLDER`]) is there, without opening anything in it. With the file, the account is the one
//!   it names; with only the folder, an account whose line says what to add, so its sign-in panel
//!   is there before any numbers; with neither, no account.
//! - `fetch` reads the file at each poll, with the time it was handed over as every window's
//!   `fetched_at` and `status_line` as its source (P-022). Numbers handed over longer ago than two
//!   polls are `stale`, with that time in words (#52), and a limit that has reset since is left out.
//!   A limit Antigravity CLI marks `disabled` is left out, as Claude's provider leaves out a limit
//!   Anthropic sends as null. A limit with an amount left and no share is never a window: the
//!   account's line, or its sign-in panel beside windows, names it with its amount and reset
//!   (P-023).
//!
//! The limits' names follow CodexBar's (`windows`), whose Antigravity provider reads the same
//! numbers through Antigravity's sign-in, which QuotaDial does not.

mod advice;
mod command;
mod handover;
mod store;
mod windows;

use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, TimeDelta, Utc};
use quotadial_core::{Account, Error, Provider, ProviderCommand, Result, Snapshot, Status};

use crate::handover::Handover;

/// The provider id used in [`Account::provider`].
pub const ID: &str = "antigravity";

/// What the provider calls itself (P-020): the limits are the Antigravity account's, which its
/// app, editor and CLI share; the CLI is only where QuotaDial gets them.
pub const NAME: &str = "Antigravity";

/// How often the daemon may look at an Antigravity account again: a read of QuotaDial's own file,
/// which asks no one.
pub const MIN_POLL_INTERVAL: Duration = Duration::from_secs(5 * 60);

/// How old the newest handover may be before its numbers are stale: two polls (#52).
pub const STALE_AFTER: Duration = Duration::from_secs(2 * 5 * 60);

/// Antigravity CLI's folder in the home folder, which holds its settings, as Antigravity's docs name
/// it ("Status line customization", antigravity.google/docs/cli/statusline, read on 2026-10-25:
/// "Add a statusLine configuration block to your ~/.gemini/antigravity-cli/settings.json file").
/// The builder's reading until the real run on #51 confirms it, so it is this one line to correct.
pub(crate) const FOLDER: &str = ".gemini/antigravity-cli";

/// The account's id when the handover names no address, and before any handover: Antigravity CLI
/// is signed in to one account at a time.
const NO_ADDRESS: &str = "antigravity-cli";

/// Where the time is read, which tests set by hand.
type Clock = Arc<dyn Fn() -> DateTime<Utc> + Send + Sync>;

/// Antigravity's limits, from what its CLI's status line hands over.
pub struct AntigravityProvider {
    /// QuotaDial's file of the newest handover, in its data folder.
    file: Option<PathBuf>,
    /// Antigravity CLI's folder, only ever looked for, never opened.
    folder: Option<PathBuf>,
    clock: Clock,
}

impl AntigravityProvider {
    /// The provider keeping Antigravity CLI's numbers in `data_folder`, QuotaDial's data folder,
    /// and looking for Antigravity CLI's folder in this process's home folder. Nothing is read
    /// until it is asked.
    #[must_use]
    pub fn new(data_folder: Option<PathBuf>) -> Self {
        Self::at(data_folder, std::env::home_dir(), Arc::new(Utc::now))
    }

    fn at(data_folder: Option<PathBuf>, home: Option<PathBuf>, clock: Clock) -> Self {
        Self {
            file: data_folder.map(|folder| folder.join(store::FILE)),
            folder: home.map(|home| home.join(FOLDER)),
            clock,
        }
    }

    /// The newest handover and when it was handed over, if QuotaDial's file holds one.
    fn handover(&self) -> Option<(Handover, DateTime<Utc>)> {
        store::read(self.file.as_deref()?)
    }
}

impl fmt::Debug for AntigravityProvider {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // No path: the data folder's has the user's name in it (#52).
        formatter
            .debug_struct("AntigravityProvider")
            .finish_non_exhaustive()
    }
}

/// The account `handover` is from: named by its address, which is also its id, as Antigravity has
/// no other, so two people's numbers never blend.
fn account(handover: &Handover) -> Account {
    let (account_id, display_name) = match &handover.email {
        Some(email) => (email.to_lowercase(), email.clone()),
        None => (NO_ADDRESS.to_owned(), String::new()),
    };
    Account {
        provider: ID.to_owned(),
        provider_name: NAME.to_owned(),
        account_id,
        display_name,
        plan: handover.plan_tier.clone(),
        org: None,
        // QuotaDial never looks for Antigravity's sign-in, so it does not know where it is.
        sign_in: None,
        sign_in_use: advice::sign_in_use(),
    }
}

/// The account shown before any handover, so its sign-in panel can say what to add.
fn waiting_account() -> Account {
    Account {
        provider: ID.to_owned(),
        provider_name: NAME.to_owned(),
        account_id: NO_ADDRESS.to_owned(),
        display_name: String::new(),
        plan: None,
        org: None,
        sign_in: None,
        sign_in_use: advice::sign_in_use(),
    }
}

/// What `account`'s `handover`, handed over at `handed_over`, shows at `now`.
fn snapshot(
    mut account: Account,
    handover: &Handover,
    handed_over: DateTime<Utc>,
    now: DateTime<Utc>,
) -> Snapshot {
    let shown = windows::shown(handover, handed_over, now);
    let stale =
        TimeDelta::from_std(STALE_AFTER).is_ok_and(|two_polls| now - handed_over > two_polls);
    let (status, error) = if stale {
        (Status::Stale, Some(advice::stale(handed_over)))
    } else if !shown.windows.is_empty() {
        (Status::Ok, None)
    } else if !shown.amounts.is_empty() {
        (
            Status::Unsupported,
            Some(advice::amounts(&shown.amounts, false)),
        )
    } else {
        (Status::Unsupported, Some(advice::nothing(handed_over)))
    };
    // A limit with only an amount that the line above does not name is named in the sign-in
    // sentence, so none is left out without a word (the manager on #73).
    if !shown.amounts.is_empty() && status != Status::Unsupported {
        account.sign_in_use = format!(
            "{} {}",
            account.sign_in_use,
            advice::amounts(&shown.amounts, !shown.windows.is_empty())
        );
    }
    Snapshot {
        account,
        windows: shown.windows,
        extra_usage: None,
        status,
        error,
    }
}

#[async_trait]
impl Provider for AntigravityProvider {
    fn id(&self) -> &'static str {
        ID
    }

    fn name(&self) -> &'static str {
        NAME
    }

    fn asks(&self) -> Option<&'static str> {
        None
    }

    /// The account the newest handover is from, or, before any, one saying what to add when
    /// Antigravity CLI's folder is there. Nothing in that folder is opened.
    fn discover_accounts(&self) -> Result<Vec<Account>> {
        if let Some((handover, _)) = self.handover() {
            return Ok(vec![account(&handover)]);
        }
        let installed = self.folder.as_deref().is_some_and(std::path::Path::is_dir);
        Ok(installed.then(waiting_account).into_iter().collect())
    }

    /// The newest handover's limits, read from QuotaDial's file: the status line asks no one, and
    /// neither does this.
    async fn fetch(&self, account: &Account) -> Result<Snapshot> {
        if account.provider != ID {
            return Err(Error::InvalidResponse {
                provider: ID,
                reason: format!(
                    "an account of {} is not one of Antigravity's",
                    quotadial_core::redact_secrets(&account.provider)
                ),
            });
        }
        let Some((handover, handed_over)) = self.handover() else {
            return Ok(Snapshot::unsupported(account.clone(), advice::waiting()));
        };
        let handed = self::account(&handover);
        if !handed.same_as(account) {
            return Ok(Snapshot::unsupported(
                account.clone(),
                advice::ANOTHER_ACCOUNT,
            ));
        }
        Ok(snapshot(handed, &handover, handed_over, (self.clock)()))
    }

    fn fallback(&self, _account: &Account) -> Option<Snapshot> {
        // The file is the only place the numbers are; there is nothing to fall back on.
        None
    }

    fn min_poll_interval(&self) -> Duration {
        MIN_POLL_INTERVAL
    }

    /// `quotadial antigravity status-line`, which keeps what Antigravity CLI's status line hands
    /// over in this provider's file.
    fn commands(&self) -> Vec<ProviderCommand> {
        let file = self.file.clone();
        let clock = Arc::clone(&self.clock);
        vec![ProviderCommand {
            name: command::NAME,
            about: command::ABOUT,
            run: Box::new(move |invocation| command::run(file.as_deref(), clock(), invocation)),
        }]
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::process::ExitCode;
    use std::sync::Mutex;

    use quotadial_core::{Invocation, Source, WindowKind, collect};
    use serde_json::{Value, json};

    use super::*;

    fn at(rfc3339: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(rfc3339)
            .expect("a time")
            .with_timezone(&Utc)
    }

    /// A machine made up by the test: a home folder, QuotaDial's data folder, and a clock set by
    /// hand.
    struct Machine {
        temp: tempfile::TempDir,
        now: Arc<Mutex<DateTime<Utc>>>,
    }

    impl Machine {
        fn new() -> Self {
            Self {
                temp: tempfile::tempdir().expect("temp dir"),
                now: Arc::new(Mutex::new(at("2026-01-05T22:30:00Z"))),
            }
        }

        fn home(&self) -> PathBuf {
            self.temp.path().join("home")
        }

        fn data(&self) -> PathBuf {
            self.temp.path().join("data")
        }

        /// Antigravity CLI installed: its folder is there, with its settings and a sign-in file
        /// QuotaDial must never open.
        fn installed(self) -> Self {
            let folder = self.home().join(FOLDER);
            std::fs::create_dir_all(&folder).expect("makes the folder");
            std::fs::write(folder.join("settings.json"), "{}").expect("writes settings");
            std::fs::write(folder.join("jetski-standalone-oauth-token"), SIGN_IN)
                .expect("writes a sign-in");
            self
        }

        fn set_now(&self, rfc3339: &str) {
            *self.now.lock().expect("the clock") = at(rfc3339);
        }

        fn provider(&self) -> AntigravityProvider {
            let now = Arc::clone(&self.now);
            AntigravityProvider::at(
                Some(self.data()),
                Some(self.home()),
                Arc::new(move || *now.lock().expect("the clock")),
            )
        }

        /// Hands `payload` to `quotadial antigravity status-line`, as Antigravity CLI does, at the
        /// machine's time.
        fn hand_over(&self, payload: &Value) -> ExitCode {
            let provider = self.provider();
            let command = provider.commands().pop().expect("a command");
            let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
            let code = (command.run)(Invocation {
                args: &[],
                stdin: &mut payload.to_string().as_bytes(),
                stdout: &mut stdout,
                stderr: &mut stderr,
            });
            assert!(
                stdout.is_empty() && stderr.is_empty(),
                "the command prints nothing"
            );
            code
        }

        /// What one look finds: each account discovered, and its snapshot.
        async fn look(&self) -> Vec<Snapshot> {
            let provider = self.provider();
            let mut found = Vec::new();
            for account in provider.discover_accounts().expect("discovery never fails") {
                found.push(provider.fetch(&account).await.expect("a snapshot"));
            }
            found
        }
    }

    /// What a test puts in Antigravity CLI's sign-in file: made up, in pieces so no scanner takes it
    /// for one. QuotaDial never reads it.
    const SIGN_IN: &str = concat!(
        "{ \"token\": { \"access_token\": \"ya29",
        ".MadeUpNeverRead0123\" } }"
    );

    /// A status line payload from someone@example.com on Pro, with `quota`.
    fn payload(quota: &Value) -> Value {
        json!({
            "cwd": "/home/someone/project",
            "model": { "id": "example-model", "display_name": "Example Model" },
            "quota": quota,
            "plan_tier": "Pro",
            "email": "Someone@Example.com"
        })
    }

    fn shares() -> Value {
        json!({
            "gemini-5h": { "remaining_fraction": 0.75, "reset_time": "2026-01-06T02:00:00Z" },
            "gemini-weekly": { "remaining_fraction": 0.5, "reset_in_seconds": 432_000 },
            "3p-5h": { "remaining_fraction": 1.0, "reset_time": "2026-01-06T01:00:00Z" },
            "3p-weekly": { "remaining_fraction": 0.9, "reset_time": "2026-01-10T00:00:00Z" }
        })
    }

    fn labels(snapshot: &Snapshot) -> Vec<&str> {
        snapshot
            .windows
            .iter()
            .map(|window| window.label.as_str())
            .collect()
    }

    #[test]
    fn its_name_and_every_sentence_it_says_of_a_sign_in_keep_to_plain_words() {
        let names = [windows::Amount {
            label: "Weekly (Quota)".into(),
            left: 1.0,
            resets_at: None,
        }];
        for said in [
            NAME.to_owned(),
            advice::sign_in_use(),
            format!(
                "{} {}",
                advice::sign_in_use(),
                advice::amounts(&names, true)
            ),
            format!(
                "{} {}",
                advice::sign_in_use(),
                advice::amounts(&names, false)
            ),
        ] {
            // #52's plain words: never "quota", "auth" or "credential" outside a name (P-020).
            assert_eq!(
                quotadial_core::text::unplain_words(&said),
                Vec::<String>::new(),
                "{said}"
            );
        }
    }

    #[test]
    fn it_asks_no_one_keeps_no_logs_and_adds_one_command() {
        let machine = Machine::new().installed();
        let provider = machine.provider();
        let account = provider.discover_accounts().expect("discovers")[0].clone();

        assert_eq!(
            (provider.id(), provider.name()),
            ("antigravity", "Antigravity")
        );
        assert_eq!(provider.asks(), None);
        assert_eq!(provider.min_poll_interval(), Duration::from_secs(300));
        assert_eq!(STALE_AFTER, 2 * provider.min_poll_interval());
        assert_eq!(provider.fallback(&account), None);
        assert!(provider.log_folders().is_empty());
        assert_eq!(provider.held_back(), 0);
        assert!(provider.discovery_warnings().is_empty());
        let names: Vec<&str> = provider
            .commands()
            .iter()
            .map(|command| command.name)
            .collect();
        assert_eq!(names, ["status-line"]);
    }

    #[tokio::test]
    async fn with_neither_its_folder_nor_a_handover_there_is_no_account() {
        assert!(Machine::new().look().await.is_empty());
        let machine = Machine::new();
        std::fs::create_dir_all(machine.home().join(".gemini")).expect("makes .gemini");
        assert!(machine.look().await.is_empty(), "Gemini CLI's folder alone");
    }

    #[tokio::test]
    async fn antigravity_cli_s_folder_shows_an_account_that_says_what_to_add_before_any_numbers() {
        let found = Machine::new().installed().look().await;

        assert_eq!(found.len(), 1);
        let snapshot = &found[0];
        assert_eq!(snapshot.account, waiting_account());
        assert_eq!(snapshot.account.provider_name, NAME);
        assert_eq!(snapshot.account.sign_in, None);
        assert_eq!(snapshot.status, Status::Unsupported);
        assert!(snapshot.windows.is_empty());
        assert_eq!(snapshot.error.as_deref(), Some(advice::waiting().as_str()));
        for said in [
            &snapshot.account.sign_in_use,
            snapshot.error.as_ref().expect("a line"),
        ] {
            assert!(said.contains(&advice::block()), "{said}");
        }
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn nothing_in_antigravity_cli_s_folder_is_ever_opened() {
        use std::os::unix::fs::PermissionsExt;

        let machine = Machine::new().installed();
        let folder = machine.home().join(FOLDER);
        std::fs::set_permissions(&folder, std::fs::Permissions::from_mode(0o000))
            .expect("takes away every permission");

        let found = machine.look().await;

        std::fs::set_permissions(&folder, std::fs::Permissions::from_mode(0o700))
            .expect("gives them back");
        assert_eq!(found.len(), 1, "seen without being opened");
        let shown = serde_json::to_string(&found).expect("serializes");
        assert!(!shown.contains("MadeUpNeverRead"), "{shown}");
    }

    #[tokio::test]
    async fn a_handover_shows_its_limits_as_windows_from_the_status_line() {
        let machine = Machine::new().installed();
        machine.set_now("2026-01-05T22:29:00Z");
        assert_eq!(machine.hand_over(&payload(&shares())), ExitCode::SUCCESS);
        machine.set_now("2026-01-05T22:30:00Z");

        let found = machine.look().await;

        assert_eq!(
            found.len(),
            1,
            "the handover's account in the waiting one's place"
        );
        let snapshot = &found[0];
        assert_eq!(
            snapshot.account,
            Account {
                provider: "antigravity".into(),
                provider_name: "Antigravity".into(),
                account_id: "someone@example.com".into(),
                display_name: "Someone@Example.com".into(),
                plan: Some("Pro".into()),
                org: None,
                sign_in: None,
                sign_in_use: advice::sign_in_use(),
            }
        );
        assert_eq!(snapshot.status, Status::Ok);
        assert_eq!(snapshot.error, None);
        assert_eq!(
            labels(snapshot),
            [
                "5-hour (Gemini)",
                "Weekly (Gemini)",
                "5-hour (Claude and GPT)",
                "Weekly (Claude and GPT)"
            ]
        );
        let handed_over = at("2026-01-05T22:29:00Z");
        for window in &snapshot.windows {
            assert_eq!(window.source, Source::StatusLine, "{}", window.label);
            assert_eq!(window.fetched_at, handed_over, "{}", window.label);
        }
        let gemini_weekly = &snapshot.windows[1];
        assert_eq!(gemini_weekly.kind, WindowKind::Weekly);
        assert_eq!(gemini_weekly.used_pct, 50.0);
        assert_eq!(
            gemini_weekly.resets_at,
            Some(at("2026-01-10T22:29:00Z")),
            "reset_in_seconds after the handover"
        );
    }

    #[tokio::test]
    async fn numbers_older_than_two_polls_say_when_they_are_from_and_a_limit_reset_since_goes() {
        let machine = Machine::new().installed();
        machine.hand_over(&payload(&shares()));

        machine.set_now("2026-01-05T22:40:00Z");
        let ten_minutes = machine.look().await;
        assert_eq!(ten_minutes[0].status, Status::Ok, "two polls exactly");

        // Days later: the five-hour limits have reset; the weekly ones have not.
        machine.set_now("2026-01-08T09:00:00Z");
        let days = machine.look().await;

        let snapshot = &days[0];
        assert_eq!(snapshot.status, Status::Stale);
        assert_eq!(
            snapshot.error.as_deref(),
            Some(advice::stale(at("2026-01-05T22:30:00Z")).as_str())
        );
        assert!(
            snapshot
                .error
                .as_deref()
                .is_some_and(|line| line.contains("on 5 Jan 2026 at 22:30 UTC")),
            "the handover's time, in words"
        );
        assert_eq!(
            labels(snapshot),
            ["Weekly (Gemini)", "Weekly (Claude and GPT)"]
        );

        // A week on, nothing is left that counts, and the line still says how old it is.
        machine.set_now("2026-01-12T00:00:00Z");
        let week = machine.look().await;
        assert_eq!(week[0].status, Status::Stale);
        assert!(week[0].windows.is_empty());
    }

    #[tokio::test]
    async fn every_limit_an_amount_shows_unsupported_with_each_named_and_its_amount_and_reset() {
        let machine = Machine::new().installed();
        machine.hand_over(&payload(&json!({
            "gemini-weekly": { "remaining_amount": 120, "reset_time": "2026-01-10T00:00:00Z" },
            "3p-weekly": { "remaining_amount": 3.5 }
        })));

        let found = machine.look().await;

        let snapshot = &found[0];
        assert_eq!(snapshot.status, Status::Unsupported);
        assert!(snapshot.windows.is_empty());
        assert_eq!(
            snapshot.error.as_deref(),
            Some(
                "Antigravity CLI gives an amount left and no share for these limits, so QuotaDial \
                 shows no share for them: \"Weekly (Gemini)\", 120 left, resetting on \
                 10 Jan 2026 at 00:00 UTC; \"Weekly (Claude and GPT)\", 3.5 left."
            )
        );
        assert_eq!(
            snapshot.account.sign_in_use,
            advice::sign_in_use(),
            "named once, in the line"
        );
    }

    #[tokio::test]
    async fn an_amount_beside_shares_is_named_in_the_sign_in_sentence_with_its_amount_and_reset() {
        let machine = Machine::new().installed();
        machine.hand_over(&payload(&json!({
            "gemini-5h": { "remaining_fraction": 0.75, "reset_time": "2026-01-06T02:00:00Z" },
            "3p-weekly": { "remaining_amount": 120, "reset_time": "2026-01-10T00:00:00Z" }
        })));

        let found = machine.look().await;

        let snapshot = &found[0];
        assert_eq!(snapshot.status, Status::Ok);
        assert_eq!(snapshot.error, None);
        assert_eq!(labels(snapshot), ["5-hour (Gemini)"]);
        assert_eq!(
            snapshot.account.sign_in_use,
            format!(
                "{} Antigravity CLI gives an amount left and no share for one more limit, which \
                 QuotaDial leaves out of its windows: \"Weekly (Claude and GPT)\", 120 left, \
                 resetting on 10 Jan 2026 at 00:00 UTC.",
                advice::sign_in_use()
            )
        );
    }

    #[tokio::test]
    async fn every_limit_a_share_names_no_amount() {
        let machine = Machine::new().installed();
        machine.hand_over(&payload(&shares()));

        let found = machine.look().await;

        assert_eq!(found[0].status, Status::Ok);
        assert_eq!(found[0].account.sign_in_use, advice::sign_in_use());
        assert_eq!(found[0].windows.len(), 4);
    }

    #[tokio::test]
    async fn limits_marked_turned_off_are_left_out_and_with_none_left_the_line_says_so() {
        let machine = Machine::new().installed();
        machine.hand_over(&payload(&json!({
            "gemini-5h": { "remaining_fraction": 0.75, "disabled": true },
            "gemini-weekly": { "remaining_fraction": 0.5, "reset_time": "2026-01-10T00:00:00Z" }
        })));
        assert_eq!(labels(&machine.look().await[0]), ["Weekly (Gemini)"]);

        machine.hand_over(&payload(&json!({
            "gemini-5h": { "remaining_fraction": 0.75, "disabled": true }
        })));
        let found = machine.look().await;

        assert_eq!(found[0].status, Status::Unsupported);
        assert!(found[0].windows.is_empty());
        assert_eq!(
            found[0].error.as_deref(),
            Some(advice::nothing(at("2026-01-05T22:30:00Z")).as_str())
        );
    }

    #[tokio::test]
    async fn a_handover_with_no_address_is_one_account_all_the_same() {
        let machine = Machine::new();
        machine.hand_over(&json!({ "quota": shares() }));

        let found = machine.look().await;

        assert_eq!(found.len(), 1, "with no folder, the handover alone");
        assert_eq!(found[0].account.account_id, "antigravity-cli");
        assert_eq!(found[0].account.display_name, "");
        assert_eq!(found[0].account.plan, None);
        assert_eq!(found[0].status, Status::Ok);
    }

    #[tokio::test]
    async fn an_account_whose_numbers_another_account_s_replaced_says_so() {
        let machine = Machine::new().installed();
        machine.hand_over(&payload(&shares()));
        let provider = machine.provider();
        let before = provider.discover_accounts().expect("discovers")[0].clone();
        let mut other = payload(&shares());
        other["email"] = json!("other@example.com");
        machine.hand_over(&other);

        let snapshot = provider.fetch(&before).await.expect("a snapshot");

        assert_eq!(snapshot.status, Status::Unsupported);
        assert_eq!(snapshot.account, before);
        assert_eq!(snapshot.error.as_deref(), Some(advice::ANOTHER_ACCOUNT));
        assert_eq!(
            provider.discover_accounts().expect("discovers")[0].account_id,
            "other@example.com",
            "the next look shows the other"
        );
    }

    #[tokio::test]
    async fn an_account_of_another_provider_is_not_one_of_its() {
        let machine = Machine::new().installed();
        let mut stranger = waiting_account();
        stranger.provider = "gemini".into();

        let error = machine
            .provider()
            .fetch(&stranger)
            .await
            .expect_err("not one of Antigravity's");

        assert_eq!(
            error.to_string(),
            "unexpected response: an account of gemini is not one of Antigravity's"
        );
    }

    #[tokio::test]
    async fn collect_shows_the_account_and_no_error() {
        let machine = Machine::new().installed();
        machine.hand_over(&payload(&shares()));
        let provider: Arc<dyn Provider> = Arc::new(machine.provider());

        let report = collect(&[provider]).await;

        assert!(report.errors.is_empty(), "{:?}", report.errors);
        assert_eq!(report.snapshots.len(), 1);
        assert_eq!(report.snapshots[0].status, Status::Ok);
    }

    #[test]
    fn its_debug_output_names_no_folder() {
        let provider = AntigravityProvider::new(Some(PathBuf::from("/Users/someone/quotadial")));

        assert_eq!(format!("{provider:?}"), "AntigravityProvider { .. }");
    }

    #[test]
    fn its_file_is_in_quotadial_s_data_folder() {
        let provider = AntigravityProvider::new(Some(PathBuf::from("/data/quotadial")));

        assert_eq!(
            provider.file.as_deref(),
            Some(
                Path::new("/data/quotadial")
                    .join("antigravity.json")
                    .as_path()
            )
        );
        assert!(AntigravityProvider::new(None).handover().is_none());
    }
}
