//! GitHub CLI's sign-in, read as a file (the manager's Copilot rules on #73): `hosts.yml` in GitHub
//! CLI's config folder, whose `github.com` entry names the account GitHub CLI is signed in to,
//! `user`, and holds its token, `oauth_token`, when GitHub CLI keeps the token there rather than in
//! the system's keychain, as it does when no keychain is at hand or it signed in with
//! `--insecure-storage` (cli/go-gh, `pkg/config/config.go`). QuotaDial never reads GitHub CLI's
//! keychain item; nor another host's entry, whose usage endpoint it never asks; nor the `users` map
//! GitHub CLI keeps of each account it has signed in to, whose layout #51's live check records
//! (D-127).
//!
//! The file is YAML as GitHub CLI writes it: block mappings indented with spaces, plain or quoted
//! scalars and comments. QuotaDial reads that much without a YAML library and skips any line it does
//! not understand. Nothing it reads, the token least of all, ever reaches a message: a file it cannot
//! use says why in fixed words, and the bytes read are wiped once used.

use std::borrow::Cow;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use quotadial_core::Secret;
use zeroize::Zeroizing;

/// `hosts.yml` is a few hundred bytes; anything this large is not one.
const MAX_BYTES: u64 = 1024 * 1024;

/// The longest token QuotaDial takes from the file. GitHub's tokens may grow to 255 characters.
const MAX_TOKEN_CHARS: usize = 1024;

/// What `hosts.yml` says of github.com.
#[derive(Debug, Default)]
pub(crate) struct GhSignIn {
    /// The account GitHub CLI is signed in to, when `user` is a GitHub login.
    pub(crate) user: Option<String>,
    /// The token, when GitHub CLI keeps it in the file.
    pub(crate) token: Option<Secret>,
}

/// Reads `hosts.yml` at `path`: `Ok(None)` when there is none, and an error, in words that hold
/// nothing from the file, when it cannot be read.
pub(crate) fn read(path: &Path) -> Result<Option<GhSignIn>, String> {
    let metadata = match std::fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(unreadable(&error)),
    };
    // A pipe or a folder in its place is never opened: a pipe could wait for a writer for good.
    if !metadata.is_file() {
        return Err("it is not a plain file".to_owned());
    }
    if metadata.len() > MAX_BYTES {
        return Err("it is larger than a megabyte, which no hosts.yml is".to_owned());
    }
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(unreadable(&error)),
    };
    // Room for the whole file, so the buffer holding the token is never reallocated and a copy
    // left behind.
    let capacity = usize::try_from(metadata.len())
        .unwrap_or(0)
        .saturating_add(1);
    let mut bytes = Zeroizing::new(Vec::with_capacity(capacity));
    file.take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| unreadable(&error))?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err("it is larger than a megabyte, which no hosts.yml is".to_owned());
    }
    let text = std::str::from_utf8(&bytes).map_err(|_| "it is not text".to_owned())?;
    Ok(Some(parse(text)))
}

fn unreadable(error: &io::Error) -> String {
    format!("{error}; check that you can read it")
}

/// What `text`, a `hosts.yml`, says of github.com: the `user` and `oauth_token` of its top-level
/// `github.com` entry, and nothing else.
pub(crate) fn parse(text: &str) -> GhSignIn {
    let mut found = GhSignIn::default();
    let mut in_github = false;
    let mut entry_indent: Option<usize> = None;
    for line in text.lines() {
        let line = line.strip_suffix('\r').unwrap_or(line);
        let content = line.trim_start_matches(' ');
        if content.is_empty() || content.starts_with('#') {
            continue;
        }
        let indent = line.len() - content.len();
        if indent == 0 {
            // A host's entry, a block mapping under its name, or anything else.
            in_github = matches!(
                key_value(content),
                Some((host, None)) if host.eq_ignore_ascii_case("github.com")
            );
            entry_indent = None;
            continue;
        }
        // YAML never indents with tabs; and only the entry's own keys count, not the `users` map
        // nested below them.
        if !in_github || content.starts_with('\t') {
            continue;
        }
        if *entry_indent.get_or_insert(indent) != indent {
            continue;
        }
        let Some((key, Some(value))) = key_value(content) else {
            continue;
        };
        match key.as_ref() {
            "user" => {
                found.user = scalar(value)
                    .filter(|user| is_login(user))
                    .map(Cow::into_owned);
            }
            "oauth_token" => {
                found.token = scalar(value)
                    .map(|token| Secret::new(token.into_owned()))
                    .filter(|token| usable(token.expose()));
            }
            _ => {}
        }
    }
    found
}

