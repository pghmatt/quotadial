//! What QuotaDial says of an Antigravity account, in #52's plain words: what it does with the
//! sign-in, carrying the exact block to add to Antigravity CLI's settings (the manager's rule (b)(3)
//! on #73, P-020's rule 2), and the one line under an account whose numbers are not current.
//!
//! A time is written as RFC 3339 in UTC, never as "3 min ago", so a line stays the same while the
//! numbers it is about do, and the daemon's history, which keeps a snapshot only when its time,
//! status or line changes, keeps one for it rather than one a minute.

use chrono::{DateTime, SecondsFormat, Utc};

use crate::windows::Amount;
use crate::{FOLDER, ID, command};

/// The command Antigravity CLI's settings name: `quotadial antigravity status-line`.
pub(crate) fn command() -> String {
    format!("quotadial {ID} {}", command::NAME)
}

/// Antigravity CLI's settings, written as its docs write the place.
pub(crate) fn settings() -> String {
    format!("~/{FOLDER}/settings.json")
}

/// The block to add to Antigravity CLI's settings, with `stack_with_default`, so Antigravity CLI's
/// own status line stays and QuotaDial's command shows below it.
pub(crate) fn block() -> String {
    format!(
        r#""statusLine": {{ "type": "command", "command": "{}", "stack_with_default": true }}"#,
        command()
    )
}

/// What QuotaDial does with an Antigravity account's sign-in: nothing. It names the block that
/// hands QuotaDial the numbers.
pub(crate) fn sign_in_use() -> String {
    format!(
        "QuotaDial never reads Antigravity's sign-in. It keeps only the numbers Antigravity CLI's \
         status line hands it once `{}` is in {}.",
        block(),
        settings()
    )
}

/// Before any numbers: what to add, as a new status line or in front of one already there.
pub(crate) fn waiting() -> String {
    format!(
        "No numbers yet: add `{}` to {}, or put `{}` in front of the status line command already \
         there, and they come the next time Antigravity CLI runs.",
        block(),
        settings(),
        command()
    )
}

/// Numbers handed over longer ago than two polls (#52).
pub(crate) fn stale(handed_over: DateTime<Utc>) -> String {
    format!(
        "Antigravity CLI last handed over its numbers at {}: they update when it runs again, and a \
         limit that has reset since is left out until then.",
        timestamp(handed_over)
    )
}

/// A handover that leaves nothing to show: each limit turned off, or reset since.
pub(crate) fn nothing(handed_over: DateTime<Utc>) -> String {
    format!(
        "Antigravity CLI handed over no limit that counts now at {}: each is turned off or has \
         reset since.",
        timestamp(handed_over)
    )
}

/// The numbers handed over now are another account's.
pub(crate) const ANOTHER_ACCOUNT: &str = "Antigravity CLI now hands over the numbers of another \
                                          account, which QuotaDial shows in this one's place at its \
                                          next look.";

/// Each limit in `amounts`, with what is left and when it resets, which no window shows, since
/// Antigravity CLI gives an amount and no share for it (P-023): as the line under an account with
/// no window, or, `beside_windows`, after what QuotaDial does with the sign-in, so none is left out
/// without a word (the manager on #73). A limit's name is Antigravity's text, so it is written in
/// quotes, as a name.
pub(crate) fn amounts(amounts: &[Amount], beside_windows: bool) -> String {
    let limits = amounts
        .iter()
        .map(|amount| {
            let mut limit = format!(
                "\"{}\", {} left",
                amount.label.replace('"', "'"),
                number(amount.left)
            );
            if let Some(at) = amount.resets_at {
                limit.push_str(&format!(", resetting at {}", timestamp(at)));
            }
            limit
        })
        .collect::<Vec<_>>()
        .join("; ");
    let lead = match (beside_windows, amounts.len() == 1) {
        (false, true) => {
            "Antigravity CLI gives an amount left and no share for this limit, so QuotaDial shows \
             no share for it"
        }
        (false, false) => {
            "Antigravity CLI gives an amount left and no share for these limits, so QuotaDial shows \
             no share for them"
        }
        (true, true) => {
            "Antigravity CLI gives an amount left and no share for one more limit, which QuotaDial \
             leaves out of its windows"
        }
        (true, false) => {
            "Antigravity CLI gives an amount left and no share for more limits, which QuotaDial \
             leaves out of its windows"
        }
    };
    format!("{lead}: {limits}.")
}

/// An amount as people write one: `120` or `12.5`, to a tenth, or `1.2e20` when it is that large.
fn number(amount: f64) -> String {
    if amount >= 1e15 {
        format!("{amount:.1e}")
    } else {
        format!("{}", (amount * 10.0).round() / 10.0)
    }
}

