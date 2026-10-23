//! Alert rules and their firings, kept in the history database (schema 3, D-093).
//!
//! A rule is kept as the words it was written in and what the grammar read from them. Checking
//! the rules against a report records each occurrence once: a firing is keyed by its rule, its
//! account, its window and the time that makes it once, such as the window's reset, so a restart, a
//! sleep or a clock change neither repeats it nor loses it. Two times count as one when they are
//! within [`SAME_TIME`] of each other, because a provider's reset time moves by a second or less
//! between fetches.
//!
//! A new database starts with the two rules that carry M3's notifications over: a window passing
//! 90% and an account needing to sign in again. In a database from an older QuotaDial they begin
//! quiet: their first check records what already holds without telling, since M3 told it already.

use chrono::{DateTime, TimeDelta, Utc};
use quotadial_core::{
    Account, Condition, Occurrence, ProFeature, Rule, Snapshot, StatusReport, Window, tier,
};
use rusqlite::{Connection, OptionalExtension, Row, params};
use serde::{Deserialize, Serialize};

use crate::deliveries::Delivery;
use crate::history::{History, HistoryError, timestamp};

/// Two times of one occurrence closer than this are the same time.
const SAME_TIME: TimeDelta = TimeDelta::minutes(10);
/// The rules a new database starts with, as written.
const STARTING_RULES: [&str; 2] = [
    "warn me when a window passes 90%",
    "tell me when an account needs to sign in again",
];

/// A rule as kept: the words it was written in, whether it is on, and what they say.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoredRule {
    pub id: i64,
    /// The words it was written in.
    pub written: String,
    pub enabled: bool,
    #[serde(flatten)]
    pub rule: Rule,
    pub created_at: DateTime<Utc>,
}

/// One time a rule's condition held, and what to tell.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Firing {
    pub id: i64,
    /// The rule's id.
    pub rule: i64,
    pub provider: String,
    pub account_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub org: Option<String>,
    /// The account's display name.
    pub account: String,
    /// The window's label; left out for an account's sign-in.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub used_pct: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resets_at: Option<DateTime<Utc>>,
    /// What a notification's title says.
    pub title: String,
    /// What a notification's text says.
    pub body: String,
    pub fired_at: DateTime<Utc>,
    /// Where it was sent besides the app's notifications, or why not (D-094).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deliveries: Vec<Delivery>,
}

/// The alert rules and what they told lately, as `GET /v1/alerts` and the MCP tool `list_alerts`
/// give them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Alerts {
    /// Every rule, oldest first.
    pub rules: Vec<StoredRule>,
    /// What the rules told in the last week, at most the newest [`RECENT_FIRINGS`], oldest first.
    pub firings: Vec<Firing>,
}

/// The most firings [`History::alerts`] gives.
pub const RECENT_FIRINGS: usize = 100;

/// Creates the alert tables, and in a database without them, the starting rules: quiet ones in a
/// database an older QuotaDial wrote (`version` above 0).
pub(crate) fn upgrade(connection: &Connection, version: i32) -> Result<(), HistoryError> {
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS rules (
             id INTEGER PRIMARY KEY,
             written TEXT NOT NULL,
             rule TEXT NOT NULL,
             enabled INTEGER NOT NULL,
             quiet INTEGER NOT NULL,
             created_at TEXT NOT NULL
         );
         CREATE TABLE IF NOT EXISTS firings (
             id INTEGER PRIMARY KEY,
             rule_id INTEGER NOT NULL,
             provider TEXT NOT NULL,
             org TEXT NOT NULL,
             account_id TEXT NOT NULL,
             display_name TEXT NOT NULL,
             label TEXT NOT NULL,
             scope_model TEXT NOT NULL,
             instance TEXT,
             used_pct REAL,
             resets_at TEXT,
             title TEXT NOT NULL,
             body TEXT NOT NULL,
             quiet INTEGER NOT NULL,
             fired_at TEXT NOT NULL
         );
         CREATE INDEX IF NOT EXISTS firings_by_occurrence
             ON firings (rule_id, provider, org, account_id, label, scope_model);
         CREATE INDEX IF NOT EXISTS firings_by_time ON firings (fired_at);
         CREATE TABLE IF NOT EXISTS deliveries (
             firing_id INTEGER NOT NULL,
             webhook TEXT NOT NULL,
             host TEXT NOT NULL,
             sent INTEGER NOT NULL,
             error TEXT,
             attempts INTEGER NOT NULL,
             last_at TEXT NOT NULL,
             PRIMARY KEY (firing_id, webhook)
         );",
    )?;
    if version < 3 {
        for written in STARTING_RULES {
            let rule =
                Rule::parse(written).map_err(|error| HistoryError::new(error.to_string()))?;
            insert_rule(connection, written, &rule, version > 0, Utc::now())?;
        }
    }
    Ok(())
}

