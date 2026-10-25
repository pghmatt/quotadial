//! Updates (D-103). Once a day, while Settings' "Check for updates" is on, the app asks GitHub for
//! the `latest.json` of QuotaDial's newest full release; a pre-release or a draft never reaches it.
//! A newer version is offered in the popover and in Help, and told once as a notification. It is
//! downloaded and installed only when the person chooses Update. Tauri's updater checks the
//! download against the public key built into the app and against the version its signature names
//! (`requireSignedVersion`), so neither a changed file nor an older release passed off as newer is
//! installed. The app then stops its daemon and starts again as the new version; on Windows the
//! installer closes the app and opens the new one.
//!
//! `docs/PRIVACY.md` says what the check sends, and `docs/RELEASE.md` how a release is signed for
//! it.

use std::fs;
use std::io;
use std::path::Path;
use std::sync::PoisonError;
use std::time::Duration;

use chrono::{DateTime, TimeDelta, Utc};
use quotadial_core::text;
use quotadial_daemon::write_private;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt as _;
use tauri_plugin_updater::{Error, Update, UpdaterExt as _};

use crate::{AfterStop, AppState};

/// How long after the app starts it first looks, so a login is not slowed and the network is up.
pub const FIRST_CHECK_AFTER: Duration = Duration::from_secs(2 * 60);
/// How often the app wakes to see whether a check is due. A check itself is at most daily.
pub const TICK: Duration = Duration::from_secs(15 * 60);
/// How often the app asks.
pub const CHECK_EVERY: TimeDelta = TimeDelta::hours(24);
/// How soon the app asks again after a check that GitHub did not answer.
pub const RETRY_AFTER: TimeDelta = TimeDelta::hours(1);

/// What the app keeps between runs, in `updates.json` in its config folder.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Record {
    /// When the app last asked, answered or not.
    pub checked_at: Option<DateTime<Utc>>,
    /// Whether that ask went unanswered, so the next one comes sooner.
    pub unanswered: bool,
    /// The newer version the last answer offered, so the offer is back soon after a restart.
    pub offered: Option<String>,
    /// The newest version told as a notification, so each one is told once.
    pub told: Option<String>,
}

impl Record {
    /// The record kept at `path`, or a new one when there is none or it cannot be read.
    #[must_use]
    pub fn load(path: &Path) -> Self {
        fs::read(path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    /// Keeps the record at `path`, privately.
    ///
    /// # Errors
    ///
    /// When the folder or the file cannot be written.
    pub fn save(&self, path: &Path) -> io::Result<()> {
        write_private(
            path,
            &serde_json::to_vec_pretty(self).map_err(io::Error::other)?,
        )
    }

    /// Whether the daily check is due at `now`: a day after the last answer, an hour after an ask
    /// that went unanswered, and at once when the clock has gone back past the last ask.
    #[must_use]
    pub fn due(&self, now: DateTime<Utc>) -> bool {
        match self.checked_at {
            None => true,
            Some(at) if at > now => true,
            Some(at) => {
                now - at
                    >= if self.unanswered {
                        RETRY_AFTER
                    } else {
                        CHECK_EVERY
                    }
            }
        }
    }

    /// Whether the last answer offered a version other than `running`, which a restart has not
    /// installed, so the first check after starting comes without waiting a day.
    #[must_use]
    pub fn offers_other_than(&self, running: &str) -> bool {
        self.offered
            .as_deref()
            .is_some_and(|offered| offered != running)
    }
}

/// Who asked for a check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trigger {
    /// The daily check, which tells a newer version as a notification once.
    Daily,
    /// The person, who is looking at the answer.
    ByHand,
}

/// Where updates stand in this run of the app.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stage {
    /// Nothing asked since the app started.
    Idle,
    Checking,
    /// This copy is the newest version.
    Current,
    /// A newer version this copy can install, with what changed in it.
    Newer {
        version: String,
        notes: Vec<String>,
    },
    /// A newer version that this kind of install cannot take by itself.
    NoUpdateHere,
    /// GitHub could not be asked, or did not answer.
    Unanswered(Trigger),
    Downloading {
        version: String,
        /// How far the download has got, once its size is known.
        percent: Option<u8>,
    },
    Installing {
        version: String,
    },
    /// An install that did not finish, and why, naming the fix.
    Failed {
        version: String,
        problem: String,
    },
}

/// What the app knows about updates.
pub struct Updates {
    pub record: Record,
    pub stage: Stage,
    /// The update the last check offered, which Update installs.
    pub offer: Option<Update>,
}

