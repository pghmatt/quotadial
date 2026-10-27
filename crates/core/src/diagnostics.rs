//! What support asks for, as one text with nothing secret in it (D-101): `quotadial doctor` prints
//! it and the app's Send feedback attaches it, so a report reads the same from either. Whoever
//! gathers the facts fills in [`Diagnostics`]; the text is built only here.
//!
//! The text holds no token, no email address in full, no home folder by name, no user name where it
//! names the user, no licence id and no webhook address: tokens are redacted, email addresses are
//! cut to their first letters, the home folder is written `~` by [`text::tilde_in`], the rule every
//! message follows, the user's name is written `[user]` where it is part of a place outside the
//! home folder by [`text::hide_user_in`], and webhooks are named by their host.

use std::path::Path;

use chrono::{DateTime, Utc};

use crate::{StatusReport, redact_secrets, text};

/// The facts support asks for.
#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostics {
    /// QuotaDial's version, such as `0.3.0`.
    pub version: String,
    /// The operating system, its version and the processor, such as `macOS 26.1 (aarch64)`.
    pub system: String,
    /// What the copy is, without the licence's id, such as `QuotaDial Pro until 24 Oct 2027`.
    pub licence: String,
    /// The data folder.
    pub data_folder: String,
    /// Whether a daemon runs, and where its API is.
    pub daemon: String,
    /// Where the numbers below came from, such as `the daemon` or `a direct fetch`.
    pub numbers_from: String,
    /// The accounts, what each provider could not look at, and the status pages.
    pub report: StatusReport,
    /// The settings in effect, as `(name, value)`, with webhooks named by their host only.
    pub settings: Vec<(String, String)>,
}

impl Diagnostics {
    /// The text, at `now`, with every place in the home folder `home` written `~`, as
    /// [`text::tilde_in`] writes it for every message, and the user's name, the home folder's own,
    /// hidden in a place outside it, as [`text::hide_user_in`] does and nowhere else: a command or a
    /// place in the home folder keeps the words it has, whatever the user is called (D-134).
    #[must_use]
    pub fn text(&self, now: DateTime<Utc>, home: Option<&Path>) -> String {
        let mut lines = vec![
            format!("QuotaDial {} on {}", self.version, self.system),
            format!("Licence: {}", self.licence),
            format!("Data folder: {}", self.data_folder),
            format!("Daemon: {}", self.daemon),
            format!("Numbers: from {}", self.numbers_from),
            String::new(),
        ];
        let report = &self.report;
        lines.push(format!("Accounts ({}):", report.snapshots.len()));
        if report.snapshots.is_empty() {
            lines.push("  none found".to_owned());
        }
        for snapshot in &report.snapshots {
            let account = &snapshot.account;
            let plan = account
                .plan
                .as_ref()
                .map(|plan| format!(" ({plan})"))
                .unwrap_or_default();
            let newest = snapshot
                .windows
                .iter()
                .max_by_key(|window| window.fetched_at);
            let fetched = newest.map_or_else(String::new, |window| {
                format!(
                    ", fetched {} from {}",
                    text::ago(window.fetched_at, now),
                    text::source_label(window.source)
                )
            });
            // The name once when it says nothing beyond the provider's label.
            lines.push(format!(
                "  {}{plan}: {}{fetched}",
                text::labelled(account.provider_label(), &account.display_name),
                text::status_label(snapshot.status)
            ));
            // What QuotaDial does with the sign-in, in the provider's words, which name where it is
            // when the provider knows (P-020). An account from an older QuotaDial says nothing.
            if !account.sign_in_use.is_empty() {
                lines.push(format!("    sign-in: {}", account.sign_in_use));
            }
            if let Some(error) = &snapshot.error {
                lines.push(format!("    {error}"));
            }
        }
        let sections = [
            (
                "Left out on QuotaDial Free",
                report
                    .left_out
                    .iter()
                    .map(|left_out| left_out.text.clone())
                    .collect::<Vec<_>>(),
            ),
            (
                "Status pages",
                // Named as the report's accounts name their provider (P-020).
                report
                    .status_pages
                    .iter()
                    .map(|page| {
                        format!(
                            "{}: {}",
                            report.provider_name(&page.provider),
                            page.description
                        )
                    })
                    .collect(),
            ),
            (
                "Problems",
                // A provider's under its own name, as every surface shows it (D-128).
                report
                    .error_lines()
                    .chain(report.problems.iter().cloned())
                    .collect(),
            ),
        ];
        for (heading, items) in sections {
            if !items.is_empty() {
                lines.push(format!("{heading}:"));
                lines.extend(items.into_iter().map(|item| format!("  {item}")));
            }
        }
        lines.push("Settings in effect:".to_owned());
        lines.extend(
            self.settings
                .iter()
                .map(|(name, value)| format!("  {name}: {value}")),
        );
        let mut text = lines.join("\n");
        text.push('\n');
        // The home folder written `~` by the one rule every message QuotaDial writes follows, so a
        // folder whose name only starts as the home folder's does, such as `/Users/anna` beside
        // `/Users/ann`, keeps its name (D-129).
        text = mask_emails(&redact_secrets(&text::tilde_in(&text, home)));
        // The user's name can be in a place outside the home folder too, such as a data folder that
        // QUOTADIAL_DATA_DIR puts in a temporary folder named after it. It is hidden there and
        // nowhere else, not in a command or a sentence that only spells it, as `claude` does for a
        // user called `claude` (D-134). Email addresses are cut first, so one that starts with the
        // name is still cut to its first letters.
        text::hide_user_in(&text, home)
    }
}

