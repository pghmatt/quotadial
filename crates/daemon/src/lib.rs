//! The QuotaDial daemon: one long-running process that owns the only poller, shared by every
//! consumer, and keeps the history of what it found.
//!
//! While it runs, nothing else polls: the CLI, the local API, the MCP server and later the tray
//! app read what the poller last found. Providers still enforce their own floors through the shared
//! gate, so a CLI run that does not know about the daemon cannot ask early either. History goes to
//! SQLite in the OS data directory, kept for 30 days. Only one daemon runs at a time.

mod alerts;
mod data_dir;
mod history;
mod instance;
mod poller;

pub use alerts::{AddError, Alerts, Firing, RECENT_FIRINGS, StoredRule};
pub use data_dir::DataDir;
pub use history::{History, HistoryError, KEEP_DAYS};
pub use instance::{Instance, InstanceError};
pub use poller::{ACTIVE_FOR, ALERTS_EVERY, DISCOVER_EVERY, IDLE_EVERY, Latest, Poller};
