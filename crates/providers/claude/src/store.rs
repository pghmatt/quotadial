//! Where QuotaDial keeps what Claude Code's status line hands over: one file for each Claude Code
//! folder, in QuotaDial's data folder, readable only by the user (the manager's rule 4 on #73,
//! D-145). Claude Code's default folder writes `claude.json`; a folder `CLAUDE_CONFIG_DIR` names,
//! which the command finds in its own environment since Claude Code's child processes inherit it,
//! writes `claude-` and the first 8 hex of the SHA-256 of the folder's path, as Claude Code was given
//! it and in Unicode's composed form, then `.json`: the same 8 hex Claude Code itself puts after
//! `Claude Code-credentials-` for that folder, so the two namings agree. The file is written whole
//! and read back as a plain file of at most a megabyte by `quotadial_status_line::file`, which
//! Antigravity's file shares (D-126).

use std::io;
use std::path::Path;

use quotadial_status_line::file;

use crate::environment::{nfc, sha256_hex};

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

/// Writes `bytes` to `file`, readable only by the user, whole (`quotadial_status_line::file::write`).
///
/// # Errors
///
/// When the folder or the file cannot be written, as on a full disk.
pub(crate) fn write(file: &Path, bytes: &[u8]) -> io::Result<()> {
    file::write(file, bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
