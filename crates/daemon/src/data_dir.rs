//! Where the daemon keeps its files: `quotadial/` in the OS data directory
//! (`~/Library/Application Support` on macOS, `$XDG_DATA_HOME` or `~/.local/share` on Linux,
//! `%APPDATA%` on Windows). Unlike the providers' cache, this is QuotaDial's own data: history
//! that should survive a cache clean-up. `QUOTADIAL_DATA_DIR` puts it elsewhere. The example build
//! keeps its own, `quotadial-fake/`, which `QUOTADIAL_FAKE_DATA_DIR` puts elsewhere
//! ([`quotadial_core::copy`], D-117), so it never shares a real copy's lock, token or history.

use std::ffi::OsString;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use quotadial_core::copy;

/// The daemon's folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataDir(PathBuf);

impl DataDir {
    /// `QUOTADIAL_DATA_DIR` when it is set, else `quotadial/` in the OS data directory, if the
    /// operating system names one: in the example build, `QUOTADIAL_FAKE_DATA_DIR` and
    /// `quotadial-fake/` instead.
    #[must_use]
    pub fn from_env() -> Option<Self> {
        Self::chosen(std::env::var_os(copy::DATA_DIR_VARIABLE), dirs::data_dir())
    }

    /// The folder `variable` names when it is set, else this copy's own folder in `system`, the OS
    /// data directory.
    fn chosen(variable: Option<OsString>, system: Option<PathBuf>) -> Option<Self> {
        variable
            .filter(|dir| !dir.is_empty())
            .map(|dir| Self(PathBuf::from(dir)))
            .or_else(|| system.map(|dir| Self(dir.join(copy::FOLDER))))
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

    /// What the cost scan has read of the CLIs' logs (D-097): a cache, rebuilt when deleted.
    #[must_use]
    pub fn costs(&self) -> PathBuf {
        self.0.join("costs.db")
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

    /// The licence key this copy keeps (D-099).
    #[must_use]
    pub fn licence(&self) -> PathBuf {
        self.0.join("licence")
    }

    /// Where the running daemon says which port its API listens on.
    #[must_use]
    pub fn runtime(&self) -> PathBuf {
        self.0.join("daemon.json")
    }
}

/// Writes `bytes` to `path`, readable only by the user, through a temporary file and a rename, so no
/// reader ever sees half of it. Its folder is made, readable only by the user, when it is missing.
///
/// # Errors
///
/// When the folder or the file cannot be written.
pub fn write_private(path: &Path, bytes: &[u8]) -> io::Result<()> {
    if let Some(folder) = path.parent() {
        let mut builder = fs::DirBuilder::new();
        builder.recursive(true);
        #[cfg(unix)]
        std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
        builder.create(folder)?;
    }
    let temporary = path.with_extension("tmp");
    let mut options = OpenOptions::new();
    options.create(true).truncate(true).write(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    let mut file = options.open(&temporary)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    drop(file);
    fs::rename(temporary, path)
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
        assert_eq!(dir.costs(), temp.path().join("quotadial/costs.db"));
        assert_eq!(dir.lock(), temp.path().join("quotadial/daemon.lock"));
        assert_eq!(dir.token(), temp.path().join("quotadial/token"));
        assert_eq!(dir.licence(), temp.path().join("quotadial/licence"));
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

    #[test]
    fn the_folder_is_the_variable_s_or_this_copy_s_own_in_the_system_s_data_directory() {
        let system = PathBuf::from("/data");
        let own = |variable: Option<&str>| {
            DataDir::chosen(variable.map(OsString::from), Some(system.clone()))
                .map(|dir| dir.path().to_owned())
        };

        assert_eq!(own(Some("/elsewhere")), Some(PathBuf::from("/elsewhere")));
        if copy::EXAMPLE {
            assert_eq!(
                own(None),
                Some(system.join("quotadial-fake")),
                "the example build never defaults to a real copy's folder"
            );
        } else {
            assert_eq!(own(None), Some(system.join("quotadial")));
        }
        assert_eq!(own(Some("")), own(None), "an empty variable is not set");
        assert_eq!(DataDir::chosen(None, None), None);
    }

    #[test]
    fn a_private_file_is_whole_and_readable_only_by_the_user() {
        let temp = tempfile::tempdir().expect("temp dir");
        let path = temp.path().join("made/settings.json");

        write_private(&path, b"first").expect("writes");
        write_private(&path, b"second").expect("replaces");

        assert_eq!(fs::read(&path).expect("reads"), b"second");
        assert!(!path.with_extension("tmp").exists());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode =
                |path: &Path| fs::metadata(path).expect("metadata").permissions().mode() & 0o777;
            assert_eq!(mode(&path), 0o600);
            assert_eq!(mode(path.parent().expect("a folder")), 0o700);
        }
    }
}
