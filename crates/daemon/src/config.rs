//! QuotaDial's settings for every process: `config.json` in its data folder. A missing or damaged
//! file means the defaults, and so does a setting of the wrong kind, setting by setting.
//!
//! The webhooks alerts are sent to (D-094) are addresses that work like passwords: a Slack
//! address holds its channel's key. They are kept as [`Secret`]s, and QuotaDial only ever shows a
//! webhook's host. A file that holds one and that other users can read gets a warning.

use std::path::Path;

use quotadial_core::{Secret, text};
use reqwest::Url;
use serde::de::DeserializeOwned;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::{DataDir, KEEP_DAYS, MAX_KEEP_DAYS};

/// QuotaDial's settings.
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    /// More folders where Claude Code keeps sign-ins, as `CLAUDE_CONFIG_DIR` would name them
    /// (#34). `~` is the home folder, and a folder ending in `/*` adds every folder inside it.
    pub extra_config_dirs: Vec<String>,
    /// At most this many accounts per provider, the shell's own first, and at least 1. All when
    /// unset. QuotaDial Free shows one, whatever this says (D-100).
    pub max_accounts: Option<usize>,
    /// Where each alert is sent besides the app's notifications: `alert_webhooks` as JSON, and
    /// `slack_webhook` as a Slack message.
    pub webhooks: Vec<Webhook>,
    /// How many days of history to keep: 30 unless `history_days` says otherwise, up to 365.
    /// QuotaDial Free keeps at most 30, whatever this says (D-090, D-100).
    pub history_days: u32,
    /// Whether the daemon reads the providers' status pages (D-096): on unless `status_pages` is
    /// `false`.
    pub status_pages: bool,
    /// Whether the daemon reads Claude Code's and the Codex CLI's session logs for the week's
    /// API-equivalent cost (D-097): on unless `cost_scan` is `false`.
    pub cost_scan: bool,
    /// What the settings could not be used for, each naming the fix and `config.json`, with the
    /// home folder written `~`, such as a webhook that is not an `https` address. Never the address
    /// itself.
    pub warnings: Vec<String>,
}

/// A webhook to send alerts to.
#[derive(Debug, Clone)]
pub struct Webhook {
    url: Secret,
    host: String,
    /// What the webhook expects.
    pub kind: WebhookKind,
}

/// What a webhook expects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebhookKind {
    /// The alert's fields as JSON.
    Json,
    /// A Slack incoming webhook's `{"text": ...}`.
    Slack,
}

impl Webhook {
    /// Reads `address` as a webhook of `kind`: an `https` address, or an `http` one on this machine
    /// for a local receiver. `None` when it is neither.
    #[must_use]
    pub fn new(address: &str, kind: WebhookKind) -> Option<Self> {
        let url = Url::parse(address.trim()).ok()?;
        let host = url.host_str()?.to_owned();
        let local = matches!(host.as_str(), "localhost" | "127.0.0.1" | "[::1]");
        match url.scheme() {
            "https" => {}
            "http" if local => {}
            _ => return None,
        }
        Some(Self {
            url: Secret::new(url.as_str()),
            host,
            kind,
        })
    }

    /// The address, for the one place it is used: the request.
    #[must_use]
    pub fn url(&self) -> &str {
        self.url.expose()
    }

    /// The webhook's host, which is all QuotaDial shows of it, such as `hooks.slack.com`.
    #[must_use]
    pub fn host(&self) -> &str {
        &self.host
    }

    /// A name for the webhook that tells it from others without holding its address: the start of
    /// the address's SHA-256, kept beside each alert sent to it.
    #[must_use]
    pub fn key(&self) -> String {
        Sha256::digest(self.url().as_bytes())
            .iter()
            .take(8)
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }
}

impl PartialEq for Webhook {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.url() == other.url()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            extra_config_dirs: Vec::new(),
            max_accounts: None,
            webhooks: Vec::new(),
            history_days: KEEP_DAYS,
            status_pages: true,
            cost_scan: true,
            warnings: Vec::new(),
        }
    }
}

impl Config {
    /// The settings in `dir`, or the defaults. A warning names `config.json` with this process's
    /// home folder written `~`.
    #[must_use]
    pub fn load(dir: &DataDir) -> Self {
        Self::loaded(dir, std::env::home_dir().as_deref())
    }

