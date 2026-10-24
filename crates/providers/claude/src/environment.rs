//! Where Claude Code keeps its sign-in, worked out exactly as Claude Code does.
//!
//! Ported from CodexBar's `Providers/Claude/ClaudeConfigPaths.swift` and checked against Claude
//! Code 2.1.286 itself:
//!
//! - The config directory is `$CLAUDE_CONFIG_DIR`, or `~/.claude` when that is unset or empty.
//! - The credentials file is `.credentials.json` in `$CLAUDE_SECURESTORAGE_CONFIG_DIR` when that
//!   is set (an empty value means `~/.claude`), otherwise in the config directory.
//! - The macOS Keychain item is the generic password `Claude Code-credentials`, filed under the
//!   user name. When `$CLAUDE_SECURESTORAGE_CONFIG_DIR` (not empty) or `$CLAUDE_CONFIG_DIR` is set,
//!   the service name gains `-` and the first 8 hex characters of the SHA-256 of that directory,
//!   NFC-normalized, so every config directory holds its own sign-in.
//! - The account details are in `.config.json` in the config directory when that file exists,
//!   otherwise in `.claude.json` in `$CLAUDE_CONFIG_DIR` or the home directory.

use std::path::{Path, PathBuf};

use quotadial_core::Secret;
use sha2::{Digest, Sha256};
use unicode_normalization::UnicodeNormalization;

pub(crate) const KEYCHAIN_SERVICE: &str = "Claude Code-credentials";

/// Claude Code's Keychain account when the user name has characters it will not pass to
/// `security`.
const FALLBACK_KEYCHAIN_ACCOUNT: &str = "claude-code-user";

/// The environment variables and home directory that decide which Claude Code sign-in is in use.
#[derive(Debug, Clone, Default)]
pub(crate) struct Environment {
    /// The home directory, as Claude Code's `os.homedir()` sees it.
    pub(crate) home: Option<PathBuf>,
    /// `CLAUDE_CONFIG_DIR`, when set and not empty.
    pub(crate) config_dir: Option<String>,
    /// `CLAUDE_SECURESTORAGE_CONFIG_DIR`, when set. Empty is meaningful: it selects the default.
    pub(crate) secure_storage_dir: Option<String>,
    /// `CLAUDE_CODE_OAUTH_TOKEN`, when set and not blank.
    pub(crate) oauth_token: Option<Secret>,
    /// `USER`, when set and not empty.
    pub(crate) user: Option<String>,
}

impl Environment {
    pub(crate) fn from_process() -> Self {
        Self {
            home: std::env::home_dir(),
            config_dir: var("CLAUDE_CONFIG_DIR").filter(|dir| !dir.is_empty()),
            secure_storage_dir: var("CLAUDE_SECURESTORAGE_CONFIG_DIR"),
            oauth_token: var("CLAUDE_CODE_OAUTH_TOKEN")
                .map(|token| token.trim().to_owned())
                .filter(|token| !token.is_empty())
                .map(Secret::new),
            user: var("USER").filter(|user| !user.is_empty()),
        }
    }

    pub(crate) fn default_config_dir(&self) -> Option<PathBuf> {
        self.home.as_ref().map(|home| home.join(".claude"))
    }

    /// What Claude Code would see with `CLAUDE_CONFIG_DIR` set to `dir` and no token in the
    /// environment: the sign-in it keeps for that folder.
    pub(crate) fn for_config_dir(&self, dir: &str) -> Self {
        Self {
            home: self.home.clone(),
            config_dir: Some(dir.to_owned()),
            secure_storage_dir: None,
            oauth_token: None,
            user: self.user.clone(),
        }
    }

    /// What Claude Code would see here without `CLAUDE_CODE_OAUTH_TOKEN`: the sign-in it keeps for
    /// this shell's folder.
    pub(crate) fn without_token(&self) -> Self {
        Self {
            oauth_token: None,
            ..self.clone()
        }
    }

    /// What Claude Code would see with none of its variables set: the sign-in in its default
    /// folder.
    pub(crate) fn default_only(&self) -> Self {
        Self {
            home: self.home.clone(),
            config_dir: None,
            secure_storage_dir: None,
            oauth_token: None,
            user: self.user.clone(),
        }
    }

    /// Whether `CLAUDE_CONFIG_DIR` or `CLAUDE_SECURESTORAGE_CONFIG_DIR` moves this sign-in away from
    /// Claude Code's default folder, and to which folder.
    pub(crate) fn scoped_dir(&self) -> Option<String> {
        match self.secure_storage_dir.as_deref() {
            Some("") => None,
            Some(dir) => Some(nfc(dir)),
            None => self.config_dir.as_deref().map(nfc),
        }
    }

    /// `path` with `~` for the home folder, as people write it.
    pub(crate) fn tilde(&self, path: &Path) -> String {
        quotadial_core::text::tilde(path, self.home.as_deref())
    }

    /// `$CLAUDE_CONFIG_DIR`, or `~/.claude`.
    pub(crate) fn config_dir(&self) -> Option<PathBuf> {
        match &self.config_dir {
            Some(dir) => Some(PathBuf::from(nfc(dir))),
            None => self.default_config_dir(),
        }
    }

    pub(crate) fn credentials_file(&self) -> Option<PathBuf> {
        let dir = match self.secure_storage_dir.as_deref() {
            Some("") => self.default_config_dir(),
            Some(dir) => Some(PathBuf::from(nfc(dir))),
            None => self.config_dir(),
        };
        dir.map(|dir| dir.join(".credentials.json"))
    }

