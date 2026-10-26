//! What QuotaDial reports for every account at one moment: the shape `quotadial status --json`
//! prints and the daemon's local API serves (`docs/CLI.md`).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{Cost, Error, ProFeature, Provider, ProviderName, Snapshot, Status, redact_secrets};

/// A snapshot per account, and every provider or account that produced none.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct StatusReport {
    /// One per account, in provider order.
    pub snapshots: Vec<Snapshot>,
    /// Providers or accounts that produced no snapshot.
    pub errors: Vec<ProviderError>,
    /// What the providers' status pages say, while the daemon reads them (D-096). Left out of a
    /// direct fetch.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub status_pages: Vec<StatusPage>,
    /// What the week's local use of each provider would have cost on its API, while the daemon
    /// reads the logs (D-097). Left out of a direct fetch.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub costs: Vec<Cost>,
    /// What this copy leaves out because it is QuotaDial Free, each in one line for where it would
    /// be (D-100). Left out on QuotaDial Pro.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub left_out: Vec<LeftOut>,
    /// What is wrong with QuotaDial itself rather than a provider, each in one line that names the
    /// fix or says it is dealt with, such as a history it could not read or save (D-107). Left out
    /// when there is none, and of a direct fetch.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub problems: Vec<String>,
    /// Every provider this copy asks, by its id and its own name for itself, in the order it asks
    /// them (P-020): what a status page, a cost, an error or anything else that has only a
    /// provider's id is named by. A report from an older QuotaDial lists none, and its ids are
    /// shown.
    #[serde(default)]
    pub providers: Vec<ProviderName>,
}

/// Something a QuotaDial Free copy leaves out of a report, such as more accounts on the machine or
/// the week's cost, named in one line.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct LeftOut {
    /// The Pro feature it would take.
    pub feature: ProFeature,
    /// The provider it is about, for its accounts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    /// What is left out, that it is part of QuotaDial Pro, and where Pro is.
    pub text: String,
}

impl LeftOut {
    /// `feature`, left out, with its line ([`ProFeature::line`]).
    #[must_use]
    pub fn feature(feature: ProFeature) -> Self {
        Self {
            feature,
            provider: None,
            text: feature.line(),
        }
    }

    /// `more` accounts of the provider `provider`, which calls itself `name`, held back by QuotaDial
    /// Free's one account per provider.
    #[must_use]
    pub fn accounts(provider: &str, name: &str, more: usize) -> Self {
        let accounts = match more {
            1 => format!("1 more {name} account is"),
            more => format!(
                "{} more {name} accounts are",
                crate::text::thousands(u64::try_from(more).unwrap_or(u64::MAX))
            ),
        };
        Self {
            feature: ProFeature::Accounts,
            provider: Some(provider.to_owned()),
            text: format!(
                "{accounts} on this machine. {}",
                ProFeature::Accounts.line()
            ),
        }
    }
}

/// What a provider's status page says: how bad things are, in the page's own words.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct StatusPage {
    /// The provider id, such as `claude`.
    pub provider: String,
    pub indicator: Indicator,
    /// The page's words, such as "Partial System Degradation".
    pub description: String,
    /// The page, for people.
    pub url: String,
    /// When QuotaDial first saw the page's current trouble. Left out when all is well.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub since: Option<DateTime<Utc>>,
    /// When QuotaDial last read the page.
    pub checked_at: DateTime<Utc>,
}

/// How bad a status page says things are, least first, as Statuspage names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum Indicator {
    /// All is well.
    None,
    /// Planned maintenance.
    Maintenance,
    /// Some of the service is degraded.
    Minor,
    /// A large part of the service is down.
    Major,
    /// The service is down.
    Critical,
}

impl Indicator {
    /// Whether the page reports an incident: anything worse than planned maintenance.
    #[must_use]
    pub fn is_incident(self) -> bool {
        self >= Self::Minor
    }
}

/// A provider or account that produced no snapshot, for example because its sign-in could not be
/// read, or what a provider could not look at while finding its accounts. The message names the
/// fix.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ProviderError {
    /// The provider's id, such as `claude`. The status report's `providers` give the provider's own
    /// name for it.
    // As `Provider::id` gives it.
    pub provider: String,
    /// What happened and what to do, with any token blanked out. It leaves out which provider, which
    /// `provider` says, and QuotaDial shows it after the provider's own name, as `Gemini CLI: could
    /// not read the sign-in from ~/.gemini/settings.json: ...` (D-128). A message from an older
    /// QuotaDial begins with the provider's id and a colon.
    // Never begins with the id: `new` and `warning` see to it.
    pub message: String,
}

