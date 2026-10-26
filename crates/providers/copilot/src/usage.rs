//! GitHub's answer from `copilot_internal/user`, as windows (D-127; the manager's Copilot rules on
//! #73):
//!
//! - **AI credits:** on a plan billed in GitHub AI Credits (`token_based_billing`), the month's
//!   credits, `quota_snapshots.premium_interactions`, as a `credits` window, "AI credits", resetting
//!   at 00:00 UTC on the 1st, its length the month's.
//! - **Premium requests:** on an annual plan still billed by request, the same snapshot counts the
//!   month's premium requests: a window of its own, "Premium requests", reset and length the same.
//! - **Copilot Free:** its chat messages and code completions, from `monthly_quotas` (the month's
//!   allowance) and `limited_user_quotas` (what is left of it), each a window named for what it is,
//!   "Chat messages" and "Code completions", resetting at `limited_user_reset_date`. The
//!   `quota_snapshots` of the same names stand in for them when they carry a share.
//! - A share used is 100 less `percent_remaining`, or, without it, what `remaining` leaves of
//!   `entitlement`: from 0 to 100, as every window's is, so an account GitHub lets go past its
//!   allowance shows its window full.
//! - An allowance GitHub marks `unlimited` is said in words, never shown as a share. A snapshot with
//!   no allowance and nothing left (`entitlement` and `remaining` both 0), GitHub's placeholder for
//!   credits billed elsewhere, as for a Copilot Business seat, is no meter rather than 0%, and is
//!   said in words too. The words go in the account's line when there is no window, and after its
//!   sign-in sentence beside windows, as D-126 does for Antigravity's amounts, so nothing is left out
//!   without a word (#52).
//! - Any other snapshot GitHub adds shows by its name when it has a share, its length unknown
//!   (`other`).
//!
//! The reset is the snapshot's own `quota_reset_at`, else the answer's `quota_reset_date_utc` or
//! `quota_reset_date`, else, for AI credits and premium requests, 00:00 UTC on the 1st of next
//! month, as GitHub's billing docs say they reset.
//!
//! The fields are those GitHub's own Copilot clients read, as CodexBar ports them
//! (`Sources/CodexBarCore/CopilotUsageModels.swift`: `QuotaSnapshot`, `isPlaceholder`,
//! `makeQuotaSnapshots` and `preferredQuotaSnapshot`; `Providers/Copilot/CopilotUsageFetcher.swift`:
//! `makeRateWindow` and `parseQuotaResetDate`; at 4654968e9). Every fixture is made up: what
//! `entitlement` counts on a plan billed in credits, and the answer's shape for a real account, are
//! what #51's live check records.

use chrono::{DateTime, Datelike, Months, NaiveDate, Utc};
use quotadial_core::{Source, Window, WindowKind, outside_text};
use serde_json::Value;

/// The month's AI credits, on a plan billed in them.
pub(crate) const AI_CREDITS: &str = "AI credits";
/// The month's premium requests, on an annual plan still billed by request.
pub(crate) const PREMIUM_REQUESTS: &str = "Premium requests";
/// Copilot Free's chat messages.
pub(crate) const CHAT: &str = "Chat messages";
/// Copilot Free's code completions.
pub(crate) const COMPLETIONS: &str = "Code completions";

/// The longest plan or snapshot name shown, from GitHub's text.
const MAX_NAME_CHARS: usize = 64;

/// What an answer says.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Reading {
    /// AI credits or premium requests first, then chat messages, code completions and any other.
    pub(crate) windows: Vec<Window>,
    /// What GitHub gives the account without limit, each as a sentence says it.
    pub(crate) unlimited: Vec<String>,
    /// What GitHub sets no allowance of, each as a sentence says it.
    pub(crate) no_allowance: Vec<String>,
    /// The plan, as GitHub names it, such as `individual_pro`.
    pub(crate) plan: Option<String>,
    /// Whom GitHub says the numbers belong to, when it says, to compare with the sign-in. Never
    /// shown.
    pub(crate) login: Option<String>,
}

