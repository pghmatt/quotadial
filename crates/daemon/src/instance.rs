//! Only one daemon runs at a time: it holds a lock on a file in the data directory for as long as
//! it runs. The operating system releases the lock when the process ends, however it ends, so the
//! lock is also how a consumer tells whether a daemon is running.

use std::fs::{File, OpenOptions, TryLockError};
use std::io;
use std::thread;
use std::time::{Duration, Instant};

use crate::DataDir;

/// How long a starting daemon waits out a consumer that is checking the lock.
const ACQUIRE_WAIT: Duration = Duration::from_secs(1);

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
        // A consumer checking whether a daemon runs holds the lock for a moment, so a held lock is
        // tried again for a second before another daemon is assumed.
        let deadline = Instant::now() + ACQUIRE_WAIT;
        loop {
            match file.try_lock() {
                Ok(()) => return Ok(Self { _lock: file }),
                Err(TryLockError::WouldBlock) if Instant::now() < deadline => {
                    thread::sleep(Duration::from_millis(20));
                }
                Err(TryLockError::WouldBlock) => return Err(InstanceError::AlreadyRunning),
                Err(TryLockError::Error(error)) => return Err(InstanceError::Io(error)),
            }
        }
    }

    /// Whether a daemon holds the claim on `dir` now.
    #[must_use]
    pub fn is_running(dir: &DataDir) -> bool {
        let Ok(file) = File::open(dir.lock()) else {
            return false;
        };
        // A shared lock, released as the file closes, so checks never hold each other up.
        matches!(file.try_lock_shared(), Err(TryLockError::WouldBlock))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_second_daemon_cannot_start_until_the_first_stops() {
        let temp = tempfile::tempdir().expect("temp dir");
        let dir = DataDir::new(temp.path().join("quotadial"));

        assert!(!Instance::is_running(&dir), "no lock file yet");
        let first = Instance::acquire(&dir).expect("the first daemon starts");
        assert!(Instance::is_running(&dir));
        assert!(matches!(
            Instance::acquire(&dir),
            Err(InstanceError::AlreadyRunning)
        ));
        assert_eq!(
            InstanceError::AlreadyRunning.to_string(),
            "a QuotaDial daemon is already running"
        );

        drop(first);
        assert!(!Instance::is_running(&dir));
        Instance::acquire(&dir).expect("starts once the first has stopped");
    }

    #[test]
    fn checking_for_a_daemon_never_stops_one_starting() {
        let temp = tempfile::tempdir().expect("temp dir");
        let dir = DataDir::new(temp.path().join("quotadial"));
        drop(Instance::acquire(&dir).expect("creates the lock file"));

        // A consumer holding the shared lock while it checks, as `is_running` does for a moment.
        let checking = File::open(dir.lock()).expect("opens");
        checking.lock_shared().expect("locks");
        let release = thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            drop(checking);
        });

        let daemon = Instance::acquire(&dir).expect("waits out the check");
        release.join().expect("joins");
        assert!(Instance::is_running(&dir));
        drop(daemon);
    }
}
