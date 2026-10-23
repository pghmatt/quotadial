# Task tracker

Progress and implementation notes. Update this file with every change.

## Current focus

The manager (a claude.ai session acting for Matt) set the order on 2026-10-23: #10 (the PR #8
review fixes, a blocker) first, then the ChatGPT/Codex provider (#2), then `watch` (#3), then #11
to #15. Each one: branch off `main`, self-review, live check, PR, merge when CI is green. "Stay on
claude" meant the model, not the task. P-001 to P-009 are accepted and in the PRD, P-010 is decided
(D-034), and P-011 and #7 are deferred (D-035). The manager's instructions arrive as GitHub issues,
PR reviews and comments, checked at the start of every turn.

## M1

| Issue | Status | Notes |
| --- | --- | --- |
| #1 Claude provider | Done (PR #8) | Reads Claude Code's sign-in, fetches `/api/oauth/usage`, 5-minute floor and 429 backoff across processes; verified live on Matt's Mac |
| #2 Codex provider | To do | |
| #3 CLI `status`, `--json`, `watch` | Partly done | `status` and `status --json` work; `watch` to do |
| #4 Redacted fixtures | Claude done (PR #9) | Live usage and redacted profile captures committed (D-033); Codex to do |
| #5 Live endpoint check | Claude done (PR #9) | Claude checked live on 2026-10-23 (D-032): User-Agent not required (P-010), `?cedar_ember=1`, idle buckets, the profile, today's shape (P-011). A 429 was not provoked on purpose. Codex to do |
| #7 Signed remote schema file | Deferred (D-035) | Split out of #1 (D-025) |
| #10 Review fixes from PR #8 | In review | Real-Keychain test gated, tokens redacted from error text (D-036), damaged state keeps the floor (D-037) |
| #11 `spawn_blocking` for version detection and Keychain find | To do | After #2 and #3 |
| #12 No `claude` spawn for the User-Agent version | To do | After #2 and #3 |
| #13 Surface unknown buckets without a known prefix | To do | After #2 and #3; coordinate with #7 |
| #14 Test gaps: two orgs, bounded profile requests, no token in the cache | To do | After #2 and #3 |
| #15 Zeroize secrets, fixed `JoinError` message | To do | After #2 and #3 |

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
  test that runs `security` looks up a made-up item by attributes only; since #10 it runs only on
  request.
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
- Merged into `main` on 2026-10-23 as squash commit `97ffcae` (PR #8), with CI green on Linux,
  macOS and Windows. The merge closed #1.

### M1 #4 and #5: Claude live checks and fixtures

- Five live requests on 2026-10-22 and 2026-10-23, at least 5 minutes apart. They were sent by a
  small script outside the repo that reads the sign-in as QuotaDial does, never prints the token,
  refuses to run within 5 minutes of the previous request (its own or QuotaDial's), and saves raw
  responses only to the gitignored `fixtures/claude/raw/`. Results are in D-032.
- Anthropic's usage response has grown well past what CodexBar decodes: per-model limits moved to
  `limits[]`, and there are `spend` (usage credits), `seven_day_breakdown` and 17 code-named
  fields. QuotaDial's mapping needed no change for them, because it shows only objects with a
  `utilization` and `weekly_scoped` limits. It now scales extra usage by `decimal_places`.
- The profile fallback now reads the plan from `organization.organization_type`.
- Windows Credential Manager (P-008): Claude Code 2.1.286 names a `windows-credman` store but the
  binary does not show the credential's target name. Building it now would mean guessing, so P-008
  waits until someone can check a real Windows sign-in with the flag on.
- Test count: 152 (core 9, Claude 126, Codex 3, CLI unit 10, CLI binaries 4).
- PR #9 was stacked on #8. Deleting #8's branch after its merge closed #9 instead of retargeting
  it, so the branch was restored, #9 reopened and pointed at `main`, and the branch deleted again.
  `main` was then merged into `feat/claude-live-checks`; the content is unchanged, because `main`
  after the squash is identical to #9's old base. For stacked PRs, point the child at `main`
  before deleting the parent's branch.

### M1 #10: review fixes from PR #8

The manager's review of PR #8 asked for three fixes before merging; PR #8 was merged without them,
so they became #10.

- `keychain.rs`: the test that runs the real `/usr/bin/security` is `#[ignore]`d and also returns
  early unless `QUOTADIAL_REAL_KEYCHAIN_TESTS=1`, so CI and plain `cargo test` never touch the login
  keychain. Run it with
  `QUOTADIAL_REAL_KEYCHAIN_TESTS=1 cargo test -p quotadial-provider-claude -- --ignored`; it passed
  on Matt's Mac on 2026-10-23.
- Token redaction (D-036): `quotadial_core::redact_secrets` replaces `sk-ant-…` tokens with
  `[redacted token]`. `api.rs` `error_message` applies it before shortening the message, so a token
  cut at 200 characters cannot slip through in part. The CLI applies it again in `collect`, to
  snapshot errors and provider errors, for text and JSON output alike. Tests: a fake token in a 401
  body and in a 400 body never reaches the snapshot, the error, the printed JSON or the cache files;
  a provider that leaks a token in its error text never gets it to the CLI's output.
- State load (D-037): a missing file starts afresh; a file that does not parse in full keeps its
  last request time, 429 block and 429 count, or counts as a request sent now when none can be
  read; the recovered state is saved at once; an unreadable file (an I/O error) fails the fetch
  with no request. Tests cover a cut-off file, a file from another format version, a snapshot change
  without a version bump (the block holds) and a directory where the file should be.
- The PRD names two M1 follow-ups that have no issue yet: `Window.unconfirmed` for the unconfirmed
  0% rule (P-005) and Windows Credential Manager (P-008).
- Test count: 162 (core 12, Claude 132 plus the opt-in Keychain test, Codex 3, CLI unit 11, CLI
  binaries 4).

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
