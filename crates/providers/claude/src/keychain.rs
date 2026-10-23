//! Claude Code's macOS Keychain item, read through `/usr/bin/security` the way Claude Code reads it.
//!
//! Claude Code 2.1.286 writes its sign-in with `security add-generic-password -U -a <user> -s
//! <service> -X <hex>`, which makes `/usr/bin/security` the item's trusted application, and reads it
//! back with `security find-generic-password -a <user> -w -s <service>`. QuotaDial runs that same
//! command, so macOS treats the read like one of Claude Code's own and normally shows no prompt.
//! If macOS does ask, for example because the keychain is locked, the prompt is macOS's own and the
//! read gives up after 30 seconds.
//!
//! CodexBar reads the item through Security.framework from its own process
//! (`Providers/Claude/ClaudeOAuth/ClaudeOAuthCredentials.swift`), which is why it needs a consent
//! setting and prompts after every token refresh. Its experimental `security` reader
//! (`ClaudeOAuthCredentials+SecurityCLIReader.swift`) is the approach used here.

use quotadial_core::Secret;

/// Read access to the keychain holding Claude Code's sign-in.
pub(crate) trait Keychain: Send + Sync {
    /// Looks up an item by its attributes without touching its secret, so macOS never asks for
    /// permission. `Ok(None)` means there is no such item.
    fn find(&self, service: &str, account: Option<&str>) -> Result<Option<ItemInfo>, String>;

    /// Reads an item's secret. `Ok(None)` means there is no such item.
    fn read(&self, service: &str, account: Option<&str>) -> Result<Option<Secret>, String>;
}

/// What QuotaDial knows about a Keychain item without reading its secret.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ItemInfo {
    /// The item's modification time as `security` prints it, such as `20261022205736Z`. Claude Code
    /// rewrites the item whenever it signs in or refreshes its token.
    pub(crate) modified: Option<String>,
}

/// The keychain on this platform: `/usr/bin/security` on macOS, none elsewhere.
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

/// Platforms where Claude Code keeps its sign-in only in `.credentials.json`.
#[cfg_attr(target_os = "macos", allow(dead_code))]
pub(crate) struct NoKeychain;

impl Keychain for NoKeychain {
    fn find(&self, _service: &str, _account: Option<&str>) -> Result<Option<ItemInfo>, String> {
        Ok(None)
    }

    fn read(&self, _service: &str, _account: Option<&str>) -> Result<Option<Secret>, String> {
        Ok(None)
    }
}

/// `/usr/bin/security`, used on macOS only. Elsewhere it is compiled, so its tests run on every
/// platform, but never called.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
mod security_cli {
    use std::process::Command;
    use std::time::Duration;

    use quotadial_core::Secret;

    use super::{ItemInfo, Keychain};
    use crate::process::{self, RunError};

    /// The macOS login keychain, through `/usr/bin/security`.
    pub(super) struct SecurityCli;

    pub(super) const SECURITY: &str = "/usr/bin/security";
    pub(super) const FIND_TIMEOUT: Duration = Duration::from_secs(5);
    pub(super) const READ_TIMEOUT: Duration = Duration::from_secs(30);
    /// `security`'s exit status for errSecItemNotFound.
    pub(super) const ITEM_NOT_FOUND: i32 = 44;
    /// `security`'s exit status when the keychain is locked or cannot ask the user.
    pub(super) const INTERACTION_NOT_ALLOWED: i32 = 36;

    impl Keychain for SecurityCli {
        fn find(&self, service: &str, account: Option<&str>) -> Result<Option<ItemInfo>, String> {
            let finished = process::run(find_command(service, account, false), FIND_TIMEOUT)
                .map_err(|error| run_error(&error, FIND_TIMEOUT))?;
            match finished.status.code() {
                Some(0) => Ok(Some(ItemInfo {
                    modified: modified_attribute(&String::from_utf8_lossy(&finished.stdout)),
                })),
                Some(ITEM_NOT_FOUND) => Ok(None),
                code => Err(exit_error(code)),
            }
        }