/// A notification to show.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notice {
    pub title: String,
    pub body: String,
}

/// Updates as the window shows them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UpdateView {
    /// One sentence saying where updates stand, such as "QuotaDial 0.2.0 is the newest version.
    /// Checked 3 min ago."
    pub text: String,
    /// The newer version on offer, while there is one, which Update installs.
    pub available: Option<String>,
    /// What changed in it, a line each, from its release's notes.
    pub notes: Vec<String>,
    /// Whether a check, a download or an install is under way, when the buttons wait.
    pub busy: bool,
    /// How far a download has got, from 0 to 100, while one runs and its size is known.
    pub progress: Option<u8>,
}

impl Updates {
    #[must_use]
    pub fn new(record: Record) -> Self {
        Self {
            record,
            stage: Stage::Idle,
            offer: None,
        }
    }

    /// Whether a check, a download or an install is under way.
    #[must_use]
    pub fn busy(&self) -> bool {
        matches!(
            self.stage,
            Stage::Checking | Stage::Downloading { .. } | Stage::Installing { .. }
        )
    }

    /// Keeps what a check at `now` found, and returns the notification to show, if any: the daily
    /// check tells each newer version once.
    pub fn checked(
        &mut self,
        found: Stage,
        now: DateTime<Utc>,
        trigger: Trigger,
    ) -> Option<Notice> {
        self.record.checked_at = Some(now);
        self.record.unanswered = matches!(found, Stage::Unanswered(_));
        if !self.record.unanswered {
            self.record.offered = match &found {
                Stage::Newer { version, .. } => Some(version.clone()),
                _ => None,
            };
        }
        let notice = match &found {
            Stage::Newer { version, .. }
                if trigger == Trigger::Daily && self.record.told.as_ref() != Some(version) =>
            {
                self.record.told = Some(version.clone());
                Some(Notice {
                    title: format!("QuotaDial {version} is available"),
                    body: "Open QuotaDial and choose Update to install it.".to_owned(),
                })
            }
            _ => None,
        };
        self.stage = found;
        notice
    }

    /// Updates as the window shows them, for a copy that is `running`, with the daily check
    /// `automatic` or not, at `now`.
    #[must_use]
    pub fn view(&self, running: &str, automatic: bool, now: DateTime<Utc>) -> UpdateView {
        let mut view = UpdateView {
            text: String::new(),
            available: None,
            notes: Vec::new(),
            busy: self.busy(),
            progress: None,
        };
        view.text = match &self.stage {
            Stage::Idle => match (automatic, self.record.checked_at) {
                (false, _) => format!("QuotaDial {running}. The daily check for updates is off."),
                (true, None) => format!("QuotaDial {running}. It checks for updates once a day."),
                (true, Some(_)) if self.record.unanswered => format!(
                    "QuotaDial {running}. The last check for updates could not reach GitHub, so \
                     it tries again within the hour."
                ),
                (true, Some(at)) => format!(
                    "QuotaDial {running}. Last checked for updates {}.",
                    text::ago(at, now)
                ),
            },
            Stage::Checking => "Checking for a newer QuotaDial…".to_owned(),
            Stage::Current => format!(
                "QuotaDial {running} is the newest version. Checked {}.",
                self.record
                    .checked_at
                    .map_or_else(|| "just now".to_owned(), |at| text::ago(at, now))
            ),
            Stage::Newer { version, notes } => {
                view.available = Some(version.clone());
                view.notes.clone_from(notes);
                format!("QuotaDial {version} is available. This copy is {running}.")
            }
            Stage::NoUpdateHere => "A newer QuotaDial is out, but this copy cannot install it \
                                    by itself. Download it from quotadial.com."
                .to_owned(),
            Stage::Unanswered(Trigger::ByHand) => "QuotaDial could not reach GitHub to check for \
                                                   updates. Check the connection, then try again."
                .to_owned(),
            Stage::Unanswered(Trigger::Daily) => format!(
                "QuotaDial {running}. The last check for updates could not reach GitHub, so it \
                 tries again within the hour."
            ),
            Stage::Downloading { version, percent } => {
                view.available = Some(version.clone());
                view.progress = *percent;
                match percent {
                    Some(percent) => format!("Downloading QuotaDial {version}… {percent}%"),
                    None => format!("Downloading QuotaDial {version}…"),
                }
            }
            Stage::Installing { version } => {
                view.available = Some(version.clone());
                format!("Installing QuotaDial {version}. It opens again when it is done.")
            }
            Stage::Failed { version, problem } => {
                view.available = Some(version.clone());
                problem.clone()
            }
        };
        view
    }
}

