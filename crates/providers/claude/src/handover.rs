//! What QuotaDial keeps of what Claude Code's status line hands over, and nothing else (the
//! manager's rule 5 on #73, D-145): the five-hour and the weekly window's share used and reset, and
//! QuotaDial's own time.
//!
//! Claude Code pipes its session's state to the command as JSON, whose fields its docs list
//! ("Customize your status line", code.claude.com/docs/en/statusline, read on 28 October 2026):
//! `rate_limits.five_hour` and `rate_limits.seven_day`, each with `used_percentage`, "Percentage of
//! the 5-hour or 7-day rate limit consumed, from 0 to 100", and `resets_at`, "Unix epoch seconds
//! when the 5-hour or 7-day rate limit window resets". `rate_limits` "appears only for claude.ai Pro
//! and Max subscribers ... and only after the first API response in the session", and each window
//! "may be independently absent". The rest, such as `cwd`, `session_id`, `transcript_path`,
//! `model`, `workspace`, `context_window`, `cost` and `rate_limits.spend_limit`, is skipped and
//! never kept.
//!
//! QuotaDial's own file holds the kept fields under the same names, with `written_at`, so one reader
//! takes both and checks every value each time: a share from 0 to 100, or the window is left out,
//! never clamped; a reset read as the docs give it, whole seconds since 1970 (a fraction dropped),
//! kept when it reads as a time at most a year after the handover, so a number in another unit can
//! never show as a reset centuries away. Reading it as seconds is the builder's reading of the page
//! until a real run on Matt's Mac (#51) confirms it (D-145).

use std::collections::BTreeMap;

use chrono::{DateTime, SecondsFormat, TimeDelta, Utc};
use serde_json::{Map, Value};

/// The most of a payload or a file QuotaDial reads. Claude Code's payload runs to a few kilobytes;
/// a larger one is taken as one QuotaDial cannot read.
pub(crate) const MOST: usize = 1024 * 1024;

/// The windows kept, by the names Claude Code's status line gives them, in the order QuotaDial
/// shows them: the five-hour window, then the weekly one.
pub(crate) const WINDOWS: [&str; 2] = [FIVE_HOUR, SEVEN_DAY];

/// The five-hour window's name.
pub(crate) const FIVE_HOUR: &str = "five_hour";

/// The weekly window's name.
pub(crate) const SEVEN_DAY: &str = "seven_day";

/// The latest a reset may be after the handover: a year. A week is the longest window Claude has.
const MOST_RESET_AHEAD: TimeDelta = TimeDelta::days(366);

// The names of the fields kept, as Claude Code's docs write them, and QuotaDial's own time.
const WRITTEN_AT: &str = "written_at";
const RATE_LIMITS: &str = "rate_limits";
const USED_PERCENTAGE: &str = "used_percentage";
const RESETS_AT: &str = "resets_at";

/// What one run of the status line handed over, as QuotaDial keeps it.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Handover {
    /// When QuotaDial's command took it, which its file records.
    pub(crate) written_at: DateTime<Utc>,
    /// Each window handed over with a share, by its name, one of [`WINDOWS`].
    pub(crate) windows: BTreeMap<&'static str, Limit>,
}

/// One window, as Claude Code's status line hands it over.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Limit {
    /// The share used, from 0 to 100.
    pub(crate) used_percentage: f64,
    /// When it resets.
    pub(crate) resets_at: Option<DateTime<Utc>>,
}

/// What the status line's JSON, `payload`, hands over, taken at `now`, or `None` when it holds no
/// window to keep: no `rate_limits`, as for a sign-in with an API key or before the session's first
/// answer, or neither window with a share from 0 to 100.
pub(crate) fn from_status_line(payload: &[u8], now: DateTime<Utc>) -> Option<Handover> {
    let handover = read(&object(payload)?, now);
    (!handover.windows.is_empty()).then_some(handover)
}

