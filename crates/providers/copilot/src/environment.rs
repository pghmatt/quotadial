//! Where GitHub CLI keeps `hosts.yml`, found as GitHub CLI finds its config folder (cli/go-gh,
//! `pkg/config/config.go`, `ConfigDir`): the folder `GH_CONFIG_DIR` names, else `gh` in
//! `XDG_CONFIG_HOME`, else on Windows `GitHub CLI` in `%AppData%`, else `~/.config/gh`.

use std::ffi::OsString;
use std::path::PathBuf;

/// The parts of the process environment the provider reads, so tests can supply their own.
#[derive(Debug, Clone, Default)]
pub(crate) struct Environment {
    /// The user's home folder.
    pub(crate) home: Option<PathBuf>,
    /// `GH_CONFIG_DIR`, when it is set.
    pub(crate) gh_config_dir: Option<PathBuf>,
    /// `XDG_CONFIG_HOME`, when it is set.
    pub(crate) xdg_config_home: Option<PathBuf>,
    /// `APPDATA`, read on Windows only, as GitHub CLI reads it.
    pub(crate) app_data: Option<PathBuf>,
}

impl Environment {
    pub(crate) fn from_process() -> Self {
        Self {
            home: std::env::home_dir(),
            gh_config_dir: set("GH_CONFIG_DIR"),
            xdg_config_home: set("XDG_CONFIG_HOME"),
            app_data: if cfg!(windows) { set("APPDATA") } else { None },
        }
    }

    /// GitHub CLI's config folder.
    pub(crate) fn gh_config_dir(&self) -> Option<PathBuf> {
        self.gh_config_dir
            .clone()
            .or_else(|| self.xdg_config_home.as_ref().map(|dir| dir.join("gh")))
            .or_else(|| self.app_data.as_ref().map(|dir| dir.join("GitHub CLI")))
            .or_else(|| {
                self.home
                    .as_ref()
                    .map(|home| home.join(".config").join("gh"))
            })
    }

    /// GitHub CLI's `hosts.yml`, which holds its sign-in when it keeps it in a file.
    pub(crate) fn hosts_file(&self) -> Option<PathBuf> {
        self.gh_config_dir().map(|dir| dir.join("hosts.yml"))
    }
}

/// An environment variable that is set to something, as GitHub CLI reads them.
fn set(name: &str) -> Option<PathBuf> {
    std::env::var_os(name)
        .filter(|value: &OsString| !value.is_empty())
        .map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn github_cli_s_folder_is_found_as_github_cli_finds_it() {
        let home = PathBuf::from("/Users/example");
        let mut env = Environment {
            home: Some(home.clone()),
            ..Environment::default()
        };
        assert_eq!(
            env.hosts_file(),
            Some(home.join(".config").join("gh").join("hosts.yml"))
        );

        env.app_data = Some(PathBuf::from("C:/Users/example/AppData/Roaming"));
        assert_eq!(
            env.gh_config_dir(),
            Some(PathBuf::from("C:/Users/example/AppData/Roaming").join("GitHub CLI"))
        );

        env.xdg_config_home = Some(PathBuf::from("/Users/example/xdg"));
        assert_eq!(
            env.gh_config_dir(),
            Some(PathBuf::from("/Users/example/xdg").join("gh"))
        );

        env.gh_config_dir = Some(PathBuf::from("/Users/example/gh-work"));
        assert_eq!(
            env.hosts_file(),
            Some(PathBuf::from("/Users/example/gh-work").join("hosts.yml")),
            "GH_CONFIG_DIR wins"
        );

        assert_eq!(Environment::default().hosts_file(), None);
    }
}
