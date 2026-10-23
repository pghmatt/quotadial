//! When QuotaDial may ask a provider's usage endpoint, and what it remembers about each account
//! between runs.
//!
//! Every provider uses it, so the floor between requests, the 429 backoff and the replay from the
//! cache behave the same way for every account. M2's single poller and SQLite store replace the
//! files kept here.

mod clock;
mod gate;
mod state;

use chrono::{DateTime, TimeDelta, Utc};

pub use clock::{Clock, SystemClock};
pub use gate::{Decision, decide, retry_after, unanswered};
pub use state::{AccountState, LOCK_WAIT, LastResult, StateLock, StateStore, account_key};

/// How often QuotaDial may ask one provider for an account's usage, and how messages name who it
/// asks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Policy {
    /// Who answers usage requests, for example `"Anthropic"`.
    pub service: &'static str,
    /// The official CLI whose sign-in QuotaDial reads, as it starts a sentence, for example
    /// `"Claude Code"`.
    pub cli: &'static str,
    /// At most one request per account this often.
    pub floor: TimeDelta,
}

impl Policy {
    /// For example "QuotaDial asks Anthropic at most every 5 minutes, so the next request is after
    /// 2099-01-01T00:05:00Z."
    fn next_request(&self, at: DateTime<Utc>) -> String {
        let every = match self.floor.num_minutes() {
            1 => "minute".to_owned(),
            minutes => format!("{minutes} minutes"),
        };
        format!(
            "QuotaDial asks {} at most every {every}, so the next request is after {}.",
            self.service,
            gate::timestamp(at)
        )
    }
}
