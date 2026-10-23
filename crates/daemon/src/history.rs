//! The daemon's history: every snapshot that brought new data, in SQLite under the data
//! directory, kept for 30 days (the PRD's free-tier history).
//!
//! The database runs in WAL mode, so readers such as a later history view never block the
//! poller's writes. A snapshot is stored only when it differs from the account's last stored one
//! in its newest fetch time, status or message, so a replay from the cache adds nothing. The file
//! is created readable only by the user, in a folder only the user can open.

use std::fs::OpenOptions;
use std::path::Path;
use std::sync::Mutex;

use chrono::{DateTime, SecondsFormat, Utc};
use quotadial_core::{Account, Snapshot};
use rusqlite::{Connection, OptionalExtension, params};

/// How long history is kept.
pub const KEEP_DAYS: i64 = 30;

/// The schema version, kept in SQLite's `user_version`.
const SCHEMA_VERSION: i32 = 1;

/// The history database.
#[derive(Debug)]
pub struct History {
    connection: Mutex<Connection>,
}

/// Why history could not be read or written.
#[derive(Debug)]
pub struct HistoryError(String);

impl std::fmt::Display for HistoryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl From<rusqlite::Error> for HistoryError {
    fn from(error: rusqlite::Error) -> Self {
        Self(error.to_string())
    }
}

impl From<std::io::Error> for HistoryError {
    fn from(error: std::io::Error) -> Self {
        Self(error.to_string())
    }
}

