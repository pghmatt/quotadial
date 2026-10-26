//! The Copilot CLI's macOS Keychain item, found and read through `/usr/bin/security`: the consent
//! path the Claude provider takes for Claude Code's item (its `keychain.rs`; P-007, D-022), so a
//! prompt from macOS is never one QuotaDial has not explained.
//!
//! - Finding the item reads its attributes only (`security find-generic-password -s copilot-cli`,
//!   without `-w`), which never touches its secret, so macOS never asks. Discovery does only this.
//! - Its secret is read (`security find-generic-password -a <account> -w -s copilot-cli`) only when
//!   a request is about to go out, so at most once every 5 minutes for an account. The program is
//!   macOS's own: if macOS asks whether to allow it, the prompt names the program and the item and
//!   the person decides, and the account's sentence has already said that macOS may ask. A read left
//!   unanswered gives up after 30 seconds, saying to answer the prompt; a read the person refuses is
//!   not tried again while the item is unchanged (the provider keeps that), so QuotaDial never asks
//!   again in the background after a no.
//!
//! The Copilot CLI writes the item itself. GitHub's docs name its service, `copilot-cli`
//! (docs.github.com/en/copilot/how-tos/copilot-cli/set-up-copilot-cli/authenticate-copilot-cli,
//! read on 2026-10-25), but not its account name or which program it trusts, so the account's form,
//! whether macOS asks at all, and the exit status a refusal gives are what #51's live check records
//! (D-127). Both of macOS's refusals, errSecAuthFailed and errSecUserCanceled, are taken as a no.

use quotadial_core::Secret;

/// The service the Copilot CLI keeps its sign-in under in the macOS Keychain.
pub(crate) const SERVICE: &str = "copilot-cli";

/// Read access to the keychain holding the Copilot CLI's sign-in.
pub(crate) trait Keychain: Send + Sync {
    /// Finds an item by its service, reading its attributes only, so macOS never asks for
    /// permission. `Ok(None)` means there is no such item.
    fn find(&self, service: &str) -> Result<Option<Item>, String>;

    /// Reads an item's secret, where macOS may ask whether to allow it. `Ok(None)` means there is
    /// no such item.
    fn read(&self, service: &str, account: Option<&str>) -> Result<Option<Secret>, ReadError>;
}

/// What QuotaDial knows about a Keychain item without reading its secret.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Item {
    /// The item's account attribute, as the Copilot CLI wrote it.
    pub(crate) account: Option<String>,
    /// Its modification time as `security` prints it, such as `20261022205736Z`, which changes
    /// whenever the Copilot CLI writes it again.
    pub(crate) modified: Option<String>,
}

/// Why an item's secret was not read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ReadError {
    /// The person told macOS not to allow it.
    Refused,
    /// Anything else, in words, never holding the secret.
    Failed(String),
}

/// The keychain on this platform: `/usr/bin/security` on macOS, none elsewhere, where the Copilot
/// CLI keeps its sign-in in the system's own store, which QuotaDial does not read.
pub(crate) fn system() -> Box<dyn Keychain> {
    #[cfg(target_os = "macos")]
    {
        Box::new(security_cli::SecurityCli)
    }
    #[cfg(not(target_os = "macos"))]
    {
        Box::new(NoKeychain)
    }
}

/// Platforms with no macOS Keychain.
#[cfg(any(test, not(target_os = "macos")))]
pub(crate) struct NoKeychain;

#[cfg(any(test, not(target_os = "macos")))]
impl Keychain for NoKeychain {
    fn find(&self, _service: &str) -> Result<Option<Item>, String> {
        Ok(None)
    }

    fn read(&self, _service: &str, _account: Option<&str>) -> Result<Option<Secret>, ReadError> {
        Ok(None)
    }
}