impl StatusReport {
    /// What the provider `id` calls itself, as the report's [`StatusReport::providers`] say
    /// (P-020): what a status page, a cost or an incident is shown under. A provider the report
    /// does not list, as in a report from an older QuotaDial, keeps its id.
    #[must_use]
    pub fn provider_name<'a>(&'a self, id: &'a str) -> &'a str {
        ProviderName::find(&self.providers, id)
    }

    /// Every provider's error in the report as every surface shows it, `status` and `watch`,
    /// doctor and the app's problems alike: "{the provider's own name}: {what happened and what to
    /// do}", the name from [`StatusReport::provider_name`], so it is never taken from a table and
    /// the surfaces cannot drift apart (D-128). A message from an older QuotaDial, which began with
    /// its provider's id, loses the id, so the provider is named once.
    pub fn error_lines(&self) -> impl Iterator<Item = String> + '_ {
        self.errors.iter().map(|error| {
            format!(
                "{}: {}",
                self.provider_name(&error.provider),
                without_id(&error.provider, &error.message)
            )
        })
    }

    /// The report as every consumer shows it: one snapshot per account (D-024), in an order that
    /// lasts. Two sign-ins can turn out to belong to one subscription only once each has been
    /// asked, as with Keychain items whose folder is gone (#34): a working snapshot beats one that
    /// needs the user, and otherwise the newer numbers win. Providers keep their order, and each
    /// provider's accounts are sorted by name, so the rows stay put whether they came from a poll,
    /// a rediscovery or the history a restarted daemon shows first.
    #[must_use]
    pub fn normalized(mut self) -> Self {
        let mut kept: Vec<Snapshot> = Vec::with_capacity(self.snapshots.len());
        for snapshot in self.snapshots {
            match kept
                .iter_mut()
                .find(|known| known.account.same_as(&snapshot.account))
            {
                Some(known) if better(&snapshot, known) => *known = snapshot,
                Some(_) => {}
                None => kept.push(snapshot),
            }
        }
        let mut providers: Vec<String> = Vec::new();
        for snapshot in &kept {
            if !providers.contains(&snapshot.account.provider) {
                providers.push(snapshot.account.provider.clone());
            }
        }
        kept.sort_by_cached_key(|snapshot| {
            let account = &snapshot.account;
            (
                providers
                    .iter()
                    .position(|provider| *provider == account.provider),
                account.display_name.to_lowercase(),
                account.account_id.clone(),
                account.org.clone(),
            )
        });
        self.snapshots = kept;
        self
    }
}

/// Whether `candidate` should be shown instead of `known` for the same account.
fn better(candidate: &Snapshot, known: &Snapshot) -> bool {
    let ok = |snapshot: &Snapshot| snapshot.status == Status::Ok;
    let newest = |snapshot: &Snapshot| {
        snapshot
            .windows
            .iter()
            .map(|window| window.fetched_at)
            .max()
    };
    match (ok(candidate), ok(known)) {
        (true, false) => true,
        (false, true) => false,
        _ => newest(candidate) > newest(known),
    }
}

/// `message` without the `<provider id>: ` that every provider's error began with before D-128:
/// exactly the id, as written, then a colon and a space.
fn without_id<'a>(provider: &str, message: &'a str) -> &'a str {
    message
        .strip_prefix(provider)
        .and_then(|rest| rest.strip_prefix(": "))
        .filter(|_| !provider.is_empty())
        .unwrap_or(message)
}

impl ProviderError {
    /// `error` from the provider `provider`, with any token blanked out of its text, which leaves the
    /// provider out as [`Error`]'s text does.
    #[must_use]
    pub fn new(provider: &str, error: &Error) -> Self {
        Self::warning(provider, &error.to_string())
    }

    /// `message`, about what `provider` could not look at, with any token blanked out of it. A
    /// message that begins with the provider's id, as every one did before D-128, loses it, so the
    /// API's `message` never begins with it, whichever provider wrote it.
    #[must_use]
    pub fn warning(provider: &str, message: &str) -> Self {
        Self {
            provider: provider.to_owned(),
            message: redact_secrets(without_id(provider, message)).into_owned(),
        }
    }
}

