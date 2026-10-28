//! The normalized model from the PRD:
//!
//! ```text
//! Account { provider, account_id, display_name, plan, org }
//! Window  { kind: session | weekly | model_weekly | daily | credits | other, label, used_pct,
//!           resets_at, window_seconds, scope_model?,
//!           source: api | local_log | cache | example | status_line, fetched_at, unconfirmed,
//!           pace? }
//! Pace    { pct_per_hour, runs_out_at? }
//! Snapshot { account, windows[], extra_usage?, status: ok | stale | auth_required |
//!            rate_limited | unsupported, error? }
//! ```
//!
//! Field names and enum values serialize exactly as written there. Fields marked `?` are optional
//! and are left out of JSON when absent. `Account.plan`, `Account.org` and `Window.resets_at` are
//! optional too: providers do not always report them (proposals P-001 and P-002 in
//! `docs/DECISIONS.md`, approved by Matt). `Window.pace` comes from the daemon's history (P-016).
//! The `example` source is the example build's alone (P-017, D-117): a release never sends it.
//! `status_line` is a number an official CLI handed to QuotaDial through its status line, not one
//! QuotaDial asked for (P-022, D-126).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// One signed-in account at one provider.
///
/// An account's identity is `provider` + `org` + `account_id`, so the same person in two
/// organizations is two accounts and their usage never blends.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Account {
    /// The provider's id, such as `"claude"`.
    // As `Provider::id` gives it.
    pub provider: String,
    /// The provider's own short name for itself, such as `Claude`, `Codex` or `Gemini CLI`, as the
    /// provider gives it, shown as given wherever the account is (P-020). An account read back from
    /// an older QuotaDial has none, and shows `provider` until the next poll.
    #[serde(default)]
    pub provider_name: String,
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
    /// Where the account's sign-in is, written for people, with `~` for the home folder, such as
    /// `~/.codex/auth.json`. Never a token. Left out when unknown, and where QuotaDial never looks
    /// for a sign-in, as for Claude and Antigravity (P-015, D-145).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sign_in: Option<String>,
    /// One sentence, in the provider's words and as the provider gives it, saying what QuotaDial
    /// does with this account's sign-in, such as `QuotaDial reads this account's sign-in from
    /// ~/.codex/auth.json. It only reads it and never changes it.`, shown as given (P-020). It never
    /// holds a token or anything from inside a sign-in, and a place in it is written with `~` for
    /// the home folder. An account read back from an older QuotaDial has none until the next poll.
    #[serde(default)]
    pub sign_in_use: String,
}

impl Account {
    /// The provider as people see it: its own name (P-020), or, for an account read back from an
    /// older QuotaDial that sent none, its id until the next poll replaces it.
    #[must_use]
    pub fn provider_label(&self) -> &str {
        if self.provider_name.is_empty() {
            &self.provider
        } else {
            &self.provider_name
        }
    }

    /// The name to show beside [`Account::provider_label`]: the display name, or none when it says
    /// nothing more, being empty or the label itself, whatever the case. So an account that is
    /// called as its provider is, such as the example build's "Example" accounts (D-117), shows its
    /// provider's label once ([`crate::text::name_beside`]).
    #[must_use]
    pub fn name_beside_provider(&self) -> Option<&str> {
        crate::text::name_beside(self.provider_label(), &self.display_name)
    }

    /// Whether `other` is the same account: the same provider, organization and account id, the
    /// three that make an account (D-024), whatever it is called or where its sign-in was found.
    #[must_use]
    pub fn same_as(&self, other: &Account) -> bool {
        self.provider == other.provider
            && self.account_id == other.account_id
            && self.org == other.org
    }
}

/// One usage limit with its own reset, such as a five-hour session or a weekly cap.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
    /// holds the reading as unconfirmed until a second fetch agrees. Left out of JSON when false.
    // The PRD's unconfirmed-0% rule (P-005).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub unconfirmed: bool,
    /// How fast the window is filling, from the daemon's history of it (P-016). Only the daemon
    /// keeps that history, so a direct fetch leaves it out, as does the daemon for a window with too
    /// few readings since it last reset.
    // `Pace::of` says when there are too few.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pace: Option<Pace>,
}

/// How fast a [`Window`] is filling: a straight line fitted through its readings since it last
/// reset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Pace {
    /// The share of the limit used per hour, in percentage points.
    pub pct_per_hour: f64,
    /// When the window reaches 100% at this pace, if that comes before it resets. Left out when
    /// it does not, or when the window is not filling.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runs_out_at: Option<DateTime<Utc>>,
}

