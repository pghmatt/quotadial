//! The words QuotaDial uses for times, sources and statuses. The CLI, the tray app and its
//! notifications all say them the same way.

use std::path::Path;

use chrono::{DateTime, TimeDelta, Utc};

use crate::{Cost, ExtraUsage, ModelCost, Pace, Source, Status};

/// `path` with `~` for the home folder, as people write it, such as `~/.codex/auth.json`.
#[must_use]
pub fn tilde(path: &Path, home: Option<&Path>) -> String {
    match home.and_then(|home| path.strip_prefix(home).ok()) {
        Some(rest) if rest.as_os_str().is_empty() => "~".to_owned(),
        Some(rest) => format!("~/{}", rest.display()),
        None => path.display().to_string(),
    }
}

/// For example `resets in 2 h 10 min`, or `reset 3 min ago` when the numbers predate a reset.
#[must_use]
pub fn resets(resets_at: Option<DateTime<Utc>>, now: DateTime<Utc>) -> String {
    match resets_at {
        None => "no reset time reported".to_owned(),
        Some(at) if at > now => format!("resets in {}", duration(at - now)),
        Some(at) => format!("reset {} ago", duration(now - at)),
    }
}

/// A window's pace, given when it resets: for example `at 8.5%/h, runs out in 1 h 30 min, 40 min
/// before it resets`, or `at 2.1%/h, lasts until it resets`.
#[must_use]
pub fn pace(pace: &Pace, resets_at: Option<DateTime<Utc>>, now: DateTime<Utc>) -> String {
    let rate = match pace.pct_per_hour {
        rate if rate >= 10.0 => format!("{rate:.0}%/h"),
        rate if rate >= 0.05 => format!("{rate:.1}%/h"),
        _ => "0%/h".to_owned(),
    };
    match (pace.runs_out_at, resets_at) {
        (Some(out), _) if out <= now => format!("at {rate}, runs out about now"),
        (Some(out), Some(reset)) => format!(
            "at {rate}, runs out in {}, {} before it resets",
            duration(out - now),
            duration(reset - out)
        ),
        (Some(out), None) => format!("at {rate}, runs out in {}", duration(out - now)),
        (None, Some(_)) => format!("at {rate}, lasts until it resets"),
        (None, None) => format!("at {rate}"),
    }
}

/// The name people know a provider by, such as `Claude` for `claude`.
#[must_use]
pub fn provider_name(id: &str) -> &str {
    match id {
        "claude" => "Claude",
        "codex" => "Codex",
        other => other,
    }
}

/// What QuotaDial does with a sign-in it reads from `place`, as a provider that reads one says it
/// (P-020): such as `QuotaDial reads this account's sign-in from ~/.codex/auth.json. It only reads
/// it and never changes it.`
#[must_use]
pub fn sign_in_read_from(place: &str) -> String {
    format!(
        "QuotaDial reads this account's sign-in from {place}. It only reads it and never changes it."
    )
}

/// A share as people write it: `80` for 80.0 and `80.5` for 80.5, to a tenth.
#[must_use]
pub fn percent(pct: f64) -> String {
    format!("{}", (pct * 10.0).round() / 10.0)
}

/// A whole number with commas between the thousands, such as `1,204`.
#[must_use]
pub fn thousands(count: u64) -> String {
    let digits = count.to_string();
    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    grouped
}

/// A count of something, such as `1 request` or `1,204 requests`.
#[must_use]
pub fn plural(count: u64, noun: &str) -> String {
    if count == 1 {
        format!("1 {noun}")
    } else {
        format!("{} {noun}s", thousands(count))
    }
}

/// An amount in US dollars as people write it: `$1,234` from $100 up, `$12.34` below that,
/// `under $0.01` for less than a cent, and `$0` for nothing.
#[must_use]
pub fn usd(amount: f64) -> String {
    if amount.is_nan() || amount <= 0.0 {
        "$0".to_owned()
    } else if amount < 0.01 {
        "under $0.01".to_owned()
    } else if amount < 100.0 {
        format!("${amount:.2}")
    } else {
        format!("${}", thousands(amount.round() as u64))
    }
}

/// A provider's cost, such as `$1,234 at API prices over the last 7 days, from 51,234 requests`,
/// or `no requests in the last 7 days`.
#[must_use]
pub fn cost(cost: &Cost) -> String {
    let days = (cost.until - cost.since).num_days();
    if cost.requests == 0 {
        return format!("no requests in the last {days} days");
    }
    format!(
        "{} at API prices over the last {days} days, from {}",
        usd(cost.usd),
        plural(cost.requests, "request")
    )
}

/// A model's part of a cost, such as `claude-opus-5-5: $1,000, 41,000 requests`, or `no price`
/// in place of the amount when the price table has none.
#[must_use]
pub fn model_cost(model: &ModelCost) -> String {
    format!(
        "{}: {}, {}",
        model.model,
        model.usd.map_or_else(|| "no price".to_owned(), usd),
        plural(model.requests, "request")
    )
}