impl History {
    /// The rules and what they told in the week before `now`.
    ///
    /// # Errors
    ///
    /// When the database cannot be read.
    pub fn alerts(&self, now: DateTime<Utc>) -> Result<Alerts, HistoryError> {
        let mut firings = self.firings(0, now - TimeDelta::days(7))?;
        firings.drain(..firings.len().saturating_sub(RECENT_FIRINGS));
        Ok(Alerts {
            rules: self.rules()?,
            firings,
        })
    }

    /// Every rule, oldest first.
    ///
    /// # Errors
    ///
    /// When the database cannot be read.
    pub fn rules(&self) -> Result<Vec<StoredRule>, HistoryError> {
        Ok(self
            .rules_with_quiet()?
            .into_iter()
            .map(|(rule, _)| rule)
            .collect())
    }

    /// Reads `written` as a rule and keeps it, on. Its condition is told the next time it holds,
    /// already holding or not.
    ///
    /// # Errors
    ///
    /// [`AddError::Unread`] when the grammar cannot read it, with the forms it reads, or
    /// [`AddError::History`] when the database cannot be written.
    pub fn add_rule(&self, written: &str, now: DateTime<Utc>) -> Result<StoredRule, AddError> {
        let rule = Rule::parse(written).map_err(|error| AddError::Unread(error.to_string()))?;
        let written = written.trim();
        let id = insert_rule(&self.lock(), written, &rule, false, now)?;
        Ok(StoredRule {
            id,
            written: written.to_owned(),
            enabled: true,
            rule,
            created_at: now,
        })
    }

    /// Removes the rule `id`. Its firings stay in the history. Returns whether there was one.
    ///
    /// # Errors
    ///
    /// When the database cannot be written.
    pub fn remove_rule(&self, id: i64) -> Result<bool, HistoryError> {
        Ok(self
            .lock()
            .execute("DELETE FROM rules WHERE id = ?1", params![id])?
            > 0)
    }

    /// Turns the rule `id` on or off, and returns it, or `None` when there is none.
    ///
    /// # Errors
    ///
    /// When the database cannot be written.
    pub fn set_rule_enabled(
        &self,
        id: i64,
        enabled: bool,
    ) -> Result<Option<StoredRule>, HistoryError> {
        self.lock().execute(
            "UPDATE rules SET enabled = ?2 WHERE id = ?1",
            params![id, enabled],
        )?;
        Ok(self.rules()?.into_iter().find(|rule| rule.id == id))
    }

    /// The firings told since `after` (a firing id) and at or after `since`, oldest first. Quiet
    /// ones, which only record what already held, are left out.
    ///
    /// # Errors
    ///
    /// When the database cannot be read.
    pub fn firings(&self, after: i64, since: DateTime<Utc>) -> Result<Vec<Firing>, HistoryError> {
        let connection = self.lock();
        let mut statement = connection.prepare_cached(
            "SELECT id, rule_id, provider, org, account_id, display_name, label, used_pct,
                    resets_at, title, body, fired_at
             FROM firings WHERE id > ?1 AND fired_at >= ?2 AND quiet = 0 ORDER BY id",
        )?;
        let rows = statement.query_map(params![after, timestamp(since)], firing)?;
        let mut firings: Vec<Firing> = rows.collect::<Result<_, _>>()?;
        drop(statement);
        drop(connection);
        for firing in &mut firings {
            firing.deliveries = self.deliveries_of(firing.id)?;
        }
        Ok(firings)
    }

