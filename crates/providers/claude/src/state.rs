//! What QuotaDial remembers about each Claude account between runs, so the 5-minute floor and the
//! 429 backoff hold across separate `quotadial` processes.
//!
//! Each account has a JSON file in the user's cache directory (`quotadial/claude/` under
//! `~/Library/Caches`, `$XDG_CACHE_HOME` or `%LOCALAPPDATA%`), holding derived data only:
//! snapshots, times and counters, never a token. A lock file beside it keeps two processes from
//! both deciding to send a request. M2's SQLite store replaces this.

use std::fs::{self, File, OpenOptions, TryLockError};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use quotadial_core::{Account, Snapshot};
use serde::{Deserialize, Serialize};

use crate::environment::sha256_hex;

const FORMAT_VERSION: u32 = 1;

/// How long a process waits for another to finish with an account before giving up. The holder
/// needs at most a Keychain read (30 s), `claude --version` (5 s) and two requests (30 s each).
pub(crate) const LOCK_WAIT: Duration = Duration::from_secs(120);

/// One account's saved state.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct AccountState {
    /// The latest snapshot with numbers, shown again, marked as cache, while Anthropic is
    /// rate-limiting or failing.
    pub(crate) last_good: Option<Snapshot>,
    /// What the latest request produced, replayed until the floor allows the next one.
    pub(crate) last_result: Option<LastResult>,
    /// When QuotaDial last sent a usage request for this account.
    pub(crate) requested_at: Option<DateTime<Utc>>,
    /// The sign-in generation that request used (see `Login::generation`).
    pub(crate) login_generation: Option<String>,
    /// No request is sent before this time, after a 429.
    pub(crate) blocked_until: Option<DateTime<Utc>>,
    /// 429 answers in a row, for the exponential backoff.
    pub(crate) rate_limit_strikes: u32,
    /// The latest request went out inside the floor, as the one retry allowed after a rejected
    /// sign-in. Another such retry waits for the floor.
    pub(crate) bypassed_floor: bool,
}

/// The outcome of the latest request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum LastResult {
    /// A snapshot, whatever its status.
    Snapshot { snapshot: Box<Snapshot> },
    /// An answer QuotaDial could not use, such as an HTTP 503.
    Failed { message: String },
}

#[derive(Serialize, Deserialize)]
struct StateFile {
    version: u32,
    state: AccountState,
}

/// The file name for an account's state: a hash of its identity, which keeps the name short and
/// safe on every file system.
pub(crate) fn account_key(account: &Account) -> String {
    let identity = format!(
        "{}\n{}\n{}",
        account.provider,
        account.org.as_deref().unwrap_or_default(),
        account.account_id
    );
    sha256_hex(&identity)[..16].to_owned()
}

/// The directory of saved account state.
#[derive(Debug, Clone)]
pub(crate) struct StateStore {
    dir: PathBuf,
}

/// Holds an account's lock until dropped.
#[derive(Debug)]
pub(crate) struct StateLock {
    _file: File,
}

impl StateStore {
    /// `quotadial/claude` in the user's cache directory.
    pub(crate) fn default_dir() -> Option<PathBuf> {
        dirs::cache_dir().map(|dir| dir.join("quotadial").join("claude"))
    }

    pub(crate) fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    pub(crate) fn dir(&self) -> &Path {
        &self.dir
    }

    /// Takes an account's lock, waiting up to `wait` while another QuotaDial process holds it.
    /// Creating the lock file also proves the directory is writable before any request is sent.
    pub(crate) fn lock(&self, key: &str, wait: Duration) -> io::Result<StateLock> {
        create_private_dir(&self.dir)?;
        let file = private_file(&self.dir.join(format!("{key}.lock")), false)?;
        let deadline = Instant::now() + wait;
        loop {
            match file.try_lock() {
                Ok(()) => return Ok(StateLock { _file: file }),
                Err(TryLockError::WouldBlock) if Instant::now() < deadline => {
                    thread::sleep(Duration::from_millis(50));
                }
                Err(TryLockError::WouldBlock) => {
                    return Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        format!(
                            "another QuotaDial process has been using this account for over {} \
                             seconds",
                            wait.as_secs_f32()
                        ),
                    ));
                }
                Err(TryLockError::Error(error)) => return Err(error),
            }
        }
    }

    /// The account's saved state. A missing, unreadable or outdated file starts afresh.
    pub(crate) fn load(&self, key: &str) -> AccountState {
        fs::read(self.path(key))
            .ok()
            .and_then(|bytes| serde_json::from_slice::<StateFile>(&bytes).ok())
            .filter(|file| file.version == FORMAT_VERSION)
            .map(|file| file.state)
            .unwrap_or_default()
    }

    /// Saves the account's state, replacing the old file in one step so a crash cannot leave half a
    /// file. Call it while holding the account's lock.
    pub(crate) fn save(&self, key: &str, state: &AccountState) -> io::Result<()> {
        let temporary = self.dir.join(format!("{key}.json.tmp"));
        let mut writer = BufWriter::new(private_file(&temporary, true)?);
        let file = StateFile {
            version: FORMAT_VERSION,
            state: state.clone(),
        };
        serde_json::to_writer(&mut writer, &file)?;
        writer.flush()?;
        drop(writer);
        fs::rename(&temporary, self.path(key))
    }

    fn path(&self, key: &str) -> PathBuf {
        self.dir.join(format!("{key}.json"))
    }
}

/// Creates the directory (and its parents) readable only by the user.
fn create_private_dir(dir: &Path) -> io::Result<()> {
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
    builder.create(dir)
}

