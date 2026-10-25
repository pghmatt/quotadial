//! Keeps tokens out of text QuotaDial prints or saves, such as an error message a provider's server
//! sent back. Tokens are found by their shape, the prefix each vendor documents and the characters
//! after it, and a shape is added before any provider can read such a token, so none can reach a
//! log, an error or a snapshot first (D-036, D-044, D-122).

use std::borrow::Cow;

/// What a token is replaced with.
pub const REDACTED: &str = "[redacted token]";

/// The token formats QuotaDial handles.
const TOKENS: &[Token] = &[
    // Anthropic: OAuth access tokens are `sk-ant-oat01-…` and refresh tokens `sk-ant-ort01-…`.
    Token::new("sk-ant-", Body::Word, 0).in_any_case(),
    // OpenAI API keys. QuotaDial never sends one, but `auth.json` can hold one.
    Token::new("sk-proj-", Body::Word, 0),
    Token::new("sk-svcacct-", Body::Word, 0),
    Token::new("sk-admin-", Body::Word, 0),
    // A JSON Web Token, such as a ChatGPT access token: base64url parts joined by dots, the first
    // always starting `eyJ` (`{"`).
    Token::new("eyJ", Body::Jwt, 0),
    // Google's OAuth access tokens, which Gemini CLI and Antigravity keep: `ya29.` and base64url
    // text that can hold more dots, as `ya29.c.ElpSB…` in the Google Authentication extension's
    // docs (Apigee). Real ones run past 100 characters.
    Token::new("ya29.", Body::Dotted, GOOGLE_BODY),
    // Google's OAuth refresh tokens: `1//` and 43 characters of base64url text in the sample answer
    // of Google Identity's "Using OAuth 2.0 for Web Server Applications".
    Token::new("1//", Body::Word, GOOGLE_BODY),
    // GitHub's tokens, by the prefixes "GitHub's token formats" lists (GitHub Docs, "About
    // authentication to GitHub"): a personal access token (classic), an OAuth access token, a
    // GitHub App's user access, installation access and refresh tokens, and a fine-grained personal
    // access token. They are written in `[A-Za-z0-9_]` and may grow to 255 characters (GitHub's
    // changelog, 2021-03-31); today's have 36 after a classic prefix and 82 after `github_pat_`.
    Token::new("ghp_", Body::Underscored, GITHUB_BODY),
    Token::new("gho_", Body::Underscored, GITHUB_BODY),
    Token::new("ghu_", Body::Underscored, GITHUB_BODY),
    Token::new("ghs_", Body::Underscored, GITHUB_BODY),
    Token::new("ghr_", Body::Underscored, GITHUB_BODY),
    Token::new("github_pat_", Body::Underscored, GITHUB_BODY),
];

/// The fewest characters after `ya29.` or `1//` that make a Google token, so a word or a path that
/// only starts like one, such as `ya29.backup`, is left alone.
const GOOGLE_BODY: usize = 16;

/// The fewest characters after a GitHub prefix that make a GitHub token, so a name such as
/// `ghp_notes` or `gho_project_name` is left alone.
const GITHUB_BODY: usize = 30;

struct Token {
    prefix: &'static str,
    /// Whether the prefix matches in any case.
    any_case: bool,
    /// Which characters follow the prefix.
    body: Body,
    /// The fewest of them a token has.
    min_body: usize,
}

/// Which bytes follow a token's prefix.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Body {
    /// Letters, digits, `-` and `_`.
    Word,
    /// Letters, digits and `_`, the characters of GitHub's tokens.
    Underscored,
    /// Letters, digits, `-`, `_` and `.`, as in Google's access tokens.
    Dotted,
    /// A JSON Web Token's: letters, digits, `-`, `_`, `.` and `=` padding, in at least three parts.
    Jwt,
}

impl Body {
    fn has(self, byte: u8) -> bool {
        byte.is_ascii_alphanumeric()
            || match self {
                Self::Word => matches!(byte, b'-' | b'_'),
                Self::Underscored => byte == b'_',
                Self::Dotted => matches!(byte, b'-' | b'_' | b'.'),
                Self::Jwt => matches!(byte, b'-' | b'_' | b'.' | b'='),
            }
    }

    /// Whether a token's last dot may be the end of a sentence instead.
    fn has_dots(self) -> bool {
        matches!(self, Self::Dotted | Self::Jwt)
    }
}

impl Token {
    const fn new(prefix: &'static str, body: Body, min_body: usize) -> Self {
        Self {
            prefix,
            any_case: false,
            body,
            min_body,
        }
    }

    /// The same token with its prefix matched in any case.
    const fn in_any_case(mut self) -> Self {
        self.any_case = true;
        self
    }

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
                    .take_while(|byte| self.body.has(**byte))
                    .count();
            if self.body.has_dots() {
                // A dot that ends a sentence is not part of the token.
                while end > body && bytes[end - 1] == b'.' {
                    end -= 1;
                }
            }
            // A token is a word of its own and long enough to be one, and a JSON Web Token has at
            // least three parts.
            let starts_a_word = start == 0 || !self.body.has(bytes[start - 1]);
            let whole = end - body >= self.min_body
                && (self.body != Body::Jwt || text[body..end].matches('.').count() >= 2);
            if !starts_a_word || !whole {
                search = body;
                continue;
            }
            return Some((start, end));
        }
        None
    }
}

