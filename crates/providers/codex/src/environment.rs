//! Where the Codex CLI keeps its sign-in and its session logs.
//!
//! Follows CodexBar's `Providers/Codex/CodexOAuth/CodexOAuthCredentials.swift`: everything is in
//! `$CODEX_HOME` when that is set, otherwise in `~/.codex`.

use std::path::PathBuf;

/// The parts of the process environment the provider reads, so tests can supply their own.
#[derive(Debug, Clone, Default)]
pub(crate) struct Environment {
    /// The user's home directory.
    pub(crate) home: Option<PathBuf>,
    /// `CODEX_HOME`, when it is set to something other than blanks.
    pub(crate) codex_home: Option<PathBuf>,
}

impl Environment {
    pub(crate) fn from_process() -> Self {
        Self {
            home: std::env::home_dir(),
            codex_home: std::env::var_os("CODEX_HOME")
                .filter(|value| !value.to_string_lossy().trim().is_empty())
                .map(PathBuf::from),
        }
    }

    /// `$CODEX_HOME`, or `~/.codex`.
    pub(crate) fn codex_dir(&self) -> Option<PathBuf> {
        self.codex_home
            .clone()
            .or_else(|| self.home.as_ref().map(|home| home.join(".codex")))
    }

    /// The Codex CLI's sign-in.
    pub(crate) fn auth_file(&self) -> Option<PathBuf> {
        self.codex_dir().map(|dir| dir.join("auth.json"))
    }

    /// The Codex CLI's session logs, in `YYYY/MM/DD/rollout-*.jsonl` below it.
    pub(crate) fn sessions_dir(&self) -> Option<PathBuf> {
        self.codex_dir().map(|dir| dir.join("sessions"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codex_home_wins_over_the_home_directory() {
        let home = PathBuf::from("/Users/example");
        let default = Environment {
            home: Some(home.clone()),
            codex_home: None,
        };
        assert_eq!(
            default.auth_file(),
            Some(home.join(".codex").join("auth.json"))
        );
        assert_eq!(
            default.sessions_dir(),
            Some(home.join(".codex").join("sessions"))
        );

        let custom = PathBuf::from("/Users/example/work-codex");
        let scoped = Environment {
            home: Some(home),
            codex_home: Some(custom.clone()),
        };
        assert_eq!(scoped.auth_file(), Some(custom.join("auth.json")));

        assert_eq!(Environment::default().auth_file(), None);
    }
}
