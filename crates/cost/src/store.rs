//! `costs.db` in the data folder: how far each log was read, and each request of the last
//! [`KEEP`](crate::KEEP), so a log is read once and a request counts once however many files hold
//! it. Nothing in it is a log's content: only token counts, model names, times, a hash of each
//! request's ids, and the logs' paths. It is a cache: deleted, it costs one full read of the
//! week's logs.

use std::collections::HashMap;
use std::fs::OpenOptions;
use std::path::Path;
use std::sync::{Mutex, MutexGuard, PoisonError};

use chrono::{DateTime, Utc};
use quotadial_core::Tokens;
use rusqlite::{Connection, TransactionBehavior, params};
use sha2::{Digest, Sha256};

use crate::{CostError, Request};

/// The schema this version writes. A database with another is a cache from another version, and
/// is started again.
const SCHEMA_VERSION: i32 = 1;

/// How far a log was read, and what the reading has shown of it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct FileState {
    /// The file's length when it was last read.
    pub(crate) len: u64,
    /// Where the first line not yet read starts.
    pub(crate) read_to: u64,
    /// For a Codex session: the model its latest turn named.
    pub(crate) model: Option<String>,
    /// For a Codex session: its usage records and its token counts.
    pub(crate) records: u64,
    pub(crate) token_counts: u64,
}

/// A request as the store keeps it.
pub(crate) struct Stored<'a> {
    pub(crate) provider: &'a str,
    pub(crate) model: &'a str,
    pub(crate) tokens: Tokens,
}

pub(crate) struct Store {
    connection: Mutex<Connection>,
}

