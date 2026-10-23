//! The words QuotaDial uses for times, sources and statuses. The CLI, the tray app and its
//! notifications all say them the same way.

use std::path::Path;

use chrono::{DateTime, TimeDelta, Utc};

use crate::{ExtraUsage, Source, Status};

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

/// Where a number came from: `the API`, `local logs` or `cache`.
#[must_use]
pub fn source_label(source: Source) -> &'static str {
    match source {
        Source::Api => "the API",
        Source::LocalLog => "local logs",
        Source::Cache => "cache",
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
}
