//! The providers' status pages (D-096): status.claude.com for Claude and status.openai.com for
//! Codex, read every 5 minutes while the daemon has an account with that provider.
//!
//! Both pages answer Statuspage's `api/v2/status.json`: an indicator (`none`, `maintenance`,
//! `minor`, `major` or `critical`) and the page's own words. QuotaDial keeps when it first saw the
//! current trouble (`since`), in the history so it survives a restart, so an incident is told once
//! however long it lasts. A page that cannot be read keeps its last reading, with its age.

use std::time::Duration;

use chrono::{DateTime, Utc};
use quotadial_core::{Indicator, StatusPage};
use rusqlite::params;
use serde::Deserialize;

use crate::history::{History, HistoryError};

/// How often the status pages are read.
pub const STATUS_EVERY: Duration = Duration::from_secs(5 * 60);

/// Each provider's status page.
pub const STATUS_PAGES: [(&str, &str); 2] = [
    ("claude", "https://status.claude.com"),
    ("codex", "https://status.openai.com"),
];

/// How long a status page gets to answer.
const TIMEOUT: Duration = Duration::from_secs(10);

/// What `api/v2/status.json` says, as far as QuotaDial reads it.
#[derive(Deserialize)]
struct Answer {
    status: AnswerStatus,
}

#[derive(Deserialize)]
struct AnswerStatus {
    indicator: Indicator,
    #[serde(default)]
    description: String,
}

/// Reads `provider`'s status page at `url` from `body`, an `api/v2/status.json` answer, at
/// `checked_at`. A page still in trouble keeps the time `before` first saw it. `None` when the
/// answer is not one.
#[must_use]
pub fn parse(
    provider: &str,
    url: &str,
    body: &[u8],
    before: Option<&StatusPage>,
    checked_at: DateTime<Utc>,
) -> Option<StatusPage> {
    let answer: Answer = serde_json::from_slice(body).ok()?;
    let indicator = answer.status.indicator;
    let since = indicator.is_incident().then(|| {
        before
            .filter(|before| before.indicator.is_incident())
            .and_then(|before| before.since)
            .unwrap_or(checked_at)
    });
    Some(StatusPage {
        provider: provider.to_owned(),
        indicator,
        description: answer.status.description.trim().to_owned(),
        url: url.to_owned(),
        since,
        checked_at,
    })
}

/// Reads status pages.
#[derive(Debug, Clone)]
pub struct StatusReader {
    client: reqwest::Client,
}

impl StatusReader {
    /// A reader that follows no redirects and asks nothing of the pages but their status.
    ///
    /// # Errors
    ///
    /// When the system's TLS cannot be set up.
    pub fn new() -> Result<Self, String> {
        reqwest::Client::builder()
            .timeout(TIMEOUT)
            .redirect(reqwest::redirect::Policy::none())
            .user_agent(concat!("quotadial/", env!("CARGO_PKG_VERSION")))
            .build()
            .map(|client| Self { client })
            .map_err(|_| "QuotaDial could not set up reading status pages".to_owned())
    }

    /// `provider`'s page at `url` now, keeping the trouble's start from `before`. `None` when the
    /// page cannot be read; its last reading stands.
    pub async fn read(
        &self,
        provider: &str,
        url: &str,
        before: Option<&StatusPage>,
    ) -> Option<StatusPage> {
        let response = self
            .client
            .get(format!("{}/api/v2/status.json", url.trim_end_matches('/')))
            .send()
            .await
            .ok()?;
        if !response.status().is_success() {
            return None;
        }
        let body = response.bytes().await.ok()?;
        parse(provider, url, &body, before, Utc::now())
    }
}

impl History {
    /// The status pages last read, so a restarted daemon keeps when each trouble began.
    ///
    /// # Errors
    ///
    /// When the database cannot be read.
    pub fn status_pages(&self) -> Result<Vec<StatusPage>, HistoryError> {
        let connection = self.lock();
        let mut statement =
            connection.prepare_cached("SELECT page FROM status_pages ORDER BY provider")?;
        let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
        let mut pages = Vec::new();
        for row in rows {
            if let Ok(page) = serde_json::from_str(&row?) {
                pages.push(page);
            }
        }
        Ok(pages)
    }