/// `at` as RFC 3339 in UTC, to the second.
fn timestamp(at: DateTime<Utc>) -> String {
    at.to_rfc3339_opts(SecondsFormat::Secs, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(rfc3339: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(rfc3339)
            .expect("a time")
            .with_timezone(&Utc)
    }

    fn amount(label: &str, left: f64, resets_at: Option<&str>) -> Amount {
        Amount {
            label: label.into(),
            left,
            resets_at: resets_at.map(at),
        }
    }

    #[test]
    fn the_block_is_the_one_antigravity_cli_s_settings_take() {
        let block = block();

        assert_eq!(
            block,
            r#""statusLine": { "type": "command", "command": "quotadial antigravity status-line", "stack_with_default": true }"#
        );
        let parsed: serde_json::Value =
            serde_json::from_str(&format!("{{ {block} }}")).expect("JSON");
        assert_eq!(parsed["statusLine"]["type"], "command");
        assert_eq!(parsed["statusLine"]["stack_with_default"], true);
        assert_eq!(settings(), "~/.gemini/antigravity-cli/settings.json");
    }

    #[test]
    fn the_sign_in_sentence_and_the_first_line_carry_the_block_and_where_it_goes() {
        for said in [sign_in_use(), waiting()] {
            assert!(said.contains(&format!("`{}`", block())), "{said}");
            assert!(said.contains(&settings()), "{said}");
        }
        assert!(sign_in_use().starts_with("QuotaDial never reads Antigravity's sign-in."));
        assert!(waiting().contains("`quotadial antigravity status-line` in front of"));
    }

    #[test]
    fn a_time_is_written_as_itself_so_a_line_stays_the_same() {
        let handed_over = at("2026-10-25T22:30:00.5Z");

        assert_eq!(
            stale(handed_over),
            "Antigravity CLI last handed over its numbers at 2026-10-25T22:30:00Z: they update \
             when it runs again, and a limit that has reset since is left out until then."
        );
        assert!(nothing(handed_over).contains(" at 2026-10-25T22:30:00Z: "));
    }

    #[test]
    fn each_amount_is_named_with_what_is_left_and_when_it_resets() {
        let one = [amount(
            "Weekly (Gemini)",
            120.0,
            Some("2026-10-31T00:00:00Z"),
        )];
        let two = [
            amount("Weekly (Gemini)", 120.0, Some("2026-10-31T00:00:00Z")),
            amount("Credits \"plus\"", 12.55, None),
        ];

        assert_eq!(
            amounts(&one, false),
            "Antigravity CLI gives an amount left and no share for this limit, so QuotaDial shows \
             no share for it: \"Weekly (Gemini)\", 120 left, resetting at 2026-10-31T00:00:00Z."
        );
        assert_eq!(
            amounts(&two, false),
            "Antigravity CLI gives an amount left and no share for these limits, so QuotaDial \
             shows no share for them: \"Weekly (Gemini)\", 120 left, resetting at \
             2026-10-31T00:00:00Z; \"Credits 'plus'\", 12.6 left."
        );
        assert_eq!(
            amounts(&one, true),
            "Antigravity CLI gives an amount left and no share for one more limit, which QuotaDial \
             leaves out of its windows: \"Weekly (Gemini)\", 120 left, resetting at \
             2026-10-31T00:00:00Z."
        );
        assert!(amounts(&two, true).contains("for more limits, which"));
        assert_eq!(number(0.0), "0");
        assert_eq!(number(1e20), "1.0e20");
    }

    #[test]
    fn every_line_is_one_sentence_or_two_in_plain_words() {
        let names = [
            amount("Weekly (Quota)", 1.0, None),
            amount("Auth credentials", 2.0, Some("2026-10-31T00:00:00Z")),
        ];
        let handed_over = at("2026-10-25T22:30:00Z");
        for said in [
            sign_in_use(),
            waiting(),
            stale(handed_over),
            nothing(handed_over),
            ANOTHER_ACCOUNT.to_owned(),
            amounts(&names, false),
            amounts(&names, true),
            amounts(&names[..1], false),
            amounts(&names[..1], true),
        ] {
            assert!(!said.contains('\n') && said.ends_with('.'), "{said}");
            assert!(
                !said.contains('\u{2014}') && !said.contains('\u{2013}'),
                "{said}"
            );
            // #52's plain words: never "quota", "auth" or "credential" outside a name, even when
            // Antigravity names a limit with one.
            assert_eq!(
                quotadial_core::text::unplain_words(&said),
                Vec::<String>::new(),
                "{said}"
            );
        }
    }
}
