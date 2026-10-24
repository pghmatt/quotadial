//! Telling the alert rules' firings as notifications (D-093). The daemon checks the rules and
//! records each firing once in its history; the app tells the firings it has not told yet, and
//! keeps on disk the newest one it told, so a restart neither repeats nor loses one. A firing the
//! app first sees long after it happened, as when a standalone daemon ran while the app was
//! closed, stays in the history without a notification.

use std::fs;
use std::io;
use std::path::Path;

use chrono::TimeDelta;
use quotadial_daemon::Firing;
use serde::{Deserialize, Serialize};

use crate::settings::{Settings, key, write_private};

/// A firing older than this when the app first sees it is not told.
pub const TELL_WITHIN: TimeDelta = TimeDelta::minutes(15);

/// A notification to show.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notice {
    pub title: String,
    pub body: String,
}

/// The newest firing told, kept on disk.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Told {
    /// The newest firing's id.
    pub last: i64,
}

impl Told {
    /// What was told before, kept at `path`, if it can be read.
    #[must_use]
    pub fn load(path: &Path) -> Option<Self> {
        serde_json::from_slice(&fs::read(path).ok()?).ok()
    }

    /// Keeps the record at `path`.
    ///
    /// # Errors
    ///
    /// When the file cannot be written.
    pub fn save(self, path: &Path) -> io::Result<()> {
        write_private(path, &serde_json::to_vec(&self).map_err(io::Error::other)?)
    }

    /// The notices for `firings`, newer than the last told, under `settings`, and the record with
    /// them told. A hidden provider's firings are passed over without a notice, and an account the
    /// user named is called by its name.
    #[must_use]
    pub fn tell(self, firings: &[Firing], settings: &Settings) -> (Vec<Notice>, Self) {
        let mut last = self.last;
        let mut notices = Vec::new();
        for firing in firings.iter().filter(|firing| firing.id > self.last) {
            last = last.max(firing.id);
            if !settings.shows(&firing.provider) {
                continue;
            }
            let alias = settings.aliases.get(&key(
                &firing.provider,
                &firing.account_id,
                firing.org.as_deref(),
            ));
            let body = match alias {
                Some(alias) => firing
                    .body
                    .strip_prefix(&format!("{}:", firing.account))
                    .map_or_else(|| firing.body.clone(), |rest| format!("{alias}:{rest}")),
                None => firing.body.clone(),
            };
            notices.push(Notice {
                title: firing.title.clone(),
                body,
            });
        }
        (notices, Self { last })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use chrono::{DateTime, Utc};

    use super::*;

    fn firing(id: i64, provider: &str) -> Firing {
        Firing {
            id,
            rule: 1,
            provider: provider.into(),
            account_id: "user-1".into(),
            org: None,
            account: "user@example.com".into(),
            window: Some("Weekly".into()),
            used_pct: Some(91.0),
            resets_at: None,
            title: format!("{provider} Weekly passed 90%"),
            body: "user@example.com: 91% used, no reset time reported.".into(),
            fired_at: DateTime::<Utc>::MIN_UTC,
            deliveries: Vec::new(),
        }
    }

    #[test]
    fn each_firing_is_told_once_in_the_users_words_for_the_account() {
        let settings = Settings {
            aliases: BTreeMap::from([("claude user-1 -".to_owned(), "Work".to_owned())]),
            ..Settings::default()
        };
        let firings = [firing(4, "claude"), firing(5, "codex")];

        let (notices, told) = Told { last: 3 }.tell(&firings, &settings);

        assert_eq!(
            notices,
            [
                Notice {
                    title: "claude Weekly passed 90%".into(),
                    body: "Work: 91% used, no reset time reported.".into(),
                },
                Notice {
                    title: "codex Weekly passed 90%".into(),
                    body: "user@example.com: 91% used, no reset time reported.".into(),
                },
            ]
        );
        assert_eq!(told, Told { last: 5 });
        assert!(told.tell(&firings, &settings).0.is_empty(), "told already");
    }

    #[test]
    fn a_hidden_providers_firings_are_passed_over_without_a_notice() {
        let settings = Settings {
            hidden_providers: vec!["codex".into()],
            ..Settings::default()
        };

        let (notices, told) = Told::default().tell(&[firing(7, "codex")], &settings);

        assert!(notices.is_empty());
        assert_eq!(told, Told { last: 7 }, "and not told later");
    }

    #[test]
    fn what_was_told_survives_a_restart() {
        let temp = tempfile::tempdir().expect("temp dir");
        let path = temp.path().join("alerts.json");
        assert_eq!(Told::load(&path), None, "nothing told yet");

        Told { last: 9 }.save(&path).expect("saves");

        assert_eq!(Told::load(&path), Some(Told { last: 9 }));
    }
}
