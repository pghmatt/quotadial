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
| 4. Prepare to build | In progress | CodexBar cloned to `../codexbar-ref` (HEAD `4654968e9`, 2026-10-22). CLI login check done. PRD path check and the M1 issue #1 plan wait for `docs/PRD.md` |

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

### Step 4: findings that shape M1

- On macOS, Claude Code keeps its OAuth credentials in the login Keychain only; there is no
  `.credentials.json` on disk. The default login is the generic password item
  `Claude Code-credentials`. A login made with `CLAUDE_CONFIG_DIR` set gets its own item named
  `Claude Code-credentials-<hash>`, so one machine can hold many Claude logins. Reading another
  app's Keychain item shows a macOS access prompt, which the Claude provider has to plan for.
- Codex CLI keeps its login in `~/.codex/auth.json` (`auth_mode`, `tokens.{access_token,
  refresh_token, id_token, account_id}`, `last_refresh`). The `id_token` claims carry the email and
  `chatgpt_plan_type`, so the account and plan are readable without a network call.
- Reference request shapes from CodexBar:
  - Claude: `GET https://api.anthropic.com/api/oauth/usage` with `Authorization: Bearer`,
    `anthropic-beta: oauth-2025-04-20` and a `claude-code/<version>` user agent. The body has
    `five_hour` and `seven_day` windows (plus optional `seven_day_opus`, `seven_day_sonnet` and
    `extra_usage`), each `{ "utilization": <percent>, "resets_at": <RFC 3339> }`. HTTP 429 carries
    `Retry-After`. Source: `Sources/CodexBarCore/Providers/Claude/ClaudeOAuth/ClaudeOAuthUsageFetcher.swift`.
  - Codex: `GET https://chatgpt.com/backend-api/wham/usage` with `Authorization: Bearer` and
    `ChatGPT-Account-Id`. The body has `plan_type` and `rate_limit.primary_window` /
    `secondary_window`, each `{ "used_percent", "reset_at" (Unix seconds), "limit_window_seconds" }`,
    plus `credits`. Source: `Sources/CodexBarCore/Providers/Codex/CodexOAuth/CodexOAuthUsageFetcher.swift`.
