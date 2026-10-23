//! The normalized model from the PRD:
//!
//! ```text
//! Account { provider, account_id, display_name, plan, org }
//! Window  { kind: session | weekly | model_weekly | daily | credits | other, label, used_pct,
//!           resets_at, window_seconds, scope_model?, source: api | local_log | cache, fetched_at,
//!           unconfirmed }
//! Snapshot { account, windows[], extra_usage?, status: ok | stale | auth_required |
//!            rate_limited | unsupported, error? }
//! ```
//!
//! Field names and enum values serialize exactly as written there. Fields marked `?` are optional
//! and are left out of JSON when absent. `Account.plan`, `Account.org` and `Window.resets_at` are
//! optional too: providers do not always report them (proposals P-001 and P-002 in
//! `docs/DECISIONS.md`, approved by Matt).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// One signed-in account at one provider.
///
/// An account's identity is `provider` + `org` + `account_id`, so the same person in two
/// organizations is two accounts and their usage never blends.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Account {
    /// The provider id, as in [`Provider::id`](crate::Provider::id), for example `"claude"`.
    pub provider: String,
    /// The provider's id for this account.
    pub account_id: String,
    /// What to call the account, usually its email address.
    pub display_name: String,
    /// The subscription plan, as the provider names it, when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan: Option<String>,
    /// The organization or workspace the account belongs to, when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub org: Option<String>,
}

/// One usage limit with its own reset, such as a five-hour session or a weekly cap.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Window {
    /// Which kind of limit this is.
    pub kind: WindowKind,
    /// A short human-readable name, for example `"Session"` or `"Weekly (Opus)"`.
    pub label: String,
    /// How much of the limit is used, as a percentage of the limit.
    pub used_pct: f64,
    /// When the window resets, when the provider says.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resets_at: Option<DateTime<Utc>>,
    /// The length of the window in seconds, for example 18000 for five hours.
    pub window_seconds: u64,
    /// The model this limit applies to, for per-model windows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope_model: Option<String>,
    /// Where the number came from.
    pub source: Source,
    /// When the number was fetched. Every number QuotaDial shows carries its age.
    pub fetched_at: DateTime<Utc>,
    /// The window reads 0% where it read more on the last fetch, before its reset, so QuotaDial
    /// holds the reading as unconfirmed until a second fetch agrees (the PRD's unconfirmed-0% rule,
    /// P-005). Left out of JSON when false.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub unconfirmed: bool,
}

/// The kinds of limit windows providers report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WindowKind {
    /// A short rolling session, such as Claude's five-hour window.
    Session,
    /// A weekly limit across all models.
    Weekly,
    /// A weekly limit for one model, named in [`Window::scope_model`].
    ModelWeekly,
    /// A daily limit.
    Daily,
    /// A credit balance.
    Credits,
    /// A limit QuotaDial does not know yet, shown by the name its provider gives it. Its length is
    /// unknown, so [`Window::window_seconds`] is 0.
    Other,
}

/// Where a [`Window`]'s numbers came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    /// The provider's usage endpoint.
    Api,
    /// The official CLI's local session logs.
    LocalLog,
    /// QuotaDial's own store, from an earlier fetch.
    Cache,
}

/// Everything QuotaDial knows about one account's usage at one moment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    /// The account the snapshot describes.
    pub account: Account,
    /// The account's limit windows. Empty when the status says why no numbers are available.
    pub windows: Vec<Window>,
    /// Pay-as-you-go usage beyond the plan's limits, when the provider reports it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra_usage: Option<ExtraUsage>,
    /// Whether the numbers are current, and if not, why.
    pub status: Status,
    /// One sentence that explains a non-`ok` status and names the fix.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Snapshot {
    /// A snapshot with no windows for an account whose usage QuotaDial cannot read.
    ///
    /// `reason` is one sentence saying why, shown in place of a misleading 0%.
    #[must_use]
    pub fn unsupported(account: Account, reason: impl Into<String>) -> Self {
        Self::without_windows(account, Status::Unsupported, reason)
    }

    /// A snapshot with no windows for an account whose stored login is missing, expired or
    /// rejected.
    ///
    /// `reason` is one sentence that names the fix, such as running the official CLI to sign in.
    #[must_use]
    pub fn auth_required(account: Account, reason: impl Into<String>) -> Self {
        Self::without_windows(account, Status::AuthRequired, reason)
    }

    /// The same snapshot served again from QuotaDial's own store: every window that came from the
    /// API now says it came from the cache. `fetched_at` keeps the time of the original fetch.
    #[must_use]
    pub fn served_from_cache(mut self) -> Self {
        for window in &mut self.windows {
            if window.source == Source::Api {
                window.source = Source::Cache;
            }
        }
        self
    }

    fn without_windows(account: Account, status: Status, reason: impl Into<String>) -> Self {
        Self {
            account,
            windows: Vec::new(),
            extra_usage: None,
            status,
            error: Some(reason.into()),
        }
    }
}