/// What an item's account attribute says of the GitHub account it holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Holder {
    /// A github.com account, by its login.
    Login(String),
    /// Nothing QuotaDial can tell a login from.
    Unnamed,
    /// An account on another host than github.com, whose usage endpoint QuotaDial never asks.
    Elsewhere,
}

/// Whom an item whose account attribute is `account` belongs to: a login on its own, or a host and
/// the login after its last colon, such as `https://github.com:octocat`, the github.com one only.
pub(crate) fn holder(account: Option<&str>) -> Holder {
    let Some(account) = account.map(str::trim) else {
        return Holder::Unnamed;
    };
    if crate::hosts::is_login(account) {
        return Holder::Login(account.to_owned());
    }
    match account.rsplit_once(':') {
        Some((host, login)) if crate::hosts::is_login(login) => {
            let host = host
                .trim_start_matches("https://")
                .trim_start_matches("http://")
                .trim_end_matches('/');
            if host.eq_ignore_ascii_case("github.com") {
                Holder::Login(login.to_owned())
            } else {
                Holder::Elsewhere
            }
        }
        _ => Holder::Unnamed,
    }
}

/// `/usr/bin/security`, used on macOS only. Elsewhere it is compiled, so its tests run on every
/// platform, but never called.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
mod security_cli {
    use std::process::Command;
    use std::time::Duration;

    use quotadial_core::Secret;
    use zeroize::Zeroize;

    use super::{Item, Keychain, ReadError};
    use crate::process::{self, RunError};

    /// The macOS login keychain, through `/usr/bin/security`.
    pub(super) struct SecurityCli;

    pub(super) const SECURITY: &str = "/usr/bin/security";
    pub(super) const FIND_TIMEOUT: Duration = Duration::from_secs(5);
    /// Long enough for a person to answer macOS's prompt.
    pub(super) const READ_TIMEOUT: Duration = Duration::from_secs(30);
    /// `security`'s exit status for errSecItemNotFound.
    pub(super) const ITEM_NOT_FOUND: i32 = 44;
    /// `security`'s exit status when the keychain is locked or cannot ask the user.
    pub(super) const INTERACTION_NOT_ALLOWED: i32 = 36;
    /// `security`'s exit statuses when the person refused: errSecAuthFailed and
    /// errSecUserCanceled, each its error's low byte, as 44 and 36 are.
    pub(super) const REFUSED: [i32; 2] = [51, 128];

    impl Keychain for SecurityCli {
        fn find(&self, service: &str) -> Result<Option<Item>, String> {
            let finished = process::run(find_command(service, None, false), FIND_TIMEOUT)
                .map_err(|error| run_error(&error, FIND_TIMEOUT))?;
            match finished.status.code() {
                Some(0) => Ok(Some(item(&String::from_utf8_lossy(&finished.stdout)))),
                Some(ITEM_NOT_FOUND) => Ok(None),
                code => Err(exit_error(code)),
            }
        }

        fn read(&self, service: &str, account: Option<&str>) -> Result<Option<Secret>, ReadError> {
            let finished = process::run(find_command(service, account, true), READ_TIMEOUT)
                .map_err(|error| ReadError::Failed(run_error(&error, READ_TIMEOUT)))?;
            read_outcome(finished.status.code(), finished.stdout)
        }
    }

    /// What a read that ended with `code`, having printed `stdout`, gives.
    pub(super) fn read_outcome(
        code: Option<i32>,
        stdout: Vec<u8>,
    ) -> Result<Option<Secret>, ReadError> {
        match code {
            Some(0) => {
                let mut text = match String::from_utf8(stdout) {
                    Ok(text) => text,
                    Err(error) => {
                        error.into_bytes().zeroize();
                        return Err(ReadError::Failed("the item does not hold text".to_owned()));
                    }
                };
                // Trimmed in place, so no copy of the secret is left behind.
                let length = text.trim_end_matches(['\r', '\n']).len();
                text.truncate(length);
                Ok(Some(Secret::new(text)))
            }
            Some(ITEM_NOT_FOUND) => Ok(None),
            Some(code) if REFUSED.contains(&code) => Err(ReadError::Refused),
            code => {
                let mut stdout = stdout;
                stdout.zeroize();
                Err(ReadError::Failed(exit_error(code)))
            }
        }
    }

