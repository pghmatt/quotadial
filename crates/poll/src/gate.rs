//! When QuotaDial may ask a provider for an account's usage, and what to show when it may not.
//!
//! Usage endpoints answer with 429 when asked too often (Anthropic's aggressively), so:
//!
//! - At most one request per account per floor ([`Policy::floor`]: 5 minutes for Anthropic),
//!   whatever the last answer was. In between, the last answer is shown again with its numbers
//!   marked as coming from the cache.
//! - A 429 blocks requests until `Retry-After` says, or else for 5, 10, 20, 40 and then 60 minutes
//!   as 429s repeat, and never for less than the floor. A block is never shortened, and the
//!   numbers from the last good fetch stay on screen, marked as cache, while it lasts.
//! - The one exception: after the provider rejected a sign-in, a new sign-in (the official CLI
//!   wrote a new token) is tried at once instead of after the floor, once per floor.
//! - A request whose answer was never recorded, because QuotaDial was stopped mid-request, still
//!   counts.
//! - If the clock moves back, the last request counts as just sent, and a block keeps its length.
//!
//! Replaces CodexBar's `Providers/Claude/ClaudeOAuth/ClaudeOAuthUsageRateLimitGate.swift`, which
//! honours `Retry-After` or a 5-minute cooldown but lets user-initiated refreshes through.
//! QuotaDial lets nothing through.

use chrono::{DateTime, SecondsFormat, TimeDelta, Utc};
use quotadial_core::{Account, Snapshot, Status};

use crate::Policy;
use crate::state::{AccountState, LastResult};

/// The longest `Retry-After` honoured, so a garbled header cannot stop requests for good.
const MAX_RETRY_AFTER_HOURS: i64 = 24;

/// What to do about one fetch.
#[derive(Debug, Clone, PartialEq)]
pub enum Decision {
    /// Send the provider a request.
    Request,
    /// Show this snapshot without a request.
    Replay(Box<Snapshot>),
    /// No request and nothing to show: the last request failed and nothing is cached.
    Wait(String),
}

/// Decides whether a fetch for `account` may send a request at `now`. `generation` identifies the
/// current sign-in, such as the modification time of the credential it came from. Call
/// [`AccountState::reanchor`] first.
pub fn decide(
    policy: &Policy,
    state: &AccountState,
    account: &Account,
    generation: Option<&str>,
    now: DateTime<Utc>,
) -> Decision {
    let Some(requested_at) = state.requested_at else {
        return Decision::Request;
    };
    if let Some(until) = state.blocked_until.filter(|until| *until > now) {
        return Decision::Replay(Box::new(rate_limited(policy, state, account, until)));
    }
    let next_request = requested_at + policy.floor;
    if now >= next_request || signed_in_again(state, generation) {
        return Decision::Request;
    }
    match &state.last_result {
        Some(LastResult::Snapshot { snapshot }) => {
            Decision::Replay(Box::new((**snapshot).clone().served_from_cache()))
        }
        Some(LastResult::Failed { message }) => match stale(state, message, next_request) {
            Some(snapshot) => Decision::Replay(Box::new(snapshot)),
            None => Decision::Wait(format!("{message} {}", policy.next_request(next_request))),
        },
        // QuotaDial was stopped before the first answer for this account was recorded.
        None => Decision::Wait(format!(
            "The request sent at {} has no recorded answer, perhaps because QuotaDial was \
             stopped. {}",
            timestamp(requested_at),
            policy.next_request(next_request)
        )),
    }
}

/// The last answer rejected the sign-in, the official CLI has written a new token since, and the
/// one early retry this allows per floor has not been used.
fn signed_in_again(state: &AccountState, generation: Option<&str>) -> bool {
    let rejected = matches!(
        &state.last_result,
        Some(LastResult::Snapshot { snapshot }) if snapshot.status == Status::AuthRequired
    );
    rejected
        && !state.bypassed_floor
        && generation.is_some()
        && generation != state.login_generation.as_deref()
}

impl AccountState {
    /// Fixes the saved times after the clock moved back: the last request counts as sent `now`,
    /// and a block keeps the length it had from that request, at most 24 hours. Returns whether
    /// anything changed, so the caller can save it. Without this, a clock set back a day would
    /// stop requests for a day.
    pub fn reanchor(&mut self, now: DateTime<Utc>) -> bool {
        let Some(requested_at) = self.requested_at.filter(|at| *at > now) else {
            return false;
        };
        let shift = requested_at - now;
        self.requested_at = Some(now);
        self.blocked_until = self
            .blocked_until
            .map(|until| (until - shift).min(now + TimeDelta::hours(MAX_RETRY_AFTER_HOURS)));
        true
    }