    /// The newest firing's id, or 0.
    ///
    /// # Errors
    ///
    /// When the database cannot be read.
    pub fn last_firing(&self) -> Result<i64, HistoryError> {
        Ok(self
            .lock()
            .query_row("SELECT COALESCE(MAX(id), 0) FROM firings", [], |row| {
                row.get(0)
            })?)
    }

    /// Checks every rule that is on against `report` at `now`, and records each occurrence not
    /// recorded before. Returns the new firings to tell. A quiet rule's first check records what
    /// already holds without telling it, and then the rule speaks like any other.
    ///
    /// # Errors
    ///
    /// When the database cannot be read or written.
    pub fn check(
        &self,
        report: &StatusReport,
        now: DateTime<Utc>,
    ) -> Result<Vec<Firing>, HistoryError> {
        let mut told = Vec::new();
        for (stored, quiet) in self.rules_with_quiet()? {
            if !stored.enabled || !allowed(&stored.rule.condition) {
                // The quiet start stands for what held at the upgrade. A rule that was off then
                // records nothing, and speaks up about what holds when it is turned on, as M3's
                // switch did.
                if quiet {
                    self.end_quiet(stored.id)?;
                }
                continue;
            }
            for snapshot in &report.snapshots {
                let account = &snapshot.account;
                let occurrences = stored.rule.occurrences(
                    snapshot,
                    |window| self.last_reset(account, window).ok().flatten(),
                    now,
                );
                for occurrence in occurrences {
                    if let Some(firing) = self.fire(&stored, snapshot, &occurrence, quiet, now)?
                        && !quiet
                    {
                        told.push(firing);
                    }
                }
            }
            if quiet {
                self.end_quiet(stored.id)?;
            }
        }
        Ok(told)
    }

    fn end_quiet(&self, id: i64) -> Result<(), HistoryError> {
        self.lock()
            .execute("UPDATE rules SET quiet = 0 WHERE id = ?1", params![id])?;
        Ok(())
    }

