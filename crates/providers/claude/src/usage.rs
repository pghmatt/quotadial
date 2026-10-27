//! Turns Anthropic's `GET /api/oauth/usage` response into normalized windows.
//!
//! Ported from CodexBar: `OAuthUsageResponse` and `OAuthExtraUsage` in
//! `Providers/Claude/ClaudeOAuth/ClaudeOAuthUsageFetcher.swift`, `mapOAuthUsage` in
//! `Providers/Claude/ClaudeUsageFetcher.swift`, the `limits` parser in
//! `Providers/Claude/ClaudeWeb/ClaudeWebExtraRateWindowParser.swift`, and
//! `Providers/Claude/ClaudeScopedWeeklyLimitMapper.swift`. Unlike CodexBar, a bucket QuotaDial has
//! no name for still appears, labelled from its key, as long as its key gives its length
//! (`five_hour_*` or `seven_day_*`). Buckets whose length cannot be told wait for the signed
//! remote schema file (issue #7).

use std::collections::HashSet;

use chrono::{DateTime, Utc};
use quotadial_core::{ExtraUsage, Source, Window, WindowKind, outside_text};
use serde_json::{Map, Value};

const FIVE_HOURS: u64 = 5 * 60 * 60;
const SEVEN_DAYS: u64 = 7 * 24 * 60 * 60;
/// The longest model or bucket name shown, from Anthropic's text.
const MAX_NAME_CHARS: usize = 64;

/// A bucket QuotaDial knows by name.
struct Bucket {
    key: &'static str,
    kind: WindowKind,
    label: &'static str,
    window_seconds: u64,
    scope_model: Option<&'static str>,
}

const KNOWN_BUCKETS: [Bucket; 5] = [
    Bucket {
        key: "five_hour",
        kind: WindowKind::Session,
        label: "Session",
        window_seconds: FIVE_HOURS,
        scope_model: None,
    },
    Bucket {
        key: "seven_day",
        kind: WindowKind::Weekly,
        label: "Weekly",
        window_seconds: SEVEN_DAYS,
        scope_model: None,
    },
    Bucket {
        key: "seven_day_opus",
        kind: WindowKind::ModelWeekly,
        label: "Weekly (Opus)",
        window_seconds: SEVEN_DAYS,
        scope_model: Some("opus"),
    },
    Bucket {
        key: "seven_day_sonnet",
        kind: WindowKind::ModelWeekly,
        label: "Weekly (Sonnet)",
        window_seconds: SEVEN_DAYS,
        scope_model: Some("sonnet"),
    },
    Bucket {
        key: "seven_day_oauth_apps",
        kind: WindowKind::Weekly,
        label: "Weekly (OAuth apps)",
        window_seconds: SEVEN_DAYS,
        scope_model: None,
    },
];

/// The keys Anthropic has used for the Routines (now Cowork) weekly limit. Only the first one
/// present is shown, as in CodexBar.
const ROUTINES_KEYS: [&str; 7] = [
    "seven_day_routines",
    "seven_day_claude_routines",
    "claude_routines",
    "routines",
    "routine",
    "seven_day_cowork",
    "cowork",
];

/// More decimal places than any currency uses; a larger `decimal_places` is ignored.
const MAX_DECIMAL_PLACES: u32 = 6;

/// Top-level keys that are not limit windows, although `extra_usage` has a `utilization`.
const NOT_WINDOWS: [&str; 4] = ["extra_usage", "iguana_necktie", "cedar_ember", "limits"];

/// What one usage response says.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Reading {
    pub(crate) windows: Vec<Window>,
    pub(crate) extra_usage: Option<ExtraUsage>,
}

