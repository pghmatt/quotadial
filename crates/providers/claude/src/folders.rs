//! The Claude Code folders QuotaDial looks in, the account each is signed in to, and the file each
//! keeps its numbers in (the manager's rule 4 on #73, D-145).
//!
//! The folders are listed as the sign-ins were (#34, D-023): the one this shell's
//! `CLAUDE_CONFIG_DIR` names first, or else Claude Code's default folder, `~/.claude`; then the
//! default folder, when it did not come first; then the other `~/.claude…` folders, the account
//! folders Codus keeps in `~/.codus/accounts`, and the folders of the `extra_config_dirs` setting.
//! A folder's account is the `oauthAccount` object in Claude Code's own account file for it
//! (`identity`): its id, address, organisation and plan, which Claude Code writes when it signs in.
//! That file is not its sign-in, which QuotaDial never reads. A folder's numbers are in its file in
//! QuotaDial's data folder (`store`), named after the folder as `CLAUDE_CONFIG_DIR` gave it to
//! Claude Code; since a folder can be given with a separator at its end, or through a link, each
//! way of writing it is looked for, and the newest numbers win (#39).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};

use crate::environment::Environment;
use crate::handover::Handover;
use crate::identity::{self, Identity};
use crate::store;

/// A Claude Code folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Folder {
    /// Claude Code's default folder, `~/.claude`, which it uses when `CLAUDE_CONFIG_DIR` is not set.
    Default,
    /// A folder `CLAUDE_CONFIG_DIR` names, as it is written.
    Named(String),
}

impl Folder {
    /// What Claude Code sees in this folder, with `env`'s home folder.
    fn env(&self, env: &Environment) -> Environment {
        match self {
            Self::Default => env.default_only(),
            Self::Named(dir) => env.for_config_dir(dir),
        }
    }

    /// The folder's path, when it is known: the default folder needs the home folder.
    pub(crate) fn path(&self, env: &Environment) -> Option<PathBuf> {
        self.env(env).config_dir()
    }

    /// The account Claude Code's own account file in this folder names, if it names one.
    pub(crate) fn identity(&self, env: &Environment) -> Option<Identity> {
        self.env(env)
            .account_config_file()
            .and_then(|file| identity::from_account_config(&file))
    }

    /// The name of this folder's file in QuotaDial's data folder, as `quotadial claude status-line`
    /// names it when Claude Code gives it the folder as written here.
    pub(crate) fn file_name(&self) -> String {
        match self {
            Self::Default => store::file_name(None),
            Self::Named(dir) => store::file_name(Some(dir)),
        }
    }

    /// The names its file can have: one for each way the folder can have been given to Claude Code,
    /// the way it is written here first.
    fn file_names(&self) -> Vec<String> {
        match self {
            Self::Default => vec![self.file_name()],
            Self::Named(dir) => {
                let mut names: Vec<String> = Vec::new();
                for spelling in spellings(dir) {
                    let name = store::file_name(Some(&spelling));
                    if !names.contains(&name) {
                        names.push(name);
                    }
                }
                names
            }
        }
    }

    /// The newest numbers this folder's file holds in `data_folder`, when they were handed over, and
    /// the file's name.
    pub(crate) fn handover(&self, data_folder: &Path) -> Option<Kept> {
        self.file_names()
            .into_iter()
            .filter_map(|file| {
                let (handover, handed_over) = store::read(&data_folder.join(&file))?;
                Some(Kept {
                    handover,
                    handed_over,
                    file,
                })
            })
            .max_by_key(|kept| kept.handed_over)
    }
}

/// The numbers a folder's file holds.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Kept {
    pub(crate) handover: Handover,
    /// When they were handed over.
    pub(crate) handed_over: DateTime<Utc>,
    /// The file's name in QuotaDial's data folder.
    pub(crate) file: String,
}

/// The folders [`list`] found.
#[derive(Debug, Default)]
pub(crate) struct Listed {
    /// Each folder, in the order its account comes.
    pub(crate) folders: Vec<Folder>,
    /// `extra_dirs` entries left out because they are not full paths.
    pub(crate) skipped: Vec<PathBuf>,
}

/// The Claude Code folders on this machine: the one `env` names first, then the default folder,
/// then [`candidate_dirs`]. `extra_dirs` adds folders, and a folder ending in `*` adds every folder
/// inside it.
pub(crate) fn list(env: &Environment, extra_dirs: &[PathBuf]) -> Listed {
    let mut folders = Vec::new();
    if let Some(own) = &env.config_dir {
        folders.push(Folder::Named(own.clone()));
    }
    folders.push(Folder::Default);
    let candidates = candidate_dirs(env, extra_dirs);
    for dir in candidates.dirs {
        let folder = Folder::Named(dir);
        if !folders.contains(&folder) {
            folders.push(folder);
        }
    }
    Listed {
        folders,
        skipped: candidates.skipped,
    }
}

