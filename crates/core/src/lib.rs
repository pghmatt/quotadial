//! Normalized usage model and the `Provider` trait shared by every QuotaDial surface.
//!
//! Every provider plugin returns these types, so the CLI, the local API, the MCP server and the
//! tray app never see a provider's own response shapes. The types follow the PRD's "Normalized
//! model" and "Provider trait" sections (`docs/PRD-public.md`) field for field.

mod alert;
mod cost;
mod error;
pub mod licence;
mod model;
mod pace;
mod provider;
mod redact;
mod report;
mod secret;
pub mod text;
mod tier;

pub use alert::{Condition, EXAMPLES, Occurrence, Rule, RuleError, Scope};
pub use cost::{Cost, ModelCost, Tokens};
pub use error::{Error, Result};
pub use licence::{KeyError, Licence, Plan, Standing};
pub use model::{Account, ExtraUsage, Pace, Snapshot, Source, Status, Window, WindowKind};
pub use pace::{MIN_READINGS, MIN_SPAN, Reading};
pub use provider::{AccountLimit, Provider};
pub use redact::{REDACTED, outside_text, redact_secrets};
pub use report::{Indicator, LeftOut, ProviderError, StatusPage, StatusReport, collect};
pub use secret::Secret;
pub use tier::{PRO_URL, ProFeature, Tier};
