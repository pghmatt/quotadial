//! What QuotaDial reads and sends (D-106): one text, `docs/PRIVACY.md`, which the app's Help shows,
//! `scripts/privacy.sh` copies into the README and the website renders, and the hosts it names.
//! Every address QuotaDial connects to on its own is held to [`HOSTS`] by a test in the crate that
//! makes the request, and every host in [`HOSTS`] is named in the text, so a new destination cannot
//! reach the code without reaching the text first.

/// What QuotaDial reads and sends, as `docs/PRIVACY.md` says it.
pub const PRIVACY: &str = include_str!("../../../docs/PRIVACY.md");

/// Every host QuotaDial connects to on its own: OpenAI's usage endpoint for Codex, the providers'
/// status pages, and GitHub for updates. Claude's numbers come from Claude Code's status line, so
/// none of Anthropic's is here (D-145). The webhooks alerts go to are the user's own, set in
/// `config.json`, and the text names them as such.
pub const HOSTS: [&str; 5] = [
    "chatgpt.com",
    "status.claude.com",
    "status.openai.com",
    "github.com",
    "release-assets.githubusercontent.com",
];

/// Whether `url` is on one of [`HOSTS`].
#[must_use]
pub fn named(url: &str) -> bool {
    host(url).is_some_and(|host| HOSTS.iter().any(|named| host.eq_ignore_ascii_case(named)))
}

/// The host of `url`, such as `chatgpt.com` for `https://chatgpt.com/backend-api/wham/usage`.
fn host(url: &str) -> Option<&str> {
    let rest = url.split_once("://")?.1;
    let end = rest.find(['/', ':', '?', '#']).unwrap_or(rest.len());
    let host = &rest[..end];
    (!host.is_empty()).then_some(host)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_text_names_every_host() {
        for host in HOSTS {
            assert!(PRIVACY.contains(&format!("**{host}**")), "{host}");
        }
        assert!(PRIVACY.contains("webhook and Slack addresses you add"));
    }

    #[test]
    fn the_text_is_plain() {
        assert!(PRIVACY.starts_with("# What QuotaDial reads and sends\n"));
        assert!(!PRIVACY.contains('\u{2014}') && !PRIVACY.contains('\u{2013}'));
    }

    #[test]
    fn quotadial_never_connects_to_anthropic() {
        // Claude's numbers come only from what Claude Code's status line hands over, never with
        // Claude Code's sign-in (D-145): no address of Anthropic's is one QuotaDial connects to,
        // and the text names none.
        assert!(!HOSTS.iter().any(|host| host.contains("anthropic")));
        assert!(!named("https://api.anthropic.com/api/oauth/usage"));
        assert!(!PRIVACY.contains("anthropic.com"), "{PRIVACY}");
    }

    #[test]
    fn an_address_is_named_by_its_host_alone() {
        assert!(named("https://chatgpt.com/backend-api/wham/usage"));
        assert!(named("https://STATUS.openai.com"));
        assert!(named(
            "https://github.com:443/pghmatt/quotadial/releases/latest/download/latest.json"
        ));
        assert!(!named("https://chatgpt.com.example.net/usage"));
        assert!(!named("https://example.com/?next=https://github.com"));
        assert!(!named("chatgpt.com"));
        assert!(!named("https:///path"));
    }
}
