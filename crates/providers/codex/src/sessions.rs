//! The newest rate-limit snapshot in the Codex CLI's session logs, for when ChatGPT's usage
//! endpoint cannot be asked.
//!
//! The Codex CLI writes one `rollout-*.jsonl` file per session under `sessions/YYYY/MM/DD/`. Its
//! first line, `session_meta`, names the ChatGPT account and user that created the session
//! (`creator_account_id`, `creator_user_id`), and every `token_count` event carries the latest
//! `rate_limits { limit_id, primary, secondary, credits, plan_type }`, each window
//! `{ used_percent, window_minutes, resets_at }`. CodexBar stopped reading these files in November
//! 2025 (it asks `codex app-server` instead), so this follows the files the Codex CLI 0.162 and
//! 0.163 wrote on Matt's Mac (D-042).
//!
//! Another account's numbers must never stand in, and a session names only who created it: a
//! session resumed after `codex login` to another account keeps writing to the same file. So a
//! file counts only if the same ChatGPT account and user created it, an event counts only if it
//! was written after the current sign-in (`auth.json`'s modification time), and an event naming
//! another plan than the sign-in's is skipped. Only files written in the last week are read,
//! because no window is longer, and a window that has reset since is left out. A session started
//! more than a week ago can still be written to today, so files are chosen by when they were
//! written, not by their folder's date.
//!
//! Files can be hundreds of megabytes, so each is read backwards from its end, at most 32 MiB of
//! it, and a line too long to be a rate-limit event is skipped without being kept in memory.

use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::ops::ControlFlow;
use std::path::{Path, PathBuf};

use chrono::{DateTime, TimeDelta, Utc};
use quotadial_core::{ExtraUsage, Source, Window};
use serde_json::Value;

use crate::usage::{self, Measured};

/// A `session_meta` line longer than this is not read.
const MAX_META_BYTES: u64 = 1024 * 1024;
/// No window is longer than a week, so older files cannot hold current numbers.
const HORIZON: TimeDelta = TimeDelta::days(7);

/// How much of a file is read, looking for its last rate-limit event.
#[derive(Debug, Clone, Copy)]
struct Limits {
    /// Bytes read at a time, backwards from the end.
    chunk: u64,
    /// How far back from the end to look. A rate-limit event follows every model turn.
    max_scan: u64,
    /// Longer lines are skipped: a `token_count` event is a few hundred bytes.
    max_line: usize,
}

const LIMITS: Limits = Limits {
    chunk: 64 * 1024,
    max_scan: 32 * 1024 * 1024,
    max_line: 256 * 1024,
};

/// What a session log must show before its numbers stand in for an account.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Owner<'a> {
    /// The ChatGPT account (workspace) that must have created the session.
    pub(crate) account_id: &'a str,
    /// The ChatGPT user that must have created it. When the sign-in names no user, the account is
    /// the whole workspace, so any of its users' sessions count.
    pub(crate) user_id: Option<&'a str>,
    /// The sign-in's plan. An event naming another plan is someone else's.
    pub(crate) plan: Option<&'a str>,
    /// When the current sign-in was written. An earlier event may come from another sign-in that
    /// resumed the session, so only later ones count.
    pub(crate) since: Option<DateTime<Utc>>,
}

/// The newest rate-limit numbers in the session logs.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LogReading {
    pub(crate) windows: Vec<Window>,
    pub(crate) extra_usage: Option<ExtraUsage>,
    pub(crate) plan: Option<String>,
    /// When the Codex CLI wrote them.
    pub(crate) at: DateTime<Utc>,
}

/// The newest rate-limit event for `owner` in the sessions under `sessions_dir`, as of `now`.
pub(crate) fn newest(
    sessions_dir: &Path,
    owner: Owner<'_>,
    now: DateTime<Utc>,
) -> Option<LogReading> {
    newest_within(sessions_dir, owner, now, LIMITS)
}