/// The file QuotaDial keeps for `handover`: `written_at` and each window under Claude Code's names,
/// as JSON a person can read.
pub(crate) fn to_file(handover: &Handover) -> Vec<u8> {
    let windows: Map<String, Value> = handover
        .windows
        .iter()
        .map(|(name, limit)| {
            let mut fields = Map::new();
            fields.insert(USED_PERCENTAGE.into(), limit.used_percentage.into());
            if let Some(at) = limit.resets_at {
                fields.insert(RESETS_AT.into(), at.timestamp().into());
            }
            ((*name).to_owned(), Value::Object(fields))
        })
        .collect();
    let mut fields = Map::new();
    fields.insert(WRITTEN_AT.into(), timestamp(handover.written_at).into());
    fields.insert(RATE_LIMITS.into(), Value::Object(windows));
    format!("{:#}\n", Value::Object(fields)).into_bytes()
}

/// `bytes` as one JSON object, or `None` when it is larger than [`MOST`] or anything else.
fn object(bytes: &[u8]) -> Option<Map<String, Value>> {
    if bytes.len() > MOST {
        return None;
    }
    match serde_json::from_slice(bytes).ok()? {
        Value::Object(fields) => Some(fields),
        _ => None,
    }
}

/// The kept windows of `fields`, taken at `written_at`, each checked: one of the wrong kind or out of
/// range is left out.
fn read(fields: &Map<String, Value>, written_at: DateTime<Utc>) -> Handover {
    let windows = fields
        .get(RATE_LIMITS)
        .and_then(Value::as_object)
        .map(|limits| {
            WINDOWS
                .iter()
                .filter_map(|name| {
                    let window = limits.get(*name)?.as_object()?;
                    Some((*name, limit(window, written_at)?))
                })
                .collect()
        })
        .unwrap_or_default();
    Handover {
        written_at,
        windows,
    }
}

/// One window's kept fields, or `None` when its share is not a number from 0 to 100.
fn limit(fields: &Map<String, Value>, written_at: DateTime<Utc>) -> Option<Limit> {
    let used = fields
        .get(USED_PERCENTAGE)
        .and_then(Value::as_f64)
        .filter(|used| used.is_finite() && (0.0..=100.0).contains(used))?;
    Some(Limit {
        // `abs` writes -0 as 0.
        used_percentage: used.abs(),
        resets_at: fields
            .get(RESETS_AT)
            .and_then(|reset| reset_time(reset, written_at)),
    })
}

/// A reset: whole seconds since 1970, a fraction dropped, that read as a time at most
/// [`MOST_RESET_AHEAD`] after `written_at`. A reset already passed is kept, so the window is left
/// out as one that has reset.
fn reset_time(value: &Value, written_at: DateTime<Utc>) -> Option<DateTime<Utc>> {
    let seconds = match value.as_i64() {
        Some(seconds) => seconds,
        // A number too large for whole seconds becomes the largest, which no time reads as.
        None => value
            .as_f64()
            .filter(|seconds| seconds.is_finite())?
            .floor() as i64,
    };
    let at = DateTime::from_timestamp(seconds, 0)?;
    (at <= written_at.checked_add_signed(MOST_RESET_AHEAD)?).then_some(at)
}

