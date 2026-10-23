//! The two notifications M3 sends (D-065): an account needs signing in again, and a window passed
//! 90%. Each is sent once. A sign-in notice comes again only after the account has worked in
//! between, and a 90% notice only after the window resets. What was sent is kept on disk, so a
//! restart does not repeat it.

use std::collections::{BTreeSet, HashSet};
use std::fs;
use std::io;
use std::path::Path;

use chrono::{DateTime, SecondsFormat, Utc};
use quotadial_core::{Status, StatusReport, text};
use serde::{Deserialize, Serialize};

use crate::meter::{HIGH_PCT, provider_name};
use crate::settings::{Settings, account_key, write_private};

/// A notification to show.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notice {
    pub title: String,
    pub body: String,
}

/// What has been sent, per account.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sent(BTreeSet<(String, String)>);

impl Sent {
    /// What was sent before, kept at `path`; nothing when there is no record or it cannot be read.
    #[must_use]
    pub fn load(path: &Path) -> Self {
        fs::read(path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    /// Keeps the record at `path`.
    ///
    /// # Errors
    ///
    /// When the file cannot be written.
    pub fn save(&self, path: &Path) -> io::Result<()> {
        write_private(path, &serde_json::to_vec(self).map_err(io::Error::other)?)
    }

    /// The notices `report` calls for under `settings`, remembering them. What no longer applies to
    /// an account in the report is forgotten, so it can be said again when it next happens.
    /// Accounts missing from this report keep their record, since nothing is known about them.
    pub fn check(
        &mut self,
        report: &StatusReport,
        settings: &Settings,
        now: DateTime<Utc>,
    ) -> Vec<Notice> {
        let mut notices = Vec::new();
        let mut present = HashSet::new();
        let mut still = BTreeSet::new();
        for snapshot in &report.snapshots {
            let account = &snapshot.account;
            let who = account_key(account);
            present.insert(who.clone());
            if !settings.shows(&account.provider) {
                continue;
            }
            let provider = provider_name(&account.provider);
            if snapshot.status == Status::AuthRequired && settings.notify_sign_in {
                let what = "sign-in".to_owned();
                if !self.0.contains(&(who.clone(), what.clone())) {
                    notices.push(Notice {
                        title: format!("{provider} needs you to sign in again"),
                        body: format!(
                            "{}: {}",
                            account.display_name,
                            snapshot.error.as_deref().unwrap_or(
                                "QuotaDial cannot read this account's usage until you sign in."
                            )
                        ),
                    });
                }
                still.insert((who.clone(), what));
            }
            if !settings.notify_high {
                continue;
            }
            for window in &snapshot.windows {
                // A sudden 0% is never high, and an unconfirmed reading waits for the next fetch.
                if window.used_pct < HIGH_PCT || window.unconfirmed {
                    continue;
                }
                let reset = window.resets_at.map_or_else(
                    || "no reset".to_owned(),
                    |at| at.to_rfc3339_opts(SecondsFormat::Secs, true),
                );
                let what = format!("high {} {reset}", window.label);
                if !self.0.contains(&(who.clone(), what.clone())) {
                    notices.push(Notice {
                        title: format!("{provider} {} passed 90%", window.label),
                        body: format!(
                            "{}: {:.0}% used, {}.",
                            account.display_name,
                            window.used_pct.floor(),
                            text::resets(window.resets_at, now)
                        ),
                    });
                }
                still.insert((who.clone(), what));
            }
        }
        self.0.retain(|(who, _)| !present.contains(who));
        self.0.extend(still);
        notices
    }
}

#[cfg(test)]
mod tests {
    use quotadial_core::{Account, Snapshot, Source, Window, WindowKind};

    use super::*;

    fn at(text: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(text)
            .expect("valid timestamp")
            .with_timezone(&Utc)
    }

    fn now() -> DateTime<Utc> {
        at("2099-01-01T00:00:00Z")
    }

    fn account(provider: &str) -> Account {
        Account {
            provider: provider.into(),
            account_id: "user-1".into(),
            display_name: "user@example.com".into(),
            plan: None,
            org: None,
            sign_in: None,
        }
    }

    fn weekly(used_pct: f64, resets_at: &str) -> Snapshot {
        Snapshot {
            account: account("codex"),
            windows: vec![Window {
                kind: WindowKind::Weekly,
                label: "Weekly".into(),
                used_pct,
                resets_at: Some(at(resets_at)),
                window_seconds: 604_800,
                scope_model: None,
                source: Source::Api,
                fetched_at: now(),
                unconfirmed: false,
            }],
            extra_usage: None,
            status: Status::Ok,
            error: None,
        }
    }

    fn report(snapshots: Vec<Snapshot>) -> StatusReport {
        StatusReport {
            snapshots,
            errors: Vec::new(),
        }
    }

    #[test]
    fn a_window_passing_ninety_percent_is_said_once_per_reset() {
        let mut sent = Sent::default();
        let settings = Settings::default();

        assert!(
            sent.check(
                &report(vec![weekly(89.9, "2099-01-03T04:00:00Z")]),
                &settings,
                now()
            )
            .is_empty()
        );
        assert_eq!(
            sent.check(
                &report(vec![weekly(91.4, "2099-01-03T04:00:00Z")]),
                &settings,
                now()
            ),
            [Notice {
                title: "Codex Weekly passed 90%".into(),
                body: "user@example.com: 91% used, resets in 2 d 4 h.".into(),
            }]
        );
        assert!(
            sent.check(
                &report(vec![weekly(95.0, "2099-01-03T04:00:00Z")]),
                &settings,
                now()
            )
            .is_empty(),
            "still the same window"
        );
        assert_eq!(
            sent.check(
                &report(vec![weekly(92.0, "2099-01-10T04:00:00Z")]),
                &settings,
                now()
            )
            .len(),
            1,
            "a new week passing 90% again"
        );
    }

    #[test]
    fn a_sign_in_that_expires_again_is_said_again() {
        let mut sent = Sent::default();
        let settings = Settings::default();
        let expired = || {
            report(vec![Snapshot::auth_required(
                account("claude"),
                "Run `claude` to sign in again.",
            )])
        };

        assert_eq!(
            sent.check(&expired(), &settings, now()),
            [Notice {
                title: "Claude needs you to sign in again".into(),
                body: "user@example.com: Run `claude` to sign in again.".into(),
            }]
        );
        assert!(sent.check(&expired(), &settings, now()).is_empty());

        let mut working = weekly(10.0, "2099-01-03T04:00:00Z");
        working.account = account("claude");
        assert!(
            sent.check(&report(vec![working]), &settings, now())
                .is_empty()
        );
        assert_eq!(
            sent.check(&expired(), &settings, now()).len(),
            1,
            "expired again after working"
        );
    }

    #[test]
    fn switched_off_hidden_unconfirmed_and_absent_say_nothing() {
        let high = || report(vec![weekly(95.0, "2099-01-03T04:00:00Z")]);

        let quiet = Settings {
            notify_high: false,
            ..Settings::default()
        };
        assert!(Sent::default().check(&high(), &quiet, now()).is_empty());

        let hidden = Settings {
            hidden_providers: vec!["codex".into()],
            ..Settings::default()
        };
        assert!(Sent::default().check(&high(), &hidden, now()).is_empty());

        let mut unconfirmed = weekly(95.0, "2099-01-03T04:00:00Z");
        unconfirmed.windows[0].unconfirmed = true;
        assert!(
            Sent::default()
                .check(&report(vec![unconfirmed]), &Settings::default(), now())
                .is_empty()
        );

        // An account missing from one report, as when its provider could not be asked, keeps
        // its record, so it is not said again when it comes back.
        let mut sent = Sent::default();
        assert_eq!(sent.check(&high(), &Settings::default(), now()).len(), 1);
        assert!(
            sent.check(&report(Vec::new()), &Settings::default(), now())
                .is_empty()
        );
        assert!(sent.check(&high(), &Settings::default(), now()).is_empty());
    }

    #[test]
    fn what_was_sent_survives_a_restart() {
        let temp = tempfile::tempdir().expect("temp dir");
        let path = temp.path().join("notified.json");
        let high = report(vec![weekly(95.0, "2099-01-03T04:00:00Z")]);
        let mut sent = Sent::load(&path);
        assert_eq!(sent.check(&high, &Settings::default(), now()).len(), 1);
        sent.save(&path).expect("saves");

        let mut restarted = Sent::load(&path);

        assert!(
            restarted
                .check(&high, &Settings::default(), now())
                .is_empty()
        );
    }
}
