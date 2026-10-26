//! `GET /v1/headroom`: which account has the most room left now. The PRD's cross-provider "what's
//! open" query: each account's headroom is 100% minus its tightest window's share used, and accounts
//! are ranked by it, then by how soon that window resets. Beside them is what the week's local use
//! of each provider would have cost at API prices (D-097).

use std::cmp::Ordering;

use chrono::{DateTime, Utc};
use quotadial_core::{Account, Cost, ProviderError, Status, StatusReport, Window};
use serde::{Deserialize, Serialize};

/// Every account's headroom, the most first, then the accounts that have none to offer now.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub struct HeadroomReport {
    /// Accounts with headroom, the most first; then accounts without numbers and accounts that
    /// cannot be used now.
    pub accounts: Vec<Headroom>,
    /// Providers or accounts that produced no snapshot.
    pub errors: Vec<ProviderError>,
    /// What the week's local use of each provider would have cost at API prices, while the daemon
    /// reads the logs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub costs: Vec<Cost>,
}

/// One account's headroom.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub struct Headroom {
    /// The account.
    pub account: Account,
    /// Whether its numbers are current, and if not, why.
    pub status: Status,
    /// 100 minus the tightest window's share used, from 0 to 100. Left out when the account has no
    /// windows, or cannot be used now: its sign-in needs the user (`auth_required`) or its plan has
    /// no limits to read (`unsupported`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub headroom_pct: Option<f64>,
    /// The window closest to its limit, which sets the headroom: its reset, length and age. For an
    /// account that cannot be used now, the last one known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tightest: Option<Window>,
    /// Why the status is not ok, naming the fix.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Ranks the accounts in `report` by headroom.
#[must_use]
pub fn rank(report: &StatusReport) -> HeadroomReport {
    let mut accounts: Vec<Headroom> = report
        .snapshots
        .iter()
        .map(|snapshot| {
            let tightest = snapshot
                .windows_tightest_first()
                .first()
                .map(|window| (*window).clone());
            // An account waiting for its owner to sign in again keeps its last numbers on show,
            // but whatever room they show cannot be used until then (#34).
            let usable = !matches!(snapshot.status, Status::AuthRequired | Status::Unsupported);
            Headroom {
                account: snapshot.account.clone(),
                status: snapshot.status,
                headroom_pct: tightest
                    .as_ref()
                    .filter(|_| usable)
                    .map(|window| (100.0 - window.used_pct).clamp(0.0, 100.0)),
                tightest,
                error: snapshot.error.clone(),
            }
        })
        .collect();
    accounts.sort_by(|a, b| match (a.headroom_pct, b.headroom_pct) {
        (Some(a_pct), Some(b_pct)) => b_pct
            .total_cmp(&a_pct)
            .then_with(|| resets_at(a).cmp(&resets_at(b))),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    });
    HeadroomReport {
        accounts,
        errors: report.errors.clone(),
        costs: report.costs.clone(),
    }
}

/// When the tightest window resets; unknown counts as last.
fn resets_at(headroom: &Headroom) -> DateTime<Utc> {
    headroom
        .tightest
        .as_ref()
        .and_then(|window| window.resets_at)
        .unwrap_or(DateTime::<Utc>::MAX_UTC)
}

#[cfg(test)]
mod tests {
    use quotadial_core::{Snapshot, Source, WindowKind};

    use super::*;

    fn ts(rfc3339: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(rfc3339)
            .expect("valid timestamp")
            .with_timezone(&Utc)
    }

    fn snapshot(provider: &str, windows: &[(&str, f64, Option<&str>)]) -> Snapshot {
        Snapshot {
            account: Account {
                provider: provider.into(),
                provider_name: format!("The {provider} provider"),
                account_id: format!("{provider}-user"),
                display_name: "user@example.com".into(),
                plan: None,
                org: None,
                sign_in: None,
                sign_in_use: "QuotaDial reads this test's sign-in.".into(),
            },
            windows: windows
                .iter()
                .map(|(label, used_pct, resets_at)| Window {
                    kind: WindowKind::Weekly,
                    label: (*label).into(),
                    used_pct: *used_pct,
                    resets_at: resets_at.map(ts),
                    window_seconds: 604_800,
                    scope_model: None,
                    source: Source::Api,
                    fetched_at: ts("2099-01-01T00:00:00Z"),
                    unconfirmed: false,
                    pace: None,
                })
                .collect(),
            extra_usage: None,
            status: Status::Ok,
            error: None,
        }
    }

