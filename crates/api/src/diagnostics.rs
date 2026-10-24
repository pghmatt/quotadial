//! Gathers what `quotadial doctor` prints and the app's Send feedback attaches (D-101): the
//! version, the system, the licence without its id, the data folder, the daemon, the numbers and
//! the settings in effect. The text itself is built in core ([`Diagnostics::text`]), so both say
//! the same, and [`feedback_url`] opens a GitHub issue with it filled in.

use std::process::Command;

use chrono::{DateTime, Utc};
use quotadial_core::{Diagnostics, StatusReport, text};
use quotadial_daemon::{Config, DataDir, Instance, Licensing, Webhook};
use reqwest::Url;

use crate::Runtime;

/// Where QuotaDial's issues are, which Send feedback opens a new one in.
pub const ISSUES_URL: &str = "https://github.com/pghmatt/quotadial/issues";

/// Every change, newest first, as `CHANGELOG.md` keeps them.
pub const CHANGELOG_URL: &str = "https://github.com/pghmatt/quotadial/blob/main/CHANGELOG.md";

/// Where people write for help.
pub const SUPPORT_EMAIL: &str = "support@quotadial.com";

/// The most of the diagnostics a new-issue address carries. GitHub takes addresses up to about 8 KB,
/// and encoding can double the text; nine accounts and their sign-ins come to about 2,500.
const MOST_IN_URL: usize = 4_000;

/// The facts about the copy keeping its data in `dir` at `now`, with `report` as the numbers, which
/// came from `numbers_from`, such as "the daemon".
#[must_use]
pub fn gather(
    dir: Option<&DataDir>,
    report: StatusReport,
    numbers_from: &str,
    now: DateTime<Utc>,
) -> Diagnostics {
    let licence = Licensing::new(dir).standing().without_id();
    let data_folder = dir.map_or_else(
        || "none: the operating system names none, so set QUOTADIAL_DATA_DIR".to_owned(),
        |dir| dir.path().display().to_string(),
    );
    let daemon = match dir {
        Some(dir) if Instance::is_running(dir) => match Runtime::read(dir) {
            Some(runtime) => format!(
                "running, process {}, API on 127.0.0.1:{}, started {}",
                runtime.pid,
                runtime.port,
                text::ago(runtime.started_at, now)
            ),
            None => "running, starting its API".to_owned(),
        },
        _ => "not running".to_owned(),
    };
    let config = dir.map(Config::load).unwrap_or_default();
    Diagnostics {
        version: env!("CARGO_PKG_VERSION").to_owned(),
        system: system(),
        licence,
        data_folder,
        daemon,
        numbers_from: numbers_from.to_owned(),
        report,
        settings: settings(&config),
    }
}

/// The settings in effect, with webhooks named by their host.
fn settings(config: &Config) -> Vec<(String, String)> {
    let listed = |items: Vec<String>| {
        if items.is_empty() {
            "none".to_owned()
        } else {
            items.join(", ")
        }
    };
    let on = |on: bool| if on { "on" } else { "off" }.to_owned();
    vec![
        (
            "extra_config_dirs".to_owned(),
            listed(config.extra_config_dirs.clone()),
        ),
        (
            "max_accounts".to_owned(),
            config
                .max_accounts
                .map_or_else(|| "every account".to_owned(), |max| max.to_string()),
        ),
        ("history_days".to_owned(), config.history_days.to_string()),
        ("status_pages".to_owned(), on(config.status_pages)),
        ("cost_scan".to_owned(), on(config.cost_scan)),
        (
            "alert webhooks".to_owned(),
            listed(
                config
                    .webhooks
                    .iter()
                    .map(Webhook::host)
                    .map(str::to_owned)
                    .collect(),
            ),
        ),
        ("warnings".to_owned(), listed(config.warnings.clone())),
    ]
}

/// The operating system, its version where it says, and the processor, such as
/// `macOS 26.1 (aarch64)`.
#[must_use]
pub fn system() -> String {
    let name = match std::env::consts::OS {
        "macos" => run("sw_vers", &["-productVersion"])
            .map_or_else(|| "macOS".to_owned(), |version| format!("macOS {version}")),
        "linux" => std::fs::read_to_string("/etc/os-release")
            .ok()
            .and_then(|release| {
                release.lines().find_map(|line| {
                    line.strip_prefix("PRETTY_NAME=")
                        .map(|name| name.trim_matches('"').to_owned())
                })
            })
            .unwrap_or_else(|| "Linux".to_owned()),
        "windows" => run("cmd", &["/C", "ver"]).unwrap_or_else(|| "Windows".to_owned()),
        other => other.to_owned(),
    };
    format!("{name} ({})", std::env::consts::ARCH)
}

/// What `program` prints, trimmed, when it runs and prints something.
fn run(program: &str, arguments: &[&str]) -> Option<String> {
    let output = Command::new(program).args(arguments).output().ok()?;
    let printed = String::from_utf8(output.stdout).ok()?.trim().to_owned();
    (output.status.success() && !printed.is_empty()).then_some(printed)
}

