//! QuotaDial's local HTTP API: the server the daemon runs on `127.0.0.1`, and the client the CLI
//! and the MCP server use to ask it. Also the MCP server itself, `quotadial mcp`, and [`Host`],
//! which runs the daemon for `quotadial daemon` and the tray app alike.
//!
//! - `GET /v1/status` returns the same report as `quotadial status --json`.
//! - `GET /v1/headroom` ranks the accounts by the room their tightest window leaves.
//!
//! [`diagnostics`] gathers what `quotadial doctor` prints and the app's Send feedback attaches.
//!
//! Every request needs the bearer token from the `token` file in the data directory, which only
//! the user can read, and must be addressed to `127.0.0.1` or `localhost`. The running daemon
//! writes its port to `daemon.json` beside it.

pub mod client;
pub mod diagnostics;
mod headroom;
mod host;
pub mod mcp;
mod runtime;
mod server;
mod token;

pub use headroom::{Headroom, HeadroomReport, rank};
pub use host::{Host, HostError};
pub use runtime::Runtime;
pub use server::{DEFAULT_PORT, bind, serve};
pub use token::Token;

#[cfg(test)]
mod tests;
