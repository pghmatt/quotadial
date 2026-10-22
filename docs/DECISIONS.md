# Decisions

Choices the PRD left open, and why. When the PRD is ambiguous, pick the option that ships the current
milestone fastest and record it here. Newest entries go at the bottom.

| # | Date | Decision | Why |
| --- | --- | --- | --- |
| D-001 | 2026-10-22 | The CLI package is named `quotadial`; the libraries are `quotadial-core`, `quotadial-provider-claude`, `quotadial-provider-codex` and `quotadial-api`. `qd` is a second binary target in the CLI package. | `cargo install quotadial` installs both binaries later. A real binary works on Windows, where a symlink alias would not. |
| D-002 | 2026-10-22 | Rust edition 2024 on the unpinned `stable` channel, with no `rust-version` before 1.0. | The setup brief asks for stable. Declaring an MSRV makes the version-aware resolver pick older dependencies for no user benefit in a shipped binary. |
| D-003 | 2026-10-22 | Clippy keeps its default groups, denies `dbg_macro`, `todo` and `unimplemented`, and warns on `print_stdout` and `print_stderr` outside the CLI. `pedantic` is off for now. | These lints guard the token and stub rules in `AGENTS.md` without slowing M1. Revisit `pedantic` before 1.0. |
| D-004 | 2026-10-22 | Branch protection on `main` requires one check, `ci-success`, which passes only when every OS in the matrix passes. Strict mode (branch must be up to date) is off and admins are not enforced. | The rule survives changes to the OS matrix. Strict mode costs a solo maintainer a rebase per merge. |
| D-005 | 2026-10-22 | The release workflow builds `x86_64-unknown-linux-gnu`, `aarch64-apple-darwin` and `x86_64-pc-windows-msvc` and uploads them as workflow artifacts on `v*` tags. | It is a stub. Signing, notarization, installers and GitHub Releases belong to the distribution milestone. |
| D-006 | 2026-10-22 | `.gitattributes` forces LF line endings. | Windows runners check out with CRLF by default, which fails `cargo fmt --check`. |
| D-007 | 2026-10-22 | The initial commit, `chore: scaffold QuotaDial workspace`, holds the repository setup. The skeleton code follows in separate small commits, and all of it is pushed when the GitHub repo is created. | The brief asks for both that initial commit message and small commits per step. |
| D-008 | 2026-10-22 | Until real captures exist, each provider's fixture is synthetic: the real response shape from the reference implementation, filled with obviously fake values, named `*.synthetic.json`. | The test harness exists before real captures, as the brief asks, and parses the real shape. The M1 fixture issue replaces these with redacted captures. |
| D-009 | 2026-10-22 | Code of Conduct reports go to matt@mattfowkes.com, the maintainer's commit email. | It is already public through the commit history. Swap it for a quotadial.com address once that mailbox exists. |
| D-010 | 2026-10-22 | Security vulnerabilities are reported through GitHub private vulnerability reporting, linked from the issue chooser. Blank issues are off. | QuotaDial handles OAuth credentials, so vulnerability reports must never start as public issues. |