    /// For a sign-in QuotaDial cannot name without asking the provider: once the official CLI has
    /// written a new token, the saved numbers may be another account's, so they are not shown
    /// again. The floor and any block still hold. A rejected sign-in is left alone, so the one
    /// early retry still applies. Returns whether anything changed.
    pub fn forget_numbers_after_new_sign_in(
        &mut self,
        policy: &Policy,
        generation: Option<&str>,
    ) -> bool {
        let rejected = matches!(
            &self.last_result,
            Some(LastResult::Snapshot { snapshot }) if snapshot.status == Status::AuthRequired
        );
        let new_token = generation.is_some()
            && self.login_generation.is_some()
            && generation != self.login_generation.as_deref();
        if rejected || !new_token {
            return false;
        }
        self.last_good = None;
        self.last_result = Some(LastResult::Failed {
            message: format!(
                "{}'s sign-in changed since QuotaDial last asked {}.",
                policy.cli, policy.service
            ),
        });
        self.login_generation = generation.map(str::to_owned);
        true
    }

    /// Marks a request as sent, before it is. Saving this first means the floor holds even if
    /// QuotaDial stops before the answer arrives.
    pub fn start_request(
        &mut self,
        policy: &Policy,
        now: DateTime<Utc>,
        generation: Option<String>,
    ) {
        self.bypassed_floor = self
            .requested_at
            .is_some_and(|requested_at| now < requested_at + policy.floor);
        self.requested_at = Some(now);
        self.login_generation = generation;
    }

    /// Records usage numbers, or an answer that the plan reports none, and returns the snapshot to
    /// show. A window that reads 0% where it read more on the last fetch, before its reset, is
    /// marked unconfirmed until a second fetch agrees (the PRD's unconfirmed-0% rule).
    pub fn record_usage(&mut self, mut snapshot: Snapshot) -> Snapshot {
        if let Some(previous) = &self.last_good {
            mark_unconfirmed_zeros(previous, &mut snapshot);
        }
        self.rate_limit_strikes = 0;
        self.blocked_until = None;
        self.last_good = (snapshot.status == Status::Ok).then(|| snapshot.clone());
        self.last_good_generation = self.last_good.as_ref().and(self.login_generation.clone());
        self.last_result = Some(LastResult::Snapshot {
            snapshot: Box::new(snapshot.clone()),
        });
        snapshot
    }

    /// Records that the provider rejected the sign-in.
    pub fn record_auth_failure(&mut self, snapshot: &Snapshot) {
        self.rate_limit_strikes = 0;
        self.blocked_until = None;
        self.last_result = Some(LastResult::Snapshot {
            snapshot: Box::new(snapshot.clone()),
        });
    }

    /// Records a 429 and returns the snapshot to show.
    pub fn record_rate_limit(
        &mut self,
        policy: &Policy,
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
            .max(now + policy.floor);
        let until = self
            .blocked_until
            .map_or(until, |existing| existing.max(until));
        self.blocked_until = Some(until);
        let snapshot = rate_limited(policy, self, account, until);
        self.last_result = Some(LastResult::Snapshot {
            snapshot: Box::new(snapshot.clone()),
        });
        snapshot
    }

    /// Records an answer QuotaDial could not use, such as an HTTP 503. Returns the last numbers,
    /// marked stale, or else the message with when the next request goes out.
    pub fn record_failure(
        &mut self,
        policy: &Policy,
        message: String,
        now: DateTime<Utc>,
    ) -> Result<Snapshot, String> {
        let next_request = now + policy.floor;
        let shown = stale(self, &message, next_request)
            .ok_or_else(|| format!("{message} {}", policy.next_request(next_request)));
        self.last_result = Some(LastResult::Failed { message });
        shown
    }
}

/// Marks each window of `snapshot` that reads 0% where the same window in `previous` read more and
/// had not reset by this fetch. A window that has reset, or whose reset is unknown, may well be
/// empty, and a 0% that follows an unconfirmed 0% agrees with it.
fn mark_unconfirmed_zeros(previous: &Snapshot, snapshot: &mut Snapshot) {
    for window in &mut snapshot.windows {
        window.unconfirmed = window.used_pct <= 0.0
            && previous.windows.iter().any(|before| {
                before.kind == window.kind
                    && before.label == window.label
                    && before.scope_model == window.scope_model
                    && before.used_pct > 0.0
                    && before
                        .resets_at
                        .is_some_and(|resets_at| resets_at > window.fetched_at)
            });
    }
}