/// What one allowance shows.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Meter {
    /// The share used, in percent.
    Share(f64),
    Unlimited,
    /// No allowance and nothing left: GitHub's placeholder.
    NoAllowance,
    /// Nothing to go on.
    Unknown,
}

/// Maps a `copilot_internal/user` answer fetched at `fetched_at`.
pub(crate) fn read(body: &Value, fetched_at: DateTime<Utc>) -> Reading {
    let snapshots = body
        .get("quota_snapshots")
        .filter(|value| value.is_object());
    let snapshot = |key: &str| snapshots.and_then(|snapshots| snapshots.get(key));
    let mut reading = Reading {
        windows: Vec::new(),
        unlimited: Vec::new(),
        no_allowance: Vec::new(),
        plan: name(body, "copilot_plan"),
        login: text(body, "login"),
    };

    // The month's AI credits, or premium requests on a plan still billed by request.
    let billed_in_credits = body
        .get("token_based_billing")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let premium = snapshot("premium_interactions");
    let label = if billed_in_credits {
        AI_CREDITS
    } else {
        PREMIUM_REQUESTS
    };
    match meter(premium) {
        Meter::Share(used_pct) => {
            let resets_at = reset_of(premium)
                .or_else(|| answer_reset(body))
                .or_else(|| first_of_next_month(fetched_at));
            reading.windows.push(window(
                WindowKind::Credits,
                label,
                used_pct,
                resets_at,
                fetched_at,
            ));
        }
        Meter::Unlimited => reading.unlimited.push(said(label)),
        Meter::NoAllowance => reading.no_allowance.push(said(label)),
        Meter::Unknown => {}
    }

    // Chat messages and code completions: Copilot Free's, or a snapshot's that carries a share.
    for (key, label) in [("chat", CHAT), ("completions", COMPLETIONS)] {
        let direct = snapshot(key);
        let (shown, resets_at) = match (meter(direct), free_share(body, key)) {
            (Meter::Share(used_pct), _) => (
                Meter::Share(used_pct),
                reset_of(direct).or_else(|| answer_reset(body)),
            ),
            (_, Some(used_pct)) => (Meter::Share(used_pct), free_reset(body)),
            (direct, None) => (direct, None),
        };
        match shown {
            Meter::Share(used_pct) => reading.windows.push(window(
                WindowKind::Credits,
                label,
                used_pct,
                resets_at,
                fetched_at,
            )),
            Meter::Unlimited => reading.unlimited.push(said(label)),
            // A placeholder for chat or completions says nothing a person needs.
            Meter::NoAllowance | Meter::Unknown => {}
        }
    }

    // Any other snapshot GitHub adds, by its name.
    for (key, value) in snapshots.and_then(Value::as_object).into_iter().flatten() {
        if matches!(
            key.as_str(),
            "premium_interactions" | "chat" | "completions"
        ) {
            continue;
        }
        let Some(label) = humanize(key) else {
            continue;
        };
        if reading.windows.iter().any(|window| window.label == label) {
            continue;
        }
        match meter(Some(value)) {
            Meter::Share(used_pct) => reading.windows.push(Window {
                window_seconds: 0,
                ..window(
                    WindowKind::Other,
                    &label,
                    used_pct,
                    reset_of(Some(value)).or_else(|| answer_reset(body)),
                    fetched_at,
                )
            }),
            Meter::Unlimited => reading.unlimited.push(format!("\"{label}\"")),
            Meter::NoAllowance | Meter::Unknown => {}
        }
    }
    reading
}

/// A window from GitHub's answer, as long as the month before its reset.
fn window(
    kind: WindowKind,
    label: &str,
    used_pct: f64,
    resets_at: Option<DateTime<Utc>>,
    fetched_at: DateTime<Utc>,
) -> Window {
    Window {
        kind,
        label: label.to_owned(),
        used_pct,
        resets_at,
        window_seconds: resets_at.map_or(0, month_before),
        scope_model: None,
        source: Source::Api,
        fetched_at,
        unconfirmed: false,
        pace: None,
    }
}