/// `text` with every token replaced by [`REDACTED`]: Anthropic tokens (`sk-ant-` in any case and
/// the token characters after it), OpenAI API keys (`sk-proj-`, `sk-svcacct-`, `sk-admin-`), JSON
/// Web Tokens (`eyJ` with at least three dot-separated parts), Google's access and refresh tokens
/// (`ya29.` and `1//`, with at least 16 characters after them) and GitHub's tokens (`ghp_`, `gho_`,
/// `ghu_`, `ghs_`, `ghr_` and `github_pat_`, with at least 30). A token starts a word, so a word
/// that merely contains a prefix is left alone.
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

    /// Made-up tokens in each vendor's documented shape, none of them real. Each is written in
    /// pieces, so no secret scanner takes this file for one that holds a token.
    const GOOGLE_ACCESS: &str =
        concat!("ya29", ".a0AfB_byMadeUpAccessToken-ForQuotaDialTests_0123");
    const GOOGLE_SERVICE_ACCESS: &str = concat!("ya29", ".c.ElpSBMadeUp.ServiceAccountToken-0123");
    const GOOGLE_REFRESH: &str = concat!("1/", "/QuotaDialMadeUpRefreshToken-ForTests_0123456789");
    const GITHUB_CLASSIC_BODY: &str = "MadeUpClassicTokenForQuotaDial012345";
    const GITHUB_FINE_GRAINED: &str = concat!(
        "github",
        "_pat_11MADEUP0000000000000000_",
        "MadeUpFineGrainedTokenForQuotaDialTests0123456789abcdefABCDEF"
    );

    #[test]
    fn google_tokens_are_replaced() {
        assert_eq!(
            redact_secrets(&format!("Bearer {GOOGLE_ACCESS} was refused.")),
            "Bearer [redacted token] was refused.",
            "an access token, and the sentence's last dot stays"
        );
        assert_eq!(
            redact_secrets(&format!("token={GOOGLE_SERVICE_ACCESS};")),
            "token=[redacted token];",
            "an access token with dots in it is replaced whole"
        );
        assert_eq!(
            redact_secrets(&format!("\"refresh_token\": \"{GOOGLE_REFRESH}\"")),
            "\"refresh_token\": \"[redacted token]\"",
            "a refresh token"
        );
    }

    #[test]
    fn github_tokens_are_replaced() {
        for prefix in ["ghp_", "gho_", "ghu_", "ghs_", "ghr_"] {
            let token = format!("{prefix}{GITHUB_CLASSIC_BODY}");
            assert_eq!(
                redact_secrets(&format!("token {token}, then more")),
                "token [redacted token], then more",
                "{prefix}"
            );
        }
        assert_eq!(
            redact_secrets(&format!("GH_TOKEN={GITHUB_FINE_GRAINED}")),
            "GH_TOKEN=[redacted token]",
            "a fine-grained token, underscore and all"
        );
        assert_eq!(
            redact_secrets(&format!("ghp_{GITHUB_CLASSIC_BODY}-tail")),
            "[redacted token]-tail",
            "a dash is not part of a GitHub token"
        );
    }

    #[test]
    fn words_and_paths_that_only_look_like_google_or_github_tokens_are_left_alone() {
        for text in [
            "ya29 ya29. ya29.short and ya29.a0AfB_byShort".to_owned(),
            "1//2, a 1// comment, and 1//shortRefresh".to_owned(),
            concat!("https://example.com/v1/", "/itemsWithAVeryLongPathName0123").to_owned(),
            "the ghp_ prefix, ghs_count and gho_project_name".to_owned(),
            "/Users/me/ghp_notes/file.txt and C:\\Users\\ghu_someone\\gh\\hosts.yml".to_owned(),
            "~/.config/gh/hosts.yml and https://github.com/pghmatt/quotadial".to_owned(),
            "github_pat_ and github_pat_TooShortToBeAToken".to_owned(),
            // A whole token's shape inside a longer word.
            format!("xghp_{GITHUB_CLASSIC_BODY}, a{GOOGLE_ACCESS} and a{GOOGLE_REFRESH}"),
        ] {
            assert_eq!(redact_secrets(&text), text);
        }
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
        fn a_google_or_github_token_anywhere_in_text_never_survives(
            before in ".{0,40}",
            token in "ya29\\.[A-Za-z0-9_-]{16,60}(\\.[A-Za-z0-9_-]{1,20})?|1//[A-Za-z0-9_-]{16,80}|gh[pousr]_[A-Za-z0-9]{36,60}|github_pat_[A-Za-z0-9_]{82}",
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
