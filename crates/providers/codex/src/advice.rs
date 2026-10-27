//! The one sentence QuotaDial shows when it has no fresh numbers for a ChatGPT account, naming the
//! fix.

use chrono::{DateTime, SecondsFormat, Utc};
use quotadial_core::text;

pub(crate) const SIGNED_OUT: &str =
    "The Codex CLI is not signed in on this machine any more. Run `codex login` to sign in.";

pub(crate) const DIFFERENT_ACCOUNT: &str = "The Codex CLI on this machine is now signed in to a \
     different account, so this account's usage cannot be read.";

/// ChatGPT answered 401, the issue's wording (#2).
pub(crate) const REJECTED: &str =
    "ChatGPT rejected the Codex CLI's sign-in. Run `codex login` to sign in again.";

pub(crate) const API_KEY: &str = "The Codex CLI is signed in with an OpenAI API key, which has no \
     ChatGPT plan limits to show. Run `codex login` with a ChatGPT account to track them.";

pub(crate) const NO_LIMITS: &str = "ChatGPT reported no usage limits for this account.";

/// The sign-in's token expired at `at`, written as a person reads it (D-142), and the Codex CLI
/// refreshes it when it next runs.
pub(crate) fn expired(at: DateTime<Utc>) -> String {
    format!(
        "The Codex CLI's sign-in expired on {}. Run `codex` once to refresh it, or `codex login` \
         to sign in again.",
        text::date_and_time(at)
    )
}

/// ChatGPT answered with a status QuotaDial cannot use.
pub(crate) fn failed(status: u16, message: Option<&str>) -> String {
    match message {
        Some(message) => format!(
            "ChatGPT's usage endpoint answered HTTP {status}: {}.",
            message.trim_end_matches('.')
        ),
        None => format!("ChatGPT's usage endpoint answered HTTP {status}."),
    }
}

/// Numbers from the Codex CLI's session log, shown when ChatGPT could not be asked.
pub(crate) fn from_session_log(at: DateTime<Utc>) -> String {
    format!(
        "From the Codex CLI's session log at {}, because ChatGPT's usage endpoint could not be \
         read just now.",
        timestamp(at)
    )
}

fn timestamp(at: DateTime<Utc>) -> String {
    at.to_rfc3339_opts(SecondsFormat::Secs, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ts(rfc3339: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(rfc3339)
            .expect("valid timestamp")
            .with_timezone(&Utc)
    }

    #[test]
    fn every_message_names_the_fix_or_the_source() {
        assert!(SIGNED_OUT.contains("codex login"));
        assert!(REJECTED.contains("codex login"));
        assert!(API_KEY.contains("codex login"));
        // The time as a person reads it, its seconds dropped (D-142).
        assert_eq!(
            expired(ts("2026-10-26T09:12:47Z")),
            "The Codex CLI's sign-in expired on 26 Oct 2026 at 09:12 UTC. Run `codex` once to \
             refresh it, or `codex login` to sign in again."
        );
        assert_eq!(
            failed(503, Some("Service unavailable.")),
            "ChatGPT's usage endpoint answered HTTP 503: Service unavailable."
        );
        assert_eq!(
            failed(502, None),
            "ChatGPT's usage endpoint answered HTTP 502."
        );
        assert_eq!(
            from_session_log(ts("2099-01-01T00:00:00Z")),
            "From the Codex CLI's session log at 2099-01-01T00:00:00Z, because ChatGPT's usage \
             endpoint could not be read just now."
        );
    }
}
