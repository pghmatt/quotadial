//! Keeps provider tokens out of text QuotaDial prints or saves, such as an error message a
//! provider's server sent back.

use std::borrow::Cow;

/// What a token is replaced with.
pub const REDACTED: &str = "[redacted token]";

/// Prefixes that start a provider token. Anthropic's start `sk-ant-`: OAuth access tokens are
/// `sk-ant-oat01-…` and refresh tokens `sk-ant-ort01-…`.
const TOKEN_PREFIXES: &[&str] = &["sk-ant-"];

/// `text` with every token replaced by [`REDACTED`]. A token is a known prefix, in any case, and
/// the token characters (`A-Z`, `a-z`, `0-9`, `-` and `_`) after it.
#[must_use]
pub fn redact_secrets(text: &str) -> Cow<'_, str> {
    // ASCII lowercasing keeps every byte where it was, so offsets found in `lower` index `text`.
    let lower = text.to_ascii_lowercase();
    let mut redacted = String::new();
    let mut copied = 0;
    while let Some((start, prefix)) = next_prefix(&lower, copied) {
        let rest = &text.as_bytes()[start + prefix.len()..];
        let length = rest
            .iter()
            .take_while(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
            .count();
        redacted.push_str(&text[copied..start]);
        redacted.push_str(REDACTED);
        copied = start + prefix.len() + length;
    }
    if copied == 0 {
        return Cow::Borrowed(text);
    }
    redacted.push_str(&text[copied..]);
    Cow::Owned(redacted)
}

/// The first token prefix in `lower` at or after byte `from`, with where it starts.
fn next_prefix(lower: &str, from: usize) -> Option<(usize, &'static str)> {
    TOKEN_PREFIXES
        .iter()
        .filter_map(|prefix| lower[from..].find(prefix).map(|at| (from + at, *prefix)))
        .min_by_key(|(start, _)| *start)
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
}
