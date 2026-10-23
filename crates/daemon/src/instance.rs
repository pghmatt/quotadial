//! Only one daemon runs at a time: it holds a lock on a file in the data directory for as long as
//! it runs. The operating system releases the lock when the process ends, however it ends.

use std::fs::{File, OpenOptions, TryLockError};
use std::io;

use crate::DataDir;

/// The running daemon's claim on its data directory, held until dropped.
#[derive(Debug)]
pub struct Instance {
    _lock: File,
}

/// Why the daemon could not start.
#[derive(Debug)]
pub enum InstanceError {
    /// Another daemon already holds the lock.
    AlreadyRunning,
    /// The data directory or its lock file could not be used.
    Io(io::Error),
}

impl std::fmt::Display for InstanceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyRunning => formatter.write_str("a QuotaDial daemon is already running"),
            Self::Io(error) => write!(formatter, "{error}"),
        }
    }
}

impl Instance {
    /// Claims `dir` for this daemon, creating it if needed.
    ///
    /// # Errors
    ///
    /// [`InstanceError::AlreadyRunning`] while another daemon holds the claim.
    pub fn acquire(dir: &DataDir) -> Result<Self, InstanceError> {
        dir.create().map_err(InstanceError::Io)?;
        let mut options = OpenOptions::new();
        options.create(true).truncate(false).write(true);
        #[cfg(unix)]
        std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
        let file = options.open(dir.lock()).map_err(InstanceError::Io)?;
        match file.try_lock() {
            Ok(()) => Ok(Self { _lock: file }),
            Err(TryLockError::WouldBlock) => Err(InstanceError::AlreadyRunning),
            Err(TryLockError::Error(error)) => Err(InstanceError::Io(error)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_second_daemon_cannot_start_until_the_first_stops() {
        let temp = tempfile::tempdir().expect("temp dir");
        let dir = DataDir::new(temp.path().join("quotadial"));

        let first = Instance::acquire(&dir).expect("the first daemon starts");
        assert!(matches!(
            Instance::acquire(&dir),
            Err(InstanceError::AlreadyRunning)
        ));
        assert_eq!(
            InstanceError::AlreadyRunning.to_string(),
            "a QuotaDial daemon is already running"
        );

        drop(first);
        Instance::acquire(&dir).expect("starts once the first has stopped");
    }
}
