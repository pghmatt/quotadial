//! When QuotaDial may ask Anthropic for an account's usage, and what to show when it may not.
//!
//! Anthropic answers its usage endpoint with 429 aggressively, so:
//!
//! - At most one request per account every 5 minutes, whatever the last answer was. In between,
//!   the last answer is shown again with its numbers marked as coming from the cache.
//! - A 429 blocks requests until `Retry-After` says, or else for 5, 10, 20, 40 and then 60 minutes
//!   as 429s repeat. A block is never shortened, and the numbers from the last good fetch stay on
//!   screen, marked as cache, while it lasts.
//! - The one exception: after Anthropic rejected a sign-in, a new sign-in (Claude Code wrote a new
//!   token) is tried at once instead of after the floor.
//!
//! Replaces CodexBar's `Providers/Claude/ClaudeOAuth/ClaudeOAuthUsageRateLimitGate.swift`, which
//! honours `Retry-After` or a 5-minute cooldown but lets user-initiated refreshes through.
//! QuotaDial lets nothing through.

use chrono::{DateTime, SecondsFormat, TimeDelta, Utc};
use quotadial_core::{Account, Snapshot, Status};

use crate::state::{AccountState, LastResult};

/// The longest `Retry-After` honoured, so a garbled header cannot stop requests for good.
const MAX_RETRY_AFTER_HOURS: i64 = 24;
/// How far in the future a saved request time may be before the clock is assumed to have moved.
const CLOCK_SKEW_MINUTES: i64 = 1;

/// What to do about one fetch.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Decision {
    /// Send Anthropic a request.
    Request,
    /// Show this snapshot without a request.
    Replay(Box<Snapshot>),
    /// No request and nothing to show: the last request failed and nothing is cached.
    Wait(String),
}

/// The 5-minute floor, as [`crate::MIN_POLL_INTERVAL`].
pub(crate) fn floor() -> TimeDelta {
    TimeDelta::minutes(5)
}

/// Decides whether a fetch for `account` may send a request at `now`. `generation` is the
/// current sign-in's generation (see `Login::generation`).
pub(crate) fn decide(
    state: &AccountState,
    account: &Account,
    generation: Option<&str>,
    now: DateTime<Utc>,
) -> Decision {
    let Some(requested_at) = state.requested_at else {
        return Decision::Request;
    };
    if requested_at > now + TimeDelta::minutes(CLOCK_SKEW_MINUTES) {
        // The clock moved back, so the saved times say nothing about now.
        return Decision::Request;
    }
    if let Some(until) = state.blocked_until.filter(|until| *until > now) {
        return Decision::Replay(Box::new(rate_limited(state, account, until)));
    }
    let next_request = requested_at + floor();
    if now >= next_request || signed_in_again(state, generation) {
        return Decision::Request;
    }
    match &state.last_result {
        Some(LastResult::Snapshot { snapshot }) => {
            Decision::Replay(Box::new((**snapshot).clone().served_from_cache()))
        }
        Some(LastResult::Failed { message }) => match stale(state, message, next_request) {
            Some(snapshot) => Decision::Replay(Box::new(snapshot)),
            None => Decision::Wait(format!(
                "{message} QuotaDial asks Anthropic at most every 5 minutes, so the next request \
                 is after {}.",
                timestamp(next_request)
            )),
        },
        None => Decision::Request,
    }
}

/// The last answer rejected the sign-in, and Claude Code has written a new token since.
fn signed_in_again(state: &AccountState, generation: Option<&str>) -> bool {
    let rejected = matches!(
        &state.last_result,
        Some(LastResult::Snapshot { snapshot }) if snapshot.status == Status::AuthRequired
    );
    rejected && generation.is_some() && generation != state.login_generation.as_deref()
}

impl AccountState {
    /// Marks a request as sent, before it is. Saving this first means the floor holds even if
    /// QuotaDial stops before the answer arrives.
    pub(crate) fn start_request(&mut self, now: DateTime<Utc>, generation: Option<String>) {
        self.requested_at = Some(now);
        self.login_generation = generation;
    }

    /// Records usage numbers, or an answer that the plan reports none.
    pub(crate) fn record_usage(&mut self, snapshot: &Snapshot) {
        self.rate_limit_strikes = 0;
        self.blocked_until = None;
        self.last_good = (snapshot.status == Status::Ok).then(|| snapshot.clone());
        self.last_result = Some(LastResult::Snapshot {
            snapshot: Box::new(snapshot.clone()),
        });
    }