/// Asks every provider for its accounts, then fetches each account, using the provider's local
/// fallback when the fetch fails. What a provider could not look at while finding its accounts is
/// reported beside them. Error text is checked for tokens here, whichever provider wrote it, so
/// none reaches a report, and what an older QuotaDial left out of a snapshot it kept is taken from
/// the account discovery found (P-020). The report lists every provider asked, by its id and its
/// own name.
pub async fn collect<P: AsRef<dyn Provider>>(providers: &[P]) -> StatusReport {
    let mut report = StatusReport {
        providers: providers
            .iter()
            .map(|provider| ProviderName::of(provider.as_ref()))
            .collect(),
        ..StatusReport::default()
    };
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
        report.errors.extend(
            provider
                .discovery_warnings()
                .iter()
                .map(|warning| ProviderError::warning(provider.id(), warning)),
        );
        for account in accounts {
            match provider.fetch(&account).await {
                Ok(snapshot) => report
                    .snapshots
                    .push(snapshot.completed_from(&account).without_secrets()),
                Err(error) => match provider.fallback(&account) {
                    Some(snapshot) => report
                        .snapshots
                        .push(snapshot.completed_from(&account).without_secrets()),
                    None => report
                        .errors
                        .push(ProviderError::new(provider.id(), &error)),
                },
            }
        }
    }
    report.normalized()
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};

    use super::*;
    use crate::{Account, Source, Window, WindowKind};

    fn at(text: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(text)
            .expect("valid timestamp")
            .with_timezone(&Utc)
    }

    fn snapshot(account_id: &str, place: &str, status: Status, fetched_at: &str) -> Snapshot {
        Snapshot {
            account: Account {
                provider: "claude".into(),
                provider_name: "Claude".into(),
                account_id: account_id.into(),
                display_name: format!("{account_id}@example.com"),
                plan: None,
                org: Some("org-1".into()),
                sign_in: Some(place.into()),
                sign_in_use: format!("QuotaDial reads this account's sign-in from {place}."),
            },
            windows: vec![Window {
                kind: WindowKind::Session,
                label: "Session".into(),
                used_pct: 10.0,
                resets_at: None,
                window_seconds: 18_000,
                scope_model: None,
                source: Source::Api,
                fetched_at: at(fetched_at),
                unconfirmed: false,
                pace: None,
            }],
            extra_usage: None,
            status,
            error: None,
        }
    }

    #[test]
    fn a_subscription_seen_through_two_sign_ins_is_one_row() {
        let report = StatusReport {
            snapshots: vec![
                snapshot("a", "first", Status::AuthRequired, "2099-01-01T00:05:00Z"),
                snapshot("b", "other", Status::Ok, "2099-01-01T00:00:00Z"),
                snapshot("a", "second", Status::Ok, "2099-01-01T00:00:00Z"),
                snapshot("b", "newer", Status::Ok, "2099-01-01T00:03:00Z"),
            ],
            errors: Vec::new(),
            status_pages: Vec::new(),
            costs: Vec::new(),
            left_out: Vec::new(),
            problems: Vec::new(),
            providers: Vec::new(),
        };

        let merged = report.normalized();

        let rows: Vec<(&str, Option<&str>)> = merged
            .snapshots
            .iter()
            .map(|snapshot| {
                (
                    snapshot.account.account_id.as_str(),
                    snapshot.account.sign_in.as_deref(),
                )
            })
            .collect();
        assert_eq!(
            rows,
            [("a", Some("second")), ("b", Some("newer"))],
            "a working snapshot beats one that needs the user; otherwise the newer numbers win"
        );
    }

    #[test]
    fn a_report_names_each_provider_as_the_provider_names_itself() {
        let report = StatusReport {
            providers: vec![
                ProviderName {
                    id: "claude".into(),
                    name: "Claude".into(),
                },
                ProviderName {
                    id: "gemini".into(),
                    name: "Gemini CLI".into(),
                },
            ],
            ..StatusReport::default()
        };

        assert_eq!(
            report.provider_name("gemini"),
            "Gemini CLI",
            "with no account of it in the report"
        );
        assert_eq!(report.provider_name("claude"), "Claude");
        assert_eq!(
            report.provider_name("codex"),
            "codex",
            "a provider the report does not list, as from an older QuotaDial, keeps its id"
        );
    }

    /// Claude's and Gemini CLI's ids and names, as a report lists the providers it asked.
    fn claude_and_gemini() -> Vec<ProviderName> {
        [("claude", "Claude"), ("gemini", "Gemini CLI")]
            .map(|(id, name)| ProviderName {
                id: id.into(),
                name: name.into(),
            })
            .to_vec()
    }

    fn error(provider: &str, message: &str) -> ProviderError {
        ProviderError {
            provider: provider.into(),
            message: message.into(),
        }
    }

    #[test]
    fn a_providers_error_is_shown_under_the_providers_own_name() {
        let report = StatusReport {
            errors: vec![
                error(
                    "gemini",
                    "could not read the sign-in from ~/.gemini/settings.json: it is not a plain \
                     file",
                ),
                error("copilot", "request failed: GitHub did not answer."),
            ],
            providers: claude_and_gemini(),
            ..StatusReport::default()
        };

        assert_eq!(
            report.error_lines().collect::<Vec<_>>(),
            [
                "Gemini CLI: could not read the sign-in from ~/.gemini/settings.json: it is not a \
                 plain file",
                "copilot: request failed: GitHub did not answer.",
            ],
            "a provider the report does not list keeps its id"
        );
    }

    #[test]
    fn an_older_quotadials_error_names_its_provider_once() {
        // Before D-128 every message began with its provider's id.
        let named = StatusReport {
            errors: vec![error(
                "claude",
                "claude: request failed: Could not reach Anthropic.",
            )],
            providers: claude_and_gemini(),
            ..StatusReport::default()
        };
        assert_eq!(
            named.error_lines().collect::<Vec<_>>(),
            ["Claude: request failed: Could not reach Anthropic."]
        );

        // One older than P-020 lists no providers either, so the id shows, once.
        let unnamed = StatusReport {
            providers: Vec::new(),
            ..named
        };
        assert_eq!(
            unnamed.error_lines().collect::<Vec<_>>(),
            ["claude: request failed: Could not reach Anthropic."]
        );

        // Only the provider's own id, as it was written, followed by a colon and a space.
        let others = StatusReport {
            errors: [
                "claudes: a word that starts like the id.",
                "gemini: another provider's id.",
                "claude:no space.",
            ]
            .map(|message| error("claude", message))
            .to_vec(),
            providers: claude_and_gemini(),
            ..StatusReport::default()
        };
        assert_eq!(
            others.error_lines().collect::<Vec<_>>(),
            [
                "Claude: claudes: a word that starts like the id.",
                "Claude: gemini: another provider's id.",
                "Claude: claude:no space.",
            ]
        );
    }

    #[test]
    fn a_reports_message_leaves_its_provider_to_provider() {
        // `provider` says whose it is, and the report's `providers` its name (D-128).
        let failed = Error::Request {
            provider: "codex",
            reason: "Could not reach ChatGPT.".into(),
        };
        assert_eq!(
            ProviderError::new("codex", &failed),
            error("codex", "request failed: Could not reach ChatGPT.")
        );

        // A warning a provider still writes with its id in front, as every one did before.
        assert_eq!(
            ProviderError::warning(
                "claude",
                "claude: could not list the Claude Code items in the macOS Keychain."
            ),
            error(
                "claude",
                "could not list the Claude Code items in the macOS Keychain."
            )
        );
        assert_eq!(
            ProviderError::warning("claude", "claudes: a word that starts like the id.").message,
            "claudes: a word that starts like the id."
        );
    }

    #[test]
    fn the_accounts_left_out_are_counted_under_the_providers_own_name() {
        assert_eq!(
            LeftOut::accounts("gemini", "Gemini CLI", 2).text,
            "2 more Gemini CLI accounts are on this machine. Showing every account is part of \
             QuotaDial Pro: quotadial.com/pro"
        );
        assert_eq!(
            LeftOut::accounts("claude", "Claude", 1).provider.as_deref(),
            Some("claude")
        );
    }

    #[test]
    fn accounts_are_sorted_by_name_within_their_provider() {
        let named = |provider: &str, account_id: &str, display_name: &str| {
            let mut snapshot = snapshot(account_id, "here", Status::Ok, "2099-01-01T00:00:00Z");
            snapshot.account.provider = provider.into();
            snapshot.account.display_name = display_name.into();
            snapshot
        };
        let report = StatusReport {
            snapshots: vec![
                named("claude", "3", "work@example.com"),
                named("claude", "1", "Home@example.com"),
                named("codex", "4", "a@example.com"),
                named("claude", "2", "personal@example.com"),
            ],
            errors: Vec::new(),
            status_pages: Vec::new(),
            costs: Vec::new(),
            left_out: Vec::new(),
            problems: Vec::new(),
            providers: Vec::new(),
        };

        let sorted = report.normalized();

        let rows: Vec<(&str, &str)> = sorted
            .snapshots
            .iter()
            .map(|snapshot| {
                (
                    snapshot.account.provider.as_str(),
                    snapshot.account.display_name.as_str(),
                )
            })
            .collect();
        assert_eq!(
            rows,
            [
                ("claude", "Home@example.com"),
                ("claude", "personal@example.com"),
                ("claude", "work@example.com"),
                ("codex", "a@example.com"),
            ],
            "providers keep their order; names sort without regard to case"
        );
    }
}
