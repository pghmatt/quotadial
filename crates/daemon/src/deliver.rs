//! Sending told alerts to webhooks and Slack (D-094).
//!
//! A webhook gets only the firing: the provider, the account's display name, the window, its
//! share used and reset, and the words the notification shows. Never a token, and never where a
//! sign-in is kept. A failed send is never described with the webhook's address, which works like a
//! password; only its host is named.

use std::collections::HashSet;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use reqwest::StatusCode;
use reqwest::header::CONTENT_TYPE;
use serde_json::{Value, json};

use crate::{Firing, Webhook, WebhookKind};

/// How long a webhook gets to answer.
const TIMEOUT: Duration = Duration::from_secs(10);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// Sends alerts.
#[derive(Debug, Clone)]
pub struct Deliverer {
    client: reqwest::Client,
    /// The firings sent to a webhook, by firing and webhook key, that the history could not record,
    /// as on a full disk, so they are not sent again while this daemon runs.
    unrecorded: Arc<Mutex<HashSet<(i64, String)>>>,
}

impl Deliverer {
    /// A sender that follows no redirects, so an alert only ever goes where the settings say.
    ///
    /// # Errors
    ///
    /// When the system's TLS cannot be set up.
    pub fn new() -> Result<Self, String> {
        reqwest::Client::builder()
            .timeout(TIMEOUT)
            .connect_timeout(CONNECT_TIMEOUT)
            .redirect(reqwest::redirect::Policy::none())
            .user_agent(concat!("quotadial/", env!("CARGO_PKG_VERSION")))
            .build()
            .map(|client| Self {
                client,
                unrecorded: Arc::default(),
            })
            .map_err(|_| "QuotaDial could not set up sending alerts".to_owned())
    }

    /// Whether `firing` went to the webhook `key` already in this run, though it could not be
    /// recorded.
    pub(crate) fn sent_unrecorded(&self, firing: i64, key: &str) -> bool {
        self.unrecorded
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .contains(&(firing, key.to_owned()))
    }

    /// Remembers that `firing` went to the webhook `key` though the history could not record it.
    pub(crate) fn remember_unrecorded(&self, firing: i64, key: &str) {
        self.unrecorded
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert((firing, key.to_owned()));
    }

    /// Sends `firing` to `webhook`.
    ///
    /// # Errors
    ///
    /// Why it was not sent, naming the webhook's host and never its address.
    pub async fn send(&self, webhook: &Webhook, firing: &Firing) -> Result<(), String> {
        let host = webhook.host();
        let response = self
            .client
            .post(webhook.url())
            .header(CONTENT_TYPE, "application/json")
            .body(payload(firing, webhook.kind).to_string())
            .send()
            .await
            .map_err(|error| {
                let why = if error.is_timeout() {
                    "it did not answer in time"
                } else if error.is_connect() {
                    "it could not be reached"
                } else {
                    "the connection failed"
                };
                format!("not sent to {host}: {why}")
            })?;
        match response.status() {
            status if status.is_success() => Ok(()),
            status if status.is_redirection() => Err(format!(
                "not sent to {host}: it answered with a redirect ({status}), which QuotaDial does \
                 not follow"
            )),
            StatusCode::NOT_FOUND | StatusCode::GONE | StatusCode::FORBIDDEN => Err(format!(
                "not sent to {host}: it answered {}; check the webhook's address",
                response.status()
            )),
            status => Err(format!("not sent to {host}: it answered {status}")),
        }
    }
}

/// What `kind` of webhook is sent about `firing`.
#[must_use]
pub fn payload(firing: &Firing, kind: WebhookKind) -> Value {
    match kind {
        WebhookKind::Json => json!({
            "provider": firing.provider,
            "account": firing.account,
            "window": firing.window,
            "used_pct": firing.used_pct,
            "resets_at": firing.resets_at,
            "title": firing.title,
            "text": firing.body,
            "fired_at": firing.fired_at,
        }),
        WebhookKind::Slack => json!({
            "text": format!("*{}*\n{}", slack(&firing.title), slack(&firing.body)),
        }),
    }
}

/// `text` with the three characters Slack's message format reserves escaped.
fn slack(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};

    use super::*;

    fn firing() -> Firing {
        Firing {
            id: 7,
            rule: 1,
            provider: "claude".into(),
            account_id: "00000000-0000-0000-0000-000000000001".into(),
            org: Some("00000000-0000-0000-0000-000000000002".into()),
            account: "user@example.com".into(),
            window: Some("Weekly".into()),
            used_pct: Some(91.0),
            resets_at: Some(
                DateTime::parse_from_rfc3339("2099-01-03T04:00:00Z")
                    .expect("valid")
                    .with_timezone(&Utc),
            ),
            title: "Claude Weekly passed 90%".into(),
            body: "user@example.com: 91% used, resets in 2 d 4 h.".into(),
            fired_at: DateTime::parse_from_rfc3339("2099-01-01T00:00:00Z")
                .expect("valid")
                .with_timezone(&Utc),
            deliveries: Vec::new(),
        }
    }

    #[test]
    fn a_webhook_gets_the_firing_and_nothing_else() {
        let sent = payload(&firing(), WebhookKind::Json);

        assert_eq!(
            sent,
            json!({
                "provider": "claude",
                "account": "user@example.com",
                "window": "Weekly",
                "used_pct": 91.0,
                "resets_at": "2099-01-03T04:00:00Z",
                "title": "Claude Weekly passed 90%",
                "text": "user@example.com: 91% used, resets in 2 d 4 h.",
                "fired_at": "2099-01-01T00:00:00Z",
            }),
            "no account or organization id, and no sign-in"
        );
    }

    #[test]
    fn slack_gets_the_title_in_bold_with_its_reserved_characters_escaped() {
        let mut firing = firing();
        firing.title = "Claude <Weekly> & more".into();

        assert_eq!(
            payload(&firing, WebhookKind::Slack),
            json!({
                "text": "*Claude &lt;Weekly&gt; &amp; more*\nuser@example.com: 91% used, resets \
                         in 2 d 4 h."
            })
        );
    }
}
