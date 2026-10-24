# Task tracker

Progress and implementation notes. Update this file with every change.

## Current focus

The manager (a claude.ai session acting for Matt) set the order on 2026-10-23: #10, then the
ChatGPT/Codex provider (#2), then `watch` (#3), then #11 to #15. Each one: branch off `main`,
self-review, live check, PR, merge when CI is green. #10 is done (PR #16), #2 (PR #17, with the
Codex halves of #4 and #5), #3 (PR #18), #11, #12, #14 and #15 (PR #19), and the PRD's
unconfirmed-0% rule, #20 (PR #21), and #13 with P-012 (PR #22). M1 is done but the captures that
cannot be made here (#4, #5), #7 (deferred) and the Windows Credential Manager check (P-008).

M2 (manager's go-ahead, 2026-10-23) came in three PRs: the daemon and its store (#23, PR #26), the
local API and its token (#24, PR #27), and the MCP server and SKILL.md (#25, PR #28). M2 is done.

M3, the tray app: the manager approved the plan in #29 on 2026-10-23 (D-065). Its PRs:
- PR 1, the scaffold with the daemon running inside the app (PR #31);
- PR 2, the popover (PR #32);
- PR 3, settings and notifications (PR #33);
- PR 4, builds for macOS, Windows and Linux (PR #35);
- PR 5, every stored Claude sign-in as its own account, added by the manager in #34 (PR #36);
- a fix found during PR 5's live check: the daemon now polls a moving account at its floor (PR #37);
- the two fixes from the manager's review of PR #36, which came in after it merged: a Keychain that
  cannot be listed is searched folder by folder, and the listing has a longer timeout (PR #38).

M3 is done. Both parts of its test pass on Matt's Mac:
- the release workflow builds a dmg, an msi, a deb and an AppImage;
- a first launch with an empty data folder shows a number in the tray after 1.0 second, with no
  typing.

The review's six smaller items (#39) are done in PR #42. With Matt's go-ahead, `v0.1.0` (M1 to M3)
is tagged at 00291e5. The Release workflow built its draft pre-release with the dmg, the msi, the
deb, the AppImage and the CLI for four platforms. With Matt's yes, relayed on #49, it was published
as a pre-release on 2026-10-24 at 21:34 UTC.

M4, alerts and history: the manager approved the plan in #40 on 2026-10-23, with six decisions and
two additions (D-090, D-091). Its PRs, in order:
- PR 1, the pace of every window (P-016, D-092), merged (PR #43);
- PR 2, alert rules written in plain words, and the one `tier()` check (D-093), merged (PR #44);
- PR 2b, alerts sent to a webhook and Slack (D-094), merged (PR #45);
- PR 3, history to 30 days (365 as a setting), `quotadial history` and a history view (D-095),
  merged (PR #46);
- PR 4, the providers' status pages and an incident badge (D-096), merged (PR #47);
- PR 5, the week's API-equivalent cost, from a bundled price table (D-097), merged (PR #48).

M4 is done. Its test passes:
- the rules engine's tests with a manual clock (jumps forward and back, a 12-hour sleep, a restart
  before delivery) give one firing per window;
- on Matt's Mac, a reset across a daemon stop and start was told once (PR #44).

Since 2026-10-24 01:15 UTC, Matt's app runs `main` at fd3ee4d, built from the same tree and swapped in
for the one built at d0e4bc6. It opened his history at schema 5, showed all nine accounts, and
showed the week's cost 11 seconds after it started.

`v0.2.0` (M4), with the same yes: the version was raised from 0.1.0 to 0.2.0 in the workspace, the
app's `Cargo.toml`, `package.json` and `tauri.conf.json`, and both lockfiles, and the price table
was refreshed from models.dev as AGENTS.md asks before a tag (only its date changed), in PR #53.
`main` is tagged at db7c9d3, and the Release workflow's draft, with all 8 files, was published as a
pre-release with the M4 notes on 2026-10-24 at 21:58 UTC. A first
measure for #52's list, from Matt's Mac before the tag: the app's own process, which holds the tray
and the daemon, had used 0.28% of one core on average over 20 hours 22 minutes (3 min 24 s of CPU)
and was 91 MB resident; the webview's processes are not counted. The full set of #52's measurements
comes with the docs PR on #49's queue.

M5, licensing and release: the manager approved the plan in #49 on 2026-10-24, with decisions (a) to
(e) (D-098), and set the queue on its 17:50 UTC comment. #51 lists what only Matt can provide, and
#52 is the standing quality bar: every PR's description ends with what going down it found. The
queue, by its numbers:
- 1, licence keys (D-099), merged (PR #55);
- 2, the Free and Pro gates (D-100), in review (PR #56);
- 3, help and feedback, with `quotadial doctor` and `CHANGELOG.md`;
- 4, checkout in Lemon Squeezy's test mode, with the signing step proposed on #54;
- 5, the website in `site/`, built but not deployed, and the shareable image;
- 6, signing and updates, each turning on when its secrets exist;
- 7 to 12, hardening: resilience, security, accessibility, the OpenAPI spec, package managers, and
  docs;
- then the M6 plan, starting with a fake provider while it waits.

Proposals:
- P-001 to P-009 are accepted and in the PRD.
- P-010 is decided (D-034).
- P-011 and #7 are deferred (D-035).
- P-012 and P-014 are in the code.
- P-013 is the manager's PRD edit.
- P-015 (`Account.sign_in`) is accepted (#29, #34) and in the code (D-079).
- P-016 (`Window.pace`) is accepted (#40) and in the code (D-092).

The manager's instructions arrive as GitHub issues, PR reviews and comments, checked every turn.

## M1

| Issue | Status | Notes |
| --- | --- | --- |
| #1 Claude provider | Done (PR #8) | Reads Claude Code's sign-in, fetches `/api/oauth/usage`, 5-minute floor and 429 backoff across processes; verified live on Matt's Mac |
| #2 Codex provider | Done (PR #17) | Reads `auth.json`, fetches `wham/usage`, windows by length, session-log fallback, 2-minute floor through the shared gate; verified live on Matt's Mac |
| #3 CLI `status`, `--json`, `watch` | Done (PR #18) | Relative times, tightest window first, `watch` and `watch --json`, errors that name the fix; output and exit codes in `docs/CLI.md` (D-047) |
| #4 Redacted fixtures | Nearly done (PR #9, PR #17) | Claude: live usage and redacted profile (D-033). Codex: redacted live Pro response and session log (D-039, D-042). A test fails if a committed fixture holds a token-like string or an email outside `example.com`. Left: a Plus capture (there is no Plus account to capture, so its shape stays synthetic) and Claude's 429 and 401 (not provoked on purpose) |
| #5 Live endpoint check | Done but one check (PR #9, PR #17) | Claude (D-032): User-Agent not required (P-010), `?cedar_ember=1`, idle buckets, the profile, today's shape (P-011); a 429 was not provoked on purpose. Codex (D-039): headers, the response shape (P-003 confirmed, P-013), Pro's single weekly window, the 401 body |
| #7 Signed remote schema file | Deferred (D-035) | Split out of #1 (D-025) |
| #10 Review fixes from PR #8 | Done (PR #16) | Real-Keychain test gated, tokens redacted from error text (D-036), damaged state keeps the floor (D-037) |
| #11 `spawn_blocking` for version detection and Keychain find | Done (PR #19) | The version spawn is gone (#12); the Keychain lookups in `fetch` run in `spawn_blocking` (D-048) |
| #12 No `claude` spawn for the User-Agent version | Done (PR #19) | A fixed `claude-code/2.1.286` (D-048) |
| #13 Surface unknown buckets without a known prefix | Done (PR #22) | Unknown buckets are windows of kind `other`, labelled from their key (P-014, D-052); the Codex credit balance went in with it (P-012) |
| #14 Test gaps: two orgs, bounded profile requests, no token in the cache | Done (PR #19) | The profile is now asked once per token (D-049); the three tests added, the one that could not fail removed |
| #15 Zeroize secrets, fixed `JoinError` message | Done (PR #19) | Secrets wiped on drop, no copies of the refresh token, fixed message (D-050) |
| #20 Unconfirmed 0% (P-005) | Done (PR #21) | A sudden 0% before the reset is flagged until a second fetch agrees, in the shared gate (D-051) |

## M2

| Issue | Status | Notes |
| --- | --- | --- |
| #23 Daemon with one shared poller and SQLite history | Done (PR #26) | `quotadial daemon`; adaptive interval; WAL history in the OS data directory; one daemon at a time (D-053, D-054) |
| #24 Local API on 127.0.0.1 with a bearer token; the CLI asks the daemon | Done (PR #27) | `GET /v1/status` and `GET /v1/headroom` on `127.0.0.1:47115`, a private bearer token and a host check; `status` and `watch` ask a running daemon and fetch for themselves only when none runs (D-056 to D-061) |
| #25 MCP server and SKILL.md | Done (PR #28) | `quotadial mcp`, the `quotadial` MCP server with `get_usage`, `get_account_headroom` and `list_alerts`; `skills/quotadial/SKILL.md` documents the HTTP API for agents (D-062 to D-064) |

## M3

| Step | Status | Notes |
| --- | --- | --- |
| PR 1: scaffold, the daemon in the app, the bundled CLI | Done (PR #31) | Tauri 2 app in `apps/tray`; runs or follows the daemon through the shared `Host`; the tray title and icons (D-066); clean handover on quit (D-068); `docs/INSTALL.md` |
| PR 2: popover | Done (PR #32) | Opens beside the tray, hides on blur or Escape, fits its content; every window with a meter, reset, source and age; the macOS popover material (D-072, D-073); P-015 asked on #29 |
| PR 3: settings and notifications | Done (PR #33) | Settings panel in the popover; launch at login; show or hide providers; the two notifications, each once per occurrence (D-074, D-075) |
| PR 4: builds | Done (PR #35) | The Release workflow builds the CLI for four platforms and the app as a universal dmg, an msi, a deb and an AppImage; a tag drafts a pre-release (D-076) |
| PR 5: every stored Claude sign-in (#34) | Done (PR #36) | Every Claude Code sign-in on the machine is an account, one row per subscription, each naming its sign-in (P-015) and the folder to sign in from; `config.json` with `extra_config_dirs` and `max_accounts`; names for accounts in the app; a lasting order; one floor per subscription whichever sign-in asks (D-077 to D-086) |
| Fix: poll at the floor | Done (PR #37) | The daemon counts each account's next poll from when its answer came back, so a moving account is asked every 5 minutes, not every 10 (D-087) |
| Fix: the review of #36 | Done (PR #38) | A failed Keychain listing keeps this shell's own sign-in and finds each folder's item by name, with a warning in `errors`; the listing gets a 20-second timeout (D-088) |
| Follow-ups from the review of #36 (#39) | Done (PR #42) | The named account kept before the request; relative extra folders left out with a warning; folders matched however they were written; `max_accounts` counts subscriptions; items with no time never left out; the two-items test (D-089) |

## M4

| Step | Status | Notes |
| --- | --- | --- |
| PR 1: pace | Done (PR #43) | `Window.pace`: the share used per hour and, when it comes before the reset, when the window runs out, fitted by the daemon to each window's readings since it reset; a `readings` table in the history (schema 2); a line under each window in the CLI and the popover; a direct fetch says pace needs the daemon (P-016, D-092) |
| PR 2: alert rules | Done (PR #44) | Rules in plain words read by a local grammar; a threshold, a reset, running out, a time before a reset and a sign-in; checked by the daemon and told once per window across restarts, sleeps and clock changes; M3's two notifications as starting rules, quiet in an older history; `quotadial alerts`, `/v1/alerts`, `list_alerts`; the app's notifications and Settings; `tier()` (D-093) |
| PR 2b: alerts to webhooks and Slack | Done (PR #45) | `alert_webhooks` and `slack_webhook` in `config.json`, read by the daemon crate's `Config`; each told alert sent with only the firing, retried for an hour; addresses kept secret and named by host; a warning for a webhook that is not `https` or a file others can read; `tier()` gates it (D-094) |
| PR 3: history | Done (PR #46) | `history_days` up to 365 (Pro past 30); `History::series`; `GET /v1/history`; `quotadial history` with a chart per window, `--csv` and `--json`; the app's History panel with charts, an hour-by-day heatmap and Export CSV (D-095) |
| PR 4: provider status | Done (PR #47) | The daemon reads status.claude.com and status.openai.com every 5 minutes for providers with accounts; `status_pages` in the report; the incident rule, once per incident; a line in `status`, ⚠ beside the tray's number and the incident in the popover; `status_pages` setting (D-096) |
| PR 5: the week's cost | Done (PR #48) | `quotadial-cost` reads the usage fields of Claude Code's and the Codex CLI's logs into `costs.db`, once per line, and prices the last 7 days from `prices.json` (models.dev, refreshed by `scripts/update-prices.sh`); `costs` in the report and `/v1/headroom`; a line per provider in `status`, the MCP tools and the popover; `cost_scan` setting; `ProFeature::CostScan` (D-097) |

## M5

| Step | Status | Notes |
| --- | --- | --- |
| 1: licence keys | Done (PR #55) | `QD1` keys: a licence (`id`, `plan`, `issued`, `until`) signed with Ed25519 and checked on the machine against the public key core carries; `Standing`; `licence` in the data folder; `quotadial licence [show \| add \| remove]`; Settings' Licence field; `quotadial-issue` and QuotaDial's signing key; `docs/LICENSING.md`; `tier()` still Pro until the gates (D-099) |
| 2: Free and Pro | In review (PR #56) | `Licensing`, the one switch, read from the key at every ask; Free shows one account per provider, keeps 30 days, checks threshold, reset and sign-in rules, sends no webhooks and reads no logs; headroom and the alert rules through the API and MCP are Pro's; `left_out` and a line where each feature would be; a key counts without a restart (D-100) |

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

### M5 PR 2: Free and Pro (D-100)

- **The switch** (`crates/daemon/src/licence.rs`): `Licensing::new(Option<&DataDir>)` with
  `standing()`, `tier()` and `allows(feature)`, reading the key each time, and
  `account_limit(setting)`, an `AccountLimit` that is the setting on Pro and 1 on Free.
  `left_out(tier, providers, costs)` gives what a Free copy leaves out. Core's process-wide `tier()`
  is gone; `ProFeature` gains `Accounts`, `Headroom` and `RulesApi`, with `what()` for lists and
  `line()` for the one line, and `PRO_URL` is `quotadial.com/pro`.
- **Accounts** (core `AccountLimit`, `Provider::held_back`): the Claude provider asks its limit at
  each discovery and counts the subscriptions it left out, so a key added shows every account at
  the next discovery with no new provider. `quotadial_providers::with_config` takes the licensing.
- **The poller:** `Poller::new` takes a `Licensing`. Every round it asks the tier; a change sends it
  to rediscover and to scan or clear the costs at once. On Free it prunes history to at most 30
  days (`History::prune_to`), checks only Free's rule kinds (`History::check` and `add_rule` take
  the tier; `needs_pro`), sends nothing to webhooks (`deliver_due` no longer asks), leaves the logs
  unread and clears costs, and keeps `left_out` after each discovery: held-back accounts, and the
  cost when the scan is on and there are accounts. `Latest::nudge` wakes it at once.
- **The API and MCP:** a `pro_only` route layer answers 403 `{error, pro_feature}` for
  `/v1/headroom` and `/v1/alerts*` on Free; status and history answer on every copy. `mcp::serve`
  takes the licensing, and its two Pro tools answer a tool error that points to `get_usage`.
  `Host` keeps the licensing, builds the cost scan whenever the setting is on, and passes the
  licensing to the poller and the API.
- **The CLI:** a direct fetch adds `left_out` as the daemon does; `status` and `watch` print
  held-back accounts under the accounts and the cost line where costs go, and the no-daemon pace
  line leaves out costs on Free. `alerts` marks waiting rules and names webhooks as Pro's; `alerts
  add` refuses Pro kinds; `history` past 30 days says why; `licence` on Free lists what Pro adds;
  the daemon's first lines say what Free leaves out.
- **The app:** `Meter.left_out` under the accounts; Settings marks a waiting rule, names webhooks
  as Pro's, and lists what Pro adds in the Licence section; `add_licence` and `remove_licence`
  nudge the daemon the app runs (`feed::run` takes a `Notify`); a hidden provider's held-back
  accounts are hidden too. The window's JavaScript is 24.4 KB.
- **Docs:** `docs/CLI.md` (QuotaDial Free and Pro, and the Pro features in Accounts, Settings,
  Alerts, the week's cost, History, the API, MCP, text and JSON output), `docs/INSTALL.md`, the
  README, SKILL.md and AGENTS.md.
- **Tests:** 482 in the workspace (13 new) and 35 in the app (1 new). Tests that use Pro features
  keep a test-key licence in their own data folder; Free tests sit beside them, including a key
  added while the CLI's daemon runs, which turns `/v1/headroom` from 403 to 200 with no restart.

### M5 PR 1: licence keys (D-099)

- **The key** (`crates/core/src/licence.rs`): `QD1.<licence>.<signature>`, the licence as base64url
  JSON and the Ed25519 signature of `QD1.<licence>`, both without padding. `Licence::verify` leaves
  out what copying adds (spaces, line breaks, zero-width characters, soft hyphens), checks the
  prefix in any case, decodes a 64-byte signature, verifies it strictly against each trusted key,
  then reads the licence. `KeyError` is `NotAKey`, `DoesNotCheckOut` or `Newer` (a later `QDn`
  format, or a plan this version does not know), each a sentence naming the fix. `Licence::sign`
  writes the JSON with sorted keys and no spaces, and signing is deterministic.
  `holds_on(today)` is true through `until`; `today()` is the earlier of the local and the UTC
  date. `Standing::of(key, today)` gives the tier, the licence and one sentence.
- **Trust:** `QUOTADIAL`, the public half of the key made by `quotadial-issue keygen` on Matt's Mac
  on 2026-10-24. A debug build also trusts `TEST_SEED`'s key, so tests sign real keys; CI's Linux
  check job runs `only_a_debug_build_trusts_the_test_key` in a release build too.
- **Keeping it** (`crates/daemon/src/licence.rs`): `licence` in the data folder, on one line, written
  through `write_private`, which moved from the tray into the daemon crate and now writes the API's
  token too. `add` refuses a key that gives nothing or whose licence has ended, keeping the one
  before; `standing` reads the file each time, so an ended licence turns the copy back to Free by
  itself; an empty file is no key; `remove` says whether there was one.
- **The CLI:** `quotadial licence` (alias `license`) with `show [--json]`, `add [<key>...]` and
  `remove`. With no key, `add` reads a pipe whole, or lines at the prompt until `licence::whole`
  says the signature is complete, a blank line, or text that cannot be a key.
- **The app:** `SettingsView.licence` (`text`, `kept`), the `add_licence` and `remove_licence`
  commands, and a Licence fieldset in Settings: the sentence, a field and Add, and Remove when a key
  is kept. The window's JavaScript is 24.0 KB.
- **`quotadial-issue`** (`crates/issue`): `keygen` (a PKCS #8 PEM file, 0600 in a 0700 folder,
  never replaced, never printed), `public`, `sign --id [--issued] [--until] [--test]` (refusing a
  signing key core does not carry, an id that is not 1 to 64 printable characters, or an `until`
  before `issued`), and `check`.
- **Docs:** `docs/LICENSING.md` (the format, trust, the two test-key keys core checks it signs byte
  for byte, the tool, changing the key), `docs/CLI.md` (Licence), `docs/INSTALL.md` (Settings),
  AGENTS.md. #51 gained a checkbox for Matt to back up the signing key, and #54 proposes how a buyer
  gets a key: a function on the website that turns a paid Lemon Squeezy key into a signed one.
- **Tests:** 469 in the workspace (33 new), with property tests that no text makes the reader
  panic and that every signed licence reads back. Their first run found a panic, slicing a format
  inside a multi-byte character, now fixed. 34 in the app (1 new).

### M4 PR 5: the week's cost (D-097)

- **Prices** (`crates/cost/src/prices.rs`): `scripts/update-prices.sh [<api.json>]` turns
  models.dev's `api.json` into `crates/cost/prices.json` with jq: Anthropic's and OpenAI's models,
  each `input`, `output`, `cache_read`, `cache_write` and `above { tokens, ... }`, per million
  tokens; `above` is the smallest context tier, or the older `context_over_200k` at 200,000
  tokens. It was run on 2026-10-23 (17 Anthropic and 47 OpenAI models). `Price::of(tokens)` prices
  one request: the `above` price for all of it when its context (input, cache read and cache write)
  is over `above.tokens`, the input price for a missing cache price, and twice the input price for
  Claude's hour-long cache writes. `Prices::of(provider, model)` matches exactly, then without
  Bedrock's, Vertex's or OpenAI's spelling, then without a date. NOTICE credits models.dev (MIT).
- **Reading** (`read.rs`, `claude.rs`, `codex.rs`): `read::lines` hands each complete line from an
  offset, in steps of a byte budget, skipping lines over 64 MiB unkept, and leaves a last line
  still being written. A line is parsed only when `memchr` finds the words its kind must have, into
  structs with only the usage fields (serde skips the rest). Claude: an `assistant` line with
  `message.usage` and `message.model`, keyed `claude:{message.id}:{requestId}` or by `sessionId`;
  a proxy's estimate is skipped. Codex: `token_usage_record` keyed `codex:{response_id}`, the model
  from the session's latest `turn_context`, the cache taken out of `input_tokens`; `token_count`
  events are only counted, to name sessions from older CLIs.
- **Store** (`store.rs`, `costs.db` in the data folder, 0600): `files` (path, length, how far read,
  the Codex session's model and counts) and `requests` (the SHA-256 of the key, cut to 8 bytes, as
  `INTEGER PRIMARY KEY`; provider, time, model, five token counts), upserted keeping the part with
  the most output. Requests older than 8 days are pruned, and a log neither listed this week nor
  found is forgotten. Another schema version starts the cache again.
- **Scan** (`scan.rs`): `CostScan::scan(folders, now)` resolves each folder (so Codus's linked
  `projects` folders are read once), lists the logs written in the week, reads what each gained
  (from the start if it got shorter), keeps each 256 MiB step as it finishes, and adds up the week
  per provider and model, pricing each request on its own. Notes name unpriced models, requests
  with no model, and Codex sessions with no records.
- **Daemon:** `Provider::log_folders` (Claude: `projects` in this shell's folder, the default and
  every candidate; Codex: `sessions` and `archived_sessions`). `Poller::with_costs` scans every
  `COSTS_EVERY` (10 minutes) in a blocking task behind a `tokio::sync::Mutex`, for providers with
  accounts, and keeps the costs in `Latest`; a failed scan keeps the last costs, which carry their
  age. `Host::start` turns it on unless `"cost_scan": false`, when `tier()` allows `CostScan` and
  `costs.db` opens, and `quotadial daemon` says so as it starts, with how to turn it off. Hosts in
  tests set `"cost_scan": false` beside `"status_pages": false`, so no test reads this machine's
  logs.
- **Where it shows:** `StatusReport.costs` and `HeadroomReport.costs` (left out when empty); a line
  per provider in `status` and `watch`, each model's part when there are several, and the notes;
  the MCP tools' descriptions; the popover's section under the accounts, with "By model", and a
  line per provider in the tray's menu and tooltip. Shared wording in `text::cost`,
  `text::model_cost`, `text::usd`, `text::plural` and `text::thousands`.
- **Tests:** prices (each kind, the cache fallback, hour-long writes, the long-context tier, cloud
  spellings and dates, the bundled table pricing the models in use), the line reader (offsets, a
  partial last line, long and too-long lines, the budget), Claude and Codex lines (keys, tokens,
  estimates, broken or unrelated lines, the model from the turn), the store (most output wins, the
  keeping time, restart, another version, 0600), scans of synthetic logs (streamed parts, copies,
  subagents and a workflow's subagents, a linked folder, the long-context price, unrecorded
  sessions, appended and rewritten logs, the week only), each provider's log folders, the poller
  reading only providers with accounts, the setting, the CLI's lines, headroom, the MCP filter, the
  tray's view and hidden providers.
- **Live check on Matt's Mac:** a test daemon from this branch, built for release, on a copy of
  the data folder. Its first scan read the week's logs (283 Claude Code files, 11.5 GB, and 117
  Codex files, 10.9 GB) in about 13 seconds, with the files in the page cache, and made a 17 MB
  `costs.db`. For the week to 2026-10-24T00:30:45Z it priced three Claude models and three Codex
  models, every model in use. An independent count in Python, written from the same rules but
  sharing no code, agreed on every model's requests, all five token counts and the dollars to the
  cent. (The amounts went to Matt, not here.) Ten minutes later the second scan read only the new
  lines, in under a second. The app, built under another name and following the test daemon,
  showed both providers' lines under the accounts.
- **Mutation checks:** breaking each of 16 rules in turn fails a test. Two slipped through at
  first, and their tests were made sharper: the week's start (a request kept for its eighth day
  but not counted) and a log that got shorter (its new request must count).

### M4 PR 4: provider status (D-096)

- **Core:** `StatusPage` (provider, indicator, description, url, since, checked_at) and
  `Indicator` (ordered `none` < `maintenance` < `minor` < `major` < `critical`, with
  `is_incident()` for `minor` or worse). `StatusReport.status_pages` is left out of JSON when
  empty. The field was added to the 33 report literals with the compiler's help; the two outside
  tests (`Settings::shown`, the MCP's `only`) now filter the pages by provider rather than drop
  them, each with a test.
- **Reading** (`crates/daemon/src/status_pages.rs`): `parse` reads `api/v2/status.json` and
  carries `since` from the reading before while the trouble lasts. `StatusReader` uses a
  10-second timeout and no redirects. The last reading per provider is kept in `status_pages`
  (schema 5) and loaded when the poller starts.
- **Poller:** `with_status_pages(pages)` is opt-in, and `Host::start` turns it on unless
  `"status_pages": false`. Every `STATUS_EVERY` it spawns one reading task (a `tokio::sync::Mutex`
  guard) for the providers it has accounts with. The task keeps each page and then checks the
  alert rules.
- **The incident rule:** `Condition::Incident`, read from `incident`, `outage` or `down`, with a
  provider or none. `Rule::incidents` gives one occurrence per page in trouble, keyed by `since`.
  `History::check` records it under the provider's name. `tier()` gates it.
- **Where it shows:** a line in `status` and `watch` for a page that is not all clear. ⚠ beside
  the tray's number, and `Status: …` in its menu and tooltip. The incident under the popover's
  headline.
- **Tests:** both live fixtures, `since` across a trouble and after it, maintenance not an incident,
  the last reading across a restart, the setting, the poller reading only providers with accounts
  from a local page, the incident rule (grammar, scope, once per incident, again for the next), the
  CLI line, the tray mark, and pages kept through both filters.
- **Live check on Matt's Mac:** the app, built under another name, ran its own daemon on a copy
  of the data folder. At 23:41 UTC it read both pages: Claude "major: Partial System Outage" and
  Codex "minor: Partial System Degradation" (both pages had changed since the fixtures were
  taken at 23:15). Both were kept with their `since`, and the popover named both under the
  headline.

### M4 PR 3: history (D-095)

- **Keeping:** `History::keeping(days)` (1 to `MAX_KEEP_DAYS`, 365) sets what `prune` keeps.
  `Config::history_days` comes from `history_days`, 30 by default. Past 30 needs
  `ProFeature::LongHistory`, and a value out of range keeps 30 with a warning. `Host::start` loads
  the settings once, for the history and the webhooks.
- **Series** (`crates/daemon/src/series.rs`): `History::series(SeriesQuery)` groups the readings
  table's rows by account and window, leaves out unconfirmed readings, and names each account
  from its newest snapshot. `Series::activity` gives the rise at each reading, counting from 0
  after a reset (a fall, or a passed reset time). `write_csv` moved here from the CLI, so the CLI
  and the app write the same CSV.
- **API:** `GET /v1/history` with `since`, `provider`, `account` and `window` (axum's `query`
  feature). A bad `since` gets a 400 that names the form.
- **CLI:** `quotadial history [--since 7d|24h|90m|<RFC 3339>] [--provider] [--account] [--window]
  [--csv | --json]` reads the history directly. The text has a line per window: a 28-mark chart of
  the period (`▁` to `█`, a space for no reading), the latest reading and its age, and the highest.
- **The app:** `history.rs` builds the view: shown accounts, account names, charts thinned to 240
  points (the latest always kept), and a heatmap of the shortest window's use by local day and
  hour, generic over the time zone so tests run in UTC. `export` writes the CSV to Downloads. The
  window's `history.tsx` has the 7- and 30-day buttons, an SVG line per window with a dashed line
  at 90%, the heatmap grid and Export CSV. The footer gains History beside Settings.
- **Tests:** keeping 90 days and at most 365; series grouped and named, narrowed by time,
  provider, account and window; unconfirmed readings left out; activity across a reset; CSV
  quoting; `GET /v1/history`; `--since` forms and the text with its chart; the app's view (hidden
  providers, names, charts), the heatmap by day and hour, thinning and the export.
- **Live check on Matt's Mac**, on copies of the data folder:
  - `quotadial history --since 12h` drew each account's windows (the weekly limit at 99% as a solid
    bar, the sessions' hours of use); `--csv` gave 1,077 rows for the 12 hours.
  - The app, built under another name: the History view had the period buttons, each account's
    charts with the 90% line, and the heatmap (Friday's use at 13:00, 15:00 and 16:00 local). With
    history only from that morning, the lines first hugged the charts' right edge, so a young
    history is now charted from its first reading, and the view says "Since Fri 23 at 09:16, when
    the history begins."

### M4 PR 2b: alerts to webhooks and Slack (D-094)

- **Settings:** `Config` moved from `quotadial-providers` to the daemon crate (`config.rs`), with
  `webhooks` and `warnings`. `Webhook::new` accepts `https`, or `http` on this machine, and keeps
  the address as a `Secret`. `host()` is all that is shown, and `key()` is the first 8 bytes of
  its SHA-256 in hex. A file with webhooks that others can read gets a `chmod 600` warning.
- **Sending** (`deliver.rs`): one `reqwest` client, 10-second timeout, no redirects. `payload`
  builds the JSON (eight fields) or Slack's `{"text": "*title*\nbody"}` with `&`, `<` and `>`
  escaped. Errors are worded from the failure's kind and the host, never from `reqwest`'s message,
  which holds the address.
- **Retries** (`deliveries.rs`, schema 4): a `deliveries` row per firing and webhook key (host,
  sent, error, attempts, last try). `undelivered` gives the told firings of the last hour not yet
  taken and past their wait (1, 2, 4, 8, then 15 minutes). `deliver_due` sends them and records
  each result at the time it was given.
- **Poller:** `with_webhooks`. After each check it spawns one sending task, unless one is still
  running (a `tokio::sync::Mutex` guard), so polling never waits on a webhook. `Host::start` loads
  the webhooks from `config.json`.
- **Where it shows:** `quotadial daemon` names the hosts at start and prints the warnings.
  `quotadial alerts` lists the hosts, the warnings and each firing's deliveries.
  `firings[].deliveries` is in the API and MCP. The app's Settings says where alerts also go and
  shows the warnings.
- **Tests:** settings (defaults, https or local, never shown, the permission warning); the payloads
  (only the firing; Slack's escaping); sending to a local receiver once, a 404 tried again after a
  minute and not after an hour, an unreachable host named by its host; the backoff; the poller
  sending a told alert; the CLI's listing.
- **Live check on Matt's Mac**, with a daemon from this branch on a copy of the data folder and a
  local receiver at `http://127.0.0.1:48123/hooks/secret-path`:
  - The daemon said "It sends alerts to 127.0.0.1 too", naming the host only.
  - `warn me when a weekly window passes 40%` posted five alerts, each with exactly the eight
    fields, no account or organization id, and no sign-in place. The starting rules stayed quiet.
  - With the receiver stopped, `tell me when codex passes 45%` was recorded as "not sent to
    127.0.0.1: it could not be reached". With the receiver back, it was sent on the retry, a minute
    later.

### M4 PR 2: alert rules (D-093)

- **Core** (`crates/core/src/alert.rs`, `tier.rs`):
  - `Rule::parse` reads plain words with a hand-written scanner, no new dependency. A rule names
    at most one share, time, window and provider.
  - `Rule::occurrences(snapshot, last_reset, now)` says where a condition holds, with the instance
    that makes it once. `Rule::message` words the notification, and `Display` says what a rule was
    read as.
  - `tier()` and `ProFeature::AlertRules` gate running out and a time before a reset.
  - `text::provider_name` moved here from the app. `text::pace` takes the reset time, and
    `text::percent` writes a share without a trailing `.0`.
- **History, schema 3** (`crates/daemon/src/alerts.rs`):
  - `rules` (written, rule JSON, on, quiet) and `firings` (rule, account, label, scope model,
    instance, share, reset, title, body, quiet, time), pruned with the rest after 30 days.
  - `check(report, now)` records each occurrence not recorded before (two instances within 10
    minutes are one) and returns the firings to tell.
  - A quiet rule's first check records without telling. A rule that is off at the upgrade ends
    its quiet start without recording.
  - `last_reset` reads the window's reading before the current one.
  - `History::open` now takes the write lock (an immediate transaction) before reading the
    version.
- **Poller:** checks after every round with polls and every minute (`ALERTS_EVERY`), and tells
  consumers through `Latest::subscribe_firings`.
- **API:** `GET` and `POST /v1/alerts` and `PATCH` and `DELETE /v1/alerts/{id}`. JSON errors name
  the fix, and changes need the token like reads. `serve` takes the history.
- **CLI:** `quotadial alerts [list [--json] | add <words> | remove | on | off]`, on the history
  directly. `add` takes the words with or without quotes and says what it read.
- **MCP:** `list_alerts` returns the rules and the last week's firings. `serve` takes a second
  source for them.
- **The app:**
  - `notify.rs` (M3's `Sent`) is gone. `alerts.rs` tells the firings newer than `alerts.json`'s
    last told, within 15 minutes, with account names and without hidden providers.
  - The feed sends `Update::Fired` from the poller, or after each refresh while following.
  - Settings lists the rules (on or off, remove, read as) with a field to add one.
  - `settings::carry_over` turns off the starting rule for an M3 switch that was off, once, and
    `notified.json` is removed.
- **Tests:** the grammar (the examples, a dozen wordings, the refusals and limits, JSON, what a
  rule reads as), each condition, the wording, `tier()`; the store (once per window, a drifting
  reset, clock jumps, a 12-hour sleep across a reset, a restart before telling, the starting
  rules new and quiet, a rule off at the upgrade, add, off, on and remove, pruning, concurrent
  upgrades); the poller telling consumers; the API's endpoints; the CLI binary; `list_alerts`;
  the app's telling, carry-over and feed.
- **Live check on Matt's Mac**, with a daemon from this branch on a copy of the data folder (schema
  2, from the app updated to PR #43). The floor state is shared, so no provider was asked early.
  - `quotadial alerts add tell me when a window resets` upgraded the copy to schema 3 and added
    rule 3.
  - The first round recorded what M3 had already told, quietly: the two weekly windows at 99% and
    100%, and the two expired sign-ins. `quotadial alerts` showed nothing told.
  - Account …ddf5's session reset at 22:29:59 while the daemon ran. It told "Claude Session has
    reset" once, at 22:33:03, from its first reading after the reset, and the checks every
    minute after added nothing.
  - The app, built under another name on the copy, listed the three rules in Settings with their
    boxes, Remove buttons, "Read as" lines and the field to add one.
  - The plan's stop-across-a-reset check: the daemon stopped at 23:25:16, after reading account
    …041b's session at 33% with a reset at 23:29:59. The session reset with no daemon running. On
    restarting at 23:33:16, it told "Claude Session has reset" once, a second later, keyed by
    23:29:59, and three minutes of checks added nothing (posted on #44).

### M4 PR 1: pace (P-016, D-092)

- **Model:** `Window.pace: Option<Pace>`, with `Pace { pct_per_hour, runs_out_at? }`, each left out
  of JSON when absent. Providers never set it; every `Window` built in code says `pace: None`.
- **The fit** (`crates/core/src/pace.rs`): `Pace::of(window, readings)` keeps the readings from
  before the window's own and since `Pace::since(window)`, its reset less its length. It adds the
  window's own reading, then drops everything up to the last fall of more than a point. Below 3
  readings or 30 minutes there is no pace, nor for a full window or an unconfirmed 0%. The slope is
  least squares in points per hour, never below 0, rounded to hundredths. `runs_out_at` counts
  from the window's own reading at the rounded rate, to the second, and is kept only before the
  reset and within 366 days. `text::pace` words it for the CLI and the app.
- **History, schema 2:** a `readings` table beside `snapshots`, `WITHOUT ROWID`, keyed by account,
  label, scope model and fetch time, with the kind, share used, reset, length, source and the
  unconfirmed flag.
  - `record` writes a snapshot's readings in the same transaction as the snapshot, with
    `INSERT OR IGNORE`, whether or not the snapshot itself is new.
  - Opening a schema-1 database creates the table and fills it from the stored snapshots, in one
    transaction with the version bump.
  - `readings(account, window, since)` returns the confirmed readings, oldest first.
    `with_pace(snapshot)` paces an `ok` snapshot's windows. `prune` drops readings older than 30
    days too.
- **Poller:** after each answer, one `spawn_blocking` records the snapshot and then paces it, and
  the paced snapshot is served. What history shows at start has no pace until its first poll, which
  comes at once.
- **CLI:** `report` says where a report came from (`Origin`). The text has a line under each window
  that has a pace. A report fetched directly ends with "Pace needs the daemon: ..." once, when it
  has windows. The JSON gains only the field.
- **MCP:** the instructions and the descriptions of `get_usage` and `get_account_headroom` explain
  pace.
- **The app:** `WindowView` has `pace` and `runs_out`. The popover shows the pace under each
  window's detail line, in the warning colour when the window runs out before it resets.
- **Tests:** 19 more in the workspace (345): the fit's rules (10), its wording, its JSON, the
  readings table (kept once, the filters, the upgrade from schema 1, pace for `ok` snapshots only,
  pruning), a window gaining its pace in the poller, and the CLI's line and note. The app has 25.
  Breaking each of 14 rules in turn fails a test: the window's start, the fall, its tolerance, the
  reset, the window's own reading, the half hour, the year, `ok` only, writing readings, the
  upgrade, unconfirmed readings, pruning, the poller's pace and the note.
- **Live check on Matt's Mac**, with a daemon from this branch on a copy of the data folder. The
  floor state lives in `~/Library/Caches/quotadial` whatever the data folder, so no provider was
  asked early.
  - Opening the copy, schema 1 with 376 snapshots from 13:16 to 21:49 UTC, upgraded it and filled
    837 readings.
  - `status` showed a pace for 19 windows of 7 accounts. Full windows and the two expired
    sign-ins showed none.
  - Recomputing every pace in Python from the copied readings matched all 19 rates and run-out
    times exactly.
  - Codex's weekly window began about 5 hours earlier and was 41% used. At 10.27%/h it runs out in
    under 6 hours, 6 days before it resets.
  - A session that began 50 minutes earlier had 3 readings over 10 minutes, so it had no pace yet.
  - A network outage from 21:11 to 21:37 UTC made every account stale at once. Stale numbers get
    no pace, and readings resumed with the network.
  - With no daemon, `status` printed no pace and the note once, and `--json` had no pace.
  - `/v1/headroom` and the MCP server's `get_usage` carry the pace.
  - The app, built under another name and identifier so it ran beside Matt's with its notifications
    off, followed the test daemon. Its popover showed the pace under each window, with the 99%
    weekly window (2 h 16 min to run out) and Codex's (5 h 40 min) in the warning colour.

### M3: the review's follow-ups, #39 (D-089)

- **Naming once per token:**
  - `AccountState` has `named` and `named_generation`. `fetch` records the profile's answer there
    before `request_as`, whatever the request's answer.
  - `known_account` reads `named` first, then the older `last_good` record.
  - `request_as` no longer records anything itself.
- **Relative extra folders:** `candidate_dirs` returns `Candidates { dirs, skipped }`.
  `locate_all`'s `Found` carries the skipped entries, and the provider warns about each one
  (`advice::skipped_dir`).
- **Spellings:** `Search::candidate` tries `spellings(dir)`: the folder with and without a trailing
  separator, and its resolved path the same two ways. It takes the first whose item is listed, so
  `Folder::Named` holds the text Claude Code was given.
- **The cap:** `ClaudeProvider::keep_subscriptions` counts each sign-in as its named account when the
  state names it, and keeps further sign-ins of an account it already keeps.
- **Items with no time:** `NotAccounts::excludes` needs a generation, and `not_an_account` records
  none without one.
- **Tests:**
  - two items with no folder for one subscription;
  - naming once after a 503;
  - the cap counting subscriptions;
  - a relative extra folder;
  - an item with no time;
  - a trailing separator;
  - a linked home;
  - the state round trip with the new fields.

  326 tests in the workspace. Reverting the naming and the cap fails six of them.

### M3: the review of PR #36 (D-088)

- The manager's review of #36 arrived three minutes after the merge, with two items to fix before
  merging. They follow in PR #38, which the manager said to merge on green CI.
- **A failed listing.** `login::locate_all` returns a `Found`: the sign-ins, and why the Keychain
  could not be listed, if it could not. Without the listing, each candidate folder is searched with
  `locate`, which looks up the folder's item by name and then its credentials file. After one
  failed lookup, the Keychain is not asked again in that discovery. Only items whose folder is
  unknown are missed.
- **The warning.** `Provider::discovery_warnings` (default: none) carries the reason. `collect` and
  the daemon add it to the report's `errors` beside the accounts found. It clears at the next
  discovery that lists the Keychain.
- **The timeout.** `dump-keychain` has its own 20-second `LIST_TIMEOUT`.
- **Tests:**
  - a listing that fails still finds this shell's own sign-in and a folder's, warns, and clears the
    warning when the listing works again;
  - a keychain that stops answering is asked once more, and then the folders' credentials files are
    read;
  - `collect` and the daemon show warnings beside the accounts, with any token blanked out;
  - the listing's timeout message.
- 318 tests in the workspace.
- Opened afterwards: #39, the follow-up issue with the review's six smaller items.
- **M3's first-run test, 2026-10-23.** The app built from `main` (0c25cdd) was launched with an
  empty data folder (`open --env QUOTADIAL_DATA_DIR=…`). Its tray showed a number after 1.0 second.
  The provider cache was kept, so no account was asked twice inside its floor. From a cold cache
  the first number would wait for one usage request, about a second more.

### M3: polling at the floor (D-087)

- Found during PR 5's live check. At the 5-minute mark every account showed "from cache": the
  daemon's next poll for an account came a few seconds before the shared gate allowed a request.
- **The cause:** `Poller::poll_due` read the time once, at the start of a round, and each account's
  next poll was due a floor after that. But the gate counts the floor from when the request went
  out, after the accounts asked earlier in the round and after finding the sign-in, which takes
  longer the first time. So the next poll arrived early, was answered from the cache, and the real
  request came a round later: every 10 minutes instead of 5.
- **The fix:** `Poller::poll` takes the time when the account's answer comes back and counts the
  next poll from it. The gate's request time is never later than that, so the next poll always
  finds the floor passed. The interval grows by the length of a fetch, a second or two.
- **The test:** `each_poll_comes_once_the_floor_since_the_request_has_passed` drives a provider that
  keeps its own floor and finds its sign-in in 2 seconds the first time and 1 second after. Every
  poll now sends a request: at 0, 5, 10 and 15 minutes. With the old timing it gave 0, 10 and 15,
  checked by reverting the fix.
- **Live check on Matt's Mac, 2026-10-23.** The app was quit and the fixed `quotadial daemon` ran on
  its own from 18:20:50Z. The real fetch times in `history.db`:

  | Account | Old poller (the app, before) | Fixed daemon |
  | --- | --- | --- |
  | Codex, 2-minute floor | every 4 minutes: 17:52, 17:56, 18:00, 18:04, 18:08 | every 2 minutes: 18:20:51, 18:22:51, 18:24:52, 18:26:52, 18:28:53, 18:30:53 |
  | Each of 6 Claude accounts, 5-minute floor | every 10 minutes: 17:44, 17:54, 18:04 | 18:25:50 to 18:25:52, then 18:30:51 to 18:30:53 |

  At 18:30:50 the old timing would have replayed the cache, since the 18:25:5x requests were not
  yet 5 minutes old. The two accounts waiting for a sign-in send no request, as before. The app was
  started again afterwards.

### M3 PR 5: every stored Claude sign-in (#34)

- The model: `Account.sign_in` (P-015), and `Account::same_as`, the one identity check (D-078).
  `StatusReport::normalized` merges two sign-ins of one subscription and sorts each provider's
  accounts by name (D-082); `collect` and the daemon's `Latest::report` both use it.
- The Claude provider (D-077 to D-080):
  - `keychain.rs`: `Keychain::list` runs `security dump-keychain` without `-d` and keeps the
    `genp` items named `Claude Code-credentials` or `Claude Code-credentials-…`, attributes only.
  - `login.rs`: `locate_all` finds the shell's own sign-in, the default one when a token or a
    folder is set, every candidate folder's sign-in (`candidate_dirs`: `~/.claude…`,
    `~/.codus/accounts/*`, `extra_config_dirs`), and the items no folder matched (`Folder::Unknown`),
    then keeps one per account. `Login::place` is the `sign_in` text, and `sign_in_command` the
    command each fix names.
  - `advice.rs`: every sign-in fix takes the `Login` and names its command.
  - `lib.rs`: `fetch` finds the account's sign-in again each time (`login_for`); a sign-in that
    expires or is rejected keeps the last good windows as cache (`needs_sign_in`). `NotAccounts`
    (`not-accounts.json`, 0600) leaves out items with no folder and no Claude sign-in.
    `with_extra_config_dirs` and `with_max_accounts` take the settings.
- The Codex provider sets `sign_in` to its `auth.json` path.
- `quotadial-providers` reads `config.json` from the data folder (D-081).
- The app (D-083):
  - `settings.rs` keeps `aliases` by `account_key` (provider, account id, organization), which
    `notify.rs` also uses.
  - `meter.rs`: `with_aliases` puts the names in and sorts each provider's accounts by the name
    shown. The tray's number leaves out accounts waiting for a sign-in. "How QuotaDial reads this
    sign-in" names the place from `sign_in`.
  - The settings panel has a Names field per account, saved when the field loses focus or the
    popover does. The popover scrolls past 640 points.
- Live check on Matt's Mac, 2026-10-23:
  - **The Keychain:** 18 Claude Code items. One is the default, 14 matched folders in
    `~/.codus/accounts` by their hash, and 3 matched no folder and held only MCP server sign-ins.
    Those 3 were read once and are left out since.
  - **`quotadial status`** (the acceptance test): 9 rows in about 0.9 seconds, inside every floor.
    There are 8 Claude subscriptions: 6 with numbers, and 2 `auth_required` whose fixes name their
    folders ("Run `CLAUDE_CONFIG_DIR=~/.codus/accounts/.pending-… claude` once to refresh it."). The
    ninth row is Codex.
  - **The app:** the popover listed all 9 accounts with their windows and scrolled to the last. The
    tray showed 100%, the fullest window among the working accounts. A name given in Settings showed
    in the popover, the menu and the tooltip, moved that account into place by name, and was
    cleared again. The order stayed the same from launch through polling and across a restart.
  - **Fixed during the check:**
    - The order moved between launches, because a restarted daemon used history's order (D-082).
    - The popover could not scroll.
    - A name typed before clicking away was saved only when the field next lost focus.
  - **Found, for the next PR:** the daemon schedules an account's next poll from when its round of
    polls began, a few seconds before the shared floor allows the request. So every other poll is a
    replay from the cache, and a moving account is really asked every 10 minutes, not every 5.
- An independent review of the branch found these, all fixed before the PR:
  - **The floor was per sign-in, not per subscription** (D-084). An environment token, or an item
    with no folder, kept its own floor even when it was the same subscription as another sign-in, so
    one subscription could be asked twice in 5 minutes. Such a token is now named, from the cache or
    by the profile, before any usage request, and asks under its account's lock and floor.
  - **Headroom ranked expired accounts first** (D-086), on their old cached numbers.
  - **One failed lookup hid every account** (D-085):
    - A Keychain listing that failed or timed out read as empty, so every folder's account looked
      signed out and notified.
    - Every fetch listed the whole Keychain again.
    - A failed discovery dropped the daemon's accounts.
    Now a failed listing is an error, a fetch looks up only its own sign-in, and the daemon keeps
    the accounts it found last.
  - **The recheck after reading a token** accepted any sign-in of the account. It now requires the
    same sign-in with the same generation (D-079).
  - **Fixes that named no folder:** "signed out" and "now another account" name the place and its
    command. Windows gets a PowerShell command, and paths that need quotes are quoted (D-079).
  - **`sign_in` was lost** once the profile named an account, and went stale when an account's
    sign-in moved. It now comes from the sign-in each fetch used, and the daemon refreshes what it
    knows about an account at each discovery.
  - **Smaller ones:**
    - Only `Claude Code-credentials-<8 hex>` items are listed.
    - `not-accounts.json` is rewritten under a lock.
    - With a token in the environment, the shell's own folder is searched.
    - Links to folders count.
    - Keychain and file times compare as times.
    - The first sign-in counts as the shell's own only when it is.
    - One wrong setting in `config.json` no longer resets the others.
    - Tests build folder paths the way discovery does, so they pass on Windows.
- Tests: 314 in the workspace. The new ones cover:
  - listing items by attributes only, and a listing that fails;
  - every stored sign-in as an account named by its place, and a folder's own token and fix;
  - the environment token as one more account, and one subscription through two sign-ins asked once
    per floor;
  - a token read during a rewrite, and a folder switched to another account mid-read;
  - MCP-only items left out, including records made at the same time;
  - the fix commands per shell, links, and newer sign-ins;
  - `config.json`, and merging and sorting the report;
  - headroom without expired accounts, and the daemon keeping accounts when discovery fails.

  The app has 24, with new ones for names, their order, and the tray's number without expired
  accounts.

### M3 PR 4: builds

- `.github/workflows/release.yml` (D-076) replaces the stub.
  - `cli` builds the CLI for Linux x64, macOS Apple silicon and Intel, and Windows x64, as
    `quotadial-<version>-<target>.tar.gz` (`.zip` on Windows).
  - `app` builds the tray app as a universal dmg on `macos-latest`, an msi on `windows-latest`, and
    a deb and an AppImage on `ubuntu-22.04`.
  - `publish` drafts a pre-release with every file, and runs only for a `v*` tag.
  - A manual run uses `dev-<sha>` as the version and keeps the files as workflow artifacts.
- `prepare-cli.mjs` builds a universal Mac CLI. Tauri wants three files: the arm64 and x86_64 CLIs
  for its per-architecture compiles, and one joined by `lipo` for the bundle.
- `tauri.<system>.conf.json` sets each system's installers: app and dmg on macOS, msi on Windows,
  deb and AppImage on Linux. The deb has a long description.
- `docs/INSTALL.md` has a Download section with each system's file, and says where the bundled
  CLI goes on Windows and Linux.
- Live check, 2026-10-23 16:40Z:
  - **Local universal build:** a 16.3 MB dmg; both binaries are x86_64 plus arm64, the Intel side
    runs under Rosetta, and `codesign --verify --deep --strict` passes.
  - **The release workflow on the branch** (run 39986123385, started by hand): every job passed and
    the publish step was skipped.
  - **What it produced:**

    | File | Size | Checked |
    | --- | --- | --- |
    | `QuotaDial_0.1.0_universal.dmg` | 17.1 MB | Mounted read-only: the app and an Applications link; both binaries universal; ad hoc signature valid; the CLI runs |
    | `QuotaDial_0.1.0_x64_en-US.msi` | 8.7 MB | Its cabinet holds the app and `quotadial.exe` |
    | `QuotaDial_0.1.0_amd64.deb` | 9.4 MB | Installs `/usr/bin/quotadial-tray` and `/usr/bin/quotadial`, plus icons and a `.desktop` entry, and depends on WebKitGTK 4.1 and Ayatana AppIndicator |
    | `QuotaDial_0.1.0_amd64.AppImage` | 87 MB | WebKitGTK inside |
    | Four CLI archives, 8 to 9 MB each | | Each holds `quotadial`, `qd`, the README, the licence and the notice |

  - **Fixed after that run:** the publish step now uploads the files rather than their folders.
- Not done: tagging a release. A `v0.1.0` tag would draft a public pre-release, so it waits for a
  go-ahead.
- The deb's package name is `quota-dial`, because Tauri makes it from the product name. It works,
  but a later PR may want `quotadial`.

### M3 PR 3: settings and notifications

- `settings.rs` (D-074):
  - It keeps `Settings` (`notify_sign_in`, `notify_high`, `hidden_providers`) in the app's config
    folder as `settings.json`, 0600, through a temporary file and a rename.
  - A damaged or partial file falls back field by field.
  - `Settings::shown` drops hidden providers from what the tray and the popover show. The daemon
    still polls them for the CLI and the MCP server.
- `notify.rs` (D-075):
  - `Sent::check` returns the notices a report calls for: sign-in needed once until the account
    works again, and passed 90% once per reset, keyed by the reset time.
  - It never notices an unconfirmed reading, and it skips hidden providers and notifications that
    are switched off.
  - It keeps accounts missing from a report.
  - `notified.json` (0600) stops a restart from repeating a notice.
- The app (`lib.rs`):
  - `tauri-plugin-notification` shows the notices, and `tauri-plugin-autostart` turns opening at
    login on and off (a LaunchAgent on macOS). Launch at login is read from the system, not stored.
  - The commands `settings` and `set_settings` back the popover's Settings panel.
  - Switching a notification on rechecks the latest report at once.
- The popover: a Settings button opens a panel with three parts: open at login, the two
  notifications, and a checkbox per shipped provider. Each change applies at once and errors show
  in the panel. The JavaScript is 18.8 KB.
- Live check, 2026-10-23 16:16Z, release build on Matt's Mac. I drove the panel through macOS
  accessibility.
  - **Panel:** it showed opening at login off, both notifications on, and Claude and Codex shown.
  - **Open at login:** turning it on wrote `~/Library/LaunchAgents/QuotaDial.plist`, which runs
    the app at load. Turning it off removed it.
  - **Hiding a provider:** hiding Codex changed the tray from 79% (Codex Weekly) to 10% (Claude
    Weekly) and saved `{"hidden_providers": ["codex"]}` in `settings.json` (0600, folder 0700).
    `quotadial status` still listed both accounts. Showing Codex again brought back 79%.
  - **Notifications:** none fired, because no limit reached 90% and no sign-in expired.
    `notified.json` was not written. The rules are covered by tests; Codex Weekly was at 79%.
- Test count: tray app 21 (feed 3, meter 6, icons 5, settings 3, notifications 4); the core
  workspace is unchanged at 292.

### M3 PR 2: the popover

- The window became a popover (D-072):
  - It has no frame, stays out of the taskbar and above other windows.
  - It opens beside the tray icon through `tauri-plugin-positioner`.
  - A click on the icon toggles it, and Escape hides it. Opened by a click, it also hides on
    blur, as a menu does. Opened by reopening the app or launching it again, it stays until
    dismissed. Logging showed that macOS takes the focus back 0.25 to 1.8 s after such an open,
    which would close it at once.
  - It fits its content through a `fit` command fed by a `ResizeObserver`.
  - On macOS it sits on the system popover material with rounded corners, via `windowEffects`
    and the macOS private API (D-073).
  - Reopening QuotaDial on macOS, or a second launch elsewhere, opens it.
- The view is built in Rust (`meter.rs`) with the shared wording:
  - the headline (what the tightest number is);
  - each account with its plan, its status and the fix when not ok;
  - each window, tightest first, with a `<meter>`, its reset, its source and age, and the
    unconfirmed badge;
  - extra usage, "How QuotaDial reads this sign-in", problems, and where the numbers come from;
  - a Quit button.
  - The feed now reports whether the app runs or follows the daemon (`Update::Role`).
- P-015 (`Account.sign_in`) is asked on #29. Until it is decided, the sign-in text lists the places
  QuotaDial checks.
- Live check, 2026-10-23 15:50Z, release build on Matt's Mac:
  - **Shape:** reopening the app showed the popover at 360 by 513 points, fitted to its content.
    It sat on the macOS popover material and was centred, since no tray event had placed it yet.
  - **Content:** "75%" in amber over "Codex Weekly, resets in 6 d 1 h". Claude's three windows
    came tightest first with blue bars, each with its reset and "from cache 3 min ago", because
    the floor was still holding from the previous run. Codex's Weekly showed an amber bar at 75%,
    with credits under extra usage. Both sign-in disclosures were there, and the footer read
    "Running the daemon, port 47115" beside Quit.
  - **Hiding:** a reopened popover stays (checked for 4 s after a fresh launch). Another app
    taking focus closes one opened by a click.
  - **Not seen here:** placement beside the tray icon and opening by a click of it. The menu bar
    auto-hides on this Mac, and macOS 14 and later do not let an app activate itself unless the
    user clicked it.
- Test count: tray app 14 (feed 3, meter 6, icons 5); the core workspace is unchanged at 292.

### M3 PR 1: the scaffold, the daemon in the app, the bundled CLI

- Shared code first:
  - `quotadial_api::Host` runs the daemon for `quotadial daemon` and the app alike (D-067).
  - `Poller::run` abandons a poll in progress when stopped.
  - `Latest::subscribe` says when the report changes.
  - The client hands over to a direct fetch when the daemon stops mid-answer and releases its lock
    within 2 seconds (D-068).
  - The shipped providers moved to `quotadial-providers`, and the wording to `quotadial_core::text`
    (D-071).
- `apps/tray`:
  - The window is Vite 8, Preact 11 and TypeScript 7 (D-070), plain CSS with the system font. It
    builds to 15 KB of JavaScript, and `scripts/check-size.mjs` holds it under 150 KB.
  - `src-tauri` is its own Cargo workspace (D-069):
    - `feed.rs` runs the daemon, follows a standalone one, takes over when it stops, and moves to a
      free port when 47115 is taken.
    - `meter.rs` holds the tightest share used, its level (70% and 90%) and the account lines.
    - `icon.rs` draws the dial and the Windows number.
    - `lib.rs` has the tray, its menu (accounts, Open, Quit), the window, one instance only, no Dock
      icon, and quitting only after the daemon stops, including when macOS quits the app without
      asking.
- The CLI is bundled with `scripts/prepare-cli.mjs` as an `externalBin`, so the Mac app carries it
  at `Contents/MacOS/quotadial`. The app's own binary is `quotadial-tray`, so the two do not collide
  on macOS's case-insensitive disk. The Mac app is signed ad hoc.
- CI has a `tray` job on all three systems: the window's types, build and size, the CLI, then fmt,
  clippy and tests. `docs/INSTALL.md` covers opening unsigned builds and putting the CLI on the
  `PATH`.
- Live check, 2026-10-23 15:25Z to 15:35Z, on Matt's Mac (Apple Silicon), with the release build:
  - **Startup:** the number was in the menu bar 1.0 s after launch: "73%", read through the
    accessibility API. The app held the lock and announced port 47115.
  - **Bundled CLI:** `QuotaDial.app/Contents/MacOS/quotadial status` answered from the app's
    daemon in 10 ms, with both accounts fresh from the APIs. A `qd` link to it works too.
  - **Window:** it showed 73% in amber, with both accounts. A second launch showed the first
    instance's window and exited.
  - **Clean quit:** quitting through macOS, while the CLI asked every 100 ms, gave 50 calls and no
    failures. The app answered 13 of them and handed the other 37 to direct fetches. `daemon.json`
    was gone and the history closed.
  - **Takeover:** with `quotadial daemon` started first, the app followed it, showing 74%. When that
    daemon stopped, the app took over in 4.0 s.
  - **Signature:** `codesign --verify --deep --strict` passes on the ad hoc signed bundle (16.5 MB;
    the dmg is 7.8 MB).
  - **Not done:** I did not click the tray menu's Quit, because the menu bar auto-hides on this Mac
    and I did not want to drive the mouse. That path runs the same stop as above, from Tauri's
    exit request.
- Test count: 292 in the core workspace (core 25, poll 36, daemon 19, API 30, providers 1, Claude
  101 plus the opt-in Keychain test, Codex 54, CLI unit 17, CLI binaries 9), and 12 in the tray app
  (feed 3, meter 4, icons 5). Four wording tests moved from the CLI to core with the code.

### M2 #25: the MCP server and SKILL.md

- `quotadial_api::mcp::serve` reads JSON-RPC lines and writes one reply per line. It is generic
  over its input and output, so the tests run it in memory. The CLI's `quotadial mcp` hands it
  standard input and output, the provider ids, and the same daemon-first `report` that `status`
  uses (D-062).
- Tools (D-063): `get_usage`, `get_account_headroom` and `list_alerts`, all read-only. Their
  answers are JSON text in `status --json`'s order, plus `structuredContent` for clients on
  revision 2025-06-18 or later. An optional `provider` narrows the first two. Wrong arguments and
  an unanswered daemon are tool errors that name the fix.
- `skills/quotadial/SKILL.md` (D-064): when to check, the three ways to ask, the HTTP API's shape,
  status codes and data folders, how to pass the token unseen, and when to warn the user.
- `docs/CLI.md` has a section on the MCP server, with the `claude mcp add` and `codex mcp add`
  commands (both checked against the installed CLIs' help). The README lists the daemon and the
  MCP server.
- Live check, 2026-10-23 13:52Z, on Matt's Mac, with the daemon running:
  - A scripted session agreed on 2025-11-25 and ranked Claude at 91% (Session) above Codex at 37%
    (Weekly), both fresh from the APIs. `get_usage` with `provider: codex` returned only the Codex
    account, and `list_alerts` returned `{"alerts": []}`.
  - The official MCP Inspector CLI (2.12.0) listed the three tools, called `get_account_headroom`
    with `provider=claude`, and got the tool error for `provider=gemini`.
  - Claude Code 2.1.286's health check (`claude mcp get`) reported the server "Connected". It was
    added with local scope for a throwaway folder and removed afterwards.
  - With the daemon stopped, the server fetched for itself, and both accounts replayed from the
    cache inside their floors.
  - The PRD's M2 test, "Claude Code can call the skill and read headroom": a one-off `claude -p`
    with Haiku, the skill in a throwaway project and `--mcp-config` for `quotadial mcp`. It found
    the skill, called `get_account_headroom`, and answered Claude 91% (Session, resets 15:40Z) and
    Codex 36% (Weekly, resets Oct 29).
  - The first run tried a made-up shell command before the tool. The skill now says the MCP tools
    are tools, not commands, and that the CLI's only command for this is `status --json`. The
    second run went straight to the tool.
- Test count: 284 (core 21, poll 36, daemon 17, API 26, Claude 101 plus the opt-in Keychain test,
  Codex 54, CLI unit 20, CLI binaries 9).

### M2 #24: the local API and its token

- `crates/api` (`quotadial-api`): `Token` (32 random bytes from `getrandom`, hex, in `token`, 0600,
  made on the daemon's first start, compared in constant time), `Runtime` (`daemon.json`: pid,
  port, start time, 0600), the axum server and `client`, which the CLI and the MCP server use.
- The server (D-056, D-057): `bind` listens on `127.0.0.1` only; a middleware refuses a `Host` that
  is not `127.0.0.1:<port>` or `localhost:<port>` (403) and then a missing or wrong bearer token
  (401 with `WWW-Authenticate: Bearer`). `GET /v1/status` serves the poller's `Latest` report;
  `GET /v1/headroom` ranks it (D-060). Other paths get a JSON 404, other methods a JSON 405.
- Finding the daemon (D-058): `Instance::is_running` checks `daemon.lock` with a momentary shared
  lock, and `Instance::acquire` waits up to a second for such a check. The token is sent only while
  the lock is held; a starting daemon deletes a crashed one's `daemon.json` before it listens.
- The CLI (D-059): `quotadial daemon [--port N]` loads or makes the token, listens, writes
  `daemon.json`, prints the API's address and the token's location, then runs the poller and the
  server until Ctrl-C or `SIGTERM`, and removes `daemon.json`. `status` and `watch` ask
  `/v1/status` while a daemon runs and call `collect` only when none does. A running daemon that
  does not answer is an error that names the fix, never a reason to poll.
- `Snapshot::windows_tightest_first` moved to core so the CLI and headroom agree (D-061).
- Live check, 2026-10-23 13:37Z, on Matt's Mac: the daemon listened on `127.0.0.1:47115` with
  `token` and `daemon.json` at 0600. Without the token the API answered 401, with another `Host`
  403. With it, `/v1/status` returned both accounts fresh from the APIs (Claude Max, Session 8%;
  Codex Pro, Weekly 62%, 1056.44 credits) and `/v1/headroom` ranked Claude 92% (Session) above
  Codex 38% (Weekly). `quotadial status` answered from the daemon in 7 ms. The curl command in
  `docs/CLI.md` worked in zsh and bash. Ctrl-C and `SIGTERM` both stopped the daemon cleanly and
  removed `daemon.json`; `status` then replayed both accounts from the cache inside their floors.
- Test count: 271 (core 21, poll 36, daemon 17, API 14, Claude 101 plus the opt-in Keychain test,
  Codex 54, CLI unit 20, CLI binaries 8).

### M2 #23: the daemon and its history

- `crates/daemon` (`quotadial-daemon`): `DataDir` (the OS data directory or `QUOTADIAL_DATA_DIR`,
  0700), `Instance` (a lock on `daemon.lock`, so one daemon runs), `History` (SQLite in WAL mode,
  0600, one row per snapshot with new data, pruned after 30 days, `user_version` 1) and `Poller`.
- The poller rediscovers accounts every minute (discovery runs off the async runtime, since it may
  run `/usr/bin/security`). It asks a moving account at its provider's floor and a still one every
  10 minutes, uses the provider's fallback when a fetch fails, and keeps a `Latest` report for
  consumers. A restarted daemon shows its stored numbers, marked as cache, until its first poll.
- `StatusReport`, `ProviderError` and `collect` moved to core (D-055).
- `quotadial daemon` prints how often it asks each provider and where its history is, and stops
  on Ctrl-C.
- Live check, 2026-10-23 13:16Z: the daemon polled both accounts at once (Claude Session 7%, Codex
  Weekly 59%), stored one row each in `~/Library/Application Support/quotadial/history.db` (WAL,
  folder 0700, files 0600), and stopped cleanly on Ctrl-C, checkpointing the WAL.
- Test count: 252 (core 20, poll 36, daemon 16, Claude 101 plus the opt-in Keychain test, Codex
  54, CLI unit 18, CLI binaries 7).

### M1 #13 and P-012: unknown buckets and the credit balance

- The manager approved P-014 and P-012 on 2026-10-23 (D-052). `WindowKind::Other`: a Claude
  bucket with a `utilization` but no known key or prefix is a window of kind `other`, labelled from
  its key, with `window_seconds: 0`. `ExtraUsage.balance`: the Codex credit balance, parsed from
  ChatGPT's decimal string; the CLI says "on, 1056.44 credits left".
- `docs/CLI.md` lists the new kind and field.
- Test count: 235 (core 20, poll 36, Claude 101 plus the opt-in Keychain test, Codex 54, CLI unit
  18, CLI binaries 6).

### M1 #20: the unconfirmed-0% rule (P-005)

- The PRD accepted P-005 as an M1 follow-up: "the model gains `unconfirmed: bool` on `Window`", and
  "a 0% that follows a non-zero reading is held as unconfirmed until a second fetch agrees". #20
  was opened for it, since no issue existed.
- `Window.unconfirmed` (left out of JSON when false). `AccountState::record_usage` now takes the
  snapshot and returns it marked: a window reading 0% where the same window read more on the last
  good fetch, with that reading's reset still ahead, is unconfirmed. The next fetch confirms an
  agreeing 0%. A window whose reset passed or is unknown is never flagged (D-051).
- The CLI prints "unconfirmed until the next fetch agrees"; `docs/CLI.md` documents the field. The
  PRD's grey badge is the M3 UI's job.
- Test count: 234 (core 20, poll 36, Claude 101 plus the opt-in Keychain test, Codex 54, CLI unit
  17, CLI binaries 6).

### M1 #11, #12, #14 and #15: the review follow-ups

- #12 and #11 (D-048): no more `claude --version`; the User-Agent is the fixed
  `claude-code/2.1.286`. The Keychain lookups in `fetch` run in `spawn_blocking`.
  `discover_accounts` is synchronous by the `Provider` trait, so M2's daemon should call it from a
  blocking task.
- #14 (D-049): an unnamed Keychain or file sign-in was named by `/api/oauth/profile` on every usage
  request. It is now named once per token, through `AccountState.last_good_generation`. Tests: one
  account in two organizations keeps two sets of numbers; an unnamed sign-in is named once per
  token, and again after a new token; no cache file holds the access or refresh token, or their hex
  encoding, after a Keychain fetch and a file fetch. The poll crate's test that could not fail is
  gone.
- #15 (D-050): `Secret` is wiped on drop and has no `PartialEq`. The Claude and Codex parsers
  deserialize only the fields QuotaDial uses, so the refresh token is skipped without a copy; the
  access token, ID token, API key and hex-decoded payloads sit in `Zeroizing` buffers; the Keychain
  read trims the secret in place instead of copying it, and the process reader reserves room so
  the buffer is not reallocated. A Keychain read that panics reports "reading it stopped
  unexpectedly; try again", tested with a panicking fake keychain.
- #13 waits for the manager: unknown buckets need either a new `other` window kind (P-014) or a
  note in the error. The question is on the issue.
- Test count: 229 (core 19, poll 34, Claude 101 plus the opt-in Keychain test, Codex 53, CLI unit
  16, CLI binaries 6).

### M1 #3: CLI `status`, `--json` and `watch`

- Text (D-047): every window as "Session: 42% used, resets in 2 h 10 min, from the API 4 min ago",
  the tightest first (highest share used, then soonest reset), and "reset 3 min ago" when the
  numbers predate a reset. JSON is unchanged and keeps absolute UTC times.
- `watch` refreshes every minute and relies on each provider's floor (5 minutes for Anthropic, 2
  for ChatGPT), so most refreshes show cached numbers with a growing age. It clears the terminal
  only when stdout is one and not on Windows (whose console needs unsafe code to read escape codes), ends each refresh with when it was updated and how often each provider
  is asked, and stops with Ctrl-C or when its reader goes away. `watch --json` prints one compact
  report per line.
- Every error names the fix or when QuotaDial tries again: a failed request with nothing cached
  gives the next request time, an unreachable provider says to check the connection, a locked or
  unanswered Keychain says to unlock it or answer the prompt, a damaged sign-in says to run
  `claude` or `codex login`, unreadable files and cache folders say what to check.
- `docs/CLI.md` documents the commands, the text and JSON output, the polling limits and the exit
  codes, for agents and for the M2 SKILL.md.
- Live check, 2026-10-23 12:28Z: `quotadial status` showed both accounts with relative times, the
  tightest window first; one `watch` refresh replayed both from the cache and printed its footer.
- Test count: 227 (core 19, poll 34, Claude 99 plus the opt-in Keychain test, Codex 53, CLI unit
  16, CLI binaries 6, including `watch` and `watch --json` run for real).

### M1 #2: ChatGPT/Codex provider

Modules in `crates/providers/codex/src/`, each naming the CodexBar files it follows:

| Module | Job |
| --- | --- |
| `environment.rs` | `$CODEX_HOME` or `~/.codex`: `auth.json` and `sessions/` |
| `auth.rs` | Parses `auth.json`: the access token, the ChatGPT account id, and from the JWT claims (decoded, not verified) the user id, email, plan and expiry. The refresh token is never read |
| `api.rs` | `GET /backend-api/wham/usage` with `ChatGPT-Account-Id` and the Codex CLI's User-Agent (D-038); no redirects, no cookies, 30 s timeout, 1 MiB cap |
| `usage.rs` | Windows by length (D-041), per-model `additional_rate_limits`, credits as extra usage |
| `sessions.rs` | The fallback: the newest rate-limit event in the account's own session logs (D-042) |
| `advice.rs` | The one sentence shown when there are no fresh numbers, naming the fix |

- Shared gate (D-043): the Claude provider's floor, 429 backoff, replay, damaged-state hold and
  state files moved to `crates/poll` (`quotadial-poll`), parameterized by a `Policy`. Codex uses a
  2-minute floor and Claude 5 minutes. Claude's messages and cache files are unchanged, and its
  tests pass as before. `Secret` moved to `quotadial-core` and the `Clock` trait to the poll crate.
- Identity (D-040): the ChatGPT user in the ChatGPT account (workspace), so one person in two
  workspaces is two accounts. `redact_secrets` now also blanks JWTs (D-044).
- Live check on Matt's Mac, 2026-10-23 11:56Z: `quotadial status --json` exited 0 in 1.4 s with
  Claude (Session 3%, Weekly 7%, Weekly (Fable) 0%) and Codex (Pro, Weekly 54%, credits on), each
  from the API. A second run replayed both from the cache in 0.06 s. No token appeared in the
  output, and the cache holds `quotadial/claude` and `quotadial/codex`, both private. Re-checked
  after the review fixes (12:18Z, HTTPS only): 1.2 s from the API, then a cached replay in 0.06 s.
- Real-data fallback check, 2026-10-23 12:18Z: with ChatGPT made unreachable (a dead proxy) and a
  fresh cache, `quotadial status --json` answered in 0.13 s from the session log: Weekly 54%,
  `local_log`, written 4 minutes earlier. The newest log is a 445 MB file in a folder dated ten days
  earlier, which is why files are chosen by when they were written.
- Not built in M1 (D-045): `chatgpt_base_url` from `config.toml`, environment tokens, and the
  response fields whose shape has not been seen with values (`code_review_rate_limit`,
  `model_usage`, `spend_control`, the reset credits). The credit balance waits for P-012.
- `crates/core/tests/fixtures.rs` fails if any committed fixture (`fixtures/**/*.json*`, not `raw/`)
  holds `eyJ`, `sk-`, `bearer` or an email address outside `example.com` (#4).
- Security review (an independent agent, read-only, 2026-10-23). It found no token leakage, no
  credential writes and no hole in the Codex floor or backoff, and confirmed the Claude refactor
  changes no behaviour or cache file name. It found six smaller problems, all fixed with tests:
  1. The fallback trusted who created a session, but a session resumed after `codex login` to
     another account keeps writing to the same file. Events must now postdate the current sign-in,
     and events naming another plan are skipped (D-042).
  2. A line longer than a chunk was copied again for every chunk, so a 31 MiB pasted line cost
     4 seconds. Lines over 256 KiB are now skipped without being kept. Files are still chosen by
     when they were written, not by folder date, because a session started weeks ago can be
     written to today.
  3. An absurd `reset_after_seconds` could overflow and panic. It is now dropped.
  4. JWT redaction matched inside words and file paths, and missed padded tokens, opaque tokens
     and OpenAI keys; plan and limit names went unredacted (D-044).
  5. An answer naming another user or workspace was not checked. It is now refused (D-040).
  6. Two tests could not fail. Both now fail when the code they guard is broken (checked by
     breaking it), and new tests cover redirects (both providers), two workspaces, a body over
     1 MiB and chunk boundaries at every offset.
  Both providers now speak HTTPS only outside tests (D-046).
- Test count: 220 (core 19, poll 34, Claude 99 plus the opt-in Keychain test, Codex 53, CLI unit
  11, CLI binaries 4).

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