    fn rules_with_quiet(&self) -> Result<Vec<(StoredRule, bool)>, HistoryError> {
        let connection = self.lock();
        let mut statement = connection.prepare_cached(
            "SELECT id, written, rule, enabled, quiet, created_at FROM rules ORDER BY id",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, bool>(3)?,
                row.get::<_, bool>(4)?,
                row.get::<_, String>(5)?,
            ))
        })?;
        let mut rules = Vec::new();
        for row in rows {
            let (id, written, rule, enabled, quiet, created_at) = row?;
            // A rule this version cannot read, from a newer one, is left alone.
            if let Ok(rule) = serde_json::from_str(&rule) {
                rules.push((
                    StoredRule {
                        id,
                        written,
                        enabled,
                        rule,
                        created_at: parse_time(&created_at).unwrap_or_default(),
                    },
                    quiet,
                ));
            }
        }
        Ok(rules)
    }

    /// When `account`'s `window` was to reset, as its reading before this one said.
    fn last_reset(
        &self,
        account: &Account,
        window: &Window,
    ) -> Result<Option<DateTime<Utc>>, HistoryError> {
        let resets_at: Option<Option<String>> = self
            .lock()
            .query_row(
                "SELECT resets_at FROM readings
                 WHERE provider = ?1 AND org = ?2 AND account_id = ?3 AND label = ?4
                     AND scope_model = ?5 AND fetched_at < ?6
                 ORDER BY fetched_at DESC LIMIT 1",
                params![
                    account.provider,
                    account.org.as_deref().unwrap_or_default(),
                    account.account_id,
                    window.label,
                    window.scope_model.as_deref().unwrap_or_default(),
                    timestamp(window.fetched_at),
                ],
                |row| row.get(0),
            )
            .optional()?;
        Ok(resets_at.flatten().as_deref().and_then(parse_time))
    }

    /// Records `occurrence` of `stored` for `snapshot`'s account, unless it was recorded before.
    /// Returns the new firing.
    fn fire(
        &self,
        stored: &StoredRule,
        snapshot: &Snapshot,
        occurrence: &Occurrence,
        quiet: bool,
        now: DateTime<Utc>,
    ) -> Result<Option<Firing>, HistoryError> {
        let account = &snapshot.account;
        let org = account.org.as_deref().unwrap_or_default();
        let label = occurrence.label.as_deref().unwrap_or_default();
        let scope_model = occurrence.scope_model.as_deref().unwrap_or_default();
        let connection = self.lock();
        let earlier: Vec<Option<String>> = {
            let mut statement = connection.prepare_cached(
                "SELECT instance FROM firings
                 WHERE rule_id = ?1 AND provider = ?2 AND org = ?3 AND account_id = ?4
                     AND label = ?5 AND scope_model = ?6",
            )?;
            let rows = statement.query_map(
                params![
                    stored.id,
                    account.provider,
                    org,
                    account.account_id,
                    label,
                    scope_model
                ],
                |row| row.get(0),
            )?;
            rows.collect::<Result<_, _>>()?
        };
        let same = |instance: &Option<String>| match (
            instance.as_deref().and_then(parse_time),
            occurrence.instance,
        ) {
            (Some(earlier), Some(now)) => (earlier - now).abs() < SAME_TIME,
            (None, None) => true,
            _ => false,
        };
        if earlier.iter().any(same) {
            return Ok(None);
        }
        let (title, body) = stored.rule.message(account, occurrence, now);
        connection.execute(
            "INSERT INTO firings
                 (rule_id, provider, org, account_id, display_name, label, scope_model, instance,
                  used_pct, resets_at, title, body, quiet, fired_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                stored.id,
                account.provider,
                org,
                account.account_id,
                account.display_name,
                label,
                scope_model,
                occurrence.instance.map(timestamp),
                occurrence.used_pct,
                occurrence.resets_at.map(timestamp),
                title,
                body,
                quiet,
                timestamp(now),
            ],
        )?;
        Ok(Some(Firing {
            id: connection.last_insert_rowid(),
            rule: stored.id,
            provider: account.provider.clone(),
            account_id: account.account_id.clone(),
            org: account.org.clone(),
            account: account.display_name.clone(),
            window: occurrence.label.clone(),
            used_pct: occurrence.used_pct,
            resets_at: occurrence.resets_at,
            title,
            body,
            fired_at: now,
            deliveries: Vec::new(),
        }))
    }
}

/// Why a rule was not added.
#[derive(Debug)]
pub enum AddError {
    /// The grammar could not read it; the message names the forms it reads.
    Unread(String),
    /// The database could not be written.
    History(HistoryError),
}

impl std::fmt::Display for AddError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unread(message) => formatter.write_str(message),
            Self::History(error) => write!(formatter, "the rule could not be saved: {error}"),
        }
    }
}

impl std::error::Error for AddError {}

impl From<HistoryError> for AddError {
    fn from(error: HistoryError) -> Self {
        Self::History(error)
    }
}

/// Whether this copy may check rules with `condition` (D-090).
fn allowed(condition: &Condition) -> bool {
    match condition {
        Condition::Threshold { .. } | Condition::Reset | Condition::SignIn => true,
        Condition::RunsOut | Condition::BeforeReset { .. } => tier().allows(ProFeature::AlertRules),
    }
}

