//! What QuotaDial remembers about each account between runs, so the floor and the 429 backoff
//! hold across separate `quotadial` processes.
//!
//! Each account has a JSON file in the user's cache directory (`quotadial/<provider>/` under
//! `~/Library/Caches`, `$XDG_CACHE_HOME` or `%LOCALAPPDATA%`), holding derived data only:
//! snapshots, times and counters, never a token. A lock file beside it keeps two processes from
//! both deciding to send a request. M2's SQLite store replaces this.

use std::fmt;
use std::fs::{self, File, OpenOptions, TryLockError};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use quotadial_core::{Account, Error, Snapshot};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const FORMAT_VERSION: u32 = 1;

/// How long a process waits for another to finish with an account before giving up. The holder
/// needs at most a credential read (30 s for a Keychain read), a few seconds of setup and two
/// requests (30 s each).
pub const LOCK_WAIT: Duration = Duration::from_secs(120);

/// One account's saved state.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AccountState {
    /// The latest snapshot with numbers, shown again, marked as cache, while Anthropic is
    /// rate-limiting or failing.
    pub last_good: Option<Snapshot>,
    /// What the latest request produced, replayed until the floor allows the next one.
    pub last_result: Option<LastResult>,
    /// When QuotaDial last sent a usage request for this account.
    pub requested_at: Option<DateTime<Utc>>,
    /// The sign-in generation that request used (see `Login::generation`).
    pub login_generation: Option<String>,
    /// No request is sent before this time, after a 429.
    pub blocked_until: Option<DateTime<Utc>>,
    /// 429 answers in a row, for the exponential backoff.
    pub rate_limit_strikes: u32,
    /// The latest request went out inside the floor, as the one retry allowed after a rejected
    /// sign-in. Another such retry waits for the floor.
    pub bypassed_floor: bool,
}

/// The outcome of the latest request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LastResult {
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