/// The folders [`candidate_dirs`] found.
#[derive(Debug, Default)]
pub(crate) struct Candidates {
    /// Full paths, sorted.
    pub(crate) dirs: Vec<String>,
    /// `extra_dirs` entries left out because they are not full paths.
    pub(crate) skipped: Vec<PathBuf>,
}

/// Folders that may be Claude Code folders besides the default one, as `CLAUDE_CONFIG_DIR` would
/// name them: `~/.claude-*` and other `~/.claude…` folders, the account folders Codus keeps in
/// `~/.codus/accounts`, and `extra_dirs`. An `extra_dirs` entry must be a full path or start with
/// `~`: a relative one would mean a different folder for every process, and could never match the
/// file Claude Code's status line writes for it (#39).
pub(crate) fn candidate_dirs(env: &Environment, extra_dirs: &[PathBuf]) -> Candidates {
    let mut dirs = BTreeSet::new();
    let mut skipped = Vec::new();
    if let Some(home) = &env.home {
        dirs.extend(folders_in(home, |name| {
            name.starts_with(".claude") && name != ".claude"
        }));
        // Joined one part at a time, so the path is written as Claude Code would be given it on
        // Windows too, and hashes to the same file name.
        dirs.extend(folders_in(&home.join(".codus").join("accounts"), |_| true));
    }
    for entry in extra_dirs {
        let dir = match (entry.strip_prefix("~"), &env.home) {
            (Ok(rest), Some(home)) => home.join(rest),
            _ => entry.clone(),
        };
        if !dir.is_absolute() {
            skipped.push(entry.clone());
        } else if dir.file_name().is_some_and(|name| name == "*") {
            if let Some(parent) = dir.parent() {
                dirs.extend(folders_in(parent, |_| true));
            }
        } else if dir.is_dir() {
            dirs.insert(dir.display().to_string());
        }
    }
    Candidates {
        dirs: dirs.into_iter().collect(),
        skipped,
    }
}

/// The folders directly inside `parent` whose names pass `keep`, as full paths. A link to a folder
/// counts as a folder.
fn folders_in(parent: &Path, keep: impl Fn(&str) -> bool) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(parent) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter(|entry| keep(&entry.file_name().to_string_lossy()))
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.path().display().to_string())
        .collect()
}

/// The ways Claude Code may have been given `dir`: as written, with and without a trailing
/// separator, and with links resolved, as when the home folder is reached through one.
fn spellings(dir: &str) -> Vec<String> {
    let separator = std::path::MAIN_SEPARATOR;
    let mut spellings: Vec<String> = Vec::new();
    let real = std::fs::canonicalize(dir)
        .ok()
        .map(|real| real.display().to_string());
    for written in std::iter::once(dir.to_owned()).chain(real) {
        let bare = written.trim_end_matches(separator).to_owned();
        for spelling in [written.clone(), bare.clone(), format!("{bare}{separator}")] {
            if !spellings.contains(&spelling) {
                spellings.push(spelling);
            }
        }
    }
    spellings
}

#[cfg(test)]
mod tests {
    use std::path::MAIN_SEPARATOR;

    use serde_json::json;

    use super::*;

    fn env_in(home: &Path) -> Environment {
        Environment {
            home: Some(home.to_path_buf()),
            ..Environment::default()
        }
    }

    /// A Claude Code folder at `name` in `home`, with an account file naming `account_uuid`.
    fn signed_in_folder(home: &Path, name: &str, account_uuid: &str) -> PathBuf {
        let folder = home.join(name);
        std::fs::create_dir_all(&folder).expect("makes the folder");
        std::fs::write(
            folder.join(".claude.json"),
            json!({ "oauthAccount": {
                "accountUuid": account_uuid,
                "emailAddress": "someone@example.com",
                "organizationUuid": "00000000-0000-0000-0000-000000000002"
            } })
            .to_string(),
        )
        .expect("writes account config");
        folder
    }