fn insert_rule(
    connection: &Connection,
    written: &str,
    rule: &Rule,
    quiet: bool,
    now: DateTime<Utc>,
) -> Result<i64, HistoryError> {
    let json = serde_json::to_string(rule)
        .map_err(|error| HistoryError::new(format!("the rule could not be saved: {error}")))?;
    connection.execute(
        "INSERT INTO rules (written, rule, enabled, quiet, created_at)
         VALUES (?1, ?2, 1, ?3, ?4)",
        params![written, json, quiet, timestamp(now)],
    )?;
    Ok(connection.last_insert_rowid())
}

pub(crate) fn firing(row: &Row<'_>) -> rusqlite::Result<Firing> {
    let org: String = row.get(3)?;
    let label: String = row.get(6)?;
    Ok(Firing {
        id: row.get(0)?,
        rule: row.get(1)?,
        provider: row.get(2)?,
        org: (!org.is_empty()).then_some(org),
        account_id: row.get(4)?,
        account: row.get(5)?,
        window: (!label.is_empty()).then_some(label),
        used_pct: row.get(7)?,
        resets_at: row
            .get::<_, Option<String>>(8)?
            .as_deref()
            .and_then(parse_time),
        title: row.get(9)?,
        body: row.get(10)?,
        fired_at: parse_time(&row.get::<_, String>(11)?).unwrap_or_default(),
        deliveries: Vec::new(),
    })
}

pub(crate) fn parse_time(text: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(text)
        .ok()
        .map(|time| time.with_timezone(&Utc))
}

#[cfg(test)]
mod tests {
    use quotadial_core::{Source, Status, WindowKind};

    use super::*;