impl Store {
    /// Opens the database at `path`, made readable only by the user, starting it again if
    /// another version wrote it.
    pub(crate) fn open(path: &Path) -> Result<Self, CostError> {
        let mut options = OpenOptions::new();
        options.create(true).truncate(false).write(true);
        #[cfg(unix)]
        std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
        options.open(path)?;

        let mut connection = Connection::open(path)?;
        connection.busy_timeout(std::time::Duration::from_secs(5))?;
        connection.pragma_update(None, "journal_mode", "WAL")?;
        connection.pragma_update(None, "synchronous", "NORMAL")?;
        let upgrade = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let version: i32 = upgrade.pragma_query_value(None, "user_version", |row| row.get(0))?;
        if version != SCHEMA_VERSION {
            upgrade.execute_batch(
                "DROP TABLE IF EXISTS files;
                 DROP TABLE IF EXISTS requests;",
            )?;
        }
        upgrade.execute_batch(
            "CREATE TABLE IF NOT EXISTS files (
                 path TEXT PRIMARY KEY,
                 len INTEGER NOT NULL,
                 read_to INTEGER NOT NULL,
                 model TEXT,
                 records INTEGER NOT NULL,
                 token_counts INTEGER NOT NULL
             );
             CREATE TABLE IF NOT EXISTS requests (
                 key INTEGER PRIMARY KEY,
                 provider TEXT NOT NULL,
                 at INTEGER NOT NULL,
                 model TEXT NOT NULL,
                 input INTEGER NOT NULL,
                 output INTEGER NOT NULL,
                 cache_read INTEGER NOT NULL,
                 cache_write INTEGER NOT NULL,
                 cache_write_1h INTEGER NOT NULL
             );
             CREATE INDEX IF NOT EXISTS requests_by_time ON requests (at);",
        )?;
        upgrade.pragma_update(None, "user_version", SCHEMA_VERSION)?;
        upgrade.commit()?;
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    fn lock(&self) -> MutexGuard<'_, Connection> {
        self.connection
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    /// Every log read before, by path.
    pub(crate) fn files(&self) -> Result<HashMap<String, FileState>, CostError> {
        let connection = self.lock();
        let mut statement = connection
            .prepare("SELECT path, len, read_to, model, records, token_counts FROM files")?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                FileState {
                    len: count(row, 1)?,
                    read_to: count(row, 2)?,
                    model: row.get(3)?,
                    records: count(row, 4)?,
                    token_counts: count(row, 5)?,
                },
            ))
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// Keeps what a reading of the log at `path` found: its requests from `keep_since` on, each
    /// replacing an earlier part of the same request unless that has more output, and how far it
    /// was read. A request the log names no key for is named by where it was found.
    pub(crate) fn record(
        &self,
        provider: &str,
        path: &str,
        state: &FileState,
        requests: &[(u64, Request)],
        keep_since: DateTime<Utc>,
    ) -> Result<(), CostError> {
        let mut connection = self.lock();
        let transaction = connection.transaction()?;
        {
            let mut insert = transaction.prepare_cached(
                "INSERT INTO requests
                     (key, provider, at, model, input, output, cache_read, cache_write,
                      cache_write_1h)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                 ON CONFLICT (key) DO UPDATE SET
                     at = excluded.at, model = excluded.model, input = excluded.input,
                     output = excluded.output, cache_read = excluded.cache_read,
                     cache_write = excluded.cache_write, cache_write_1h = excluded.cache_write_1h
                 WHERE excluded.output >= requests.output",
            )?;
            for (offset, request) in requests {
                if request.at < keep_since {
                    continue;
                }
                let key = match &request.key {
                    Some(key) => key.clone(),
                    None => format!("{provider}:{path}:{offset}"),
                };
                let tokens = request.tokens;
                insert.execute(params![
                    hash(&key),
                    provider,
                    request.at.timestamp(),
                    request.model,
                    int(tokens.input),
                    int(tokens.output),
                    int(tokens.cache_read),
                    int(tokens.cache_write),
                    int(tokens.cache_write_1h),
                ])?;
            }
            transaction.execute(
                "INSERT INTO files (path, len, read_to, model, records, token_counts)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT (path) DO UPDATE SET
                     len = excluded.len, read_to = excluded.read_to, model = excluded.model,
                     records = excluded.records, token_counts = excluded.token_counts",
                params![
                    path,
                    int(state.len),
                    int(state.read_to),
                    state.model,
                    int(state.records),
                    int(state.token_counts),
                ],
            )?;
        }
        transaction.commit()?;
        Ok(())
    }

    /// Forgets the logs at `paths`, which are gone or older than the week.
    pub(crate) fn forget(&self, paths: &[&str]) -> Result<(), CostError> {
        let mut connection = self.lock();
        let transaction = connection.transaction()?;
        {
            let mut delete = transaction.prepare_cached("DELETE FROM files WHERE path = ?1")?;
            for path in paths {
                delete.execute([path])?;
            }
        }
        transaction.commit()?;
        Ok(())
    }

    /// Deletes requests made before `before`.
    pub(crate) fn prune(&self, before: DateTime<Utc>) -> Result<(), CostError> {
        self.lock()
            .execute("DELETE FROM requests WHERE at < ?1", [before.timestamp()])?;
        Ok(())
    }

    /// Hands each request made from `since` to `until` to `each`.
    pub(crate) fn requests(
        &self,
        since: DateTime<Utc>,
        until: DateTime<Utc>,
        mut each: impl FnMut(Stored<'_>),
    ) -> Result<(), CostError> {
        let connection = self.lock();
        let mut statement = connection.prepare(
            "SELECT provider, model, input, output, cache_read, cache_write, cache_write_1h
             FROM requests WHERE at >= ?1 AND at <= ?2",
        )?;
        let mut rows = statement.query([since.timestamp(), until.timestamp()])?;
        while let Some(row) = rows.next()? {
            let (provider, model) = (row.get_ref(0)?.as_str()?, row.get_ref(1)?.as_str()?);
            each(Stored {
                provider,
                model,
                tokens: Tokens {
                    input: count(row, 2)?,
                    output: count(row, 3)?,
                    cache_read: count(row, 4)?,
                    cache_write: count(row, 5)?,
                    cache_write_1h: count(row, 6)?,
                },
            });
        }
        Ok(())
    }
}

/// A count as SQLite keeps it, which is signed.
fn int(count: u64) -> i64 {
    i64::try_from(count).unwrap_or(i64::MAX)
}

/// A count read back.
fn count(row: &rusqlite::Row<'_>, index: usize) -> rusqlite::Result<u64> {
    Ok(u64::try_from(row.get::<_, i64>(index)?).unwrap_or(0))
}