/// `text` with every email address cut to the first letter of its name and of its domain, and the
/// domain's ending, such as `m***@a***.com`.
#[must_use]
pub fn mask_emails(text: &str) -> String {
    let is_name = |c: char| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '%' | '+' | '-');
    let is_domain = |c: char| c.is_ascii_alphanumeric() || matches!(c, '.' | '-');
    let chars: Vec<char> = text.chars().collect();
    let mut masked = String::with_capacity(text.len());
    let mut index = 0;
    while index < chars.len() {
        if chars[index] != '@' {
            masked.push(chars[index]);
            index += 1;
            continue;
        }
        // The name is what was already copied, back to the first character that cannot be in it.
        let name_length = masked.chars().rev().take_while(|c| is_name(*c)).count();
        let domain: String = chars[index + 1..]
            .iter()
            .take_while(|c| is_domain(**c))
            .collect();
        let domain = domain.trim_end_matches('.');
        let ending = domain.rsplit_once('.').map(|(_, ending)| ending);
        match ending {
            Some(ending)
                if name_length > 0 && !ending.is_empty() && domain.len() > ending.len() + 1 =>
            {
                let kept: String = masked
                    .chars()
                    .take(masked.chars().count() - name_length)
                    .collect();
                let first_of_name = masked.chars().rev().nth(name_length - 1).unwrap_or('*');
                let first_of_domain = domain.chars().next().unwrap_or('*');
                masked = kept;
                masked.push(first_of_name);
                masked.push_str("***@");
                masked.push(first_of_domain);
                masked.push_str("***.");
                masked.push_str(ending);
                index += 1 + domain.chars().count();
            }
            _ => {
                masked.push('@');
                index += 1;
            }
        }
    }
    masked
}

#[cfg(test)]
mod tests {
    use std::path::{MAIN_SEPARATOR_STR, PathBuf};

    use proptest::prelude::*;

    use super::*;
    use crate::{
        Account, Indicator, LeftOut, ProFeature, ProviderError, ProviderName, Snapshot, Source,
        Status, StatusPage, Window, WindowKind,
    };

    fn at(text: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(text)
            .expect("a time")
            .with_timezone(&Utc)
    }