/// Reads a usage response fetched at `fetched_at`. Buckets without a utilization are left out:
/// Anthropic sends them as `null` when a limit does not apply.
pub(crate) fn read(body: &Value, fetched_at: DateTime<Utc>) -> Reading {
    let Some(buckets) = body.as_object() else {
        return Reading {
            windows: Vec::new(),
            extra_usage: None,
        };
    };
    let window = |kind, label: String, window_seconds, scope_model: Option<String>, bucket| {
        bucket_window(bucket).map(|(used_pct, resets_at)| Window {
            kind,
            label,
            used_pct,
            resets_at,
            window_seconds,
            scope_model,
            source: Source::Api,
            fetched_at,
            unconfirmed: false,
            pace: None,
        })
    };

    let mut windows = Vec::new();
    for known in &KNOWN_BUCKETS {
        if let Some(bucket) = buckets.get(known.key) {
            windows.extend(window(
                known.kind,
                known.label.to_owned(),
                known.window_seconds,
                known.scope_model.map(str::to_owned),
                bucket,
            ));
        }
    }
    if let Some((key, bucket)) = ROUTINES_KEYS
        .iter()
        .find_map(|key| Some((*key, buckets.get(*key).filter(|bucket| bucket.is_object())?)))
    {
        let label = if key.contains("cowork") {
            "Weekly (Cowork)"
        } else {
            "Weekly (Routines)"
        };
        windows.extend(window(
            WindowKind::Weekly,
            label.to_owned(),
            SEVEN_DAYS,
            None,
            bucket,
        ));
    }
    for (key, bucket) in unknown_buckets(buckets) {
        // A known prefix gives the length; any other bucket appears by name, its length unknown,
        // until #7's signed schema can name it (P-014).
        let (kind, label, window_seconds) = if let Some(name) = key.strip_prefix("seven_day_") {
            (
                WindowKind::Weekly,
                format!("Weekly ({})", humanize(name)),
                SEVEN_DAYS,
            )
        } else if let Some(name) = key.strip_prefix("five_hour_") {
            (
                WindowKind::Session,
                format!("Session ({})", humanize(name)),
                FIVE_HOURS,
            )
        } else {
            (WindowKind::Other, humanize(key), 0)
        };
        windows.extend(window(kind, label, window_seconds, None, bucket));
    }
    windows.extend(scoped_weekly_limits(body, &windows, fetched_at));

    Reading {
        windows,
        extra_usage: buckets.get("extra_usage").and_then(extra_usage),
    }
}

/// Top-level objects with a `utilization` that no other rule claims, in key order.
fn unknown_buckets(buckets: &Map<String, Value>) -> impl Iterator<Item = (&str, &Value)> {
    buckets.iter().filter_map(|(key, bucket)| {
        let claimed = KNOWN_BUCKETS.iter().any(|known| known.key == key)
            || ROUTINES_KEYS.contains(&key.as_str())
            || NOT_WINDOWS.contains(&key.as_str());
        (!claimed && bucket.get("utilization").is_some()).then_some((key.as_str(), bucket))
    })
}

/// `{ "utilization": <percent>, "resets_at": <RFC 3339> }` as a percentage and reset time.
fn bucket_window(bucket: &Value) -> Option<(f64, Option<DateTime<Utc>>)> {
    let used_pct = number(bucket.get("utilization")?)?;
    Some((used_pct, bucket.get("resets_at").and_then(timestamp)))
}

/// The newer `limits` array: model-scoped weekly limits such as a promotional "Fable only" window.
/// A model already shown by a flat `seven_day_<model>` bucket is not shown twice.
fn scoped_weekly_limits(body: &Value, shown: &[Window], fetched_at: DateTime<Utc>) -> Vec<Window> {
    let Some(limits) = body.get("limits").and_then(Value::as_array) else {
        return Vec::new();
    };
    let mut seen: HashSet<String> = shown
        .iter()
        .filter_map(|window| window.scope_model.as_deref().map(slug))
        .collect();
    limits
        .iter()
        .filter(|limit| {
            limit.get("kind").and_then(Value::as_str) == Some("weekly_scoped")
                && limit.get("group").and_then(Value::as_str) == Some("weekly")
        })
        .filter_map(|limit| {
            let used_pct = number(limit.get("percent")?)?;
            let model = limit.get("scope")?.get("model")?;
            let name = outside_text(model.get("display_name")?.as_str()?, MAX_NAME_CHARS)?;
            let name = name.as_str();
            let id = model.get("id").and_then(Value::as_str).map(str::trim);
            if is_all_models(name, id) || !seen.insert(slug(name)) {
                return None;
            }
            Some(Window {
                kind: WindowKind::ModelWeekly,
                label: format!("Weekly ({name})"),
                used_pct,
                resets_at: limit.get("resets_at").and_then(timestamp),
                window_seconds: SEVEN_DAYS,
                scope_model: Some(name.to_lowercase()),
                source: Source::Api,
                fetched_at,
                unconfirmed: false,
                pace: None,
            })
        })
        .collect()
}