/// What one of `quota_snapshots` shows (CodexBar's `QuotaSnapshot`).
fn meter(snapshot: Option<&Value>) -> Meter {
    let Some(snapshot) = snapshot.filter(|snapshot| snapshot.is_object()) else {
        return Meter::Unknown;
    };
    if snapshot.get("unlimited").and_then(Value::as_bool) == Some(true) {
        return Meter::Unlimited;
    }
    let entitlement = number(snapshot.get("entitlement"));
    let remaining = number(snapshot.get("remaining"));
    // Even beside `percent_remaining: 100`, which would otherwise read as 0% used.
    if entitlement == Some(0.0) && remaining == Some(0.0) {
        return Meter::NoAllowance;
    }
    let left =
        number(snapshot.get("percent_remaining")).or_else(|| match (entitlement, remaining) {
            (Some(entitlement), Some(remaining)) if entitlement > 0.0 => {
                Some(remaining / entitlement * 100.0)
            }
            _ => None,
        });
    left.map_or(Meter::Unknown, |left| Meter::Share(used(left)))
}

/// Copilot Free's share used of `key`, `chat` or `completions`: what `limited_user_quotas` leaves of
/// `monthly_quotas` (CodexBar's `makeQuotaSnapshots`). None without both, or with no allowance.
fn free_share(body: &Value, key: &str) -> Option<f64> {
    let monthly = number(body.get("monthly_quotas")?.get(key))?;
    let left = number(body.get("limited_user_quotas")?.get(key))?;
    (monthly > 0.0).then(|| used((left.max(0.0) / monthly * 100.0).min(100.0)))
}

/// The share used of an allowance with `left_pct` of it left, from 0 to 100, to a millionth of a
/// point.
fn used(left_pct: f64) -> f64 {
    let used = 100.0 - left_pct.clamp(0.0, 100.0);
    (used * 1e6).round() / 1e6
}

/// A finite number, written as a number or as text, as GitHub's clients accept them.
fn number(value: Option<&Value>) -> Option<f64> {
    let value = value?;
    value
        .as_f64()
        .or_else(|| value.as_str()?.trim().parse().ok())
        .filter(|number: &f64| number.is_finite())
}

/// A snapshot's own reset, `quota_reset_at`, in seconds since 1970.
fn reset_of(snapshot: Option<&Value>) -> Option<DateTime<Utc>> {
    let seconds = number(snapshot?.get("quota_reset_at"))?;
    // 0 says none, and nothing sensible is a trillion seconds away.
    if seconds <= 0.0 || seconds >= 1e12 {
        return None;
    }
    DateTime::from_timestamp(seconds as i64, 0)
}

/// The answer's reset: `quota_reset_date_utc`, or `quota_reset_date` at 00:00 UTC.
fn answer_reset(body: &Value) -> Option<DateTime<Utc>> {
    time(body.get("quota_reset_date_utc")).or_else(|| time(body.get("quota_reset_date")))
}

/// Copilot Free's reset, `limited_user_reset_date`, or the answer's.
fn free_reset(body: &Value) -> Option<DateTime<Utc>> {
    time(body.get("limited_user_reset_date")).or_else(|| answer_reset(body))
}

/// A time written as RFC 3339, or a date, taken as 00:00 UTC.
fn time(value: Option<&Value>) -> Option<DateTime<Utc>> {
    let text = value?.as_str()?.trim();
    if let Ok(at) = DateTime::parse_from_rfc3339(text) {
        return Some(at.with_timezone(&Utc));
    }
    NaiveDate::parse_from_str(text, "%Y-%m-%d")
        .ok()?
        .and_hms_opt(0, 0, 0)
        .map(|at| at.and_utc())
}