/// What a check's answer means. An endpoint with no `latest.json`, as before the first signed
/// release, has nothing newer to offer.
#[must_use]
pub fn found(answer: Result<Option<(&str, Option<&str>)>, &Error>) -> Stage {
    match answer {
        Ok(Some((version, notes))) => Stage::Newer {
            version: version.to_owned(),
            notes: notes.map(lines).unwrap_or_default(),
        },
        Ok(None) | Err(Error::ReleaseNotFound) => Stage::Current,
        Err(Error::TargetNotFound(_) | Error::TargetsNotFound(_)) => Stage::NoUpdateHere,
        Err(_) => Stage::Unanswered(Trigger::ByHand),
    }
}

/// A release's notes, which are its section of `CHANGELOG.md`, as plain lines, one per change.
#[must_use]
pub fn lines(notes: &str) -> Vec<String> {
    notes
        .lines()
        .filter_map(|line| line.trim_start().strip_prefix("- "))
        .map(crate::help::plain)
        .collect()
}

/// Why installing `version` failed, in one sentence that names the fix. `home` is written `~`.
#[must_use]
pub fn problem(error: &Error, version: &str, home: Option<&Path>) -> String {
    match error {
        Error::Minisign(_)
        | Error::SignedVersionMismatch { .. }
        | Error::MissingSignedVersion
        | Error::Base64(_)
        | Error::SignatureUtf8(_) => format!(
            "QuotaDial {version} did not match QuotaDial's signature, so it was not installed. If \
             this happens again, write to support@quotadial.com."
        ),
        Error::Reqwest(_) | Error::Network(_) => format!(
            "The download of QuotaDial {version} stopped. Check the connection, then choose \
             Update again."
        ),
        Error::AuthenticationFailed => format!(
            "QuotaDial {version} needs your permission to install. Choose Update again and allow \
             it."
        ),
        Error::Io(error) if error.kind() == io::ErrorKind::PermissionDenied => format!(
            "QuotaDial could not replace itself without your permission. Choose Update again and \
             allow it, or download {version} from quotadial.com."
        ),
        Error::PreviousAppNotRestored(kept) => format!(
            "QuotaDial {version} could not be installed, and the copy you had is now at {}. Move \
             it back, or download {version} from quotadial.com.",
            text::tilde(kept, home)
        ),
        _ => format!(
            "QuotaDial {version} could not be installed here. Download it from quotadial.com and \
             install it over this copy."
        ),
    }
}

/// Checks once a day while the setting is on, from a little after the app starts until it quits.
/// A build from source never offers to replace itself.
pub async fn schedule(app: AppHandle) {
    if cfg!(debug_assertions) {
        return;
    }
    tokio::time::sleep(FIRST_CHECK_AFTER).await;
    let running = app.package_info().version.to_string();
    let mut first = true;
    loop {
        let due = {
            let state = app.state::<AppState>();
            let on = lock(&state.settings).check_updates;
            let updates = lock(&state.updates);
            on && !updates.busy()
                && (updates.record.due(Utc::now())
                    || (first && updates.record.offers_other_than(&running)))
        };
        first = false;
        if due {
            check(&app, Trigger::Daily).await;
        }
        tokio::time::sleep(TICK).await;
    }
}

/// Asks GitHub whether there is a newer version, keeps the answer and shows it.
pub async fn check(app: &AppHandle, trigger: Trigger) {
    {
        let state = app.state::<AppState>();
        let mut updates = lock(&state.updates);
        if updates.busy() {
            return;
        }
        updates.stage = Stage::Checking;
    }
    emit(app);
    // On Windows the installer closes the app without the exit an app usually gets, so the tray
    // icon is taken down first rather than left behind until the pointer passes over it.
    let closing = app.clone();
    let answer = match app
        .updater_builder()
        .on_before_exit(move || closing.cleanup_before_exit())
        .build()
    {
        Ok(updater) => updater.check().await,
        Err(error) => Err(error),
    };
    let notice = {
        let state = app.state::<AppState>();
        let mut updates = lock(&state.updates);
        let mut stage = found(answer.as_ref().map(|offer| {
            offer
                .as_ref()
                .map(|update| (update.version.as_str(), update.body.as_deref()))
        }));
        if stage == Stage::Unanswered(Trigger::ByHand) {
            stage = Stage::Unanswered(trigger);
        }
        let notice = updates.checked(stage, Utc::now(), trigger);
        updates.offer = answer.ok().flatten();
        if let Some(files) = &state.files {
            // A record that cannot be kept only means the next start checks sooner.
            let _ = updates.record.save(&files.updates);
        }
        notice
    };
    if let Some(notice) = notice {
        // Best effort: the offer is in the popover either way.
        let _ = app
            .notification()
            .builder()
            .title(notice.title)
            .body(notice.body)
            .show();
    }
    emit(app);
}

