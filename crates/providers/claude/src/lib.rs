//! Claude for QuotaDial: the five-hour and weekly windows of each Claude account, from the numbers
//! Claude Code's status line hands to a command the user adds to Claude Code's settings (D-145).
//!
//! QuotaDial never reads Claude Code's sign-in and asks no one about Claude (D-145, P-021). Claude
//! Code's status line runs a command the user names with the session's state, which carries each
//! window's share used and reset for claude.ai Pro and Max subscribers ("Customize your status
//! line", code.claude.com/docs/en/statusline, read on 28 October 2026), and QuotaDial never edits
//! Claude Code's settings to add itself: `doctor`, the account's sign-in panel and
//! `docs/INSTALL.md` give the line to add.
//!
//! - `quotadial claude status-line` (`command`), through the provider's own commands (P-018):
//!   Claude Code runs it with its session's state on standard input; it keeps only the five-hour and
//!   weekly windows' share used and reset, and its own time (`handover`), in its folder's file in
//!   QuotaDial's data folder (`store`), then runs any status line command named after it, with the
//!   same input, through `quotadial-status-line`, which Antigravity's command shares.
//! - Discovery (`folders`) lists Claude Code's folders as the sign-ins were listed, this shell's
//!   first, and takes each one's account from Claude Code's own account file in it: its id,
//!   address, organisation and plan. A folder whose account file names none but which has a file
//!   of numbers is one account, `claude` for the default folder, or `claude-` and its file's 8 hex
//!   for one `CLAUDE_CONFIG_DIR` names. A subscription seen from two folders is one account (D-024).
//! - `fetch` reads the account's folders' files at each poll, the newest numbers winning, with the
//!   time they were handed over as each window's `fetched_at` and `status_line` as its source
//!   (P-022). Numbers handed over longer ago than two polls are `stale`, with that time in words
//!   (#52), and a window that has reset since is left out (D-126); with none left, the account is
//!   `unsupported`, saying when Claude Code last handed numbers over. Before any numbers, its line
//!   says what to add.
//!
//! The cost scan of Claude Code's session logs, in each folder's `projects`, is unchanged (D-097).

mod advice;
mod command;
mod environment;
mod folders;
mod handover;
mod identity;
mod plan;
mod store;
mod windows;

use std::fmt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, TimeDelta, Utc};
use quotadial_core::{
    Account, AccountLimit, Error, Provider, ProviderCommand, Result, Snapshot, Status,
};

use crate::environment::Environment;
use crate::folders::{Folder, Kept};
use crate::identity::Identity;

/// The provider id used in [`Account::provider`].
pub const ID: &str = "claude";

/// What the provider calls itself (P-020).
pub const NAME: &str = "Claude";

/// How often the daemon may look at a Claude account again: a read of QuotaDial's own files, which
/// asks no one. The floor stays 5 minutes (D-126's, the manager's rule 7 on #73).
pub const MIN_POLL_INTERVAL: Duration = Duration::from_secs(5 * 60);

/// How old the newest numbers may be before they are stale: two polls (#52, D-126).
pub const STALE_AFTER: Duration = Duration::from_secs(2 * 5 * 60);

/// Where the time is read, which tests set by hand.
type Clock = Arc<dyn Fn() -> DateTime<Utc> + Send + Sync>;

/// Claude's limits, from what Claude Code's status line hands over, in every Claude Code folder on
/// this machine, not only the one this shell would use (#34).
pub struct ClaudeProvider {
    env: Environment,
    /// QuotaDial's data folder, where `quotadial claude status-line` keeps each folder's numbers.
    data_folder: Option<PathBuf>,
    clock: Clock,
    /// More Claude Code folders to look in, from QuotaDial's settings.
    extra_dirs: Vec<PathBuf>,
    /// At most this many accounts, this shell's own first, asked at each discovery.
    max_accounts: AccountLimit,
    /// The accounts the last discovery found, and their folders, so a fetch reads only its own.
    found: Mutex<Vec<Found>>,
    /// How many more accounts the last discovery found than `max_accounts` let it show.
    held_back: AtomicUsize,
    /// What the last discovery could not look at.
    warnings: Mutex<Vec<String>>,
}

/// An account and the folders signed in to it, in the order they were found.
#[derive(Debug, Clone)]
struct Found {
    account: Account,
    folders: Vec<Folder>,
}

