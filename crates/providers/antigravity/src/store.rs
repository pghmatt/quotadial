//! Where QuotaDial keeps what Antigravity CLI's status line hands over: `antigravity.json` in
//! QuotaDial's data folder, readable only by the user (D-126). The command writes it whole, through
//! a temporary file in the same folder and a rename, so a reader never sees half of one; the
//! provider reads it at each look, as a plain file of at most a megabyte. Writing it whole and
//! reading it as a plain file are `quotadial_status_line::file`'s, lifted from here so Claude's
//! file is kept the same way (D-145); what it holds is this crate's.

use std::io;
use std::path::Path;

use chrono::{DateTime, Utc};
use quotadial_status_line::file;

use crate::handover::{self, Handover};

/// The file's name in QuotaDial's data folder.
pub(crate) const FILE: &str = "antigravity.json";

/// Writes `bytes` to `file`, readable only by the user, whole (`quotadial_status_line::file::write`).
///
/// # Errors
///
/// When the folder or the file cannot be written, as on a full disk.
pub(crate) fn write(file: &Path, bytes: &[u8]) -> io::Result<()> {
    file::write(file, bytes)
}

/// What `file` holds and when it was handed over: the time the command recorded, or the file's
/// modified time when that is earlier, so a copy made later never looks newer than it is and a
/// time written by hand never looks newer than the file. `None` when it is missing, not a plain
/// file, larger than a megabyte, or not a handover.
pub(crate) fn read(file: &Path) -> Option<(Handover, DateTime<Utc>)> {
    let kept = file::read(file, handover::MOST)?;
    let handover = handover::from_file(&kept.bytes)?;
    let handed_over = kept.handed_over(handover.written_at);
    Some((handover, handed_over))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs::{self, File};
    use std::time::SystemTime;

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
}
