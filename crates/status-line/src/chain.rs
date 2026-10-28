//! The status line the user already had, which a provider's `status-line` command runs after it
//! keeps its numbers, so adding QuotaDial costs nothing the user had (the manager's rule (b)(3) on
//! #73): the words after `status-line` are a program and its arguments, run as they are, with no
//! shell of QuotaDial's own, since the CLI's shell has already read them. The program gets the same
//! standard input, and its output, standard error and exit status come back unchanged.
//!
//! Antigravity CLI stops any run of its status line that takes longer than 5 seconds (its docs,
//! "Status line customization"), which would stop QuotaDial and leave the program running, so
//! QuotaDial stops the program itself at [`WAIT`] and ends as a stopped run would, with nothing
//! printed and a failure. Claude Code shows a status line's output only once its run ends, and a
//! slow run keeps the line from updating until it does (its docs, "Customize your status line"), so
//! the same stop keeps a program that hangs from holding Claude Code's line past it. A program that
//! leaves another process holding its output open cannot hold QuotaDial past that either.
//!
//! Lifted whole from `crates/providers/antigravity/src/chain.rs` (D-126), with its tests and its
//! behaviour unchanged, so that Antigravity's command and Claude's chain alike (D-145).

use std::io::{self, Read, Write};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::{Arc, Mutex, PoisonError, mpsc};
use std::thread;
use std::time::{Duration, Instant};

/// How long the program may run: less than Antigravity CLI's 5 seconds, so the run ends with the
/// program stopped rather than left running when Antigravity CLI stops QuotaDial.
pub const WAIT: Duration = Duration::from_millis(4500);

/// How often QuotaDial looks to see whether the program has ended.
const LOOK_EVERY: Duration = Duration::from_millis(5);

/// The exit status a shell gives a program it cannot find, and one it cannot run.
const NOT_FOUND: u8 = 127;
const CANNOT_RUN: u8 = 126;

/// The exit status of a run that failed some other way: stopped at [`WAIT`], or ended with a status
/// that does not fit in a byte, as on Windows.
const FAILED: u8 = 1;

/// What the program printed, and how it ended.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Ran {
    /// What it printed on standard output.
    pub stdout: Vec<u8>,
    /// What it printed on standard error.
    pub stderr: Vec<u8>,
    /// The exit status it ended with, as a shell gives it.
    pub code: u8,
}

/// Runs `program` with `args`, `input` on its standard input, for at most `wait`.
#[must_use]
pub fn run(program: &str, args: &[String], input: &[u8], wait: Duration) -> Ran {
    let deadline = Instant::now() + wait;
    let mut child = match Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        // As a shell says it, on standard error, which the status line does not show.
        Err(error) => {
            return Ran {
                stdout: Vec::new(),
                stderr: format!(
                    "quotadial: {} could not be started: {error}\n",
                    quotadial_core::redact_secrets(program)
                )
                .into_bytes(),
                code: if error.kind() == io::ErrorKind::NotFound {
                    NOT_FOUND
                } else {
                    CANNOT_RUN
                },
            };
        }
    };
    if let Some(mut stdin) = child.stdin.take() {
        let input = input.to_vec();
        // On a thread of its own, so a program that prints before it reads cannot wait on QuotaDial
        // while QuotaDial waits on it. Dropped when written, which ends the program's input; a
        // program that ends without reading it all is no failure of QuotaDial's.
        thread::spawn(move || {
            let _ = stdin.write_all(&input);
        });
    }
    let stdout = Collected::from(child.stdout.take());
    let stderr = Collected::from(child.stderr.take());
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) if Instant::now() < deadline => thread::sleep(LOOK_EVERY),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                break None;
            }
        }
    };
    match status {
        Some(status) => Ran {
            stdout: stdout.until(deadline),
            stderr: stderr.until(deadline),
            code: code(status),
        },
        None => Ran {
            code: FAILED,
            ..Ran::default()
        },
    }
}

/// The exit status the program ended with, as one byte: its own when it fits, and 128 and the
/// signal for one a signal ended, as a shell gives it. A failure never becomes 0.
fn code(status: ExitStatus) -> u8 {
    match status.code() {
        Some(code) => u8::try_from(code).unwrap_or(FAILED),
        None => signal(status),
    }
}

#[cfg(unix)]
fn signal(status: ExitStatus) -> u8 {
    use std::os::unix::process::ExitStatusExt;

    status
        .signal()
        .and_then(|signal| u8::try_from(128 + signal).ok())
        .unwrap_or(FAILED)
}

#[cfg(not(unix))]
fn signal(_status: ExitStatus) -> u8 {
    FAILED
}

/// What one of the program's outputs gives, read on a thread of its own as it comes.
struct Collected {
    read: Arc<Mutex<Vec<u8>>>,
    /// Told when the output ends.
    ended: mpsc::Receiver<()>,
}

