//! Where Gemini CLI keeps its sign-in and its settings, so the provider looks where Gemini CLI
//! itself does.
//!
//! From Gemini CLI's source (google-gemini/gemini-cli at cd1b45c, read on 2026-10-25): its home is
//! `GEMINI_CLI_HOME` when that is set to anything, otherwise the user's home folder
//! (`homedir()` in `packages/core/src/utils/paths.ts`), and its folder is `.gemini` in it, holding
//! `oauth_creds.json`, where a Google sign-in is kept by default, and `settings.json`
//! (`Storage.getOAuthCredsPath` and `Storage.getGlobalSettingsPath` in
//! `packages/core/src/config/storage.ts`).

use std::path::PathBuf;

/// The parts of the process environment the provider reads, so tests can supply their own.
#[derive(Debug, Clone, Default)]
pub(crate) struct Environment {
    /// The user's home folder.
    pub(crate) home: Option<PathBuf>,
    /// `GEMINI_CLI_HOME`, when it is set to anything at all, as Gemini CLI reads it.
    pub(crate) gemini_cli_home: Option<PathBuf>,
}

impl Environment {
    pub(crate) fn from_process() -> Self {
        Self {
            home: std::env::home_dir(),
            gemini_cli_home: std::env::var_os("GEMINI_CLI_HOME")
                .filter(|value| !value.is_empty())
                .map(PathBuf::from),
        }
    }

    /// `.gemini` in `GEMINI_CLI_HOME`, or in the home folder.
    fn gemini_dir(&self) -> Option<PathBuf> {
        self.gemini_cli_home
            .as_ref()
            .or(self.home.as_ref())
            .map(|home| home.join(".gemini"))
    }

    /// The file Gemini CLI keeps a Google sign-in in. QuotaDial only checks that it is there.
    pub(crate) fn sign_in_file(&self) -> Option<PathBuf> {
        self.gemini_dir().map(|dir| dir.join("oauth_creds.json"))
    }

    /// Gemini CLI's own settings, which say how it signs in.
    pub(crate) fn settings_file(&self) -> Option<PathBuf> {
        self.gemini_dir().map(|dir| dir.join("settings.json"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gemini_cli_home_wins_over_the_home_folder() {
        let home = PathBuf::from("/Users/example");
        let default = Environment {
            home: Some(home.clone()),
            gemini_cli_home: None,
        };
        assert_eq!(
            default.sign_in_file(),
            Some(home.join(".gemini").join("oauth_creds.json"))
        );
        assert_eq!(
            default.settings_file(),
            Some(home.join(".gemini").join("settings.json"))
        );

        let elsewhere = PathBuf::from("/Volumes/work");
        let moved = Environment {
            home: Some(home),
            gemini_cli_home: Some(elsewhere.clone()),
        };
        assert_eq!(
            moved.sign_in_file(),
            Some(elsewhere.join(".gemini").join("oauth_creds.json"))
        );
        assert_eq!(
            moved.settings_file(),
            Some(elsewhere.join(".gemini").join("settings.json"))
        );

        assert_eq!(Environment::default().sign_in_file(), None);
        assert_eq!(Environment::default().settings_file(), None);
    }
}
