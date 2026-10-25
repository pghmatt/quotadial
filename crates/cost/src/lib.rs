//! The API-equivalent cost of the week's local Claude Code and Codex use (D-097): what the
//! requests in the CLIs' session logs would have cost at each provider's API prices, from a table
//! bundled with each release. Ported from CodexBar's `CostUsageScanner*.swift` and
//! `CostUsagePricing.swift`.
//!
//! Only the usage fields of each log line are read: never a prompt, a reply or a file's content.
//! What a scan finds is kept in `costs.db`, so each log is read once, and then only what was added
//! to it.

mod claude;
mod codex;
mod prices;
mod read;
mod scan;
mod store;

use chrono::{DateTime, Utc};
use quotadial_core::Tokens;

pub use prices::{Above, Price, Prices};
pub use scan::{CostScan, KEEP, WEEK};

/// One model request found in a log.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Request {
    /// What names the request in every copy of it, from the ids the log gives. `None` when it
    /// gives none: the request is then named by where it was found.
    pub(crate) key: Option<String>,
    /// When the response was written.
    pub(crate) at: DateTime<Utc>,
    pub(crate) model: String,
    pub(crate) tokens: Tokens,
}

/// Why a scan could not be done: `costs.db` could not be opened, read or written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CostError {
    message: String,
    /// Whether SQLite found the file damaged, or not a database at all.
    damaged: bool,
}

impl CostError {
    fn new(message: String) -> Self {
        Self {
            message,
            damaged: false,
        }
    }

    /// Whether SQLite found `costs.db` damaged, or not a database at all.
    #[must_use]
    pub fn is_damaged(&self) -> bool {
        self.damaged
    }
}

impl std::fmt::Display for CostError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for CostError {}

impl From<rusqlite::Error> for CostError {
    fn from(error: rusqlite::Error) -> Self {
        Self {
            damaged: matches!(
                error.sqlite_error_code(),
                Some(rusqlite::ErrorCode::DatabaseCorrupt | rusqlite::ErrorCode::NotADatabase)
            ),
            message: format!("the cost scan's database: {error}"),
        }
    }
}

impl From<rusqlite::types::FromSqlError> for CostError {
    fn from(error: rusqlite::types::FromSqlError) -> Self {
        Self::new(format!("the cost scan's database: {error}"))
    }
}

impl From<std::io::Error> for CostError {
    fn from(error: std::io::Error) -> Self {
        Self::new(format!("the cost scan's database: {error}"))
    }
}
