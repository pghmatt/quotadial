//! What QuotaDial reports for every account at one moment: the shape `quotadial status --json`
//! prints and the daemon's local API serves (`docs/CLI.md`).

use serde::{Deserialize, Serialize};

use crate::{Error, Provider, Snapshot, Status, redact_secrets};

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

impl StatusReport {
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
                account_id: account_id.into(),
                display_name: format!("{account_id}@example.com"),
                plan: None,
                org: Some("org-1".into()),
                sign_in: Some(place.into()),
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