    /// Records that Anthropic rejected the sign-in.
    pub(crate) fn record_auth_failure(&mut self, snapshot: &Snapshot) {
        self.rate_limit_strikes = 0;
        self.blocked_until = None;
        self.last_result = Some(LastResult::Snapshot {
            snapshot: Box::new(snapshot.clone()),
        });
    }

    /// Records a 429 and returns the snapshot to show.
    pub(crate) fn record_rate_limit(
        &mut self,
        account: &Account,
        retry_after: Option<DateTime<Utc>>,
        now: DateTime<Utc>,
    ) -> Snapshot {
        self.rate_limit_strikes = self.rate_limit_strikes.saturating_add(1);
        let until = retry_after
            .filter(|until| *until > now)
            .map_or_else(
                || now + backoff(self.rate_limit_strikes),
                |until| until.min(now + TimeDelta::hours(MAX_RETRY_AFTER_HOURS)),
            )
            .max(now + floor());
        let until = self
            .blocked_until
            .map_or(until, |existing| existing.max(until));
        self.blocked_until = Some(until);
        let snapshot = rate_limited(self, account, until);
        self.last_result = Some(LastResult::Snapshot {
            snapshot: Box::new(snapshot.clone()),
        });
        snapshot
    }

    /// Records an answer QuotaDial could not use, such as an HTTP 503. Returns the last numbers,
    /// marked stale, or the message when there are none.
    pub(crate) fn record_failure(
        &mut self,
        message: String,
        now: DateTime<Utc>,
    ) -> Result<Snapshot, String> {
        let next_request = now + floor();
        let shown = stale(self, &message, next_request).ok_or_else(|| message.clone());
        self.last_result = Some(LastResult::Failed { message });
        shown
    }
}

/// What to show when no answer arrived at all, which does not count against the floor: the last
/// numbers, marked stale, or the message when there are none.
pub(crate) fn unanswered(state: &AccountState, message: &str) -> Result<Snapshot, String> {
    let mut snapshot = state
        .last_good
        .clone()
        .ok_or_else(|| message.to_owned())?
        .served_from_cache();
    snapshot.status = Status::Stale;
    snapshot.error = Some(format!("{message} Showing the last numbers fetched."));
    Ok(snapshot)
}

/// 5 minutes, doubling with each 429 in a row, up to an hour.
fn backoff(strikes: u32) -> TimeDelta {
    let doublings = strikes.clamp(1, 5) - 1;
    TimeDelta::minutes(5 << doublings).min(TimeDelta::hours(1))
}

/// Parses `Retry-After`: a number of seconds, or an HTTP date.
pub(crate) fn retry_after(value: &str, now: DateTime<Utc>) -> Option<DateTime<Utc>> {
    let value = value.trim();
    if let Ok(seconds) = value.parse::<u32>() {
        return Some(now + TimeDelta::seconds(i64::from(seconds)));
    }
    DateTime::parse_from_rfc2822(value)
        .ok()
        .map(|at| at.with_timezone(&Utc))
}

fn rate_limited(state: &AccountState, account: &Account, until: DateTime<Utc>) -> Snapshot {
    let mut snapshot = state.last_good.clone().map_or_else(
        || Snapshot {
            account: account.clone(),
            windows: Vec::new(),
            extra_usage: None,
            status: Status::Ok,
            error: None,
        },
        Snapshot::served_from_cache,
    );
    snapshot.status = Status::RateLimited;
    snapshot.error = Some(format!(
        "Anthropic is rate-limiting usage requests, so QuotaDial will ask again after {}.",
        timestamp(until)
    ));
    snapshot
}

fn stale(state: &AccountState, message: &str, next_request: DateTime<Utc>) -> Option<Snapshot> {
    let mut snapshot = state.last_good.clone()?.served_from_cache();
    snapshot.status = Status::Stale;
    snapshot.error = Some(format!(
        "{message} Showing the last numbers fetched; QuotaDial asks again after {}.",
        timestamp(next_request)
    ));
    Some(snapshot)
}

fn timestamp(at: DateTime<Utc>) -> String {
    at.to_rfc3339_opts(SecondsFormat::Secs, true)
}

#[cfg(test)]
mod tests {
    use quotadial_core::{Source, Window, WindowKind};

    use super::*;

    fn ts(rfc3339: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(rfc3339)
            .expect("valid timestamp")
            .with_timezone(&Utc)
    }

    fn t0() -> DateTime<Utc> {
        ts("2099-01-01T00:00:00Z")
    }

