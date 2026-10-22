//! QuotaDial command-line interface, shared by the `quotadial` and `qd` binaries.

use std::process::ExitCode;

/// Runs the CLI and returns the process exit code.
#[must_use]
pub fn run() -> ExitCode {
    ExitCode::SUCCESS
}