fn newest_within(
    sessions_dir: &Path,
    owner: Owner<'_>,
    now: DateTime<Utc>,
    limits: Limits,
) -> Option<LogReading> {
    let oldest = owner
        .since
        .map_or(now - HORIZON, |since| since.max(now - HORIZON));
    let mut files: Vec<(DateTime<Utc>, PathBuf)> = rollout_files(sessions_dir)
        .into_iter()
        .filter_map(|path| {
            let modified = DateTime::<Utc>::from(fs::metadata(&path).ok()?.modified().ok()?);
            (modified >= oldest).then_some((modified, path))
        })
        .collect();
    files.sort_by_key(|(modified, _)| std::cmp::Reverse(*modified));

    let mut best: Option<(DateTime<Utc>, Value)> = None;
    for (modified, path) in files {
        // A file cannot hold an event newer than its last write.
        if best.as_ref().is_some_and(|(at, _)| modified < *at) {
            break;
        }
        if !created_by(&path, owner) {
            continue;
        }
        if let Some((at, rate_limits)) = last_rate_limits(&path, owner, limits)
            && best.as_ref().is_none_or(|(best_at, _)| at > *best_at)
        {
            best = Some((at, rate_limits));
        }
    }
    let (at, rate_limits) = best?;
    Some(LogReading {
        windows: log_windows(&rate_limits, at, now),
        extra_usage: rate_limits.get("credits").and_then(usage::credits),
        plan: usage::name(&rate_limits, "plan_type"),
        at,
    })
}

/// `sessions/YYYY/MM/DD/rollout-*.jsonl`.
fn rollout_files(sessions_dir: &Path) -> Vec<PathBuf> {
    let children = |dir: &Path| -> Vec<PathBuf> {
        fs::read_dir(dir)
            .map(|entries| {
                entries
                    .filter_map(|entry| Some(entry.ok()?.path()))
                    .collect()
            })
            .unwrap_or_default()
    };
    children(sessions_dir)
        .iter()
        .flat_map(|year| children(year))
        .flat_map(|month| children(&month))
        .flat_map(|day| children(&day))
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("rollout-") && name.ends_with(".jsonl"))
                && path.is_file()
        })
        .collect()
}

/// Whether the file's `session_meta` names `owner` as the session's creator.
fn created_by(path: &Path, owner: Owner<'_>) -> bool {
    let Ok(file) = File::open(path) else {
        return false;
    };
    let mut line = String::new();
    if BufReader::new(file.take(MAX_META_BYTES))
        .read_line(&mut line)
        .is_err()
    {
        return false;
    }
    let Ok(meta) = serde_json::from_str::<Value>(&line) else {
        return false;
    };
    let payload = &meta["payload"];
    meta["type"] == "session_meta"
        && payload["creator_account_id"].as_str() == Some(owner.account_id)
        && owner
            .user_id
            .is_none_or(|user_id| payload["creator_user_id"].as_str() == Some(user_id))
}

/// The last `token_count` event in the file that counts for `owner`, with its time. Events for
/// other limits (`limit_id: "premium"`) and for another plan are skipped, and the search stops at
/// the first event older than the sign-in.
fn last_rate_limits(
    path: &Path,
    owner: Owner<'_>,
    limits: Limits,
) -> Option<(DateTime<Utc>, Value)> {
    lines_backwards(path, limits, |line| {
        let Some((at, rate_limits)) = rate_limit_event(line) else {
            return ControlFlow::Continue(());
        };
        if owner.since.is_some_and(|since| at < since) {
            return ControlFlow::Break(None);
        }
        let plan = rate_limits.get("plan_type").and_then(Value::as_str);
        if let (Some(expected), Some(plan)) = (owner.plan, plan)
            && !expected.eq_ignore_ascii_case(plan)
        {
            return ControlFlow::Continue(());
        }
        ControlFlow::Break(Some((at, rate_limits)))
    })
}