/// Whether `login` is a GitHub login: letters, digits and hyphens, at most 39, not starting with a
/// hyphen.
pub(crate) fn is_login(login: &str) -> bool {
    !login.is_empty()
        && login.len() <= 39
        && !login.starts_with('-')
        && login
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}

/// Whether `token` could go in a request header: printable characters with no space.
fn usable(token: &str) -> bool {
    !token.is_empty()
        && token.len() <= MAX_TOKEN_CHARS
        && token.bytes().all(|byte| byte.is_ascii_graphic())
}

/// A mapping line's key, and what follows its colon, if anything but a comment.
fn key_value(content: &str) -> Option<(Cow<'_, str>, Option<&str>)> {
    let (key, rest) = match content.as_bytes().first()? {
        quote @ (b'"' | b'\'') => {
            let (key, after) = quoted(content, *quote)?;
            (key, after.strip_prefix(':')?)
        }
        _ => {
            let colon = content
                .char_indices()
                .find(|(at, char)| {
                    *char == ':'
                        && content[at + 1..]
                            .chars()
                            .next()
                            .is_none_or(char::is_whitespace)
                })?
                .0;
            (
                Cow::Borrowed(content[..colon].trim_end()),
                &content[colon + 1..],
            )
        }
    };
    if !rest.is_empty() && !rest.starts_with(char::is_whitespace) {
        return None;
    }
    let value = rest.trim();
    Some((
        key,
        (!value.is_empty() && !value.starts_with('#')).then_some(value),
    ))
}

/// A scalar value: quoted, or plain up to a comment. None for null, and for anything else YAML can
/// write there, such as a flow mapping, an anchor or a block scalar, which GitHub CLI does not.
fn scalar(value: &str) -> Option<Cow<'_, str>> {
    match value.as_bytes().first()? {
        quote @ (b'"' | b'\'') => {
            let (text, rest) = quoted(value, *quote)?;
            let rest = rest.trim_start();
            (rest.is_empty() || rest.starts_with('#')).then_some(text)
        }
        b'{' | b'[' | b'&' | b'*' | b'!' | b'|' | b'>' | b'%' | b'@' | b'`' => None,
        _ => {
            // A comment starts at a `#` after a space. Borrowed, so a token is never copied outside
            // the buffer that is wiped.
            let end = value
                .char_indices()
                .find(|(at, char)| *char == '#' && value[..*at].ends_with([' ', '\t']))
                .map_or(value.len(), |(at, _)| at);
            let plain = value[..end].trim();
            (!plain.is_empty() && !matches!(plain, "~" | "null" | "Null" | "NULL"))
                .then_some(Cow::Borrowed(plain))
        }
    }
}