/// 00:00 UTC on the 1st of the month after `at`'s, when GitHub's AI credits and premium requests
/// reset.
fn first_of_next_month(at: DateTime<Utc>) -> Option<DateTime<Utc>> {
    let first = NaiveDate::from_ymd_opt(at.year(), at.month(), 1)?;
    Some(
        first
            .checked_add_months(Months::new(1))?
            .and_hms_opt(0, 0, 0)?
            .and_utc(),
    )
}

/// The length in seconds of the month that ends at `reset`, such as 2,678,400 for January's.
fn month_before(reset: DateTime<Utc>) -> u64 {
    reset
        .checked_sub_months(Months::new(1))
        .and_then(|start| u64::try_from((reset - start).num_seconds()).ok())
        .unwrap_or(0)
}

/// `label` as a sentence says it: `AI credits` as it is, `Chat messages` as `chat messages`.
fn said(label: &str) -> String {
    let first_word = label.split(' ').next().unwrap_or_default();
    if first_word.chars().all(|char| char.is_ascii_uppercase()) {
        return label.to_owned();
    }
    let mut chars = label.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_lowercase().chain(chars).collect()
    })
}

/// `"agent_requests"` → `"Agent requests"`, cleaned like any text from GitHub: tokens are blanked
/// before `_` becomes a space, since GitHub's own tokens are written with one.
fn humanize(key: &str) -> Option<String> {
    let words = outside_text(
        &quotadial_core::redact_secrets(key).replace('_', " "),
        MAX_NAME_CHARS,
    )?;
    let mut chars = words.chars();
    let first = chars.next()?;
    Some(first.to_uppercase().chain(chars).collect())
}

/// A name from GitHub's text that QuotaDial shows, such as the plan: on one line, with any token
/// blanked out, at most 64 characters.
fn name(value: &Value, key: &str) -> Option<String> {
    outside_text(value.get(key)?.as_str()?, MAX_NAME_CHARS)
}

/// A non-blank string field.
fn text(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)?
        .as_str()
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_owned)
}

#[cfg(test)]
pub(crate) mod tests {
    use serde_json::json;

    use super::*;

    /// The made-up answers, one for each shape GitHub's clients read.
    pub(crate) const PRO_CREDITS: &str =
        include_str!("../../../../fixtures/copilot/user-pro-credits.synthetic.json");
    pub(crate) const ANNUAL_PREMIUM: &str =
        include_str!("../../../../fixtures/copilot/user-annual-premium.synthetic.json");
    pub(crate) const FREE: &str =
        include_str!("../../../../fixtures/copilot/user-free.synthetic.json");
    pub(crate) const BUSINESS: &str =
        include_str!("../../../../fixtures/copilot/user-business.synthetic.json");
    pub(crate) const UNLIMITED: &str =
        include_str!("../../../../fixtures/copilot/user-unlimited.synthetic.json");

    pub(crate) fn ts(rfc3339: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(rfc3339)
            .expect("valid timestamp")
            .with_timezone(&Utc)
    }

    fn now() -> DateTime<Utc> {
        ts("2026-01-15T12:00:00Z")
    }

    fn reading(fixture: &str) -> Reading {
        read(&serde_json::from_str(fixture).expect("JSON"), now())
    }

    /// The month of January, 2026.
    const JANUARY: u64 = 31 * 24 * 60 * 60;

    #[test]
    fn a_plan_billed_in_ai_credits_shows_the_month_s_credits_and_says_what_is_unlimited() {
        let reading = reading(PRO_CREDITS);

        assert_eq!(
            reading.windows,
            [Window {
                kind: WindowKind::Credits,
                label: "AI credits".into(),
                used_pct: 25.0,
                resets_at: Some(ts("2026-02-01T00:00:00Z")),
                window_seconds: JANUARY,
                scope_model: None,
                source: Source::Api,
                fetched_at: now(),
                unconfirmed: false,
                pace: None,
            }]
        );
        assert_eq!(reading.unlimited, ["chat messages", "code completions"]);
        assert!(reading.no_allowance.is_empty());
        assert_eq!(reading.plan.as_deref(), Some("individual_pro"));
        assert_eq!(reading.login, None);
    }

