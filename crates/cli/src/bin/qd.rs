//! `qd`: short alias for the `quotadial` binary.

use std::process::ExitCode;

fn main() -> ExitCode {
    quotadial_cli::run()
}
