# Task tracker

Progress and implementation notes. Update this file with every change.

## Current focus

Kickoff (2026-10-22): GitHub setup, the CodexBar path check, then the plan for M1 issue #1 (Claude
provider), which waits for Matt's go-ahead before any provider code is written.

## Kickoff

| Step | Status | Notes |
| --- | --- | --- |
| 1. Project setup | Done | Cargo workspace, licence, NOTICE, gitignore, toolchain, editor and lint config, AGENTS.md (CLAUDE.md is a symlink to it), README, CI and release workflows, issue templates, Code of Conduct |
| Public PRD | Done | `docs/PRD.md` gitignored; `scripts/public-prd.sh` writes `docs/PRD-public.md`; `public-scope` CI job |
| 2. Skeleton code | Done | Core model and `Provider` trait as in the PRD, Claude and Codex stubs, `quotadial status` and `status --json`; 23 tests |
| 3. GitHub | In progress | Public repo `pghmatt/quotadial` created with topics and private vulnerability reporting; push, branch protection, milestones and M1 issues next |
| 4. Prepare to build | In progress | CodexBar cloned to `../codexbar-ref` (HEAD `4654968e9`, 2026-10-22); CLI login check done; PRD path check and the M1 issue #1 plan pending |

## Implementation notes

### Step 1: project setup

- Workspace members: `crates/core`, `crates/providers/claude`, `crates/providers/codex`,
  `crates/api`, `crates/cli`. Shared package metadata, dependency versions and lints live in the
  root `Cargo.toml`; every crate opts in with `[lints] workspace = true`.
- The CLI package `quotadial` has a library target `quotadial_cli` holding all logic, plus two thin
  binaries, `quotadial` (`src/main.rs`) and `qd` (`src/bin/qd.rs`). `autobins = false` keeps Cargo
  from inventing a third binary from the package name.
- `fixtures/**/raw/` is gitignored at every depth (checked with `git check-ignore`).
- Toolchain verified locally: rustc 1.99.1, clippy 0.1.99, rustfmt 1.10.0.

### Public PRD

- `scripts/public-prd.sh` applies the three rules from `docs/PUBLIC_SCOPE.md` with one awk program.
  `--check` reruns the same rules over the public copy and fails if any line would be removed, so
  the generator and the CI check cannot disagree.
- On the 2026-10-22 PRD it removes 35 lines: the 33-line "Competitive teardown" section, the
  "Pro price" open question and the open-sourcing open question. The public copy equals the private
  PRD minus exactly those lines (checked with `diff`). Lines with `$CODEX_HOME` and
  `$CLAUDE_CONFIG_DIR` stay, because a dollar amount means `$` followed by a digit.
- Verified both ways: appending a price line makes `--check` fail, and committing `docs/PRD.md` (or
  `docs/prd.md`) in a scratch repo makes the CI tracking check fail.
- CI job `public-scope`: tracking check, `shellcheck scripts/*.sh`, `scripts/public-prd.sh --check`.
  `ci-success` depends on it, so branch protection enforces it.

### Step 2: skeleton code

- `quotadial-core`: `Account`, `Window`, `WindowKind`, `Source`, `Snapshot`, `Status`, `ExtraUsage`,
  `Error`/`Result` and the `Provider` trait. Tests pin every JSON field name and enum value to the
  PRD, check that optional fields are omitted, and round-trip a snapshot. `Snapshot::unsupported`
  builds the "say why instead of 0%" snapshot.
- Providers: stubs with TODOs naming the CodexBar files to port. Tests check the synthetic fixtures
  against CodexBar's field names, the stub's `unsupported` answer, and the poll floors (5 minutes
  for Claude, 2 minutes for Codex).
- CLI: `collect` asks each provider for accounts, fetches each, and falls back to local logs when a
  fetch fails. Unit tests drive every branch with a scripted test provider. Integration tests run
  the real `quotadial` and `qd` binaries with `HOME`, `USERPROFILE`, `CLAUDE_CONFIG_DIR` and
  `CODEX_HOME` pointed at an empty directory.
- Test count: 23 (core 6, Claude 3, Codex 3, CLI unit 7, CLI binaries 4). Release build: 766 KB per
  binary on aarch64 macOS.

### Step 4: findings that shape M1

- On macOS, Claude Code keeps its OAuth credentials in the login Keychain only; there is no
  `.credentials.json` on disk. The default login is the generic password item
  `Claude Code-credentials`. A login made with `CLAUDE_CONFIG_DIR` set gets its own item named
  `Claude Code-credentials-<first 8 hex characters of SHA-256 of the directory path>` (verified),
  so one machine can hold many Claude logins. Reading another app's Keychain item shows a macOS
  access prompt, which the Claude provider has to plan for.
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
