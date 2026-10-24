//! What became of each told alert at each webhook (schema 4, D-094).
//!
//! The daemon sends each told firing to every webhook in the settings. One that is not taken, as
//! when the network is down just after the machine wakes, is tried again a minute later, then after
//! 2, 4 and 8 minutes and every 15 after that, for as long as an hour after it fired. A webhook is
//! kept as the start of its address's SHA-256 and its host, never the address itself.

use std::sync::Arc;

use chrono::{DateTime, TimeDelta, Utc};
use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};

use crate::alerts::{firing, parse_time};
use crate::history::{History, HistoryError, timestamp};
use crate::{Deliverer, Firing, Webhook};

/// How long a told firing keeps being sent to a webhook that has not taken it.
pub const DELIVER_WITHIN: TimeDelta = TimeDelta::hours(1);

/// Where a firing was sent, or why not.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Delivery {
    /// The webhook's host.
    pub to: String,
    /// Whether the webhook took it.
    pub sent: bool,
    /// Why it was not sent, the last time it was tried.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl History {
    /// The told firings of the hour before `now` that a webhook in `webhooks` has not taken, and
    /// that are due another try, with that webhook's key.
    ///
    /// # Errors
    ///
    /// When the database cannot be read.
    pub fn undelivered(
        &self,
        webhooks: &[Webhook],
        now: DateTime<Utc>,
    ) -> Result<Vec<(Firing, String)>, HistoryError> {
        let firings: Vec<Firing> = {
            let connection = self.lock();
            let mut statement = connection.prepare_cached(
                "SELECT id, rule_id, provider, org, account_id, display_name, label, used_pct,
                        resets_at, title, body, fired_at
                 FROM firings WHERE quiet = 0 AND fired_at >= ?1 ORDER BY id",
            )?;
            let rows = statement.query_map(params![timestamp(now - DELIVER_WITHIN)], firing)?;
            rows.collect::<Result<_, _>>()?
        };
        let mut due = Vec::new();
        for firing in firings {
            for webhook in webhooks {
                let key = webhook.key();
                let tried: Option<(bool, i64, String)> = self
                    .lock()
                    .query_row(
                        "SELECT sent, attempts, last_at FROM deliveries
                         WHERE firing_id = ?1 AND webhook = ?2",
                        params![firing.id, key],
                        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                    )
                    .optional()?;
                let ready = match tried {
                    None => true,
                    Some((true, _, _)) => false,
                    Some((false, attempts, last_at)) => parse_time(&last_at)
                        .is_none_or(|last_at| now - last_at >= retry_after(attempts)),
                };
                if ready {
                    due.push((firing.clone(), key));
                }
            }
        }
        Ok(due)
    }

    /// Records that the firing `firing` was sent to the webhook `key` at `host` at `now`, or why
    /// not.
    ///
    /// # Errors
    ///
    /// When the database cannot be written.
    pub fn record_delivery(
        &self,
        firing: i64,
        key: &str,
        host: &str,
        sent: Result<(), &str>,
        now: DateTime<Utc>,
    ) -> Result<(), HistoryError> {
        self.lock().execute(
            "INSERT INTO deliveries (firing_id, webhook, host, sent, error, attempts, last_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6)
             ON CONFLICT (firing_id, webhook) DO UPDATE SET
                 host = excluded.host, sent = excluded.sent, error = excluded.error,
                 attempts = attempts + 1, last_at = excluded.last_at",
            params![firing, key, host, sent.is_ok(), sent.err(), timestamp(now)],
        )?;
        Ok(())
    }

    /// Where the firing `firing` was sent, or why not.
    pub(crate) fn deliveries_of(&self, firing: i64) -> Result<Vec<Delivery>, HistoryError> {
        let connection = self.lock();
        let mut statement = connection.prepare_cached(
            "SELECT host, sent, error FROM deliveries WHERE firing_id = ?1 ORDER BY host",
        )?;
        let rows = statement.query_map(params![firing], |row| {
            Ok(Delivery {
                to: row.get(0)?,
                sent: row.get(1)?,
                error: row.get(2)?,
            })
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }
}

/// Sends what is due to `webhooks` at `now` and records how each went (D-094). Sending alerts
/// beyond the app's notifications is a Pro feature (D-090), which the poller asks before it sends.
pub async fn deliver_due(
    history: &Arc<History>,
    deliverer: &Deliverer,
    webhooks: &[Webhook],
    now: DateTime<Utc>,
) {
    if webhooks.is_empty() {
        return;
    }
    let (found, wanted) = (Arc::clone(history), webhooks.to_vec());
    let Ok(Ok(due)) = tokio::task::spawn_blocking(move || found.undelivered(&wanted, now)).await
    else {
        return;
    };
    for (firing, key) in due {
        let Some(webhook) = webhooks.iter().find(|webhook| webhook.key() == key) else {
            continue;
        };
        let sent = deliverer.send(webhook, &firing).await;
        let (record, host) = (Arc::clone(history), webhook.host().to_owned());
        // A result that cannot be recorded is sent again later; a webhook may get it twice.
        let _ = tokio::task::spawn_blocking(move || {
            record.record_delivery(
                firing.id,
                &key,
                &host,
                sent.as_ref().map(|_| ()).map_err(String::as_str),
                now,
            )
        })
        .await;
    }
}

/// How long after its `attempts`-th failed try a delivery is tried again.
fn retry_after(attempts: i64) -> TimeDelta {
    TimeDelta::minutes((1_i64 << (attempts - 1).clamp(0, 4)).min(15))
}

#[cfg(test)]
mod tests {
    use quotadial_core::{Account, Snapshot, Source, Status, StatusReport, Window, WindowKind};

    use super::*;
    use crate::WebhookKind;
    use crate::test_support::receiver;

    /// A history whose 90% rule has just told one firing.
    fn told() -> (tempfile::TempDir, Arc<History>) {
        let dir = tempfile::tempdir().expect("temp dir");
        let history = Arc::new(History::open(&dir.path().join("history.db")).expect("opens"));
        let report = StatusReport {
            snapshots: vec![Snapshot {
                account: Account {
                    provider: "claude".into(),
                    account_id: "user-1".into(),
                    display_name: "user@example.com".into(),
                    plan: None,
                    org: Some("org-1".into()),
                    sign_in: Some("the macOS Keychain item \"Claude Code-credentials\"".into()),
                },
                windows: vec![Window {
                    kind: WindowKind::Weekly,
                    label: "Weekly".into(),
                    used_pct: 95.0,
                    resets_at: None,
                    window_seconds: 604_800,
                    scope_model: None,
                    source: Source::Api,
                    fetched_at: Utc::now(),
                    unconfirmed: false,
                    pace: None,
                }],
                extra_usage: None,
                status: Status::Ok,
                error: None,
            }],
            errors: Vec::new(),
            status_pages: Vec::new(),
            costs: Vec::new(),
            left_out: Vec::new(),
        };
        assert_eq!(
            history
                .check(&report, Utc::now(), quotadial_core::Tier::Pro)
                .expect("checks")
                .len(),
            1
        );
        (dir, history)
    }

    #[tokio::test]
    async fn a_told_alert_is_sent_once_with_only_the_firing() {
        let (_dir, history) = told();
        let (address, bodies) = receiver("200 OK").await;
        let webhook = Webhook::new(&address, WebhookKind::Json).expect("a webhook");
        let deliverer = Deliverer::new().expect("sets up");

        deliver_due(
            &history,
            &deliverer,
            std::slice::from_ref(&webhook),
            Utc::now(),
        )
        .await;
        deliver_due(
            &history,
            &deliverer,
            std::slice::from_ref(&webhook),
            Utc::now(),
        )
        .await;

        let bodies = bodies.lock().expect("lock").clone();
        assert_eq!(bodies.len(), 1, "taken once, so sent once");
        let sent: serde_json::Value = serde_json::from_str(&bodies[0]).expect("JSON");
        assert_eq!(sent["title"], "Claude Weekly passed 90%");
        assert_eq!(sent["account"], "user@example.com");
        for kept_back in ["user-1", "org-1", "Keychain"] {
            assert!(
                !bodies[0].contains(kept_back),
                "{kept_back} in {}",
                bodies[0]
            );
        }
        let listed = history
            .firings(0, Utc::now() - TimeDelta::hours(1))
            .expect("reads");
        assert_eq!(
            listed[0].deliveries,
            [Delivery {
                to: "127.0.0.1".into(),
                sent: true,
                error: None
            }]
        );
    }

    #[tokio::test]
    async fn a_webhook_that_fails_is_tried_again_later_and_its_address_never_shown() {
        let (_dir, history) = told();
        let (address, bodies) = receiver("404 Not Found").await;
        let webhook = Webhook::new(&address, WebhookKind::Slack).expect("a webhook");
        let deliverer = Deliverer::new().expect("sets up");
        let now = Utc::now();

        deliver_due(&history, &deliverer, std::slice::from_ref(&webhook), now).await;
        let listed = history
            .firings(0, now - TimeDelta::hours(1))
            .expect("reads");
        let error = listed[0].deliveries[0].error.clone().expect("an error");
        assert_eq!(
            error,
            "not sent to 127.0.0.1: it answered 404 Not Found; check the webhook's address"
        );
        assert!(!error.contains("secret-path"));

        deliver_due(&history, &deliverer, std::slice::from_ref(&webhook), now).await;
        assert_eq!(bodies.lock().expect("lock").len(), 1, "not again at once");
        deliver_due(
            &history,
            &deliverer,
            std::slice::from_ref(&webhook),
            now + TimeDelta::minutes(1),
        )
        .await;
        assert_eq!(
            bodies.lock().expect("lock").len(),
            2,
            "again a minute later"
        );
        deliver_due(
            &history,
            &deliverer,
            std::slice::from_ref(&webhook),
            now + DELIVER_WITHIN + TimeDelta::minutes(1),
        )
        .await;
        assert_eq!(
            bodies.lock().expect("lock").len(),
            2,
            "and not after an hour"
        );
    }

    #[tokio::test]
    async fn a_webhook_that_cannot_be_reached_says_so_by_its_host() {
        let (_dir, history) = told();
        let closed = std::net::TcpListener::bind("127.0.0.1:0").expect("binds");
        let address = format!(
            "http://{}/hooks/secret-path",
            closed.local_addr().expect("address")
        );
        drop(closed);
        let webhook = Webhook::new(&address, WebhookKind::Json).expect("a webhook");

        deliver_due(
            &history,
            &Deliverer::new().expect("sets up"),
            &[webhook],
            Utc::now(),
        )
        .await;

        let listed = history
            .firings(0, Utc::now() - TimeDelta::hours(1))
            .expect("reads");
        assert_eq!(
            listed[0].deliveries[0].error.as_deref(),
            Some("not sent to 127.0.0.1: it could not be reached")
        );
    }

    #[test]
    fn a_failed_delivery_waits_longer_each_time_up_to_a_quarter_hour() {
        let waits: Vec<i64> = (1..=6)
            .map(|attempts| retry_after(attempts).num_minutes())
            .collect();

        assert_eq!(waits, [1, 2, 4, 8, 15, 15]);
    }
}