/// A scope that covers every model is the plain weekly limit again, not a per-model one.
fn is_all_models(name: &str, id: Option<&str>) -> bool {
    slug(name) == "all-models"
        || id
            .filter(|id| !id.is_empty())
            .map(slug)
            .is_some_and(|id| id == "all-models" || id.ends_with("-all-models"))
}

/// `"Claude Fable 5"` → `"claude-fable-5"`.
fn slug(text: &str) -> String {
    let mut slug = String::new();
    for c in text.to_lowercase().chars() {
        if c.is_alphanumeric() {
            slug.push(c);
        } else if !slug.ends_with('-') {
            slug.push('-');
        }
    }
    slug.trim_matches('-').to_owned()
}

/// `"example_model"` → `"Example model"`. The key is Anthropic's text, so it is cleaned like any.
fn humanize(key: &str) -> String {
    let words = outside_text(&key.replace('_', " "), MAX_NAME_CHARS).unwrap_or_default();
    let mut chars = words.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => words,
    }
}

/// `extra_usage`, with amounts converted from minor units. The live response (2026-10-23) carries
/// `decimal_places`, which says how many; the new `spend` object spells the same thing as
/// `amount_minor` with an `exponent`. When `decimal_places` is missing, amounts are cents, as
/// CodexBar found: "OAuth extra usage is always in cents, including Enterprise spend-only
/// responses". `currency` is Anthropic's text, so it is kept only as a currency's three letters
/// ([`ExtraUsage::currency_code`]); one that is missing, or anything else, is left unknown, never
/// taken for dollars, which is how CodexBar takes a missing one (the manager's review of #108).
fn extra_usage(value: &Value) -> Option<ExtraUsage> {
    if !value.is_object() {
        return None;
    }
    let scale = value
        .get("decimal_places")
        .and_then(Value::as_u64)
        .and_then(|places| u32::try_from(places).ok())
        .filter(|places| *places <= MAX_DECIMAL_PLACES)
        .map_or(100.0, |places| 10_f64.powi(places.cast_signed()));
    let amount = |key| value.get(key).and_then(number).map(|minor| minor / scale);
    let used = amount("used_credits");
    let limit = amount("monthly_limit");
    let currency = value
        .get("currency")
        .and_then(Value::as_str)
        .and_then(ExtraUsage::currency_code);
    Some(ExtraUsage {
        enabled: value
            .get("is_enabled")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        used_pct: value.get("utilization").and_then(number),
        used,
        limit,
        currency,
        balance: None,
    })
}

/// A finite JSON number. Booleans and strings are not numbers.
fn number(value: &Value) -> Option<f64> {
    value.as_f64().filter(|number| number.is_finite())
}

