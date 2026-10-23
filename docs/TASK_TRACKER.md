# Task tracker

Progress and implementation notes. Update this file with every change.

## Current focus

M1 issue #1 (Claude provider) is built and in review on branch `feat/claude-provider`. Matt
approved its plan and all five recommendations on 2026-10-22 (D-021 to D-025). Next: #2 (Codex
provider). Proposals P-001 to P-009 in `docs/DECISIONS.md` wait for the manager; P-001 and P-002
are approved by Matt and already in the code.

## M1

| Issue | Status | Notes |
| --- | --- | --- |
| #1 Claude provider | In review | Reads Claude Code's sign-in, fetches `/api/oauth/usage`, 5-minute floor and 429 backoff across processes; verified live on Matt's Mac |
| #2 Codex provider | To do | |
| #3 CLI `status`, `--json`, `watch` | Partly done | `status` and `status --json` work; `watch` to do |
| #4 Redacted fixtures | To do | Claude fixtures are still synthetic (D-008) |
| #5 Live endpoint check | Partly done | Claude usage request verified live on 2026-10-22 with the PRD's headers (below). Still to do: whether the User-Agent is required, `?cedar_ember=1`, idle buckets, the profile endpoint, 429 behaviour, and all of Codex |
| #7 Signed remote schema file | To do | Split out of #1 (D-025) |

## Kickoff

| Step | Status | Notes |
| --- | --- | --- |
| 1. Project setup | Done | Cargo workspace, licence, NOTICE, gitignore, toolchain, editor and lint config, AGENTS.md (CLAUDE.md is a symlink to it), README, CI and release workflows, issue templates, Code of Conduct |
| Public PRD | Done | `docs/PRD.md` gitignored; `scripts/public-prd.sh` writes `docs/PRD-public.md`; `public-scope` CI job |
| 2. Skeleton code | Done | Core model and `Provider` trait as in the PRD, Claude and Codex stubs, `quotadial status` and `status --json`; 23 tests |
| 3. GitHub | Done | Public repo `pghmatt/quotadial` with topics and private vulnerability reporting; `main` pushed, CI green and protected (requires `ci-success`); milestones M1 to M6; M1 issues #1 to #5 |
| 4. Prepare to build | Done | CodexBar path check (below; corrections proposed as P-006); CLI login check; plan for M1 issue #1 sent to Matt |

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

### Step 3: GitHub

- Repo: https://github.com/pghmatt/quotadial (public), with the description and topics from the
  brief. Private vulnerability reporting is on, for the issue chooser's security link.
- Labels: `m1`, `provider:claude`, `provider:codex`, `provider-request`, plus `bug` and
  `enhancement`, which the issue templates apply. The new repo started with no default labels.
- Milestones M1 to M6 copy the PRD's deliverable and "done when" text; due dates per D-020.
- M1 issues: #1 Claude provider, #2 Codex provider, #3 CLI, #4 redacted fixtures, #5 live endpoint
  check. The provider stubs' TODOs reference #1 and #2.
- Branch protection on `main`: required check `ci-success`, strict mode off, admins not enforced,
  no force pushes or deletions (D-004). This tracker update was the first change merged through a
  pull request under it.

### Step 4: CodexBar path check

Every file the PRD's "Leveraging CodexBar" table names exists at CodexBar `4654968e9`. Only
`ClaudeConfigPaths.swift` is in a different folder: `Providers/Claude/`, not
`Providers/Claude/ClaudeOAuth/`, and it has always been there. Several descriptions are out of date;
P-006 lists the corrections. The ones that change the M1 plan:

- `UsageFetcher.swift` no longer reads `~/.codex/sessions`. Since November 2025 it asks
  `codex app-server` over JSON-RPC, so the Codex session-log fallback has nothing to port.
- CodexBar never refreshes tokens owned by Claude Code or the Codex CLI. It hands refresh back to
  those CLIs, which matches the PRD's rule.
- CodexBar reads only the default `Claude Code-credentials` Keychain item, and only after the user
  opts in, because Claude Code rewrites the item's access list on every refresh (P-007).
- CodexBar has no generic pass-through for unknown Claude buckets. The PRD's "unknown buckets
  appear by name" requirement is new work, not a port.
