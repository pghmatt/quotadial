//! What QuotaDial's status line commands share (D-145). `quotadial antigravity status-line` (D-126)
//! and `quotadial claude status-line` (D-145) are each run by a CLI's status line with the session's
//! state on standard input; each keeps what it needs in a file of its own in QuotaDial's data
//! folder, then runs the status line the user already had. This crate is the part that is the same
//! for both, lifted from the Antigravity provider with its behaviour and its tests unchanged:
//!
//! - [`run`], the command itself, from `crates/providers/antigravity/src/command.rs`: it reads the
//!   whole of standard input, hands it to the provider to keep, then runs the command named after
//!   it, and prints nothing of its own, ever.
//! - [`chain`], the status line named after it, from `crates/providers/antigravity/src/chain.rs`:
//!   run with the same input and no shell of QuotaDial's, stopped at 4.5 seconds, its output and
//!   exit status passed back unchanged, 127 or 126 when it cannot be found or run, and 128 and the
//!   signal when a signal ends it.
//! - [`file`], the file each command keeps its numbers in, from
//!   `crates/providers/antigravity/src/store.rs`: written whole, through a temporary file and a
//!   rename, 0600 in a 0700 folder, with no sync; read back only as a plain file of a size the
//!   provider allows, with the time it was last changed.
//!
//! It knows files, processes and exit codes. What a CLI's status line hands over, such as
//! Antigravity's `quota` or Claude Code's `rate_limits`, is each provider's own and never reaches
//! it, and it asks no one.

pub mod chain;
pub mod file;

use std::process::ExitCode;

use quotadial_core::Invocation;

/// Runs a provider's status line command: reads all of standard input, whatever it holds, hands it
/// to `keep`, then runs the status line command named after it, if any, with the same input, and
/// passes its output and exit status back unchanged.
///
/// `keep` runs first, so a slow or failing status line never costs the handover, and it returns
/// nothing: what goes wrong while keeping, as with no data folder or a full disk, is dropped there
/// and can never reach the status line. Nothing of QuotaDial's own is printed, ever: with no command
/// named after it, the run ends with 0 whatever happened.
pub fn run(invocation: Invocation<'_>, keep: impl FnOnce(&[u8])) -> ExitCode {
    let Invocation {
        args,
        stdin,
        stdout,
        stderr,
    } = invocation;
    let mut payload = Vec::new();
    // All of it, whatever it holds: a command named after this one gets the same bytes.
    let _ = stdin.read_to_end(&mut payload);
    keep(&payload);
    let Some((program, args)) = args.split_first() else {
        return ExitCode::SUCCESS;
    };
    let ran = chain::run(program, args, &payload, chain::WAIT);
    let _ = stdout.write_all(&ran.stdout).and_then(|()| stdout.flush());
    let _ = stderr.write_all(&ran.stderr).and_then(|()| stderr.flush());
    ExitCode::from(ran.code)
}
