//! The daemon's history: every snapshot that brought new data, and every reading of every window,
//! in SQLite under the data directory, kept for 30 days (the PRD's free-tier history) or as long
//! as `history_days` in `config.json` says, up to a year.
//!
//! The database runs in WAL mode, so readers such as a later history view never block the
//! poller's writes. A snapshot is stored only when it differs from the account's last stored one
//! in its newest fetch time, status or message, so a replay from the cache adds nothing. A window's
//! reading is stored once per fetch time, in `readings`, which is what a window's pace is fitted to
//! (P-016). The file is created readable only by the user, in a folder only the user can open.

use std::fs::OpenOptions;
use std::path::Path;
use std::sync::Mutex;

use chrono::{DateTime, SecondsFormat, Utc};
use quotadial_core::{Account, Pace, Reading, Snapshot, Status, Window};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::Serialize;

/// How long history is kept unless `history_days` says otherwise.
pub const KEEP_DAYS: u32 = 30;
/// The longest history can be kept: a year (the PRD's Pro history).
pub const MAX_KEEP_DAYS: u32 = 365;

/// The schema version, kept in SQLite's `user_version`. Version 2 added `readings`, version 3 the
/// alert rules and their firings, and version 4 where each firing was sent.
const SCHEMA_VERSION: i32 = 4;

/// The history database.
#[derive(Debug)]
pub struct History {
    connection: Mutex<Connection>,
    /// How many days of history are kept.
    keep_days: u32,
}

/// Why history could not be read or written.
#[derive(Debug)]
pub struct HistoryError(String);

