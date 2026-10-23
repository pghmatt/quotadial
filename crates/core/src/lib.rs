//! Normalized usage model and the `Provider` trait shared by every QuotaDial surface.
//!
//! Every provider plugin returns these types, so the CLI, the local API, the MCP server and the
//! tray app never see a provider's own response shapes. The types follow the PRD's "Normalized
//! model" and "Provider trait" sections (`docs/PRD-public.md`) field for field.

mod error;
mod model;
mod pace;
mod provider;
mod redact;
mod report;
mod secret;
pub mod text;

pub use error::{Error, Result};
pub use model::{Account, ExtraUsage, Pace, Snapshot, Source, Status, Window, WindowKind};
pub use pace::{MIN_READINGS, MIN_SPAN, Reading};
pub use provider::Provider;
pub use redact::{REDACTED, outside_text, redact_secrets};
pub use report::{ProviderError, StatusReport, collect};
pub use secret::Secret;