impl History {
    /// Opens or creates the database at `path`, in WAL mode.
    ///
    /// # Errors
    ///
    /// When the file cannot be created or opened, or holds a schema this version does not know.
    pub fn open(path: &Path) -> Result<Self, HistoryError> {
        // Created first, so SQLite opens a file only the user can read.
        let mut options = OpenOptions::new();
        options.create(true).truncate(false).write(true);
        #[cfg(unix)]
        std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
        options.open(path)?;

        let connection = Connection::open(path)?;
        connection.busy_timeout(std::time::Duration::from_secs(5))?;
        let mode: String =
            connection.pragma_update_and_check(None, "journal_mode", "WAL", |row| row.get(0))?;
        if !mode.eq_ignore_ascii_case("wal") {
            return Err(HistoryError(format!(
                "SQLite would not use WAL mode for {} (it chose {mode})",
                path.display()
            )));
        }
        connection.pragma_update(None, "synchronous", "NORMAL")?;
        let version: i32 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
        if version > SCHEMA_VERSION {
            return Err(HistoryError(format!(
                "{} was written by a newer QuotaDial (schema {version})",
                path.display()
            )));
        }
        connection.execute_batch(
            "CREATE TABLE IF NOT EXISTS snapshots (
                 id INTEGER PRIMARY KEY,
                 provider TEXT NOT NULL,
                 org TEXT NOT NULL,
                 account_id TEXT NOT NULL,
                 polled_at TEXT NOT NULL,
                 fetched_at TEXT,
                 status TEXT NOT NULL,
                 error TEXT,
                 snapshot TEXT NOT NULL
             );
             CREATE INDEX IF NOT EXISTS snapshots_by_account
                 ON snapshots (provider, org, account_id, id);
             CREATE INDEX IF NOT EXISTS snapshots_by_time ON snapshots (polled_at);",
        )?;
        connection.pragma_update(None, "user_version", SCHEMA_VERSION)?;
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    /// Stores `snapshot`, polled at `polled_at`, if it brings new data. Returns whether it did.
    ///
    /// # Errors
    ///
    /// When the database cannot be written.
    pub fn record(
        &self,
        snapshot: &Snapshot,
        polled_at: DateTime<Utc>,
    ) -> Result<bool, HistoryError> {
        let account = &snapshot.account;
        let org = account.org.as_deref().unwrap_or_default();
        let fetched_at = newest_fetch(snapshot).map(timestamp);
        let status = serde_json::to_value(snapshot.status)
            .ok()
            .and_then(|value| value.as_str().map(str::to_owned))
            .unwrap_or_default();
        let connection = self.lock();
        let last: Option<(Option<String>, String, Option<String>)> = connection
            .query_row(
                "SELECT fetched_at, status, error FROM snapshots
                 WHERE provider = ?1 AND org = ?2 AND account_id = ?3
                 ORDER BY id DESC LIMIT 1",
                params![account.provider, org, account.account_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()?;
        if last.as_ref() == Some(&(fetched_at.clone(), status.clone(), snapshot.error.clone())) {
            return Ok(false);
        }
        let json = serde_json::to_string(snapshot)
            .map_err(|error| HistoryError(format!("the snapshot could not be saved: {error}")))?;
        connection.execute(
            "INSERT INTO snapshots
                 (provider, org, account_id, polled_at, fetched_at, status, error, snapshot)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                account.provider,
                org,
                account.account_id,
                timestamp(polled_at),
                fetched_at,
                status,
                snapshot.error,
                json
            ],
        )?;
        Ok(true)
    }

    /// Deletes what was polled more than [`KEEP_DAYS`] before `now`. Returns how many snapshots
    /// went.
    ///
    /// # Errors
    ///
    /// When the database cannot be written.
    pub fn prune(&self, now: DateTime<Utc>) -> Result<usize, HistoryError> {
        let before = timestamp(now - chrono::TimeDelta::days(KEEP_DAYS));
        Ok(self.lock().execute(
            "DELETE FROM snapshots WHERE polled_at < ?1",
            params![before],
        )?)
    }

    /// The newest stored snapshot of every account, so a restarted daemon has numbers to show at
    /// once.
    ///
    /// # Errors
    ///
    /// When the database cannot be read.
    pub fn latest(&self) -> Result<Vec<Snapshot>, HistoryError> {
        let connection = self.lock();
        let mut statement = connection.prepare(
            "SELECT snapshot FROM snapshots WHERE id IN
                 (SELECT MAX(id) FROM snapshots GROUP BY provider, org, account_id)
             ORDER BY id",
        )?;
        let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
        let mut snapshots = Vec::new();
        for row in rows {
            // A row this version cannot read, from an older schema of the snapshot, is skipped.
            if let Ok(snapshot) = serde_json::from_str::<Snapshot>(&row?) {
                snapshots.push(snapshot);
            }
        }
        Ok(snapshots)
    }

    /// How many snapshots are stored for `account`.
    ///
    /// # Errors
    ///
    /// When the database cannot be read.
    pub fn count(&self, account: &Account) -> Result<usize, HistoryError> {
        let count: i64 = self.lock().query_row(
            "SELECT COUNT(*) FROM snapshots
             WHERE provider = ?1 AND org = ?2 AND account_id = ?3",
            params![
                account.provider,
                account.org.as_deref().unwrap_or_default(),
                account.account_id
            ],
            |row| row.get(0),
        )?;
        Ok(usize::try_from(count).unwrap_or_default())
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Connection> {
        // A panic while holding the connection leaves SQLite consistent, so the lock is reused.
        self.connection
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

/// The newest `fetched_at` among the snapshot's windows.
fn newest_fetch(snapshot: &Snapshot) -> Option<DateTime<Utc>> {
    snapshot
        .windows
        .iter()
        .map(|window| window.fetched_at)
        .max()
}

/// A UTC time in a fixed format, so text order is time order.
fn timestamp(at: DateTime<Utc>) -> String {
    at.to_rfc3339_opts(SecondsFormat::Micros, true)
}

#[cfg(test)]
mod tests {
    use quotadial_core::{Source, Status, Window, WindowKind};

    use super::*;

    fn ts(rfc3339: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(rfc3339)
            .expect("valid timestamp")
            .with_timezone(&Utc)
    }

    fn account(org: &str) -> Account {
        Account {
            provider: "claude".into(),
            account_id: "00000000-0000-0000-0000-000000000001".into(),
            display_name: "user@example.com".into(),
            plan: Some("max".into()),
            org: Some(org.into()),
        }
    }

    fn snapshot(org: &str, used_pct: f64, fetched_at: &str, source: Source) -> Snapshot {
        Snapshot {
            account: account(org),
            windows: vec![Window {
                kind: WindowKind::Session,
                label: "Session".into(),
                used_pct,
                resets_at: Some(ts("2099-01-01T05:00:00Z")),
                window_seconds: 18_000,
                scope_model: None,
                source,
                fetched_at: ts(fetched_at),
                unconfirmed: false,
            }],
            extra_usage: None,
            status: Status::Ok,
            error: None,
        }
    }

    fn open() -> (tempfile::TempDir, History) {
        let dir = tempfile::tempdir().expect("temp dir");
        let history = History::open(&dir.path().join("history.db")).expect("opens");
        (dir, history)
    }

    #[test]
    fn the_database_runs_in_wal_mode_and_is_private() {
        let (dir, history) = open();

        let mode: String = history
            .lock()
            .pragma_query_value(None, "journal_mode", |row| row.get(0))
            .expect("reads the mode");
        assert_eq!(mode, "wal");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(dir.path().join("history.db"))
                .expect("metadata")
                .permissions()
                .mode();
            assert_eq!(mode & 0o777, 0o600);
        }
        drop(dir);
    }

    #[test]
    fn only_new_data_is_stored() {
        let (_dir, history) = open();
        let first = snapshot("org-a", 10.0, "2099-01-01T00:00:00Z", Source::Api);

        assert!(
            history
                .record(&first, ts("2099-01-01T00:00:00Z"))
                .expect("records")
        );
        // The same numbers replayed from the cache a minute later add nothing.
        let replay = first.clone().served_from_cache();
        assert!(
            !history
                .record(&replay, ts("2099-01-01T00:01:00Z"))
                .expect("records")
        );
        // A new fetch does.
        let next = snapshot("org-a", 12.0, "2099-01-01T00:05:00Z", Source::Api);
        assert!(
            history
                .record(&next, ts("2099-01-01T00:05:00Z"))
                .expect("records")
        );
        // So does a change of status with the same numbers.
        let mut limited = next.clone().served_from_cache();
        limited.status = Status::RateLimited;
        limited.error = Some("Anthropic is rate-limiting usage requests.".into());
        assert!(
            history
                .record(&limited, ts("2099-01-01T00:10:00Z"))
                .expect("records")
        );

        assert_eq!(history.count(&account("org-a")).expect("counts"), 3);
    }

    #[test]
    fn each_account_keeps_its_own_history() {
        let (_dir, history) = open();
        let personal = snapshot("org-a", 10.0, "2099-01-01T00:00:00Z", Source::Api);
        let work = snapshot("org-b", 50.0, "2099-01-01T00:00:00Z", Source::Api);

        assert!(
            history
                .record(&personal, ts("2099-01-01T00:00:00Z"))
                .expect("records")
        );
        assert!(
            history
                .record(&work, ts("2099-01-01T00:00:00Z"))
                .expect("records")
        );

        let latest = history.latest().expect("reads");
        assert_eq!(latest, vec![personal, work]);
    }

    #[test]
    fn history_older_than_thirty_days_is_pruned() {
        let (_dir, history) = open();
        history
            .record(
                &snapshot("org-a", 10.0, "2099-01-01T00:00:00Z", Source::Api),
                ts("2099-01-01T00:00:00Z"),
            )
            .expect("records");
        history
            .record(
                &snapshot("org-a", 20.0, "2099-01-20T00:00:00Z", Source::Api),
                ts("2099-01-20T00:00:00Z"),
            )
            .expect("records");

        let removed = history.prune(ts("2099-02-05T00:00:00Z")).expect("prunes");

        assert_eq!(removed, 1);
        assert_eq!(history.count(&account("org-a")).expect("counts"), 1);
    }

    #[test]
    fn a_database_from_a_newer_quotadial_is_refused() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("history.db");
        drop(History::open(&path).expect("opens"));
        Connection::open(&path)
            .expect("opens")
            .pragma_update(None, "user_version", 99)
            .expect("sets the version");

        let error = History::open(&path).expect_err("refused");

        assert!(
            error
                .to_string()
                .ends_with("was written by a newer QuotaDial (schema 99)")
        );
    }

    #[test]
    fn the_history_survives_a_restart() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("history.db");
        let first = snapshot("org-a", 10.0, "2099-01-01T00:00:00Z", Source::Api);
        History::open(&path)
            .expect("opens")
            .record(&first, ts("2099-01-01T00:00:00Z"))
            .expect("records");

        let reopened = History::open(&path).expect("reopens");

        assert_eq!(reopened.latest().expect("reads"), vec![first.clone()]);
        assert!(
            !reopened
                .record(&first, ts("2099-01-01T00:02:00Z"))
                .expect("records")
        );
    }
}