- The Claude request starts with `User-Agent: claude-cli/<version> (external, cli)` and
  `?cedar_ember=1`, retrying with `claude-code/<version>` and no query on 400, or on 403 without
  `user:profile`.

### M1 #1: Claude provider

Modules in `crates/providers/claude/src/`, each naming the CodexBar files it follows:

| Module | Job |
| --- | --- |
| `environment.rs` | Where Claude Code keeps its sign-in: config dir, `.credentials.json`, the Keychain service name (`Claude Code-credentials`, plus `-<8 hex of SHA-256 of the NFC dir>` when `CLAUDE_CONFIG_DIR` or `CLAUDE_SECURESTORAGE_CONFIG_DIR` is set), the account config file |
| `login.rs` | Finds the sign-in this shell would use (env token, then Keychain, then file) without reading any secret |
| `keychain.rs`, `process.rs` | `/usr/bin/security` lookups with timeouts (5 s for attributes, 30 s for the secret) |
| `credentials.rs`, `secret.rs` | Parses `claudeAiOauth` (also hex-encoded output); the token lives in a `Secret` whose `Debug` is redacted |
| `identity.rs`, `plan.rs` | Account and plan from `oauthAccount`, or from `/api/oauth/profile` |
| `api.rs` | The HTTP requests, response classification, `claude --version` for the User-Agent |
| `usage.rs` | Response buckets to windows |
| `gate.rs`, `state.rs` | The 5-minute floor, 429 backoff, replay from cache, the per-account state file and lock |
| `advice.rs` | The one sentence shown when there are no numbers, naming the fix |

- Checked against Claude Code 2.1.286 itself (the installed binary), not only CodexBar: the
  Keychain read command, the service and account naming, the credentials file location, the
  keychain-then-file order, the plain usage request and its headers, and the profile response
  shape. Differences from CodexBar are proposals P-007 to P-009.
- Live check on Matt's Mac, 2026-10-22 23:47Z, with the account Claude Code uses in the Brain's
  shell: `quotadial status --json` exited 0 in 0.88 s with no macOS prompt and printed a Max
  account with Session 2%, Weekly 5% and Weekly (Fable) 0% (the last from `limits[]`), each from
  the API with its reset time. A second run (`qd status`) a minute later replayed the cache without
  a request. The cache file is mode 0600 in a 0700 directory and holds no token. Re-checked after
  the security review fixes (2026-10-23 00:12Z): fresh numbers from the API in 0.8 s, then a cached
  replay in 0.06 s.
- Test count: 148 (core 9, Claude 122, Codex 3, CLI unit 10, CLI binaries 4). The Claude tests run
  against a local HTTP server (wiremock) with a fake keychain, a manual clock and a temporary cache
  directory, so no test touches the real Keychain, Claude Code's files or the network. The macOS
  test that runs `security` looks up a made-up item by attributes only.
- Linux and Windows lints are checked locally by building with the macOS-only Keychain code
  switched off (a simulated non-macOS `cfg`), because CI is the only real non-macOS build.
- Security review (an independent agent, read-only, 2026-10-22). It found no token leakage, no
  credential writes and no subprocess injection, and judged the locking correct on all three
  platforms. It found eight floor and account-separation gaps, all fixed with tests:
  1. A timed-out or unreadable answer gave the floor back. Now only a failure to connect does, and
     a 429 or 401 is judged before its body is read.
  2. Every environment token shared one account id, so two shells with different tokens shared
     saved numbers. Fixed with a token hash (D-031). Its other half, separate floors for an
     environment token and a stored sign-in to the same account, is a known limit until M2.
  3. A first request stopped midway did not hold the floor.
  4. The account was read before the lock, and not re-checked after the token was read.
  5. A clock moved back skipped the floor and the 429 block. Saved times are now re-anchored.
  6. The early retry after a rejected sign-in had no cap. Now once per floor.
  7. A failed save after an answer was ignored. Now it is retried, then fails the fetch.
  8. Two waits had no deadline: a subprocess's output, and the account lock (now 2 minutes).
- `fixtures/claude/oauth-usage-scoped.synthetic.json` uses a made-up key,
  `seven_day_example_model`, to show an unknown bucket appearing by name.

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
