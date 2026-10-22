# Task tracker

Progress and implementation notes. Update this file with every change.

## Current focus

Project kickoff (2026-10-22): scaffold, skeleton code, GitHub setup, then the M1 plan for the Claude
provider.

## Kickoff

| Step | Status | Notes |
| --- | --- | --- |
| 1. Project setup | Done | Cargo workspace, licence, NOTICE, gitignore, toolchain, editor and lint config, AGENTS.md (CLAUDE.md is a symlink to it), README, CI and release workflows, issue templates, Code of Conduct |
| 2. Skeleton code | Waiting for `docs/PRD.md` | Core types must match the PRD's "Provider trait" and "Normalized model" sections exactly |
| 3. GitHub | Not started | Public repo `pghmatt/quotadial`, branch protection, milestones M1 to M6, M1 issues |
| 4. Prepare to build | Not started | CodexBar path check, CLI login check, plan for M1 issue #1 |

## Implementation notes

### Step 1: project setup

- Workspace members: `crates/core`, `crates/providers/claude`, `crates/providers/codex`,
  `crates/api`, `crates/cli`. Shared package metadata and lints live in the root `Cargo.toml`
  (`[workspace.package]`, `[workspace.lints]`); every crate opts in with `[lints] workspace = true`.
- The CLI package `quotadial` has a library target `quotadial_cli` holding all logic, plus two thin
  binaries, `quotadial` (`src/main.rs`) and `qd` (`src/bin/qd.rs`). `autobins = false` keeps Cargo
  from inventing a third binary from the package name.
- `fixtures/**/raw/` is gitignored at every depth (checked with `git check-ignore`).
- CI runs fmt, clippy `-D warnings` and tests on ubuntu, macos and windows, then a `ci-success`
  job that branch protection requires.
- Toolchain verified locally: rustc 1.99.1, clippy 0.1.99, rustfmt 1.10.0.