/// The fields that hold requests back. Every format version keeps their names and meaning, so they
/// can still be read from a file that no longer parses in full: a damaged one, or one whose
/// snapshots another version of QuotaDial wrote.
#[derive(Deserialize)]
struct HoldFile {
    state: Hold,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct Hold {
    requested_at: Option<DateTime<Utc>>,
    blocked_until: Option<DateTime<Utc>>,
    rate_limit_strikes: u32,
}

/// Shown in place of the last answer when the saved state could not be read in full.
const DAMAGED: &str =
    "QuotaDial could not read its saved state for this account, so the last answer is lost.";

/// The file name for an account's state: a hash of its identity, which keeps the name short and
/// safe on every file system.
#[must_use]
pub fn account_key(account: &Account) -> String {
    let identity = format!(
        "{}\n{}\n{}",
        account.provider,
        account.org.as_deref().unwrap_or_default(),
        account.account_id
    );
    Sha256::digest(identity.as_bytes())
        .iter()
        .take(8)
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// The directory of one provider's saved account state.
#[derive(Debug, Clone)]
pub struct StateStore {
    provider: &'static str,
    dir: PathBuf,
}

/// Holds an account's lock until dropped.
#[derive(Debug)]
pub struct StateLock {
    _file: File,
}

impl StateStore {
    /// `quotadial/<provider>` in the user's cache directory.
    #[must_use]
    pub fn default_dir(provider: &str) -> Option<PathBuf> {
        dirs::cache_dir().map(|dir| dir.join("quotadial").join(provider))
    }

    /// The store for `provider`'s accounts in `dir`.
    #[must_use]
    pub fn new(provider: &'static str, dir: PathBuf) -> Self {
        Self { provider, dir }
    }

    /// Where the files are.
    #[must_use]
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// The error for a cache that cannot be read or written, naming where it is and the fix.
    pub fn error(&self, reason: &dyn fmt::Display) -> Error {
        self.cache_error(format!(
            "{reason}; check that you can read and write that folder"
        ))
    }

    fn cache_error(&self, reason: String) -> Error {
        Error::Cache {
            provider: self.provider,
            location: self.dir.display().to_string(),
            reason,
        }
    }

    /// Takes an account's lock without blocking the async runtime while another process holds
    /// it, giving up after [`LOCK_WAIT`].
    ///
    /// # Errors
    ///
    /// [`Error::Cache`] when the directory cannot be used or the lock is held too long.
    pub async fn lock_account(&self, key: &str) -> Result<StateLock, Error> {
        let (store, key) = (self.clone(), key.to_owned());
        tokio::task::spawn_blocking(move || store.lock(&key, LOCK_WAIT))
            .await
            .map_err(|_| {
                self.cache_error("the task waiting for the account's lock stopped".to_owned())
            })?
            .map_err(|error| match error.kind() {
                io::ErrorKind::TimedOut => {
                    self.cache_error(format!("{error}; try again once it finishes"))
                }
                _ => self.error(&error),
            })
    }

    /// Takes an account's lock, waiting up to `wait` while another QuotaDial process holds it.
    /// Creating the lock file also proves the directory is writable before any request is sent.
    ///
    /// # Errors
    ///
    /// Any error creating the directory or the lock file, or [`io::ErrorKind::TimedOut`].
    pub fn lock(&self, key: &str, wait: Duration) -> io::Result<StateLock> {
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

    /// The account's saved state, and whether the file could not be read in full and should be
    /// saved again.
    ///
    /// A missing file starts afresh. A file that does not parse in full, because it is damaged or
    /// another format version wrote it, keeps the hold it records: the last request time, any 429
    /// block and the 429 count. If no request time can be read, it counts as a request sent at
    /// `now`. A file that cannot be read at all is an error, so no request is sent. Either way a
    /// damaged file never lets a request out early.
    ///
    /// # Errors
    ///
    /// Any error reading the file other than its being missing.
    pub fn load(&self, key: &str, now: DateTime<Utc>) -> io::Result<(AccountState, bool)> {
        let bytes = match fs::read(self.path(key)) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Ok((AccountState::default(), false));
            }
            Err(error) => return Err(error),
        };
        if let Ok(file) = serde_json::from_slice::<StateFile>(&bytes)
            && file.version == FORMAT_VERSION
        {
            return Ok((file.state, false));
        }
        let hold = serde_json::from_slice::<HoldFile>(&bytes)
            .map(|file| file.state)
            .unwrap_or_default();
        let state = AccountState {
            last_result: Some(LastResult::Failed {
                message: DAMAGED.to_owned(),
            }),
            requested_at: Some(hold.requested_at.unwrap_or(now)),
            blocked_until: hold.blocked_until,
            rate_limit_strikes: hold.rate_limit_strikes,
            ..AccountState::default()
        };
        Ok((state, true))
    }

    /// Saves the account's state, replacing the old file in one step so a crash cannot leave half a
    /// file. Call it while holding the account's lock.
    ///
    /// # Errors
    ///
    /// Any error writing or renaming the file.
    pub fn save(&self, key: &str, state: &AccountState) -> io::Result<()> {
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

    /// Saves the state after an answer, trying twice: a lost 429 block would let the next request
    /// out too soon.
    ///
    /// # Errors
    ///
    /// [`Error::Cache`] when both tries fail, so the fetch fails rather than hide the problem.
    pub fn save_answer(&self, key: &str, state: &AccountState) -> Result<(), Error> {
        self.save(key, state)
            .or_else(|_| self.save(key, state))
            .map_err(|error| self.error(&error))
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
        let store = StateStore::new("claude", dir.path().join("quotadial").join("claude"));
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

        assert_eq!(
            store.load(&key, ts("2099-01-01T00:01:00Z")).expect("loads"),
            (state, false)
        );
        assert!(
            !dir.path()
                .join("quotadial/claude")
                .join(format!("{key}.json.tmp"))
                .exists()
        );
    }

    /// What a file that could not be read in full loads as.
    fn held(
        requested_at: &str,
        blocked_until: Option<&str>,
        rate_limit_strikes: u32,
    ) -> (AccountState, bool) {
        let state = AccountState {
            last_result: Some(LastResult::Failed {
                message: DAMAGED.to_owned(),
            }),
            requested_at: Some(ts(requested_at)),
            blocked_until: blocked_until.map(ts),
            rate_limit_strikes,
            ..AccountState::default()
        };
        (state, true)
    }

    #[test]
    fn a_missing_file_starts_afresh() {
        let dir = tempfile::tempdir().expect("temp dir");
        let store = StateStore::new("claude", dir.path().to_path_buf());

        assert_eq!(
            store
                .load("missing", ts("2099-01-01T00:00:00Z"))
                .expect("loads"),
            (AccountState::default(), false)
        );
    }

    #[test]
    fn a_damaged_file_counts_as_a_request_sent_now() {
        let dir = tempfile::tempdir().expect("temp dir");
        let store = StateStore::new("claude", dir.path().to_path_buf());
        let now = "2099-01-01T00:02:00Z";

        for (key, damage) in [
            ("not-json", "{ not json"),
            (
                "cut-off",
                r#"{"version":1,"state":{"requested_at":"2099-01-01T00:00"#,
            ),
            ("empty", ""),
            ("no-state", r#"{"version":1}"#),
            (
                "bad-time",
                r#"{"version":1,"state":{"requested_at":"yesterday"}}"#,
            ),
        ] {
            fs::write(dir.path().join(format!("{key}.json")), damage).expect("writes");

            assert_eq!(
                store.load(key, ts(now)).expect("loads"),
                held(now, None, 0),
                "{key}"
            );
        }
    }

    #[test]
    fn a_file_that_no_longer_parses_in_full_keeps_its_hold() {
        let dir = tempfile::tempdir().expect("temp dir");
        let store = StateStore::new("claude", dir.path().to_path_buf());
        let now = ts("2099-01-01T00:02:00Z");
        // A snapshot written by another version: a window this version cannot read.
        let unknown_snapshot = r#"{"kind":"snapshot","snapshot":{"windows":[{"shape":"new"}]}}"#;

        fs::write(
            dir.path().join("same-version.json"),
            format!(
                r#"{{"version":1,"state":{{"last_result":{unknown_snapshot},
                "requested_at":"2099-01-01T00:00:00Z","blocked_until":"2099-01-01T00:30:00Z",
                "rate_limit_strikes":2}}}}"#
            ),
        )
        .expect("writes");
        assert_eq!(
            store.load("same-version", now).expect("loads"),
            held("2099-01-01T00:00:00Z", Some("2099-01-01T00:30:00Z"), 2)
        );

