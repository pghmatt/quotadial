//! How the CLI and the MCP server find the running daemon: once its API is listening, the daemon
//! writes `daemon.json` (its process id and port) to the data directory, and it removes the file
//! when it stops. A file left behind by a daemon that crashed only costs a refused connection.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};

use chrono::{DateTime, Utc};
use quotadial_daemon::DataDir;
use serde::{Deserialize, Serialize};

/// What the running daemon says about itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Runtime {
    /// The daemon's process id.
    pub pid: u32,
    /// The port its API listens on, at `127.0.0.1`.
    pub port: u16,
    /// When it started.
    pub started_at: DateTime<Utc>,
}

impl Runtime {
    /// Writes `self` to `dir`, readable only by the user.
    ///
    /// # Errors
    ///
    /// When the file cannot be written.
    pub fn write(&self, dir: &DataDir) -> io::Result<()> {
        let temporary = dir.path().join("daemon.json.tmp");
        let mut options = OpenOptions::new();
        options.create(true).truncate(true).write(true);
        #[cfg(unix)]
        std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
        let mut file = options.open(&temporary)?;
        file.write_all(&serde_json::to_vec(self).map_err(io::Error::other)?)?;
        drop(file);
        fs::rename(temporary, dir.runtime())
    }

    /// What the daemon in `dir` last wrote, if anything.
    #[must_use]
    pub fn read(dir: &DataDir) -> Option<Self> {
        serde_json::from_slice(&fs::read(dir.runtime()).ok()?).ok()
    }

    /// Removes the file, when the daemon stops.
    pub fn remove(dir: &DataDir) {
        // Already gone is fine, and a file left behind only costs a refused connection.
        let _ = fs::remove_file(dir.runtime());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_daemon_says_where_it_listens_until_it_stops() {
        let temp = tempfile::tempdir().expect("temp dir");
        let dir = DataDir::new(temp.path().join("quotadial"));
        dir.create().expect("creates");
        let runtime = Runtime {
            pid: 4242,
            port: 47_115,
            started_at: Utc::now(),
        };

        assert_eq!(Runtime::read(&dir), None);
        runtime.write(&dir).expect("writes");
        assert_eq!(Runtime::read(&dir), Some(runtime));

        Runtime::remove(&dir);
        assert_eq!(Runtime::read(&dir), None);
        Runtime::remove(&dir);
    }
}