/// The kinds of limit windows providers report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum WindowKind {
    /// A short rolling session, such as Claude's five-hour window.
    Session,
    /// A weekly limit across all models.
    Weekly,
    /// A weekly limit for one model, named in the window's `scope_model`.
    ModelWeekly,
    /// A daily limit.
    Daily,
    /// A credit balance.
    Credits,
    /// A limit QuotaDial does not know yet, shown by the name its provider gives it. Its length is
    /// unknown, so the window's `window_seconds` is 0.
    Other,
}

/// Where a [`Window`]'s numbers came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum Source {
    /// The provider's usage endpoint.
    Api,
    /// The official CLI's local session logs.
    LocalLog,
    /// QuotaDial's own store, from an earlier fetch.
    Cache,
    /// Made up by QuotaDial's example build, for pictures of the app; a release never sends it.
    // The fake provider's (D-117), so no picture of an example is taken for a real account.
    Example,
    /// Handed over by the official CLI's status line, which runs a command of QuotaDial's with its
    /// numbers, as Antigravity CLI's does (P-022). QuotaDial asked no one for them.
    StatusLine,
}

/// Everything QuotaDial knows about one account's usage at one moment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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

    /// The windows closest to their limits first, and of those, the one that resets soonest.
    #[must_use]
    pub fn windows_tightest_first(&self) -> Vec<&Window> {
        let mut sorted: Vec<&Window> = self.windows.iter().collect();
        sorted.sort_by(|a, b| {
            b.used_pct.total_cmp(&a.used_pct).then_with(|| {
                let reset = |window: &Window| window.resets_at.unwrap_or(DateTime::<Utc>::MAX_UTC);
                reset(a).cmp(&reset(b))
            })
        });
        sorted
    }

    /// The same snapshot with any token blanked out of the text a provider wrote: its error, where
    /// its account's sign-in is and what QuotaDial does with it (P-020). A provider never puts a
    /// token there; this pass makes sure, as it does for everything a provider writes.
    #[must_use]
    pub fn without_secrets(mut self) -> Self {
        let redacted = |text: &str| crate::redact_secrets(text).into_owned();
        self.error = self.error.as_deref().map(redacted);
        self.account.sign_in = self.account.sign_in.as_deref().map(redacted);
        self.account.sign_in_use = redacted(&self.account.sign_in_use);
        self
    }

    /// The same snapshot, with what an older QuotaDial left out of its account, the provider's name
    /// and what QuotaDial does with the sign-in, taken from `account`, the one discovery found just
    /// now (P-020). A snapshot an older QuotaDial kept in its cache is then shown as the provider
    /// names it now; what a provider said itself, such as where the sign-in was when it fetched,
    /// stays.
    #[must_use]
    pub fn completed_from(mut self, account: &Account) -> Self {
        if self.account.provider_name.is_empty() {
            self.account
                .provider_name
                .clone_from(&account.provider_name);
        }
        if self.account.sign_in_use.is_empty() {
            self.account.sign_in_use.clone_from(&account.sign_in_use);
        }
        self
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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
// The PRD names this field without a shape; see `docs/DECISIONS.md`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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

impl ExtraUsage {
    /// `text` as the currency of `used` and `limit`, when it is one: three ASCII letters once
    /// trimmed, as ISO 4217 writes a currency's code, in capitals, so ` usd ` is `USD`. Anything
    /// else, nothing at all included, is a currency QuotaDial cannot name, and so none: the amounts
    /// are then written with no symbol and no code, never as dollars, since calling an unknown
    /// currency dollars would be a wrong figure. A provider's currency is text from outside, held to
    /// this where the provider reads it, so a snapshot never carries more than three letters (the
    /// manager's review of #108).
    #[must_use]
    pub fn currency_code(text: &str) -> Option<String> {
        let code = text.trim();
        (code.len() == 3 && code.bytes().all(|byte| byte.is_ascii_alphabetic()))
            .then(|| code.to_ascii_uppercase())
    }
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
            provider_name: "Claude".into(),
            account_id: "00000000-0000-0000-0000-000000000001".into(),
            display_name: "user@example.com".into(),
            plan: Some("max".into()),
            org: Some("00000000-0000-0000-0000-000000000002".into()),
            sign_in: None,
            sign_in_use: "QuotaDial reads this account's sign-in from a test. It only reads it."
                .into(),
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
            pace: None,
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
    fn a_currency_is_three_letters_in_capitals_or_none() {
        for (text, code) in [
            ("USD", "USD"),
            (" usd ", "USD"),
            ("eur", "EUR"),
            ("\tJpY\n", "JPY"),
        ] {
            assert_eq!(
                ExtraUsage::currency_code(text).as_deref(),
                Some(code),
                "{text:?}"
            );
        }
        // Anything else is a currency QuotaDial cannot name, never taken for dollars: nothing at
        // all, too few or too many letters, words, signs, and letters outside ASCII (the euro
        // sign, an accented capital, and full-width letters).
        for text in [
            "",
            "   ",
            "US",
            "EURO",
            "US Dollars",
            "U S",
            "US$",
            "12A",
            "\u{20ac}",
            "\u{c9}UR",
            "\u{ff35}\u{ff33}\u{ff24}",
        ] {
            assert_eq!(ExtraUsage::currency_code(text), None, "{text:?}");
        }
    }

    #[test]
    fn field_names_match_the_prd() {
        let snapshot = Snapshot {
            account: account(),
            windows: vec![Window {
                pace: Some(Pace {
                    pct_per_hour: 2.5,
                    runs_out_at: Some(ts("2099-01-01T04:00:00Z")),
                }),
                ..window(WindowKind::ModelWeekly, Some("opus"), Source::Api)
            }],
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
            [
                "account_id",
                "display_name",
                "org",
                "plan",
                "provider",
                "provider_name",
                "sign_in_use"
            ]
        );
        assert_eq!(
            keys(&value["windows"][0]),
            [
                "fetched_at",
                "kind",
                "label",
                "pace",
                "resets_at",
                "scope_model",
                "source",
                "used_pct",
                "window_seconds"
            ]
        );
        assert_eq!(
            keys(&value["windows"][0]["pace"]),
            ["pct_per_hour", "runs_out_at"]
        );
        assert_eq!(
            value["windows"][0]["pace"]["runs_out_at"],
            json!("2099-01-01T04:00:00Z")
        );
    }

    #[test]
    fn an_account_names_its_provider_and_says_what_quotadial_does_with_its_sign_in() {
        let value = serde_json::to_value(account()).expect("serializes");

        assert_eq!(value["provider_name"], json!("Claude"));
        assert_eq!(
            value["sign_in_use"],
            json!("QuotaDial reads this account's sign-in from a test. It only reads it.")
        );
        let read_back: Account = serde_json::from_value(value).expect("deserializes");
        assert_eq!(read_back, account());
    }

    #[test]
    fn a_name_that_says_nothing_beyond_the_providers_label_is_not_shown_beside_it() {
        let named = |provider_name: &str, display_name: &str| Account {
            provider_name: provider_name.into(),
            display_name: display_name.into(),
            ..account()
        };

        assert_eq!(
            named("Claude", "user@example.com").name_beside_provider(),
            Some("user@example.com"),
            "a real address shows"
        );
        assert_eq!(named("Example", "Example").name_beside_provider(), None);
        assert_eq!(
            named("Example", "example").name_beside_provider(),
            None,
            "whatever the case"
        );
        assert_eq!(named("Claude", "").name_beside_provider(), None, "empty");
        assert_eq!(named("Claude", "  ").name_beside_provider(), None, "blank");
        // An account from an older QuotaDial is labelled by its id, and keeps its address.
        assert_eq!(
            named("", "user@example.com").name_beside_provider(),
            Some("user@example.com")
        );
        assert_eq!(named("", "claude").name_beside_provider(), None);
    }

    #[test]
    fn an_account_from_an_older_quotadial_reads_back_and_shows_its_providers_id() {
        let older = json!({
            "provider": "claude",
            "account_id": "00000000-0000-0000-0000-000000000001",
            "display_name": "user@example.com"
        });

        let read: Account = serde_json::from_value(older).expect("deserializes");

        assert_eq!(
            read.provider_name, "",
            "a serde default, as P-015 had for sign_in"
        );
        assert_eq!(read.sign_in_use, "");
        assert_eq!(
            read.provider_label(),
            "claude",
            "its id until the next poll, never a name from a table"
        );
        assert_eq!(account().provider_label(), "Claude");
    }

    #[test]
    fn the_redact_pass_covers_what_an_account_says_of_its_sign_in() {
        // Written in pieces, so no secret scanner takes this file for one holding a token.
        let token = concat!("sk-ant-", "oat01-", "abcdefghijklmnopqrstuvwxyz0123456789");
        let leaky = Snapshot {
            account: Account {
                sign_in: Some(format!("the variable holding {token}")),
                sign_in_use: format!("QuotaDial reads {token} and never changes it."),
                ..account()
            },
            windows: Vec::new(),
            extra_usage: None,
            status: Status::Ok,
            error: None,
        };

        let shown = leaky.without_secrets();

        assert_eq!(
            shown.account.sign_in.as_deref(),
            Some(format!("the variable holding {}", crate::REDACTED).as_str())
        );
        assert_eq!(
            shown.account.sign_in_use,
            format!("QuotaDial reads {} and never changes it.", crate::REDACTED)
        );
    }

    #[test]
    fn a_snapshot_kept_by_an_older_quotadial_is_completed_from_its_account_now() {
        let older: Account = serde_json::from_value(json!({
            "provider": "claude",
            "account_id": "00000000-0000-0000-0000-000000000001",
            "display_name": "user@example.com",
            "plan": "pro"
        }))
        .expect("an older account");
        let kept = Snapshot {
            account: older,
            windows: vec![window(WindowKind::Session, None, Source::Cache)],
            extra_usage: None,
            status: Status::Ok,
            error: None,
        };

        let completed = kept.clone().completed_from(&account());

        assert_eq!(completed.account.provider_name, account().provider_name);
        assert_eq!(completed.account.sign_in_use, account().sign_in_use);
        assert_eq!(
            completed.account.plan.as_deref(),
            Some("pro"),
            "the rest as kept"
        );
        assert_eq!(completed.windows, kept.windows);

        // What a provider said when it fetched, such as where the sign-in was then, stays.
        let fresh = Snapshot {
            account: Account {
                provider_name: "Claude".into(),
                sign_in_use: "QuotaDial reads this account's sign-in from ~/.claude-new.".into(),
                ..account()
            },
            ..kept
        };
        assert_eq!(fresh.clone().completed_from(&account()), fresh);
    }

    #[test]
    fn a_pace_that_does_not_run_out_leaves_out_its_time() {
        let window = Window {
            pace: Some(Pace {
                pct_per_hour: 0.0,
                runs_out_at: None,
            }),
            ..window(WindowKind::Session, None, Source::Api)
        };
        let value = serde_json::to_value(&window).expect("serializes");

        assert_eq!(keys(&value["pace"]), ["pct_per_hour"]);
        let read_back: Window = serde_json::from_value(value).expect("deserializes");
        assert_eq!(read_back, window);
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
            (Source::Example, "example"),
            (Source::StatusLine, "status_line"),
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
            [
                "account_id",
                "display_name",
                "provider",
                "provider_name",
                "sign_in_use"
            ]
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
    fn the_tightest_window_comes_first() {
        let window = |label: &str, used_pct, resets_at: Option<&str>| Window {
            kind: WindowKind::Weekly,
            label: label.into(),
            used_pct,
            resets_at: resets_at.map(ts),
            window_seconds: 604_800,
            scope_model: None,
            source: Source::Api,
            fetched_at: ts("2099-01-01T00:00:00Z"),
            unconfirmed: false,
            pace: None,
        };
        let snapshot = Snapshot {
            account: account(),
            windows: vec![
                window("Session", 10.0, Some("2099-01-01T05:00:00Z")),
                window("Weekly", 70.0, Some("2099-01-07T00:00:00Z")),
                window("Weekly (Opus)", 70.0, Some("2099-01-03T00:00:00Z")),
                window("Weekly (unknown reset)", 70.0, None),
            ],
            extra_usage: None,
            status: Status::Ok,
            error: None,
        };

        let order: Vec<&str> = snapshot
            .windows_tightest_first()
            .into_iter()
            .map(|window| window.label.as_str())
            .collect();

        assert_eq!(
            order,
            [
                "Weekly (Opus)",
                "Weekly",
                "Weekly (unknown reset)",
                "Session"
            ]
        );
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

    #[test]
    fn an_example_served_again_still_says_it_is_an_example() {
        let snapshot = Snapshot {
            account: account(),
            windows: vec![window(WindowKind::Session, None, Source::Example)],
            extra_usage: None,
            status: Status::Ok,
            error: None,
        };

        let cached = snapshot.clone().served_from_cache();

        assert_eq!(cached.windows[0].source, Source::Example);
        assert_eq!(cached.windows[0].fetched_at, snapshot.windows[0].fetched_at);
    }
}
