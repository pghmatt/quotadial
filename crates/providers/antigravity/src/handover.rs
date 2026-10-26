//! What QuotaDial keeps of what Antigravity CLI's status line hands over, and nothing else (the
//! manager's rule (b)(1) on #73, D-126): each limit's share or amount left, its reset and whether it
//! is turned off, the plan, the account's address, and QuotaDial's own time.
//!
//! Antigravity CLI pipes its session's state to the command as JSON, whose fields its docs list
//! ("Status line customization", antigravity.google/docs/cli/statusline, read on 2026-10-25):
//! `quota` maps each limit's id to `remaining_fraction` or `remaining_amount`, `reset_time`,
//! `reset_in_seconds` and an optional `disabled`; `plan_tier` (optional) and `email` name the
//! account. The rest, such as `cwd`, `transcript_path`, `conversation_title`, `model`, `workspace`,
//! `context_window` and `cost`, is skipped and never kept. QuotaDial's own file holds the kept
//! fields under the same names, with `written_at`, so one reader takes both and checks every value
//! each time: a share from 0 to 1, an amount of 0 or more, a time that reads as one, a reset at most
//! a year off, an address with an @ and no space, and every other text from outside cut to one
//! line with any token blanked out (`quotadial_core::outside_text`).

use std::collections::BTreeMap;

use chrono::{DateTime, SecondsFormat, TimeDelta, Utc};
use quotadial_core::outside_text;
use serde_json::{Map, Value};

/// The most of a payload or a file QuotaDial reads. Antigravity CLI's payload runs to a few
/// kilobytes; a larger one is taken as one QuotaDial cannot read.
pub(crate) const MOST: usize = 1024 * 1024;

/// The most limits kept from one handover. Antigravity has four today, a five-hour and a weekly one
/// for each of two groups of models.
pub(crate) const MOST_LIMITS: usize = 32;

/// The longest reset `reset_in_seconds` may give: a year. A week is the longest limit Antigravity
/// has.
const MOST_RESET_SECONDS: u64 = 366 * 24 * 60 * 60;

/// The longest a limit's id or the plan's name is kept, in characters.
const MOST_NAME_CHARS: usize = 64;

/// The longest an email address can be.
const MOST_ADDRESS_CHARS: usize = 254;

// The names of the fields kept, as Antigravity CLI's docs write them, and QuotaDial's own time.
const WRITTEN_AT: &str = "written_at";
const EMAIL: &str = "email";
const PLAN_TIER: &str = "plan_tier";
const QUOTA: &str = "quota";
const REMAINING_FRACTION: &str = "remaining_fraction";
const REMAINING_AMOUNT: &str = "remaining_amount";
const RESET_TIME: &str = "reset_time";
const RESET_IN_SECONDS: &str = "reset_in_seconds";
const DISABLED: &str = "disabled";

/// What one run of the status line handed over, as QuotaDial keeps it.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Handover {
    /// When QuotaDial's command took it, which its file records.
    pub(crate) written_at: DateTime<Utc>,
    /// The signed-in account's address.
    pub(crate) email: Option<String>,
    /// The plan, as Antigravity names it, such as `Pro`.
    pub(crate) plan_tier: Option<String>,
    /// Each limit with a share or an amount left, by its id, such as `gemini-weekly`.
    pub(crate) limits: BTreeMap<String, Limit>,
}

/// One limit, as Antigravity CLI hands it over.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Limit {
    /// The share of the limit left, from 0 to 1.
    pub(crate) remaining_fraction: Option<f64>,
    /// How much of the limit is left, in units Antigravity does not name, for a limit it gives no
    /// share of.
    pub(crate) remaining_amount: Option<f64>,
    /// When it resets.
    pub(crate) reset_time: Option<DateTime<Utc>>,
    /// How long after the handover it resets.
    pub(crate) reset_in_seconds: Option<u64>,
    /// Whether Antigravity CLI marks it turned off.
    pub(crate) disabled: bool,
}

impl Limit {
    /// When it resets: its `reset_time`, or `reset_in_seconds` after `handed_over`.
    pub(crate) fn resets_at(&self, handed_over: DateTime<Utc>) -> Option<DateTime<Utc>> {
        self.reset_time.or_else(|| {
            let seconds = i64::try_from(self.reset_in_seconds?).ok()?;
            handed_over.checked_add_signed(TimeDelta::try_seconds(seconds)?)
        })
    }
}

