//! Where the daemon keeps its files: `quotadial/` in the OS data directory
//! (`~/Library/Application Support` on macOS, `$XDG_DATA_HOME` or `~/.local/share` on Linux,
//! `%APPDATA%` on Windows). Unlike the providers' cache, this is QuotaDial's own data: history
//! that should survive a cache clean-up. `QUOTADIAL_DATA_DIR` puts it elsewhere.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// The daemon's folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataDir(PathBuf);

impl DataDir {
    /// `QUOTADIAL_DATA_DIR` when it is set, else `quotadial/` in the OS data directory, if the
    /// operating system names one.
    #[must_use]
    pub fn from_env() -> Option<Self> {
        std::env::var_os("QUOTADIAL_DATA_DIR")
            .filter(|dir| !dir.is_empty())
            .map(|dir| Self(PathBuf::from(dir)))
            .or_else(|| dirs::data_dir().map(|dir| Self(dir.join("quotadial"))))
    }

    /// The folder at `path`, as in tests.
    #[must_use]
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }

    /// Where the folder is.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.0
    }

    /// Creates the folder, readable only by the user.
    ///
    /// # Errors
    ///
    /// Any error creating it.
    pub fn create(&self) -> io::Result<()> {
        let mut builder = fs::DirBuilder::new();
        builder.recursive(true);
        #[cfg(unix)]
        std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
        builder.create(&self.0)
    }

    /// The history database.
    #[must_use]
    pub fn history(&self) -> PathBuf {
        self.0.join("history.db")
    }

    /// The lock that keeps a second daemon from starting.
    #[must_use]
    pub fn lock(&self) -> PathBuf {
        self.0.join("daemon.lock")
    }

    /// The local API's bearer token.
    #[must_use]
    pub fn token(&self) -> PathBuf {
        self.0.join("token")
    }

    /// Where the running daemon says which port its API listens on.
    #[must_use]
    pub fn runtime(&self) -> PathBuf {
        self.0.join("daemon.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_files_live_in_one_private_folder() {
        let temp = tempfile::tempdir().expect("temp dir");
        let dir = DataDir::new(temp.path().join("quotadial"));

        dir.create().expect("creates");

        assert_eq!(dir.history(), temp.path().join("quotadial/history.db"));
        assert_eq!(dir.lock(), temp.path().join("quotadial/daemon.lock"));
        assert_eq!(dir.token(), temp.path().join("quotadial/token"));
        assert_eq!(dir.runtime(), temp.path().join("quotadial/daemon.json"));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(dir.path())
                .expect("metadata")
                .permissions()
                .mode();
            assert_eq!(mode & 0o777, 0o700);
        }
    }
}