impl HistoryError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

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

        let mut connection = Connection::open(path)?;
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
        // One transaction, so an upgrade cut short is done again in full next time. It takes the
        // write lock before it reads the version, so two processes opening an older database at
        // once upgrade it once.
        let upgrade = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let version: i32 = upgrade.pragma_query_value(None, "user_version", |row| row.get(0))?;
        if version > SCHEMA_VERSION {
            return Err(HistoryError(format!(
                "{} was written by a newer QuotaDial (schema {version})",
                path.display()
            )));
        }
        upgrade.execute_batch(
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
             CREATE INDEX IF NOT EXISTS snapshots_by_time ON snapshots (polled_at);
             CREATE TABLE IF NOT EXISTS readings (
                 provider TEXT NOT NULL,
                 org TEXT NOT NULL,
                 account_id TEXT NOT NULL,
                 label TEXT NOT NULL,
                 scope_model TEXT NOT NULL,
                 fetched_at TEXT NOT NULL,
                 kind TEXT NOT NULL,
                 used_pct REAL NOT NULL,
                 resets_at TEXT,
                 window_seconds INTEGER NOT NULL,
                 source TEXT NOT NULL,
                 unconfirmed INTEGER NOT NULL,
                 PRIMARY KEY (provider, org, account_id, label, scope_model, fetched_at)
             ) WITHOUT ROWID;
             CREATE INDEX IF NOT EXISTS readings_by_time ON readings (fetched_at);",
        )?;
        if version < 2 {
            // Version 1 kept whole snapshots only. Their windows become readings, so a window's
            // pace can count what was fetched before the upgrade.
            let mut statement = upgrade.prepare("SELECT snapshot FROM snapshots ORDER BY id")?;
            let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
            for row in rows {
                if let Ok(snapshot) = serde_json::from_str::<Snapshot>(&row?) {
                    insert_readings(&upgrade, &snapshot)?;
                }
            }
        }
        crate::alerts::upgrade(&upgrade, version)?;
        upgrade.pragma_update(None, "user_version", SCHEMA_VERSION)?;
        upgrade.commit()?;
        Ok(Self {
            connection: Mutex::new(connection),
            keep_days: KEEP_DAYS,
        })
    }

    /// The history, keeping `days` of it, from 1 to [`MAX_KEEP_DAYS`].
    #[must_use]
    pub fn keeping(mut self, days: u32) -> Self {
        self.keep_days = days.clamp(1, MAX_KEEP_DAYS);
        self
    }

    /// How many days of history are kept.
    #[must_use]
    pub fn keep_days(&self) -> u32 {
        self.keep_days
    }

    /// Stores `snapshot`, polled at `polled_at`, if it brings new data, and its window readings that
    /// are not stored yet. Returns whether the snapshot was stored.
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
        let status = name(snapshot.status);
        let connection = self.lock();
        let transaction = connection.unchecked_transaction()?;
        insert_readings(&transaction, snapshot)?;
        let last: Option<(Option<String>, String, Option<String>)> = transaction
            .query_row(
                "SELECT fetched_at, status, error FROM snapshots
                 WHERE provider = ?1 AND org = ?2 AND account_id = ?3
                 ORDER BY id DESC LIMIT 1",
                params![account.provider, org, account.account_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()?;
        if last.as_ref() == Some(&(fetched_at.clone(), status.clone(), snapshot.error.clone())) {
            // Its readings are kept all the same: one window's can be new when the newest is not.
            transaction.commit()?;
            return Ok(false);
        }
        let json = serde_json::to_string(snapshot)
            .map_err(|error| HistoryError(format!("the snapshot could not be saved: {error}")))?;
        transaction.execute(
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
        transaction.commit()?;
        Ok(true)
    }

    /// The confirmed readings of `account`'s `window` fetched at or after `since`, oldest first:
    /// what the window's pace is fitted to.
    ///
    /// # Errors
    ///
    /// When the database cannot be read.
    pub fn readings(
        &self,
        account: &Account,
        window: &Window,
        since: Option<DateTime<Utc>>,
    ) -> Result<Vec<Reading>, HistoryError> {
        let connection = self.lock();
        let mut statement = connection.prepare_cached(
            "SELECT fetched_at, used_pct FROM readings
             WHERE provider = ?1 AND org = ?2 AND account_id = ?3 AND label = ?4
                 AND scope_model = ?5 AND fetched_at >= ?6 AND unconfirmed = 0
             ORDER BY fetched_at",
        )?;
        let rows = statement.query_map(
            params![
                account.provider,
                account.org.as_deref().unwrap_or_default(),
                account.account_id,
                window.label,
                window.scope_model.as_deref().unwrap_or_default(),
                // The empty string sorts before every time.
                since.map(timestamp).unwrap_or_default(),
            ],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?)),
        )?;
        let mut readings = Vec::new();
        for row in rows {
            let (fetched_at, used_pct) = row?;
            if let Ok(fetched_at) = DateTime::parse_from_rfc3339(&fetched_at) {
                readings.push(Reading {
                    fetched_at: fetched_at.with_timezone(&Utc),
                    used_pct,
                });
            }
        }
        Ok(readings)
    }

    /// `snapshot` with every window's pace, from the readings here (P-016). Only an `ok` snapshot's
    /// numbers are current enough to project from, so any other gets none, as does a window whose
    /// readings cannot be read.
    #[must_use]
    pub fn with_pace(&self, mut snapshot: Snapshot) -> Snapshot {
        if snapshot.status != Status::Ok {
            return snapshot;
        }
        let account = &snapshot.account;
        for window in &mut snapshot.windows {
            window.pace = self
                .readings(account, window, Pace::since(window))
                .ok()
                .and_then(|readings| Pace::of(window, &readings));
        }
        snapshot
    }

    /// Deletes what is older than the days kept, before `now`. Returns how many snapshots went.
    ///
    /// # Errors
    ///
    /// When the database cannot be written.
    pub fn prune(&self, now: DateTime<Utc>) -> Result<usize, HistoryError> {
        let before = timestamp(now - chrono::TimeDelta::days(i64::from(self.keep_days)));
        let connection = self.lock();
        connection.execute(
            "DELETE FROM readings WHERE fetched_at < ?1",
            params![before],
        )?;
        connection.execute("DELETE FROM firings WHERE fired_at < ?1", params![before])?;
        Ok(connection.execute(
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

    pub(crate) fn lock(&self) -> std::sync::MutexGuard<'_, Connection> {
        // A panic while holding the connection leaves SQLite consistent, so the lock is reused.
        self.connection
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

/// Stores each of `snapshot`'s window readings that is not stored yet.
fn insert_readings(connection: &Connection, snapshot: &Snapshot) -> rusqlite::Result<()> {
    let account = &snapshot.account;
    let mut statement = connection.prepare_cached(
        "INSERT OR IGNORE INTO readings
             (provider, org, account_id, label, scope_model, fetched_at, kind, used_pct, resets_at,
              window_seconds, source, unconfirmed)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
    )?;
    for window in &snapshot.windows {
        statement.execute(params![
            account.provider,
            account.org.as_deref().unwrap_or_default(),
            account.account_id,
            window.label,
            window.scope_model.as_deref().unwrap_or_default(),
            timestamp(window.fetched_at),
            name(window.kind),
            window.used_pct,
            window.resets_at.map(timestamp),
            i64::try_from(window.window_seconds).unwrap_or(i64::MAX),
            name(window.source),
            window.unconfirmed,
        ])?;
    }
    Ok(())
}

/// An enum's name as its JSON has it, such as `ok` or `model_weekly`.
fn name(value: impl Serialize) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_default()
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
pub(crate) fn timestamp(at: DateTime<Utc>) -> String {
    at.to_rfc3339_opts(SecondsFormat::Micros, true)
}

#[cfg(test)]
mod tests {
    use quotadial_core::{Source, WindowKind};

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
            sign_in: None,
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
                pace: None,
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
    fn each_window_reading_is_kept_once() {
        let (_dir, history) = open();
        let first = snapshot("org-a", 10.0, "2099-01-01T00:00:00Z", Source::Api);
        let window = first.windows[0].clone();

        history
            .record(&first, ts("2099-01-01T00:00:00Z"))
            .expect("records");
        history
            .record(
                &first.clone().served_from_cache(),
                ts("2099-01-01T00:01:00Z"),
            )
            .expect("records");
        let mut limited = first.clone().served_from_cache();
        limited.status = Status::RateLimited;
        history
            .record(&limited, ts("2099-01-01T00:02:00Z"))
            .expect("records");
        history
            .record(
                &snapshot("org-a", 12.0, "2099-01-01T00:05:00Z", Source::Api),
                ts("2099-01-01T00:05:00Z"),
            )
            .expect("records");

        assert_eq!(
            history
                .readings(&account("org-a"), &window, None)
                .expect("reads"),
            [
                Reading {
                    fetched_at: ts("2099-01-01T00:00:00Z"),
                    used_pct: 10.0
                },
                Reading {
                    fetched_at: ts("2099-01-01T00:05:00Z"),
                    used_pct: 12.0
                },
            ],
            "a replay and a change of status are the same reading"
        );
        assert!(
            history
                .readings(&account("org-b"), &window, None)
                .expect("reads")
                .is_empty(),
            "another account's"
        );
        let opus = Window {
            label: "Weekly (Opus)".into(),
            scope_model: Some("opus".into()),
            ..window
        };
        assert!(
            history
                .readings(&account("org-a"), &opus, None)
                .expect("reads")
                .is_empty(),
            "another window's"
        );
    }

    #[test]
    fn readings_leave_out_unconfirmed_ones_and_those_before_the_window_began() {
        let (_dir, history) = open();
        for (used_pct, fetched_at) in [
            (10.0, "2099-01-01T00:00:00Z"),
            (12.0, "2099-01-01T00:05:00Z"),
        ] {
            history
                .record(
                    &snapshot("org-a", used_pct, fetched_at, Source::Api),
                    ts(fetched_at),
                )
                .expect("records");
        }
        let mut sudden_zero = snapshot("org-a", 0.0, "2099-01-01T00:10:00Z", Source::Api);
        sudden_zero.windows[0].unconfirmed = true;
        history
            .record(&sudden_zero, ts("2099-01-01T00:10:00Z"))
            .expect("records");
        let window = &sudden_zero.windows[0];

        let used = |since| {
            history
                .readings(&account("org-a"), window, since)
                .expect("reads")
                .iter()
                .map(|reading| reading.used_pct)
                .collect::<Vec<_>>()
        };

        assert_eq!(used(None), [10.0, 12.0]);
        assert_eq!(used(Some(ts("2099-01-01T00:05:00Z"))), [12.0]);
    }

    #[test]
    fn an_ok_snapshot_gets_a_pace_from_its_readings() {
        let (_dir, history) = open();
        for (used_pct, fetched_at) in [
            (10.0, "2099-01-01T00:00:00Z"),
            (15.0, "2099-01-01T00:15:00Z"),
            (20.0, "2099-01-01T00:30:00Z"),
        ] {
            history
                .record(
                    &snapshot("org-a", used_pct, fetched_at, Source::Api),
                    ts(fetched_at),
                )
                .expect("records");
        }
        let latest = snapshot("org-a", 20.0, "2099-01-01T00:30:00Z", Source::Api);

        let paced = history.with_pace(latest.clone());

        // 80 points to go at 20 an hour is 4 hours, half an hour before the session resets.
        assert_eq!(
            paced.windows[0].pace,
            Some(Pace {
                pct_per_hour: 20.0,
                runs_out_at: Some(ts("2099-01-01T04:30:00Z")),
            })
        );
        let mut stale = latest.served_from_cache();
        stale.status = Status::Stale;
        assert_eq!(
            history.with_pace(stale).windows[0].pace,
            None,
            "numbers that are not current are not projected"
        );
    }

    #[test]
    fn a_version_1_history_gains_the_readings_of_its_snapshots() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("history.db");
        let stored = [
            snapshot("org-a", 10.0, "2099-01-01T00:00:00Z", Source::Api),
            snapshot("org-a", 12.0, "2099-01-01T00:05:00Z", Source::Api),
        ];
        {
            // The schema QuotaDial 0.1.0 wrote.
            let connection = Connection::open(&path).expect("opens");
            connection
                .execute_batch(
                    "CREATE TABLE snapshots (
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
                     PRAGMA user_version = 1;",
                )
                .expect("creates");
            for snapshot in &stored {
                connection
                    .execute(
                        "INSERT INTO snapshots
                             (provider, org, account_id, polled_at, fetched_at, status, snapshot)
                         VALUES ('claude', 'org-a', ?1, ?2, ?2, 'ok', ?3)",
                        params![
                            snapshot.account.account_id,
                            timestamp(snapshot.windows[0].fetched_at),
                            serde_json::to_string(snapshot).expect("serializes")
                        ],
                    )
                    .expect("inserts");
            }
        }

        let history = History::open(&path).expect("upgrades");

        let readings = history
            .readings(&account("org-a"), &stored[0].windows[0], None)
            .expect("reads");
        assert_eq!(readings.len(), 2);
        assert_eq!(history.latest().expect("reads"), [stored[1].clone()]);
        let version: i32 = history
            .lock()
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("reads the version");
        assert_eq!(version, SCHEMA_VERSION);
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
        let window = snapshot("org-a", 0.0, "2099-01-01T00:00:00Z", Source::Api).windows[0].clone();
        assert_eq!(
            history
                .readings(&account("org-a"), &window, None)
                .expect("reads")
                .len(),
            1,
            "the readings go with them"
        );
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
