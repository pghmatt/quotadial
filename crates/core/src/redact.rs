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
    /// Whether the token is made of dot-separated parts. Such a token counts only if it has a dot,
    /// so a word that merely starts with the prefix is left alone.
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
                if !text[body..end].contains('.') {
                    search = body;
                    continue;
                }
            }
            return Some((start, end));
        }
        None
    }

    fn is_token_byte(&self, byte: u8) -> bool {
        byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_') || (self.dotted && byte == b'.')
    }
}

/// `text` with every token replaced by [`REDACTED`]: Anthropic tokens (`sk-ant-` in any case and
/// the token characters after it) and JSON Web Tokens (`eyJ…` with at least one dot).
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
            redact_secrets("eyJ without dots, EYJA.B or eyJ."),
            "eyJ without dots, EYJA.B or eyJ.",
            "only the exact prefix with a dotted body counts"
        );
    }
}
