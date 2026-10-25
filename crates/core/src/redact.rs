//! Keeps provider tokens out of text QuotaDial prints or saves, such as an error message a
//! provider's server sent back.

use std::borrow::Cow;

/// What a token is replaced with.
pub const REDACTED: &str = "[redacted token]";

/// The token formats QuotaDial handles.
const TOKENS: &[Token] = &[
    // Anthropic: OAuth access tokens are `sk-ant-oat01-…` and refresh tokens `sk-ant-ort01-…`.
    Token {
        prefix: "sk-ant-",
        any_case: true,
        dotted: false,
    },
    // OpenAI API keys. QuotaDial never sends one, but `auth.json` can hold one.
    Token {
        prefix: "sk-proj-",
        any_case: false,
        dotted: false,
    },
    Token {
        prefix: "sk-svcacct-",
        any_case: false,
        dotted: false,
    },
    Token {
        prefix: "sk-admin-",
        any_case: false,
        dotted: false,
    },
    // A JSON Web Token, such as a ChatGPT access token: base64url parts joined by dots, the first
    // always starting `eyJ` (`{"`).
    Token {
        prefix: "eyJ",
        any_case: false,
        dotted: true,
    },
];

struct Token {
    prefix: &'static str,
    /// Whether the prefix matches in any case.
    any_case: bool,
    /// Whether the token is a JSON Web Token, with at least three dot-separated parts.
    dotted: bool,
}

impl Token {
    /// The first token of this kind starting at or after byte `from`, as a byte range of `text`.
    /// `lower` is `text` in ASCII lowercase.
    fn find(&self, text: &str, lower: &str, from: usize) -> Option<(usize, usize)> {
        let haystack = if self.any_case { lower } else { text };
        let bytes = text.as_bytes();
        let mut search = from;
        while let Some(at) = haystack[search..].find(self.prefix) {
            let start = search + at;
            let body = start + self.prefix.len();
            let mut end = body
                + bytes[body..]
                    .iter()
                    .take_while(|byte| self.is_token_byte(**byte))
                    .count();
            if self.dotted {
                // A dot that ends a sentence is not part of the token.
                while end > body && bytes[end - 1] == b'.' {
                    end -= 1;
                }
            }
            // A token is a word of its own, and a JSON Web Token has at least three parts.
            let starts_a_word = start == 0 || !self.is_token_byte(bytes[start - 1]);
            if !starts_a_word || (self.dotted && text[body..end].matches('.').count() < 2) {
                search = body;
                continue;
            }
            return Some((start, end));
        }
        None
    }

    fn is_token_byte(&self, byte: u8) -> bool {
        byte.is_ascii_alphanumeric()
            || matches!(byte, b'-' | b'_')
            || (self.dotted && matches!(byte, b'.' | b'='))
    }
}

/// `text` with every token replaced by [`REDACTED`]: Anthropic tokens (`sk-ant-` in any case and
/// the token characters after it), OpenAI API keys (`sk-proj-`, `sk-svcacct-`, `sk-admin-`) and
/// JSON Web Tokens (`eyJ` with at least three dot-separated parts). A token starts a word, so a
/// word that merely contains a prefix is left alone.
#[must_use]
pub fn redact_secrets(text: &str) -> Cow<'_, str> {
    // ASCII lowercasing keeps every byte where it was, so offsets found in `lower` index `text`.
    let lower = text.to_ascii_lowercase();
    let mut redacted = String::new();
    let mut copied = 0;
    while let Some((start, end)) = TOKENS
        .iter()
        .filter_map(|token| token.find(text, &lower, copied))
        .min_by_key(|(start, _)| *start)
    {
        redacted.push_str(&text[copied..start]);
        redacted.push_str(REDACTED);
        copied = end;
    }
    if copied == 0 {
        return Cow::Borrowed(text);
    }
    redacted.push_str(&text[copied..]);
    Cow::Owned(redacted)
}