    /// `security find-generic-password [-a <account>] [-w] -s <service>`. Without `-w` it prints
    /// only the item's attributes.
    pub(super) fn find_command(service: &str, account: Option<&str>, with_secret: bool) -> Command {
        let mut command = Command::new(SECURITY);
        command.arg("find-generic-password");
        if let Some(account) = account {
            command.args(["-a", account]);
        }
        if with_secret {
            command.arg("-w");
        }
        command.args(["-s", service]);
        command
    }

    pub(super) fn run_error(error: &RunError, timeout: Duration) -> String {
        match error {
            RunError::Launch(error) => format!("could not run {SECURITY}: {error}"),
            RunError::TimedOut => format!(
                "{SECURITY} did not answer within {} seconds; if macOS is asking whether to allow \
                 access, answer it and try again",
                timeout.as_secs()
            ),
        }
    }

    pub(super) fn exit_error(code: Option<i32>) -> String {
        match code {
            Some(INTERACTION_NOT_ALLOWED) => format!(
                "{SECURITY} exited with status {INTERACTION_NOT_ALLOWED}, which usually means the \
                 keychain is locked; unlock it and try again"
            ),
            Some(code) => format!("{SECURITY} exited with status {code}"),
            None => format!("{SECURITY} was stopped by a signal"),
        }
    }

    /// The item `security` printed the attributes of.
    pub(super) fn item(attributes: &str) -> Item {
        Item {
            account: blob_attribute(attributes, "acct"),
            modified: modified_attribute(attributes),
        }
    }

    /// A text attribute in `security`'s listing, from a line such as `    "acct"<blob>="text"`.
    fn blob_attribute(attributes: &str, name: &str) -> Option<String> {
        let marker = format!("\"{name}\"<blob>=\"");
        let line = attributes
            .lines()
            .find(|line| line.trim_start().starts_with(&marker))?;
        let value = line.trim_start().strip_prefix(&marker)?.strip_suffix('"')?;
        (!value.is_empty()).then(|| value.to_owned())
    }