    /// Keeps `page` as its provider's last reading.
    ///
    /// # Errors
    ///
    /// When the database cannot be written.
    pub fn record_status_page(&self, page: &StatusPage) -> Result<(), HistoryError> {
        let json = serde_json::to_string(page).map_err(|error| {
            HistoryError::new(format!("the status could not be saved: {error}"))
        })?;
        self.lock().execute(
            "INSERT INTO status_pages (provider, page) VALUES (?1, ?2)
             ON CONFLICT (provider) DO UPDATE SET page = excluded.page",
            params![page.provider, json],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(text: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(text)
            .expect("valid timestamp")
            .with_timezone(&Utc)
    }

    fn fixture(name: &str) -> Vec<u8> {
        std::fs::read(format!(
            "{}/../../fixtures/status/{name}",
            env!("CARGO_MANIFEST_DIR")
        ))
        .expect("the fixture")
    }

    #[test]
    fn both_providers_pages_read_as_an_indicator_and_their_words() {
        let now = at("2026-10-23T23:15:00Z");

        let claude = parse(
            "claude",
            STATUS_PAGES[0].1,
            &fixture("claude-status.json"),
            None,
            now,
        )
        .expect("a status");
        let codex = parse(
            "codex",
            STATUS_PAGES[1].1,
            &fixture("openai-status.json"),
            None,
            now,
        )
        .expect("a status");

        assert_eq!(claude.indicator, Indicator::None);
        assert_eq!(claude.description, "All Systems Operational");
        assert_eq!(claude.since, None);
        assert_eq!(codex.indicator, Indicator::Minor);
        assert_eq!(codex.description, "Partial System Degradation");
        assert_eq!(codex.since, Some(now), "first seen now");
        assert_eq!(codex.url, "https://status.openai.com");
        assert_eq!(
            parse("codex", "https://status.openai.com", b"<html>", None, now),
            None
        );
    }

    #[test]
    fn a_trouble_keeps_when_it_began_until_the_page_is_well_again() {
        let minor = br#"{"status": {"indicator": "minor", "description": "Degraded"}}"#;
        let major = br#"{"status": {"indicator": "major", "description": "Outage"}}"#;
        let well =
            br#"{"status": {"indicator": "none", "description": "All Systems Operational"}}"#;
        let maintenance = br#"{"status": {"indicator": "maintenance", "description": "Planned"}}"#;
        let url = "https://status.openai.com";

        let first = parse("codex", url, minor, None, at("2099-01-01T00:00:00Z")).expect("parses");
        let worse = parse(
            "codex",
            url,
            major,
            Some(&first),
            at("2099-01-01T00:05:00Z"),
        )
        .expect("parses");
        assert_eq!(
            worse.since,
            Some(at("2099-01-01T00:00:00Z")),
            "the same trouble"
        );

        let over =
            parse("codex", url, well, Some(&worse), at("2099-01-01T00:10:00Z")).expect("parses");
        assert_eq!(over.since, None);
        let again =
            parse("codex", url, minor, Some(&over), at("2099-01-01T00:15:00Z")).expect("parses");
        assert_eq!(
            again.since,
            Some(at("2099-01-01T00:15:00Z")),
            "a new trouble"
        );

        let planned =
            parse("codex", url, maintenance, None, at("2099-01-01T00:20:00Z")).expect("parses");
        assert_eq!(planned.since, None, "maintenance is no incident");
    }

    #[test]
    fn the_last_reading_survives_a_restart() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("history.db");
        let page = parse(
            "codex",
            STATUS_PAGES[1].1,
            &fixture("openai-status.json"),
            None,
            at("2099-01-01T00:00:00Z"),
        )
        .expect("a status");
        History::open(&path)
            .expect("opens")
            .record_status_page(&page)
            .expect("records");

        let reopened = History::open(&path).expect("reopens");

        assert_eq!(reopened.status_pages().expect("reads"), [page]);
    }
}
