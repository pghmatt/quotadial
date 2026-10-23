//! Normalized usage model and the `Provider` trait shared by every QuotaDial surface.
//!
//! Every provider plugin returns these types, so the CLI, the local API, the MCP server and the
//! tray app never see a provider's own response shapes. The types follow the PRD's "Normalized
//! model" and "Provider trait" sections (`docs/PRD-public.md`) field for field.

mod error;
mod model;
mod provider;
mod redact;
mod report;
mod secret;

pub use error::{Error, Result};
pub use model::{Account, ExtraUsage, Snapshot, Source, Status, Window, WindowKind};
pub use provider::Provider;
pub use redact::{REDACTED, outside_text, redact_secrets};
pub use report::{ProviderError, StatusReport, collect};
pub use secret::Secret;