/// How current a [`Snapshot`] is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    /// Fresh numbers from the latest fetch.
    Ok,
    /// Numbers from an earlier fetch, because the latest one did not succeed.
    Stale,
    /// The stored login is missing or expired. The user has to sign in with the official CLI.
    AuthRequired,
    /// The provider is rate-limiting requests. QuotaDial backs off and retries later.
    RateLimited,
    /// The provider reports no usage for this account or plan.
    Unsupported,
}

/// Pay-as-you-go usage beyond a plan's limits, such as Claude's extra usage.
///
/// The PRD names this field without a shape; see `docs/DECISIONS.md`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtraUsage {
    /// Whether extra usage is turned on for the account.
    pub enabled: bool,
    /// How much of the monthly extra-usage limit is used, as a percentage.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub used_pct: Option<f64>,
    /// The amount spent this month, in `currency`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub used: Option<f64>,
    /// The monthly spending limit, in `currency`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<f64>,
    /// The ISO 4217 currency code of `used` and `limit`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    /// Credits left to spend beyond the plan's limits, in the provider's own credit units rather
    /// than `currency`, when the provider reports a balance instead of a spending limit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub balance: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    fn ts(rfc3339: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(rfc3339)
            .expect("valid timestamp")
            .with_timezone(&Utc)
    }

    fn account() -> Account {
        Account {
            provider: "claude".into(),
            account_id: "00000000-0000-0000-0000-000000000001".into(),
            display_name: "user@example.com".into(),
            plan: Some("max".into()),
            org: Some("00000000-0000-0000-0000-000000000002".into()),
        }
    }

    fn window(kind: WindowKind, scope_model: Option<&str>, source: Source) -> Window {
        Window {
            kind,
            label: "Session".into(),
            used_pct: 12.5,
            resets_at: Some(ts("2099-01-01T05:00:00Z")),
            window_seconds: 18_000,
            scope_model: scope_model.map(Into::into),
            source,
            fetched_at: ts("2099-01-01T00:00:00Z"),
            unconfirmed: false,
        }
    }

    fn keys(value: &Value) -> Vec<&str> {
        let mut keys: Vec<&str> = value
            .as_object()
            .expect("JSON object")
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();
        keys
    }

    #[test]
    fn field_names_match_the_prd() {
        let snapshot = Snapshot {
            account: account(),
            windows: vec![window(WindowKind::ModelWeekly, Some("opus"), Source::Api)],
            extra_usage: Some(ExtraUsage {
                enabled: true,
                used_pct: Some(10.0),
                used: Some(2.0),
                limit: Some(20.0),
                currency: Some("USD".into()),
                balance: None,
            }),
            status: Status::Stale,
            error: Some("Showing the last numbers fetched.".into()),
        };
        let value = serde_json::to_value(&snapshot).expect("serializes");

        assert_eq!(
            keys(&value),
            ["account", "error", "extra_usage", "status", "windows"]
        );
        assert_eq!(
            keys(&value["account"]),
            ["account_id", "display_name", "org", "plan", "provider"]
        );
        assert_eq!(
            keys(&value["windows"][0]),
            [
                "fetched_at",
                "kind",
                "label",
                "resets_at",
                "scope_model",
                "source",
                "used_pct",
                "window_seconds"
            ]
        );
    }

    #[test]
    fn enum_values_match_the_prd() {
        let kinds = [
            (WindowKind::Session, "session"),
            (WindowKind::Weekly, "weekly"),
            (WindowKind::ModelWeekly, "model_weekly"),
            (WindowKind::Daily, "daily"),
            (WindowKind::Credits, "credits"),
            (WindowKind::Other, "other"),
        ];
        for (kind, expected) in kinds {
            assert_eq!(
                serde_json::to_value(kind).expect("serializes"),
                json!(expected)
            );
        }

        let sources = [
            (Source::Api, "api"),
            (Source::LocalLog, "local_log"),
            (Source::Cache, "cache"),
        ];
        for (source, expected) in sources {
            assert_eq!(
                serde_json::to_value(source).expect("serializes"),
                json!(expected)
            );
        }

        let statuses = [
            (Status::Ok, "ok"),
            (Status::Stale, "stale"),
            (Status::AuthRequired, "auth_required"),
            (Status::RateLimited, "rate_limited"),
            (Status::Unsupported, "unsupported"),
        ];
        for (status, expected) in statuses {
            assert_eq!(
                serde_json::to_value(status).expect("serializes"),
                json!(expected)
            );
        }
    }

    #[test]
    fn optional_fields_are_left_out_when_absent() {
        let snapshot = Snapshot {
            account: Account {
                plan: None,
                org: None,
                ..account()
            },
            windows: vec![Window {
                resets_at: None,
                ..window(WindowKind::Session, None, Source::Cache)
            }],
            extra_usage: None,
            status: Status::Ok,
            error: None,
        };
        let value = serde_json::to_value(&snapshot).expect("serializes");

        assert_eq!(keys(&value), ["account", "status", "windows"]);
        assert_eq!(
            keys(&value["account"]),
            ["account_id", "display_name", "provider"]
        );
        assert_eq!(
            keys(&value["windows"][0]),
            [
                "fetched_at",
                "kind",
                "label",
                "source",
                "used_pct",
                "window_seconds"
            ]
        );
    }

    #[test]
    fn absent_optional_fields_read_back_as_none() {
        let value = json!({
            "account": {
                "provider": "claude",
                "account_id": "00000000-0000-0000-0000-000000000001",
                "display_name": "user@example.com"
            },
            "windows": [{
                "kind": "model_weekly",
                "label": "Weekly (Opus)",
                "used_pct": 0.0,
                "window_seconds": 604_800,
                "source": "api",
                "fetched_at": "2099-01-01T00:00:00Z"
            }],
            "status": "ok"
        });
        let snapshot: Snapshot = serde_json::from_value(value).expect("deserializes");

        assert_eq!(snapshot.account.plan, None);
        assert_eq!(snapshot.account.org, None);
        assert_eq!(snapshot.windows[0].resets_at, None);
        assert_eq!(snapshot.windows[0].scope_model, None);
        assert!(!snapshot.windows[0].unconfirmed);
    }

    #[test]
    fn unconfirmed_appears_in_json_only_when_true() {
        let confirmed = serde_json::to_value(window(WindowKind::Session, None, Source::Api))
            .expect("serializes");
        assert!(confirmed.get("unconfirmed").is_none());

        let unconfirmed = Window {
            used_pct: 0.0,
            unconfirmed: true,
            ..window(WindowKind::Session, None, Source::Api)
        };
        let value = serde_json::to_value(&unconfirmed).expect("serializes");
        assert_eq!(value["unconfirmed"], json!(true));
        let read_back: Window = serde_json::from_value(value).expect("deserializes");
        assert_eq!(read_back, unconfirmed);
    }

    #[test]
    fn timestamps_serialize_as_rfc3339_utc() {
        let value = serde_json::to_value(window(WindowKind::Session, None, Source::Api))
            .expect("serializes");

        assert_eq!(value["resets_at"], json!("2099-01-01T05:00:00Z"));
        assert_eq!(value["fetched_at"], json!("2099-01-01T00:00:00Z"));
    }

    #[test]
    fn snapshots_round_trip_through_json() {
        let snapshot = Snapshot {
            account: account(),
            windows: vec![
                window(WindowKind::Session, None, Source::Api),
                window(WindowKind::ModelWeekly, Some("sonnet"), Source::LocalLog),
            ],
            extra_usage: None,
            status: Status::RateLimited,
            error: Some("Anthropic is rate-limiting usage requests; retrying in 5 minutes.".into()),
        };
        let json = serde_json::to_string(&snapshot).expect("serializes");

        assert_eq!(
            serde_json::from_str::<Snapshot>(&json).expect("deserializes"),
            snapshot
        );
    }

    #[test]
    fn unsupported_snapshot_explains_itself_instead_of_showing_zero() {
        let snapshot = Snapshot::unsupported(account(), "Free plans report no usage limits.");

        assert_eq!(snapshot.status, Status::Unsupported);
        assert!(snapshot.windows.is_empty());
        assert_eq!(
            snapshot.error.as_deref(),
            Some("Free plans report no usage limits.")
        );
    }

    #[test]
    fn auth_required_snapshot_names_the_fix() {
        let snapshot = Snapshot::auth_required(account(), "Run `claude` to sign in again.");

        assert_eq!(snapshot.status, Status::AuthRequired);
        assert!(snapshot.windows.is_empty());
        assert!(snapshot.extra_usage.is_none());
        assert_eq!(
            snapshot.error.as_deref(),
            Some("Run `claude` to sign in again.")
        );
    }

    #[test]
    fn served_from_cache_relabels_api_windows_and_keeps_their_age() {
        let snapshot = Snapshot {
            account: account(),
            windows: vec![
                window(WindowKind::Session, None, Source::Api),
                window(WindowKind::ModelWeekly, Some("opus"), Source::LocalLog),
            ],
            extra_usage: None,
            status: Status::Ok,
            error: None,
        };
        let cached = snapshot.clone().served_from_cache();

        assert_eq!(cached.windows[0].source, Source::Cache);
        assert_eq!(cached.windows[0].fetched_at, snapshot.windows[0].fetched_at);
        assert_eq!(cached.windows[1].source, Source::LocalLog);
        assert_eq!(cached.status, Status::Ok);
    }
}