    fn at(minutes: i64) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2099-01-01T00:00:00Z")
            .expect("valid timestamp")
            .with_timezone(&Utc)
            + TimeDelta::minutes(minutes)
    }

    fn account() -> Account {
        Account {
            provider: "claude".into(),
            account_id: "user-1".into(),
            display_name: "user@example.com".into(),
            plan: None,
            org: Some("org-1".into()),
            sign_in: None,
        }
    }

    /// A report of one session window: `used_pct` read at minute `fetched`, resetting at minute
    /// `resets`.
    fn report(used_pct: f64, fetched: i64, resets: i64) -> StatusReport {
        StatusReport {
            snapshots: vec![Snapshot {
                account: account(),
                windows: vec![Window {
                    kind: WindowKind::Session,
                    label: "Session".into(),
                    used_pct,
                    resets_at: Some(at(resets)),
                    window_seconds: 18_000,
                    scope_model: None,
                    source: Source::Api,
                    fetched_at: at(fetched),
                    unconfirmed: false,
                    pace: None,
                }],
                extra_usage: None,
                status: Status::Ok,
                error: None,
            }],
            errors: Vec::new(),
        }
    }

    fn open() -> (tempfile::TempDir, History) {
        let dir = tempfile::tempdir().expect("temp dir");
        let history = History::open(&dir.path().join("history.db")).expect("opens");
        (dir, history)
    }

    /// A history with only `written` as a rule.
    fn with_rule(written: &str) -> (tempfile::TempDir, History) {
        let (dir, history) = open();
        for rule in history.rules().expect("reads") {
            history.remove_rule(rule.id).expect("removes");
        }
        history.add_rule(written, at(0)).expect("adds");
        (dir, history)
    }

    fn told(history: &History, report: &StatusReport, now: i64) -> Vec<String> {
        history
            .check(report, at(now))
            .expect("checks")
            .into_iter()
            .map(|firing| firing.title)
            .collect()
    }

    #[test]
    fn a_rule_is_told_once_per_window_however_often_it_is_checked() {
        let (_dir, history) = with_rule("warn me at 80%");

        assert!(told(&history, &report(79.0, 0, 300), 0).is_empty());
        assert_eq!(
            told(&history, &report(85.0, 5, 300), 5),
            ["Claude Session passed 80%"]
        );
        assert!(
            told(&history, &report(85.0, 5, 300), 6).is_empty(),
            "checked again"
        );
        // The same reset, read a second later, as providers' reset times drift.
        let mut drifted = report(95.0, 10, 300);
        drifted.snapshots[0].windows[0].resets_at = Some(at(300) + TimeDelta::seconds(1));
        assert!(told(&history, &drifted, 10).is_empty(), "the same window");
        assert_eq!(
            told(&history, &report(81.0, 310, 600), 310),
            ["Claude Session passed 80%"],
            "the next window"
        );
    }

    #[test]
    fn a_clock_that_jumps_back_or_forward_repeats_nothing() {
        let (_dir, history) = with_rule("tell me 30 minutes before the session resets");
        let session = report(40.0, 0, 300);

        assert!(told(&history, &session, 0).is_empty(), "5 hours before");
        // The clock jumps forward into the half hour before the reset: told, early, once.
        assert_eq!(
            told(&history, &session, 280),
            ["Claude Session resets soon"]
        );
        // Back again, then forward again: nothing more for this window.
        assert!(told(&history, &session, 0).is_empty());
        assert!(told(&history, &session, 290).is_empty());
    }

    #[test]
    fn a_sleep_across_a_reset_tells_the_reset_once_on_waking() {
        let (_dir, history) = with_rule("tell me when the session resets");
        let before = report(60.0, 0, 300);
        history
            .record(&before.snapshots[0], at(0))
            .expect("records");
        assert!(told(&history, &before, 0).is_empty());

        // Twelve hours asleep, past the reset at minute 300, into a new session.
        let woke = report(5.0, 720, 900);
        history
            .record(&woke.snapshots[0], at(720))
            .expect("records");

        assert_eq!(told(&history, &woke, 720), ["Claude Session has reset"]);
        assert!(told(&history, &woke, 721).is_empty());
        let later = report(9.0, 730, 900);
        history
            .record(&later.snapshots[0], at(730))
            .expect("records");
        assert!(
            told(&history, &later, 730).is_empty(),
            "the reading before this one is already after the reset"
        );
    }

    #[test]
    fn a_restart_between_a_check_and_its_telling_tells_it_once() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("history.db");
        let history = History::open(&path).expect("opens");
        let seen = history.last_firing().expect("reads");
        history.add_rule("warn me at 50%", at(0)).expect("adds");
        let fired = history.check(&report(55.0, 0, 300), at(0)).expect("checks");
        assert_eq!(fired.len(), 1);
        drop(history);

        // The app comes back and tells what it had not told yet.
        let reopened = History::open(&path).expect("reopens");
        let untold = reopened.firings(seen, at(-60)).expect("reads");
        assert_eq!(untold, fired);
        assert!(
            reopened
                .check(&report(56.0, 5, 300), at(5))
                .expect("checks")
                .is_empty(),
            "not recorded twice"
        );
    }

    #[test]
    fn a_new_history_starts_with_m3_s_two_notifications_as_rules() {
        let (_dir, history) = open();

        let rules = history.rules().expect("reads");
        assert_eq!(
            rules
                .iter()
                .map(|rule| (rule.written.as_str(), rule.enabled))
                .collect::<Vec<_>>(),
            [
                ("warn me when a window passes 90%", true),
                ("tell me when an account needs to sign in again", true),
            ]
        );
        assert_eq!(
            told(&history, &report(95.0, 0, 300), 0),
            ["Claude Session passed 90%"],
            "a new history tells what holds at once"
        );
    }

    #[test]
    fn an_older_history_s_starting_rules_first_record_what_m3_already_told() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("history.db");
        drop(History::open(&path).expect("opens"));
        {
            // As QuotaDial 0.1.0 left it: schema 1, with no alert tables.
            let connection = Connection::open(&path).expect("opens");
            connection
                .execute_batch(
                    "DROP TABLE rules; DROP TABLE firings; DROP TABLE readings;
                     PRAGMA user_version = 1;",
                )
                .expect("goes back to schema 1");
        }
        let history = History::open(&path).expect("upgrades");
        let mut expired = report(40.0, 0, 300);
        expired.snapshots[0].status = Status::AuthRequired;
        expired.snapshots[0].error = Some("Run `claude` to sign in again.".into());

        assert!(
            told(&history, &report(95.0, 0, 300), 0).is_empty(),
            "M3 told this 90% already"
        );
        assert_eq!(
            told(&history, &expired, 1),
            ["Claude needs you to sign in again"],
            "a sign-in that expired after the first check is new"
        );
        assert_eq!(
            told(&history, &report(91.0, 310, 600), 310),
            ["Claude Session passed 90%"],
            "after their first check they speak"
        );
    }

    #[test]
    fn a_starting_rule_that_was_off_at_the_upgrade_speaks_up_when_turned_on() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("history.db");
        drop(History::open(&path).expect("opens"));
        Connection::open(&path)
            .expect("opens")
            .execute_batch("DROP TABLE rules; DROP TABLE firings; PRAGMA user_version = 2;")
            .expect("goes back to schema 2");
        let history = History::open(&path).expect("upgrades");
        let high = history.rules().expect("reads").remove(0);
        // M3's 90% switch was off, so the carried-over rule starts off.
        history.set_rule_enabled(high.id, false).expect("writes");
        assert!(told(&history, &report(95.0, 0, 300), 0).is_empty(), "off");

        history.set_rule_enabled(high.id, true).expect("writes");

        assert_eq!(
            told(&history, &report(95.0, 1, 300), 1),
            ["Claude Session passed 90%"]
        );
    }

    #[test]
    fn processes_opening_an_older_history_at_once_upgrade_it_once() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("history.db");
        drop(History::open(&path).expect("opens"));
        Connection::open(&path)
            .expect("opens")
            .execute_batch("DROP TABLE rules; DROP TABLE firings; PRAGMA user_version = 2;")
            .expect("goes back to schema 2");
        let start = std::sync::Barrier::new(4);

        std::thread::scope(|scope| {
            for _ in 0..4 {
                scope.spawn(|| {
                    start.wait();
                    History::open(&path).expect("opens")
                });
            }
        });

        assert_eq!(
            History::open(&path)
                .expect("opens")
                .rules()
                .expect("reads")
                .len(),
            2
        );
    }

    #[test]
    fn rules_can_be_added_turned_off_and_removed() {
        let (_dir, history) = with_rule("warn me at 50%");
        let rule = history.rules().expect("reads").remove(0);
        assert_eq!(rule.rule.condition, Condition::Threshold { pct: 50.0 });

        let unread = history
            .add_rule("make it stop", at(0))
            .expect_err("not a rule");
        assert!(matches!(&unread, AddError::Unread(message) if message.contains("warn me at 80%")));

        let off = history
            .set_rule_enabled(rule.id, false)
            .expect("writes")
            .expect("there is one");
        assert!(!off.enabled);
        assert!(told(&history, &report(60.0, 0, 300), 0).is_empty(), "off");
        history.set_rule_enabled(rule.id, true).expect("writes");
        assert_eq!(
            told(&history, &report(60.0, 5, 300), 5).len(),
            1,
            "on again"
        );

        assert!(history.remove_rule(rule.id).expect("removes"));
        assert!(
            !history.remove_rule(rule.id).expect("removes"),
            "already gone"
        );
        assert!(history.rules().expect("reads").is_empty());
        assert_eq!(
            history.firings(0, at(-60)).expect("reads").len(),
            1,
            "its firings stay in the history"
        );
        assert!(
            history
                .set_rule_enabled(rule.id, true)
                .expect("writes")
                .is_none()
        );
    }

    #[test]
    fn firings_go_with_the_rest_of_history_after_thirty_days() {
        let (_dir, history) = with_rule("warn me at 50%");
        history.check(&report(60.0, 0, 300), at(0)).expect("checks");

        history.prune(at(31 * 24 * 60)).expect("prunes");

        assert!(history.firings(0, at(-60)).expect("reads").is_empty());
    }
}