    /// [`Config::load`], with `home` as the home folder.
    fn loaded(dir: &DataDir, home: Option<&Path>) -> Self {
        let path = dir.path().join("config.json");
        let Some(settings) = std::fs::read(&path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
        else {
            return Self::default();
        };
        let mut config = Self {
            extra_config_dirs: setting(&settings, "extra_config_dirs").unwrap_or_default(),
            max_accounts: setting(&settings, "max_accounts").filter(|max| *max >= 1),
            status_pages: setting(&settings, "status_pages").unwrap_or(true),
            cost_scan: setting(&settings, "cost_scan").unwrap_or(true),
            ..Self::default()
        };
        if let Some(days) = setting::<u32>(&settings, "history_days") {
            if !(1..=MAX_KEEP_DAYS).contains(&days) {
                config.warnings.push(format!(
                    "history_days in {} must be from 1 to {MAX_KEEP_DAYS}, so QuotaDial keeps \
                     {KEEP_DAYS} days.",
                    path.display()
                ));
            } else {
                config.history_days = days;
            }
        }
        let addresses = setting::<Vec<String>>(&settings, "alert_webhooks")
            .unwrap_or_default()
            .into_iter()
            .map(|address| (address, WebhookKind::Json))
            .chain(
                setting::<String>(&settings, "slack_webhook")
                    .map(|address| (address, WebhookKind::Slack)),
            );
        for (address, kind) in addresses {
            match Webhook::new(&address, kind) {
                Some(webhook) => config.webhooks.push(webhook),
                None => config.warnings.push(format!(
                    "A webhook in {} is not an https address, so no alerts are sent to it; \
                     correct it there.",
                    path.display()
                )),
            }
        }
        if !config.webhooks.is_empty() && readable_by_others(&path) {
            config.warnings.push(format!(
                "{} holds webhook addresses, which work like passwords, and other users can read \
                 it; run `chmod 600 {}`.",
                path.display(),
                shell_word(&path, home)
            ));
        }
        // Each names config.json: the one pass its text goes through where it is made writes the
        // home folder `~` (#52, D-129). It leaves the chmod command as `shell_word` wrote it: the
        // pass writes `~` only for the home folder's own path, and the command holds that path
        // nowhere the pass takes it for the home folder, `~` standing for it already and a place
        // outside the home folder not starting in it. D-130 has the one exception, never a folder
        // QuotaDial makes: a place whose text holds the home folder's path beside a space, a quote
        // or the like, such as `/Users/ann (old)` beside `/Users/ann`.
        for warning in &mut config.warnings {
            *warning = text::tilde_in(warning, home);
        }
        config
    }
}

/// The setting called `name`, if it is there and of the right kind.
fn setting<T: DeserializeOwned>(settings: &Value, name: &str) -> Option<T> {
    serde_json::from_value(settings.get(name)?.clone()).ok()
}

/// `place` as a POSIX shell takes it, for a command a user pastes as written (D-130): in the home
/// folder `home`, `~/` left unquoted, so the shell expands it, and the rest in single quotes, so a
/// space does not split it, such as `~/'Library/Application Support/quotadial/config.json'`;
/// anywhere else, or with no home folder or one that is a whole disk, the whole place in single
/// quotes. A single quote in a name is written `'\''`: the quotes closed, a quote escaped, and the
/// quotes opened again. Quoted whatever the place holds, so no character a shell reads can be
/// missed. In the home folder by whole names, as [`text::tilde`] finds it, so `/Users/anna` is not
/// in `/Users/ann`. `~/` on every system, being for a POSIX shell: the warning that uses it cannot
/// occur on Windows.
fn shell_word(place: &Path, home: Option<&Path>) -> String {
    let quoted = |path: &Path| format!("'{}'", path.display().to_string().replace('\'', r"'\''"));
    match home
        .filter(|home| home.parent().is_some())
        .and_then(|home| place.strip_prefix(home).ok())
    {
        Some(rest) if rest.as_os_str().is_empty() => "~".to_owned(),
        Some(rest) => format!("~/{}", quoted(rest)),
        None => quoted(place),
    }
}

/// Whether users other than the owner can read the file at `path`.
fn readable_by_others(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(path).is_ok_and(|metadata| metadata.permissions().mode() & 0o077 != 0)
    }
    // Windows keeps a user's profile folder, where the data folder is, to that user.
    #[cfg(not(unix))]
    {
        let _ = path;
        false
    }
}

#[cfg(test)]
mod tests {
    use std::path::{MAIN_SEPARATOR, MAIN_SEPARATOR_STR, PathBuf};

    use super::*;

    fn written(json: &str) -> (tempfile::TempDir, DataDir) {
        let temp = tempfile::tempdir().expect("temp dir");
        let dir = written_in(temp.path().to_path_buf(), json);
        (temp, dir)
    }