    #[test]
    fn premium_requests_on_an_annual_plan_are_their_own_window() {
        let reading = reading(ANNUAL_PREMIUM);

        let summary: Vec<_> = reading
            .windows
            .iter()
            .map(|window| {
                (
                    window.kind,
                    window.label.as_str(),
                    window.used_pct,
                    window.window_seconds,
                )
            })
            .collect();
        assert_eq!(
            summary,
            [(WindowKind::Credits, "Premium requests", 60.0, JANUARY)]
        );
        assert_eq!(
            reading.windows[0].resets_at,
            Some(ts("2026-02-01T00:00:00Z"))
        );
        assert_eq!(reading.plan.as_deref(), Some("individual"));
    }

    #[test]
    fn copilot_free_s_chat_and_completions_are_windows_named_for_what_they_are() {
        let reading = reading(FREE);

        let summary: Vec<_> = reading
            .windows
            .iter()
            .map(|window| (window.label.as_str(), window.used_pct, window.resets_at))
            .collect();
        let reset = Some(ts("2026-02-01T00:00:00Z"));
        assert_eq!(
            summary,
            [
                ("Chat messages", 20.0, reset),
                ("Code completions", 25.0, reset)
            ]
        );
        assert!(
            reading.windows.iter().all(
                |window| window.kind == WindowKind::Credits && window.window_seconds == JANUARY
            )
        );
        assert!(reading.unlimited.is_empty() && reading.no_allowance.is_empty());
    }

    #[test]
    fn a_placeholder_with_no_allowance_is_no_meter_and_said_in_words() {
        let reading = reading(BUSINESS);

        assert!(
            reading.windows.is_empty(),
            "never 0%: {:?}",
            reading.windows
        );
        assert_eq!(reading.no_allowance, ["AI credits"]);
        assert_eq!(reading.unlimited, ["chat messages", "code completions"]);
        assert_eq!(reading.plan.as_deref(), Some("business"));
    }

    #[test]
    fn unlimited_is_said_in_words_and_never_a_share() {
        let reading = reading(UNLIMITED);

        assert!(reading.windows.is_empty());
        assert_eq!(
            reading.unlimited,
            ["premium requests", "chat messages", "code completions"]
        );
    }

    #[test]
    fn a_share_comes_from_what_is_left_and_is_full_past_the_allowance() {
        let share = |snapshot: Value| {
            let body = json!({ "token_based_billing": true, "quota_snapshots": {
                "premium_interactions": snapshot } });
            read(&body, now())
                .windows
                .first()
                .map(|window| window.used_pct)
        };

        assert_eq!(
            share(json!({ "entitlement": 1500, "remaining": 1200 })),
            Some(20.0),
            "no percent_remaining: what is left of the allowance"
        );
        assert_eq!(
            share(json!({ "entitlement": "7000", "remaining": "700", "percent_remaining": "10" })),
            Some(90.0),
            "numbers written as text"
        );
        assert_eq!(
            share(json!({ "percent_remaining": -20 })),
            Some(100.0),
            "past the allowance, full, as every share is at most 100"
        );
        assert_eq!(share(json!({ "percent_remaining": -1e300 })), Some(100.0));
        assert_eq!(share(json!({ "percent_remaining": 140 })), Some(0.0));
        assert_eq!(
            share(json!({ "percent_remaining": 66.6667 })),
            Some(33.3333)
        );
        assert_eq!(share(json!({ "entitlement": 0, "remaining": 5 })), None);
        assert_eq!(share(json!({ "quota_id": "premium_interactions" })), None);
        assert_eq!(share(json!("not an object")), None);
    }