/// Calls `visit` with each line in the last `limits.max_scan` bytes of the file, newest first,
/// until it breaks, and returns what it broke with. Lines longer than `limits.max_line` are
/// skipped.
fn lines_backwards<T>(
    path: &Path,
    limits: Limits,
    mut visit: impl FnMut(&[u8]) -> ControlFlow<Option<T>>,
) -> Option<T> {
    let mut file = File::open(path).ok()?;
    let length = file.metadata().ok()?.len();
    let floor = length.saturating_sub(limits.max_scan);
    let mut end = length;
    // The part already read of a line that starts before `end`, or `None` once that line has
    // grown too long to keep.
    let mut tail: Option<Vec<u8>> = Some(Vec::new());
    while end > floor {
        let start = end.saturating_sub(limits.chunk).max(floor);
        let mut chunk = vec![0; usize::try_from(end - start).ok()?];
        file.seek(SeekFrom::Start(start)).ok()?;
        file.read_exact(&mut chunk).ok()?;
        // Newest first. The first piece ends the tail's line; the last starts before `start`,
        // unless `start` is the file's start.
        let pieces: Vec<&[u8]> = chunk.split(|byte| *byte == b'\n').rev().collect();
        let (newest_piece, older) = pieces.split_first()?;
        let tail_line = tail.and_then(|tail| joined(newest_piece, &tail, limits.max_line));
        let Some((oldest_piece, middle)) = older.split_last() else {
            // No line break in this chunk: the tail's line goes on before it.
            if start == 0 {
                return tail_line
                    .and_then(|line| visit(&line).break_value())
                    .flatten();
            }
            tail = tail_line;
            end = start;
            continue;
        };
        let complete = tail_line
            .iter()
            .map(Vec::as_slice)
            .chain(middle.iter().copied())
            .filter(|line| line.len() <= limits.max_line);
        for line in complete {
            if let ControlFlow::Break(found) = visit(line) {
                return found;
            }
        }
        if start == 0 {
            return (oldest_piece.len() <= limits.max_line)
                .then(|| visit(oldest_piece).break_value())
                .flatten()
                .flatten();
        }
        tail = (oldest_piece.len() <= limits.max_line).then(|| oldest_piece.to_vec());
        end = start;
    }
    None
}

/// `head` followed by `tail`, unless together they are longer than `max`.
fn joined(head: &[u8], tail: &[u8], max: usize) -> Option<Vec<u8>> {
    (head.len() + tail.len() <= max).then(|| [head, tail].concat())
}

fn rate_limit_event(line: &[u8]) -> Option<(DateTime<Utc>, Value)> {
    if !contains(line, b"\"token_count\"") || !contains(line, b"\"rate_limits\"") {
        return None;
    }
    let mut event: Value = serde_json::from_slice(line).ok()?;
    let payload = event.get_mut("payload")?;
    if payload.get("type")?.as_str()? != "token_count" {
        return None;
    }
    let rate_limits = payload.get_mut("rate_limits")?.take();
    let limit_id = rate_limits.get("limit_id").and_then(Value::as_str);
    if !rate_limits.is_object() || limit_id.is_some_and(|id| id != "codex") {
        return None;
    }
    let at = DateTime::parse_from_rfc3339(event.get("timestamp")?.as_str()?)
        .ok()?
        .with_timezone(&Utc);
    Some((at, rate_limits))
}