    /// The data folder `folder`, made if it is not there, with `json` as its `config.json`,
    /// readable only by its owner.
    fn written_in(folder: PathBuf, json: &str) -> DataDir {
        let dir = DataDir::new(folder);
        dir.create().expect("makes the folder");
        let path = dir.path().join("config.json");
        std::fs::write(&path, json).expect("writes");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))
                .expect("private");
        }
        dir
    }

    /// `parts` in the home folder, as `text::tilde` writes it: `~` and the system's own separator,
    /// so `~/quotadial/config.json` on macOS and Linux and `~\quotadial\config.json` on Windows.
    fn in_home(parts: &[&str]) -> String {
        Path::new("~")
            .join(parts.iter().collect::<PathBuf>())
            .display()
            .to_string()
    }

    #[test]
    fn a_warning_names_config_json_with_the_home_folder_written_with_a_tilde() {
        // #52: no message shows a path with the user's name in it (D-129).
        let temp = tempfile::tempdir().expect("temp dir");
        let home = temp.path().join("ann");
        let warned = |folder: PathBuf| {
            let dir = written_in(
                folder,
                r#"{"history_days": 0, "alert_webhooks": ["http://example.com/plain"]}"#,
            );
            Config::loaded(&dir, Some(&home)).warnings
        };
        let warnings = |file: &str| {
            [
                format!(
                    "history_days in {file} must be from 1 to 365, so QuotaDial keeps 30 days."
                ),
                format!(
                    "A webhook in {file} is not an https address, so no alerts are sent to it; \
                     correct it there."
                ),
            ]
        };

        assert_eq!(
            warned(home.join("quotadial")),
            warnings(&in_home(&["quotadial", "config.json"]))
        );
        // Another user's folder, whose name starts as the home folder's does.
        let elsewhere = temp.path().join("anna").join("quotadial");
        assert_eq!(
            warned(elsewhere.clone()),
            warnings(&elsewhere.join("config.json").display().to_string()),
            "a place outside the home folder stays as it is"
        );
    }

    #[test]
    fn settings_come_from_config_json_with_defaults_for_the_rest() {
        let temp = tempfile::tempdir().expect("temp dir");
        assert_eq!(
            Config::load(&DataDir::new(temp.path().to_path_buf())),
            Config::default(),
            "no file yet"
        );

        let (_temp, dir) = written(r#"{"extra_config_dirs": ["~/claude-accounts/*"]}"#);
        assert_eq!(
            Config::load(&dir),
            Config {
                extra_config_dirs: vec!["~/claude-accounts/*".into()],
                ..Config::default()
            }
        );

        let (_temp, dir) =
            written(r#"{"extra_config_dirs": ["~/claude-accounts/*"], "max_accounts": "3"}"#);
        assert_eq!(
            Config::load(&dir),
            Config {
                extra_config_dirs: vec!["~/claude-accounts/*".into()],
                ..Config::default()
            },
            "a setting of the wrong kind falls back alone"
        );

        let (_temp, dir) = written(r#"{"max_accounts": 0}"#);
        assert_eq!(
            Config::load(&dir).max_accounts,
            None,
            "at least one account"
        );

        let (_temp, dir) = written("not json");
        assert_eq!(Config::load(&dir), Config::default(), "a damaged file");
    }

    #[test]
    fn history_is_kept_30_days_unless_set_from_1_to_365() {
        assert_eq!(Config::default().history_days, 30);
        let (_temp, dir) = written(r#"{"history_days": 90}"#);
        assert_eq!(Config::load(&dir).history_days, 90);

        for wrong in ["0", "400"] {
            let (_temp, dir) = written(&format!(r#"{{"history_days": {wrong}}}"#));
            let config = Config::load(&dir);
            assert_eq!(config.history_days, 30, "{wrong}");
            assert_eq!(config.warnings.len(), 1, "{wrong}");
            assert!(config.warnings[0].starts_with("history_days in "));
            assert!(
                config.warnings[0].ends_with("must be from 1 to 365, so QuotaDial keeps 30 days.")
            );
        }
        let (_temp, dir) = written(r#"{"history_days": "ninety"}"#);
        assert_eq!(Config::load(&dir).history_days, 30, "the wrong kind");
    }

    #[test]
    fn status_pages_are_read_unless_turned_off() {
        assert!(Config::default().status_pages);
        let (_temp, dir) = written(r#"{"status_pages": false}"#);
        assert!(!Config::load(&dir).status_pages);
        let (_temp, dir) = written(r#"{"status_pages": "no"}"#);
        assert!(
            Config::load(&dir).status_pages,
            "the wrong kind keeps the default"
        );
    }

    #[test]
    fn the_cost_scan_runs_unless_turned_off() {
        assert!(Config::default().cost_scan);
        let (_temp, dir) = written(r#"{"cost_scan": false}"#);
        assert!(!Config::load(&dir).cost_scan);
        let (_temp, dir) = written(r#"{"cost_scan": 0}"#);
        assert!(
            Config::load(&dir).cost_scan,
            "the wrong kind keeps the default"
        );
    }

    #[test]
    fn webhooks_are_https_or_on_this_machine_and_never_shown() {
        let (_temp, dir) = written(
            r#"{
                "alert_webhooks": ["https://example.com/hooks/quota", "http://127.0.0.1:8080/in",
                                   "http://example.com/plain", "not an address"],
                "slack_webhook": "https://hooks.slack.com/services/T000/B000/XXXXXXXX"
            }"#,
        );

        let config = Config::load(&dir);

        let kept: Vec<(&str, WebhookKind)> = config
            .webhooks
            .iter()
            .map(|webhook| (webhook.host(), webhook.kind))
            .collect();
        assert_eq!(
            kept,
            [
                ("example.com", WebhookKind::Json),
                ("127.0.0.1", WebhookKind::Json),
                ("hooks.slack.com", WebhookKind::Slack),
            ]
        );
        assert_eq!(config.warnings.len(), 2, "{:?}", config.warnings);
        assert!(
            config.warnings[0].ends_with(
                "is not an https address, so no alerts are sent to it; correct it there."
            )
        );
        let shown = format!("{config:?}");
        assert!(!shown.contains("XXXXXXXX"), "{shown}");
        assert!(!shown.contains("/hooks/quota"), "{shown}");
        assert_ne!(config.webhooks[0].key(), config.webhooks[1].key());
        assert_eq!(config.webhooks[0].key().len(), 16);
    }

    #[cfg(unix)]
    #[test]
    fn a_file_with_webhooks_that_others_can_read_gets_a_warning() {
        use std::os::unix::fs::PermissionsExt;
        let temp = tempfile::tempdir().expect("temp dir");
        let home = temp.path().join("ann");
        let readable = |folder: PathBuf| {
            let dir = written_in(
                folder,
                r#"{"slack_webhook": "https://hooks.slack.com/services/T/B/X"}"#,
            );
            let path = dir.path().join("config.json");
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).expect("chmod");
            (dir, path)
        };
        let warning = |file: &str, word: &str| {
            format!(
                "{file} holds webhook addresses, which work like passwords, and other users can \
                 read it; run `chmod 600 {word}`."
            )
        };

        // Written `~` where it is in the home folder (#52, D-129), and in the command `~` and the
        // rest quoted, so it works as written where a name has a space, as macOS's data folder's
        // does (D-130).
        let (dir, path) = readable(
            home.join("Library")
                .join("Application Support")
                .join("quotadial"),
        );
        assert_eq!(
            Config::loaded(&dir, Some(&home)).warnings,
            [warning(
                "~/Library/Application Support/quotadial/config.json",
                "~/'Library/Application Support/quotadial/config.json'"
            )]
        );
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).expect("chmod");
        assert!(Config::loaded(&dir, Some(&home)).warnings.is_empty());
        // Another user's folder, whose name starts as the home folder's does: as it is, and the
        // whole place quoted in the command.
        let (dir, path) = readable(temp.path().join("anna").join("quotadial"));
        let file = path.display().to_string();
        assert_eq!(
            Config::loaded(&dir, Some(&home)).warnings,
            [warning(&file, &format!("'{file}'"))]
        );
    }

    #[cfg(unix)]
    #[test]
    fn the_warnings_chmod_command_works_as_written_in_a_shell() {
        use std::os::unix::fs::PermissionsExt;
        use std::process::Command;
        // #52: every error names the fix, in one sentence a stranger can act on (D-130). The
        // command, taken from the warning as a user sees it, run by `sh` with the same home folder,
        // on a file in a folder whose name has a space, as macOS's data folder's does, then a quote
        // too, in the home folder and outside it.
        let temp = tempfile::tempdir().expect("temp dir");
        let home = temp.path().join("ann");
        for folder in [
            home.join("Library")
                .join("Application Support")
                .join("quotadial"),
            home.join("Ann's QuotaDial"),
            temp.path().join("O'Brien's data").join("quotadial"),
        ] {
            let dir = written_in(
                folder,
                r#"{"slack_webhook": "https://hooks.slack.com/services/T/B/X"}"#,
            );
            let path = dir.path().join("config.json");
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).expect("chmod");
            let warnings = Config::loaded(&dir, Some(&home)).warnings;
            let command = warnings
                .first()
                .and_then(|warning| warning.split_once("; run `"))
                .and_then(|(_, run)| run.strip_suffix("`."))
                .expect("the warning names a command");

            // In the temporary folder, so a place the shell split could name nothing outside it.
            let ran = Command::new("sh")
                .arg("-c")
                .arg(command)
                .env("HOME", &home)
                .current_dir(temp.path())
                .status()
                .expect("sh runs");

            assert!(ran.success(), "{command}");
            let mode = std::fs::metadata(&path)
                .expect("there")
                .permissions()
                .mode();
            assert_eq!(mode & 0o777, 0o600, "{command}");
            assert!(
                Config::loaded(&dir, Some(&home)).warnings.is_empty(),
                "{command}"
            );
        }
    }

    /// `shell_word`'s word for `place`, held to the warning's pass leaving it as it was written.
    fn word(place: &Path, home: Option<&Path>) -> String {
        let word = shell_word(place, home);
        assert_eq!(text::tilde_in(&word, home), word, "the warning's pass");
        word
    }

    #[test]
    fn a_place_in_the_home_folder_is_quoted_after_a_bare_tilde() {
        // macOS keeps the data folder in `~/Library/Application Support`, whose space split the
        // command, and a shell expands `~` only where it is not quoted (D-130).
        let home: PathBuf = [MAIN_SEPARATOR_STR, "Users", "ann"].iter().collect();
        let rest: PathBuf = ["Library", "Application Support", "quotadial", "config.json"]
            .iter()
            .collect();
        assert_eq!(
            word(&home.join(&rest), Some(&home)),
            format!("~/'{}'", rest.display())
        );
        assert_eq!(word(&home, Some(&home)), "~", "the home folder itself");
    }

    #[test]
    fn a_place_outside_the_home_folder_is_quoted_whole() {
        let home: PathBuf = [MAIN_SEPARATOR_STR, "Users", "ann"].iter().collect();
        let beside: PathBuf = [MAIN_SEPARATOR_STR, "Users", "anna", "config.json"]
            .iter()
            .collect();
        let elsewhere: PathBuf = [MAIN_SEPARATOR_STR, "Volumes", "Data", "config.json"]
            .iter()
            .collect();
        for place in [beside, elsewhere] {
            assert_eq!(
                word(&place, Some(&home)),
                format!("'{}'", place.display()),
                "a folder whose name only starts as the home folder's does, and another disk"
            );
        }
        let place = home.join("config.json");
        assert_eq!(
            word(&place, None),
            format!("'{}'", place.display()),
            "no home folder"
        );
        assert_eq!(
            word(&place, Some(Path::new(MAIN_SEPARATOR_STR))),
            format!("'{}'", place.display()),
            "a home folder that is a whole disk, which the warning's sentence leaves as it is too"
        );
    }

    #[test]
    fn a_single_quote_in_a_place_is_written_as_a_shell_reads_it() {
        // `'\''`: the quotes closed, a quote escaped, and the quotes opened again.
        let home: PathBuf = [MAIN_SEPARATOR_STR, "Users", "ann"].iter().collect();
        assert_eq!(
            word(&home.join("Ann's data").join("config.json"), Some(&home)),
            format!(r"~/'Ann'\''s data{MAIN_SEPARATOR}config.json'")
        );
        let elsewhere: PathBuf = [MAIN_SEPARATOR_STR, "Volumes", "O'Brien", "config.json"]
            .iter()
            .collect();
        assert_eq!(
            word(&elsewhere, Some(&home)),
            format!(
                r"'{MAIN_SEPARATOR}Volumes{MAIN_SEPARATOR}O'\''Brien{MAIN_SEPARATOR}config.json'"
            )
        );
    }

    proptest::proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig::with_cases(64))]

        #[test]
        fn no_settings_file_makes_loading_panic(
            settings in quotadial_core::testing::json(),
            webhooks in proptest::collection::vec(".{0,60}", 0..4),
            slack in ".{0,60}",
            days in proptest::prelude::any::<i64>(),
        ) {
            let (_temp, dir) = written(&settings.to_string());
            let _ = Config::load(&dir);
            let shaped = serde_json::json!({
                "alert_webhooks": webhooks,
                "slack_webhook": slack,
                "history_days": days,
                "max_accounts": days,
                "extra_config_dirs": settings,
            });
            let (_temp, dir) = written(&shaped.to_string());
            let config = Config::load(&dir);
            for webhook in &config.webhooks {
                proptest::prop_assert!(!webhook.host().is_empty());
            }
        }
    }
}
