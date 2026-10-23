//! ChatGPT's usage response, mapped to normalized windows.
//!
//! Ported from CodexBar's `Providers/Codex/CodexOAuth/CodexOAuthUsageFetcher.swift`
//! (`CodexUsageResponse`), `CodexRateWindowNormalizer.swift` and
//! `CodexAdditionalRateLimitMapper.swift`, and checked against a live Pro response on 2026-10-23
//! (D-039). The response is `rate_limit { primary_window, secondary_window }`, each
//! `{ used_percent, limit_window_seconds, reset_at, reset_after_seconds }`, plus `plan_type`,
//! `credits` and `additional_rate_limits[]` (per-model limits with the same window shape).
//!
//! Windows are classified by their length, never by their slot: a Pro account's weekly window
//! arrives as `primary_window`, with no five-hour window at all. Five hours is the session window
//! and seven days the weekly one; any other length keeps its length in the label. Fields this
//! mapping does not know (`model_usage`, `code_review_rate_limit`, `spend_control`, `promo`, the
//! reset credits) are left out until their shape is seen live.

use chrono::{DateTime, TimeDelta, Utc};
use quotadial_core::{ExtraUsage, Source, Window, WindowKind, outside_text};
use serde_json::Value;

/// The longest plan or limit name shown, from ChatGPT's text.
const MAX_NAME_CHARS: usize = 64;
const SESSION_SECONDS: u64 = 5 * 60 * 60;
const DAY_SECONDS: u64 = 24 * 60 * 60;
const WEEK_SECONDS: u64 = 7 * DAY_SECONDS;

/// What a usage response says.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Reading {
    /// Shortest first.
    pub(crate) windows: Vec<Window>,
    pub(crate) extra_usage: Option<ExtraUsage>,
    pub(crate) plan: Option<String>,
    /// Who ChatGPT says the numbers belong to, to compare with the sign-in. Never shown.
    pub(crate) user_id: Option<String>,
    pub(crate) account_id: Option<String>,
}

/// Maps a `wham/usage` response fetched at `fetched_at`.
pub(crate) fn read(body: &Value, fetched_at: DateTime<Utc>) -> Reading {
    let mut windows = body
        .get("rate_limit")
        .map(|limit| api_windows(limit, None, fetched_at))
        .unwrap_or_default();
    for limit in body
        .get("additional_rate_limits")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let Some(model) = name(limit, "limit_name").or_else(|| name(limit, "metered_feature"))
        else {
            continue;
        };
        if let Some(rate_limit) = limit.get("rate_limit") {
            windows.extend(api_windows(rate_limit, Some(&model), fetched_at));
        }
    }
    Reading {
        windows,
        extra_usage: body.get("credits").and_then(credits),
        plan: name(body, "plan_type"),
        user_id: text(body, "user_id"),
        account_id: text(body, "account_id"),
    }
}

/// The windows of one `{ primary_window, secondary_window }` object, shortest first.
fn api_windows(rate_limit: &Value, model: Option<&str>, fetched_at: DateTime<Utc>) -> Vec<Window> {
    let mut windows: Vec<Window> = ["primary_window", "secondary_window"]
        .iter()
        .filter_map(|slot| {
            let window = rate_limit.get(slot)?;
            let used_pct = window.get("used_percent")?.as_f64()?;
            let seconds = window.get("limit_window_seconds")?.as_u64()?;
            let resets_at = window
                .get("reset_at")
                .and_then(Value::as_i64)
                .and_then(|at| DateTime::from_timestamp(at, 0))
                .or_else(|| {
                    let after = window.get("reset_after_seconds")?.as_i64()?;
                    fetched_at.checked_add_signed(TimeDelta::try_seconds(after)?)
                });
            classified(Measured {
                seconds,
                used_pct,
                resets_at,
                model,
                source: Source::Api,
                fetched_at,
            })
        })
        .collect();
    windows.sort_by_key(|window| window.window_seconds);
    windows
}

/// One window's numbers, before it is classified.
pub(crate) struct Measured<'a> {
    pub(crate) seconds: u64,
    pub(crate) used_pct: f64,
    pub(crate) resets_at: Option<DateTime<Utc>>,
    /// The model a per-model limit applies to.
    pub(crate) model: Option<&'a str>,
    pub(crate) source: Source,
    pub(crate) fetched_at: DateTime<Utc>,
}

