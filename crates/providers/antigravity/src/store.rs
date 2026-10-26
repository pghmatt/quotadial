//! Where QuotaDial keeps what Antigravity CLI's status line hands over: `antigravity.json` in
//! QuotaDial's data folder, readable only by the user (D-126). The command writes it whole, through
//! a temporary file in the same folder and a rename, so a reader never sees half of one; the
//! provider reads it at each look, as a plain file of at most a megabyte.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Utc};

use crate::handover::{self, Handover};

/// The file's name in QuotaDial's data folder.
pub(crate) const FILE: &str = "antigravity.json";

/// Writes `bytes` to `file`, readable only by the user, through a temporary file beside it and a
/// rename, so no reader ever sees half of it. The folder is made, readable only by the user, when it
/// is missing. The temporary file's name is this process's own, since two Antigravity CLIs can run
/// their status lines at once, and it is removed when the write fails. Nothing is synced to the
/// disk: losing the newest numbers in a power cut costs one handover, which the next brings back.
///
/// # Errors
///
/// When the folder or the file cannot be written, as on a full disk.
pub(crate) fn write(file: &Path, bytes: &[u8]) -> io::Result<()> {
    let folder = file
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "the file has no folder"))?;
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
    builder.create(folder)?;
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.subsec_nanos());
    let temporary = folder.join(format!(".{FILE}.{}.{nanos}.tmp", std::process::id()));
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

/// What `file` holds and when it was handed over: the time the command recorded, or the file's
/// modified time when that is earlier, so a copy made later never looks newer than it is and a
/// time written by hand never looks newer than the file. `None` when it is missing, not a plain
/// file, larger than a megabyte, or not a handover.
pub(crate) fn read(file: &Path) -> Option<(Handover, DateTime<Utc>)> {
    // Only a plain file is opened: a pipe left in its place would wait for a writer forever.
    if !fs::metadata(file).ok()?.is_file() {
        return None;
    }
    let opened = File::open(file).ok()?;
    let metadata = opened.metadata().ok()?;
    let most = u64::try_from(handover::MOST).ok()?;
    if metadata.len() > most {
        return None;
    }
    let mut bytes = Vec::new();
    opened.take(most + 1).read_to_end(&mut bytes).ok()?;
    let handover = handover::from_file(&bytes)?;
    let handed_over = metadata
        .modified()
        .ok()
        .map(DateTime::<Utc>::from)
        .map_or(handover.written_at, |modified| {
            modified.min(handover.written_at)
        });
    Some((handover, handed_over))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use chrono::TimeDelta;

    use super::*;
    use crate::handover::Limit;

    fn at(rfc3339: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(rfc3339)
            .expect("a time")
            .with_timezone(&Utc)
    }

    fn handover(written_at: DateTime<Utc>) -> Handover {
        Handover {
            written_at,
            email: Some("someone@example.com".into()),
            plan_tier: Some("Pro".into()),
            limits: BTreeMap::from([(
                "gemini-5h".to_owned(),
                Limit {
                    remaining_fraction: Some(0.75),
                    remaining_amount: None,
                    reset_time: Some(at("2026-10-26T02:00:00Z")),
                    reset_in_seconds: None,
                    disabled: false,
                },
            )]),
        }
    }

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

    /// The whole second `ago` before now, which every file system keeps exactly.
    fn whole_second(ago: TimeDelta) -> DateTime<Utc> {
        DateTime::from_timestamp((Utc::now() - ago).timestamp(), 0).expect("a time")
    }

    #[test]
    fn a_handover_is_written_whole_and_read_back_with_its_time() {
        let temp = tempfile::tempdir().expect("temp dir");
        let file = temp.path().join("made").join(FILE);
        let earlier = whole_second(TimeDelta::minutes(2));
        let handed = whole_second(TimeDelta::minutes(1));

        write(&file, &handover::to_file(&handover(earlier))).expect("writes");
        write(&file, &handover::to_file(&handover(handed))).expect("replaces");

        let (read, handed_over) = read(&file).expect("reads");
        assert_eq!(read, handover(handed));
        assert_eq!(
            handed_over, handed,
            "the recorded time, earlier than the file's"
        );
        assert!(leftovers(file.parent().expect("a folder")).is_empty());
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

    #[test]
    fn a_file_written_later_than_it_says_is_as_old_as_it_says_and_never_newer_than_itself() {
        let temp = tempfile::tempdir().expect("temp dir");
        let file = temp.path().join(FILE);

        // A copy made now of numbers handed over three days ago.
        let days_ago = whole_second(TimeDelta::days(3));
        write(&file, &handover::to_file(&handover(days_ago))).expect("writes");
        assert_eq!(read(&file).expect("reads").1, days_ago);

        // A time written by hand a day ahead, in a file last changed an hour ago.
        write(
            &file,
            &handover::to_file(&handover(whole_second(-TimeDelta::days(1)))),
        )
        .expect("writes");
        let changed = whole_second(TimeDelta::hours(1));
        File::options()
            .write(true)
            .open(&file)
            .expect("opens")
            .set_modified(SystemTime::from(changed))
            .expect("sets the time");
        assert_eq!(read(&file).expect("reads").1, changed);
    }

    #[test]
    fn what_cannot_be_read_as_a_handover_is_none() {
        let temp = tempfile::tempdir().expect("temp dir");
        let file = temp.path().join(FILE);

        assert_eq!(read(&file), None, "missing");
        fs::write(&file, "not JSON").expect("writes");
        assert_eq!(read(&file), None, "not JSON");
        fs::write(&file, r#"{ "quota": {} }"#).expect("writes");
        assert_eq!(read(&file), None, "no time");
        let large = format!(
            "{}{}",
            String::from_utf8(handover::to_file(&handover(at("2026-10-25T22:30:00Z"))))
                .expect("UTF-8"),
            " ".repeat(handover::MOST)
        );
        fs::write(&file, large).expect("writes");
        assert_eq!(read(&file), None, "larger than a megabyte");
        assert_eq!(read(temp.path()), None, "a folder");
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
            let _ = sent.send(read(&pipe));
        });

        assert_eq!(
            read_back.recv_timeout(Duration::from_secs(5)),
            Ok(None),
            "reading waited on a pipe that nothing writes to"
        );
    }
}