    fn diagnostics() -> Diagnostics {
        let place = "the macOS Keychain item \"Claude Code-credentials\" (CLAUDE_CONFIG_DIR \
                     /home/someone/.claude-work)";
        let account = Account {
            provider: "claude".into(),
            provider_name: "Claude".into(),
            account_id: "a-1".into(),
            display_name: "matt@example.com".into(),
            plan: Some("max".into()),
            org: None,
            sign_in: Some(place.into()),
            sign_in_use: format!(
                "QuotaDial reads this account's sign-in from {place}. It only reads it and never \
                 changes it."
            ),
        };
        Diagnostics {
            version: "0.3.0".into(),
            system: "macOS 26.1 (aarch64)".into(),
            licence: "QuotaDial Free, with no licence key.".into(),
            data_folder: "/home/someone/.local/share/quotadial".into(),
            daemon: "running with the app, API on 127.0.0.1:47115".into(),
            numbers_from: "the daemon".into(),
            report: StatusReport {
                snapshots: vec![Snapshot {
                    account,
                    windows: vec![Window {
                        kind: WindowKind::Session,
                        label: "Session".into(),
                        used_pct: 12.0,
                        resets_at: None,
                        window_seconds: 18_000,
                        scope_model: None,
                        source: Source::Api,
                        fetched_at: at("2099-01-01T00:00:00Z"),
                        unconfirmed: false,
                        pace: None,
                    }],
                    extra_usage: None,
                    status: Status::Ok,
                    error: None,
                }],
                errors: vec![ProviderError {
                    provider: "codex".into(),
                    message: "the sign-in in /home/someone/.codex/auth.json for someone@corp.example.org could not be read; token sk-ant-oat01-abcdefghijklmnopqrstuvwxyz0123456789"
                        .into(),
                }],
                status_pages: Vec::new(),
                costs: Vec::new(),
                left_out: vec![LeftOut::feature(ProFeature::CostScan)],
                problems: Vec::new(),
                providers: vec![
                    ProviderName {
                        id: "claude".into(),
                        name: "Claude".into(),
                    },
                    ProviderName {
                        id: "codex".into(),
                        name: "Codex".into(),
                    },
                ],
            },
            settings: vec![
                ("history_days".into(), "30".into()),
                ("alert webhooks".into(), "hooks.slack.com".into()),
            ],
        }
    }

    #[test]
    fn the_text_says_what_support_asks_for_with_nothing_secret_in_it() {
        let text = diagnostics().text(at("2099-01-01T00:03:00Z"), Some(Path::new("/home/someone")));

        assert_eq!(
            text,
            "QuotaDial 0.3.0 on macOS 26.1 (aarch64)\n\
             Licence: QuotaDial Free, with no licence key.\n\
             Data folder: ~/.local/share/quotadial\n\
             Daemon: running with the app, API on 127.0.0.1:47115\n\
             Numbers: from the daemon\n\
             \n\
             Accounts (1):\n  \
             Claude m***@e***.com (max): ok, fetched 3 min ago from the API\n    \
             sign-in: QuotaDial reads this account's sign-in from the macOS Keychain item \
             \"Claude Code-credentials\" (CLAUDE_CONFIG_DIR ~/.claude-work). It only reads it and \
             never changes it.\n\
             Left out on QuotaDial Free:\n  \
             The week's cost at API prices is part of QuotaDial Pro: quotadial.com/pro\n\
             Problems:\n  \
             Codex: the sign-in in ~/.codex/auth.json for s***@c***.org could not be read; token [redacted token]\n\
             Settings in effect:\n  \
             history_days: 30\n  \
             alert webhooks: hooks.slack.com\n"
        );
    }

    #[test]
    fn an_account_from_an_older_quotadial_shows_its_providers_id_and_no_sign_in_line() {
        let mut facts = diagnostics();
        let account = &mut facts.report.snapshots[0].account;
        account.provider_name = String::new();
        account.sign_in_use = String::new();

        let text = facts.text(at("2099-01-01T00:03:00Z"), Some(Path::new("/home/someone")));

        assert!(
            text.contains(
                "Accounts (1):\n  claude m***@e***.com (max): ok, fetched 3 min ago from the \
                 API\nLeft out"
            ),
            "{text}"
        );
    }

