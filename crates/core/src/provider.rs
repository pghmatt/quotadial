use std::time::Duration;

use async_trait::async_trait;

use crate::{Account, Result, Snapshot};

/// The contract every provider plugin meets. From the PRD:
///
/// ```text
/// trait Provider {
///     fn id(&self) -> &'static str;
///     fn discover_accounts(&self) -> Result<Vec<Account>>;   // read local credentials
///     async fn fetch(&self, account: &Account) -> Result<Snapshot>;
///     fn fallback(&self, account: &Account) -> Option<Snapshot>; // local logs
///     fn min_poll_interval(&self) -> Duration;
/// }
/// ```
///
/// `#[async_trait]` keeps the trait object-safe, so the CLI, daemon and API can hold every
/// provider as a `Box<dyn Provider>`. `Send + Sync` lets one poller share providers across tasks.
#[async_trait]
pub trait Provider: Send + Sync {
    /// The stable provider id, for example `"claude"`. Matches [`Account::provider`].
    fn id(&self) -> &'static str;

    /// Finds the accounts this machine is signed in to by reading the official CLI's stored
    /// credentials. Never writes, refreshes or copies them.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Credentials`](crate::Error::Credentials) when stored credentials exist but
    /// cannot be read. Finding no credentials is not an error: it returns an empty list.
    fn discover_accounts(&self) -> Result<Vec<Account>>;

    /// Fetches the account's current usage from the provider.
    ///
    /// # Errors
    ///
    /// Returns an error only when no snapshot can be built. An expired login, a rate limit or an
    /// unsupported plan is reported through the snapshot's status instead.
    async fn fetch(&self, account: &Account) -> Result<Snapshot>;

    /// Builds a snapshot from the official CLI's local logs, for when the endpoint cannot be used.
    fn fallback(&self, account: &Account) -> Option<Snapshot>;

    /// The shortest interval at which this provider's endpoint may be polled for one account.
    fn min_poll_interval(&self) -> Duration;
}