/// For example `4 min ago`, or `just now`.
#[must_use]
pub fn ago(at: DateTime<Utc>, now: DateTime<Utc>) -> String {
    if now - at < TimeDelta::minutes(1) {
        return "just now".to_owned();
    }
    format!("{} ago", duration(now - at))
}

/// For example `under a minute`, `45 min`, `2 h 10 min` or `3 d 4 h`, rounded down.
#[must_use]
pub fn duration(length: TimeDelta) -> String {
    let minutes = length.num_minutes();
    let (days, hours) = (minutes / (24 * 60), minutes / 60 % 24);
    match (days, hours, minutes % 60) {
        _ if minutes < 1 => "under a minute".to_owned(),
        (0, 0, minutes) => format!("{minutes} min"),
        (0, hours, 0) => format!("{hours} h"),
        (0, hours, minutes) => format!("{hours} h {minutes} min"),
        (days, 0, _) => format!("{days} d"),
        (days, hours, _) => format!("{days} d {hours} h"),
    }
}

/// For example `on, 12.50 of 50.00 USD used (25%)`, `on, 1056.44 credits left`, or `off`.
#[must_use]
pub fn extra_usage(extra: &ExtraUsage) -> String {
    if !extra.enabled {
        return "off".to_owned();
    }
    let mut text = "on".to_owned();
    if let (Some(used), Some(limit)) = (extra.used, extra.limit) {
        let currency = extra.currency.as_deref().unwrap_or("USD");
        text.push_str(&format!(", {used:.2} of {limit:.2} {currency} used"));
    }
    if let Some(used_pct) = extra.used_pct {
        text.push_str(&format!(" ({used_pct:.0}%)"));
    }
    if let Some(balance) = extra.balance {
        text.push_str(&format!(", {balance:.2} credits left"));
    }
    text
}

/// For example `ok`, `stale` or `sign-in required`.
#[must_use]
pub fn status_label(status: Status) -> &'static str {
    match status {
        Status::Ok => "ok",
        Status::Stale => "stale",
        Status::AuthRequired => "sign-in required",
        Status::RateLimited => "rate limited",
        Status::Unsupported => "unsupported",
    }
}

