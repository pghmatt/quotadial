//! The newest rate-limit snapshot in the Codex CLI's session logs, for when ChatGPT's usage
//! endpoint cannot be asked.
//!
//! The Codex CLI writes one `rollout-*.jsonl` file per session under `sessions/YYYY/MM/DD/`. Its
//! first line, `session_meta`, names the ChatGPT account and user that created the session
//! (`creator_account_id`, `creator_user_id`), and every `token_count` event carries the latest
//! `rate_limits { limit_id, primary, secondary, credits, plan_type }`, each window
//! `{ used_percent, window_minutes, resets_at }`. CodexBar stopped reading these files in November
//! 2025 (it asks `codex app-server` instead), so this follows the files the Codex CLI 0.162 and
//! 0.163 wrote on Matt's Mac (D-040).
//!
//! Only sessions created by the same ChatGPT account and user are read, so another account's
//! numbers never stand in. Only files written in the last week are read, because no window is
//! longer, and a window that has reset since is left out. Files can be hundreds of megabytes, so
//! each is read backwards from its end, at most [`MAX_SCAN_BYTES`] of it.

use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use chrono::{DateTime, TimeDelta, Utc};
use quotadial_core::{ExtraUsage, Source, Window};
use serde_json::Value;

use crate::usage::{self, Measured};

/// How far back from a file's end to look for a rate-limit event. One follows every model turn.
const MAX_SCAN_BYTES: u64 = 32 * 1024 * 1024;
const CHUNK_BYTES: u64 = 64 * 1024;
/// A `session_meta` line longer than this is not read.
const MAX_META_BYTES: u64 = 1024 * 1024;
/// No window is longer than a week, so older files cannot hold current numbers.
const HORIZON: TimeDelta = TimeDelta::days(7);

/// Who created a session: the ChatGPT account (workspace) and, when known, the user.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Owner<'a> {
    pub(crate) account_id: &'a str,
    pub(crate) user_id: Option<&'a str>,
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

/// The newest rate-limit event in `owner`'s sessions under `sessions_dir`, as of `now`.
pub(crate) fn newest(
    sessions_dir: &Path,
    owner: Owner<'_>,
    now: DateTime<Utc>,
) -> Option<LogReading> {
    let mut files: Vec<(DateTime<Utc>, PathBuf)> = rollout_files(sessions_dir)
        .into_iter()
        .filter_map(|path| {
            let modified = DateTime::<Utc>::from(fs::metadata(&path).ok()?.modified().ok()?);
            (modified >= now - HORIZON).then_some((modified, path))
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
        if let Some((at, rate_limits)) = last_rate_limits(&path)
            && best.as_ref().is_none_or(|(best_at, _)| at > *best_at)
        {
            best = Some((at, rate_limits));
        }
    }
    let (at, rate_limits) = best?;
    Some(LogReading {
        windows: log_windows(&rate_limits, at, now),
        extra_usage: rate_limits.get("credits").and_then(usage::credits),
        plan: usage::text(&rate_limits, "plan_type"),
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

/// The last `token_count` event in the file with rate limits for the Codex limit itself, with its
/// time. Events for other limits (`limit_id: "premium"`) are skipped.
fn last_rate_limits(path: &Path) -> Option<(DateTime<Utc>, Value)> {
    let mut file = File::open(path).ok()?;
    let length = file.metadata().ok()?.len();
    let floor = length.saturating_sub(MAX_SCAN_BYTES);
    let mut end = length;
    // The start of a line whose beginning lies before the part read so far.
    let mut carry: Vec<u8> = Vec::new();
    while end > floor {
        let start = end.saturating_sub(CHUNK_BYTES).max(floor);
        let mut buffer = vec![0; usize::try_from(end - start).ok()?];
        file.seek(SeekFrom::Start(start)).ok()?;
        file.read_exact(&mut buffer).ok()?;
        buffer.extend_from_slice(&carry);
        let mut lines = buffer.split(|byte| *byte == b'\n');
        // Unless this is the file's start, the first piece continues a line from before `start`.
        let first = if start > 0 { lines.next() } else { None };
        let complete: Vec<&[u8]> = lines.collect();
        if let Some(event) = complete
            .iter()
            .rev()
            .find_map(|line| rate_limit_event(line))
        {
            return Some(event);
        }
        carry = first.map(<[u8]>::to_vec).unwrap_or_default();
        end = start;
    }
    None
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

    /// A redacted session log from the Codex CLI (D-040).
    pub(crate) const ROLLOUT_FIXTURE: &str =
        include_str!("../../../../fixtures/codex/rollout.jsonl");

    const OWNER: Owner<'static> = Owner {
        account_id: "00000000-0000-0000-0000-000000000002",
        user_id: Some("user-000000000000000000000001"),
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
