//! Where QuotaDial keeps what Claude Code's status line hands over: one file for each Claude Code
//! folder, in QuotaDial's data folder, readable only by the user (the manager's rule 4 on #73,
//! D-145). Claude Code's default folder writes `claude.json`; a folder `CLAUDE_CONFIG_DIR` names,
//! which the command finds in its own environment since Claude Code's child processes inherit it,
//! writes `claude-` and the first 8 hex of the SHA-256 of the folder's path, as Claude Code was
//! given it and in Unicode's composed form, then `.json`: the same 8 hex Claude Code itself puts
//! after `Claude Code-credentials-` for that folder, so the two namings agree. The file is written whole
//! and read back as a plain file of at most a megabyte by `quotadial_status_line::file`, which
//! Antigravity's file shares (D-126).

use std::io;
use std::path::Path;

use chrono::{DateTime, Utc};
use quotadial_status_line::file;

use crate::environment::{nfc, sha256_hex};
use crate::handover::{self, Handover};

/// The file Claude Code's default folder writes.
pub(crate) const DEFAULT_FILE: &str = "claude.json";

/// The name of the file for the folder `config_dir` names, `CLAUDE_CONFIG_DIR` as Claude Code was
/// given it, or for Claude Code's default folder when there is none or it is empty, as Claude Code
/// takes an empty one.
pub(crate) fn file_name(config_dir: Option<&str>) -> String {
    match config_dir.filter(|dir| !dir.is_empty()) {
        Some(dir) => format!("claude-{}.json", &sha256_hex(&nfc(dir))[..8]),
        None => DEFAULT_FILE.to_owned(),
    }
}

/// Writes `bytes` to `file`, readable only by the user, whole
/// (`quotadial_status_line::file::write`).
///
/// # Errors
///
/// When the folder or the file cannot be written, as on a full disk.
pub(crate) fn write(file: &Path, bytes: &[u8]) -> io::Result<()> {
    file::write(file, bytes)
}

/// What `file` holds and when it was handed over: the time the command recorded, or the file's
/// modified time when that is earlier, so a copy made later never looks newer than it is and a time
/// written by hand never looks newer than the file. `None` when it is missing, not a plain file,
/// larger than a megabyte, or not a handover.
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
    use crate::handover::{FIVE_HOUR, Limit};

    fn handover(written_at: DateTime<Utc>) -> Handover {
        Handover {
            written_at,
            windows: BTreeMap::from([(
                FIVE_HOUR,
                Limit {
                    used_percentage: 23.5,
                    resets_at: DateTime::from_timestamp(1_738_425_600, 0),
                },
            )]),
        }
    }

    /// The whole second `ago` before now, which every file system keeps exactly.
    fn whole_second(ago: TimeDelta) -> DateTime<Utc> {
        DateTime::from_timestamp((Utc::now() - ago).timestamp(), 0).expect("a time")
    }

    #[test]
    fn the_default_folder_writes_claude_json_and_a_named_one_its_own_file() {
        assert_eq!(file_name(None), "claude.json");
        assert_eq!(file_name(Some("")), "claude.json", "an empty one is none");
        // printf '%s' /Users/example/.claude-work | shasum -a 256 → dd1118a7…, the hash Claude Code
        // puts after `Claude Code-credentials-` for that folder.
        assert_eq!(
            file_name(Some("/Users/example/.claude-work")),
            "claude-dd1118a7.json"
        );
        // As Claude Code was given it: a separator at the end is another folder to Claude Code.
        assert_ne!(
            file_name(Some("/Users/example/.claude-work/")),
            "claude-dd1118a7.json"
        );
    }

    #[test]
    fn a_handover_is_written_whole_and_read_back_with_its_time() {
        let temp = tempfile::tempdir().expect("temp dir");
        let file = temp.path().join("made").join(DEFAULT_FILE);
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
    }

    #[test]
    fn a_file_written_later_than_it_says_is_as_old_as_it_says_and_never_newer_than_itself() {
        let temp = tempfile::tempdir().expect("temp dir");
        let file = temp.path().join(DEFAULT_FILE);

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
        let file = temp.path().join(DEFAULT_FILE);

        assert_eq!(read(&file), None, "missing");
        fs::write(&file, "not JSON").expect("writes");
        assert_eq!(read(&file), None, "not JSON");
        fs::write(&file, r#"{ "rate_limits": {} }"#).expect("writes");
        assert_eq!(read(&file), None, "no time");
        let large = format!(
            "{}{}",
            String::from_utf8(handover::to_file(&handover(Utc::now()))).expect("UTF-8"),
            " ".repeat(handover::MOST)
        );
        fs::write(&file, large).expect("writes");
        assert_eq!(read(&file), None, "larger than a megabyte");
        assert_eq!(read(temp.path()), None, "a folder");
    }
}
