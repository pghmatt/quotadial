//! Reading a log's complete lines from where the last scan stopped. Logs are appended to while
//! they are read, so a last line without its newline is left for the next scan, and a line too
//! long to hold usage is skipped without being kept in memory.

use std::fs::File;
use std::io::{self, BufRead, BufReader, Seek, SeekFrom};

/// A line longer than this is skipped: a request's usage line is a few kilobytes, or a few
/// megabytes when the model wrote a large file.
pub(crate) const MAX_LINE: usize = 64 * 1024 * 1024;

/// How much of a file is read at a time.
const CHUNK: usize = 256 * 1024;

/// Where a reading stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Stop {
    /// Where the first line not handed starts.
    pub(crate) at: u64,
    /// Whether the reading reached the end of the file, rather than its budget.
    pub(crate) end: bool,
}

/// Hands each complete, non-empty line of `file` from byte `from` to `each`, with the offset it
/// starts at, until the end of the file or the first line to end `budget` bytes or more after
/// `from`. A last line still being written is left for the next reading. A line longer than
/// `max_line` is skipped.
///
/// # Errors
///
/// When the file cannot be read.
pub(crate) fn lines(
    file: &mut File,
    from: u64,
    budget: u64,
    max_line: usize,
    mut each: impl FnMut(u64, &[u8]),
) -> io::Result<Stop> {
    let mut reader = BufReader::with_capacity(CHUNK, file);
    reader.seek(SeekFrom::Start(from))?;
    // The line being gathered: where it starts, how many of its bytes were read, and those bytes
    // unless it is too long. A line found whole in the buffer is handed without a copy.
    let mut start = from;
    let mut length: u64 = 0;
    let mut line: Vec<u8> = Vec::new();
    let mut too_long = false;
    loop {
        if start - from >= budget {
            return Ok(Stop {
                at: start,
                end: false,
            });
        }
        let buffer = match reader.fill_buf() {
            Ok([]) => {
                return Ok(Stop {
                    at: start,
                    end: true,
                });
            }
            Ok(buffer) => buffer,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        };
        let newline = memchr::memchr(b'\n', buffer);
        let used = newline.map_or(buffer.len(), |at| at + 1);
        let part = &buffer[..newline.unwrap_or(buffer.len())];
        if !too_long {
            if line.len() + part.len() > max_line {
                too_long = true;
                line = Vec::new();
            } else if newline.is_some() && line.is_empty() {
                if !part.is_empty() {
                    each(start, part);
                }
            } else {
                line.extend_from_slice(part);
            }
        }
        reader.consume(used);
        length += used as u64;
        if newline.is_some() {
            if !too_long && !line.is_empty() {
                each(start, &line);
            }
            start += length;
            length = 0;
            too_long = false;
            if line.capacity() > 4 * CHUNK {
                line = Vec::new();
            } else {
                line.clear();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    fn read_within(
        contents: &[u8],
        from: u64,
        budget: u64,
        max_line: usize,
    ) -> (Vec<(u64, String)>, Stop) {
        let mut file = tempfile::tempfile().expect("temp file");
        file.write_all(contents).expect("writes");
        let mut seen = Vec::new();
        let stop = lines(&mut file, from, budget, max_line, |offset, line| {
            seen.push((offset, String::from_utf8_lossy(line).into_owned()));
        })
        .expect("reads");
        (seen, stop)
    }

    fn read(contents: &[u8], from: u64, max_line: usize) -> (Vec<(u64, String)>, u64) {
        let (seen, stop) = read_within(contents, from, u64::MAX, max_line);
        assert!(stop.end);
        (seen, stop.at)
    }

    #[test]
    fn complete_lines_come_with_their_offsets_and_a_partial_last_line_waits() {
        let (seen, next) = read(b"one\ntwo\n\nthree", 0, MAX_LINE);

        assert_eq!(seen, [(0, "one".to_owned()), (4, "two".to_owned())]);
        assert_eq!(next, 9, "the next scan starts at \"three\"");

        let (seen, next) = read(b"one\ntwo\n\nthree\n", 9, MAX_LINE);
        assert_eq!(seen, [(9, "three".to_owned())]);
        assert_eq!(next, 15);
    }

    #[test]
    fn a_line_longer_than_the_buffer_is_gathered_and_one_too_long_is_skipped() {
        let long = "x".repeat(CHUNK * 3);
        let contents = format!("{long}\nshort\n");

        let (seen, next) = read(contents.as_bytes(), 0, MAX_LINE);
        assert_eq!(seen.len(), 2);
        assert_eq!(seen[0].1.len(), CHUNK * 3);
        assert_eq!(seen[1], ((CHUNK * 3 + 1) as u64, "short".to_owned()));
        assert_eq!(next, contents.len() as u64);

        let (seen, next) = read(contents.as_bytes(), 0, CHUNK);
        assert_eq!(seen, [((CHUNK * 3 + 1) as u64, "short".to_owned())]);
        assert_eq!(next, contents.len() as u64, "skipped, yet read past");
    }

    #[test]
    fn a_budget_stops_the_reading_at_a_line_s_end() {
        let contents = b"aaaa\nbbbb\ncccc\n";

        let (seen, stop) = read_within(contents, 0, 6, MAX_LINE);
        assert_eq!(seen, [(0, "aaaa".to_owned()), (5, "bbbb".to_owned())]);
        assert_eq!(stop, Stop { at: 10, end: false });

        let (seen, stop) = read_within(contents, 10, 6, MAX_LINE);
        assert_eq!(seen, [(10, "cccc".to_owned())]);
        assert_eq!(
            stop,
            Stop { at: 15, end: true },
            "the end before the budget"
        );

        let (seen, stop) = read_within(contents, 0, 5, MAX_LINE);
        assert_eq!(seen, [(0, "aaaa".to_owned())]);
        assert_eq!(stop, Stop { at: 5, end: false }, "the budget spent exactly");
    }
}