impl ClaudeProvider {
    /// The provider for this process's environment, keeping what Claude Code's status line hands
    /// over in `data_folder`, QuotaDial's data folder. Nothing is read until it is asked.
    #[must_use]
    pub fn new(data_folder: Option<PathBuf>) -> Self {
        Self::at(Environment::from_process(), data_folder, Arc::new(Utc::now))
    }

    fn at(env: Environment, data_folder: Option<PathBuf>, clock: Clock) -> Self {
        Self {
            env,
            data_folder,
            clock,
            extra_dirs: Vec::new(),
            max_accounts: AccountLimit::none(),
            found: Mutex::default(),
            held_back: AtomicUsize::new(0),
            warnings: Mutex::default(),
        }
    }

    /// Also looks in `dirs`, the `extra_config_dirs` setting. A folder ending in `*` adds every
    /// folder inside it.
    #[must_use]
    pub fn with_extra_config_dirs(mut self, dirs: Vec<PathBuf>) -> Self {
        self.extra_dirs = dirs;
        self
    }

    /// Shows at most as many accounts as `limit` says at each discovery, this shell's own first:
    /// the `max_accounts` setting, and one on QuotaDial Free.
    #[must_use]
    pub fn with_max_accounts(mut self, limit: AccountLimit) -> Self {
        self.max_accounts = limit;
        self
    }

    /// The file `quotadial claude status-line` keeps its numbers in, for the Claude Code folder this
    /// process runs for: the one `CLAUDE_CONFIG_DIR` names, inherited from Claude Code, or its
    /// default folder.
    fn command_file(&self) -> Option<PathBuf> {
        let data_folder = self.data_folder.as_ref()?;
        Some(data_folder.join(store::file_name(self.env.config_dir.as_deref())))
    }

    /// Every account a Claude Code folder is signed in to, one per account, in the folders' order,
    /// and the `extra_config_dirs` entries left out.
    fn find(&self) -> (Vec<Found>, Vec<PathBuf>) {
        let listed = folders::list(&self.env, &self.extra_dirs);
        let mut found: Vec<Found> = Vec::new();
        for folder in listed.folders {
            let Some(account) = self.account_in(&folder) else {
                continue;
            };
            match found
                .iter_mut()
                .find(|known| known.account.same_as(&account))
            {
                Some(known) => known.folders.push(folder),
                None => found.push(Found {
                    account,
                    folders: vec![folder],
                }),
            }
        }
        (found, listed.skipped)
    }

    /// The account `folder` is signed in to, by Claude Code's account file in it, or, with none
    /// named there, the one its file of numbers stands for; `None` with neither.
    fn account_in(&self, folder: &Folder) -> Option<Account> {
        if let Some(identity) = folder.identity(&self.env) {
            return Some(self.named(&identity, folder));
        }
        let kept = folder.handover(self.data_folder.as_deref()?)?;
        Some(self.unnamed(folder, &kept.file))
    }

    /// The account `identity` names, found in `folder`.
    fn named(&self, identity: &Identity, folder: &Folder) -> Account {
        Account {
            provider: ID.to_owned(),
            provider_name: NAME.to_owned(),
            account_id: identity.account_uuid.clone(),
            display_name: identity
                .email
                .clone()
                .unwrap_or_else(|| identity.account_uuid.clone()),
            plan: identity.plan.clone(),
            org: identity.org_uuid.clone(),
            // QuotaDial never looks for Claude Code's sign-in, so it does not know where it is.
            sign_in: None,
            sign_in_use: advice::sign_in_use(folder, &folder.file_name(), &self.env),
        }
    }

    /// The account a folder whose account file names none stands for, by its `file` of numbers:
    /// `claude` for the default folder, as Antigravity's is `antigravity-cli`, and `claude-` and the
    /// file's 8 hex for another, named by the folder, so two never blend.
    fn unnamed(&self, folder: &Folder, file: &str) -> Account {
        let display_name = match folder {
            Folder::Default => String::new(),
            Folder::Named(dir) => self.env.tilde(std::path::Path::new(dir)),
        };
        Account {
            provider: ID.to_owned(),
            provider_name: NAME.to_owned(),
            account_id: file.strip_suffix(".json").unwrap_or(file).to_owned(),
            display_name,
            plan: None,
            org: None,
            sign_in: None,
            sign_in_use: advice::sign_in_use(folder, file, &self.env),
        }
    }