/// Where a number came from: `the API`, `local logs`, `cache`, or `the example build` for a number
/// made up for pictures of the app (D-117).
#[must_use]
pub fn source_label(source: Source) -> &'static str {
    match source {
        Source::Api => "the API",
        Source::LocalLog => "local logs",
        Source::Cache => "cache",
        Source::Example => "the example build",
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn paths_in_the_home_folder_start_with_a_tilde() {
        let home = PathBuf::from("/Users/example");
        assert_eq!(
            tilde(&home.join(".codex/auth.json"), Some(&home)),
            "~/.codex/auth.json"
        );
        assert_eq!(tilde(&home, Some(&home)), "~");
        assert_eq!(tilde(Path::new("/etc/x"), Some(&home)), "/etc/x");
        assert_eq!(tilde(Path::new("/etc/x"), None), "/etc/x");
    }

    fn ts(rfc3339: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(rfc3339)
            .expect("valid timestamp")
            .with_timezone(&Utc)
    }

    /// Four minutes after the times the tests start from.
    fn now() -> DateTime<Utc> {
        ts("2099-01-01T00:04:00Z")
    }

    #[test]
    fn durations_read_naturally_and_round_down() {
        assert_eq!(duration(TimeDelta::seconds(59)), "under a minute");
        assert_eq!(duration(TimeDelta::minutes(45)), "45 min");
        assert_eq!(duration(TimeDelta::hours(2)), "2 h");
        assert_eq!(
            duration(TimeDelta::minutes(130) + TimeDelta::seconds(59)),
            "2 h 10 min"
        );
        assert_eq!(duration(TimeDelta::days(3)), "3 d");
        assert_eq!(
            duration(TimeDelta::days(3) + TimeDelta::hours(4) + TimeDelta::minutes(5)),
            "3 d 4 h"
        );
    }

    #[test]
    fn reset_times_and_ages_are_relative_to_now() {
        assert_eq!(
            resets(Some(ts("2099-01-01T05:00:00Z")), now()),
            "resets in 4 h 56 min"
        );
        assert_eq!(
            resets(Some(ts("2099-01-01T00:01:00Z")), now()),
            "reset 3 min ago",
            "numbers from before a reset say so"
        );
        assert_eq!(resets(None, now()), "no reset time reported");
        assert_eq!(ago(ts("2099-01-01T00:00:00Z"), now()), "4 min ago");
        assert_eq!(ago(ts("2099-01-01T00:03:30Z"), now()), "just now");
        assert_eq!(
            ago(ts("2099-01-01T00:09:00Z"), now()),
            "just now",
            "a clock that moved back"
        );
    }

    #[test]
    fn a_pace_says_whether_the_window_runs_out_before_it_resets() {
        let resets_at = Some(ts("2099-01-01T02:14:00Z"));
        let pace = |pct_per_hour, runs_out_at: Option<&str>| Pace {
            pct_per_hour,
            runs_out_at: runs_out_at.map(ts),
        };

        assert_eq!(
            super::pace(&pace(8.46, Some("2099-01-01T01:34:00Z")), resets_at, now()),
            "at 8.5%/h, runs out in 1 h 30 min, 40 min before it resets"
        );
        assert_eq!(
            super::pace(&pace(25.0, None), resets_at, now()),
            "at 25%/h, lasts until it resets"
        );
        assert_eq!(
            super::pace(&pace(0.01, None), resets_at, now()),
            "at 0%/h, lasts until it resets"
        );
        assert_eq!(
            super::pace(&pace(90.0, Some("2099-01-01T00:03:00Z")), resets_at, now()),
            "at 90%/h, runs out about now",
            "a run-out the clock has passed since the reading"
        );
        assert_eq!(
            super::pace(&pace(2.0, Some("2099-01-01T19:04:00Z")), None, now()),
            "at 2.0%/h, runs out in 19 h"
        );
        assert_eq!(super::pace(&pace(0.0, None), None, now()), "at 0%/h");
    }

    #[test]
    fn every_source_is_named_as_the_sentence_after_from_reads() {
        assert_eq!(source_label(Source::Api), "the API");
        assert_eq!(source_label(Source::LocalLog), "local logs");
        assert_eq!(source_label(Source::Cache), "cache");
        assert_eq!(
            source_label(Source::Example),
            "the example build",
            "so an example is never taken for a real account's numbers"
        );
    }

    #[test]
    fn a_sign_in_read_from_a_place_says_so_and_that_it_is_never_changed() {
        assert_eq!(
            sign_in_read_from("~/.codex/auth.json"),
            "QuotaDial reads this account's sign-in from ~/.codex/auth.json. It only reads it and \
             never changes it."
        );
    }

    #[test]
    fn providers_and_shares_are_named_as_people_write_them() {
        assert_eq!(provider_name("claude"), "Claude");
        assert_eq!(provider_name("codex"), "Codex");
        assert_eq!(provider_name("other"), "other");
        assert_eq!(percent(80.0), "80");
        assert_eq!(percent(80.5), "80.5");
        assert_eq!(percent(33.333), "33.3");
    }

    #[test]
    fn extra_usage_that_is_off_says_so() {
        let extra = ExtraUsage {
            enabled: false,
            used_pct: None,
            used: None,
            limit: None,
            currency: None,
            balance: None,
        };

        assert_eq!(extra_usage(&extra), "off");
    }

    #[test]
    fn a_credit_balance_is_shown() {
        let extra = ExtraUsage {
            enabled: true,
            used_pct: None,
            used: None,
            limit: None,
            currency: None,
            balance: Some(1_056.436_375),
        };

        assert_eq!(extra_usage(&extra), "on, 1056.44 credits left");
    }

    #[test]
    fn counts_and_money_read_as_people_write_them() {
        assert_eq!(thousands(0), "0");
        assert_eq!(thousands(999), "999");
        assert_eq!(thousands(1_204), "1,204");
        assert_eq!(thousands(12_345_678), "12,345,678");
        assert_eq!(plural(1, "request"), "1 request");
        assert_eq!(plural(0, "request"), "0 requests");
        assert_eq!(plural(51_234, "request"), "51,234 requests");

        assert_eq!(usd(0.0), "$0");
        assert_eq!(usd(f64::NAN), "$0");
        assert_eq!(usd(0.004), "under $0.01");
        assert_eq!(usd(12.345), "$12.35");
        assert_eq!(usd(99.99), "$99.99");
        assert_eq!(usd(1_234.5), "$1,235");
    }

    #[test]
    fn a_cost_says_what_the_week_would_have_cost() {
        let until = DateTime::parse_from_rfc3339("2026-10-24T00:00:00Z")
            .expect("valid")
            .with_timezone(&Utc);
        let mut cost = Cost {
            provider: "claude".to_owned(),
            since: until - TimeDelta::days(7),
            until,
            usd: 1_234.4,
            requests: 51_234,
            tokens: crate::Tokens::default(),
            models: Vec::new(),
            prices_updated: "2026-10-23".to_owned(),
            notes: Vec::new(),
        };

        assert_eq!(
            super::cost(&cost),
            "$1,234 at API prices over the last 7 days, from 51,234 requests"
        );
        cost.requests = 0;
        cost.usd = 0.0;
        assert_eq!(super::cost(&cost), "no requests in the last 7 days");

        let mut model = ModelCost {
            model: "claude-opus-5-5".to_owned(),
            usd: Some(1_000.25),
            requests: 41_000,
            tokens: crate::Tokens::default(),
        };
        assert_eq!(
            model_cost(&model),
            "claude-opus-5-5: $1,000, 41,000 requests"
        );
        model.usd = None;
        model.requests = 1;
        assert_eq!(model_cost(&model), "claude-opus-5-5: no price, 1 request");
    }
}
