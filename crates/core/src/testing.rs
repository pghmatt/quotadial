//! For other crates' tests, with the `testing` feature: arbitrary JSON for the property tests that
//! prove a parser of outside text never panics, whatever a provider, a log or a settings file
//! holds (#52). The keys and strings real answers use are mixed in, so arbitrary values land
//! where the parsers look rather than only beside them.

use proptest::prelude::*;
use serde_json::{Map, Value};

/// Any JSON value, nested up to four levels.
pub fn json() -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        any::<i64>().prop_map(Value::from),
        any::<u64>().prop_map(Value::from),
        any::<f64>()
            .prop_filter("JSON holds finite numbers", |number| number.is_finite())
            .prop_map(Value::from),
        text().prop_map(Value::String),
    ];
    leaf.prop_recursive(4, 64, 8, |inner| {
        prop_oneof![
            prop::collection::vec(inner.clone(), 0..8).prop_map(Value::Array),
            prop::collection::vec((key(), inner), 0..8)
                .prop_map(|entries| Value::Object(entries.into_iter().collect::<Map<_, _>>())),
        ]
    })
}

/// One line of a JSON Lines log: an arbitrary object, or arbitrary bytes.
pub fn line() -> impl Strategy<Value = Vec<u8>> {
    prop_oneof![
        json().prop_map(|value| value.to_string().into_bytes()),
        any::<Vec<u8>>(),
    ]
}

/// A key real answers, logs and settings use, or another.
fn key() -> impl Strategy<Value = String> {
    prop_oneof![
        3 => prop::sample::select(KEYS).prop_map(str::to_owned),
        1 => "[a-z_]{1,16}",
    ]
}

/// A string real answers hold, or any other.
fn text() -> impl Strategy<Value = String> {
    prop_oneof![
        any::<String>(),
        prop::sample::select(STRINGS).prop_map(str::to_owned),
    ]
}

/// The keys of the providers' answers, the CLIs' logs, the status pages, `config.json` and MCP,
/// from the fixtures and the code that reads them.
const KEYS: &[&str] = &[
    "access_token",
    "accessToken",
    "account",
    "account_id",
    "additional_rate_limits",
    "alert_webhooks",
    "arguments",
    "balance",
    "cache_creation_input_tokens",
    "cache_read_input_tokens",
    "cache_write_input_tokens",
    "cached_input_tokens",
    "claudeAiOauth",
    "cost_scan",
    "credits",
    "description",
    "display_name",
    "email",
    "expiresAt",
    "extra_config_dirs",
    "extra_usage",
    "five_hour",
    "has_credits",
    "history_days",
    "id",
    "id_token",
    "indicator",
    "info",
    "input_tokens",
    "is_enabled",
    "jsonrpc",
    "last_token_usage",
    "limit_window_seconds",
    "max_accounts",
    "message",
    "method",
    "model",
    "monthly_limit",
    "name",
    "organization",
    "output_tokens",
    "page",
    "params",
    "payload",
    "plan_type",
    "primary",
    "primary_window",
    "provider",
    "rate_limit",
    "rate_limits",
    "reasoning_output_tokens",
    "reset_after_seconds",
    "reset_at",
    "resets_at",
    "resets_in_seconds",
    "scope",
    "secondary",
    "secondary_window",
    "seven_day",
    "seven_day_opus",
    "seven_day_sonnet",
    "slack_webhook",
    "status",
    "status_pages",
    "subscriptionType",
    "timestamp",
    "tokens",
    "total_token_usage",
    "type",
    "updated_at",
    "usage",
    "used_credits",
    "used_percent",
    "utilization",
    "uuid",
    "window_minutes",
];

/// Strings real answers hold: times in their formats, numbers written as text, and names.
const STRINGS: &[&str] = &[
    "",
    "2026-10-25T01:12:00Z",
    "2026-10-25T01:12:00.123456+00:00",
    "2026-10-25 01:12:00",
    "not a time",
    "42",
    "-1",
    "1e400",
    "NaN",
    "user@example.com",
    "https://hooks.slack.com/services/T0/B0/X",
    "http://127.0.0.1:9/hook",
    "max",
    "pro",
    "plus",
    "team",
    "none",
    "minor",
    "major",
    "critical",
    "token_count",
    "turn_context",
    "event_msg",
    "response_item",
    "assistant",
    "claude-opus-5-5",
    "gpt-6-astra",
    "2.0",
    "tools/call",
    "initialize",
];