    fn minutes(n: i64) -> TimeDelta {
        TimeDelta::minutes(n)
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

    fn good_snapshot() -> Snapshot {
        Snapshot {
            account: account(),
            windows: vec![Window {
                kind: WindowKind::Session,
                label: "Session".into(),
                used_pct: 42.0,
                resets_at: Some(ts("2099-01-01T05:00:00Z")),
                window_seconds: 18_000,
                scope_model: None,
                source: Source::Api,
                fetched_at: t0(),
            }],
            extra_usage: None,
            status: Status::Ok,
            error: None,
        }
    }

    /// The state after a successful request at `t0`.
    fn after_success() -> AccountState {
        let mut state = AccountState::default();
        state.start_request(t0(), Some("gen-1".into()));
        state.record_usage(&good_snapshot());
        state
    }

    #[test]
    fn floor_matches_the_public_constant() {
        assert_eq!(
            floor().to_std().expect("positive"),
            crate::MIN_POLL_INTERVAL
        );
    }

    #[test]
    fn a_fresh_account_may_request() {
        assert_eq!(
            decide(&AccountState::default(), &account(), None, t0()),
            Decision::Request
        );
    }

    #[test]
    fn within_five_minutes_the_last_numbers_are_replayed_from_the_cache() {
        let state = after_success();

        let Decision::Replay(snapshot) =
            decide(&state, &account(), Some("gen-1"), t0() + minutes(4))
        else {
            panic!("expected a replay");
        };
        assert_eq!(snapshot.status, Status::Ok);
        assert_eq!(snapshot.windows[0].source, Source::Cache);
        assert_eq!(snapshot.windows[0].fetched_at, t0());
        assert_eq!(snapshot.windows[0].used_pct, 42.0);
    }

    #[test]
    fn after_five_minutes_a_request_is_allowed() {
        let state = after_success();

        assert_eq!(
            decide(&state, &account(), Some("gen-1"), t0() + minutes(5)),
            Decision::Request
        );
    }

    #[test]
    fn a_new_token_does_not_bypass_the_floor_after_a_success() {
        let state = after_success();

        assert!(matches!(
            decide(&state, &account(), Some("gen-2"), t0() + minutes(1)),
            Decision::Replay(_)
        ));
    }

    #[test]
    fn a_new_sign_in_after_a_rejection_is_tried_at_once() {
        let mut state = AccountState::default();
        state.start_request(t0(), Some("gen-1".into()));
        state.record_auth_failure(&Snapshot::auth_required(account(), "Run `claude`."));

        let same_login = decide(&state, &account(), Some("gen-1"), t0() + minutes(1));
        let Decision::Replay(snapshot) = same_login else {
            panic!("expected a replay");
        };
        assert_eq!(snapshot.status, Status::AuthRequired);

        assert_eq!(
            decide(&state, &account(), Some("gen-2"), t0() + minutes(1)),
            Decision::Request
        );
        assert!(matches!(
            decide(&state, &account(), None, t0() + minutes(1)),
            Decision::Replay(_)
        ));
    }

    #[test]
    fn a_429_without_retry_after_backs_off_exponentially_to_an_hour() {
        let mut state = after_success();
        let mut blocks = Vec::new();
        let mut now = t0();
        for _ in 0..7 {
            now = state.blocked_until.unwrap_or(now) + minutes(1);
            state.start_request(now, Some("gen-1".into()));
            state.record_rate_limit(&account(), None, now);
            blocks.push((state.blocked_until.expect("blocked") - now).num_minutes());
        }

        assert_eq!(blocks, [5, 10, 20, 40, 60, 60, 60]);
    }

    #[test]
    fn a_429_honours_retry_after_but_never_less_than_the_floor() {
        let mut long = after_success();
        long.record_rate_limit(&account(), Some(t0() + minutes(30)), t0());
        assert_eq!(long.blocked_until, Some(t0() + minutes(30)));

        let mut short = after_success();
        short.record_rate_limit(&account(), Some(t0() + TimeDelta::seconds(30)), t0());
        assert_eq!(short.blocked_until, Some(t0() + minutes(5)));

        let mut huge = after_success();
        huge.record_rate_limit(&account(), Some(t0() + TimeDelta::days(30)), t0());
        assert_eq!(huge.blocked_until, Some(t0() + TimeDelta::hours(24)));
    }

    #[test]
    fn a_later_429_never_shortens_a_block() {
        let mut state = after_success();
        state.record_rate_limit(&account(), Some(t0() + minutes(50)), t0());
        state.record_rate_limit(&account(), Some(t0() + minutes(10)), t0() + minutes(1));

        assert_eq!(state.blocked_until, Some(t0() + minutes(50)));
    }

    #[test]
    fn while_blocked_the_last_numbers_show_as_rate_limited_cache() {
        let mut state = after_success();
        let shown = state.record_rate_limit(&account(), None, t0() + minutes(6));

        assert_eq!(shown.status, Status::RateLimited);
        assert_eq!(shown.windows[0].source, Source::Cache);
        assert_eq!(shown.windows[0].used_pct, 42.0);
        assert_eq!(
            shown.error.as_deref(),
            Some(
                "Anthropic is rate-limiting usage requests, so QuotaDial will ask again after \
                 2099-01-01T00:11:00Z."
            )
        );

        // Even a new sign-in waits out the block.
        let Decision::Replay(replayed) =
            decide(&state, &account(), Some("gen-2"), t0() + minutes(9))
        else {
            panic!("expected a replay");
        };
        assert_eq!(replayed.status, Status::RateLimited);
        assert_eq!(replayed.windows[0].source, Source::Cache);
    }

    #[test]
    fn a_429_with_nothing_cached_shows_the_account_without_numbers() {
        let mut state = AccountState::default();
        let shown = state.record_rate_limit(&account(), None, t0());

        assert_eq!(shown.status, Status::RateLimited);
        assert!(shown.windows.is_empty());
        assert_eq!(shown.account, account());
    }

    #[test]
    fn a_failed_answer_shows_the_last_numbers_as_stale() {
        let mut state = after_success();
        state.start_request(t0() + minutes(10), Some("gen-1".into()));
        let shown = state
            .record_failure("Anthropic answered HTTP 503.".into(), t0() + minutes(10))
            .expect("stale snapshot");

        assert_eq!(shown.status, Status::Stale);
        assert_eq!(shown.windows[0].source, Source::Cache);
        assert_eq!(
            shown.error.as_deref(),
            Some(
                "Anthropic answered HTTP 503. Showing the last numbers fetched; QuotaDial asks \
                 again after 2099-01-01T00:15:00Z."
            )
        );
        // The floor still holds after a failure.
        assert!(matches!(
            decide(&state, &account(), Some("gen-1"), t0() + minutes(12)),
            Decision::Replay(snapshot) if snapshot.status == Status::Stale
        ));
    }

    #[test]
    fn a_failed_answer_with_nothing_cached_waits_with_the_reason() {
        let mut state = AccountState::default();
        state.start_request(t0(), None);
        let shown = state.record_failure("Anthropic answered HTTP 503.".into(), t0());

        assert_eq!(shown, Err("Anthropic answered HTTP 503.".to_owned()));
        assert_eq!(
            decide(&state, &account(), None, t0() + minutes(2)),
            Decision::Wait(
                "Anthropic answered HTTP 503. QuotaDial asks Anthropic at most every 5 minutes, \
                 so the next request is after 2099-01-01T00:05:00Z."
                    .into()
            )
        );
    }

    #[test]
    fn an_unanswered_request_shows_stale_numbers_or_the_reason() {
        let state = after_success();
        let shown = unanswered(&state, "Could not reach Anthropic: connection refused.")
            .expect("stale snapshot");
        assert_eq!(shown.status, Status::Stale);
        assert_eq!(
            shown.error.as_deref(),
            Some(
                "Could not reach Anthropic: connection refused. Showing the last numbers fetched."
            )
        );

        assert_eq!(
            unanswered(&AccountState::default(), "Could not reach Anthropic."),
            Err("Could not reach Anthropic.".to_owned())
        );
    }

    #[test]
    fn an_unsupported_answer_clears_the_cached_numbers() {
        let mut state = after_success();
        state.record_usage(&Snapshot::unsupported(account(), "No limits."));

        assert_eq!(state.last_good, None);
    }

    #[test]
    fn a_clock_that_moved_back_does_not_block_requests() {
        let mut state = after_success();
        state.blocked_until = Some(t0() + minutes(60));

        assert_eq!(
            decide(
                &state,
                &account(),
                Some("gen-1"),
                t0() - TimeDelta::hours(3)
            ),
            Decision::Request
        );
    }

    #[test]
    fn retry_after_takes_seconds_or_an_http_date() {
        assert_eq!(retry_after("120", t0()), Some(t0() + minutes(2)));
        assert_eq!(
            retry_after("Thu, 01 Jan 2099 00:30:00 GMT", t0()),
            Some(t0() + minutes(30))
        );
        assert_eq!(retry_after("soon", t0()), None);
        assert_eq!(retry_after("-5", t0()), None);
    }
}