    /// The modification time in `security`'s listing, from a line such as
    /// `    "mdat"<timedate>=0x32303236…00  "20261022205736Z\000"`.
    fn modified_attribute(attributes: &str) -> Option<String> {
        let line = attributes
            .lines()
            .find(|line| line.trim_start().starts_with("\"mdat\""))?;
        let quoted = line.rsplit('"').nth(1)?;
        let value = quoted.trim_end_matches("\\000");
        (!value.is_empty()).then(|| value.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use super::security_cli::{
        READ_TIMEOUT, SECURITY, exit_error, find_command, item, read_outcome, run_error,
    };
    use super::*;
    use crate::process::RunError;

    /// What `security find-generic-password -s copilot-cli` prints, made up.
    const ATTRIBUTES: &str = r#"keychain: "/Users/example/Library/Keychains/login.keychain-db"
version: 512
class: "genp"
attributes:
    0x00000007 <blob>="copilot-cli"
    0x00000008 <blob>=<NULL>
    "acct"<blob>="https://github.com:example-user"
    "cdat"<timedate>=0x32303236303931383133313834385A00  "20260918131848Z\000"
    "mdat"<timedate>=0x32303236313032323230353733365A00  "20261022205736Z\000"
    "svce"<blob>="copilot-cli"
"#;

    fn args(command: &Command) -> Vec<String> {
        command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn finding_the_item_reads_its_attributes_and_never_its_secret() {
        let find = find_command(SERVICE, None, false);
        assert_eq!(find.get_program(), SECURITY);
        assert_eq!(
            args(&find),
            ["find-generic-password", "-s", "copilot-cli"],
            "no -w and no -g, so no secret is printed and macOS never asks"
        );

        let read = find_command(SERVICE, Some("https://github.com:example-user"), true);
        assert_eq!(
            args(&read),
            [
                "find-generic-password",
                "-a",
                "https://github.com:example-user",
                "-w",
                "-s",
                "copilot-cli"
            ]
        );
    }

    #[test]
    fn an_items_account_and_modification_time_come_from_its_attributes() {
        assert_eq!(
            item(ATTRIBUTES),
            Item {
                account: Some("https://github.com:example-user".into()),
                modified: Some("20261022205736Z".into()),
            }
        );
        assert_eq!(
            item("class: \"genp\"\nattributes:\n    \"acct\"<blob>=<NULL>\n"),
            Item {
                account: None,
                modified: None
            }
        );
    }

    #[test]
    fn a_read_says_whether_the_person_refused_it() {
        // Written in pieces, so no secret scanner takes this file for one holding a token.
        let made_up = concat!("gho_", "MadeUpCopilotCliTokenForQuotaDial0123");
        let secret = read_outcome(Some(0), format!("{made_up}\n").into_bytes())
            .expect("read")
            .expect("an item");
        assert_eq!(secret.expose(), made_up, "the newline is trimmed");

        assert!(matches!(read_outcome(Some(44), Vec::new()), Ok(None)));
        for refused in [51, 128] {
            assert!(
                matches!(
                    read_outcome(Some(refused), Vec::new()),
                    Err(ReadError::Refused)
                ),
                "{refused}"
            );
        }
        assert!(matches!(
            read_outcome(Some(36), Vec::new()),
            Err(ReadError::Failed(reason)) if reason.contains("keychain is locked")
        ));
        assert!(matches!(
            read_outcome(None, Vec::new()),
            Err(ReadError::Failed(reason)) if reason.contains("stopped by a signal")
        ));
        assert!(matches!(
            read_outcome(Some(0), vec![0xff, 0xfe]),
            Err(ReadError::Failed(reason)) if reason == "the item does not hold text"
        ));
    }

    #[test]
    fn failures_name_the_exit_status_and_an_unanswered_prompt() {
        assert_eq!(
            exit_error(Some(1)),
            "/usr/bin/security exited with status 1"
        );
        assert_eq!(
            run_error(&RunError::TimedOut, READ_TIMEOUT),
            "/usr/bin/security did not answer within 30 seconds; if macOS is asking whether to \
             allow access, answer it and try again"
        );
    }

    #[test]
    fn an_items_account_names_a_github_com_login_or_no_one() {
        assert_eq!(
            holder(Some("example-user")),
            Holder::Login("example-user".into())
        );
        assert_eq!(
            holder(Some("https://github.com:Example-User")),
            Holder::Login("Example-User".into())
        );
        assert_eq!(
            holder(Some("github.com:example-user")),
            Holder::Login("example-user".into())
        );
        assert_eq!(
            holder(Some("https://octocorp.ghe.com:example-user")),
            Holder::Elsewhere,
            "only github.com's endpoint is ever asked"
        );
        for unnamed in [
            None,
            Some(""),
            Some("someone@example.com"),
            Some("https://github.com:"),
            Some("-leading-hyphen"),
            Some("a name with spaces"),
        ] {
            assert_eq!(holder(unnamed), Holder::Unnamed, "{unnamed:?}");
        }
    }

    #[test]
    fn platforms_without_a_keychain_find_nothing() {
        assert_eq!(NoKeychain.find(SERVICE), Ok(None));
        assert!(matches!(NoKeychain.read(SERVICE, None), Ok(None)));
    }

    proptest::proptest! {
        #[test]
        fn no_listing_or_account_makes_the_reader_panic(listing in ".*", account in ".*") {
            let _ = item(&listing);
            let _ = holder(Some(&account));
        }
    }
}