fn timestamp(value: &Value) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value.as_str()?.trim())
        .ok()
        .map(|at| at.with_timezone(&Utc))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// A live response from 2026-10-23, committed as Anthropic sent it: it holds no identifiers.
    const LIVE_FIXTURE: &str = include_str!("../../../../fixtures/claude/oauth-usage.json");
    const USAGE_FIXTURE: &str =
        include_str!("../../../../fixtures/claude/oauth-usage.synthetic.json");
    const SCOPED_FIXTURE: &str =
        include_str!("../../../../fixtures/claude/oauth-usage-scoped.synthetic.json");

    fn ts(rfc3339: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(rfc3339)
            .expect("valid timestamp")
            .with_timezone(&Utc)
    }

    fn fetched_at() -> DateTime<Utc> {
        ts("2099-01-01T00:00:00Z")
    }

    fn summary(reading: &Reading) -> Vec<(WindowKind, &str, f64, u64, Option<&str>)> {
        reading
            .windows
            .iter()
            .map(|window| {
                (
                    window.kind,
                    window.label.as_str(),
                    window.used_pct,
                    window.window_seconds,
                    window.scope_model.as_deref(),
                )
            })
            .collect()
    }

    #[test]
    fn maps_a_live_response() {
        let body: Value = serde_json::from_str(LIVE_FIXTURE).expect("fixture is valid JSON");
        let reading = read(&body, fetched_at());

        // The per-model buckets are null; the scoped weekly limit comes from `limits[]`. The
        // breakdown, the `spend` object and the code-named nulls are not windows.
        assert_eq!(
            summary(&reading),
            [
                (WindowKind::Session, "Session", 4.0, 18_000, None),
                (WindowKind::Weekly, "Weekly", 6.0, 604_800, None),
                (
                    WindowKind::ModelWeekly,
                    "Weekly (Fable)",
                    0.0,
                    604_800,
                    Some("fable")
                ),
            ]
        );
        assert_eq!(
            reading.windows[0].resets_at,
            Some(ts("2026-10-23T04:10:00.451650Z"))
        );
        assert_eq!(
            reading.windows[2].resets_at,
            Some(ts("2026-10-26T23:00:00Z"))
        );
        assert_eq!(
            reading.extra_usage,
            Some(ExtraUsage {
                enabled: false,
                used_pct: None,
                used: None,
                limit: None,
                currency: None,
                balance: None,
            })
        );
    }

    #[test]
    fn maps_the_codexbar_fixture() {
        let body: Value = serde_json::from_str(USAGE_FIXTURE).expect("fixture is valid JSON");
        let reading = read(&body, fetched_at());

        assert_eq!(
            summary(&reading),
            [
                (WindowKind::Session, "Session", 12.0, 18_000, None),
                (WindowKind::Weekly, "Weekly", 34.0, 604_800, None),
                (
                    WindowKind::ModelWeekly,
                    "Weekly (Opus)",
                    0.0,
                    604_800,
                    Some("opus")
                ),
                (
                    WindowKind::ModelWeekly,
                    "Weekly (Sonnet)",
                    56.0,
                    604_800,
                    Some("sonnet")
                ),
            ]
        );
        assert_eq!(
            reading.windows[0].resets_at,
            Some(ts("2099-01-01T05:00:00Z"))
        );
        // A bucket with no reset time keeps its number instead of being dropped.
        assert_eq!(reading.windows[2].resets_at, None);
        for window in &reading.windows {
            assert_eq!(window.source, Source::Api);
            assert_eq!(window.fetched_at, fetched_at());
        }
        assert_eq!(
            reading.extra_usage,
            Some(ExtraUsage {
                enabled: false,
                used_pct: None,
                used: None,
                limit: None,
                currency: None,
                balance: None,
            })
        );
    }

    #[test]
    fn maps_scoped_limits_routines_unknown_buckets_and_extra_usage() {
        let body: Value = serde_json::from_str(SCOPED_FIXTURE).expect("fixture is valid JSON");
        let reading = read(&body, fetched_at());

        assert_eq!(
            summary(&reading),
            [
                (WindowKind::Session, "Session", 81.5, 18_000, None),
                (WindowKind::Weekly, "Weekly", 47.0, 604_800, None),
                (
                    WindowKind::ModelWeekly,
                    "Weekly (Sonnet)",
                    22.0,
                    604_800,
                    Some("sonnet")
                ),
                (WindowKind::Weekly, "Weekly (Cowork)", 3.0, 604_800, None),
                (
                    WindowKind::Weekly,
                    "Weekly (Example model)",
                    9.0,
                    604_800,
                    None
                ),
                (
                    WindowKind::ModelWeekly,
                    "Weekly (Fable)",
                    64.0,
                    604_800,
                    Some("fable")
                ),
            ]
        );
        assert_eq!(
            reading.extra_usage,
            Some(ExtraUsage {
                enabled: true,
                used_pct: Some(25.0),
                used: Some(12.5),
                limit: Some(50.0),
                currency: Some("USD".into()),
                balance: None,
            })
        );
    }

    #[test]
    fn null_buckets_and_null_utilization_are_left_out() {
        let body = json!({
            "five_hour": null,
            "seven_day": { "utilization": null, "resets_at": "2099-01-07T00:00:00Z" },
            "seven_day_opus": { "utilization": true },
            "seven_day_sonnet": { "utilization": "12" }
        });

        assert!(read(&body, fetched_at()).windows.is_empty());
    }

    #[test]
    fn unknown_buckets_appear_by_name() {
        let body = json!({
            "five_hour_fable": { "utilization": 5.0, "resets_at": null },
            "seven_day_haiku": { "utilization": 6.0 },
            "monthly_mystery": { "utilization": 7.0, "resets_at": "2099-02-01T00:00:00Z" },
            "notice": { "message": "hello" }
        });
        let reading = read(&body, fetched_at());

        assert_eq!(
            summary(&reading),
            [
                (WindowKind::Session, "Session (Fable)", 5.0, 18_000, None),
                (WindowKind::Other, "Monthly mystery", 7.0, 0, None),
                (WindowKind::Weekly, "Weekly (Haiku)", 6.0, 604_800, None),
            ]
        );
        let monthly = reading
            .windows
            .iter()
            .find(|window| window.kind == WindowKind::Other)
            .expect("the unknown bucket");
        assert_eq!(monthly.resets_at, Some(ts("2099-02-01T00:00:00Z")));
    }

    #[test]
    fn scoped_limits_skip_all_models_duplicates_and_other_kinds() {
        let body = json!({
            "seven_day_opus": { "utilization": 10.0 },
            "limits": [
                { "kind": "weekly_scoped", "group": "weekly", "percent": 1.0,
                  "scope": { "model": { "id": "opus", "display_name": "Opus" } } },
                { "kind": "weekly_scoped", "group": "weekly", "percent": 2.0,
                  "scope": { "model": { "id": "claude-all-models", "display_name": "Everything" } } },
                { "kind": "weekly_scoped", "group": "weekly", "percent": 3.0,
                  "scope": { "model": { "display_name": "All models" } } },
                { "kind": "weekly", "group": "weekly", "percent": 4.0,
                  "scope": { "model": { "display_name": "Haiku" } } },
                { "kind": "weekly_scoped", "group": "weekly", "percent": false,
                  "scope": { "model": { "display_name": "Haiku" } } },
                { "kind": "weekly_scoped", "group": "weekly", "percent": 5.0,
                  "scope": { "model": { "display_name": "  " } } },
                { "kind": "weekly_scoped", "group": "weekly", "percent": 6.0,
                  "resets_at": "2099-01-07T00:00:00Z",
                  "scope": { "model": { "id": "fable", "display_name": "Fable" } } },
                { "kind": "weekly_scoped", "group": "weekly", "percent": 7.0,
                  "scope": { "model": { "id": "fable-again", "display_name": "Fable" } } }
            ]
        });
        let reading = read(&body, fetched_at());

        assert_eq!(
            summary(&reading),
            [
                (
                    WindowKind::ModelWeekly,
                    "Weekly (Opus)",
                    10.0,
                    604_800,
                    Some("opus")
                ),
                (
                    WindowKind::ModelWeekly,
                    "Weekly (Fable)",
                    6.0,
                    604_800,
                    Some("fable")
                ),
            ]
        );
        assert_eq!(
            reading.windows[1].resets_at,
            Some(ts("2099-01-07T00:00:00Z"))
        );
    }

    #[test]
    fn an_empty_response_has_no_windows() {
        assert_eq!(
            read(&json!({}), fetched_at()),
            Reading {
                windows: Vec::new(),
                extra_usage: None
            }
        );
        assert!(read(&json!([1, 2]), fetched_at()).windows.is_empty());
    }

    #[test]
    fn extra_usage_amounts_follow_decimal_places() {
        let usage = |places: Value| {
            extra_usage(&json!({
                "is_enabled": true,
                "monthly_limit": 50_000,
                "used_credits": 12_345,
                "currency": "JPY",
                "decimal_places": places
            }))
            .expect("extra usage")
        };

        assert_eq!(usage(json!(0)).used, Some(12_345.0));
        assert_eq!(usage(json!(3)).limit, Some(50.0));
        // Missing, or not a sensible count: cents, as CodexBar found.
        assert_eq!(usage(json!(null)).used, Some(123.45));
        assert_eq!(usage(json!(40)).used, Some(123.45));
        assert_eq!(usage(json!("2")).used, Some(123.45));
        assert_eq!(usage(json!(0)).currency.as_deref(), Some("JPY"));
    }

    #[test]
    fn a_currency_is_kept_as_its_three_letters_and_anything_else_is_unknown() {
        let currency = |currency: Option<Value>| {
            let mut answer = json!({
                "is_enabled": true,
                "monthly_limit": 2000,
                "used_credits": 150,
                "utilization": 7.5
            });
            if let Some(currency) = currency {
                answer["currency"] = currency;
            }
            extra_usage(&answer).expect("extra usage").currency
        };

        assert_eq!(currency(Some(json!("USD"))).as_deref(), Some("USD"));
        assert_eq!(currency(Some(json!(" eur "))).as_deref(), Some("EUR"));
        // Missing, or not a currency's three letters: unknown, and never taken for dollars, which
        // would be a wrong figure for any other currency (the manager's review of #108).
        for unknown in [
            None,
            Some(json!(null)),
            Some(json!("")),
            Some(json!("US Dollars")),
            Some(json!("US$")),
            Some(json!(840)),
        ] {
            assert_eq!(currency(unknown.clone()), None, "{unknown:?}");
        }
        let usage = extra_usage(&json!({
            "is_enabled": true,
            "monthly_limit": 2000,
            "used_credits": 150
        }))
        .expect("extra usage");
        assert_eq!((usage.used, usage.limit), (Some(1.5), Some(20.0)));
    }

    /// Each window Anthropic answers with, holding anything.
    fn answer() -> impl proptest::strategy::Strategy<Value = Value> {
        use proptest::prelude::*;
        prop::collection::vec(
            (
                prop::sample::select(
                    &[
                        "five_hour",
                        "seven_day",
                        "seven_day_opus",
                        "seven_day_sonnet",
                        "seven_day_oauth_apps",
                        "extra_usage",
                        "limits",
                    ][..],
                ),
                quotadial_core::testing::json(),
            ),
            0..7,
        )
        .prop_map(|windows| {
            Value::Object(
                windows
                    .into_iter()
                    .map(|(name, value)| (name.to_owned(), value))
                    .collect(),
            )
        })
    }

    proptest::proptest! {
        #[test]
        fn no_answer_makes_the_reading_panic(
            body in proptest::prop_oneof![quotadial_core::testing::json(), answer()],
        ) {
            let fetched_at = DateTime::parse_from_rfc3339("2026-10-25T01:12:00Z")
                .expect("a time")
                .with_timezone(&Utc);
            let reading = read(&body, fetched_at);
            for window in &reading.windows {
                proptest::prop_assert!(window.used_pct.is_finite());
            }
        }

        /// Whatever Anthropic sends as `currency`, the snapshot keeps three capital letters or
        /// nothing, and the line people read has the amounts with a dollar sign, a currency's code
        /// or neither, and nothing else of what was sent.
        #[test]
        fn a_currency_reaches_the_line_as_three_capital_letters_or_not_at_all(
            currency in currency(),
        ) {
            let usage = extra_usage(&json!({
                "is_enabled": true,
                "monthly_limit": 5000,
                "used_credits": 1250,
                "utilization": 25.0,
                "currency": currency,
            }))
            .expect("extra usage");
            let line = quotadial_core::text::extra_usage(&usage);
            match usage.currency.as_deref() {
                None => proptest::prop_assert_eq!(line, "12.50 of 50.00 used (25%)"),
                Some("USD") => proptest::prop_assert_eq!(line, "$12.50 of $50.00 used (25%)"),
                Some(code) => {
                    proptest::prop_assert!(
                        code.len() == 3 && code.bytes().all(|byte| byte.is_ascii_uppercase()),
                        "{code:?}"
                    );
                    proptest::prop_assert_eq!(line, format!("12.50 of 50.00 {code} used (25%)"));
                }
            }
        }
    }

    /// What Anthropic could send as `currency`: anything at all, any text, or a few letters with
    /// white space around them, as often a currency's three as not.
    fn currency() -> impl proptest::strategy::Strategy<Value = Value> {
        use proptest::prelude::*;
        prop_oneof![
            quotadial_core::testing::json(),
            any::<String>().prop_map(Value::String),
            "\\s{0,2}[A-Za-z]{2,4}\\s{0,2}".prop_map(Value::String),
        ]
    }
}