/// The address of a new GitHub issue with `diagnostics`, the text from [`Diagnostics::text`], filled
/// in below a place for the person to write. A long text is cut short and says where the rest is.
#[must_use]
pub fn feedback_url(diagnostics: &str) -> String {
    let shown = if diagnostics.len() > MOST_IN_URL {
        let mut end = MOST_IN_URL;
        while !diagnostics.is_char_boundary(end) {
            end -= 1;
        }
        let cut = diagnostics[..end]
            .rfind('\n')
            .map_or(&diagnostics[..end], |line| &diagnostics[..=line]);
        format!("{cut}… cut short: `quotadial doctor` prints all of it.\n")
    } else {
        diagnostics.to_owned()
    };
    let body = format!(
        "**What happened, or what would help?**\n\n\n\n**About this copy**, from QuotaDial's Send \
         feedback. It holds no token, no whole email address and no licence id.\n\n```text\n\
         {shown}```\n"
    );
    Url::parse_with_params(&format!("{ISSUES_URL}/new"), [("body", body.as_str())])
        .map_or_else(|_| format!("{ISSUES_URL}/new"), String::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_facts_come_from_the_data_folder_with_the_licence_s_id_left_out() {
        let temp = tempfile::tempdir().expect("temp dir");
        let dir = DataDir::new(temp.path().join("quotadial"));
        dir.create().expect("creates");
        std::fs::write(
            dir.path().join("config.json"),
            r#"{"history_days": 90, "slack_webhook": "https://hooks.slack.com/services/T0/B0/secret"}"#,
        )
        .expect("writes");
        let pro = crate::tests::pro(&dir);
        assert_eq!(pro.tier(), quotadial_core::Tier::Pro);

        let facts = gather(
            Some(&dir),
            StatusReport::default(),
            "a direct fetch",
            Utc::now(),
        );

        assert_eq!(facts.version, env!("CARGO_PKG_VERSION"));
        assert!(
            facts
                .system
                .ends_with(&format!("({})", std::env::consts::ARCH))
        );
        assert_eq!(
            facts.licence,
            "QuotaDial Pro, with a licence that does not end"
        );
        assert_eq!(facts.daemon, "not running");
        assert_eq!(facts.numbers_from, "a direct fetch");
        assert!(
            facts
                .settings
                .contains(&("history_days".into(), "90".into()))
        );
        assert!(
            facts
                .settings
                .contains(&("alert webhooks".into(), "hooks.slack.com".into()))
        );
        let text = facts.text(Utc::now(), Some(temp.path()));
        assert!(
            !text.contains("secret"),
            "a webhook is named by its host only"
        );
        assert!(!text.contains("api-test"), "no licence id");
        let data_folder = format!(
            "Data folder: {}",
            std::path::Path::new("~").join("quotadial").display()
        );
        assert!(text.contains(&data_folder), "{text}");
    }

    #[test]
    fn with_no_data_folder_the_facts_say_so() {
        let facts = gather(None, StatusReport::default(), "a direct fetch", Utc::now());

        assert_eq!(facts.licence, "QuotaDial Free, with no licence key.");
        assert!(facts.data_folder.contains("QUOTADIAL_DATA_DIR"));
        assert_eq!(facts.daemon, "not running");
    }

    #[test]
    fn feedback_opens_a_new_issue_with_the_diagnostics_filled_in() {
        let url = Url::parse(&feedback_url("QuotaDial 0.3.0 on macOS 26.1 (aarch64)\n"))
            .expect("an address");

        assert_eq!(
            url.as_str().split('?').next(),
            Some("https://github.com/pghmatt/quotadial/issues/new")
        );
        let body = url
            .query_pairs()
            .find(|(name, _)| name == "body")
            .map(|(_, body)| body.into_owned())
            .expect("a body");
        assert!(body.starts_with("**What happened, or what would help?**"));
        assert!(body.contains("```text\nQuotaDial 0.3.0 on macOS 26.1 (aarch64)\n```"));
    }

    #[test]
    fn long_diagnostics_are_cut_at_a_line_and_say_where_the_rest_is() {
        let line = "  claude m***@e***.com (max): ok, fetched 3 min ago from the API ✓\n";
        let long = line.repeat(200);

        let url = feedback_url(&long);

        assert!(url.len() < 8_000, "{}", url.len());
        let body = Url::parse(&url)
            .expect("an address")
            .query_pairs()
            .find(|(name, _)| name == "body")
            .map(|(_, body)| body.into_owned())
            .expect("a body");
        assert!(body.contains(&format!(
            "{line}… cut short: `quotadial doctor` prints all of it.\n"
        )));
    }

    #[test]
    fn the_system_names_its_processor() {
        assert!(system().ends_with(&format!("({})", std::env::consts::ARCH)));
    }
}