/// Text from outside that QuotaDial shows or saves, such as a server's error message or a model's
/// name: tokens blanked out, whitespace collapsed to single spaces, and cut to `max` characters
/// with an ellipsis. Tokens are blanked first, so one cut off at the end cannot slip through in
/// part. `None` when nothing is left.
#[must_use]
pub fn outside_text(text: &str, max: usize) -> Option<String> {
    let redacted = redact_secrets(text);
    let line = redacted.split_whitespace().collect::<Vec<_>>().join(" ");
    if line.is_empty() {
        return None;
    }
    Some(match line.char_indices().nth(max) {
        Some((cut, _)) => format!("{}…", &line[..cut]),
        None => line,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_without_a_token_is_left_alone() {
        let text = "Anthropic's usage endpoint answered HTTP 503: Overloaded.";

        assert!(matches!(redact_secrets(text), Cow::Borrowed(same) if same == text));
        assert!(matches!(redact_secrets(""), Cow::Borrowed("")));
    }

    #[test]
    fn every_token_is_replaced_and_the_rest_kept() {
        assert_eq!(
            redact_secrets(
                "invalid token: sk-ant-oat01-AbC_12-3. Refresh with SK-ANT-ort01-xyz, or not"
            ),
            "invalid token: [redacted token]. Refresh with [redacted token], or not"
        );
        assert_eq!(
            redact_secrets("sk-ant-oat01-abc"),
            "[redacted token]",
            "a token that is the whole text"
        );
        assert_eq!(
            redact_secrets("sk-ant-"),
            "[redacted token]",
            "a bare prefix"
        );
    }

    #[test]
    fn text_around_a_token_keeps_its_characters() {
        assert_eq!(
            redact_secrets("café sk-ant-oat01-ünïcode é"),
            "café [redacted token]ünïcode é",
            "a token ends at the first character that cannot be part of one"
        );
    }

    #[test]
    fn json_web_tokens_are_replaced() {
        assert_eq!(
            redact_secrets("Bearer eyJhbGciOiJSUzI1NiJ9.eyJzdWIiOiIxIn0.c2lnbmF0dXJl rejected."),
            "Bearer [redacted token] rejected."
        );
        assert_eq!(
            redact_secrets("the token was eyJhbGciOiJSUzI1NiJ9.eyJzdWIiOiIxIn0.c2ln."),
            "the token was [redacted token].",
            "a sentence's last dot stays"
        );
        assert_eq!(
            redact_secrets("padded eyJhbGciOiJSUzI1NiJ9.eyJzdWIiOiIxIn0=.c2lnbmF0dXJl=="),
            "padded [redacted token]",
            "base64 padding is part of the token"
        );
    }

    #[test]
    fn words_that_only_look_like_tokens_are_left_alone() {
        for text in [
            "eyJ without dots, EYJA.B or eyJ.",
            "eyJhbGci.only-two-parts",
            "call keyJSON.parse.now",
            r"C:\Users\JoeyJ.Smith.Jr\.codex\auth.json",
            "a task-ant-farm and a desk-proj-plan",
        ] {
            assert_eq!(redact_secrets(text), text);
        }
    }

    #[test]
    fn openai_api_keys_are_replaced() {
        assert_eq!(
            redact_secrets("key sk-proj-AbC123_x, sk-svcacct-1 and sk-admin-2"),
            "key [redacted token], [redacted token] and [redacted token]"
        );
    }

    #[test]
    fn outside_text_is_one_short_line_without_tokens() {
        assert_eq!(
            outside_text("  invalid\n token: sk-ant-oat01-x  ", 200).as_deref(),
            Some("invalid token: [redacted token]")
        );
        let long = format!("a\nb {}", "x".repeat(300));
        let line = outside_text(&long, 200).expect("text");
        assert!(line.starts_with("a b "));
        assert!(line.ends_with('…'));
        assert_eq!(line.chars().count(), 201);
        assert_eq!(outside_text(" \n ", 10), None);

        // A token straddling the cut is blanked before the cut.
        let at_the_cut = format!("{} sk-ant-oat01-{}", "x".repeat(195), "A".repeat(60));
        let line = outside_text(&at_the_cut, 200).expect("text");
        assert!(
            !line.contains("sk-ant-") && !line.contains("AAAA"),
            "{line}"
        );
    }

    proptest::proptest! {
        #[test]
        fn no_text_makes_redaction_panic(text in ".*", max in 0_usize..300) {
            let _ = redact_secrets(&text);
            if let Some(line) = outside_text(&text, max) {
                proptest::prop_assert!(!line.contains('\n'));
                proptest::prop_assert!(line.chars().count() <= max + 1);
            }
        }

        #[test]
        fn a_token_anywhere_in_text_never_survives(
            before in ".{0,40}",
            token in "(sk-ant-|SK-ANT-|sk-proj-|sk-svcacct-|sk-admin-)[A-Za-z0-9_-]{20,60}",
            after in ".{0,40}",
        ) {
            let text = format!("{before} {token} {after}");
            proptest::prop_assert!(!redact_secrets(&text).contains(&token));
            proptest::prop_assert!(
                outside_text(&text, 1_000).is_none_or(|line| !line.contains(&token))
            );
        }

        #[test]
        fn a_json_web_token_anywhere_in_text_never_survives(
            before in ".{0,40}",
            parts in proptest::collection::vec("[A-Za-z0-9_-]{4,40}", 3..5),
            after in ".{0,40}",
        ) {
            let token = format!("eyJ{}", parts.join("."));
            let text = format!("{before} {token} {after}");
            proptest::prop_assert!(!redact_secrets(&text).contains(&token));
        }
    }
}