/// What to show when no answer arrived at all, which does not count against the floor: the last
/// numbers, marked stale, or the message when there are none.
pub fn unanswered(state: &AccountState, message: &str) -> Result<Snapshot, String> {
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
pub fn retry_after(value: &str, now: DateTime<Utc>) -> Option<DateTime<Utc>> {
    let value = value.trim();
    if let Ok(seconds) = value.parse::<u32>() {
        return Some(now + TimeDelta::seconds(i64::from(seconds)));
    }
    DateTime::parse_from_rfc2822(value)
        .ok()
        .map(|at| at.with_timezone(&Utc))
}

fn rate_limited(
    policy: &Policy,
    state: &AccountState,
    account: &Account,
    until: DateTime<Utc>,
) -> Snapshot {
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
        "{} is rate-limiting usage requests, so QuotaDial will ask again after {}.",
        policy.service,
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

pub(crate) fn timestamp(at: DateTime<Utc>) -> String {
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

    const POLICY: Policy = Policy {
        service: "Anthropic",
        cli: "Claude Code",
        floor: TimeDelta::minutes(5),
    };

    fn account() -> Account {
        Account {
            provider: "claude".into(),
            account_id: "00000000-0000-0000-0000-000000000001".into(),
            display_name: "user@example.com".into(),
            plan: Some("max".into()),
            org: Some("00000000-0000-0000-0000-000000000002".into()),
            sign_in: None,
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
                unconfirmed: false,
                pace: None,
            }],
            extra_usage: None,
            status: Status::Ok,
            error: None,
        }
    }

    /// The state after a successful request at `t0`.
    fn after_success() -> AccountState {
        let mut state = AccountState::default();
        state.start_request(&POLICY, t0(), Some("gen-1".into()));
        state.record_usage(good_snapshot());
        state
    }

    #[test]
    fn the_floor_and_the_messages_follow_the_policy() {
        let codex = Policy {
            service: "OpenAI",
            cli: "The Codex CLI",
            floor: TimeDelta::minutes(2),
        };
        let mut state = AccountState::default();
        state.start_request(&codex, t0(), None);
        let shown = state.record_failure(&codex, "OpenAI answered HTTP 503.".into(), t0());

        assert_eq!(
            shown,
            Err(
                "OpenAI answered HTTP 503. QuotaDial asks OpenAI at most every 2 minutes, so the \
                 next request is after 2099-01-01T00:02:00Z."
                    .to_owned()
            )
        );
        assert_eq!(
            decide(&codex, &state, &account(), None, t0() + minutes(1)),
            Decision::Wait(
                "OpenAI answered HTTP 503. QuotaDial asks OpenAI at most every 2 minutes, so the \
                 next request is after 2099-01-01T00:02:00Z."
                    .into()
            )
        );
        assert_eq!(
            decide(&codex, &state, &account(), None, t0() + minutes(2)),
            Decision::Request
        );
        let blocked = state.record_rate_limit(&codex, &account(), None, t0() + minutes(2));
        assert_eq!(
            blocked.error.as_deref(),
            Some(
                "OpenAI is rate-limiting usage requests, so QuotaDial will ask again after \
                 2099-01-01T00:07:00Z."
            )
        );
    }

    #[test]
    fn a_fresh_account_may_request() {
        assert_eq!(
            decide(&POLICY, &AccountState::default(), &account(), None, t0()),
            Decision::Request
        );
    }

    #[test]
    fn within_five_minutes_the_last_numbers_are_replayed_from_the_cache() {
        let state = after_success();

        let Decision::Replay(snapshot) = decide(
            &POLICY,
            &state,
            &account(),
            Some("gen-1"),
            t0() + minutes(4),
        ) else {
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
            decide(
                &POLICY,
                &state,
                &account(),
                Some("gen-1"),
                t0() + minutes(5)
            ),
            Decision::Request
        );
    }

    #[test]
    fn a_new_token_does_not_bypass_the_floor_after_a_success() {
        let state = after_success();

        assert!(matches!(
            decide(
                &POLICY,
                &state,
                &account(),
                Some("gen-2"),
                t0() + minutes(1)
            ),
            Decision::Replay(_)
        ));
    }

    #[test]
    fn a_new_sign_in_after_a_rejection_is_tried_at_once() {
        let mut state = AccountState::default();
        state.start_request(&POLICY, t0(), Some("gen-1".into()));
        state.record_auth_failure(&Snapshot::auth_required(account(), "Run `claude`."));

        let same_login = decide(
            &POLICY,
            &state,
            &account(),
            Some("gen-1"),
            t0() + minutes(1),
        );
        let Decision::Replay(snapshot) = same_login else {
            panic!("expected a replay");
        };
        assert_eq!(snapshot.status, Status::AuthRequired);

        assert_eq!(
            decide(
                &POLICY,
                &state,
                &account(),
                Some("gen-2"),
                t0() + minutes(1)
            ),
            Decision::Request
        );
        assert!(matches!(
            decide(&POLICY, &state, &account(), None, t0() + minutes(1)),
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
            state.start_request(&POLICY, now, Some("gen-1".into()));
            state.record_rate_limit(&POLICY, &account(), None, now);
            blocks.push((state.blocked_until.expect("blocked") - now).num_minutes());
        }

        assert_eq!(blocks, [5, 10, 20, 40, 60, 60, 60]);
    }

    #[test]
    fn a_429_honours_retry_after_but_never_less_than_the_floor() {
        let mut long = after_success();
        long.record_rate_limit(&POLICY, &account(), Some(t0() + minutes(30)), t0());
        assert_eq!(long.blocked_until, Some(t0() + minutes(30)));

        let mut short = after_success();
        short.record_rate_limit(
            &POLICY,
            &account(),
            Some(t0() + TimeDelta::seconds(30)),
            t0(),
        );
        assert_eq!(short.blocked_until, Some(t0() + minutes(5)));

        let mut huge = after_success();
        huge.record_rate_limit(&POLICY, &account(), Some(t0() + TimeDelta::days(30)), t0());
        assert_eq!(huge.blocked_until, Some(t0() + TimeDelta::hours(24)));
    }

    #[test]
    fn a_later_429_never_shortens_a_block() {
        let mut state = after_success();
        state.record_rate_limit(&POLICY, &account(), Some(t0() + minutes(50)), t0());
        state.record_rate_limit(
            &POLICY,
            &account(),
            Some(t0() + minutes(10)),
            t0() + minutes(1),
        );

        assert_eq!(state.blocked_until, Some(t0() + minutes(50)));
    }

    #[test]
    fn while_blocked_the_last_numbers_show_as_rate_limited_cache() {
        let mut state = after_success();
        let shown = state.record_rate_limit(&POLICY, &account(), None, t0() + minutes(6));

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
        let Decision::Replay(replayed) = decide(
            &POLICY,
            &state,
            &account(),
            Some("gen-2"),
            t0() + minutes(9),
        ) else {
            panic!("expected a replay");
        };
        assert_eq!(replayed.status, Status::RateLimited);
        assert_eq!(replayed.windows[0].source, Source::Cache);
    }

    #[test]
    fn a_429_with_nothing_cached_shows_the_account_without_numbers() {
        let mut state = AccountState::default();
        let shown = state.record_rate_limit(&POLICY, &account(), None, t0());

        assert_eq!(shown.status, Status::RateLimited);
        assert!(shown.windows.is_empty());
        assert_eq!(shown.account, account());
    }

    #[test]
    fn a_failed_answer_shows_the_last_numbers_as_stale() {
        let mut state = after_success();
        state.start_request(&POLICY, t0() + minutes(10), Some("gen-1".into()));
        let shown = state
            .record_failure(
                &POLICY,
                "Anthropic answered HTTP 503.".into(),
                t0() + minutes(10),
            )
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
            decide(&POLICY, &state, &account(), Some("gen-1"), t0() + minutes(12)),
            Decision::Replay(snapshot) if snapshot.status == Status::Stale
        ));
    }

    #[test]
    fn a_failed_answer_with_nothing_cached_waits_with_the_reason() {
        let mut state = AccountState::default();
        state.start_request(&POLICY, t0(), None);
        let shown = state.record_failure(&POLICY, "Anthropic answered HTTP 503.".into(), t0());

        assert_eq!(
            shown,
            Err(
                "Anthropic answered HTTP 503. QuotaDial asks Anthropic at most every 5 minutes, \
                 so the next request is after 2099-01-01T00:05:00Z."
                    .to_owned()
            )
        );
        assert_eq!(
            decide(&POLICY, &state, &account(), None, t0() + minutes(2)),
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
    fn good_numbers_remember_the_sign_in_they_came_from() {
        let mut state = after_success();
        assert_eq!(state.last_good_generation.as_deref(), Some("gen-1"));

        state.start_request(&POLICY, t0() + minutes(5), Some("gen-2".into()));
        state.record_usage(Snapshot::unsupported(account(), "No limits."));
        assert_eq!(state.last_good_generation, None);
    }

    /// The good snapshot with its window at `used_pct`, fetched `minutes_later` after `t0`.
    fn reading(used_pct: f64, minutes_later: i64) -> Snapshot {
        let mut snapshot = good_snapshot();
        snapshot.windows[0].used_pct = used_pct;
        snapshot.windows[0].fetched_at = t0() + minutes(minutes_later);
        snapshot
    }

    #[test]
    fn a_sudden_zero_is_unconfirmed_until_a_second_fetch_agrees() {
        let mut state = after_success();

        state.start_request(&POLICY, t0() + minutes(5), Some("gen-1".into()));
        let first = state.record_usage(reading(0.0, 5));
        assert!(first.windows[0].unconfirmed, "42% then 0% before the reset");
        let Decision::Replay(replayed) = decide(
            &POLICY,
            &state,
            &account(),
            Some("gen-1"),
            t0() + minutes(7),
        ) else {
            panic!("expected a replay");
        };
        assert!(replayed.windows[0].unconfirmed, "replays keep the flag");

        let second = state.record_usage(reading(0.0, 10));
        assert!(!second.windows[0].unconfirmed, "the second 0% agrees");

        let rise = state.record_usage(reading(3.0, 15));
        assert!(!rise.windows[0].unconfirmed);
    }

    #[test]
    fn a_zero_after_a_reset_or_with_no_known_reset_is_trusted() {
        // The session window reset at 05:00.
        let mut after_reset = after_success();
        let shown = after_reset.record_usage(reading(0.0, 6 * 60));
        assert!(!shown.windows[0].unconfirmed);

        let mut unknown = AccountState::default();
        let mut no_reset = good_snapshot();
        no_reset.windows[0].resets_at = None;
        unknown.record_usage(no_reset);
        let shown = unknown.record_usage(reading(0.0, 5));
        assert!(!shown.windows[0].unconfirmed);

        // Another window's earlier reading says nothing about this one.
        let mut other = after_success();
        let mut weekly = reading(0.0, 5);
        weekly.windows[0].label = "Weekly".into();
        assert!(!other.record_usage(weekly).windows[0].unconfirmed);
    }

    #[test]
    fn an_unsupported_answer_clears_the_cached_numbers() {
        let mut state = after_success();
        state.record_usage(Snapshot::unsupported(account(), "No limits."));

        assert_eq!(state.last_good, None);
    }

    #[test]
    fn a_clock_moved_back_counts_the_last_request_as_just_sent() {
        let mut state = after_success();
        let now = t0() - TimeDelta::hours(3);

        assert!(state.reanchor(now));
        assert_eq!(state.requested_at, Some(now));
        assert!(matches!(
            decide(&POLICY, &state, &account(), Some("gen-1"), now + minutes(4)),
            Decision::Replay(_)
        ));
        assert_eq!(
            decide(&POLICY, &state, &account(), Some("gen-1"), now + minutes(5)),
            Decision::Request
        );
        // Once fixed, the times are left alone.
        assert!(!state.reanchor(now + minutes(1)));
    }

    #[test]
    fn a_block_keeps_its_length_when_the_clock_moves_back() {
        let mut state = after_success();
        state.start_request(&POLICY, t0() + minutes(10), Some("gen-1".into()));
        state.record_rate_limit(
            &POLICY,
            &account(),
            Some(t0() + minutes(40)),
            t0() + minutes(10),
        );
        let now = t0() - TimeDelta::days(2);

        assert!(state.reanchor(now));
        assert_eq!(state.blocked_until, Some(now + minutes(30)));
        assert!(matches!(
            decide(&POLICY, &state, &account(), Some("gen-1"), now + minutes(29)),
            Decision::Replay(snapshot) if snapshot.status == Status::RateLimited
        ));
    }

    #[test]
    fn a_block_saved_under_a_far_fast_clock_is_capped_at_a_day() {
        let mut state = AccountState {
            requested_at: Some(t0() + TimeDelta::days(400)),
            blocked_until: Some(t0() + TimeDelta::days(800)),
            ..AccountState::default()
        };

        assert!(state.reanchor(t0()));
        assert_eq!(state.blocked_until, Some(t0() + TimeDelta::hours(24)));
    }

    #[test]
    fn a_first_request_with_no_recorded_answer_still_holds_the_floor() {
        let mut state = AccountState::default();
        state.start_request(&POLICY, t0(), Some("gen-1".into()));

        assert_eq!(
            decide(
                &POLICY,
                &state,
                &account(),
                Some("gen-1"),
                t0() + minutes(1)
            ),
            Decision::Wait(
                "The request sent at 2099-01-01T00:00:00Z has no recorded answer, perhaps because \
                 QuotaDial was stopped. QuotaDial asks Anthropic at most every 5 minutes, so the \
                 next request is after 2099-01-01T00:05:00Z."
                    .into()
            )
        );
        assert_eq!(
            decide(
                &POLICY,
                &state,
                &account(),
                Some("gen-1"),
                t0() + minutes(5)
            ),
            Decision::Request
        );
    }

    #[test]
    fn a_request_stopped_midway_replays_the_previous_answer() {
        let mut state = after_success();
        state.start_request(&POLICY, t0() + minutes(6), Some("gen-1".into()));

        assert!(matches!(
            decide(&POLICY, &state, &account(), Some("gen-1"), t0() + minutes(7)),
            Decision::Replay(snapshot) if snapshot.status == Status::Ok
        ));
    }

    #[test]
    fn numbers_of_an_unnamed_sign_in_are_forgotten_when_its_token_changes() {
        let mut state = after_success();

        assert!(!state.forget_numbers_after_new_sign_in(&POLICY, Some("gen-1")));
        assert!(state.forget_numbers_after_new_sign_in(&POLICY, Some("gen-2")));
        assert_eq!(state.last_good, None);
        assert_eq!(
            decide(&POLICY, &state, &account(), Some("gen-2"), t0() + minutes(2)),
            Decision::Wait(
                "Claude Code's sign-in changed since QuotaDial last asked Anthropic. QuotaDial asks \
                 Anthropic at most every 5 minutes, so the next request is after \
                 2099-01-01T00:05:00Z."
                    .into()
            )
        );
        // Blocks still hold, without the old numbers.
        let mut blocked = after_success();
        blocked.record_rate_limit(&POLICY, &account(), None, t0());
        assert!(blocked.forget_numbers_after_new_sign_in(&POLICY, Some("gen-2")));
        let Decision::Replay(snapshot) = decide(
            &POLICY,
            &blocked,
            &account(),
            Some("gen-2"),
            t0() + minutes(1),
        ) else {
            panic!("expected a replay");
        };
        assert_eq!(snapshot.status, Status::RateLimited);
        assert!(snapshot.windows.is_empty());
    }

    #[test]
    fn a_rejected_unnamed_sign_in_keeps_its_early_retry() {
        let mut state = AccountState::default();
        state.start_request(&POLICY, t0(), Some("gen-1".into()));
        state.record_auth_failure(&Snapshot::auth_required(account(), "Run `claude`."));

        assert!(!state.forget_numbers_after_new_sign_in(&POLICY, Some("gen-2")));
        assert_eq!(
            decide(
                &POLICY,
                &state,
                &account(),
                Some("gen-2"),
                t0() + minutes(1)
            ),
            Decision::Request
        );
    }

    #[test]
    fn only_one_early_retry_per_floor_after_a_rejection() {
        let mut state = AccountState::default();
        state.start_request(&POLICY, t0(), Some("gen-1".into()));
        state.record_auth_failure(&Snapshot::auth_required(account(), "Run `claude`."));

        // The first new token is tried at once.
        assert_eq!(
            decide(
                &POLICY,
                &state,
                &account(),
                Some("gen-2"),
                t0() + minutes(1)
            ),
            Decision::Request
        );
        state.start_request(&POLICY, t0() + minutes(1), Some("gen-2".into()));
        assert!(state.bypassed_floor);
        state.record_auth_failure(&Snapshot::auth_required(account(), "Run `claude`."));

        // A third token inside the same floor waits.
        assert!(matches!(
            decide(
                &POLICY,
                &state,
                &account(),
                Some("gen-3"),
                t0() + minutes(2)
            ),
            Decision::Replay(_)
        ));
        // After the floor, requests resume and the early retry is available again.
        assert_eq!(
            decide(
                &POLICY,
                &state,
                &account(),
                Some("gen-3"),
                t0() + minutes(6)
            ),
            Decision::Request
        );
        state.start_request(&POLICY, t0() + minutes(6), Some("gen-3".into()));
        assert!(!state.bypassed_floor);
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