/// The windows in a log event written at `at`, leaving out any that has reset by `now`.
fn log_windows(rate_limits: &Value, at: DateTime<Utc>, now: DateTime<Utc>) -> Vec<Window> {
    let mut windows: Vec<Window> = ["primary", "secondary"]
        .iter()
        .filter_map(|slot| {
            let window = rate_limits.get(slot)?;
            let resets_at = window
                .get("resets_at")
                .and_then(Value::as_i64)
                .and_then(|at| DateTime::from_timestamp(at, 0));
            if resets_at.is_some_and(|resets_at| resets_at <= now) {
                return None;
            }
            usage::classified(Measured {
                seconds: window.get("window_minutes")?.as_u64()?.checked_mul(60)?,
                used_pct: window.get("used_percent")?.as_f64()?,
                resets_at,
                model: None,
                source: Source::LocalLog,
                fetched_at: at,
            })
        })
        .collect();
    windows.sort_by_key(|window| window.window_seconds);
    windows
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

#[cfg(test)]
pub(crate) mod tests {
    use std::time::SystemTime;

    use quotadial_core::WindowKind;
    use serde_json::json;
    use tempfile::TempDir;

    use super::*;

    /// Dates a file, so a test can make it look recent or old.
    pub(crate) fn set_modified(path: &Path, at: DateTime<Utc>) {
        let file = File::options().write(true).open(path).expect("opens");
        file.set_modified(SystemTime::from(at))
            .expect("sets the time");
    }

    /// A redacted session log from the Codex CLI (D-042).
    pub(crate) const ROLLOUT_FIXTURE: &str =
        include_str!("../../../../fixtures/codex/rollout.jsonl");

    const OWNER: Owner<'static> = Owner {
        account_id: "00000000-0000-0000-0000-000000000002",
        user_id: Some("user-000000000000000000000001"),
        plan: Some("pro"),
        since: None,
    };

    fn ts(rfc3339: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(rfc3339)
            .expect("valid timestamp")
            .with_timezone(&Utc)
    }

    fn now() -> DateTime<Utc> {
        ts("2026-10-23T12:00:00Z")
    }

    /// Writes a session log under `sessions/2026/10/23/` and dates it.
    fn write_log(dir: &TempDir, name: &str, contents: &str, modified: DateTime<Utc>) -> PathBuf {
        let day = dir.path().join("2026").join("10").join("23");
        fs::create_dir_all(&day).expect("creates");
        let path = day.join(name);
        fs::write(&path, contents).expect("writes");
        set_modified(&path, modified);
        path
    }

    #[test]
    fn reads_the_newest_rate_limits_of_the_owners_sessions() {
        let dir = tempfile::tempdir().expect("temp dir");
        write_log(
            &dir,
            "rollout-a.jsonl",
            ROLLOUT_FIXTURE,
            ts("2026-10-23T09:15:02Z"),
        );

        let reading = newest(dir.path(), OWNER, now()).expect("a reading");

        assert_eq!(reading.at, ts("2026-10-23T09:15:00Z"));
        assert_eq!(
            reading.windows,
            vec![Window {
                kind: WindowKind::Weekly,
                label: "Weekly".into(),
                used_pct: 54.0,
                resets_at: Some(ts("2026-10-29T17:09:48Z")),
                window_seconds: 604_800,
                scope_model: None,
                source: Source::LocalLog,
                fetched_at: ts("2026-10-23T09:15:00Z"),
            }]
        );
        assert_eq!(reading.plan.as_deref(), Some("pro"));
        assert_eq!(reading.extra_usage.map(|extra| extra.enabled), Some(true));
    }

    #[test]
    fn another_accounts_sessions_never_stand_in() {
        let dir = tempfile::tempdir().expect("temp dir");
        write_log(
            &dir,
            "rollout-a.jsonl",
            ROLLOUT_FIXTURE,
            ts("2026-10-23T09:15:02Z"),
        );

        let other_account = Owner {
            account_id: "00000000-0000-0000-0000-000000000009",
            ..OWNER
        };
        let other_user = Owner {
            user_id: Some("user-000000000000000000000009"),
            ..OWNER
        };
        assert_eq!(newest(dir.path(), other_account, now()), None);
        assert_eq!(newest(dir.path(), other_user, now()), None);
        let user_unknown = Owner {
            user_id: None,
            ..OWNER
        };
        assert!(newest(dir.path(), user_unknown, now()).is_some());
    }

    #[test]
    fn the_newest_event_wins_across_sessions() {
        let dir = tempfile::tempdir().expect("temp dir");
        write_log(
            &dir,
            "rollout-a.jsonl",
            ROLLOUT_FIXTURE,
            ts("2026-10-23T09:15:02Z"),
        );
        let later = ROLLOUT_FIXTURE
            .replace("2026-10-23T09:15:00.000Z", "2026-10-23T10:00:00.000Z")
            .replace("\"used_percent\":54.0", "\"used_percent\":61.0");
        write_log(&dir, "rollout-b.jsonl", &later, ts("2026-10-23T10:00:02Z"));
        // A session last written before the newest event is not read at all.
        write_log(
            &dir,
            "rollout-c.jsonl",
            ROLLOUT_FIXTURE,
            ts("2026-10-23T08:00:00Z"),
        );

        let reading = newest(dir.path(), OWNER, now()).expect("a reading");

        assert_eq!(reading.at, ts("2026-10-23T10:00:00Z"));
        assert_eq!(reading.windows[0].used_pct, 61.0);
    }

    #[test]
    fn old_files_and_windows_that_have_reset_are_left_out() {
        let dir = tempfile::tempdir().expect("temp dir");
        write_log(
            &dir,
            "rollout-a.jsonl",
            ROLLOUT_FIXTURE,
            ts("2026-10-15T00:00:00Z"),
        );
        assert_eq!(newest(dir.path(), OWNER, now()), None, "older than a week");

        let fresh = tempfile::tempdir().expect("temp dir");
        write_log(
            &fresh,
            "rollout-a.jsonl",
            ROLLOUT_FIXTURE,
            ts("2026-10-23T09:15:02Z"),
        );
        let after_reset = ts("2026-10-29T18:00:00Z");
        set_modified(
            &fresh.path().join("2026/10/23/rollout-a.jsonl"),
            ts("2026-10-29T17:00:00Z"),
        );
        let reading = newest(fresh.path(), OWNER, after_reset).expect("a reading");
        assert!(
            reading.windows.is_empty(),
            "the weekly window reset at 17:09:48"
        );
    }

    #[test]
    fn events_far_from_the_end_of_a_long_file_are_found_across_chunks() {
        let dir = tempfile::tempdir().expect("temp dir");
        let filler = json!({ "timestamp": "2026-10-23T09:20:00.000Z", "type": "response_item",
            "payload": { "type": "message", "content": "x".repeat(1000) } })
        .to_string();
        let mut contents = ROLLOUT_FIXTURE.to_owned();
        for _ in 0..300 {
            contents.push_str(&filler);
            contents.push('\n');
        }
        write_log(
            &dir,
            "rollout-a.jsonl",
            &contents,
            ts("2026-10-23T09:30:00Z"),
        );

        let reading = newest(dir.path(), OWNER, now()).expect("a reading");

        assert_eq!(reading.at, ts("2026-10-23T09:15:00Z"));
    }

    #[test]
    fn events_written_before_the_current_sign_in_never_stand_in() {
        let dir = tempfile::tempdir().expect("temp dir");
        write_log(
            &dir,
            "rollout-a.jsonl",
            ROLLOUT_FIXTURE,
            ts("2026-10-23T09:30:00Z"),
        );
        let since = |at: &str| Owner {
            since: Some(ts(at)),
            ..OWNER
        };

        let reading = newest(dir.path(), since("2026-10-23T09:10:00Z"), now());
        assert_eq!(reading.map(|r| r.at), Some(ts("2026-10-23T09:15:00Z")));
        // Written to after the sign-in, but its last event is from before it.
        assert_eq!(
            newest(dir.path(), since("2026-10-23T09:16:00Z"), now()),
            None
        );
        // Not written to at all since the sign-in.
        assert_eq!(
            newest(dir.path(), since("2026-10-23T09:31:00Z"), now()),
            None
        );
    }

    #[test]
    fn events_for_another_plan_are_skipped() {
        let dir = tempfile::tempdir().expect("temp dir");
        let mixed = ROLLOUT_FIXTURE.replacen("\"plan_type\":\"pro\"", "\"plan_type\":\"plus\"", 1);
        write_log(&dir, "rollout-a.jsonl", &mixed, ts("2026-10-23T09:15:02Z"));

        let plus = Owner {
            plan: Some("plus"),
            ..OWNER
        };
        let reading = newest(dir.path(), plus, now()).expect("the older plus event");
        assert_eq!(reading.at, ts("2026-10-23T09:05:00Z"));
        assert_eq!(reading.windows[0].used_pct, 50.0);

        let team = Owner {
            plan: Some("team"),
            ..OWNER
        };
        assert_eq!(newest(dir.path(), team, now()), None);
        let unknown = Owner {
            plan: None,
            ..OWNER
        };
        assert_eq!(
            newest(dir.path(), unknown, now()).map(|r| r.at),
            Some(ts("2026-10-23T09:15:00Z"))
        );
    }

    #[test]
    fn the_newest_event_is_found_whatever_the_chunk_boundaries() {
        let dir = tempfile::tempdir().expect("temp dir");
        let unix = write_log(
            &dir,
            "rollout-a.jsonl",
            ROLLOUT_FIXTURE,
            ts("2026-10-23T09:15:02Z"),
        );
        let windows = ROLLOUT_FIXTURE.replace('\n', "\r\n");
        let crlf = write_log(
            &dir,
            "rollout-b.jsonl",
            &windows,
            ts("2026-10-23T09:15:02Z"),
        );
        let unterminated = ROLLOUT_FIXTURE.trim_end().to_owned();
        let no_final_newline = write_log(
            &dir,
            "rollout-c.jsonl",
            &unterminated,
            ts("2026-10-23T09:15:02Z"),
        );

        for path in [unix, crlf, no_final_newline] {
            for chunk in 1..=80 {
                let limits = Limits {
                    chunk,
                    max_scan: 1024 * 1024,
                    max_line: 4096,
                };
                let found = last_rate_limits(&path, OWNER, limits).map(|(at, _)| at);
                assert_eq!(
                    found,
                    Some(ts("2026-10-23T09:15:00Z")),
                    "{} with {chunk}-byte chunks",
                    path.display()
                );
            }
        }
    }

    #[test]
    fn long_lines_are_skipped_without_losing_the_event_before_them() {
        let dir = tempfile::tempdir().expect("temp dir");
        let mut contents = ROLLOUT_FIXTURE.to_owned();
        contents.push_str(&format!("{{\"pasted\":\"{}\"}}\n", "x".repeat(10_000)));
        let path = write_log(
            &dir,
            "rollout-a.jsonl",
            &contents,
            ts("2026-10-23T09:30:00Z"),
        );
        let limits = |max_line| Limits {
            chunk: 1024,
            max_scan: 1024 * 1024,
            max_line,
        };

        assert_eq!(
            last_rate_limits(&path, OWNER, limits(2048)).map(|(at, _)| at),
            Some(ts("2026-10-23T09:15:00Z"))
        );
        // A rate-limit event longer than the limit is skipped as well.
        assert_eq!(last_rate_limits(&path, OWNER, limits(100)), None);
    }

    #[test]
    fn events_further_back_than_the_scan_limit_are_not_read() {
        let dir = tempfile::tempdir().expect("temp dir");
        let filler = json!({ "type": "response_item", "payload": { "text": "x".repeat(200) } });
        let mut contents = ROLLOUT_FIXTURE.to_owned();
        for _ in 0..25 {
            contents.push_str(&filler.to_string());
            contents.push('\n');
        }
        let path = write_log(
            &dir,
            "rollout-a.jsonl",
            &contents,
            ts("2026-10-23T09:30:00Z"),
        );
        let limits = |max_scan| Limits {
            chunk: 512,
            max_scan,
            max_line: 4096,
        };

        assert_eq!(last_rate_limits(&path, OWNER, limits(2048)), None);
        assert!(last_rate_limits(&path, OWNER, limits(64 * 1024)).is_some());
    }

    #[test]
    fn files_that_are_not_session_logs_are_ignored() {
        let dir = tempfile::tempdir().expect("temp dir");
        write_log(
            &dir,
            "notes.jsonl",
            ROLLOUT_FIXTURE,
            ts("2026-10-23T09:15:02Z"),
        );
        write_log(&dir, "rollout-empty.jsonl", "", ts("2026-10-23T09:15:02Z"));
        write_log(
            &dir,
            "rollout-damaged.jsonl",
            "{ not json\n{\"type\":\"event_msg\"}\n",
            ts("2026-10-23T09:15:02Z"),
        );

        assert_eq!(newest(dir.path(), OWNER, now()), None);
        assert_eq!(newest(&dir.path().join("missing"), OWNER, now()), None);
    }
}
