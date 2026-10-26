//! Every sentence the Copilot provider says, in #52's plain words, each naming the fix where there
//! is one.

use quotadial_core::text;

use crate::Source;

/// What QuotaDial does with the sign-in at `place` (P-020): only reads it; for the Copilot CLI's
/// Keychain item, that macOS may ask first, and that a no is kept.
pub(crate) fn sign_in_use(source: &Source, place: &str) -> String {
    let read = text::sign_in_read_from(place);
    match source {
        Source::GhFile(_) => read,
        Source::Keychain(_) => format!(
            "{read} macOS may ask you to allow that. If you deny it, QuotaDial does not ask again \
             until the Copilot CLI signs in again or QuotaDial starts again."
        ),
    }
}

/// The sign-in where the account was found is gone.
pub(crate) fn signed_out(source: &Source, place: &str) -> String {
    format!(
        "QuotaDial no longer finds {} sign-in in {place}. {}",
        whose(source),
        sign_in_again(source)
    )
}

/// The sign-in where the account was found now holds another GitHub account.
pub(crate) fn different_account(source: &Source, place: &str) -> String {
    let whose = whose(source);
    let mut said = whose.chars();
    let whose = said.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(said).collect()
    });
    format!(
        "{whose} sign-in in {place} is now for another GitHub account, so this one's usage cannot \
         be read there."
    )
}

/// GitHub answered 401.
pub(crate) fn rejected(source: &Source) -> String {
    format!(
        "GitHub rejected {} sign-in. {}",
        whose(source),
        sign_in_again(source)
    )
}

/// The person told macOS not to let QuotaDial read the Copilot CLI's Keychain item.
pub(crate) fn refused(place: &str) -> String {
    format!(
        "macOS did not let QuotaDial read the Copilot CLI's sign-in in {place}, so QuotaDial does \
         not ask again until the Copilot CLI signs in again or QuotaDial starts again."
    )
}

/// The Keychain item changed between finding it and reading it.
pub(crate) fn changed_while_read(place: &str) -> String {
    format!(
        "The Copilot CLI's sign-in in {place} changed while QuotaDial read it. QuotaDial reads it \
         again at the next check."
    )
}

/// The Keychain could not be searched, so a Copilot CLI sign-in may be missing. Like every
/// warning, it leaves the provider out: the report shows it under the provider's own name (D-128).
pub(crate) fn keychain_not_searched(reason: &str) -> String {
    format!("could not look for the Copilot CLI's sign-in in the macOS Keychain: {reason}.")
}

/// GitHub CLI's `hosts.yml` at `place`, as `text::tilde` writes it, could not be read, so its
/// sign-in may be missing. It leaves the provider out, as every warning does (D-128).
pub(crate) fn gh_unreadable(place: &str, reason: &str) -> String {
    format!("could not read GitHub CLI's sign-in in {place}: {reason}.")
}

/// GitHub answered with a status QuotaDial cannot use.
pub(crate) fn failed(status: u16, message: Option<&str>) -> String {
    match message {
        Some(message) => format!(
            "GitHub's Copilot usage endpoint answered HTTP {status}: {}.",
            message.trim_end_matches('.')
        ),
        None => format!("GitHub's Copilot usage endpoint answered HTTP {status}."),
    }
}

/// No sign-in for the account was found, and nothing is known about where it was.
pub(crate) const NOT_FOUND: &str = "QuotaDial no longer finds this account's Copilot sign-in on \
     this machine. Run `gh auth login`, or `copilot` and sign in with /login.";

/// The answer had no allowance to show at all.
pub(crate) const NO_LIMITS: &str = "GitHub reported no Copilot limits for this account.";

/// The answer named another GitHub account than the sign-in's.
pub(crate) const ANOTHER_ACCOUNT: &str =
    "GitHub's answer names another account than the sign-in, so it was not used.";

/// What GitHub gives no share for, in one sentence, or none: allowances it sets none of, such as a
/// Copilot Business seat's AI credits, and those it gives without limit. Each is said as
/// [`crate::usage::Reading`] has it.
pub(crate) fn unshared(no_allowance: &[String], unlimited: &[String]) -> Option<String> {
    let no_share = "so QuotaDial has no share to show for them";
    match (listed(no_allowance), listed(unlimited)) {
        (None, None) => None,
        (Some(none), None) => Some(format!(
            "GitHub sets no allowance of {none} for this account, {no_share}."
        )),
        (None, Some(unlimited)) => Some(format!(
            "GitHub gives this account unlimited {unlimited}, {no_share}."
        )),
        (Some(none), Some(unlimited)) => Some(format!(
            "GitHub sets no allowance of {none} for this account and gives it unlimited \
             {unlimited}, {no_share}."
        )),
    }
}

/// `items` as a sentence lists them: "A", "A and B", "A, B and C".
fn listed(items: &[String]) -> Option<String> {
    match items {
        [] => None,
        [only] => Some(only.clone()),
        [rest @ .., last] => Some(format!("{} and {last}", rest.join(", "))),
    }
}

/// Whose sign-in it is, as the middle of a sentence says it.
fn whose(source: &Source) -> &'static str {
    match source {
        Source::GhFile(_) => "GitHub CLI's",
        Source::Keychain(_) => "the Copilot CLI's",
    }
}