/// `at` as RFC 3339 in UTC, with as many digits of the second as it has.
fn timestamp(at: DateTime<Utc>) -> String {
    at.to_rfc3339_opts(SecondsFormat::AutoSi, true)
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use quotadial_core::testing::json;
    use serde_json::json;

    use super::*;

    /// Claude Code's docs' own example of what its status line hands over, every value theirs.
    const DOCUMENTED: &str = include_str!("../../../../fixtures/claude/status-line.synthetic.json");
    /// The same, as a sign-in with an API key hands it over: no `rate_limits`.
    const API_KEY: &str =
        include_str!("../../../../fixtures/claude/status-line-api-key.synthetic.json");
    /// The same, after the five-hour window's reset, which Claude Code drops.
    const ONE_WINDOW: &str =
        include_str!("../../../../fixtures/claude/status-line-one-window.synthetic.json");

    fn at(rfc3339: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(rfc3339)
            .expect("a time")
            .with_timezone(&Utc)
    }

    /// When the docs' example was handed over: before its two resets.
    fn then() -> DateTime<Utc> {
        at("2025-02-01T12:00:00Z")
    }

    fn now() -> DateTime<Utc> {
        at("2026-10-28T17:30:00Z")
    }

    fn seconds(epoch: i64) -> Option<DateTime<Utc>> {
        DateTime::from_timestamp(epoch, 0)
    }

    fn limit(used_percentage: f64, resets_at: Option<i64>) -> Limit {
        Limit {
            used_percentage,
            resets_at: resets_at.and_then(seconds),
        }
    }

    /// `rate_limits` handed over now.
    fn handed(rate_limits: &Value) -> Option<Handover> {
        from_status_line(
            json!({ "rate_limits": rate_limits }).to_string().as_bytes(),
            now(),
        )
    }

    #[test]
    fn the_documented_payload_hands_over_its_two_windows_and_nothing_of_the_spend_limit() {
        let handover = from_status_line(DOCUMENTED.as_bytes(), then()).expect("a handover");

        assert_eq!(
            handover,
            Handover {
                written_at: then(),
                windows: BTreeMap::from([
                    (FIVE_HOUR, limit(23.5, Some(1_738_425_600))),
                    (SEVEN_DAY, limit(41.2, Some(1_738_857_600))),
                ]),
            }
        );
        let kept = String::from_utf8(to_file(&handover)).expect("UTF-8");
        assert!(
            !kept.contains("spend_limit") && !kept.contains("62.8"),
            "{kept}"
        );
    }

    #[test]
    fn a_sign_in_with_an_api_key_hands_over_nothing() {
        assert_eq!(from_status_line(API_KEY.as_bytes(), then()), None);
    }

    #[test]
    fn a_window_claude_code_has_dropped_is_not_handed_over() {
        let handover = from_status_line(ONE_WINDOW.as_bytes(), then()).expect("a handover");

        assert_eq!(
            handover.windows,
            BTreeMap::from([(SEVEN_DAY, limit(41.2, Some(1_738_857_600)))])
        );
    }

    #[test]
    fn a_share_outside_0_to_100_leaves_its_window_out_and_is_never_clamped() {
        for (used, kept) in [
            (json!(0), Some(0.0)),
            (json!(100), Some(100.0)),
            (json!(41.2), Some(41.2)),
            (json!(-0.0), Some(0.0)),
            (json!(100.5), None),
            (json!(-1), None),
            (json!(250), None),
            (json!("41.2"), None),
            (json!(true), None),
            (json!(null), None),
        ] {
            let handover = handed(&json!({ "seven_day": { "used_percentage": used } }));
            let shown = handover.and_then(|handover| {
                handover
                    .windows
                    .get(SEVEN_DAY)
                    .map(|limit| limit.used_percentage)
            });
            assert_eq!(shown, kept, "{used}");
            if let Some(shown) = shown {
                assert!(shown.is_sign_positive(), "{used}");
            }
        }
    }

    #[test]
    fn a_reset_is_whole_seconds_since_1970_at_most_a_year_ahead_and_otherwise_none() {
        let reset = |resets_at: Value| {
            handed(&json!({ "five_hour": { "used_percentage": 10, "resets_at": resets_at } }))
                .expect("a handover")
                .windows[FIVE_HOUR]
                .resets_at
        };
        let in_an_hour = now().timestamp() + 3600;

        assert_eq!(reset(json!(in_an_hour)), seconds(in_an_hour));
        assert_eq!(reset(json!(in_an_hour as f64 + 0.75)), seconds(in_an_hour));
        // Already passed, as Claude Code itself never hands over: kept, so the window is left out.
        assert_eq!(reset(json!(1_738_425_600)), seconds(1_738_425_600));
        // The same time in milliseconds, centuries ahead as seconds.
        assert_eq!(reset(json!(in_an_hour * 1000)), None);
        assert_eq!(reset(json!(now().timestamp() + 367 * 24 * 3600)), None);
        for not_seconds in [
            json!("2026-10-28T18:30:00Z"),
            json!(in_an_hour.to_string()),
            json!(null),
            json!(true),
            json!(u64::MAX),
            json!(-1e300),
            json!(1e300),
        ] {
            assert_eq!(reset(not_seconds.clone()), None, "{not_seconds}");
        }
    }

    #[test]
    fn a_payload_with_no_window_to_keep_hands_over_nothing() {
        for payload in [
            "",
            "not JSON",
            "[]",
            "\"rate_limits\"",
            "{}",
            r#"{ "rate_limits": null }"#,
            r#"{ "rate_limits": [] }"#,
            r#"{ "rate_limits": {} }"#,
            r#"{ "rate_limits": { "five_hour": {} } }"#,
            r#"{ "rate_limits": { "five_hour": { "resets_at": 1738425600 } } }"#,
            r#"{ "rate_limits": { "spend_limit": { "used_percentage": 62.8, "resets_at": 1740787200 } } }"#,
            r#"{ "rate_limits": { "Five_Hour": { "used_percentage": 1 } } }"#,
            r#"{ "five_hour": { "used_percentage": 23.5 } }"#,
        ] {
            assert_eq!(
                from_status_line(payload.as_bytes(), now()),
                None,
                "{payload}"
            );
        }
        let large = format!(
            r#"{{ "rate_limits": {{ "five_hour": {{ "used_percentage": 1 }} }}, "pad": "{}" }}"#,
            " ".repeat(MOST)
        );
        assert_eq!(from_status_line(large.as_bytes(), now()), None, "too large");
    }

    /// Values a window's fields can hold: what Claude Code sends, and anything.
    fn field() -> impl Strategy<Value = Value> {
        prop_oneof![
            json(),
            any::<f64>().prop_map(Value::from),
            (-10.0..110.0_f64).prop_map(Value::from),
            any::<u64>().prop_map(Value::from),
            any::<i64>().prop_map(Value::from),
            (1_700_000_000_i64..1_900_000_000).prop_map(Value::from),
            any::<bool>().prop_map(Value::from),
        ]
    }

    /// A window's fields, each the field Claude Code names with any value, or anything at all.
    fn window() -> impl Strategy<Value = Value> {
        prop_oneof![
            json(),
            (field(), field()).prop_map(|(used, reset)| {
                json!({ "used_percentage": used, "resets_at": reset })
            }),
        ]
    }

    /// A payload: any bytes, any JSON, or Claude Code's shape with any values where it keeps them,
    /// and the same as QuotaDial's file with any time.
    fn payloads() -> impl Strategy<Value = Vec<u8>> {
        prop_oneof![
            any::<Vec<u8>>(),
            json().prop_map(|value| value.to_string().into_bytes()),
            (window(), window(), json(), json(), "[0-9T:.Z+-]{0,30}").prop_map(
                |(five_hour, seven_day, spend, other, written)| {
                    json!({
                        "rate_limits": {
                            "five_hour": five_hour,
                            "seven_day": seven_day,
                            "spend_limit": spend
                        },
                        "cwd": other,
                        "written_at": written
                    })
                    .to_string()
                    .into_bytes()
                }
            ),
        ]
    }

    proptest! {
        #[test]
        fn no_payload_makes_the_reader_panic_and_what_it_keeps_is_written_whole(payload in payloads()) {
            if let Some(handover) = from_status_line(&payload, now()) {
                prop_assert!(!handover.windows.is_empty() && handover.windows.len() <= WINDOWS.len());
                for (name, limit) in &handover.windows {
                    prop_assert!(WINDOWS.contains(name));
                    prop_assert!((0.0..=100.0).contains(&limit.used_percentage));
                    prop_assert!(limit.used_percentage.is_sign_positive());
                    prop_assert!(limit.resets_at.is_none_or(|at| at <= now() + MOST_RESET_AHEAD));
                }
                let file: Value = serde_json::from_slice(&to_file(&handover)).expect("JSON");
                let written = file["rate_limits"].as_object().expect("its windows");
                prop_assert!(written.keys().map(String::as_str).eq(handover.windows.keys().copied()));
            }
        }
    }
}