        fn read(&self, service: &str, account: Option<&str>) -> Result<Option<Secret>, String> {
            let finished = process::run(find_command(service, account, true), READ_TIMEOUT)
                .map_err(|error| run_error(&error, READ_TIMEOUT))?;
            match finished.status.code() {
                Some(0) => {
                    // The error would carry the bytes, so it is dropped unread.
                    let text = String::from_utf8(finished.stdout)
                        .map_err(|_| "the item does not hold text".to_owned())?;
                    Ok(Some(Secret::new(text.trim_end_matches(['\r', '\n']))))
                }
                Some(ITEM_NOT_FOUND) => Ok(None),
                code => Err(exit_error(code)),
            }
        }
    }

    /// `security find-generic-password [-a <account>] [-w] -s <service>`, in Claude Code's argument
    /// order. Without `-w` it prints only the item's attributes.
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

    /// Finds the modification time in `security`'s attribute listing, from a line such as
    /// `    "mdat"<timedate>=0x32303236…00  "20261022205736Z\000"`.
    pub(super) fn modified_attribute(attributes: &str) -> Option<String> {
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
        READ_TIMEOUT, SECURITY, exit_error, find_command, modified_attribute, run_error,
    };
    use super::*;
    use crate::process::RunError;

    const ATTRIBUTES: &str = r#"keychain: "/Users/example/Library/Keychains/login.keychain-db"
version: 512
class: "genp"
attributes:
    0x00000007 <blob>="Claude Code-credentials"
    "acct"<blob>="example"
    "cdat"<timedate>=0x32303236303931383133313834385A00  "20260918131848Z\000"
    "mdat"<timedate>=0x32303236313032323230353733365A00  "20261022205736Z\000"
    "svce"<blob>="Claude Code-credentials"
"#;

    #[test]
    fn reads_the_modification_time_from_the_attribute_listing() {
        assert_eq!(
            modified_attribute(ATTRIBUTES).as_deref(),
            Some("20261022205736Z")
        );
        assert_eq!(modified_attribute("class: \"genp\"\n"), None);
    }

    #[test]
    fn commands_match_claude_codes_own_reads() {
        let args = |command: &Command| -> Vec<String> {
            command
                .get_args()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect()
        };

        let read = find_command("Claude Code-credentials", Some("example"), true);
        assert_eq!(read.get_program(), SECURITY);
        assert_eq!(
            args(&read),
            [
                "find-generic-password",
                "-a",
                "example",
                "-w",
                "-s",
                "Claude Code-credentials"
            ]
        );

        let find = find_command("Claude Code-credentials-dd1118a7", None, false);
        assert_eq!(
            args(&find),
            [
                "find-generic-password",
                "-s",
                "Claude Code-credentials-dd1118a7"
            ]
        );
    }

    #[test]
    fn failures_name_the_exit_status() {
        assert_eq!(
            exit_error(Some(36)),
            "/usr/bin/security exited with status 36, which usually means the keychain is locked; \
             unlock it and try again"
        );
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
    fn platforms_without_a_keychain_find_nothing() {
        assert_eq!(NoKeychain.find("Claude Code-credentials", None), Ok(None));
        assert_eq!(NoKeychain.read("Claude Code-credentials", None), Ok(None));
    }

    /// Looks up an item that cannot exist with the real `/usr/bin/security`. An attribute lookup
    /// shows no prompt and reads no secret, but tests leave the login keychain alone, and a locked
    /// keychain on a headless machine fails with status 36. So this runs only on request:
    /// `QUOTADIAL_REAL_KEYCHAIN_TESTS=1 cargo test -p quotadial-provider-claude -- --ignored`.
    #[cfg(target_os = "macos")]
    #[test]
    #[ignore = "uses the real login keychain; set QUOTADIAL_REAL_KEYCHAIN_TESTS=1 and pass --ignored"]
    fn security_reports_a_missing_item_as_none() {
        if std::env::var_os("QUOTADIAL_REAL_KEYCHAIN_TESTS").is_none_or(|value| value != "1") {
            return;
        }
        let found = security_cli::SecurityCli.find(
            "QuotaDial test item that does not exist",
            Some("quotadial-test"),
        );

        assert_eq!(found, Ok(None));
    }
}