    /// The folders the last discovery found `account` in, or, when this process has not looked
    /// yet, the folders a look of its own finds it in.
    fn folders_of(&self, account: &Account) -> Vec<Folder> {
        let known = self
            .found
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .iter()
            .find(|found| found.account.same_as(account))
            .map(|found| found.folders.clone());
        known.unwrap_or_else(|| {
            self.find()
                .0
                .into_iter()
                .find(|found| found.account.same_as(account))
                .map(|found| found.folders)
                .unwrap_or_default()
        })
    }
}

impl fmt::Debug for ClaudeProvider {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // No path: the data folder's and the home folder's have the user's name in them (#52).
        formatter
            .debug_struct("ClaudeProvider")
            .finish_non_exhaustive()
    }
}

/// What `account`'s newest numbers, `kept`, show at `now`.
fn snapshot(account: Account, kept: &Kept, now: DateTime<Utc>) -> Snapshot {
    let windows = windows::shown(&kept.handover, kept.handed_over, now);
    if windows.is_empty() {
        return Snapshot::unsupported(account, advice::nothing(kept.handed_over));
    }
    let stale =
        TimeDelta::from_std(STALE_AFTER).is_ok_and(|two_polls| now - kept.handed_over > two_polls);
    let (status, error) = if stale {
        (Status::Stale, Some(advice::stale(kept.handed_over)))
    } else {
        (Status::Ok, None)
    };
    Snapshot {
        account,
        windows,
        extra_usage: None,
        status,
        error,
    }
}