    #[test]
    fn an_account_called_as_its_provider_is_named_once() {
        let mut facts = diagnostics();
        let account = &mut facts.report.snapshots[0].account;
        account.provider_name = "Example".into();
        account.display_name = "Example".into();

        let text = facts.text(at("2099-01-01T00:03:00Z"), Some(Path::new("/home/someone")));

        assert!(
            text.contains("Accounts (1):\n  Example (max): ok, fetched 3 min ago from the API\n"),
            "{text}"
        );
    }

    #[test]
    fn a_status_page_is_named_as_its_provider_names_itself() {
        let mut facts = diagnostics();
        facts.report.status_pages.push(StatusPage {
            provider: "claude".into(),
            indicator: Indicator::Minor,
            description: "Partially Degraded Service".into(),
            url: "https://status.anthropic.com".into(),
            since: None,
            checked_at: at("2099-01-01T00:00:00Z"),
        });

        let text = facts.text(at("2099-01-01T00:03:00Z"), Some(Path::new("/home/someone")));

        assert!(
            text.contains("Status pages:\n  Claude: Partially Degraded Service\n"),
            "{text}"
        );
    }

    #[test]
    fn a_providers_problem_is_named_as_the_provider_names_itself() {
        let mut facts = diagnostics();
        facts.report.providers.push(ProviderName {
            id: "gemini".into(),
            name: "Gemini CLI".into(),
        });
        facts.report.errors = vec![
            ProviderError {
                provider: "gemini".into(),
                message: "could not read the sign-in from /home/someone/.gemini/settings.json: it \
                          is not a plain file"
                    .into(),
            },
            ProviderError {
                provider: "copilot".into(),
                message: "request failed: GitHub did not answer.".into(),
            },
            // From a daemon older than D-128, which began each message with its provider's id.
            ProviderError {
                provider: "codex".into(),
                message: "codex: request failed: Could not reach ChatGPT.".into(),
            },
        ];

        let text = facts.text(at("2099-01-01T00:03:00Z"), Some(Path::new("/home/someone")));

        assert!(
            text.contains(
                "Problems:\n  \
                 Gemini CLI: could not read the sign-in from ~/.gemini/settings.json: it is not a \
                 plain file\n  \
                 copilot: request failed: GitHub did not answer.\n  \
                 Codex: request failed: Could not reach ChatGPT.\n\
                 Settings in effect:"
            ),
            "{text}"
        );
    }

    #[test]
    fn the_account_s_name_is_hidden_outside_the_home_folder_too() {
        let mut facts = diagnostics();
        facts.data_folder = "/private/tmp/claude-501/-Users-someone--work/data".into();

        let text = facts.text(at("2099-01-01T00:03:00Z"), Some(Path::new("/home/someone")));

        assert!(
            text.contains("Data folder: /private/tmp/claude-501/-Users-[user]--work/data"),
            "{text}"
        );
        assert!(!text.contains("someone"), "{text}");
    }

