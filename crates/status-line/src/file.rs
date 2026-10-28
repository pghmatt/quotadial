//! The file a status line command keeps its numbers in, in QuotaDial's data folder, readable only by
//! the user: written whole, through a temporary file in the same folder and a rename, so a reader
//! never sees half of one, and read back at each look as a plain file no larger than its provider
//! allows, with the time it was last changed.
//!
//! Lifted from `crates/providers/antigravity/src/store.rs` (D-126), its behaviour unchanged (D-145):
//! the write whole, and the read's part that knows files. What a file holds, and the time it records
//! inside, are each provider's own.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Utc};

/// Writes `bytes` to `file`, readable only by the user, through a temporary file beside it and a
/// rename, so no reader ever sees half of it. The folder is made, readable only by the user, when it
/// is missing. The temporary file's name is this process's own, since two CLIs can run their status
/// lines at once, and it is removed when the write fails. Nothing is synced to the disk: losing the
/// newest numbers in a power cut costs one handover, which the next brings back.
///
/// # Errors
///
/// When the folder or the file cannot be written, as on a full disk.
pub fn write(file: &Path, bytes: &[u8]) -> io::Result<()> {
    let folder = file
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "the file has no folder"))?;
    let name = file
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "the file has no name"))?;
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
    builder.create(folder)?;
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.subsec_nanos());
    let temporary = folder.join(format!(
        ".{}.{}.{nanos}.tmp",
        name.to_string_lossy(),
        std::process::id()
    ));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    let mut temporary_file = options.open(&temporary)?;
    let written = temporary_file.write_all(bytes).and_then(|()| {
        drop(temporary_file);
        fs::rename(&temporary, file)
    });
    if written.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    written
}

/// What a file holds, as [`read`] found it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Kept {
    /// Its bytes.
    pub bytes: Vec<u8>,
    /// When it was last changed, when the system says.
    pub modified: Option<DateTime<Utc>>,
}

impl Kept {
    /// When what the file holds was handed over: `written_at`, the time the command recorded in it,
    /// or the file's modified time when that is earlier, so a copy made later never looks newer than
    /// it is and a time written by hand never looks newer than the file.
    #[must_use]
    pub fn handed_over(&self, written_at: DateTime<Utc>) -> DateTime<Utc> {
        self.modified
            .map_or(written_at, |modified| modified.min(written_at))
    }
}

/// What `file` holds and when it was last changed, or `None` when it is missing, not a plain file,
/// or larger than `most` bytes.
#[must_use]
pub fn read(file: &Path, most: usize) -> Option<Kept> {
    // Only a plain file is opened: a pipe left in its place would wait for a writer forever.
    if !fs::metadata(file).ok()?.is_file() {
        return None;
    }
    let opened = File::open(file).ok()?;
    let metadata = opened.metadata().ok()?;
    let most = u64::try_from(most).ok()?;
    if metadata.len() > most {
        return None;
    }
    let mut bytes = Vec::new();
    opened.take(most + 1).read_to_end(&mut bytes).ok()?;
    if u64::try_from(bytes.len()).ok()? > most {
        return None;
    }
    Some(Kept {
        bytes,
        modified: metadata.modified().ok().map(DateTime::<Utc>::from),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The name a test's file has, as a provider's would.
    const FILE: &str = "antigravity.json";

    /// The temporary files a write left in `folder`.
    fn leftovers(folder: &Path) -> Vec<String> {
        fs::read_dir(folder)
            .expect("lists")
            .map(|entry| {
                entry
                    .expect("an entry")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .filter(|name| name.ends_with(".tmp"))
            .collect()
    }

    #[cfg(unix)]
    #[test]
    fn the_file_and_a_folder_it_makes_are_readable_only_by_the_user() {
        use std::os::unix::fs::PermissionsExt;

        let temp = tempfile::tempdir().expect("temp dir");
        let file = temp.path().join("data").join(FILE);

        write(&file, b"{}").expect("writes");

        let mode = |path: &Path| fs::metadata(path).expect("metadata").permissions().mode() & 0o777;
        assert_eq!(mode(&file), 0o600);
        assert_eq!(mode(file.parent().expect("a folder")), 0o700);
    }

    #[cfg(unix)]
    #[test]
    fn the_file_is_replaced_whole_and_never_written_over_in_place() {
        let temp = tempfile::tempdir().expect("temp dir");
        let file = temp.path().join(FILE);
        write(&file, b"the first handover").expect("writes");
        // A second name for the first file: a rename gives the file's name a new file and leaves
        // this one as it was, where writing over the file in place would change both.
        let first = temp.path().join("first");
        fs::hard_link(&file, &first).expect("links");

        write(&file, b"the second").expect("replaces");

        assert_eq!(fs::read(&file).expect("reads"), b"the second");
        assert_eq!(fs::read(&first).expect("reads"), b"the first handover");
    }

    #[test]
    fn a_file_that_cannot_be_written_leaves_nothing_behind() {
        let temp = tempfile::tempdir().expect("temp dir");
        // A folder in the file's place: the rename fails, after the temporary file is written.
        let file = temp.path().join(FILE);
        fs::create_dir(&file).expect("a folder");
        fs::write(file.join("kept"), "x").expect("writes");

        assert!(write(&file, b"{}").is_err());
        assert!(
            leftovers(temp.path()).is_empty(),
            "{:?}",
            leftovers(temp.path())
        );

        // A file in the folder's place: nothing can be made in it.
        let blocked = temp.path().join("blocked");
        fs::write(&blocked, "not a folder").expect("writes");
        assert!(write(&blocked.join(FILE), b"{}").is_err());
    }

    #[cfg(unix)]
    #[test]
    fn a_pipe_in_place_of_the_file_is_never_opened() {
        // Imported here: the test is built only on Unix, and an import nothing else in the module
        // uses fails clippy on Windows.
        use std::time::Duration;

        let temp = tempfile::tempdir().expect("temp dir");
        let pipe = temp.path().join(FILE);
        let made = std::process::Command::new("mkfifo")
            .arg(&pipe)
            .status()
            .expect("runs mkfifo");
        assert!(made.success(), "mkfifo {made}");
        let (sent, read_back) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = sent.send(read(&pipe, 1024));
        });

        assert_eq!(
            read_back.recv_timeout(Duration::from_secs(5)),
            Ok(None),
            "reading waited on a pipe that nothing writes to"
        );
    }
}