#[async_trait]
impl Provider for ClaudeProvider {
    fn id(&self) -> &'static str {
        ID
    }

    fn name(&self) -> &'static str {
        NAME
    }

    fn asks(&self) -> Option<&'static str> {
        None
    }

    /// The account each Claude Code folder is signed in to, by Claude Code's own account file in
    /// it, this shell's first, and the account a folder's file of numbers stands for where none is
    /// named. Nothing of Claude Code's sign-in is read.
    fn discover_accounts(&self) -> Result<Vec<Account>> {
        let (mut found, skipped) = self.find();
        let held_back = match self.max_accounts.get() {
            Some(max) if found.len() > max => {
                let held_back = found.len() - max;
                found.truncate(max);
                held_back
            }
            _ => 0,
        };
        self.held_back.store(held_back, Ordering::Relaxed);
        *self.warnings.lock().unwrap_or_else(PoisonError::into_inner) =
            skipped.iter().map(|dir| advice::skipped_dir(dir)).collect();
        let accounts = found.iter().map(|found| found.account.clone()).collect();
        *self.found.lock().unwrap_or_else(PoisonError::into_inner) = found;
        Ok(accounts)
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

    /// The newest numbers in the account's folders' files: the status line asks no one, and
    /// neither does this.
    async fn fetch(&self, account: &Account) -> Result<Snapshot> {
        if account.provider != ID {
            return Err(Error::InvalidResponse {
                provider: ID,
                reason: format!(
                    "an account of {} is not one of Claude's",
                    quotadial_core::redact_secrets(&account.provider)
                ),
            });
        }
        let folders = self.folders_of(account);
        let Some(first) = folders.first() else {
            return Ok(Snapshot::unsupported(account.clone(), advice::NOT_FOUND));
        };
        // Each folder still signed in to it, by its account file now.
        let current: Vec<&Folder> = folders
            .iter()
            .filter(|folder| {
                self.account_in(folder)
                    .is_some_and(|now| now.same_as(account))
            })
            .collect();
        let Some(own) = current.first() else {
            return Ok(Snapshot::unsupported(
                account.clone(),
                advice::another_account(first, &self.env),
            ));
        };
        let newest = self.data_folder.as_deref().and_then(|data_folder| {
            current
                .iter()
                .filter_map(|folder| Some((*folder, folder.handover(data_folder)?)))
                .max_by_key(|(_, kept)| kept.handed_over)
        });
        let Some((folder, kept)) = newest else {
            let account = Account {
                sign_in_use: advice::sign_in_use(own, &own.file_name(), &self.env),
                ..account.clone()
            };
            return Ok(Snapshot::unsupported(
                account,
                advice::waiting(own, &self.env),
            ));
        };
        // Named where its numbers were read, which may be another way of writing its folder.
        let account = Account {
            sign_in_use: advice::sign_in_use(folder, &kept.file, &self.env),
            ..account.clone()
        };
        Ok(snapshot(account, &kept, (self.clock)()))
    }

    fn fallback(&self, _account: &Account) -> Option<Snapshot> {
        // The files are the only place the numbers are; there is nothing to fall back on.
        None
    }

    fn min_poll_interval(&self) -> Duration {
        MIN_POLL_INTERVAL
    }

    /// `projects` in each Claude Code folder: this shell's, the default and every candidate, as
    /// Claude Code writes a session's log in the folder it runs in.
    fn log_folders(&self) -> Vec<PathBuf> {
        let mut folders: Vec<PathBuf> = Vec::new();
        let candidates = folders::candidate_dirs(&self.env, &self.extra_dirs).dirs;
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

    /// `quotadial claude status-line`, which keeps what Claude Code's status line hands over in this
    /// folder's file (D-145).
    fn commands(&self) -> Vec<ProviderCommand> {
        let file = self.command_file();
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

    use quotadial_core::{Invocation, Source, WindowKind, collect};
    use serde_json::{Value, json};

    use super::*;

    const DOCUMENTED: &str = include_str!("../../../../fixtures/claude/status-line.synthetic.json");

    /// Made up, in pieces so no scanner takes it for one: what a test puts where Claude Code keeps
    /// its sign-in, which QuotaDial never opens.
    const SIGN_IN: &str = concat!(
        "{ \"claudeAiOauth\": { \"accessToken\": \"sk-ant-oat01",
        "-MadeUpNeverRead0123\" } }"
    );

    const WORK: &str = "00000000-0000-0000-0000-00000000000b";
    const PERSONAL: &str = "00000000-0000-0000-0000-000000000001";

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
        extra: Vec<PathBuf>,
    }

    impl Machine {
        fn new() -> Self {
            let machine = Self {
                temp: tempfile::tempdir().expect("temp dir"),
                now: Arc::new(Mutex::new(at("2026-01-05T22:30:00Z"))),
                extra: Vec::new(),
            };
            std::fs::create_dir_all(machine.home()).expect("makes the home folder");
            machine
        }

        fn home(&self) -> PathBuf {
            self.temp.path().join("home")
        }

        fn data(&self) -> PathBuf {
            self.temp.path().join("data")
        }

        /// A folder `CLAUDE_CONFIG_DIR` can name, in the home folder.
        fn named(&self, name: &str) -> String {
            self.home().join(name).display().to_string()
        }

        /// Claude Code signed in, in `folder`, to the account `uuid`: its account file, and a
        /// sign-in QuotaDial must never open.
        fn signed_in(self, folder: &Folder, uuid: &str, email: &str) -> Self {
            let (dir, account_file) = match folder {
                Folder::Default => (
                    self.home().join(".claude"),
                    self.home().join(".claude.json"),
                ),
                Folder::Named(dir) => (PathBuf::from(dir), Path::new(dir).join(".claude.json")),
            };
            std::fs::create_dir_all(&dir).expect("makes the folder");
            std::fs::write(dir.join(".credentials.json"), SIGN_IN).expect("writes a sign-in");
            std::fs::write(
                account_file,
                json!({
                    "numStartups": 3,
                    "oauthAccount": {
                        "accountUuid": uuid,
                        "emailAddress": email,
                        "organizationUuid": "00000000-0000-0000-0000-000000000002",
                        "organizationType": "claude_max"
                    }
                })
                .to_string(),
            )
            .expect("writes the account file");
            self
        }

        fn set_now(&self, rfc3339: &str) {
            *self.now.lock().expect("the clock") = at(rfc3339);
        }

        /// The provider as a process with `config_dir` as its `CLAUDE_CONFIG_DIR` makes it.
        fn provider_in(&self, config_dir: Option<&str>) -> ClaudeProvider {
            let now = Arc::clone(&self.now);
            ClaudeProvider::at(
                Environment {
                    home: Some(self.home()),
                    config_dir: config_dir.map(str::to_owned),
                },
                Some(self.data()),
                Arc::new(move || *now.lock().expect("the clock")),
            )
            .with_extra_config_dirs(self.extra.clone())
        }

        fn provider(&self) -> ClaudeProvider {
            self.provider_in(None)
        }

        /// Hands `payload` to `quotadial claude status-line`, as Claude Code in a session with
        /// `config_dir` as its `CLAUDE_CONFIG_DIR` does, at the machine's time.
        fn hand_over(&self, config_dir: Option<&str>, payload: &Value) -> ExitCode {
            let command = self
                .provider_in(config_dir)
                .commands()
                .pop()
                .expect("a command");
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

    /// What Claude Code's status line hands over, with each window's share used and reset.
    fn payload(five_hour: Option<(f64, &str)>, seven_day: Option<(f64, &str)>) -> Value {
        let window = |(used, resets): (f64, &str)| json!({ "used_percentage": used, "resets_at": at(resets).timestamp() });
        let mut rate_limits = serde_json::Map::new();
        if let Some(five_hour) = five_hour {
            rate_limits.insert("five_hour".into(), window(five_hour));
        }
        if let Some(seven_day) = seven_day {
            rate_limits.insert("seven_day".into(), window(seven_day));
        }
        json!({
            "cwd": "/home/someone/project",
            "model": { "id": "example-model", "display_name": "Example" },
            "cost": { "total_cost_usd": 0.42 },
            "rate_limits": rate_limits
        })
    }

    fn both() -> Value {
        payload(
            Some((23.5, "2026-01-06T02:00:00Z")),
            Some((41.2, "2026-01-10T00:00:00Z")),
        )
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
        let machine = Machine::new();
        let env = machine.provider().env;
        for said in [
            NAME.to_owned(),
            advice::sign_in_use(&Folder::Default, "claude.json", &env),
            advice::waiting(&Folder::Default, &env),
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
    fn it_asks_no_one_keeps_its_logs_where_they_were_and_adds_one_command() {
        let machine = Machine::new().signed_in(&Folder::Default, PERSONAL, "someone@example.com");
        let work = machine.named(".claude-work");
        let machine = machine.signed_in(&Folder::Named(work.clone()), WORK, "work@example.com");
        let provider = machine.provider();
        let account = provider.discover_accounts().expect("discovers")[0].clone();

        assert_eq!((provider.id(), provider.name()), ("claude", "Claude"));
        assert_eq!(provider.asks(), None);
        assert_eq!(provider.min_poll_interval(), Duration::from_secs(300));
        assert_eq!(STALE_AFTER, 2 * provider.min_poll_interval());
        assert_eq!(provider.fallback(&account), None);
        assert_eq!(provider.held_back(), 0);
        assert!(provider.discovery_warnings().is_empty());
        // The cost scan reads the same folders as before (D-097).
        assert_eq!(
            provider.log_folders(),
            [
                machine.home().join(".claude").join("projects"),
                PathBuf::from(&work).join("projects")
            ]
        );
        let names: Vec<&str> = provider
            .commands()
            .iter()
            .map(|command| command.name)
            .collect();
        assert_eq!(names, ["status-line"]);
    }

    #[tokio::test]
    async fn with_no_claude_code_account_there_is_no_account() {
        assert!(Machine::new().look().await.is_empty());
        let machine = Machine::new();
        std::fs::create_dir_all(machine.home().join(".claude")).expect("makes the folder");
        std::fs::write(
            machine.home().join(".claude.json"),
            r#"{ "numStartups": 1 }"#,
        )
        .expect("writes an account file with no account in it");
        assert!(
            machine.look().await.is_empty(),
            "a folder signed in with an API key, or not at all"
        );
    }

    #[tokio::test]
    async fn a_signed_in_folder_shows_its_account_and_the_line_to_add_before_any_numbers() {
        let machine = Machine::new().signed_in(&Folder::Default, PERSONAL, "someone@example.com");

        let found = machine.look().await;

        assert_eq!(found.len(), 1);
        let snapshot = &found[0];
        let env = machine.provider().env;
        assert_eq!(
            snapshot.account,
            Account {
                provider: "claude".into(),
                provider_name: "Claude".into(),
                account_id: PERSONAL.into(),
                display_name: "someone@example.com".into(),
                plan: Some("max".into()),
                org: Some("00000000-0000-0000-0000-000000000002".into()),
                sign_in: None,
                sign_in_use: advice::sign_in_use(&Folder::Default, "claude.json", &env),
            }
        );
        assert_eq!(snapshot.status, Status::Unsupported);
        assert!(snapshot.windows.is_empty());
        assert_eq!(snapshot.extra_usage, None);
        assert_eq!(
            snapshot.error.as_deref(),
            Some(advice::waiting(&Folder::Default, &env).as_str())
        );
        for said in [
            &snapshot.account.sign_in_use,
            snapshot.error.as_ref().expect("a line"),
        ] {
            assert!(said.contains(&advice::block()), "{said}");
        }
    }

    #[tokio::test]
    async fn a_handover_shows_the_two_windows_from_the_status_line_as_they_were_before() {
        let machine = Machine::new().signed_in(&Folder::Default, PERSONAL, "someone@example.com");
        machine.set_now("2026-01-05T22:29:00Z");
        assert_eq!(machine.hand_over(None, &both()), ExitCode::SUCCESS);
        machine.set_now("2026-01-05T22:30:00Z");

        let found = machine.look().await;

        let snapshot = &found[0];
        assert_eq!(snapshot.account.account_id, PERSONAL);
        assert_eq!(snapshot.status, Status::Ok);
        assert_eq!(snapshot.error, None);
        assert_eq!(snapshot.extra_usage, None, "the status line carries none");
        assert_eq!(labels(snapshot), ["Session", "Weekly"]);
        let handed_over = at("2026-01-05T22:29:00Z");
        for window in &snapshot.windows {
            assert_eq!(window.source, Source::StatusLine, "{}", window.label);
            assert_eq!(window.fetched_at, handed_over, "{}", window.label);
            assert_eq!(window.scope_model, None, "{}", window.label);
        }
        let (session, weekly) = (&snapshot.windows[0], &snapshot.windows[1]);
        assert_eq!(
            (session.kind, session.window_seconds, session.used_pct),
            (WindowKind::Session, 18_000, 23.5)
        );
        assert_eq!(session.resets_at, Some(at("2026-01-06T02:00:00Z")));
        assert_eq!(
            (weekly.kind, weekly.window_seconds, weekly.used_pct),
            (WindowKind::Weekly, 604_800, 41.2)
        );
    }

    #[tokio::test]
    async fn numbers_older_than_two_polls_say_when_they_are_from_and_with_none_left_say_so() {
        let machine = Machine::new().signed_in(&Folder::Default, PERSONAL, "someone@example.com");
        machine.hand_over(None, &both());

        machine.set_now("2026-01-05T22:40:00Z");
        assert_eq!(
            machine.look().await[0].status,
            Status::Ok,
            "two polls exactly"
        );

        // Hours later: the five-hour window has reset; the weekly one has not.
        machine.set_now("2026-01-06T09:00:00Z");
        let hours = machine.look().await;
        let snapshot = &hours[0];
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
        assert_eq!(labels(snapshot), ["Weekly"]);

        // Once the weekly window has reset too, nothing is left: D-126's rule, and the manager's
        // rule 3, with when Claude Code last handed numbers over.
        machine.set_now("2026-01-12T00:00:00Z");
        let week = machine.look().await;
        assert_eq!(week[0].status, Status::Unsupported);
        assert!(week[0].windows.is_empty());
        assert_eq!(
            week[0].error.as_deref(),
            Some(advice::nothing(at("2026-01-05T22:30:00Z")).as_str())
        );
    }

    #[tokio::test]
    async fn a_handover_with_no_rate_limits_leaves_the_numbers_as_they_were() {
        let machine = Machine::new().signed_in(&Folder::Default, PERSONAL, "someone@example.com");
        machine.hand_over(None, &both());
        machine.set_now("2026-01-05T22:35:00Z");

        // As a session's first run, before its first answer, hands it over.
        machine.hand_over(None, &payload(None, None));

        let snapshot = &machine.look().await[0];
        assert_eq!(snapshot.status, Status::Ok);
        assert_eq!(snapshot.windows[0].fetched_at, at("2026-01-05T22:30:00Z"));
    }

    #[tokio::test]
    async fn each_folder_s_account_shows_its_own_file_s_numbers_this_shell_s_folder_first() {
        let machine = Machine::new().signed_in(&Folder::Default, PERSONAL, "someone@example.com");
        let work = machine.named(".claude-work");
        let machine = machine.signed_in(&Folder::Named(work.clone()), WORK, "work@example.com");
        machine.hand_over(None, &both());
        machine.hand_over(
            Some(&work),
            &payload(Some((90.0, "2026-01-06T01:00:00Z")), None),
        );

        let plain = machine.look().await;
        let provider = machine.provider_in(Some(&work));
        let own_first = provider.discover_accounts().expect("discovers");

        let accounts: Vec<(&str, f64)> = plain
            .iter()
            .map(|snapshot| {
                (
                    snapshot.account.account_id.as_str(),
                    snapshot.windows[0].used_pct,
                )
            })
            .collect();
        assert_eq!(accounts, [(PERSONAL, 23.5), (WORK, 90.0)]);
        assert_eq!(
            own_first[0].account_id, WORK,
            "this shell's own first (D-023)"
        );
        let file = store::file_name(Some(&work));
        let said = &plain[1].account.sign_in_use;
        assert!(said.contains(&format!(" in {file} in ")), "{said}");
        let settings = provider.env.tilde(&Path::new(&work).join("settings.json"));
        assert!(said.contains(&format!("is in {settings}")), "{said}");
    }

    #[tokio::test]
    async fn two_folders_signed_in_to_one_account_are_one_account_with_the_newest_numbers() {
        let machine = Machine::new().signed_in(&Folder::Default, PERSONAL, "someone@example.com");
        let other = machine.named(".claude-other");
        let machine = machine.signed_in(
            &Folder::Named(other.clone()),
            PERSONAL,
            "someone@example.com",
        );
        machine.hand_over(None, &both());
        machine.set_now("2026-01-05T22:35:00Z");
        machine.hand_over(
            Some(&other),
            &payload(Some((30.0, "2026-01-06T02:00:00Z")), None),
        );

        let found = machine.look().await;

        assert_eq!(found.len(), 1, "one subscription (D-024)");
        assert_eq!(found[0].windows[0].used_pct, 30.0, "the newer handover");
        assert_eq!(found[0].windows[0].fetched_at, at("2026-01-05T22:35:00Z"));
    }

    #[tokio::test]
    async fn a_file_whose_folder_names_no_account_is_one_account_named_claude() {
        let machine = Machine::new();
        let work = machine.named(".claude-work");
        std::fs::create_dir_all(&work).expect("makes the folder");
        machine.hand_over(None, &both());
        machine.hand_over(Some(&work), &both());

        let found = machine.look().await;

        let accounts: Vec<(&str, &str)> = found
            .iter()
            .map(|snapshot| {
                (
                    snapshot.account.account_id.as_str(),
                    snapshot.account.display_name.as_str(),
                )
            })
            .collect();
        let named = store::file_name(Some(&work));
        let folder = machine.provider().env.tilde(Path::new(&work));
        assert_eq!(
            accounts,
            [
                ("claude", ""),
                (
                    named.strip_suffix(".json").expect("a file"),
                    folder.as_str()
                )
            ]
        );
        assert!(found.iter().all(|snapshot| snapshot.status == Status::Ok));
    }

    #[tokio::test]
    async fn an_account_whose_folder_is_now_signed_in_to_another_says_so() {
        let machine = Machine::new().signed_in(&Folder::Default, PERSONAL, "someone@example.com");
        machine.hand_over(None, &both());
        let provider = machine.provider();
        let before = provider.discover_accounts().expect("discovers")[0].clone();
        let _machine = machine.signed_in(&Folder::Default, WORK, "work@example.com");

        let snapshot = provider.fetch(&before).await.expect("a snapshot");

        assert_eq!(snapshot.status, Status::Unsupported);
        assert_eq!(snapshot.account, before);
        assert_eq!(
            snapshot.error.as_deref(),
            Some(advice::another_account(&Folder::Default, &provider.env).as_str())
        );
        assert_eq!(
            provider.discover_accounts().expect("discovers")[0].account_id,
            WORK,
            "the next look shows the other"
        );
    }

    #[tokio::test]
    async fn an_account_no_folder_names_any_more_says_so() {
        let machine = Machine::new().signed_in(&Folder::Default, PERSONAL, "someone@example.com");
        let mut gone = machine.provider().discover_accounts().expect("discovers")[0].clone();
        gone.account_id = "00000000-0000-0000-0000-0000000000ff".into();

        let snapshot = machine.provider().fetch(&gone).await.expect("a snapshot");

        assert_eq!(snapshot.status, Status::Unsupported);
        assert_eq!(snapshot.error.as_deref(), Some(advice::NOT_FOUND));
    }

    #[test]
    fn at_most_the_limit_s_accounts_show_this_shell_s_own_first() {
        let machine = Machine::new().signed_in(&Folder::Default, PERSONAL, "someone@example.com");
        let work = machine.named(".claude-work");
        let machine = machine.signed_in(&Folder::Named(work.clone()), WORK, "work@example.com");
        let provider = machine
            .provider_in(Some(&work))
            .with_max_accounts(AccountLimit::fixed(Some(1)));

        let accounts = provider.discover_accounts().expect("discovers");

        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].account_id, WORK);
        assert_eq!(provider.held_back(), 1);
    }

    #[test]
    fn an_extra_folder_that_is_not_a_full_path_is_left_out_with_a_warning() {
        let mut machine = Machine::new();
        machine.extra = vec![PathBuf::from("claude-work")];
        let provider = machine.provider();

        provider.discover_accounts().expect("discovers");

        assert_eq!(
            provider.discovery_warnings(),
            [advice::skipped_dir(Path::new("claude-work"))]
        );
    }

    #[cfg(unix)]
    #[test]
    fn claude_code_s_sign_in_is_never_opened() {
        // A pipe where Claude Code keeps its sign-in: opening it would wait for a writer forever.
        let machine = Machine::new().signed_in(&Folder::Default, PERSONAL, "someone@example.com");
        let work = machine.named(".claude-work");
        let machine = machine.signed_in(&Folder::Named(work.clone()), WORK, "work@example.com");
        machine.hand_over(None, &both());
        for folder in [machine.home().join(".claude"), PathBuf::from(&work)] {
            let sign_in = folder.join(".credentials.json");
            std::fs::rename(&sign_in, folder.join("moved")).expect("moves the sign-in aside");
            let made = std::process::Command::new("mkfifo")
                .arg(&sign_in)
                .status()
                .expect("runs mkfifo");
            assert!(made.success(), "mkfifo {made}");
        }
        let (sent, looked) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .build()
                .expect("a runtime");
            let found = runtime.block_on(machine.look());
            let _ = sent.send(serde_json::to_string(&found).expect("serializes"));
        });

        let shown = looked
            .recv_timeout(std::time::Duration::from_secs(10))
            .expect("looking waited on a pipe where the sign-in is");

        assert!(shown.contains(PERSONAL) && shown.contains(WORK), "{shown}");
        assert!(!shown.contains("MadeUpNeverRead"), "{shown}");
    }

    #[tokio::test]
    async fn an_account_of_another_provider_is_not_one_of_its() {
        let machine = Machine::new().signed_in(&Folder::Default, PERSONAL, "someone@example.com");
        let mut stranger = machine.provider().discover_accounts().expect("discovers")[0].clone();
        stranger.provider = "codex".into();

        let error = machine
            .provider()
            .fetch(&stranger)
            .await
            .expect_err("not one of Claude's");

        assert_eq!(
            error.to_string(),
            "unexpected response: an account of codex is not one of Claude's"
        );
    }

    #[tokio::test]
    async fn collect_shows_the_account_and_no_error() {
        let machine = Machine::new().signed_in(&Folder::Default, PERSONAL, "someone@example.com");
        machine.hand_over(None, &both());
        let provider: Arc<dyn Provider> = Arc::new(machine.provider());

        let report = collect(&[provider]).await;

        assert!(report.errors.is_empty(), "{:?}", report.errors);
        assert_eq!(report.snapshots.len(), 1);
        assert_eq!(report.snapshots[0].status, Status::Ok);
    }

    #[test]
    fn its_debug_output_names_no_folder() {
        let provider = ClaudeProvider::new(Some(PathBuf::from("/Users/someone/quotadial")));

        assert_eq!(format!("{provider:?}"), "ClaudeProvider { .. }");
    }

    #[test]
    fn its_command_keeps_each_claude_code_folder_s_numbers_in_a_file_of_its_own() {
        let machine = Machine::new();
        let documented: Value = serde_json::from_str(DOCUMENTED).expect("JSON");
        machine.set_now("2025-02-01T12:00:00Z");

        assert_eq!(machine.hand_over(None, &documented), ExitCode::SUCCESS);
        assert_eq!(
            machine.hand_over(Some("/Users/example/.claude-work"), &documented),
            ExitCode::SUCCESS
        );

        let mut files: Vec<String> = std::fs::read_dir(machine.data())
            .expect("the data folder is made")
            .map(|entry| {
                entry
                    .expect("an entry")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        files.sort();
        assert_eq!(files, ["claude-dd1118a7.json", "claude.json"]);
        for file in files {
            let (handover, _) = store::read(&machine.data().join(&file)).expect("a handover");
            assert_eq!(handover.windows.len(), 2, "{file}");
        }
    }

    #[test]
    fn with_no_data_folder_its_command_keeps_nothing_and_still_prints_nothing() {
        let machine = Machine::new();
        let mut provider = machine.provider();
        provider.data_folder = None;
        let command = provider.commands().pop().expect("a command");
        let (mut stdout, mut stderr) = (Vec::new(), Vec::new());

        let code = (command.run)(Invocation {
            args: &[],
            stdin: &mut DOCUMENTED.as_bytes(),
            stdout: &mut stdout,
            stderr: &mut stderr,
        });

        assert_eq!(code, ExitCode::SUCCESS);
        assert!(stdout.is_empty() && stderr.is_empty());
        assert!(!machine.data().exists());
    }
}