    /// Doctor's text for a user called `name`, with every place and command in it written for that
    /// user's home folder, as a provider writes them: a place in it is `~`, a command is the shell's
    /// form for it, and the rest is sentences. Nothing in it is outside the home folder, so what
    /// the user is called can change nothing in it (D-134).
    fn text_for_user(name: &str) -> String {
        let home: PathBuf = [MAIN_SEPARATOR_STR, "Users", name].iter().collect();
        let in_home = |parts: &[&str]| {
            parts
                .iter()
                .fold(home.clone(), |place, part| place.join(part))
        };
        let work = in_home(&[".claude-work"]);
        let config = in_home(&["Library", "Application Support", "quotadial", "config.json"]);
        let mut facts = diagnostics();
        facts.data_folder = in_home(&["Library", "Application Support", "quotadial"])
            .display()
            .to_string();
        facts.numbers_from = "nowhere: the QuotaDial daemon is running, but its API at \
                              http://127.0.0.1:47115/v1/status could not be reached; restart it \
                              with `quotadial daemon`."
            .into();
        let snapshot = &mut facts.report.snapshots[0];
        snapshot.account.sign_in_use = text::sign_in_read_from(&text::tilde(
            &in_home(&[".claude", ".credentials.json"]),
            Some(&home),
        ));
        snapshot.error = Some(format!(
            "Anthropic rejected this Claude Code sign-in. Run `CLAUDE_CONFIG_DIR={} claude` to \
             sign in again.",
            text::shell_word(&work, Some(&home))
        ));
        facts.report.errors = [(".codex", "auth.json"), (".gemini", "settings.json")]
            .into_iter()
            .map(|(folder, file)| ProviderError {
                provider: folder.trim_start_matches('.').into(),
                message: format!(
                    "could not read the sign-in from {}",
                    in_home(&[folder, file]).display()
                ),
            })
            .collect();
        facts.report.problems = vec![format!(
            "Run `$env:CLAUDE_CONFIG_DIR = {}; claude` to sign in again.",
            text::powershell_string(&work, Some(&home))
        )];
        facts.settings = vec![
            (
                "extra_config_dirs".into(),
                format!("{}, ~/claude-accounts/*", work.display()),
            ),
            ("history_days".into(), "30".into()),
            ("status_pages".into(), "on".into()),
            (
                "warnings".into(),
                format!(
                    "{} holds webhook addresses, which work like passwords, and other users can \
                     read it; run `chmod 600 {}`.",
                    config.display(),
                    text::shell_word(&config, Some(&home))
                ),
            ),
        ];
        facts.text(at("2099-01-01T00:03:00Z"), Some(&home))
    }

    #[test]
    fn a_user_called_as_a_word_in_the_text_sees_every_command_and_place_as_written() {
        // A user whose name is a word doctor writes: one of a command (`claude`), a folder it names
        // (`codex`, `gemini`, `quotadial`, and macOS's `Support`), a setting (`history`, `status`,
        // `config`) or its daemon. Every place the text names is in the home folder, `~`, and the
        // rest is commands and sentences, so it is the text any other user sees (D-134). The plain
        // replace of the whole word wrote `CLAUDE_CONFIG_DIR=~/'.[user]-work' [user]` for `claude`.
        let neutral = text_for_user("ann");
        for name in [
            "claude",
            "codex",
            "gemini",
            "daemon",
            "history",
            "status",
            "config",
            "quotadial",
            "Support",
        ] {
            assert_eq!(text_for_user(name), neutral, "a user called {name}");
        }
        for held in [
            "CLAUDE_CONFIG_DIR=~/'.claude-work' claude",
            "http://127.0.0.1:47115/v1/status",
            "quotadial.com/pro",
            "status_pages: on",
            "history_days: 30",
        ] {
            assert!(neutral.contains(held), "{held}\n{neutral}");
        }
    }

    #[test]
    fn the_users_name_is_hidden_in_a_place_outside_the_home_folder_whoever_the_user_is() {
        // Where it is a component of a place that is not in the home folder, as the temporary folder
        // Claude Code makes for a working folder is, its name written with `/` as `-`, or another
        // disk's copy of the home folder, even for a user called `claude` or `codex` (D-134). Every
        // place is made from its parts.
        for name in ["ann", "claude", "codex"] {
            let home: PathBuf = [MAIN_SEPARATOR_STR, "Users", name].iter().collect();
            let places = |who: &str| -> (PathBuf, PathBuf) {
                let temporary = [
                    MAIN_SEPARATOR_STR.to_owned(),
                    "private".to_owned(),
                    "tmp".to_owned(),
                    format!("{who}-501"),
                    format!("-Users-{who}--work"),
                    "data".to_owned(),
                ];
                let backup = [MAIN_SEPARATOR_STR, "Volumes", "Backup", "Users", who];
                (
                    temporary.iter().collect(),
                    backup
                        .iter()
                        .collect::<PathBuf>()
                        .join(".keys")
                        .join("cache.json"),
                )
            };
            let (temporary, backup) = places(name);
            let (temporary_hidden, backup_hidden) = places("[user]");
            let mut facts = diagnostics();
            facts.data_folder = temporary.display().to_string();
            facts.report.errors = vec![ProviderError {
                provider: "codex".into(),
                message: format!("could not read the sign-in from {}", backup.display()),
            }];
            // And a sentence that only spells the name, which stays.
            facts.report.problems = vec![format!("run {name} once")];

            let text = facts.text(at("2099-01-01T00:03:00Z"), Some(&home));
            assert!(
                text.contains(&format!("\n  run {name} once\n")),
                "{name}\n{text}"
            );

            assert!(
                text.contains(&format!("Data folder: {}\n", temporary_hidden.display())),
                "{name}\n{text}"
            );
            assert!(
                text.contains(&format!(
                    "Codex: could not read the sign-in from {}\n",
                    backup_hidden.display()
                )),
                "{name}\n{text}"
            );
            assert!(
                !text.contains(&temporary.display().to_string()),
                "{name}\n{text}"
            );
            assert!(
                !text.contains(&backup.display().to_string()),
                "{name}\n{text}"
            );
        }
    }

