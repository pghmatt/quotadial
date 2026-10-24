//! What support asks for, as one text with nothing secret in it (D-101): `quotadial doctor` prints
//! it and the app's Send feedback attaches it, so a report reads the same from either. Whoever
//! gathers the facts fills in [`Diagnostics`]; the text is built only here.
//!
//! The text holds no token, no email address in full, no home folder by name, no licence id and
//! no webhook address: tokens are redacted, email addresses are cut to their first letters, the
//! home folder is written `~`, and webhooks are named by their host.

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
    /// What the copy is, without the licence's id, such as `QuotaDial Pro until 2027-10-24`.
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
    /// The text, at `now`, with the home folder `home` written `~`.
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
            lines.push(format!(
                "  {} {}{plan}: {}{fetched}",
                account.provider,
                account.display_name,
                text::status_label(snapshot.status)
            ));
            if let Some(sign_in) = &account.sign_in {
                lines.push(format!("    sign-in: {sign_in}"));
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
                report
                    .status_pages
                    .iter()
                    .map(|page| format!("{}: {}", page.provider, page.description))
                    .collect(),
            ),
            (
                "Problems",
                report
                    .errors
                    .iter()
                    .map(|error| error.message.clone())
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
        if let Some(home) = home.map(|home| home.display().to_string())
            && !home.is_empty()
            && home != "/"
        {
            text = text.replace(&home, "~");
        }
        mask_emails(&redact_secrets(&text))
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
    use proptest::prelude::*;

    use super::*;
    use crate::{
        Account, LeftOut, ProFeature, ProviderError, Snapshot, Source, Status, Window, WindowKind,
    };

    fn at(text: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(text)
            .expect("a time")
            .with_timezone(&Utc)
    }

    fn diagnostics() -> Diagnostics {
        let account = Account {
            provider: "claude".into(),
            account_id: "a-1".into(),
            display_name: "matt@example.com".into(),
            plan: Some("max".into()),
            org: None,
            sign_in: Some(
                "the macOS Keychain item \"Claude Code-credentials\" (CLAUDE_CONFIG_DIR /home/someone/.claude-work)"
                    .into(),
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
                    message: "codex: the sign-in in /home/someone/.codex/auth.json for someone@corp.example.org could not be read; token sk-ant-oat01-abcdefghijklmnopqrstuvwxyz0123456789"
                        .into(),
                }],
                status_pages: Vec::new(),
                costs: Vec::new(),
                left_out: vec![LeftOut::feature(ProFeature::CostScan)],
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
             claude m***@e***.com (max): ok, fetched 3 min ago from the API\n    \
             sign-in: the macOS Keychain item \"Claude Code-credentials\" (CLAUDE_CONFIG_DIR ~/.claude-work)\n\
             Left out on QuotaDial Free:\n  \
             The week's cost at API prices is part of QuotaDial Pro: quotadial.com/pro\n\
             Problems:\n  \
             codex: the sign-in in ~/.codex/auth.json for s***@c***.org could not be read; token [redacted token]\n\
             Settings in effect:\n  \
             history_days: 30\n  \
             alert webhooks: hooks.slack.com\n"
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
