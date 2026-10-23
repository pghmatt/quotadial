//! What QuotaDial reports for every account at one moment: the shape `quotadial status --json`
//! prints and the daemon's local API serves (`docs/CLI.md`).

use serde::{Deserialize, Serialize};

use crate::{Error, Provider, Snapshot, redact_secrets};

/// A snapshot per account, and every provider or account that produced none.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StatusReport {
    /// One per account, in provider order.
    pub snapshots: Vec<Snapshot>,
    /// Providers or accounts that produced no snapshot.
    pub errors: Vec<ProviderError>,
}

/// A provider or account that produced no snapshot, for example because its credentials were
/// unreadable. The message names the fix.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderError {
    /// The provider id, as in [`Provider::id`].
    pub provider: String,
    /// What happened and what to do, with any token blanked out.
    pub message: String,
}

impl ProviderError {
    /// `error` from the provider `provider`, with any token blanked out of its text.
    #[must_use]
    pub fn new(provider: &str, error: &Error) -> Self {
        Self {
            provider: provider.to_owned(),
            message: redact_secrets(&error.to_string()).into_owned(),
        }
    }
}

/// Asks every provider for its accounts, then fetches each account, using the provider's local
/// fallback when the fetch fails. Error text is checked for tokens here, whichever provider wrote
/// it, so none reaches a report.
pub async fn collect<P: AsRef<dyn Provider>>(providers: &[P]) -> StatusReport {
    let mut report = StatusReport::default();
    for provider in providers {
        let provider = provider.as_ref();
        let accounts = match provider.discover_accounts() {
            Ok(accounts) => accounts,
            Err(error) => {
                report
                    .errors
                    .push(ProviderError::new(provider.id(), &error));
                continue;
            }
        };
        for account in accounts {
            match provider.fetch(&account).await {
                Ok(snapshot) => report.snapshots.push(snapshot.without_secrets()),
                Err(error) => match provider.fallback(&account) {
                    Some(snapshot) => report.snapshots.push(snapshot.without_secrets()),
                    None => report
                        .errors
                        .push(ProviderError::new(provider.id(), &error)),
                },
            }
        }
    }
    report
}