    /// The file whose `oauthAccount` object says who is signed in.
    pub(crate) fn account_config_file(&self) -> Option<PathBuf> {
        let legacy = self.config_dir().map(|dir| dir.join(".config.json"));
        if let Some(legacy) = legacy.filter(|file| file.is_file()) {
            return Some(legacy);
        }
        match &self.config_dir {
            Some(dir) => Some(PathBuf::from(dir).join(".claude.json")),
            None => self.home.as_ref().map(|home| home.join(".claude.json")),
        }
    }

    pub(crate) fn keychain_service(&self) -> String {
        match self.scoped_dir() {
            Some(dir) => format!("{KEYCHAIN_SERVICE}-{}", &sha256_hex(&dir)[..8]),
            None => KEYCHAIN_SERVICE.to_owned(),
        }
    }

    /// The Keychain account Claude Code files its item under, when the user name is known.
    pub(crate) fn keychain_account(&self) -> Option<String> {
        self.user.as_deref().map(|user| {
            let safe = user
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'));
            if safe {
                user
            } else {
                FALLBACK_KEYCHAIN_ACCOUNT
            }
            .to_owned()
        })
    }
}

fn var(name: &str) -> Option<String> {
    std::env::var_os(name).map(|value| value.to_string_lossy().into_owned())
}

fn nfc(text: &str) -> String {
    text.nfc().collect()
}

/// Lowercase hex SHA-256 of `text`.
pub(crate) fn sha256_hex(text: &str) -> String {
    Sha256::digest(text.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env() -> Environment {
        Environment {
            home: Some(PathBuf::from("/Users/example")),
            user: Some("example".into()),
            ..Environment::default()
        }
    }

    #[test]
    fn the_default_login_uses_the_plain_keychain_item() {
        assert_eq!(env().keychain_service(), "Claude Code-credentials");
    }

    #[test]
    fn a_config_dir_gets_its_own_keychain_item() {
        let env = Environment {
            config_dir: Some("/Users/example/.claude-work".into()),
            ..env()
        };

        // printf '%s' /Users/example/.claude-work | shasum -a 256 → dd1118a7…
        assert_eq!(env.keychain_service(), "Claude Code-credentials-dd1118a7");
    }

    #[test]
    fn secure_storage_dir_overrides_the_config_dir_for_the_keychain_item() {
        let scoped = Environment {
            config_dir: Some("/Users/example/.claude-work".into()),
            secure_storage_dir: Some("/Users/example/secure".into()),
            ..env()
        };
        let default = Environment {
            config_dir: Some("/Users/example/.claude-work".into()),
            secure_storage_dir: Some(String::new()),
            ..env()
        };

        assert_eq!(
            scoped.keychain_service(),
            "Claude Code-credentials-4409e8d0"
        );
        assert_eq!(default.keychain_service(), "Claude Code-credentials");
    }

    #[test]
    fn the_keychain_hash_uses_the_nfc_form_of_the_path() {
        let decomposed = Environment {
            config_dir: Some("/Users/cafe\u{301}/.claude".into()),
            ..env()
        };
        let composed = Environment {
            config_dir: Some("/Users/caf\u{e9}/.claude".into()),
            ..env()
        };

        assert_eq!(
            decomposed.keychain_service(),
            "Claude Code-credentials-da474c2a"
        );
        assert_eq!(composed.keychain_service(), decomposed.keychain_service());
    }

    #[test]
    fn the_keychain_account_is_the_user_name_when_it_is_safe() {
        assert_eq!(env().keychain_account().as_deref(), Some("example"));

        let odd = Environment {
            user: Some("first last".into()),
            ..env()
        };
        assert_eq!(odd.keychain_account().as_deref(), Some("claude-code-user"));

        let unknown = Environment {
            user: None,
            ..env()
        };
        assert_eq!(unknown.keychain_account(), None);
    }

    #[test]
    fn the_credentials_file_follows_the_secure_storage_and_config_dirs() {
        assert_eq!(
            env().credentials_file(),
            Some(PathBuf::from("/Users/example/.claude/.credentials.json"))
        );

        let configured = Environment {
            config_dir: Some("/work/claude".into()),
            ..env()
        };
        assert_eq!(
            configured.credentials_file(),
            Some(PathBuf::from("/work/claude/.credentials.json"))
        );

        let secure = Environment {
            secure_storage_dir: Some("/secure".into()),
            ..configured.clone()
        };
        assert_eq!(
            secure.credentials_file(),
            Some(PathBuf::from("/secure/.credentials.json"))
        );

        let secure_default = Environment {
            secure_storage_dir: Some(String::new()),
            ..configured
        };
        assert_eq!(
            secure_default.credentials_file(),
            Some(PathBuf::from("/Users/example/.claude/.credentials.json"))
        );
    }

    #[test]
    fn account_details_come_from_claude_json_unless_a_config_json_exists() {
        let home = tempfile::tempdir().expect("temp dir");
        let config_dir = home.path().join("profile");
        std::fs::create_dir_all(&config_dir).expect("config dir");
        let env = Environment {
            home: Some(home.path().to_path_buf()),
            ..Environment::default()
        };
        let configured = Environment {
            config_dir: Some(config_dir.to_string_lossy().into_owned()),
            ..env.clone()
        };

        assert_eq!(
            env.account_config_file(),
            Some(home.path().join(".claude.json"))
        );
        assert_eq!(
            configured.account_config_file(),
            Some(config_dir.join(".claude.json"))
        );

        std::fs::write(config_dir.join(".config.json"), "{}").expect("legacy config");
        assert_eq!(
            configured.account_config_file(),
            Some(config_dir.join(".config.json"))
        );
    }

    #[test]
    fn no_home_directory_means_no_default_locations() {
        let env = Environment::default();

        assert_eq!(env.config_dir(), None);
        assert_eq!(env.credentials_file(), None);
        assert_eq!(env.account_config_file(), None);
        assert_eq!(env.keychain_service(), "Claude Code-credentials");
    }
}
