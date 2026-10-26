/// Shorthand for results whose error is a QuotaDial [`Error`].
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Failures that leave a provider with no snapshot to return.
///
/// Expected states are not errors. An expired login, a rate limit or an unsupported plan comes back
/// as a [`Snapshot`](crate::Snapshot) whose [`Status`](crate::Status) and `error` message name the
/// fix. Messages never contain tokens, cookies or keys.
///
/// The text says what happened and what to do, in #52's plain words, and leaves the provider out:
/// a report keeps it beside the provider's id, and every surface shows it under the provider's own
/// name ([`StatusReport::error_lines`](crate::StatusReport::error_lines), D-128). Each variant's
/// `provider` is the id, for code.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The sign-in the official CLI stored is there but could not be read or parsed.
    #[error("could not read the sign-in from {location}: {reason}")]
    Credentials {
        /// The provider id, as in [`Provider::id`](crate::Provider::id).
        provider: &'static str,
        /// The file, keychain item or variable that was read, so the user can see exactly what was
        /// touched, written to follow "from", such as `~/.codex/auth.json` or `the macOS Keychain
        /// item "Claude Code-credentials"`.
        location: String,
        /// Why reading failed.
        reason: String,
    },

    /// The request to the provider failed before a usable response arrived.
    #[error("request failed: {reason}")]
    Request {
        /// The provider id, as in [`Provider::id`](crate::Provider::id).
        provider: &'static str,
        /// Why the request failed.
        reason: String,
    },

    /// The provider answered with a response QuotaDial cannot interpret.
    #[error("unexpected response: {reason}")]
    InvalidResponse {
        /// The provider id, as in [`Provider::id`](crate::Provider::id).
        provider: &'static str,
        /// What was wrong with the response.
        reason: String,
    },

    /// QuotaDial's own cache could not be used. Without it the provider cannot enforce its polling
    /// limits across runs, so it sends no request.
    #[error("could not use QuotaDial's cache at {location}: {reason}")]
    Cache {
        /// The provider id, as in [`Provider::id`](crate::Provider::id).
        provider: &'static str,
        /// The cache directory.
        location: String,
        /// Why it could not be used.
        reason: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::unplain_words;

    /// One of each variant, filled in as providers fill them in. The match names every variant, so
    /// a new one cannot be added without joining the tests below.
    fn one_of_each() -> Vec<Error> {
        let errors = vec![
            Error::Credentials {
                provider: "claude",
                location: "~/.claude/.credentials.json".into(),
                reason: "it is not valid JSON; run `claude` to sign in again".into(),
            },
            Error::Request {
                provider: "codex",
                reason: "Could not reach ChatGPT: no answer. Check the internet connection.".into(),
            },
            Error::InvalidResponse {
                provider: "gemini",
                reason: "someone-else is not one of the ways Gemini CLI signs in that QuotaDial \
                         shows"
                    .into(),
            },
            Error::Cache {
                provider: "antigravity",
                location: "~/Library/Caches/quotadial".into(),
                reason: "the folder cannot be written; check that you can write it".into(),
            },
        ];
        for error in &errors {
            match error {
                Error::Credentials { .. }
                | Error::Request { .. }
                | Error::InvalidResponse { .. }
                | Error::Cache { .. } => {}
            }
        }
        errors
    }

    #[test]
    fn every_error_says_what_happened_in_plain_words() {
        // #52: never "quota", "auth" or "credential" in anything a user reads; a sign-in is a
        // sign-in.
        for error in one_of_each() {
            let text = error.to_string();
            assert_eq!(unplain_words(&text), Vec::<String>::new(), "{text}");
        }
    }

    #[test]
    fn no_error_begins_with_its_provider_which_the_report_names() {
        for error in one_of_each() {
            let (Error::Credentials { provider, .. }
            | Error::Request { provider, .. }
            | Error::InvalidResponse { provider, .. }
            | Error::Cache { provider, .. }) = &error;
            let text = error.to_string();
            assert!(!text.starts_with(provider), "{text}");
        }
    }

    #[test]
    fn a_sign_in_that_cannot_be_read_names_where_it_is_and_why() {
        let error = Error::Credentials {
            provider: "codex",
            location: "~/.codex/auth.json".into(),
            reason: "the file is not valid JSON; run `codex login` to sign in again".into(),
        };

        assert_eq!(
            error.to_string(),
            "could not read the sign-in from ~/.codex/auth.json: the file is not valid JSON; run \
             `codex login` to sign in again"
        );
    }
}