/// What the status line's JSON, `payload`, hands over, taken at `now`, or `None` when it holds no
/// share or amount to keep, as before Antigravity CLI has asked for its limits.
pub(crate) fn from_status_line(payload: &[u8], now: DateTime<Utc>) -> Option<Handover> {
    let handover = read(&object(payload)?, now);
    (!handover.limits.is_empty()).then_some(handover)
}

/// What QuotaDial's own file holds, or `None` when it is not one: not JSON, or with no time.
pub(crate) fn from_file(file: &[u8]) -> Option<Handover> {
    let fields = object(file)?;
    let written_at = fields
        .get(WRITTEN_AT)
        .and_then(Value::as_str)
        .and_then(time)?;
    Some(read(&fields, written_at))
}

/// The file QuotaDial keeps for `handover`: the kept fields under Antigravity's names, and
/// `written_at`, as JSON a person can read.
pub(crate) fn to_file(handover: &Handover) -> Vec<u8> {
    let mut fields = Map::new();
    fields.insert(WRITTEN_AT.into(), timestamp(handover.written_at).into());
    if let Some(email) = &handover.email {
        fields.insert(EMAIL.into(), email.clone().into());
    }
    if let Some(plan) = &handover.plan_tier {
        fields.insert(PLAN_TIER.into(), plan.clone().into());
    }
    let limits: Map<String, Value> = handover
        .limits
        .iter()
        .map(|(id, limit)| (id.clone(), Value::Object(limit_fields(limit))))
        .collect();
    fields.insert(QUOTA.into(), Value::Object(limits));
    format!("{:#}\n", Value::Object(fields)).into_bytes()
}

