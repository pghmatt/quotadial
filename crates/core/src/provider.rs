use std::fmt;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;

use crate::{Account, Result, Snapshot};

/// A command a provider adds to QuotaDial's CLI, which runs it as `quotadial <provider id> <name>`
/// (P-018). The CLI, its completions and its manual pages find a provider's commands through
/// [`Provider::commands`], so a provider adds one without the CLI knowing the provider's name.
pub struct ProviderCommand {
    /// One word, such as `statusline`.
    pub name: &'static str,
    /// One line saying what it is for, which `quotadial <provider id> --help` and the manual pages
    /// show.
    pub about: &'static str,
    /// Runs the command and gives the exit status the CLI exits with.
    pub run: Box<dyn Fn(Invocation<'_>) -> ExitCode>,
}

impl fmt::Debug for ProviderCommand {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProviderCommand")
            .field("name", &self.name)
            .field("about", &self.about)
            .finish_non_exhaustive()
    }
}

/// What a provider's command runs with: the words typed after its name and the standard streams,
/// which a test replaces with its own.
pub struct Invocation<'a> {
    /// The words after the command's name, as they were typed.
    pub args: &'a [String],
    /// Standard input.
    pub stdin: &'a mut dyn Read,
    /// Standard output.
    pub stdout: &'a mut dyn Write,
    /// Standard error.
    pub stderr: &'a mut dyn Write,
}

/// The most accounts a provider shows, asked afresh at each discovery, so a change, such as a
/// licence key added, applies without a restart (D-100). `None` is every account.
#[derive(Clone)]
pub struct AccountLimit(Arc<dyn Fn() -> Option<usize> + Send + Sync>);

impl AccountLimit {
    /// No limit: every account.
    #[must_use]
    pub fn none() -> Self {
        Self::fixed(None)
    }

    /// Always `max`.
    #[must_use]
    pub fn fixed(max: Option<usize>) -> Self {
        Self(Arc::new(move || max))
    }

    /// Whatever `limit` says at each discovery.
    #[must_use]
    pub fn new(limit: impl Fn() -> Option<usize> + Send + Sync + 'static) -> Self {
        Self(Arc::new(limit))
    }

    /// The limit now, at least 1 when there is one.
    #[must_use]
    pub fn get(&self) -> Option<usize> {
        (self.0)().map(|max| max.max(1))
    }
}

impl Default for AccountLimit {
    fn default() -> Self {
        Self::none()
    }
}

impl fmt::Debug for AccountLimit {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("AccountLimit")
            .field(&self.get())
            .finish()
    }
}

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

    /// How many more accounts the last [`Provider::discover_accounts`] found than it gave, held back
    /// by the provider's [`AccountLimit`]. None by default.
    fn held_back(&self) -> usize {
        0
    }

    /// What the last [`Provider::discover_accounts`] could not look at, each a sentence naming what
    /// may be missing and why, such as a keychain that could not be listed. They are shown beside
    /// the accounts it did find. None by default.
    fn discovery_warnings(&self) -> Vec<String> {
        Vec::new()
    }

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

    /// The folders where the official CLI writes its session logs, which the cost scan reads for
    /// token counts (D-097). None by default.
    fn log_folders(&self) -> Vec<PathBuf> {
        Vec::new()
    }

    /// The provider's own commands, which the CLI runs as `quotadial <id> <name>` and lists in its
    /// help, completions and manual pages (P-018). None by default.
    fn commands(&self) -> Vec<ProviderCommand> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    /// A provider that adds nothing to the trait's defaults.
    struct Plain;

    #[async_trait]
    impl Provider for Plain {
        fn id(&self) -> &'static str {
            "plain"
        }

        fn discover_accounts(&self) -> Result<Vec<Account>> {
            Ok(Vec::new())
        }

        async fn fetch(&self, account: &Account) -> Result<Snapshot> {
            Ok(Snapshot::unsupported(account.clone(), "No usage."))
        }

        fn fallback(&self, _account: &Account) -> Option<Snapshot> {
            None
        }

        fn min_poll_interval(&self) -> Duration {
            Duration::from_secs(60)
        }
    }

    #[test]
    fn a_provider_has_no_commands_of_its_own_unless_it_adds_them() {
        assert!(Plain.commands().is_empty());
    }

    #[test]
    fn a_commands_debug_output_names_it() {
        let command = ProviderCommand {
            name: "statusline",
            about: "Keeps what the status line hands over.",
            run: Box::new(|_| ExitCode::SUCCESS),
        };

        assert_eq!(
            format!("{command:?}"),
            "ProviderCommand { name: \"statusline\", about: \"Keeps what the status line hands \
             over.\", .. }"
        );
    }

    #[test]
    fn a_limit_is_asked_afresh_and_is_never_below_one() {
        let max = Arc::new(AtomicUsize::new(3));
        let read = Arc::clone(&max);
        let limit = AccountLimit::new(move || Some(read.load(Ordering::Relaxed)));

        assert_eq!(limit.get(), Some(3));
        max.store(0, Ordering::Relaxed);
        assert_eq!(limit.get(), Some(1));
        assert_eq!(AccountLimit::default().get(), None);
        assert_eq!(AccountLimit::fixed(Some(2)).get(), Some(2));
    }
}