    #[test]
    fn the_account_with_the_most_room_comes_first() {
        let report = StatusReport {
            snapshots: vec![
                snapshot(
                    "claude",
                    &[
                        ("Session", 30.0, None),
                        ("Weekly", 80.0, Some("2099-01-03T00:00:00Z")),
                    ],
                ),
                snapshot("codex", &[("Weekly", 55.0, Some("2099-01-05T00:00:00Z"))]),
            ],
            errors: Vec::new(),
            status_pages: Vec::new(),
            costs: Vec::new(),
            left_out: Vec::new(),
            problems: Vec::new(),
            providers: Vec::new(),
        };

        let ranked = rank(&report);

        let order: Vec<(&str, Option<f64>, &str)> = ranked
            .accounts
            .iter()
            .map(|headroom| {
                (
                    headroom.account.provider.as_str(),
                    headroom.headroom_pct,
                    headroom
                        .tightest
                        .as_ref()
                        .map_or("", |window| window.label.as_str()),
                )
            })
            .collect();
        assert_eq!(
            order,
            [
                ("codex", Some(45.0), "Weekly"),
                ("claude", Some(20.0), "Weekly")
            ],
            "Claude's tightest window is its weekly one at 80%"
        );
    }

    #[test]
    fn equal_headroom_goes_to_the_sooner_reset_and_accounts_without_numbers_go_last() {
        let mut signed_out = snapshot("gemini", &[]);
        signed_out.status = Status::AuthRequired;
        signed_out.error = Some("Run `gemini` to sign in again.".into());
        let report = StatusReport {
            snapshots: vec![
                signed_out,
                snapshot("claude", &[("Weekly", 40.0, Some("2099-01-07T00:00:00Z"))]),
                snapshot("codex", &[("Weekly", 40.0, Some("2099-01-02T00:00:00Z"))]),
            ],
            errors: vec![ProviderError {
                provider: "other".into(),
                message: "could not read the sign-in".into(),
            }],
            status_pages: Vec::new(),
            costs: Vec::new(),
            left_out: Vec::new(),
            problems: Vec::new(),
            providers: Vec::new(),
        };

        let ranked = rank(&report);

        let order: Vec<&str> = ranked
            .accounts
            .iter()
            .map(|headroom| headroom.account.provider.as_str())
            .collect();
        assert_eq!(order, ["codex", "claude", "gemini"]);
        let last = &ranked.accounts[2];
        assert_eq!(last.headroom_pct, None);
        assert_eq!(
            last.error.as_deref(),
            Some("Run `gemini` to sign in again.")
        );
        assert_eq!(ranked.errors, report.errors);
    }

    #[test]
    fn an_account_waiting_for_a_sign_in_has_no_headroom_whatever_it_last_showed() {
        let mut expired = snapshot("claude", &[("Weekly", 5.0, Some("2099-01-02T00:00:00Z"))]);
        expired.status = Status::AuthRequired;
        expired.error = Some("Run `CLAUDE_CONFIG_DIR=~/.claude-work claude` once.".into());
        let report = StatusReport {
            snapshots: vec![
                expired,
                snapshot("codex", &[("Weekly", 85.0, Some("2099-01-05T00:00:00Z"))]),
            ],
            errors: Vec::new(),
            status_pages: Vec::new(),
            costs: Vec::new(),
            left_out: Vec::new(),
            problems: Vec::new(),
            providers: Vec::new(),
        };

        let ranked = rank(&report);

        assert_eq!(ranked.accounts[0].account.provider, "codex");
        assert_eq!(ranked.accounts[0].headroom_pct, Some(15.0));
        let waiting = &ranked.accounts[1];
        assert_eq!(waiting.status, Status::AuthRequired);
        assert_eq!(waiting.headroom_pct, None, "its 95% left cannot be used");
        assert_eq!(
            waiting.tightest.as_ref().map(|window| window.used_pct),
            Some(5.0),
            "its last numbers still show"
        );
    }

    #[test]
    fn headroom_stays_between_zero_and_one_hundred() {
        let report = StatusReport {
            snapshots: vec![snapshot("claude", &[("Weekly", 104.0, None)])],
            errors: Vec::new(),
            status_pages: Vec::new(),
            costs: Vec::new(),
            left_out: Vec::new(),
            problems: Vec::new(),
            providers: Vec::new(),
        };

        assert_eq!(rank(&report).accounts[0].headroom_pct, Some(0.0));
    }

    #[test]
    fn the_week_s_costs_sit_beside_the_accounts() {
        let cost = Cost {
            provider: "codex".into(),
            since: ts("2098-12-25T00:00:00Z"),
            until: ts("2099-01-01T00:00:00Z"),
            usd: 312.5,
            requests: 4_000,
            tokens: quotadial_core::Tokens::default(),
            models: Vec::new(),
            prices_updated: "2026-10-23".into(),
            notes: Vec::new(),
        };
        let report = StatusReport {
            snapshots: vec![snapshot("codex", &[("Weekly", 55.0, None)])],
            errors: Vec::new(),
            status_pages: Vec::new(),
            costs: vec![cost.clone()],
            left_out: Vec::new(),
            problems: Vec::new(),
            providers: Vec::new(),
        };

        let ranked = rank(&report);

        assert_eq!(ranked.costs, [cost]);
        let json = serde_json::to_value(rank(&StatusReport::default())).expect("serializes");
        assert!(json.get("costs").is_none(), "left out when there are none");
    }
}