/// The text inside the quotes `text` starts with, and what comes after the closing one. A single
/// quote is written twice inside single quotes; double quotes with a backslash in them, YAML's
/// escapes, are not read, since GitHub CLI never needs them.
fn quoted(text: &str, quote: u8) -> Option<(Cow<'_, str>, &str)> {
    let body = &text[1..];
    if quote == b'"' {
        let end = body.find('"')?;
        let inner = &body[..end];
        return (!inner.contains('\\')).then(|| (Cow::Borrowed(inner), &body[end + 1..]));
    }
    let bytes = body.as_bytes();
    let mut owned: Option<String> = None;
    let (mut start, mut at) = (0, 0);
    while at < bytes.len() {
        if bytes[at] == b'\'' {
            if bytes.get(at + 1) == Some(&b'\'') {
                owned
                    .get_or_insert_with(String::new)
                    .push_str(&body[start..=at]);
                at += 2;
                start = at;
                continue;
            }
            let text = match owned {
                Some(mut owned) => {
                    owned.push_str(&body[start..at]);
                    Cow::Owned(owned)
                }
                None => Cow::Borrowed(&body[..at]),
            };
            return Some((text, &body[at + 1..]));
        }
        at += 1;
    }
    None
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// A made-up token in GitHub's OAuth shape, written in pieces so no secret scanner takes this
    /// file for one holding a token.
    pub(crate) fn made_up_token(tail: &str) -> String {
        format!("{}{tail}", concat!("gho_", "MadeUpGitHubCliToken"))
    }

    /// `hosts.yml` as GitHub CLI writes it when it keeps the token in the file, made up: four
    /// spaces of indent, the `users` map of every account it signed in to, and the signed-in
    /// account's `user` and `oauth_token` beside `git_protocol`.
    pub(crate) fn hosts_yml(user: &str, token: Option<&str>) -> String {
        let token_line =
            token.map_or_else(String::new, |token| format!("    oauth_token: {token}\n"));
        let nested = token.map_or_else(String::new, |token| {
            format!("            oauth_token: {token}\n")
        });
        format!(
            "github.com:\n    users:\n        {user}:\n{nested}    git_protocol: https\n{token_line}    \
             user: {user}\n"
        )
    }

    #[test]
    fn the_signed_in_account_and_its_token_are_read_from_github_com_s_entry() {
        let token = made_up_token("ForQuotaDialTests0123456789");
        let found = parse(&hosts_yml("example-user", Some(&token)));

        assert_eq!(found.user.as_deref(), Some("example-user"));
        assert_eq!(
            found.token.as_ref().map(Secret::expose),
            Some(token.as_str())
        );
        assert!(
            !format!("{found:?}").contains("MadeUp"),
            "Debug hides the token"
        );
    }

    #[test]
    fn a_token_github_cli_keeps_in_the_keychain_is_not_in_the_file() {
        let found = parse(&hosts_yml("example-user", None));

        assert_eq!(found.user.as_deref(), Some("example-user"));
        assert!(found.token.is_none());
    }

    #[test]
    fn only_github_com_s_own_keys_count() {
        let token = made_up_token("OnlyGitHubComCounts000000000");
        let text = format!(
            "# GitHub CLI's hosts\n---\nghe.example.com:\n    user: elsewhere-user\n    oauth_token: \
             {}\ngithub.com:   # the public host\n  users:\n    nested-user:\n      oauth_token: \
             nested\n  'user': \"example-user\" # the account\n  oauth_token: '{token}'\r\n\nother: \
             value\n  user: not-this-one\n",
            made_up_token("ForAnotherHost0000000000000")
        );

        let found = parse(&text);

        assert_eq!(found.user.as_deref(), Some("example-user"));
        assert_eq!(
            found.token.as_ref().map(Secret::expose),
            Some(token.as_str())
        );
    }

    #[test]
    fn the_accounts_map_and_hosts_after_github_com_are_never_read() {
        let token = made_up_token("TheSignedInAccounts000000000");
        let others = made_up_token("AnotherAccountsToken00000000");
        let text = format!(
            "github.com:\n    user: example-user\n    oauth_token: {token}\n    users:\n        \
             other-user:\n            oauth_token: {others}\n        user: nested-user\n\
             ghe.example.com:\n    user: elsewhere-user\n    oauth_token: {others}\n"
        );

        let found = parse(&text);

        assert_eq!(found.user.as_deref(), Some("example-user"));
        assert_eq!(
            found.token.as_ref().map(Secret::expose),
            Some(token.as_str()),
            "the signed-in account's own token, not another's from the map or another host"
        );

        // A token only in the map, as GitHub CLI keeps none there for the signed-in account, is
        // no token for it.
        let nested_only = format!(
            "github.com:\n    users:\n        example-user:\n            oauth_token: {others}\n    \
             user: example-user\n"
        );
        assert!(parse(&nested_only).token.is_none());
    }

    #[test]
    fn what_github_cli_never_writes_is_not_read() {
        for text in [
            "github.com: {user: example-user, oauth_token: flow-style}\n",
            "github.com:\n    user: &anchor example-user\n    oauth_token: |\n        block\n",
            "github.com:\n\tuser: tabbed-user\n",
            "github.com:\n    user: \"escaped\\u0041\"\n    oauth_token: \"with\\\"escape\"\n",
            "github.com:\n    user: has spaces\n    oauth_token: has spaces\n",
            "github.com:\n    user: someone@example.com\n    oauth_token: ~\n",
            "github.com:\n    user:\n    oauth_token: null\n",
            "github.com:\n    user: 'unclosed\n",
            "- github.com\n- user: example-user\n",
            "",
        ] {
            let found = parse(text);
            assert_eq!(found.user, None, "{text}");
            assert!(found.token.is_none(), "{text}");
        }
    }

    #[test]
    fn single_quotes_inside_single_quotes_are_written_twice() {
        assert_eq!(
            quoted("'it''s'", b'\'').map(|(text, rest)| (text.into_owned(), rest)),
            Some(("it's".to_owned(), ""))
        );
        assert_eq!(
            scalar("'example-user' # a comment").as_deref(),
            Some("example-user")
        );
        assert_eq!(
            scalar("plain value # a comment").as_deref(),
            Some("plain value")
        );
    }

    #[test]
    fn logins_are_github_s() {
        let (longest, too_long) = ("a".repeat(39), "a".repeat(40));
        let logins: [&str; 5] = ["example-user", "Example", "a", "user123", &longest];
        for login in logins {
            assert!(is_login(login), "{login}");
        }
        let not_logins: [&str; 6] = [
            "",
            "-user",
            "user name",
            "user@example.com",
            &too_long,
            "ünïcode",
        ];
        for not in not_logins {
            assert!(!is_login(not), "{not}");
        }
    }

    #[test]
    fn a_file_that_cannot_be_used_says_why_and_nothing_from_it() {
        let dir = tempfile::tempdir().expect("temp dir");

        assert!(matches!(read(&dir.path().join("hosts.yml")), Ok(None)));
        assert_eq!(
            read(dir.path()).map(|_| ()),
            Err("it is not a plain file".to_owned())
        );

        let large = dir.path().join("large.yml");
        std::fs::write(&large, "x".repeat(1024 * 1024 + 1)).expect("writes");
        assert_eq!(
            read(&large).map(|_| ()),
            Err("it is larger than a megabyte, which no hosts.yml is".to_owned())
        );

        let binary = dir.path().join("binary.yml");
        let secretish = made_up_token("InABinaryFile00000000000000");
        let mut bytes = secretish.clone().into_bytes();
        bytes.extend([0xff, 0xfe]);
        std::fs::write(&binary, bytes).expect("writes");
        let error = read(&binary).map(|_| ()).expect_err("not text");
        assert_eq!(error, "it is not text");
        assert!(!error.contains("MadeUp"));

        let good = dir.path().join("good.yml");
        std::fs::write(&good, hosts_yml("example-user", None)).expect("writes");
        let found = read(&good).expect("reads").expect("a file");
        assert_eq!(found.user.as_deref(), Some("example-user"));
    }

    #[cfg(unix)]
    #[test]
    fn a_pipe_in_its_place_is_never_opened() {
        let dir = tempfile::tempdir().expect("temp dir");
        let pipe = dir.path().join("hosts.yml");
        let made = std::process::Command::new("mkfifo")
            .arg(&pipe)
            .status()
            .expect("mkfifo runs");
        assert!(made.success());

        // Read on another thread: opening a pipe would wait for a writer that never comes.
        let (sender, answer) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = sender.send(read(&pipe).map(|_| ()));
        });
        assert_eq!(
            answer
                .recv_timeout(std::time::Duration::from_secs(5))
                .expect("answered at once, without waiting for a writer"),
            Err("it is not a plain file".to_owned())
        );
    }

    proptest::proptest! {
        #[test]
        fn no_text_makes_the_reader_panic(text in ".*", entry in "( {0,6}[a-z_'\"#:{ -]{0,12}[^\n]{0,20}\n){0,8}") {
            let _ = parse(&text);
            let found = parse(&format!("github.com:\n{entry}"));
            if let Some(user) = &found.user {
                proptest::prop_assert!(is_login(user));
            }
            if let Some(token) = &found.token {
                proptest::prop_assert!(usable(token.expose()));
            }
        }
    }
}