        fs::write(
            dir.path().join("future.json"),
            r#"{"version":99,"state":{"requested_at":"2099-01-01T00:01:00Z","new_field":true}}"#,
        )
        .expect("writes");
        assert_eq!(
            store.load("future", now).expect("loads"),
            held("2099-01-01T00:01:00Z", None, 0)
        );

        // Another version that records no request time still holds from now.
        fs::write(
            dir.path().join("future-empty.json"),
            r#"{"version":99,"state":{"rate_limit_strikes":3}}"#,
        )
        .expect("writes");
        assert_eq!(
            store.load("future-empty", now).expect("loads"),
            held("2099-01-01T00:02:00Z", None, 3)
        );
    }

    #[test]
    fn a_file_that_cannot_be_read_is_an_error() {
        let dir = tempfile::tempdir().expect("temp dir");
        let store = StateStore::new("claude", dir.path().to_path_buf());
        // A directory where the file should be, which no platform can read as a file.
        fs::create_dir(dir.path().join("unreadable.json")).expect("creates");

        assert!(
            store
                .load("unreadable", ts("2099-01-01T00:00:00Z"))
                .is_err()
        );
    }

    #[test]
    fn the_saved_file_holds_no_token() {
        let dir = tempfile::tempdir().expect("temp dir");
        let store = StateStore::new("claude", dir.path().to_path_buf());
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
        let store = StateStore::new("claude", dir.path().join("claude"));
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
        let store = StateStore::new("claude", dir.path().to_path_buf());
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
        let store = StateStore::new("claude", dir.path().to_path_buf());
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