/// Downloads the update on offer, which the updater checks against QuotaDial's signature, installs
/// it, and starts the app again as the new version.
pub async fn install(app: &AppHandle) {
    let (update, version) = {
        let state = app.state::<AppState>();
        let mut updates = lock(&state.updates);
        let Some(update) = updates.offer.clone() else {
            return;
        };
        if updates.busy() {
            return;
        }
        let version = update.version.clone();
        updates.stage = Stage::Downloading {
            version: version.clone(),
            percent: None,
        };
        (update, version)
    };
    emit(app);
    let mut received: u64 = 0;
    let mut shown = None;
    let downloaded = update
        .download(
            |chunk, size| {
                received += chunk as u64;
                let percent = size
                    .filter(|size| *size > 0)
                    .map(|size| u8::try_from((received * 100 / size).min(100)).unwrap_or(100));
                if percent != shown {
                    shown = percent;
                    set_stage(
                        app,
                        Stage::Downloading {
                            version: version.clone(),
                            percent,
                        },
                    );
                }
            },
            || {},
        )
        .await;
    let bytes = match downloaded {
        Ok(bytes) => bytes,
        Err(error) => {
            fail(app, &version, &error);
            return;
        }
    };
    set_stage(
        app,
        Stage::Installing {
            version: version.clone(),
        },
    );
    // Windows' installer replaces the app and closes it, so the daemon stops before it runs.
    if cfg!(windows) {
        crate::stop_daemon(app, AfterStop::Stay).await;
    }
    match update.install(&bytes) {
        Ok(()) => crate::stop_daemon(app, AfterStop::Restart).await,
        Err(error) => {
            let problem = fail(app, &version, &error);
            if cfg!(windows) {
                // The daemon stopped for the installer; starting again brings it back, and the
                // notification keeps what went wrong.
                let _ = app
                    .notification()
                    .builder()
                    .title("QuotaDial")
                    .body(problem)
                    .show();
                app.request_restart();
            }
        }
    }
}

/// Updates as the window shows them now.
#[must_use]
pub fn view(app: &AppHandle) -> UpdateView {
    let state = app.state::<AppState>();
    let automatic = lock(&state.settings).check_updates;
    lock(&state.updates).view(
        &app.package_info().version.to_string(),
        automatic,
        Utc::now(),
    )
}

/// Tells the window where updates stand.
pub fn emit(app: &AppHandle) {
    let _ = app.emit("update", view(app));
}

fn set_stage(app: &AppHandle, stage: Stage) {
    lock(&app.state::<AppState>().updates).stage = stage;
    emit(app);
}

/// Shows why installing `version` failed, and returns it.
fn fail(app: &AppHandle, version: &str, error: &Error) -> String {
    let problem = problem(error, version, app.path().home_dir().ok().as_deref());
    set_stage(
        app,
        Stage::Failed {
            version: version.to_owned(),
            problem: problem.clone(),
        },
    );
    problem
}

