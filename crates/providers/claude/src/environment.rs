//! Where Claude Code keeps its settings and its account file, worked out exactly as Claude Code
//! does.
//!
//! Ported from CodexBar's `Providers/Claude/ClaudeConfigPaths.swift` and checked against Claude
//! Code 2.1.286 itself:
//!
//! - The config directory is `$CLAUDE_CONFIG_DIR`, or `~/.claude` when that is unset or empty.
//! - The account details are in `.config.json` in the config directory when that file exists,
//!   otherwise in `.claude.json` in `$CLAUDE_CONFIG_DIR` or the home directory.
//! - Claude Code names what it keeps for a config directory after the first 8 hex characters of the
//!   SHA-256 of that directory, NFC-normalized, as its sign-in item `Claude Code-credentials-`
//!   and those 8 hex; QuotaDial names its file of a folder's numbers the same way (D-145).
//!
//! Claude Code's sign-in, and `CLAUDE_CODE_OAUTH_TOKEN` and `CLAUDE_SECURESTORAGE_CONFIG_DIR`, which
//! say where to find one, are never read (D-145).

use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use unicode_normalization::UnicodeNormalization;

/// The home directory and `CLAUDE_CONFIG_DIR`, which decide which Claude Code folder is in use.
#[derive(Debug, Clone, Default)]
pub(crate) struct Environment {
    /// The home directory, as Claude Code's `os.homedir()` sees it.
    pub(crate) home: Option<PathBuf>,
    /// `CLAUDE_CONFIG_DIR`, when set and not empty.
    pub(crate) config_dir: Option<String>,
}

impl Environment {
    pub(crate) fn from_process() -> Self {
        Self {
            home: std::env::home_dir(),
            config_dir: std::env::var_os("CLAUDE_CONFIG_DIR")
                .map(|dir| dir.to_string_lossy().into_owned())
                .filter(|dir| !dir.is_empty()),
        }
    }

    pub(crate) fn default_config_dir(&self) -> Option<PathBuf> {
        self.home.as_ref().map(|home| home.join(".claude"))
    }

    /// What Claude Code would see with `CLAUDE_CONFIG_DIR` set to `dir`.
    pub(crate) fn for_config_dir(&self, dir: &str) -> Self {
        Self {
            home: self.home.clone(),
            config_dir: Some(dir.to_owned()),
        }
    }

    /// What Claude Code would see with `CLAUDE_CONFIG_DIR` not set: its default folder.
    pub(crate) fn default_only(&self) -> Self {
        Self {
            home: self.home.clone(),
            config_dir: None,
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
}

/// `text` in Unicode's composed form (NFC), as Claude Code writes a folder's path before it hashes
/// it.
pub(crate) fn nfc(text: &str) -> String {
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
            ..Environment::default()
        }
    }

    #[test]
    fn a_folder_s_file_is_named_after_its_path_in_composed_form_as_claude_code_names_its_item() {
        let decomposed = Environment {
            config_dir: Some("/Users/cafe\u{301}/.claude".into()),
            ..env()
        };
        let composed = Environment {
            config_dir: Some("/Users/caf\u{e9}/.claude".into()),
            ..env()
        };

        assert_eq!(
            crate::store::file_name(decomposed.config_dir.as_deref()),
            "claude-da474c2a.json"
        );
        assert_eq!(
            crate::store::file_name(composed.config_dir.as_deref()),
            crate::store::file_name(decomposed.config_dir.as_deref())
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
        assert_eq!(env.account_config_file(), None);
    }
}