    #[test]
    fn a_snapshot_s_share_comes_before_free_s_counts_unless_it_is_unlimited() {
        let free = json!({ "monthly_quotas": { "chat": 50 }, "limited_user_quotas": { "chat": 10 },
            "limited_user_reset_date": "2026-01-20" });
        let with = |chat: Value| {
            let mut body = free.clone();
            body["quota_snapshots"] = json!({ "chat": chat });
            let reading = read(&body, now());
            (
                reading
                    .windows
                    .first()
                    .map(|window| (window.used_pct, window.resets_at)),
                reading.unlimited,
            )
        };

        assert_eq!(
            with(json!({ "percent_remaining": 50, "quota_reset_at": 1_769_904_000 })),
            (Some((50.0, Some(ts("2026-02-01T00:00:00Z")))), Vec::new())
        );
        assert_eq!(
            with(json!({ "unlimited": true })),
            (Some((80.0, Some(ts("2026-01-20T00:00:00Z")))), Vec::new()),
            "Free's own counts, as GitHub's clients prefer them"
        );
        let reset_unknown = json!({ "monthly_quotas": { "chat": 50 },
            "limited_user_quotas": { "chat": 50 } });
        let reading = read(&reset_unknown, now());
        assert_eq!(reading.windows[0].used_pct, 0.0);
        assert_eq!(reading.windows[0].resets_at, None);
        assert_eq!(reading.windows[0].window_seconds, 0, "its length unknown");
        let no_allowance =
            json!({ "monthly_quotas": { "chat": 0 }, "limited_user_quotas": { "chat": 0 } });
        assert!(read(&no_allowance, now()).windows.is_empty());
    }

    #[test]
    fn a_window_is_as_long_as_the_month_before_its_reset() {
        let length = |reset: &str| month_before(ts(reset)) / (24 * 60 * 60);

        assert_eq!(length("2026-02-01T00:00:00Z"), 31, "January");
        assert_eq!(length("2026-03-01T00:00:00Z"), 28, "February");
        assert_eq!(length("2028-03-01T00:00:00Z"), 29, "a leap February");
        assert_eq!(length("2026-05-01T00:00:00Z"), 30, "April");
        assert_eq!(
            length("2026-03-31T00:00:00Z"),
            31,
            "from the last day February has"
        );
    }

    #[test]
    fn with_no_reset_in_the_answer_credits_reset_on_the_1st_as_github_s_docs_say() {
        let body = json!({ "token_based_billing": true, "quota_snapshots": {
            "premium_interactions": { "entitlement": 1500, "remaining": 1500, "quota_reset_at": 0 } } });

        let window = read(&body, ts("2026-12-31T23:59:59Z")).windows[0].clone();

        assert_eq!(window.used_pct, 0.0);
        assert_eq!(window.resets_at, Some(ts("2027-01-01T00:00:00Z")));
        assert_eq!(window.window_seconds, JANUARY, "December's length");
        assert_eq!(
            first_of_next_month(ts("2026-01-15T12:00:00Z")),
            Some(ts("2026-02-01T00:00:00Z"))
        );
    }

    #[test]
    fn a_snapshot_s_own_reset_comes_first_then_the_answer_s() {
        let body = |snapshot: Value| {
            json!({ "token_based_billing": true, "quota_reset_date": "2026-02-01",
                "quota_snapshots": { "premium_interactions": snapshot } })
        };
        let reset = |snapshot: Value| read(&body(snapshot), now()).windows[0].resets_at;

        assert_eq!(
            reset(json!({ "percent_remaining": 50, "quota_reset_at": 1_772_323_200 })),
            Some(ts("2026-03-01T00:00:00Z"))
        );
        assert_eq!(
            reset(json!({ "percent_remaining": 50, "quota_reset_at": "never" })),
            Some(ts("2026-02-01T00:00:00Z")),
            "the answer's date, at 00:00 UTC"
        );
        let utc = json!({ "token_based_billing": true,
            "quota_reset_date_utc": "2026-02-01T00:00:00.000Z", "quota_reset_date": "2099-01-01",
            "quota_snapshots": { "premium_interactions": { "percent_remaining": 50 } } });
        assert_eq!(
            read(&utc, now()).windows[0].resets_at,
            Some(ts("2026-02-01T00:00:00Z"))
        );
    }