/// The window, classified by its length. `None` for a window without a length.
pub(crate) fn classified(measured: Measured<'_>) -> Option<Window> {
    if measured.seconds == 0 || !measured.used_pct.is_finite() {
        return None;
    }
    let scoped = measured.model.is_some();
    let (kind, name) = match measured.seconds {
        SESSION_SECONDS => (WindowKind::Session, "Session".to_owned()),
        DAY_SECONDS => (WindowKind::Daily, "Daily".to_owned()),
        WEEK_SECONDS if scoped => (WindowKind::ModelWeekly, "Weekly".to_owned()),
        WEEK_SECONDS => (WindowKind::Weekly, "Weekly".to_owned()),
        seconds if seconds < DAY_SECONDS => (WindowKind::Session, length(seconds)),
        seconds if scoped => (WindowKind::ModelWeekly, length(seconds)),
        seconds => (WindowKind::Weekly, length(seconds)),
    };
    Some(Window {
        kind,
        label: match measured.model {
            Some(model) => format!("{name} ({model})"),
            None => name,
        },
        used_pct: measured.used_pct,
        resets_at: measured.resets_at,
        window_seconds: measured.seconds,
        scope_model: measured.model.map(str::to_lowercase),
        source: measured.source,
        fetched_at: measured.fetched_at,
    })
}

/// For example `3-hour window` or `30-day window`.
fn length(seconds: u64) -> String {
    let (count, unit) = if seconds.is_multiple_of(DAY_SECONDS) {
        (seconds / DAY_SECONDS, "day")
    } else if seconds.is_multiple_of(3600) {
        (seconds / 3600, "hour")
    } else if seconds.is_multiple_of(60) {
        (seconds / 60, "minute")
    } else {
        (seconds, "second")
    };
    format!("{count}-{unit} window")
}

/// `credits { has_credits, unlimited, balance }`: whether usage beyond the plan's limits can go on.
/// The balance has no field in the model yet (P-012).
pub(crate) fn credits(value: &Value) -> Option<ExtraUsage> {
    let flag = |key| value.get(key).and_then(Value::as_bool).unwrap_or(false);
    value.is_object().then(|| ExtraUsage {
        enabled: flag("has_credits") || flag("unlimited"),
        used_pct: None,
        used: None,
        limit: None,
        currency: None,
    })
}

/// A name from ChatGPT's text that QuotaDial shows, such as a plan or a limit: on one line, with
/// any token blanked out, at most 64 characters.
pub(crate) fn name(value: &Value, key: &str) -> Option<String> {
    outside_text(value.get(key)?.as_str()?, MAX_NAME_CHARS)
}

