//! What the week's local use would have cost on the providers' APIs (D-097): the requests in
//! Claude Code's and the Codex CLI's session logs, priced from a table bundled with each release.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Tokens of one request, or summed over many, by kind.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tokens {
    /// Input read fresh, outside the prompt cache.
    pub input: u64,
    /// Output, reasoning included.
    pub output: u64,
    /// Input read from the prompt cache.
    pub cache_read: u64,
    /// Input written to the prompt cache.
    pub cache_write: u64,
    /// The part of `cache_write` kept for an hour rather than five minutes, which costs more.
    /// Claude only; left out of JSON when 0.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub cache_write_1h: u64,
}

fn is_zero(count: &u64) -> bool {
    *count == 0
}

// Counts come from logs on disk, which can hold anything, so sums saturate rather than overflow:
// a damaged count can make a total absurd, never panic or wrap round to a small one.
impl Tokens {
    /// Every token, read or written.
    #[must_use]
    pub fn total(self) -> u64 {
        self.input
            .saturating_add(self.output)
            .saturating_add(self.cache_read)
            .saturating_add(self.cache_write)
    }

    /// The context the model read: fresh input and the cache, read or written.
    #[must_use]
    pub fn context(self) -> u64 {
        self.input
            .saturating_add(self.cache_read)
            .saturating_add(self.cache_write)
    }
}

impl std::ops::AddAssign for Tokens {
    fn add_assign(&mut self, other: Self) {
        self.input = self.input.saturating_add(other.input);
        self.output = self.output.saturating_add(other.output);
        self.cache_read = self.cache_read.saturating_add(other.cache_read);
        self.cache_write = self.cache_write.saturating_add(other.cache_write);
        self.cache_write_1h = self.cache_write_1h.saturating_add(other.cache_write_1h);
    }
}

/// What one provider's local use over a period would have cost on its API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cost {
    /// The provider id, such as `claude`.
    pub provider: String,
    /// The start of the period: a week before `until`.
    pub since: DateTime<Utc>,
    /// The end of the period: when the logs were last read.
    pub until: DateTime<Utc>,
    /// The period's requests at the API's prices, in US dollars. Models with no price are left
    /// out, and named in `notes`.
    pub usd: f64,
    /// How many requests the logs hold for the period.
    pub requests: u64,
    pub tokens: Tokens,
    /// Each model, the costliest first.
    pub models: Vec<ModelCost>,
    /// When the price table was made, such as `2026-10-23`.
    pub prices_updated: String,
    /// What the cost leaves out, each a sentence.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

/// One model's part of a [`Cost`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelCost {
    /// The model, as the logs name it.
    pub model: String,
    /// Its requests at the API's prices, in US dollars. Left out when the price table has no price
    /// for it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usd: Option<f64>,
    pub requests: u64,
    pub tokens: Tokens,
}