    #[test]
    fn this_shell_s_folder_comes_first_then_the_default_then_the_others() {
        let home = tempfile::tempdir().expect("temp dir");
        let work = signed_in_folder(
            home.path(),
            ".claude-work",
            "00000000-0000-0000-0000-00000000000a",
        );
        let other = signed_in_folder(
            home.path(),
            ".claude-other",
            "00000000-0000-0000-0000-00000000000b",
        );
        let work = work.display().to_string();
        let other = other.display().to_string();

        let plain = list(&env_in(home.path()), &[]).folders;
        let own = list(
            &Environment {
                config_dir: Some(work.clone()),
                ..env_in(home.path())
            },
            &[],
        )
        .folders;

        assert_eq!(
            plain,
            [
                Folder::Default,
                Folder::Named(other.clone()),
                Folder::Named(work.clone())
            ]
        );
        assert_eq!(
            own,
            [Folder::Named(work), Folder::Default, Folder::Named(other)]
        );
    }

    #[test]
    fn a_folder_s_account_is_the_one_its_own_account_file_names() {
        let home = tempfile::tempdir().expect("temp dir");
        let work = signed_in_folder(
            home.path(),
            ".claude-work",
            "00000000-0000-0000-0000-00000000000a",
        );
        std::fs::write(
            home.path().join(".claude.json"),
            json!({ "oauthAccount": { "accountUuid": "00000000-0000-0000-0000-000000000001" } })
                .to_string(),
        )
        .expect("writes the default folder's account file");
        let env = env_in(home.path());

        let account = |folder: &Folder| folder.identity(&env).map(|identity| identity.account_uuid);

        assert_eq!(
            account(&Folder::Default).as_deref(),
            Some("00000000-0000-0000-0000-000000000001"),
            "the default folder's account file is ~/.claude.json"
        );
        assert_eq!(
            account(&Folder::Named(work.display().to_string())).as_deref(),
            Some("00000000-0000-0000-0000-00000000000a")
        );
        assert_eq!(
            account(&Folder::Named(
                home.path().join("none").display().to_string()
            )),
            None
        );
    }

    #[test]
    fn a_folder_given_with_a_separator_at_its_end_finds_the_file_written_for_it() {
        let temp = tempfile::tempdir().expect("temp dir");
        let work = temp.path().join(".claude-work");
        std::fs::create_dir_all(&work).expect("makes the folder");
        let listed = work.display().to_string();
        // As `CLAUDE_CONFIG_DIR=~/.claude-work/ claude` gives it to Claude Code.
        let given = format!("{listed}{MAIN_SEPARATOR}");

        let names = Folder::Named(listed.clone()).file_names();

        assert_eq!(
            names[0],
            store::file_name(Some(&listed)),
            "as written, first"
        );
        assert!(names.contains(&store::file_name(Some(&given))), "{names:?}");
    }

    #[cfg(unix)]
    #[test]
    fn a_folder_reached_through_a_link_finds_the_file_written_for_its_real_path() {
        let real = tempfile::tempdir().expect("temp dir");
        let work = real.path().join(".claude-work");
        std::fs::create_dir_all(&work).expect("makes the folder");
        let links = tempfile::tempdir().expect("temp dir");
        let home = links.path().join("home");
        std::os::unix::fs::symlink(real.path(), &home).expect("links");
        // Claude Code was given the real path; QuotaDial lists the folder through the link.
        let real_work = std::fs::canonicalize(&work).expect("resolves");

        let names = Folder::Named(home.join(".claude-work").display().to_string()).file_names();

        assert!(
            names.contains(&store::file_name(Some(&real_work.display().to_string()))),
            "{names:?}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_link_to_a_folder_is_searched_like_the_folder() {
        let home = tempfile::tempdir().expect("temp dir");
        let real = signed_in_folder(
            home.path(),
            "claude-accounts-real",
            "00000000-0000-0000-0000-00000000000c",
        );
        std::os::unix::fs::symlink(&real, home.path().join(".claude-linked")).expect("links");

        let dirs = candidate_dirs(&env_in(home.path()), &[]).dirs;

        assert_eq!(
            dirs,
            [home.path().join(".claude-linked").display().to_string()]
        );
    }

    #[test]
    fn a_relative_extra_folder_is_left_out() {
        let home = tempfile::tempdir().expect("temp dir");
        std::fs::create_dir_all(home.path().join("elsewhere")).expect("makes the folder");

        let candidates = candidate_dirs(
            &env_in(home.path()),
            &[PathBuf::from("claude-work"), PathBuf::from("~/elsewhere")],
        );

        assert_eq!(candidates.skipped, [PathBuf::from("claude-work")]);
        assert_eq!(
            candidates.dirs,
            [home.path().join("elsewhere").display().to_string()]
        );
    }
}
