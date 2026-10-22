//! Runs a short-lived helper program, such as `security` or `claude --version`, with a deadline.

use std::io::{self, Read};
use std::process::{Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

/// A program that ran to completion.
pub(crate) struct Finished {
    pub(crate) status: ExitStatus,
    pub(crate) stdout: Vec<u8>,
}

/// Why a program produced no result.
#[derive(Debug)]
pub(crate) enum RunError {
    /// It could not be started.
    Launch(io::Error),
    /// It was still running at the deadline and was killed.
    TimedOut,
}

/// Runs `command` with no input, returning its exit status and standard output. Standard error is
/// discarded, so nothing the program prints there can reach a message. The program is killed if it
/// has not finished within `timeout`.
pub(crate) fn run(mut command: Command, timeout: Duration) -> Result<Finished, RunError> {
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(RunError::Launch)?;
    // Read on another thread, so output larger than the pipe buffer cannot stall the program.
    let reader = child.stdout.take().map(|mut stdout| {
        thread::spawn(move || {
            let mut output = Vec::new();
            // A read error leaves whatever arrived; the exit status still decides the outcome.
            let _ = stdout.read_to_end(&mut output);
            output
        })
    });

    let deadline = Instant::now() + timeout;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(10)),
            Ok(None) => {
                // Best effort: the child may exit on its own between the check and the kill.
                let _ = child.kill();
                let _ = child.wait();
                return Err(RunError::TimedOut);
            }
            Err(error) => {
                let _ = child.kill();
                return Err(RunError::Launch(error));
            }
        }
    };
    let stdout = reader
        .and_then(|reader| reader.join().ok())
        .unwrap_or_default();
    Ok(Finished { status, stdout })
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    fn shell(script: &str) -> Command {
        let mut command = Command::new("/bin/sh");
        command.args(["-c", script]);
        command
    }

    #[test]
    fn returns_the_exit_status_and_standard_output() {
        let finished = run(shell("printf 'hello'; exit 3"), Duration::from_secs(10)).expect("runs");

        assert_eq!(finished.status.code(), Some(3));
        assert_eq!(finished.stdout, b"hello");
    }

    #[test]
    fn output_larger_than_a_pipe_buffer_does_not_stall() {
        let finished =
            run(shell("head -c 200000 /dev/zero"), Duration::from_secs(10)).expect("runs");

        assert!(finished.status.success());
        assert_eq!(finished.stdout.len(), 200_000);
    }

    #[test]
    fn a_program_that_overruns_its_deadline_is_killed() {
        let started = Instant::now();
        let result = run(shell("sleep 30"), Duration::from_millis(200));

        assert!(matches!(result, Err(RunError::TimedOut)));
        assert!(started.elapsed() < Duration::from_secs(10));
    }

    #[test]
    fn a_missing_program_is_a_launch_error() {
        let result = run(
            Command::new("/nonexistent/quotadial-test-program"),
            Duration::from_secs(1),
        );

        assert!(matches!(result, Err(RunError::Launch(_))));
    }
}