/// Opens a file for writing, readable only by the user, emptying it if `truncate`.
fn private_file(path: &Path, truncate: bool) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.create(true).write(true).truncate(truncate);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    options.open(path)
}

#[cfg(test)]
mod tests {
    use quotadial_core::Status;

    use super::*;

    fn account(org: Option<&str>) -> Account {
        Account {
            provider: "claude".into(),
            account_id: "00000000-0000-0000-0000-000000000001".into(),
            display_name: "user@example.com".into(),
            plan: Some("max".into()),
            org: org.map(Into::into),
        }
    }

    fn ts(rfc3339: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(rfc3339)
            .expect("valid timestamp")
            .with_timezone(&Utc)
    }

    #[test]
    fn keys_differ_by_organization_and_are_file_name_safe() {
        let personal = account_key(&account(None));
        let work = account_key(&account(Some("00000000-0000-0000-0000-000000000002")));

        assert_ne!(personal, work);
        assert_eq!(personal.len(), 16);
        assert!(personal.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn state_round_trips_through_the_cache_directory() {
        let dir = tempfile::tempdir().expect("temp dir");
        let store = StateStore::new(dir.path().join("quotadial").join("claude"));
        let key = account_key(&account(None));
        let state = AccountState {
            last_good: None,
            last_result: Some(LastResult::Snapshot {
                snapshot: Box::new(Snapshot::auth_required(
                    account(None),
                    "Run `claude` to sign in again.",
                )),
            }),
            requested_at: Some(ts("2099-01-01T00:00:00Z")),
            login_generation: Some("20991231000000Z".into()),
            blocked_until: Some(ts("2099-01-01T00:10:00Z")),
            rate_limit_strikes: 2,
            bypassed_floor: true,
        };

        let _lock = store.lock(&key, LOCK_WAIT).expect("locks");
        store.save(&key, &state).expect("saves");

        assert_eq!(store.load(&key), state);
        assert!(
            !dir.path()
                .join("quotadial/claude")
                .join(format!("{key}.json.tmp"))
                .exists()
        );
    }

    #[test]
    fn a_missing_or_damaged_file_starts_afresh() {
        let dir = tempfile::tempdir().expect("temp dir");
        let store = StateStore::new(dir.path().to_path_buf());

        assert_eq!(store.load("missing"), AccountState::default());

        fs::write(dir.path().join("damaged.json"), "{ not json").expect("writes");
        assert_eq!(store.load("damaged"), AccountState::default());

        fs::write(
            dir.path().join("future.json"),
            r#"{"version":99,"state":{"rate_limit_strikes":3}}"#,
        )
        .expect("writes");
        assert_eq!(store.load("future"), AccountState::default());
    }

    #[test]
    fn the_saved_file_holds_no_token() {
        let dir = tempfile::tempdir().expect("temp dir");
        let store = StateStore::new(dir.path().to_path_buf());
        let snapshot = Snapshot {
            account: account(None),
            windows: Vec::new(),
            extra_usage: None,
            status: Status::Ok,
            error: None,
        };
        let state = AccountState {
            last_good: Some(snapshot.clone()),
            last_result: Some(LastResult::Snapshot {
                snapshot: Box::new(snapshot),
            }),
            ..AccountState::default()
        };

        let _lock = store.lock("account", LOCK_WAIT).expect("locks");
        store.save("account", &state).expect("saves");
        let saved = fs::read_to_string(dir.path().join("account.json")).expect("reads");

        assert!(!saved.contains("sk-ant-"));
        assert!(!saved.to_lowercase().contains("token"));
    }

    #[cfg(unix)]
    #[test]
    fn files_are_private_to_the_user() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().expect("temp dir");
        let store = StateStore::new(dir.path().join("claude"));
        let _lock = store.lock("account", LOCK_WAIT).expect("locks");
        store
            .save("account", &AccountState::default())
            .expect("saves");

        let mode =
            |path: PathBuf| fs::metadata(path).expect("metadata").permissions().mode() & 0o777;
        assert_eq!(mode(dir.path().join("claude")), 0o700);
        assert_eq!(mode(dir.path().join("claude/account.json")), 0o600);
        assert_eq!(mode(dir.path().join("claude/account.lock")), 0o600);
    }

    #[test]
    fn a_lock_held_too_long_is_given_up_on() {
        let dir = tempfile::tempdir().expect("temp dir");
        let store = StateStore::new(dir.path().to_path_buf());
        let _held = store.lock("account", LOCK_WAIT).expect("locks");

        let error = store
            .lock("account", Duration::from_millis(200))
            .expect_err("gives up");

        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
        assert_eq!(
            error.to_string(),
            "another QuotaDial process has been using this account for over 0.2 seconds"
        );
    }

    #[test]
    fn a_second_lock_waits_for_the_first() {
        let dir = tempfile::tempdir().expect("temp dir");
        let store = StateStore::new(dir.path().to_path_buf());
        let first = store.lock("account", LOCK_WAIT).expect("locks");

        let other = store.clone();
        let (sender, receiver) = std::sync::mpsc::channel();
        let waiter = std::thread::spawn(move || {
            let _second = other.lock("account", LOCK_WAIT).expect("locks");
            sender.send(()).expect("sends");
        });

        assert!(
            receiver
                .recv_timeout(std::time::Duration::from_millis(200))
                .is_err(),
            "the second lock must wait while the first is held"
        );
        drop(first);
        receiver
            .recv_timeout(std::time::Duration::from_secs(10))
            .expect("the second lock is granted once the first is released");
        waiter.join().expect("thread finishes");
    }
}