/// The fields of `limit` that hold something.
fn limit_fields(limit: &Limit) -> Map<String, Value> {
    let mut fields = Map::new();
    if let Some(fraction) = limit.remaining_fraction {
        fields.insert(REMAINING_FRACTION.into(), fraction.into());
    }
    if let Some(amount) = limit.remaining_amount {
        fields.insert(REMAINING_AMOUNT.into(), amount.into());
    }
    if let Some(reset) = limit.reset_time {
        fields.insert(RESET_TIME.into(), timestamp(reset).into());
    }
    if let Some(seconds) = limit.reset_in_seconds {
        fields.insert(RESET_IN_SECONDS.into(), seconds.into());
    }
    if limit.disabled {
        fields.insert(DISABLED.into(), true.into());
    }
    fields
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

/// The kept fields of `fields`, taken at `written_at`, each checked: one of the wrong kind or out of
/// range is left out.
fn read(fields: &Map<String, Value>, written_at: DateTime<Utc>) -> Handover {
    Handover {
        written_at,
        email: fields.get(EMAIL).and_then(Value::as_str).and_then(address),
        plan_tier: fields
            .get(PLAN_TIER)
            .and_then(Value::as_str)
            .and_then(|plan| outside_text(plan, MOST_NAME_CHARS)),
        limits: fields
            .get(QUOTA)
            .and_then(Value::as_object)
            .map(limits)
            .unwrap_or_default(),
    }
}

/// The limits in `quota` with a share or an amount left, the first [`MOST_LIMITS`] of them by id.
fn limits(quota: &Map<String, Value>) -> BTreeMap<String, Limit> {
    let mut kept: BTreeMap<String, Limit> = quota
        .iter()
        .filter_map(|(id, fields)| {
            Some((
                outside_text(id, MOST_NAME_CHARS)?,
                limit(fields.as_object()?)?,
            ))
        })
        .collect();
    while kept.len() > MOST_LIMITS {
        kept.pop_last();
    }
    kept
}

/// One limit's kept fields, or `None` when it has neither a share nor an amount left.
fn limit(fields: &Map<String, Value>) -> Option<Limit> {
    let number = |name| {
        fields
            .get(name)
            .and_then(Value::as_f64)
            .filter(|number| number.is_finite())
    };
    let limit = Limit {
        // `abs` writes -0 as 0.
        remaining_fraction: number(REMAINING_FRACTION)
            .filter(|fraction| (0.0..=1.0).contains(fraction))
            .map(f64::abs),
        remaining_amount: number(REMAINING_AMOUNT)
            .filter(|amount| *amount >= 0.0)
            .map(f64::abs),
        reset_time: fields
            .get(RESET_TIME)
            .and_then(Value::as_str)
            .and_then(time),
        reset_in_seconds: fields.get(RESET_IN_SECONDS).and_then(seconds),
        disabled: fields.get(DISABLED).and_then(Value::as_bool) == Some(true),
    };
    (limit.remaining_fraction.is_some() || limit.remaining_amount.is_some()).then_some(limit)
}

/// A whole number of seconds from 0 to a year; a fraction of a second is dropped.
fn seconds(value: &Value) -> Option<u64> {
    let seconds = match value.as_u64() {
        Some(seconds) => seconds,
        None => {
            let seconds = value.as_f64().filter(|seconds| {
                seconds.is_finite() && (0.0..=MOST_RESET_SECONDS as f64).contains(seconds)
            })?;
            seconds.floor() as u64
        }
    };
    (seconds <= MOST_RESET_SECONDS).then_some(seconds)
}

/// An email address: one word with an @, cut to one line, with no token in it.
fn address(text: &str) -> Option<String> {
    let address = outside_text(text, MOST_ADDRESS_CHARS)?;
    (address.contains('@') && !address.contains(char::is_whitespace) && !address.ends_with('…'))
        .then_some(address)
}

/// An RFC 3339 time, such as `2026-07-06T07:50:32Z`.
fn time(text: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(text.trim())
        .ok()
        .map(|at| at.with_timezone(&Utc))
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

    fn at(rfc3339: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(rfc3339)
            .expect("a time")
            .with_timezone(&Utc)
    }

    fn now() -> DateTime<Utc> {
        at("2026-10-25T22:30:00Z")
    }

    /// `payload` handed over now.
    fn handed(payload: &Value) -> Option<Handover> {
        from_status_line(payload.to_string().as_bytes(), now())
    }

    fn limit(fraction: Option<f64>, amount: Option<f64>) -> Limit {
        Limit {
            remaining_fraction: fraction,
            remaining_amount: amount,
            reset_time: None,
            reset_in_seconds: None,
            disabled: false,
        }
    }

    /// A payload of the documented shape, every value made up: Antigravity CLI's fields with one
    /// limit, of the kind its docs' own example shows.
    fn documented() -> Value {
        json!({
            "cwd": "/home/someone/project",
            "session_id": "00000000-0000-0000-0000-000000000001",
            "conversation_id": "00000000-0000-0000-0000-000000000001",
            "transcript_path": "/home/someone/.gemini/antigravity/brain/transcript.jsonl",
            "conversation_title": "A made-up conversation",
            "model": { "id": "example-model", "display_name": "Example Model", "effort": "medium" },
            "workspace": { "current_dir": "/home/someone/project", "project_dir": "/home/someone/project" },
            "version": "1.4.5",
            "context_window": {
                "total_input_tokens": 1200, "total_output_tokens": 340, "context_window_size": 200000,
                "used_percentage": 0.8, "remaining_percentage": 99.2, "current_usage": null
            },
            "exceeds_200k_tokens": false,
            "product": "antigravity",
            "quota": {
                "gemini-weekly": {
                    "remaining_fraction": 0.9378,
                    "reset_time": "2026-10-31T07:50:32Z",
                    "reset_in_seconds": 465_032
                }
            },
            "agent_state": "idle",
            "vcs": { "type": "git" },
            "cycle_mode": "plan",
            "sandbox": { "enabled": false },
            "artifact_count": 2,
            "plan_tier": "Pro",
            "email": "someone@example.com",
            "task_count": 1,
            "terminal_width": 111,
            "cost": { "total_usd": 0.42, "subagent_usd": 0, "estimated": false }
        })
    }

    #[test]
    fn the_documented_payload_hands_over_its_one_limit_the_plan_and_the_address() {
        let handover = handed(&documented()).expect("a handover");

        assert_eq!(
            handover,
            Handover {
                written_at: now(),
                email: Some("someone@example.com".into()),
                plan_tier: Some("Pro".into()),
                limits: BTreeMap::from([(
                    "gemini-weekly".into(),
                    Limit {
                        remaining_fraction: Some(0.9378),
                        remaining_amount: None,
                        reset_time: Some(at("2026-10-31T07:50:32Z")),
                        reset_in_seconds: Some(465_032),
                        disabled: false,
                    }
                )]),
            }
        );
    }

    #[test]
    fn every_shape_the_docs_give_is_read() {
        let payload = json!({
            "quota": {
                "gemini-5h": { "remaining_fraction": 0.75, "reset_in_seconds": 9000 },
                "gemini-weekly": { "remaining_fraction": 0.5, "reset_time": "2026-10-31T00:00:00Z" },
                "3p-weekly": { "remaining_amount": 120, "reset_time": "2026-10-31T00:00:00Z" },
                "3p-5h": { "remaining_fraction": 1, "disabled": true },
                "made-up-limit": { "remaining_fraction": 0.25, "more": "fields", "kept": false }
            },
            "email": "someone@example.com",
            "conversation_title": "skipped"
        });

        let handover = handed(&payload).expect("a handover");

        assert_eq!(handover.plan_tier, None, "the plan is optional");
        assert_eq!(
            handover.limits,
            BTreeMap::from([
                (
                    "3p-5h".into(),
                    Limit {
                        disabled: true,
                        ..limit(Some(1.0), None)
                    }
                ),
                (
                    "3p-weekly".into(),
                    Limit {
                        reset_time: Some(at("2026-10-31T00:00:00Z")),
                        ..limit(None, Some(120.0))
                    }
                ),
                (
                    "gemini-5h".into(),
                    Limit {
                        reset_in_seconds: Some(9000),
                        ..limit(Some(0.75), None)
                    }
                ),
                (
                    "gemini-weekly".into(),
                    Limit {
                        reset_time: Some(at("2026-10-31T00:00:00Z")),
                        ..limit(Some(0.5), None)
                    }
                ),
                ("made-up-limit".into(), limit(Some(0.25), None)),
            ])
        );
    }

    #[test]
    fn a_value_of_the_wrong_kind_or_out_of_range_is_not_kept() {
        let payload = json!({
            "quota": {
                "too-much": { "remaining_fraction": 1.5, "remaining_amount": 3 },
                "below-none": { "remaining_fraction": -0.25, "remaining_amount": -2, "reset_in_seconds": 9 },
                "as-text": { "remaining_fraction": "0.5", "remaining_amount": 4 },
                "bad-times": {
                    "remaining_fraction": 0.5,
                    "reset_time": "next Tuesday",
                    "reset_in_seconds": -60,
                    "disabled": "yes"
                },
                "far-off": { "remaining_fraction": 0.5, "reset_in_seconds": 400 * 24 * 60 * 60 },
                "part-second": { "remaining_fraction": 0.5, "reset_in_seconds": 90.75 },
                "minus-zero": { "remaining_fraction": -0.0, "remaining_amount": -0.0 },
                "not-a-limit": 0.5
            },
            "email": "not an address",
            "plan_tier": 7
        });

        let handover = handed(&payload).expect("a handover");

        assert_eq!(handover.email, None, "{handover:?}");
        assert_eq!(handover.plan_tier, None);
        let kept = |id: &str| handover.limits.get(id).cloned();
        assert_eq!(kept("too-much"), Some(limit(None, Some(3.0))));
        assert_eq!(kept("below-none"), None, "nothing left to keep");
        assert_eq!(kept("as-text"), Some(limit(None, Some(4.0))));
        assert_eq!(kept("bad-times"), Some(limit(Some(0.5), None)));
        assert_eq!(kept("far-off"), Some(limit(Some(0.5), None)));
        assert_eq!(
            kept("part-second").and_then(|limit| limit.reset_in_seconds),
            Some(90)
        );
        let zero = kept("minus-zero").expect("kept");
        assert!(zero.remaining_fraction.is_some_and(f64::is_sign_positive));
        assert!(zero.remaining_amount.is_some_and(f64::is_sign_positive));
        assert_eq!(kept("not-a-limit"), None);
    }

    #[test]
    fn an_address_is_one_word_with_an_at() {
        for (given, kept) in [
            ("someone@example.com", Some("someone@example.com")),
            ("  someone@example.com\n", Some("someone@example.com")),
            ("Someone@Example.com", Some("Someone@Example.com")),
            ("someone", None),
            ("some one@example.com", None),
            ("", None),
        ] {
            let payload =
                json!({ "email": given, "quota": { "x-5h": { "remaining_fraction": 1 } } });
            assert_eq!(
                handed(&payload).and_then(|handover| handover.email),
                kept.map(str::to_owned),
                "{given:?}"
            );
        }
        let long = format!("{}@example.com", "a".repeat(300));
        let payload = json!({ "email": long, "quota": { "x-5h": { "remaining_fraction": 1 } } });
        assert_eq!(
            handed(&payload).and_then(|handover| handover.email),
            None,
            "too long to be one"
        );
    }

    #[test]
    fn a_payload_with_no_share_or_amount_hands_over_nothing() {
        for payload in [
            "",
            "not JSON",
            "[]",
            "\"quota\"",
            "{}",
            r#"{ "quota": null }"#,
            r#"{ "quota": [] }"#,
            r#"{ "quota": {} }"#,
            r#"{ "quota": { "gemini-5h": {} } }"#,
            r#"{ "quota": { "gemini-5h": { "reset_in_seconds": 60, "disabled": true } } }"#,
            r#"{ "email": "someone@example.com", "plan_tier": "Pro" }"#,
        ] {
            assert_eq!(
                from_status_line(payload.as_bytes(), now()),
                None,
                "{payload}"
            );
        }
        let large = format!(
            r#"{{ "quota": {{ "gemini-5h": {{ "remaining_fraction": 1 }} }}, "pad": "{}" }}"#,
            " ".repeat(MOST)
        );
        assert_eq!(from_status_line(large.as_bytes(), now()), None, "too large");
    }

    #[test]
    fn text_from_outside_is_cut_to_one_line_with_no_token_in_it() {
        // Written in pieces, so no secret scanner takes this file for one holding a token.
        let token = concat!("ya29", ".MadeUpAccessTokenThatAntigravityNeverSends0123");
        let payload = json!({
            "plan_tier": format!("Pro {token}"),
            "email": format!("{token}@example.com"),
            "quota": {
                format!("weekly {token}"): { "remaining_fraction": 0.5 },
                "a\nlong\tname": { "remaining_fraction": 0.5 },
                "x".repeat(100): { "remaining_fraction": 0.5 }
            }
        });

        let handover = handed(&payload).expect("a handover");
        let kept = String::from_utf8(to_file(&handover)).expect("UTF-8");

        assert!(!kept.contains("MadeUpAccessToken"), "{kept}");
        assert_eq!(handover.plan_tier.as_deref(), Some("Pro [redacted token]"));
        assert_eq!(handover.email, None, "a token is no address");
        let ids: Vec<&str> = handover.limits.keys().map(String::as_str).collect();
        assert_eq!(
            ids,
            [
                "a long name",
                "weekly [redacted token]",
                &format!("{}…", "x".repeat(64))
            ]
        );
    }

    #[test]
    fn at_most_32_limits_are_kept_the_first_by_id() {
        let quota: Map<String, Value> = (0..40)
            .map(|index| {
                (
                    format!("limit-{index:02}"),
                    json!({ "remaining_fraction": 0.5 }),
                )
            })
            .collect();

        let handover = handed(&json!({ "quota": quota })).expect("a handover");

        assert_eq!(handover.limits.len(), MOST_LIMITS);
        assert_eq!(
            handover.limits.keys().last().map(String::as_str),
            Some("limit-31")
        );
    }

    #[test]
    fn the_file_reads_back_as_it_was_written() {
        let handover = Handover {
            written_at: at("2026-10-25T22:30:00.123456789Z"),
            ..handed(&documented()).expect("a handover")
        };

        let file = to_file(&handover);

        assert_eq!(from_file(&file), Some(handover));
        assert!(file.ends_with(b"}\n"), "a text file");
    }

    #[test]
    fn a_file_without_its_time_is_not_one() {
        assert_eq!(from_file(br#"{ "quota": {} }"#), None);
        assert_eq!(
            from_file(br#"{ "written_at": "yesterday", "quota": {} }"#),
            None
        );
        assert_eq!(
            from_file(br#"{ "written_at": "2026-10-25T22:30:00Z" }"#),
            Some(Handover {
                written_at: now(),
                email: None,
                plan_tier: None,
                limits: BTreeMap::new(),
            }),
            "a file with no limit left is still one"
        );
    }

    #[test]
    fn a_reset_is_its_time_or_its_seconds_after_the_handover() {
        let time = Limit {
            reset_time: Some(at("2026-10-26T01:00:00Z")),
            reset_in_seconds: Some(60),
            ..limit(Some(0.5), None)
        };
        let seconds = Limit {
            reset_in_seconds: Some(9000),
            ..limit(Some(0.5), None)
        };

        assert_eq!(time.resets_at(now()), Some(at("2026-10-26T01:00:00Z")));
        assert_eq!(seconds.resets_at(now()), Some(at("2026-10-26T01:00:00Z")));
        assert_eq!(limit(Some(0.5), None).resets_at(now()), None);
    }

    /// Ids Antigravity uses, and others.
    const IDS: &[&str] = &[
        "gemini-5h",
        "gemini-weekly",
        "3p-5h",
        "3p-weekly",
        "gemini-4-argon-weekly",
        "",
        " ",
        "weekly",
    ];

    /// Values a limit's fields can hold: what Antigravity sends, and anything.
    fn field() -> impl Strategy<Value = Value> {
        prop_oneof![
            json(),
            any::<f64>().prop_map(Value::from),
            (-2.0..2.0_f64).prop_map(Value::from),
            any::<u64>().prop_map(Value::from),
            any::<i64>().prop_map(Value::from),
            any::<bool>().prop_map(Value::from),
            "[0-9T:.Z+-]{0,30}".prop_map(Value::from),
        ]
    }

    /// A limit's fields, each the field Antigravity names with any value, or anything at all.
    fn limit_value() -> impl Strategy<Value = Value> {
        prop_oneof![
            json(),
            (field(), field(), field(), field(), field()).prop_map(
                |(fraction, amount, time, seconds, disabled)| {
                    json!({
                        "remaining_fraction": fraction,
                        "remaining_amount": amount,
                        "reset_time": time,
                        "reset_in_seconds": seconds,
                        "disabled": disabled
                    })
                }
            ),
        ]
    }

    /// A payload: any bytes, any JSON, or Antigravity CLI's shape with any values where it keeps
    /// them, and the same as QuotaDial's file with any time.
    fn payloads() -> impl Strategy<Value = Vec<u8>> {
        let quota = prop::collection::vec(
            (
                prop_oneof![prop::sample::select(IDS).prop_map(str::to_owned), ".{0,80}"],
                limit_value(),
            ),
            0..40,
        )
        .prop_map(|limits| Value::Object(limits.into_iter().collect()));
        prop_oneof![
            any::<Vec<u8>>(),
            json().prop_map(|value| value.to_string().into_bytes()),
            (quota, field(), field(), json(), field()).prop_map(
                |(quota, email, plan, other, written)| {
                    json!({
                        "quota": quota,
                        "email": email,
                        "plan_tier": plan,
                        "cwd": other,
                        "written_at": written
                    })
                    .to_string()
                    .into_bytes()
                }
            ),
        ]
    }

    /// Whether `read` is `written` read back from its file: the same but for the last digit of a
    /// number, which serde_json's parser may round otherwise than its printer wrote it.
    fn read_back(written: &Handover, read: &Handover) -> bool {
        let close = |a: Option<f64>, b: Option<f64>| match (a, b) {
            (Some(a), Some(b)) => (a - b).abs() <= a.abs().max(b.abs()) * 1e-12,
            (a, b) => a.is_none() && b.is_none(),
        };
        written.written_at == read.written_at
            && written.email == read.email
            && written.plan_tier == read.plan_tier
            && written.limits.len() == read.limits.len()
            && written
                .limits
                .iter()
                .zip(&read.limits)
                .all(|((id, a), (other, b))| {
                    id == other
                        && close(a.remaining_fraction, b.remaining_fraction)
                        && close(a.remaining_amount, b.remaining_amount)
                        && (a.reset_time, a.reset_in_seconds, a.disabled)
                            == (b.reset_time, b.reset_in_seconds, b.disabled)
                })
    }

    proptest! {
        #[test]
        fn no_payload_makes_the_reader_panic_and_what_it_keeps_reads_back(payload in payloads()) {
            let _ = from_file(&payload);
            if let Some(handover) = from_status_line(&payload, now()) {
                prop_assert!(!handover.limits.is_empty() && handover.limits.len() <= MOST_LIMITS);
                for limit in handover.limits.values() {
                    prop_assert!(limit.remaining_fraction.is_some() || limit.remaining_amount.is_some());
                    prop_assert!(limit.remaining_fraction.is_none_or(|fraction| (0.0..=1.0).contains(&fraction)));
                    prop_assert!(limit.remaining_amount.is_none_or(|amount| amount >= 0.0 && amount.is_finite()));
                    prop_assert!(limit.reset_in_seconds.is_none_or(|seconds| seconds <= MOST_RESET_SECONDS));
                }
                let read = from_file(&to_file(&handover));
                prop_assert!(read.as_ref().is_some_and(|read| read_back(&handover, read)), "{:?}", read);
            }
        }
    }
}