    #[test]
    fn only_the_home_folder_is_written_with_a_tilde_never_a_longer_name() {
        // With `/Users/ann` as the home folder, `/Users/anna` is another user's folder, which
        // doctor's own plain replace cut to `~a`. Doctor now writes the home folder `~` by the rule
        // every message QuotaDial writes uses (D-129).
        let home: PathBuf = ["/", "Users", "ann"].iter().collect();
        let anna = home.with_file_name("anna");
        let backup: PathBuf = ["/", "Volumes", "Backup", "Users", "ann"].iter().collect();
        let auth = |folder: &Path| folder.join(".codex").join("auth.json");
        let mut facts = diagnostics();
        facts.data_folder = home.join("quotadial").display().to_string();
        facts.report.errors = [&anna, &backup, &home]
            .into_iter()
            .map(|folder| ProviderError {
                provider: "codex".into(),
                message: format!("could not read the sign-in from {}", auth(folder).display()),
            })
            .collect();

        let text = facts.text(at("2099-01-01T00:03:00Z"), Some(&home));

        let in_home = |parts: &[&str]| {
            Path::new("~")
                .join(parts.iter().collect::<PathBuf>())
                .display()
                .to_string()
        };
        assert!(
            text.contains(&format!("Data folder: {}\n", in_home(&["quotadial"]))),
            "{text}"
        );
        // The home folder's own words inside a longer path are not the home folder, and the user's
        // name in them is hidden, as anywhere outside the home folder.
        let hidden: PathBuf = ["/", "Volumes", "Backup", "Users", "[user]"]
            .iter()
            .collect();
        assert!(
            text.contains(&format!(
                "Problems:\n  \
                 Codex: could not read the sign-in from {}\n  \
                 Codex: could not read the sign-in from {}\n  \
                 Codex: could not read the sign-in from {}\n",
                auth(&anna).display(),
                auth(&hidden).display(),
                in_home(&[".codex", "auth.json"])
            )),
            "{text}"
        );
    }

    #[test]
    fn an_email_address_keeps_only_its_first_letters_and_ending() {
        assert_eq!(mask_emails("matt@alleghenyoutdoor.com"), "m***@a***.com");
        assert_eq!(
            mask_emails("to a.b+c@mail.example.co.uk, then"),
            "to a***@m***.uk, then"
        );
        assert_eq!(mask_emails("x@y.z."), "x***@y***.z.");
        for text in [
            "no address",
            "@",
            "a@",
            "@b.com",
            "a@b",
            "user@localhost",
            "1 @ 2",
        ] {
            assert_eq!(mask_emails(text), text, "{text:?}");
        }
        assert_eq!(
            mask_emails("é@é.é"),
            "é@é.é",
            "only ASCII names are addresses"
        );
    }

    proptest! {
        #[test]
        fn no_text_makes_masking_panic_or_leaves_a_whole_address(name in "[a-z]{2,8}", domain in "[a-z]{2,8}", before in ".*", after in "[ ,;]{1}.*") {
            let address = format!("{name}@{domain}.com");
            let masked = mask_emails(&format!("{before} {address}{after}"));
            prop_assert!(!masked.contains(&address));
        }

        #[test]
        fn masking_never_panics(text in ".*") {
            let _ = mask_emails(&text);
        }
    }
}