impl Collected {
    fn from(output: Option<impl Read + Send + 'static>) -> Self {
        let read = Arc::new(Mutex::new(Vec::new()));
        let (end, ended) = mpsc::channel();
        if let Some(mut output) = output {
            let into = Arc::clone(&read);
            thread::spawn(move || {
                let mut chunk = [0_u8; 8192];
                loop {
                    match output.read(&mut chunk) {
                        Ok(0) => break,
                        Ok(length) => into
                            .lock()
                            .unwrap_or_else(PoisonError::into_inner)
                            .extend_from_slice(&chunk[..length]),
                        Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                        Err(_) => break,
                    }
                }
                let _ = end.send(());
            });
        }
        Self { read, ended }
    }

    /// What was read once the output ends, or at `deadline` if it is still open then.
    fn until(self, deadline: Instant) -> Vec<u8> {
        let _ = self
            .ended
            .recv_timeout(deadline.saturating_duration_since(Instant::now()));
        std::mem::take(&mut *self.read.lock().unwrap_or_else(PoisonError::into_inner))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A program that cannot be on any machine.
    const NOWHERE: &str = "quotadial-test-no-such-program-0a1b2c";

    #[test]
    fn a_program_that_cannot_be_found_ends_as_a_shell_says_it_and_prints_nothing() {
        let ran = run(NOWHERE, &[], b"{}", WAIT);

        assert_eq!(ran.code, NOT_FOUND);
        assert!(ran.stdout.is_empty());
        let said = String::from_utf8(ran.stderr).expect("UTF-8");
        assert!(
            said.starts_with(&format!("quotadial: {NOWHERE} could not be started: ")),
            "{said}"
        );
    }

    #[cfg(unix)]
    fn sh(script: &str) -> (String, Vec<String>) {
        (
            "/bin/sh".to_owned(),
            vec!["-c".to_owned(), script.to_owned()],
        )
    }

    #[cfg(unix)]
    #[test]
    fn the_program_gets_the_same_input_and_its_output_and_status_come_back_unchanged() {
        let (program, args) =
            sh("cat; printf 'its own \\033[1mline\\033[0m\\n'; printf 'to stderr' >&2; exit 3");
        let input = b"{ \"quota\": {} }\n\xff not even UTF-8";

        let ran = run(&program, &args, input, WAIT);

        let mut expected = input.to_vec();
        expected.extend_from_slice(b"its own \x1b[1mline\x1b[0m\n");
        assert_eq!(
            ran,
            Ran {
                stdout: expected,
                stderr: b"to stderr".to_vec(),
                code: 3,
            }
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_program_that_ends_well_ends_with_0_and_one_a_signal_ends_as_a_shell_says() {
        let (program, args) = sh("exit 0");
        assert_eq!(run(&program, &args, b"", WAIT).code, 0);

        let (program, args) = sh("kill -9 $$");
        assert_eq!(run(&program, &args, b"", WAIT).code, 128 + 9);
    }

    #[cfg(unix)]
    #[test]
    fn a_program_that_runs_too_long_is_stopped_at_the_wait_with_nothing_printed() {
        let (program, args) = sh("printf started; exec sleep 30");
        let started = Instant::now();

        let ran = run(&program, &args, b"", Duration::from_millis(300));

        assert!(
            started.elapsed() < Duration::from_secs(5),
            "{:?}",
            started.elapsed()
        );
        assert_eq!(
            ran,
            Ran {
                code: FAILED,
                ..Ran::default()
            }
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_program_that_leaves_its_output_open_cannot_hold_quotadial_past_the_wait() {
        // The shell ends at once, but the sleep it leaves behind holds its output open.
        let (program, args) = sh("printf shown; sleep 3 &");
        let started = Instant::now();

        let ran = run(&program, &args, b"", Duration::from_millis(500));

        assert!(
            started.elapsed() < Duration::from_secs(2),
            "{:?}",
            started.elapsed()
        );
        assert_eq!(ran.stdout, b"shown");
        assert_eq!(ran.code, 0);
    }

    #[cfg(unix)]
    #[test]
    fn a_program_that_reads_none_of_its_input_is_no_failure() {
        let input = vec![b'x'; 1024 * 1024];
        let (program, args) = sh("printf done");

        let ran = run(&program, &args, &input, WAIT);

        assert_eq!(ran.stdout, b"done");
        assert_eq!(ran.code, 0);
    }

    #[cfg(unix)]
    #[test]
    fn an_exit_status_comes_back_as_a_shell_gives_it() {
        use std::os::unix::process::ExitStatusExt;

        // A wait status: the exit code in the second byte, or the signal in the first.
        assert_eq!(code(ExitStatus::from_raw(0)), 0);
        assert_eq!(code(ExitStatus::from_raw(3 << 8)), 3);
        assert_eq!(code(ExitStatus::from_raw(255 << 8)), 255);
        assert_eq!(code(ExitStatus::from_raw(15)), 128 + 15);
    }
}