fn lock<T>(mutex: &std::sync::Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn at(text: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(text)
            .expect("valid timestamp")
            .with_timezone(&Utc)
    }

    fn newer(version: &str) -> Stage {
        Stage::Newer {
            version: version.to_owned(),
            notes: vec!["Pace for each window.".to_owned()],
        }
    }

    #[test]
    fn updates_come_only_from_the_latest_full_release_signed_for_their_version() {
        let config: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.conf.json")).expect("valid JSON");
        let updater = &config["plugins"]["updater"];
        let endpoints = updater["endpoints"].as_array().expect("endpoints");
        assert_eq!(endpoints.len(), 1);
        let endpoint = endpoints[0].as_str().expect("an address");
        assert_eq!(
            endpoint,
            "https://github.com/pghmatt/quotadial/releases/latest/download/latest.json"
        );
        assert!(quotadial_core::privacy::named(endpoint));
        assert_eq!(updater["requireSignedVersion"], true);
        // "untrusted comment: minisign public key: …", base64 encoded, as `tauri signer` writes it.
        assert!(
            updater["pubkey"]
                .as_str()
                .expect("a public key")
                .starts_with("dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6")
        );
        assert!(
            config["bundle"].get("createUpdaterArtifacts").is_none(),
            "only a release with the signing key makes updates (docs/RELEASE.md)"
        );
    }

    #[test]
    fn a_check_is_due_daily_hourly_after_no_answer_and_when_the_clock_goes_back() {
        let now = at("2099-01-02T12:00:00Z");
        assert!(Record::default().due(now), "never checked");

        let answered = |checked: &str| Record {
            checked_at: Some(at(checked)),
            ..Record::default()
        };
        assert!(!answered("2099-01-02T00:00:00Z").due(now));
        assert!(answered("2099-01-01T12:00:00Z").due(now), "a day later");
        assert!(
            answered("2099-01-03T00:00:00Z").due(now),
            "the clock went back past the last check"
        );

        let unanswered = Record {
            unanswered: true,
            ..answered("2099-01-02T11:30:00Z")
        };
        assert!(!unanswered.due(now));
        assert!(
            unanswered.due(now + TimeDelta::minutes(30)),
            "an hour later"
        );
    }

    #[test]
    fn an_offer_not_installed_is_checked_again_soon_after_a_start() {
        let record = Record {
            offered: Some("0.3.0".into()),
            ..Record::default()
        };
        assert!(record.offers_other_than("0.2.0"));
        assert!(!record.offers_other_than("0.3.0"), "installed");
        assert!(!Record::default().offers_other_than("0.2.0"));
    }

    #[test]
    fn the_record_survives_a_restart_and_a_damaged_one_starts_over() {
        let temp = tempfile::tempdir().expect("temp dir");
        let path = temp.path().join("config/updates.json");
        let record = Record {
            checked_at: Some(at("2099-01-02T12:00:00Z")),
            unanswered: false,
            offered: Some("0.3.0".into()),
            told: Some("0.3.0".into()),
        };

        record.save(&path).expect("saves");
        assert_eq!(Record::load(&path), record);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(&path).expect("metadata").permissions().mode();
            assert_eq!(mode & 0o777, 0o600);
        }
        fs::write(&path, "not json").expect("writes");
        assert_eq!(Record::load(&path), Record::default());
    }

    #[test]
    fn the_daily_check_tells_each_newer_version_once() {
        let now = at("2099-01-02T12:00:00Z");
        let mut updates = Updates::new(Record::default());

        let notice = updates.checked(newer("0.3.0"), now, Trigger::Daily);
        assert_eq!(
            notice,
            Some(Notice {
                title: "QuotaDial 0.3.0 is available".into(),
                body: "Open QuotaDial and choose Update to install it.".into(),
            })
        );
        assert_eq!(updates.record.offered.as_deref(), Some("0.3.0"));
        assert_eq!(updates.record.checked_at, Some(now));

        assert_eq!(updates.checked(newer("0.3.0"), now, Trigger::Daily), None);
        assert!(
            updates
                .checked(newer("0.3.1"), now, Trigger::Daily)
                .is_some()
        );
    }

    #[test]
    fn a_check_by_hand_tells_nothing_since_the_person_is_looking() {
        let mut updates = Updates::new(Record::default());

        assert_eq!(
            updates.checked(newer("0.3.0"), at("2099-01-02T12:00:00Z"), Trigger::ByHand),
            None
        );
        assert_eq!(updates.record.told, None);
    }

    #[test]
    fn an_unanswered_check_keeps_the_last_offer_and_comes_again_sooner() {
        let now = at("2099-01-02T12:00:00Z");
        let mut updates = Updates::new(Record::default());
        updates.checked(newer("0.3.0"), now, Trigger::Daily);

        updates.checked(Stage::Unanswered(Trigger::Daily), now, Trigger::Daily);
        assert!(updates.record.unanswered);
        assert_eq!(updates.record.offered.as_deref(), Some("0.3.0"));

        updates.checked(Stage::Current, now, Trigger::Daily);
        assert!(!updates.record.unanswered);
        assert_eq!(updates.record.offered, None, "installed or withdrawn");
    }

    #[test]
    fn answers_mean_what_the_window_says() {
        assert_eq!(
            found(Ok(Some((
                "0.3.0",
                Some("### Added\n\n- **Pace** for each window.\n")
            )))),
            newer("0.3.0")
        );
        assert_eq!(found(Ok(None)), Stage::Current);
        assert_eq!(
            found(Err(&Error::ReleaseNotFound)),
            Stage::Current,
            "no latest.json yet"
        );
        assert_eq!(
            found(Err(&Error::TargetsNotFound(vec![
                "linux-x86_64-deb".into()
            ]))),
            Stage::NoUpdateHere
        );
        assert_eq!(
            found(Err(&Error::Network("connection refused".into()))),
            Stage::Unanswered(Trigger::ByHand)
        );
    }

    #[test]
    fn notes_are_the_changes_in_plain_words() {
        assert_eq!(
            lines(
                "### Added\n\n- **Pace** for each window.\n- See [the docs](https://x.y).\n\n### Fixed\n\n- `qd` works.\n"
            ),
            ["Pace for each window.", "See the docs.", "qd works."]
        );
        assert!(lines("").is_empty());
    }

    #[test]
    fn the_window_says_where_updates_stand() {
        let now = at("2099-01-02T12:00:00Z");
        let mut updates = Updates::new(Record::default());
        let text = |updates: &Updates, automatic: bool| updates.view("0.2.0", automatic, now).text;

        assert_eq!(
            text(&updates, true),
            "QuotaDial 0.2.0. It checks for updates once a day."
        );
        assert_eq!(
            text(&updates, false),
            "QuotaDial 0.2.0. The daily check for updates is off."
        );
        updates.record.checked_at = Some(now - TimeDelta::hours(3));
        assert_eq!(
            text(&updates, true),
            "QuotaDial 0.2.0. Last checked for updates 3 h ago."
        );

        updates.stage = Stage::Checking;
        assert!(updates.view("0.2.0", true, now).busy);

        updates.checked(Stage::Current, now - TimeDelta::minutes(5), Trigger::ByHand);
        assert_eq!(
            text(&updates, true),
            "QuotaDial 0.2.0 is the newest version. Checked 5 min ago."
        );

        updates.checked(newer("0.3.0"), now, Trigger::ByHand);
        let offered = updates.view("0.2.0", true, now);
        assert_eq!(
            offered.text,
            "QuotaDial 0.3.0 is available. This copy is 0.2.0."
        );
        assert_eq!(offered.available.as_deref(), Some("0.3.0"));
        assert_eq!(offered.notes, ["Pace for each window."]);
        assert!(!offered.busy);

        updates.stage = Stage::Downloading {
            version: "0.3.0".into(),
            percent: Some(45),
        };
        let downloading = updates.view("0.2.0", true, now);
        assert_eq!(downloading.text, "Downloading QuotaDial 0.3.0… 45%");
        assert_eq!(downloading.progress, Some(45));
        assert!(downloading.busy);

        updates.stage = Stage::Installing {
            version: "0.3.0".into(),
        };
        assert_eq!(
            text(&updates, true),
            "Installing QuotaDial 0.3.0. It opens again when it is done."
        );

        updates.checked(Stage::Unanswered(Trigger::ByHand), now, Trigger::ByHand);
        assert_eq!(
            text(&updates, true),
            "QuotaDial could not reach GitHub to check for updates. Check the connection, then try \
             again."
        );
        updates.stage = Stage::Idle;
        assert_eq!(
            text(&updates, true),
            "QuotaDial 0.2.0. The last check for updates could not reach GitHub, so it tries \
             again within the hour."
        );
    }

    #[test]
    fn every_failure_names_the_fix_and_no_path_shows_the_user_name() {
        let home = PathBuf::from("/Users/someone");
        let problem = |error: &Error| problem(error, "0.3.0", Some(&home));

        assert!(
            problem(&Error::SignedVersionMismatch {
                signed: "0.2.0".into(),
                announced: "0.3.0".into(),
            })
            .starts_with(
                "QuotaDial 0.3.0 did not match QuotaDial's signature, so it was not installed."
            )
        );
        assert!(problem(&Error::MissingSignedVersion).contains("signature"));
        assert!(problem(&Error::Network("503".into())).contains("Check the connection"));
        assert!(problem(&Error::AuthenticationFailed).contains("permission"));
        assert!(
            problem(&Error::Io(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "denied"
            )))
            .contains("without your permission")
        );
        let kept = problem(&Error::PreviousAppNotRestored(
            home.join(".Trash/QuotaDial.app"),
        ));
        assert!(kept.contains("is now at ~"), "{kept}");
        assert!(kept.contains("QuotaDial.app"), "{kept}");
        assert!(!kept.contains("someone"), "{kept}");
        assert!(problem(&Error::TempDirNotOnSameMountPoint).contains("quotadial.com"));
    }
}