    #[test]
    fn another_snapshot_shows_by_its_name_and_names_from_github_are_cleaned() {
        let token = concat!("ghp_", "MadeUpClassicTokenForQuotaDial012345");
        let body = json!({
            "copilot_plan": format!("pro\n{}", "x".repeat(100)),
            "login": "  example-user ",
            "quota_reset_date": "2026-02-01",
            "quota_snapshots": {
                "agent_requests": { "percent_remaining": 90, "quota_reset_at": 1_769_904_000 },
                "review_runs": { "unlimited": true },
                format!("leak {token}"): { "percent_remaining": 10 },
                "Chat messages": { "percent_remaining": 1 },
                "chat": { "percent_remaining": 75 },
                "placeholder": { "entitlement": 0, "remaining": 0 },
            }
        });

        let reading = read(&body, now());

        let labels: Vec<(&str, WindowKind, u64)> = reading
            .windows
            .iter()
            .map(|window| (window.label.as_str(), window.kind, window.window_seconds))
            .collect();
        assert_eq!(
            labels,
            [
                ("Chat messages", WindowKind::Credits, JANUARY),
                ("Agent requests", WindowKind::Other, 0),
                ("Leak [redacted token]", WindowKind::Other, 0),
            ],
            "known first; another by its name, with no length, a token blanked out, and none twice"
        );
        assert_eq!(reading.unlimited, ["\"Review runs\""]);
        let plan = reading.plan.expect("plan");
        assert!(
            plan.starts_with("pro x") && plan.chars().count() == 65,
            "{plan}"
        );
        assert_eq!(reading.login.as_deref(), Some("example-user"));
    }

    #[test]
    fn labels_are_said_in_a_sentence_as_people_write_them() {
        assert_eq!(said(AI_CREDITS), "AI credits");
        assert_eq!(said(PREMIUM_REQUESTS), "premium requests");
        assert_eq!(said(CHAT), "chat messages");
        assert_eq!(said(COMPLETIONS), "code completions");
        assert_eq!(humanize("  "), None);
    }

    #[test]
    fn an_answer_with_nothing_in_it_shows_nothing() {
        let reading = read(&json!({}), now());

        assert!(reading.windows.is_empty());
        assert!(reading.unlimited.is_empty() && reading.no_allowance.is_empty());
        assert_eq!(reading.plan, None);
        assert_eq!(reading.login, None);
    }

    proptest::proptest! {
        #[test]
        fn no_answer_makes_the_reading_panic(
            body in quotadial_core::testing::json(),
            premium in quotadial_core::testing::json(),
            chat in quotadial_core::testing::json(),
            monthly in quotadial_core::testing::json(),
            left in quotadial_core::testing::json(),
            reset in quotadial_core::testing::json(),
            billed in proptest::option::of(proptest::bool::ANY),
        ) {
            let fetched_at = ts("2026-10-25T01:12:00Z");
            let _ = read(&body, fetched_at);
            let shaped = json!({
                "copilot_plan": "individual_pro",
                "token_based_billing": billed,
                "quota_reset_date_utc": reset,
                "quota_snapshots": {
                    "premium_interactions": premium,
                    "chat": chat,
                    "other": premium,
                },
                "monthly_quotas": monthly,
                "limited_user_quotas": left,
                "limited_user_reset_date": reset,
            });
            let reading = read(&shaped, fetched_at);
            let mut labels = Vec::new();
            for window in &reading.windows {
                proptest::prop_assert!(window.used_pct.is_finite());
                proptest::prop_assert!((0.0..=100.0).contains(&window.used_pct));
                proptest::prop_assert!(!labels.contains(&window.label));
                labels.push(window.label.clone());
            }
        }
    }
}
