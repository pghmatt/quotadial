/// Shorthand for results whose error is a QuotaDial [`Error`].
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Failures that leave a provider with no snapshot to return.
///
/// Expected states are not errors. An expired login, a rate limit or an unsupported plan comes back
/// as a [`Snapshot`](crate::Snapshot) whose [`Status`](crate::Status) and `error` message name the
/// fix. Messages never contain tokens, cookies or keys.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Stored credentials exist but could not be read or parsed.
    #[error("{provider}: could not read credentials from {location}: {reason}")]
    Credentials {
        /// The provider id, as in [`Provider::id`](crate::Provider::id).
        provider: &'static str,
        /// The file or keychain item that was read, so the user can see exactly what was touched.
        location: String,
        /// Why reading failed.
        reason: String,
    },

    /// The request to the provider failed before a usable response arrived.
    #[error("{provider}: request failed: {reason}")]
    Request {
        /// The provider id, as in [`Provider::id`](crate::Provider::id).
        provider: &'static str,
        /// Why the request failed.
        reason: String,
    },

    /// The provider answered with a response QuotaDial cannot interpret.
    #[error("{provider}: unexpected response: {reason}")]
    InvalidResponse {
        /// The provider id, as in [`Provider::id`](crate::Provider::id).
        provider: &'static str,
        /// What was wrong with the response.
        reason: String,
    },

    /// QuotaDial's own cache could not be used. Without it the provider cannot enforce its polling
    /// limits across runs, so it sends no request.
    #[error("{provider}: could not use QuotaDial's cache at {location}: {reason}")]
    Cache {
        /// The provider id, as in [`Provider::id`](crate::Provider::id).
        provider: &'static str,
        /// The cache directory.
        location: String,
        /// Why it could not be used.
        reason: String,
    },
}
