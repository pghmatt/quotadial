//! The QuotaDial daemon: one long-running process that owns the only poller, shared by every
//! consumer, and keeps the history of what it found.
//!
//! While it runs, nothing else polls: the CLI, the local API, the MCP server and later the tray
//! app read what the poller last found. Providers still enforce their own floors through the shared
//! gate, so a CLI run that does not know about the daemon cannot ask early either. History goes to
//! SQLite in the OS data directory, kept for 30 days. Only one daemon runs at a time.

mod alerts;
mod config;
mod data_dir;
mod deliver;
mod deliveries;
mod history;
mod instance;
mod poller;
mod series;
mod status_pages;
#[cfg(test)]
mod test_support;

pub use alerts::{AddError, Alerts, Firing, RECENT_FIRINGS, StoredRule};
pub use config::{Config, Webhook, WebhookKind};
pub use data_dir::DataDir;
pub use deliver::{Deliverer, payload};
pub use deliveries::{DELIVER_WITHIN, Delivery, deliver_due};
pub use history::{History, HistoryError, KEEP_DAYS, MAX_KEEP_DAYS};
pub use instance::{Instance, InstanceError};
pub use poller::{ACTIVE_FOR, ALERTS_EVERY, DISCOVER_EVERY, IDLE_EVERY, Latest, Poller};
pub use series::{Point, Series, SeriesQuery, write_csv};
pub use status_pages::{STATUS_EVERY, STATUS_PAGES, StatusReader, parse as parse_status_page};