/// A non-blank string field.
pub(crate) fn text(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)?
        .as_str()
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    /// The live Pro response from 2026-10-23, redacted (D-039).
    const LIVE_FIXTURE: &str = include_str!("../../../../fixtures/codex/wham-usage.json");
    /// CodexBar's field names with a Plus plan's two windows.
    const PLUS_FIXTURE: &str = include_str!("../../../../fixtures/codex/wham-usage.synthetic.json");

    fn ts(rfc3339: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(rfc3339)
            .expect("valid timestamp")
            .with_timezone(&Utc)
    }

    fn now() -> DateTime<Utc> {
        ts("2026-10-23T11:36:32Z")
    }

    #[test]
    fn maps_a_live_pro_response() {
        let body: Value = serde_json::from_str(LIVE_FIXTURE).expect("fixture is JSON");

        let reading = read(&body, now());

        assert_eq!(
            reading.windows,
            vec![Window {
                kind: WindowKind::Weekly,
                label: "Weekly".into(),
                used_pct: 54.0,
                resets_at: Some(ts("2026-10-29T17:09:48Z")),
                window_seconds: 604_800,
                scope_model: None,
                source: Source::Api,
                fetched_at: now(),
            }]
        );
        assert_eq!(reading.plan.as_deref(), Some("pro"));
        assert_eq!(
            reading.extra_usage,
            Some(ExtraUsage {
                enabled: true,
                used_pct: None,
                used: None,
                limit: None,
                currency: None,
            })
        );
    }

    #[test]
    fn maps_a_plus_response_with_both_windows() {
        let body: Value = serde_json::from_str(PLUS_FIXTURE).expect("fixture is JSON");

        let reading = read(&body, now());

        let summary: Vec<_> = reading
            .windows
            .iter()
            .map(|window| (window.kind, window.label.as_str(), window.used_pct))
            .collect();
        assert_eq!(
            summary,
            [
                (WindowKind::Session, "Session", 25.0),
                (WindowKind::Weekly, "Weekly", 50.0)
            ]
        );
        assert_eq!(
            reading.windows[0].resets_at,
            Some(ts("2099-01-01T05:00:00Z"))
        );
        assert_eq!(reading.plan.as_deref(), Some("plus"));
        assert_eq!(reading.extra_usage.map(|extra| extra.enabled), Some(false));
    }

    #[test]
    fn windows_are_classified_by_length_not_by_slot() {
        let body = json!({ "rate_limit": {
            "primary_window": { "used_percent": 70, "limit_window_seconds": 604_800, "reset_at": 4_071_513_600_u64 },
            "secondary_window": { "used_percent": 10, "limit_window_seconds": 18_000, "reset_at": 4_070_926_800_u64 }
        } });

        let labels: Vec<_> = read(&body, now())
            .windows
            .into_iter()
            .map(|window| (window.label, window.used_pct))
            .collect();

        assert_eq!(
            labels,
            [("Session".to_owned(), 10.0), ("Weekly".to_owned(), 70.0)]
        );
    }

    #[test]
    fn other_lengths_keep_their_length_in_the_label() {
        let window = |seconds| Measured {
            seconds,
            used_pct: 1.0,
            resets_at: None,
            model: None,
            source: Source::Api,
            fetched_at: now(),
        };
        let classify = |seconds| {
            let window = classified(window(seconds)).expect("window");
            (window.kind, window.label)
        };

        assert_eq!(
            classify(3 * 3600),
            (WindowKind::Session, "3-hour window".to_owned())
        );
        assert_eq!(classify(86_400), (WindowKind::Daily, "Daily".to_owned()));
        assert_eq!(
            classify(30 * 86_400),
            (WindowKind::Weekly, "30-day window".to_owned())
        );
        assert_eq!(
            classify(90 * 60),
            (WindowKind::Session, "90-minute window".to_owned())
        );
        assert_eq!(classified(window(0)), None);
    }

    #[test]
    fn per_model_limits_are_named_and_scoped() {
        let body = json!({ "additional_rate_limits": [
            { "limit_name": "GPT-5.3-Codex-Spark", "metered_feature": "codex_spark",
              "rate_limit": {
                "primary_window": { "used_percent": 12.5, "limit_window_seconds": 604_800, "reset_at": 4_071_513_600_u64 },
                "secondary_window": null } },
            { "rate_limit": { "primary_window": { "used_percent": 1, "limit_window_seconds": 18_000 } } },
            "not an object"
        ] });

        let windows = read(&body, now()).windows;

        assert_eq!(windows.len(), 1, "an unnamed limit is left out");
        assert_eq!(windows[0].kind, WindowKind::ModelWeekly);
        assert_eq!(windows[0].label, "Weekly (GPT-5.3-Codex-Spark)");
        assert_eq!(
            windows[0].scope_model.as_deref(),
            Some("gpt-5.3-codex-spark")
        );
        assert_eq!(windows[0].used_pct, 12.5);
    }

    #[test]
    fn a_missing_reset_time_comes_from_the_seconds_left() {
        let body = json!({ "rate_limit": { "primary_window": {
            "used_percent": 5, "limit_window_seconds": 18_000, "reset_after_seconds": 3600
        } } });

        let windows = read(&body, now()).windows;

        assert_eq!(windows[0].resets_at, Some(now() + TimeDelta::hours(1)));
    }

    #[test]
    fn an_absurd_reset_time_is_dropped_instead_of_overflowing() {
        let body = json!({ "rate_limit": { "primary_window": {
            "used_percent": 5, "limit_window_seconds": 18_000,
            "reset_after_seconds": 9_000_000_000_000_i64
        } } });

        let windows = read(&body, now()).windows;

        assert_eq!(windows[0].resets_at, None);
    }

    #[test]
    fn names_from_chatgpt_are_cleaned_before_they_are_shown() {
        let body = json!({
            "plan_type": format!("pro\n{}", "x".repeat(100)),
            "additional_rate_limits": [{
                "limit_name": "GPT sk-proj-NOT-A-REAL-KEY",
                "rate_limit": { "primary_window": { "used_percent": 1, "limit_window_seconds": 604_800 } }
            }],
            "user_id": "user-000000000000000000000001",
            "account_id": "00000000-0000-0000-0000-000000000002"
        });

        let reading = read(&body, now());

        let plan = reading.plan.expect("plan");
        assert!(
            plan.starts_with("pro x") && plan.chars().count() == 65,
            "{plan}"
        );
        assert_eq!(reading.windows[0].label, "Weekly (GPT [redacted token])");
        assert_eq!(
            reading.user_id.as_deref(),
            Some("user-000000000000000000000001")
        );
        assert_eq!(
            reading.account_id.as_deref(),
            Some("00000000-0000-0000-0000-000000000002")
        );
    }

    #[test]
    fn windows_missing_numbers_are_left_out() {
        let body = json!({
            "rate_limit": {
                "primary_window": { "limit_window_seconds": 18_000 },
                "secondary_window": { "used_percent": 5, "limit_window_seconds": 0 }
            },
            "credits": null
        });

        let reading = read(&body, now());

        assert!(reading.windows.is_empty());
        assert_eq!(reading.extra_usage, None);
        assert_eq!(reading.plan, None);
    }
}