/// A request's key as the store keeps it: the first 8 bytes of its SHA-256. Two of a week's
/// requests share one by chance about once in ten billion weeks.
fn hash(key: &str) -> i64 {
    let digest = Sha256::digest(key.as_bytes());
    let mut first = [0; 8];
    first.copy_from_slice(&digest[..8]);
    i64::from_be_bytes(first)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(text: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(text)
            .expect("valid timestamp")
            .with_timezone(&Utc)
    }

    fn request(key: Option<&str>, at_text: &str, output: u64) -> Request {
        Request {
            key: key.map(str::to_owned),
            at: at(at_text),
            model: "claude-opus-5-5".to_owned(),
            tokens: Tokens {
                input: 10,
                output,
                ..Tokens::default()
            },
        }
    }

    fn outputs(store: &Store) -> Vec<u64> {
        let mut found = Vec::new();
        store
            .requests(
                at("2026-01-01T00:00:00Z"),
                at("2027-01-01T00:00:00Z"),
                |stored| {
                    found.push(stored.tokens.output);
                },
            )
            .expect("reads");
        found.sort_unstable();
        found
    }

    #[test]
    fn a_request_counts_once_with_its_most_output() {
        let dir = tempfile::tempdir().expect("temp dir");
        let store = Store::open(&dir.path().join("costs.db")).expect("opens");
        let state = FileState::default();
        let old = at("2026-01-01T00:00:00Z");

        store
            .record(
                "claude",
                "/a.jsonl",
                &state,
                &[
                    (0, request(Some("claude:m1:r1"), "2026-10-23T12:00:00Z", 1)),
                    (
                        10,
                        request(Some("claude:m1:r1"), "2026-10-23T12:00:01Z", 50),
                    ),
                    (20, request(None, "2026-10-23T12:00:02Z", 7)),
                ],
                old,
            )
            .expect("records");
        store
            .record(
                "claude",
                "/b.jsonl",
                &state,
                &[
                    (0, request(Some("claude:m1:r1"), "2026-10-23T12:00:00Z", 1)),
                    (20, request(None, "2026-10-23T12:00:02Z", 7)),
                ],
                old,
            )
            .expect("records");

        assert_eq!(
            outputs(&store),
            [7, 7, 50],
            "a copy's earlier part does not replace the whole response; unnamed requests count where they were found"
        );
    }

    #[test]
    fn requests_before_the_keeping_time_are_not_kept() {
        let dir = tempfile::tempdir().expect("temp dir");
        let store = Store::open(&dir.path().join("costs.db")).expect("opens");

        store
            .record(
                "codex",
                "/s.jsonl",
                &FileState::default(),
                &[
                    (0, request(Some("codex:old"), "2026-10-01T00:00:00Z", 1)),
                    (1, request(Some("codex:new"), "2026-10-20T00:00:00Z", 2)),
                ],
                at("2026-10-15T00:00:00Z"),
            )
            .expect("records");
        assert_eq!(outputs(&store), [2]);

        store.prune(at("2026-10-21T00:00:00Z")).expect("prunes");
        assert_eq!(outputs(&store), Vec::<u64>::new());
    }

    #[test]
    fn how_far_each_log_was_read_survives_a_restart() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("costs.db");
        let state = FileState {
            len: 120,
            read_to: 100,
            model: Some("gpt-6-astra".to_owned()),
            records: 3,
            token_counts: 4,
        };
        Store::open(&path)
            .expect("opens")
            .record("codex", "/s.jsonl", &state, &[], at("2026-10-15T00:00:00Z"))
            .expect("records");

        let store = Store::open(&path).expect("reopens");
        assert_eq!(
            store.files().expect("reads"),
            HashMap::from([("/s.jsonl".to_owned(), state)])
        );
        store.forget(&["/s.jsonl"]).expect("forgets");
        assert!(store.files().expect("reads").is_empty());
    }

    #[test]
    fn a_cache_from_another_version_is_started_again() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("costs.db");
        Store::open(&path)
            .expect("opens")
            .record(
                "codex",
                "/s.jsonl",
                &FileState::default(),
                &[(0, request(Some("codex:r"), "2026-10-20T00:00:00Z", 2))],
                at("2026-10-15T00:00:00Z"),
            )
            .expect("records");
        Connection::open(&path)
            .expect("opens")
            .pragma_update(None, "user_version", SCHEMA_VERSION + 1)
            .expect("sets");

        let store = Store::open(&path).expect("reopens");

        assert!(store.files().expect("reads").is_empty());
        assert_eq!(outputs(&store), Vec::<u64>::new());
    }

    #[cfg(unix)]
    #[test]
    fn only_the_user_can_read_it() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("costs.db");
        Store::open(&path).expect("opens");

        let mode = std::fs::metadata(&path)
            .expect("exists")
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600);
    }
}