/// How to sign in again with the CLI the sign-in is from.
fn sign_in_again(source: &Source) -> &'static str {
    match source {
        Source::GhFile(_) => "Run `gh auth login` to sign in again.",
        Source::Keychain(_) => "Run `copilot` and sign in again with /login.",
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::path::PathBuf;

    use super::*;

    fn gh() -> Source {
        Source::GhFile(PathBuf::from("/Users/example/.config/gh/hosts.yml"))
    }

    fn keychain() -> Source {
        Source::Keychain(Some("example-user".into()))
    }

    const GH_PLACE: &str = "~/.config/gh/hosts.yml";
    const KEYCHAIN_PLACE: &str = "the macOS Keychain item \"copilot-cli\"";

    /// Every sentence the provider can say, with each place it can name.
    pub(crate) fn every_sentence() -> Vec<String> {
        let mut said = Vec::new();
        for (source, place) in [(gh(), GH_PLACE), (keychain(), KEYCHAIN_PLACE)] {
            said.extend([
                sign_in_use(&source, place),
                signed_out(&source, place),
                different_account(&source, place),
                rejected(&source),
            ]);
        }
        let labels = |labels: &[&str]| {
            labels
                .iter()
                .map(|label| (*label).to_owned())
                .collect::<Vec<_>>()
        };
        said.extend([
            refused(KEYCHAIN_PLACE),
            changed_while_read(KEYCHAIN_PLACE),
            keychain_not_searched("/usr/bin/security exited with status 1"),
            gh_unreadable(GH_PLACE, "it is not text"),
            failed(404, Some("Not Found")),
            failed(502, None),
            NO_LIMITS.to_owned(),
            NOT_FOUND.to_owned(),
            ANOTHER_ACCOUNT.to_owned(),
        ]);
        for (none, unlimited) in [
            (labels(&["AI credits"]), labels(&[])),
            (labels(&[]), labels(&["chat messages", "code completions"])),
            (
                labels(&["premium requests"]),
                labels(&["\"Quota runs\"", "code completions"]),
            ),
        ] {
            said.extend(unshared(&none, &unlimited));
        }
        said
    }

    #[test]
    fn every_sentence_keeps_to_plain_words_and_holds_no_token() {
        for sentence in every_sentence() {
            // #52's plain words: never "quota", "auth" or "credential" outside a name.
            assert_eq!(
                text::unplain_words(&sentence),
                Vec::<String>::new(),
                "{sentence}"
            );
            assert!(
                matches!(
                    quotadial_core::redact_secrets(&sentence),
                    std::borrow::Cow::Borrowed(_)
                ),
                "{sentence}"
            );
            assert!(!sentence.contains('\u{2014}') && !sentence.contains('\u{2013}'));
        }
    }

    #[test]
    fn what_discovery_could_not_look_at_is_said_in_plain_words_without_the_providers_id() {
        let warnings = [
            keychain_not_searched("/usr/bin/security exited with status 1"),
            gh_unreadable(GH_PLACE, "it is not a plain file"),
        ];
        for warning in warnings {
            // The report names the provider, by its own name (D-128).
            assert!(!warning.starts_with(crate::ID), "{warning}");
            // #52's plain words: never "quota", "auth" or "credential" outside a name.
            assert_eq!(
                text::unplain_words(&warning),
                Vec::<String>::new(),
                "{warning}"
            );
        }
        assert_eq!(
            gh_unreadable(GH_PLACE, "it is not a plain file"),
            "could not read GitHub CLI's sign-in in ~/.config/gh/hosts.yml: it is not a plain file."
        );
    }

    #[test]
    fn each_sentence_names_the_cli_and_its_fix() {
        assert_eq!(
            sign_in_use(&gh(), GH_PLACE),
            "QuotaDial reads this account's sign-in from ~/.config/gh/hosts.yml. It only reads it \
             and never changes it."
        );
        assert_eq!(
            sign_in_use(&keychain(), KEYCHAIN_PLACE),
            "QuotaDial reads this account's sign-in from the macOS Keychain item \"copilot-cli\". \
             It only reads it and never changes it. macOS may ask you to allow that. If you deny \
             it, QuotaDial does not ask again until the Copilot CLI signs in again or QuotaDial \
             starts again."
        );
        assert_eq!(
            signed_out(&gh(), GH_PLACE),
            "QuotaDial no longer finds GitHub CLI's sign-in in ~/.config/gh/hosts.yml. Run `gh \
             auth login` to sign in again."
        );
        assert_eq!(
            different_account(&keychain(), KEYCHAIN_PLACE),
            "The Copilot CLI's sign-in in the macOS Keychain item \"copilot-cli\" is now for \
             another GitHub account, so this one's usage cannot be read there."
        );
        assert_eq!(
            rejected(&keychain()),
            "GitHub rejected the Copilot CLI's sign-in. Run `copilot` and sign in again with \
             /login."
        );
        assert_eq!(
            failed(503, Some("Service unavailable.")),
            "GitHub's Copilot usage endpoint answered HTTP 503: Service unavailable."
        );
    }

    #[test]
    fn what_has_no_share_is_said_in_one_sentence() {
        let owned = |labels: &[&str]| {
            labels
                .iter()
                .map(|label| (*label).to_owned())
                .collect::<Vec<_>>()
        };

        assert_eq!(unshared(&[], &[]), None);
        assert_eq!(
            unshared(&owned(&["AI credits"]), &[]).as_deref(),
            Some(
                "GitHub sets no allowance of AI credits for this account, so QuotaDial has no \
                 share to show for them."
            )
        );
        assert_eq!(
            unshared(&[], &owned(&["chat messages", "code completions"])).as_deref(),
            Some(
                "GitHub gives this account unlimited chat messages and code completions, so \
                 QuotaDial has no share to show for them."
            )
        );
        assert_eq!(
            unshared(
                &owned(&["AI credits"]),
                &owned(&["premium requests", "chat messages", "code completions"])
            )
            .as_deref(),
            Some(
                "GitHub sets no allowance of AI credits for this account and gives it unlimited \
                 premium requests, chat messages and code completions, so QuotaDial has no share \
                 to show for them."
            )
        );
    }
}
