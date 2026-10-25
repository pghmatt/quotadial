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
- 2, the Free and Pro gates (D-100), merged (PR #56);
- 3, help and feedback, with `quotadial doctor` and `CHANGELOG.md` (D-101), merged (PR #57);
- 4, the key page's function, decided on #54 (D-102), merged (PR #58): it signs with a second,
  online issuing key (`ISSUER`, beside the root `QUOTADIAL`, which stays on Matt's Mac), reached
  from the receipt's `#licence=` link; the Buy button and the page itself come with the site;
- 5, in two pull requests: the website in `site/`, built but not deployed (D-113), merged
  (PR #68); and the shareable image (decision f on #40), a feature of the app's History
  (D-115), merged (PR #70);
- 6, signing and updates, each turning on when its secrets exist (D-103 to D-106), merged (PR #59);
- 7 to 12, hardening: resilience (D-107, D-108), merged (PR #61); security (D-109), merged
  (PR #63); accessibility (D-110), merged (PR #65); the OpenAPI description (D-111), merged
  (PR #66); package managers, completions and manual pages (D-112), merged (PR #67); and docs, in
  three PRs: the fake provider and the example build (D-117), merged (PR #71); the release copies,
  the site's downloads and guides and the README's pictures (D-116), merged (PR #72); the install
  check and #52's figures from Matt's copy, merged (PR #74); and the first-run recording with the
  example build's figures (PR D), waiting on Matt's go-ahead for the example in his menu bar (#51);
- then M6. The fake provider #52 asks for is done: the manager decided on #49 (14:25 UTC)
  to build it with item 12 (D-117). The M6 plan is #73, approved by the manager at 20:29 UTC with
  its decisions (Gemini CLI detected and explained, then Antigravity through its status line, then
  Copilot; Cursor out of M6); its first PR is being built on `feat/m6-groundwork`.
The manager's 03:30 UTC check on 2026-10-25 set the order: 5, then #64's fix, 10, 11, 12, then M6.
Items 10 and 11 merged while 5 was being built, ahead of that order (the check landed a minute after
the builder last read #49; said so on #49 at 04:45 UTC). Item 5 merged (PRs #68 and #70) and
#64's fix (PR #69); from here: 12 (#71, #72 and #74 merged; PR D, the recording, waits on Matt),
then M6 (#73).
The manager's review of #68, posted after it merged (point 1 repeated on #70 and #49 as blocking
the 0.3.0 bump), is answered on `docs/review-fixes`, from `feat/docs`: every point, wrangler with
an undici override (notes below).

M6, the next providers: the manager approved the plan on #73 on 2026-10-25 with decisions (a) to
(d): Gemini CLI detected and explained, never read (D-118); Antigravity through its CLI's status
line only (D-119); no Cursor provider (D-120); and the order 1, 3, 2, each provider behind its own
feature (D-121). PR 1, the groundwork, is built on `feat/m6-groundwork` (D-122): P-018's hook for a
provider's own commands, Google's and GitHub's tokens redacted by their shape, and the Gemini
provider behind a `gemini` feature. Gemini is held off by default, since the app cannot show its
account correctly without a change for one provider (its name, and its sign-in panel's sentence);
that is the manager's to decide. Next: PR 3, Antigravity, then PR 2, Copilot, its merge held on its
live check (#51).

Proposals:
- P-001 to P-009 are accepted and in the PRD.
- P-010 is decided (D-034).
- P-011 and #7 are deferred (D-035).
- P-012 and P-014 are in the code.
- P-013 is the manager's PRD edit.
- P-015 (`Account.sign_in`) is accepted (#29, #34) and in the code (D-079).
- P-016 (`Window.pace`) is accepted (#40) and in the code (D-092).
- P-017 (`Window.source` `example`) is asked for by the manager's rule 1 on #49 and in the code
  (D-117); the PRD edit is the manager's.
- P-018 (a provider's own commands) is accepted in principle on #73 and in the code (D-122).
- P-019 (the deferred-provider rows) is asked for by the manager on #73; the PRD edit is the
  manager's.

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
| 2: Free and Pro | Done (PR #56) | `Licensing`, the one switch, read from the key at every ask; Free shows one account per provider, keeps 30 days, checks threshold, reset and sign-in rules, sends no webhooks and reads no logs; headroom and the alert rules through the API and MCP are Pro's; `left_out` and a line where each feature would be; a key counts without a restart (D-100) |
| 3: help and feedback | Done (PR #57) | `quotadial doctor` and Send feedback from one text built in core, with emails cut to their first letters and nothing secret; Help in the popover and the tray's menu with What's new from the built-in `CHANGELOG.md`, support and About; release notes from the changelog (D-101) |
| 4: the key page's function | Done (PR #58) | `POST /api/key` in `site/`: validates the receipt's key with Lemon Squeezy (valid, `inactive` or `active`, QuotaDial's store and product) and signs `ls-<id>` with the issuing key, deterministically; the issuing key `ISSUER` beside the root; `KeyError::OrderKey`; the ended-licence sentence names the renewal; a `site` CI job (D-102) |
| Free at once (#56's review) | Done (PR #60) | A lapsed key now ends the week's cost and Pro-length history at once: scans keep their result only within the costs generation they began in, and a tier change prunes now; `left_out` asks `Tier::allows` per feature; `alerts --json` marks `waits`; the nudge's wake-up is tested through `run()` |
| 7: resilience | Done (PR #61) | A damaged history is moved aside under a dated name and a new one started with the alert rules it still gives, checked page by page after an unclean stop; a full disk is said in the report until a save works, and webhook sends are not repeated; a wake from sleep or a clock jump ahead asks everything again at once; an unreachable provider is asked again within a minute; `problems` in the report (D-107, D-108) |
| 8: security | Done (PR #63) | `cargo deny` and `cargo audit` on both workspaces in CI, weekly too, with each exception's reason; `SECURITY.md` and the threat model in `docs/SECURITY.md`; property tests over every parser of outside text, which found a token-count overflow, now saturating; an SBOM for the command and the app on each release (D-109) |
| 9: accessibility | Done (PR #65) | Every control reached by Tab in the order shown, with its ring, pressed by the keyboard, and named with its role and state; panels take the focus and Done gives it back; the heatmap read as a line a day; WCAG AA contrast in both themes, now including the chosen period and text fields' edges; Command or Control and = zooms, the window growing with it; nothing cut off at 200% or by a long name; nothing moves with less motion; `scripts/check-window.mjs` checks all of it in Chrome in CI on three systems; `docs/ACCESSIBILITY.md`. The menu bar icon on macOS waits on #64 (D-110) |
| #64: the menu bar icon by keyboard and VoiceOver | Done (PR #69) | On macOS a press of the menu bar icon, as VoiceOver and Full Keyboard Access make it, opens the popover as a click does: the icon's button gets a target and action in `icon_press.rs`, the tray app's one `unsafe` exception; `tests/icon_press.rs` proves it on a real status item in CI (D-114) |
| 10: the OpenAPI description | Done (PR #66) | `GET /v1/openapi.json` serves `docs/openapi.json`, OpenAPI 3.1, whose schemas are written from the types the API sends and takes, through schemars behind a `schema` feature that only tests turn on; a test fails when the file and the code disagree, another sends every method to every path on a Pro and a Free copy and fails on an answer the file does not list (D-111) |
| 11: package managers, completions and manual pages | Done (PR #67) | Completions for bash, zsh, fish and PowerShell and a manual page per command, written from the commands in a test and committed in `packaging/`; in every archive, the deb and the macOS app; `scripts/packages.mjs` writes the Homebrew formula `quotadial` and cask `quotadial-app`, the Scoop manifest and winget's manifests from a release's checksums; the Packages workflow publishes them on a full release once each token is set (D-112) |
| 5: the website (first PR) | Done (PR #68) | quotadial.com in `site/`: a static Next.js export for Cloudflare Pages with the key page's function; home, /download, /pro, /key, /privacy, /support, /changelog and a 404, the share card and brand files; the privacy, install, changelog and doctor texts rendered from the repository; no cookies, analytics or third-party requests; built by three lanes on a foundation and passed by two independent reviews; deploys only with `CLOUDFLARE_API_TOKEN` and after a release carrying `SHA256SUMS.txt` (D-113) |
| 5: the shareable image (second PR) | Done (PR #70) | On QuotaDial Pro, each account in History has Share image: the window draws a 2400 by 1350 PNG of its history from the view's own numbers, in the window's theme, font and tabular figures, labelled by the name given the account or its plan and never its address; `save_share_image` checks the tier through `Licensing`, reads the PNG's signature, header and end by hand, and saves it in Downloads under a name Rust chooses, never overwriting; on Free one line says it is part of Pro; `ProFeature::ShareImage` everywhere Pro's list is printed; the site's `PRO_ADDS` held to `tier.rs` by a test (D-115) |
| 12: docs | Done (PR #72) | Every download also under a name without its version, copied in the Release workflow's publish job before the checksums, so `releases/latest/download/<name>` always fetches the newest full release (D-116); /download's buttons start the download from `docs/INSTALL.md`'s links; /install and /cli render `docs/INSTALL.md` and `docs/CLI.md` whole; the site deploys only when the newest full release has every copy; the README shows the window in both themes |
| 12, PR A: the fake provider | Done (PR #71) | `quotadial-provider-fake`, id `fake`: made-up accounts called Example, with a plan and no address, from `fixtures/fake/example.json` or the file `QUOTADIAL_FAKE_ACCOUNTS` names, every window an `example` (P-017) that fills at its pace and resets, so the daemon finds it a pace; the `fake` feature lists it alone, and keeps the copy's own data folder, cache folder and port (`quotadial-fake`, 47116); QuotaDial Example, `com.quotadial.example`, from `tauri.example.conf.json`, never updates; a `fake` CI job, and the Release workflow fails if a CLI it ships names `fake` (D-117) |
| 12, PR C: the install check and #52's figures from Matt's copy | Done (PR #74) | `docs/INSTALL.md`'s command alone fetches the archive and `SHA256SUMS.txt` into a new folder, checks the archive and only then unpacks and installs it, making `/usr/local/bin` first, which a new Apple silicon Mac does not have (#72's review); Matt's copy's idle figures read from its processes over two hours (the tray process 0.295% of one core, 37.8 MiB of footprint, 88.2 MiB resident); #52's memory targets judged on the footprint (D-123); one changelog line |
| 12, PR D: the first-run recording and the example's figures | Waiting on Matt (#51) | The README's recording, first run to the number, the popover's open time, the example's idle hour with the popover closed and the daemon alone, all from QuotaDial Example, which needs a few minutes in Matt's menu bar; the scripts are written and dry-run |
| 6: signing and updates | Done (PR #59) | Tauri's updater from the latest full release, checked daily while Settings' switch is on and by hand from Help, installing only on Update, with the version bound into each signature; signing for macOS (Developer ID, notarization), Windows (Azure Artifact Signing) and updates, each a workflow step that runs only when its secrets exist; `latest.json` and `SHA256SUMS.txt` on each release; `docs/RELEASE.md`; one privacy text, `docs/PRIVACY.md`, in Help and the README, with every destination held to it by tests (D-103 to D-106) |

## M6

| Step | Status | Notes |
| --- | --- | --- |
| 1: the groundwork, and Gemini CLI | Built on `feat/m6-groundwork`; Gemini held off by default | P-018: `Provider::commands`, run by the CLI as `quotadial <provider id> <name>` and listed in its help, completions and pages; Google's (`ya29.`, `1//`) and GitHub's (`ghp_`, `gho_`, `ghu_`, `ghs_`, `ghr_`, `github_pat_`) tokens redacted by shape, with every fixture held to it; `quotadial-provider-gemini`: an account when Gemini CLI is signed in, from the sign-in file's presence and one settings field, `unsupported` with one line naming `/model`, nothing read inside the sign-in and nothing sent; the `gemini` feature off until the manager settles what the app lacks; CI lints and tests the Gemini build; P-019 and D-118 to D-122 |
| 3: Antigravity, through its status line | Next | D-119's five rules: a command through P-018 that keeps each window's share and reset in one 0600 file, read by the provider at the next poll; `status_line` as its source; offline from made-up status-line JSON |
| 2: Copilot | After 3 | Built from GitHub's docs and fixtures under the manager's Copilot rules on #73; `api.github.com` in `privacy::HOSTS` and `docs/PRIVACY.md` first; its merge held on its live check (#51) |
| Cursor | Out of M6 (D-120) | Until Cursor says in writing that reading one's own usage with the editor's sign-in is allowed; asking is a #51 checkbox |

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

### M6 PR 1: the groundwork (P-018, P-019, D-118 to D-122)

- **The decisions:** the manager's on #73 (2026-10-25, 20:29 UTC), taken as decided at 20:32 UTC:
  (a) to (d) are D-118 to D-121, the deferred-provider rows are P-019, and P-018 is the one change
  M6 makes in core. The branch is `feat/m6-groundwork`, from `main` at a5cda9e, committed and not
  pushed.
- **P-018** (`crates/core/src/provider.rs`, `crates/cli/src/lib.rs`): `ProviderCommand { name,
  about, run }`, whose `run` takes an `Invocation { args, stdin, stdout, stderr }` and gives an
  `ExitCode`, and `Provider::commands`, empty by default. The CLI's `command()` adds, after its own
  commands, one subcommand per provider that has commands ("Commands the <id> provider adds"), each
  taking any words after its name; `run_provider_command` finds the provider by its id and the
  command by its name, and runs it with the process's streams. `own_commands` drops a provider whose
  id is one of QuotaDial's own commands, so those always win. `run()` makes the provider list once
  and hands it to `status`, `watch`, `mcp`, `doctor` and `daemon`. `packaging.rs` writes its files
  from `crate::command` with the list a release ships (`with_config` with the default settings and
  no licence, so no folder is read); none has a command, so `packaging/` is unchanged. A later
  provider, such as Antigravity's `statusline`, returns its command from `commands`, its run
  function capturing what the provider was set up with.
- **Redaction** (`crates/core/src/redact.rs`): each rule is now a prefix, the bytes that may follow
  it (`Word`, `Underscored`, `Dotted`, `Jwt`) and the fewest that make a token, and the four old
  rules keep their exact behaviour. Google: `ya29.` (more dots allowed, a sentence's last dot kept)
  and `1//`, with 16 or more characters after. GitHub: the six prefixes, with 30 or more of
  `[A-Za-z0-9_]` after. The made-up tokens in the tests are written in pieces, so no secret scanner,
  such as GitHub's push protection, takes a file for one holding a token, and Google's own sample
  refresh token is described rather than quoted. `crates/core/tests/fixtures.rs` now fails on any
  committed fixture that redaction would change.
- **Gemini CLI** (`crates/providers/gemini`), from Gemini CLI's source at cd1b45c (a shallow clone
  in the builder's scratch folder): `homedir()` in `packages/core/src/utils/paths.ts`
  (`GEMINI_CLI_HOME` when it is set to anything, else the home folder); `Storage` in
  `packages/core/src/config/storage.ts` (`.gemini/oauth_creds.json`, `.gemini/settings.json`);
  `AuthType` in `packages/core/src/core/contentGenerator.ts` (`oauth-personal`, `gemini-api-key`,
  `vertex-ai`, `cloud-shell`, `compute-default-credentials`, `gateway`); `settingsSchema.ts`
  (`security.auth.selectedType`); `settings.ts` (comments stripped by `strip-json-comments`, a file
  that is not an object taken as no settings); and the sign-in's other homes (`oauth2.ts`,
  `oauth-credential-storage.ts`: with `GEMINI_FORCE_ENCRYPTED_FILE_STORAGE=true`, the keychain
  service `gemini-cli-oauth` or `gemini-credentials.json`). `/auth signout` deletes the file and
  clears the setting, and choosing another way deletes the file, so the file being there is a sound
  sign of a Google sign-in. What it shows in each case is in D-122.
- **`/stats model`:** it exists (`statsCommand.ts`, `docs/reference/commands.md`), and Gemini CLI's
  docs name it under "Check usage and limits", but `ModelStatsDisplay.tsx` answers "No API calls
  have been made in this session." until the session's first request, and shows the pooled daily
  limit only while the model is auto. `/model` (`modelCommand.ts`) asks for the limits afresh and
  opens the model dialog, whose `ModelQuotaDisplay` shows each model's share used and reset. The
  line names `/model`.
- **The app, checked without changing it,** by reading `apps/tray/src-tauri/src/meter.rs`
  (`account_view`, `sign_in`) and `crates/core/src/text.rs` (`provider_name`); the app is not run
  here, since it would read this Mac's sign-ins. It lacks two things: the provider's name (the
  popover's heading, the tooltip and Settings' providers and accounts would say "gemini", since
  `provider_name` knows only Claude and Codex), and a true sentence for the sign-in panel, which
  would say "QuotaDial reads this account's sign-in from ~/.gemini/oauth_creds.json. It only reads
  it and never changes it." of a file QuotaDial only sees is there. Each needs core or the app to
  change for one provider, so Gemini is held off by default and the question goes to the manager
  (D-122).
- **For the turn-on, after the manager's answer:** `gemini` into `quotadial-providers`' default
  features and the list tests' `cfg` swapped; the app's lockfile gains the crate then. The lines
  people read wait for it: `docs/PRIVACY.md` and the README's copy, the README, `docs/CLI.md` and
  `docs/INSTALL.md` where they name the providers, `skills/quotadial/SKILL.md`, the CLI's help and
  `--provider`'s, its manual pages' ENVIRONMENT (`GEMINI_CLI_HOME`), the Release workflow's
  `no-example-provider.sh` (unset `GEMINI_CLI_HOME`), and a changelog line. A third thing to settle:
  a copy without a licence offers the week's cost for a machine with only a Gemini account, which
  the cost scan cannot give, since it reads no Gemini logs.
- **No changelog line:** a release behaves as before. P-018 adds no command yet, Gemini is off, and
  no shipped provider can put a Google or GitHub token in any text.
- **Tests:** in the workspace, 583 passed (1 ignored) before and 625 (1 ignored) after; the example
  build 583 (2 ignored) before and 625 (2 ignored) after; with `gemini`, 626 (1 ignored). By crate,
  before and after: core 98 to 104 (redaction 4, the hook 2); the CLI 44 to 51 (P-018's 7; 52 with
  `gemini`, whose real-binary test shows a Gemini account in `status`, `status --json` and
  `doctor`); `quotadial-provider-gemini` 29 (environment 1, the lines 2, the settings reader 12 with
  its property test, the provider 14); `quotadial-providers` 1, a test for each build's list; the
  rest unchanged (API 54, cost 30, daemon 100, issue 6, poll 37, Claude 134 and 1 ignored, Codex 61,
  fake 18). Every new test failed first, against a stub or with its change left out, and each guard
  that passed against a stub failed under a mutation instead, each restored after: the redaction
  minimums, a fixture holding a made-up GitHub token, the CLI's empty and clash checks, a `codex`
  alias on `doctor`, and thirteen in the Gemini crate. Two of those, each removing one of the size
  cap's two checks, survived as they should, since the other check holds; removing both failed. A
  panic at a file's end was caught only once the property test's strategy ended files mid-string and
  mid-comment, and an explicit test of those ends was added.
- **A pipe in place of the settings:** opening it would wait for a writer for good, holding up
  discovery and `quotadial status`; the reader now opens only a plain file, as the sign-in file's
  check already looks only for one, and a test fails if reading a pipe takes over five seconds (it
  did, before).
- **Checks run here:** `cargo fmt --all -- --check`; clippy with `-D warnings` on the workspace by
  default, with `fake` and with `gemini`; the tests in each; CI's new Gemini steps; the packaging
  check; actionlint on `ci.yml`; shellcheck, `scripts/public-prd.sh --check`, `scripts/privacy.sh
  --check`, the release notes for 0.2.0 and the 36 script tests. **Not run:** the tray app's job,
  which needs `npm ci`, a release build of the bundled CLI and a cold Tauri build (the app is
  unchanged; its one `Provider` is a test's, which the defaulted hook leaves compiling; and `cargo
  metadata --locked` holds its lockfile current); `cargo deny` and `cargo audit`, not installed here
  (the lockfile gains only the new crate, no third-party one); and the site's job (no change).
- **Safety on this Mac:** every cargo run had `QUOTADIAL_DATA_DIR`, `QUOTADIAL_FAKE_DATA_DIR`,
  `CLAUDE_CONFIG_DIR`, `CODEX_HOME` and `GEMINI_CLI_HOME` in the scratchpad and no token in its
  environment; the CLI's real-binary tests ran in empty homes, now with `GEMINI_CLI_HOME` pointed
  there too; no vendor CLI was run, no `~/.gemini` file, keychain item or sign-in was read, and no
  app or daemon ran against Matt's data. Gemini CLI's facts come from its public source and docs.

### M5 queue item 12, PR D: the first-run recording and the example's figures

- **Waits on** Matt's go-ahead for a few minutes with a second QuotaDial icon in his menu bar, for
  the README's recording and #52's timings (#51). The manager's review of #74 moved this here so the
  install check did not wait on it; the 0.3.0 bump does not wait on it either.
- **To measure, with Matt's go-ahead:**
  - First run to the number, with the example build.
  - The popover's open time: the median of five presses from a warm daemon.
  - The example's idle hour: the app's share of one core, and its footprint and resident size with
    the popover closed, with its helpers.
  - The daemon alone, as the command-line install runs it: the example build's `quotadial daemon`
    for an hour, with its own data folder and port; #52's 40 MB is judged on its footprint (D-123).
  - The README's first-run recording.
  - Readings of Matt's copy's footprint as they come, and one line on whether it grows.
- **How they will be measured:** four scripts and a small Swift helper, kept outside the repository
  with the builder's notes. Each finds QuotaDial Example by its bundle identifier, checked against
  the app's signature too, and refuses every other app.
  - First run: launches the app through Launch Services with `QUOTADIAL_FAKE_DATA_DIR` set to a new,
    empty folder, after moving the example's own earlier folders aside (its cache, settings and
    WebKit folders; never Matt's), and reads its menu bar item through accessibility every 10 ms
    until the item shows a share. It then checks that the app has files open in the new folder and
    none in the default `quotadial-fake`.
  - Popover: presses the item through accessibility, as VoiceOver does, five times, and times from
    the press to the popover's window listed on screen, its first pixels, and the popover drawn as
    it settles (the first picture that matches it 1.5 s later), from ScreenCaptureKit watching the
    example's windows alone; the medians.
  - Idle: as Matt's second hour, footprint included, with the helpers found by responsibility and
    whether the popover was open at each sample.
  - Recording: ScreenCaptureKit with a filter that includes the example's windows and nothing else,
    so the menu bar, other apps' items (Matt's QuotaDial among them), their windows and the desktop
    are never captured, not even in the raw file; then a check that every frame is the plain
    background outside the icon and the popover, a crop to the two, an MP4, a GIF and a poster. The
    account is labelled in the picture itself: the example's accounts are called Example (D-117).
  - Checked without the app: the helper's self-test (the share read from the title, the median,
    lightness, and when the popover first showed and was drawn, on made-up pictures); bad arguments
    refused; every command that needs the example refuses while it is not running; the build passes
    the app check, and four wrong bundles are refused (the real identifier, unsigned, an edited
    Info.plist, a signature naming the real identifier); in a made-up home only the example's
    folders are moved; the crop and the plain check on two made-up recordings, the check catching a
    stray window in one; and ScreenCaptureKit recording two seconds of a 16-point square with no app
    in the filter, every pixel of it the background.
  - Not checked until the app runs: the launch through Launch Services with the variable; the item's
    title as accessibility gives it; the press; the popover's window and its pixels; the example's
    helpers; the recording of the example's windows; the click; HEVC for a screen over 4096 pixels
    wide.
  - Matt's menu bar hides itself, so its icons sit above the screen until the pointer reaches the
    top: the recording needs "Automatically hide and show the menu bar" set to Never while it runs,
    or holds the pointer at the top of the screen (`--reveal`).

### M5 queue item 12, PR C (#74): the install check, and #52's figures from Matt's copy

- **The rule:** the manager's rule 4 on #49 (14:25 UTC): "The recording and the timings come from
  that build: first run to the number, popover open time, the two idle figures. Crop to QuotaDial's
  icon and popover and label the account as an example in the picture itself, not only in the
  caption. The idle figures of Matt's own copy may be read from its process figures as you said,
  nothing more." Running QuotaDial Example puts a second icon in Matt's menu bar, so its timings and
  the recording wait on his go-ahead. The manager's review of #74 (21:26 UTC) split the work: this
  PR carries the install check and the figures read from Matt's copy and merges now, since the
  guide is safer with the check than without; the recording and the example build's figures are
  PR D. One changelog line, for the install guide's lines: whether a person notices is the test,
  not whether it is docs (the review; D-101).
- **#52's targets ("Fast and light"):** the daemon idle under 0.5% of one core averaged over an
  hour and under 40 MB resident; the tray app idle under 80 MB resident with the popover closed; the
  popover open and drawn in under 100 ms from a warm daemon, and the tray's number changed within
  1 s of a poll; first run, a number in the tray within 60 s with no typing (the PRD), today 1 s,
  kept under 5 s; the window bundle within the 150 KB budget `npm run build` enforces. The memory
  targets are judged on the physical footprint, in MiB, with the resident size beside it, and the
  daemon's on the daemon alone (the manager on #74 and #52, 21:26 UTC; D-123).
- **The install line (#72's review, note 1):** `docs/INSTALL.md`'s "The command alone" piped `curl`
  into `tar` and `sudo install` with no checksum step. Its lines now go into a new folder
  (`cd "$(mktemp -d)"`), fetch the archive and `SHA256SUMS.txt` from the version-less addresses
  (`curl -fsSLO`), run the check table's own macOS line, `shasum -a 256 -c SHA256SUMS.txt
  --ignore-missing`, and only then `tar -xzf`, `sudo mkdir -p /usr/local/bin` and `sudo install`,
  every line but the last ending in `&&`, so a step that fails stops the rest. A new Apple silicon
  Mac has no `/usr/local/bin`, and `install` will not make the folder it is given ("target directory
  ... does not exist", exit 64), so the old last line failed there; the guide's own lines for
  linking the app's command already made the folder. The new folder means the check and the install see
  only what the lines fetched: in a folder of earlier downloads, `--ignore-missing` would also check
  an older file listed by the same name and fail on it, and `quotadial-v*/` could match an older
  unpacked version. The text says what to change for another system and for Linux (`sha256sum` in
  place of `shasum -a 256`, which keeps the line's `&&`), and /download's paragraph says the same.
  `shasum` and GNU's `sha256sum` both exit 1 when no listed file was checked, so an archive
  `SHA256SUMS.txt` does not list stops the lines as well.
- **Its tests,** each seen failing on the old lines first: `site/test/conversion.test.ts` holds the
  block's order (new folder, both fetches, the table's check, unpack, the folder, install) and its
  `&&`s, and
  the built page's tests read every line, not only the first (React writes the first line's quotes
  as `&quot;`). `scripts/workflows.test.mjs` makes a release with the Release workflow's own Package
  and publish steps (the archive holding `quotadial-v0.2.0-aarch64-apple-darwin/`, both of its names
  in `SHA256SUMS.txt`) and runs the lines as written, only the address (`file://`), `sudo` and
  `/usr/local/bin` changed, the last to a folder that does not exist until the lines make it, in sh,
  bash and zsh: they install it (with the folder missing, the lines without `mkdir` failed); a byte changed in the gzip header,
  which still unpacks, so only the check can stop it, stops them at the check with nothing unpacked
  or installed; so does an archive `SHA256SUMS.txt` does not list; and Linux's lines, made as the
  text says, do the same where `sha256sum` is GNU's (CI's Linux; skipped on macOS, and passed here
  with GNU coreutils first on the `PATH`). The test notes the folder `mktemp -d` made before it
  asserts anything, so a failure leaves nothing behind. `scripts/latest-names.test.mjs` lets
  INSTALL.md link `releases/latest/download/SHA256SUMS.txt`, which every release carries by that
  name, and still refuses any other name (a misspelt one failed it).
- **The dry run,** against a stand-in release made in a scratch folder: in zsh, bash and sh, the
  archive passes (`quotadial-aarch64-apple-darwin.tar.gz: OK`), is unpacked in the new folder, and
  `quotadial` and `qd` are installed into a scratch prefix and run; with one byte changed, `shasum`
  prints FAILED, the lines exit 1, the new folder holds only the two downloads and nothing is
  installed; Linux's lines pass with GNU's `sha256sum`.
- **The example build:** `npm ci`, then `npx tauri build --config src-tauri/tauri.example.conf.json
  --bundles app` in `apps/tray` with `CARGO_BUILD_JOBS=4`, at a5cda9e:
  `apps/tray/src-tauri/target/release/bundle/macos/QuotaDial Example.app`, 19,652,310 bytes
  (18.74 MiB), arm64. Its Info.plist has `CFBundleIdentifier` `com.quotadial.example`,
  `CFBundleName` QuotaDial Example and `LSUIElement`; it is signed ad hoc as
  `com.quotadial.example`; the app's binary carries `com.quotadial.example`, `quotadial-fake`,
  `QUOTADIAL_FAKE_DATA_DIR` and `QUOTADIAL_FAKE_ACCOUNTS` and not `com.quotadial.desktop`, and the
  command it bundles carries `QUOTADIAL_FAKE_DATA_DIR`. Built, not run.
- **The window bundle:** `npm run build` in `apps/tray`, the example build's first step: "The
  window's JavaScript is 33.9 KB, within the 150 KB budget" (one file, 34.75 kB as vite counts,
  13.01 kB gzipped; the stylesheet 7.14 kB).
- **Matt's copy, idle (read only):** his app's process figures and nothing more, as the manager
  allowed; his copy was built from main at fd3ee4d and has run since 2026-10-24 01:15 UTC. Two hours
  on 2026-10-25, a sample a minute (61 each):
  - 13:38:51 to 14:38:52 UTC, with `ps -o time=,rss=`. It took the two helpers whose process ids
    followed the app's, and so missed the third, GPU.
  - 20:05:02 to 21:05:02 UTC, with `ps` and each process's physical footprint from
    `proc_pid_rusage`, a kernel query that neither pauses nor touches the process. It found every
    helper by the process macOS holds responsible for it
    (`responsibility_get_pid_responsible_for_pid`), looked up again at every sample.

  Sizes are MiB, as `ps` counts KiB. The second hour:

  | Process | CPU over the hour | Share of one core | Resident, MiB: mean (min to max) | Footprint, MiB: mean (min to max) |
  | --- | --- | --- | --- | --- |
  | quotadial-tray (the tray and the daemon, one process) | 10.64 s | 0.295% | 88.2 (86.2 to 96.1) | 37.8 (35.8 to 39.7) |
  | WebKit GPU | 2.66 s | 0.074% | 24.3 (19.6 to 26.9) | 15.1 (10.2 to 15.6) |
  | WebKit WebContent | 1.70 s | 0.047% | 31.3 (28.2 to 39.1) | 37.6 (27.8 to 60.6) |
  | WebKit Networking | 0.42 s | 0.012% | 9.8 (9.5 to 10.5) | 6.5 (6.4 to 6.5) |

  All four together: 153.6 MiB resident and 97.0 MiB of footprint on average, 119.8 MiB of
  footprint at most. The first hour had the tray at 10.36 s of CPU (0.288% of one core) and 78.5 MiB
  resident (75.8 to 88.3), Networking at 0.58 s and 9.6 MiB, WebContent at 1.63 s and 30.5 MiB.
- **Resident and footprint:** a process's resident size counts every page it has in memory, the
  system's shared libraries and frameworks included; its physical footprint, the figure Activity
  Monitor's Memory column shows, counts what the process itself costs the machine, its own memory
  whether in RAM or compressed. That is why WebContent's footprint can sit above its resident size.
- **Against #52:** the daemon runs inside the tray's process, so the daemon's figure and the tray
  app's are one process.
  - CPU: 0.295% of one core over the second hour and 0.288% over the first, under #52's 0.5%.
  - Memory, judged on the footprint (D-123): the tray process's 37.8 MiB passes the tray app's
    80 MB, though only process figures were read, so the popover's state in either hour is not
    known; 88.2 MiB (92.5 MB) resident, beside it. The daemon's 40 MB is measured on the daemon
    alone, in PR D, not on the tray process that carries both. The example's hour, also PR D, gives
    the popover-closed figure.
  - The tray's resident size was 78.5 MiB on average in the first hour and 88.2 in the second; the
    footprint was read only in the second. Whether that is growth over its 44 hours of running or
    the popover having been opened is not known: the example's hour starts fresh, and readings of
    Matt's copy's footprint over the next days, noted on PR D, show whether it grows. A footprint
    that grows across days would be a bug for M6's queue, not a figure for the tracker (the
    review).
- **Tests:** the release scripts 40 (36 before: 4 new, one changed); the site 90 (one rewritten,
  two built-page tests changed); `npm ci`, `npm run check`, `npm run build` and `npm test` in
  `site/` pass, and CI passed at 165f915, its Linux job running all 40 of the release scripts'
  tests, the GNU `sha256sum` one included. The Rust workspaces are unchanged.

### M5: the manager's review of #68, the website

The review came after #68 merged: one point blocking (the changelog tests), one already closed by
#70 (`site/test/pro.test.ts`), and four smaller ones. Each is a commit on `docs/review-fixes`.

- **The changelog tests and the version bump:** `reference.test.ts` and `markdown.test.ts` pinned
  today's order and size (Unreleased first with more than ten additions, then 0.2.0 and 0.1.0), so
  the bump, which renames Unreleased, would have turned CI's site job red and failed the first
  deploy the gate let through. They now assert the shape: versions newest first by number and date;
  Unreleased, when there is one, first with no date; 0.2.0 dated 2026-10-24 with 6 Added and 0
  Fixed; every release dated with a date `sayDate` accepts; Keep a Changelog's kinds only; an
  anchor per version. Each assertion takes the changelog's text, and `test/changelogs.ts` builds
  the shapes they run over from the real file: as it is, as the bump leaves it (Unreleased renamed
  to the next release, dated the day after the newest), and that with an empty Unreleased above.
  The old assertions fail on both (2 of 19 each); the new pass on both, and so does the whole
  suite against the review's own copy, `## [0.3.0] - 2026-10-26`, with and without an empty
  Unreleased above it. Nine broken changelogs and pages each fail them (Unreleased below 0.2.0,
  a release with no date, an impossible date, versions out of order, a number below the last,
  0.2.0 with 5 additions, "### Fixes", the page not using `versionsShown`, `versionsShown`
  keeping an empty Unreleased).
- **Found on the way:** an empty Unreleased heading gave /changelog a plate saying changes are not
  yet in a version, with none under it, and a table row of noughts (seen in a build). The page now
  shows Unreleased only while it lists a change (`versionsShown` in
  `components/reference/text.ts`); with no Unreleased entry it shows no plate, as before, and
  today's page is the same markup.
- **The 30-day History picture:** `site-fixtures.mjs` answered 30 days with the 7-day view, so the
  window check skipped the 30-day picture to share (it does so for a copy with no month of its
  own). It is now `history(30)`, and `history()` takes its days from the period: a row a day
  labelled as the app labels them, the dates as the app writes them (28 September to 27 October
  2026), four readings a day, weekly windows climbing and resetting each week, each day the same in
  either period. The 7-day view is the same, value for value, and every capture is the same pixels;
  the window check with the site fixtures passes 3048 checks over 8 views (2176 before), now
  pressing Share image over 30 days in both themes. `test/examples.test.ts` holds each period to
  the days it is named for; it fails on the old line, and on `history(30)` with the old
  `history()`.
- **The pictures:** no committed picture changes. The accounts and settings pictures in
  `site/public/images/app/` match today's captures byte for byte; the history pictures, which no
  page uses, still predate #70's Share image button and session readings, as noted under item 5's
  second PR, and are left for a regeneration with `node scripts/app-pictures.mjs`.
- **No third parties, held on every push:** `test/hosts.test.ts` reads every built page and the
  stylesheet in out/ (skipping before a build, with the message the other built-page tests use,
  now shared from `test/built.ts`). Every address written in them must be the site's or on its
  allowlist, each host with why: github.com only as `pghmatt/quotadial` and
  `pghmatt/homebrew-quotadial`, www.w3.org (the SVG namespace), schema.org, the two status pages
  and rustup.rs (links in the guides), and 127.0.0.1, example.com and hooks.slack.com (examples on
  /cli); an entry no page uses fails. Nothing in an element's attributes, but a link's href, or in
  a style sheet may be fetched from anywhere but the site. The Buy button's Lemon Squeezy link is
  allowed as that one link when the tests run with the build's `QUOTADIAL_CHECKOUT_URL`; a build
  with one showed the page's script data writes `&` as `&`, which cut the address short until
  the test read those escapes. Planted in a built page, a Google Fonts stylesheet, an analytics
  script, a picture from the repository, a link to another repository, and /cli without its Slack
  example each fail; a third test plants them in memory on every run.
- **wrangler, under the lockfile:** as an exact devDependency alone, `npm audit
  --audit-level=low` failed: miniflare 5.20261019.2-beta, which wrangler 4.152.0 (the newest)
  needs, pins undici 7.29.1, under GHSA-4qv8-2p9j-2685 (moderate), GHSA-f72v-6w45-q472 (moderate)
  and GHSA-xjj6-f33q-929c (high), fixed in undici 7.30.1. The coordinator chose the fix: an
  `overrides` entry pins undici to 7.30.1. The audit passes, `npm ls undici` shows only 7.30.1
  under wrangler, `npm ci` and the build work with out/ the same but for the build id, and the
  locked wrangler starts. What no override reaches: wrangler's own `cli.js` bundles undici 7.29.1
  (110 modules, the three advisories' among them, the content-encodings fix missing), and the
  deploy's calls to Cloudflare's API go through it; npm audit never saw that copy, before or after.
  The override comes out once wrangler's own undici is 7.30.1 or later (D-113, item 12's notes).
- **The deploy:** it publishes with `npx --no -- wrangler pages deploy out ...`, the wrangler
  `npm ci` installed: in CI npx assumes `--yes` and would fetch the newest wrangler if the locked
  one were missing, and without `--` npx reads `wrangler` as the value of `--no`. It builds before
  it tests, so the tests of the built pages run on what it publishes, and `QUOTADIAL_CHECKOUT_URL`
  is now the job's, since the host test allows the Buy link only when it sees the build's; with a
  link set for both, the build and all 72 site tests pass. A workflow test runs the publish step's
  own shell beside a stand-in wrangler, offline, and holds both workflows to npm ci, the build,
  then the tests; it fails on the old publish line, a plain `npx wrangler`, wrangler missing from
  or a range in `site/package.json`, the old order, and the link set on the Build step alone.
- **The guard:** CI's site job runs `npx --no -- wrangler --version` and
  `npx --no -- wrangler pages deploy --help` after `npm ci` (telemetry off), so a wrangler that no
  longer starts fails CI before a deploy. Both commands load miniflare and its undici (a trace of
  what wrangler loads showed it). The step's own shell passes as committed; fails with undici
  overridden to a package that cannot load ("Cannot find module 'undici'") and with wrangler gone
  from the lockfile; and passes once each is put back. An undici that loads but misbehaves (5.29.0
  was tried) still passes: the step proves wrangler starts, not that its HTTP works.
- **The home description** may be 120 to 160 characters, with why in the message; it failed at
  119 and 161 and passed at 120, 140 and 160.
- **D-113** now says what holds /pro's list to `ProFeature::what` (`site/test/pro.test.ts`, since
  #70), names `test/hosts.test.ts`, and says why undici is pinned and when the override comes out.
- **Tests:** the site 72 (65 before: 7 new, the two changelog tests rewritten and the home test's
  length check changed); the release scripts 36 (2 new); the window check with the site fixtures
  3048 checks over 8 views. `npm audit --audit-level=low` (0 found), `npm run check`, the site's
  build, the tray's build and actionlint 1.7.12 (with shellcheck 0.11.0) on every workflow pass.

### M5 queue item 12: docs (D-116)

Item 12 is the docs. This part is the release side of the manager's decision on #49 (05:30 UTC):
from the release after 0.2.0, each download is on the release a second time, under a name without
the version, which the install guide and the site's buttons link to (D-116). `docs/INSTALL.md`'s
tables, already on `feat/docs`, and the site's buttons are the item's other parts.

- **The copies:** `scripts/latest-names.mjs <folder> <version>` copies each of the eight downloads
  (`QuotaDial_universal.dmg`, `QuotaDial_x64_en-US.msi`, `QuotaDial_amd64.deb`,
  `QuotaDial_amd64.AppImage`, and `quotadial-<target>.tar.gz` or `.zip` for the command's four
  builds) beside its versioned file, in whichever folder the download step left it (`dmg/`, `deb/`,
  `appimage/` or the top). It finds the eight by exact name, so a `.sig`, the macOS update,
  `latest.json` or a bill of materials is never copied; it names every missing file before copying
  anything, and refuses a name found twice. `--names` prints the eight for the deploy gate.
- **The publish job** runs it after `latest.json` and before `SHA256SUMS.txt`, in the step now
  called "The updater's latest.json, the copies without the version and every file's checksum".
  `latest.json` cannot name a copy: it is written from the `.sig` files, and a copy has none.
- **The site deploy gate** used to ask for the newest published release, pre-releases included,
  and look only for `SHA256SUMS.txt`. It now checks out main and asks
  `gh api repos/<repo>/releases/latest`, the release `releases/latest/download/` resolves to, and
  sets `ready` only when that release has `SHA256SUMS.txt` and the eight names
  `node scripts/latest-names.mjs --names` prints. GitHub's 404 (every release a pre-release, as
  today) skips with a line in the summary; any other gh error, or a list of names that is missing
  or empty, fails the gate. On an error `gh api --jq` prints GitHub's answer unfiltered and exits 1,
  so the gate reads the answer only after gh succeeds.
- **Found on the way:** Node gives `import.meta.url` as a module's real path and `process.argv[1]`
  as typed, so a script that compares the two as given does nothing, and exits 0, when it is
  started through a link, as macOS's `/var` is, even on input that should fail. `latest-names.mjs`
  compared real paths from the start; `updater-manifest.mjs` and `packages.mjs` compared them as
  given, which was safe on the runners' plain paths, and now compare real paths too, each change
  kept to that check. A test starts each of the three through a link. Without the gate's
  empty-list check, a list of names that came back empty would have let the site out having
  checked no copy (its control showed it).
- **Tests** (CI's `node --test scripts/*.test.mjs`, now 34): `latest-names.test.mjs` (8) holds the
  eight to v0.2.0's real file names, copies each byte for byte into a file of its own beside it,
  copies nothing else, names every missing file and copies nothing, refuses a name found twice,
  prints `--names`, runs through a link, and holds `docs/INSTALL.md`'s `releases/latest/download/`
  links to exactly the eight. `workflows.test.mjs` (8) runs the publish step's own shell, read from
  `release.yml`, on a release laid out as the download step leaves it and signed for v0.2.0, then
  checks every file with `sha256sum -c` (`shasum -a 256 -c` on macOS) and the copies alone with
  `--ignore-missing`; and runs the gate's own shell against GitHub's answers from a fake `gh`:
  ready with all eight, GitHub's real 404 for this repository, one copy missing, no checksums and
  v0.2.0's files as a full release, gh unable to connect, no list of names or an empty one, and no
  token. `updater-manifest.test.mjs` and `packages.test.mjs` gain two each: the copies leave
  `latest.json` naming the versioned files, and no package names a copy or `releases/latest`; and
  each script, started through a symlink in a temporary folder, writes what it writes when started
  directly.
- **Controls, each seen failing, then restored:** the version left in the command's names; a hard
  link, and a rename, in place of the copy; the copy taking its `.sig` too (which also makes the
  manifest refuse the release); no missing-file check, only the first missing file named, and the
  copies made before the check; no twice check; `--names` on one line; the as-given main-module
  check, in all three scripts (in `updater-manifest.mjs` and `packages.mjs` it was the code before
  the fix, which printed nothing through a link); an INSTALL.md link changed, and one removed; the
  publish step without the copy, and with it after the checksums; packages pointing at
  `releases/latest`, or at the msi's copy; and in the gate: no `ready`, another endpoint, no 404
  branch, the old condition (checksums only, which let a release missing a copy through), no
  checksums in the list, a gh error that exits 0, no empty-list check (which let the site out),
  and no token check.
- **How it was checked:** the publish step's shell from `release.yml`, run on a scratch release of
  v0.2.0's names, wrote `latest.json` for 9 targets, all versioned, made the 8 copies, and
  `sha256sum -c` and `shasum -a 256 -c` printed OK for all 24 files, both names of each download
  among them; with only two copies downloaded, `--ignore-missing` passed for both. actionlint
  1.7.12 with shellcheck 0.11.0 passes on all four workflows, and reports an unquoted variable put
  into the gate on purpose. `gh api repos/pghmatt/quotadial/releases/latest` answers 404 today, as
  the gate expects. shellcheck on `scripts/*.sh`, `scripts/public-prd.sh --check`,
  `scripts/privacy.sh --check` and the changelog check pass, and so do the site's type check, its
  44 tests and its build, which read the changelog's new line.
- **Docs:** `docs/RELEASE.md` (step 3's list, step 4, a step 6 for the website, "Names without the
  version", and the checksums under both names); the supply chain in `docs/SECURITY.md`; a line in
  `CHANGELOG.md`; the scripts row in `AGENTS.md`; D-116.
- **Not done here:** no release has the copies until the next tag, so `releases/latest/download/`
  answers 404 until the first full release after it; the Release workflow was not run here.
- **Follow-up, the undici override:** `site/package.json` pins undici to 7.30.1 because the
  miniflare that wrangler 4.152.0 needs pins 7.29.1 (GHSA-xjj6-f33q-929c, GHSA-4qv8-2p9j-2685,
  GHSA-f72v-6w45-q472), and wrangler's own `cli.js` bundles a 7.29.1 that no override reaches.
  Take the override out once wrangler's own undici, in its miniflare (`npm ls undici`) and in the
  `undici@` paths of `node_modules/wrangler/wrangler-dist/cli.js`, is 7.30.1 or later (D-113).

### M5 queue item 12, PR A: the fake provider (D-117)

- **The decision:** the manager's on #49 (2026-10-25, 14:25 UTC): build #52's fake provider with
  item 12, since the README's recording (rule 4, a later PR) is the first thing that needs it. This
  PR is rules 1 to 3 and 5; nothing on the site.
- **The provider** (`crates/providers/fake`, `quotadial-provider-fake`, id `fake`): implements
  `quotadial_core::Provider` like the other two and reads no sign-in, no Keychain and no network.
  `discover_accounts` gives each example as an account called `Example`, with its plan, id
  `example-1`, `example-2` and so on, and no org or sign-in; `fetch` reads each window at the time
  its clock gives; `fallback` is none, `log_folders` none, the floor a minute. Its accounts all
  show on Free and Pro alike: they stand for what a first run finds, one account per provider.
- **The examples** (`fixtures/fake/example.json`, built in with `include_str!`): Claude Max 5x
  with Weekly 91% (2%/h, resets in 1 d 15 h), Weekly (Opus) 74% (1.2%/h) and Session 38% (8.5%/h,
  resets in 2 h 10 min); ChatGPT Pro with Weekly 46% (0.4%/h, resets in 5 d 3 h) and Session 12%
  (3%/h, resets in 3 h 41 min): the site's pictures' numbers (D-113). `QUOTADIAL_FAKE_ACCOUNTS`
  names a file of the same form in their place, read once when the provider is made, at most
  64 KB, every field checked before anything shows: unknown fields, a share or a pace outside 0 to
  100, a window under a minute or over a year and a day, a reset further off than the window, an
  empty name, and an @ in a plan, label or model are refused, with one sentence naming the account,
  the window and the field, and how to fix it. A file that cannot be used shows as one error for
  `fake` and no account.
- **The clock:** examples start at the first whole minute after the provider is made, so the first
  reading says "resets in 2 h 10 min" as the file gives it, not "2 h 9 min". Each window fills at
  its pace from its share then, stops at 100%, and from its first reset starts each window at 0%.
  A reading is never back-dated: `fetched_at` is when it was read, and before the start (a clock
  set back) the numbers are the start's. So the daemon's readings give each window its pace after
  half an hour, as a real account's do; a test fits one from four readings.
- **The source** (P-017): `Source::Example`, `example` in JSON, "the example build" in words
  ("from the example build just now"); `served_from_cache` keeps it. `docs/openapi.json` is
  written again, and `docs/CLI.md` and the skill name it.
- **The feature:** `fake` on `quotadial-providers` (with `dep:quotadial-provider-fake` and
  `quotadial-core/fake`), off by default; `with_config` is two functions, one per build, so a
  release's list is untouched. The CLI and the tray app have a `fake` feature that turns it on,
  and `cargo test --workspace --features fake` turns on core's, the list's and the CLI's together;
  a CLI test fails, naming the command to use, when the list's is on without the CLI's.
- **Nothing shared** (rule 3): `quotadial_core::copy` holds what differs: `EXAMPLE`, `FOLDER`
  (`quotadial-fake` for the data folder and the cache folder) and `DATA_DIR_VARIABLE`
  (`QUOTADIAL_FAKE_DATA_DIR`, so a real copy's `QUOTADIAL_DATA_DIR` cannot point the example at its
  data); the API's `DEFAULT_PORT` is 47116 in the example. `daemon.lock`, `token`, `daemon.json`,
  `history.db`, `costs.db`, `config.json` and the licence all live in the data folder, so all
  follow. The messages that name the variable name the copy's own. Status pages are read only for
  providers with accounts (`claude`, `codex`) and the cost scan reads only theirs, so the example
  build asks no one.
- **The example app:** `apps/tray/src-tauri/tauri.example.conf.json`, merged by
  `npx tauri build --config src-tauri/tauri.example.conf.json` from `apps/tray`: productName
  QuotaDial Example, identifier `com.quotadial.example` (its own settings folder, launch-at-login
  item `QuotaDial Example`, single-instance socket and notifications), `build.features ["fake"]`,
  and before-commands that bundle the example's CLI (`npm run cli -- --fake`, which
  `prepare-cli.mjs` builds with `--features fake`). `run()` refuses to start when the fake
  provider and the example's name do not go together, either way round. The example never checks
  for updates (`Stage::Never`, "QuotaDial 0.2.0, the example build, which never checks for
  updates."), since an update would make it QuotaDial itself. An account whose every number is an
  example says it has no sign-in, in place of where its sign-in is read. Not run here: the
  recording is the next PR's.
- **Proof** (rule 2): the list's tests (exactly `claude` then `codex`; exactly `fake`); a `fake`
  CI job on three systems (clippy and tests, both workspaces), in `ci-success`'s needs; and
  `scripts/no-example-provider.sh`, which the Release workflow runs on the CLI each `cli` leg
  builds (Rosetta first on the Intel leg) and on the CLI each app leg bundles. It runs `status
  --json` and `doctor` in a scratch home and data folder with no token in the environment, stops
  each after 60 seconds, ignores their exit status (no sign-in is fine), needs each to print
  something, and fails if either names `fake`. Run here: on the example build it failed, naming
  both lines; on a default build it passed; on a stand-in that printed nothing and one that hung,
  it failed (the hanging one stopped after 60 seconds, by its own PID). actionlint is clean.
- **Safety on this Mac:** every test run had both data-folder variables in the scratchpad; the
  default build ran only in empty homes (the CLI's own tests and the release check), where the
  Keychain's user search list is empty, `CLAUDE_CONFIG_DIR` and `CODEX_HOME` are empty folders and
  no token is set. The example CLI ran with its own default folders: under `sandbox-exec` with a
  profile that denies every read and write in Matt's data, cache and app folders, every
  `~/.claude*` and `~/.codex*` path and `~/Library/Keychains`, running `/usr/bin/security`, and all
  network but this machine's own. `status`, `status --json` and `doctor` showed the two examples,
  `doctor` with `~/Library/Application Support/quotadial-fake` as the data folder; its daemon
  listened on 127.0.0.1:47116 with only that folder's files open (`lsof`), while Matt's
  `daemon.lock` stayed held by his app's process alone, before, during and after; his folders had
  the same 36 entries before and after; the example's cache folder was never made. Its data folder,
  with the example's own history and token, is left for the recording.
- **The CLI's tests** now hold each build to what an empty home shows (nothing, or the two
  examples), with `ASKS` for how often each provider is asked; a test's daemon is stopped if the
  test fails, where before a failed daemon test left the daemon running, holding the run's output
  open (a control here waited 13 minutes on one). The packaging test is ignored in the example
  build, whose daemon's default port differs from the pages'.
- **Tests:** 583 in the workspace in either build (25 new; the example build runs 14 CLI tests where
  a release runs 13, and ignores the packaging test), 18 of them the fake provider's (2 property
  tests), and 81 in the app in either build (4 new). Each new test was seen to fail on code broken
  on purpose, 43 controls in all, each restored after: the source's name, label and replay; the
  copy's folders, variable and port in each build; every refusal, the clock, the pace, the reset and
  the 100% stop in the provider (its property test first passed with the stop removed, as random
  files were rarely readable, so its strategy now makes mostly readable ones, and then failed); each
  list; the CLI's tests on a renamed example, a wrong label, a slower floor and a release's folder,
  and the build check with only the list's feature on; the app's name, its refusal, the update stage
  and its words, and the sign-in line.
- **No changelog line:** a release behaves as before; only the API's description and the docs
  name the `example` source, which a release never sends.
- **Left for later:** rule 4, the recording and timings made with the example app. On Windows the
  data folder comes from the system, not the home folder, so the test of the example's default
  folder runs on macOS and Linux. The Release workflow's new steps run first with the next tag.

### M5 queue item 5, second PR: the shareable image (D-115)

- **The Pro feature:** `ProFeature::ShareImage`, "a picture of an account's history to share",
  right after a year of history in `ALL`, with its line "A picture of an account's history to
  share is part of QuotaDial Pro: quotadial.com/pro". Everything that prints the list follows:
  `quotadial licence` on Free (its test), Settings' list on Free (its test now finds the week's
  cost at index 5), the OpenAPI document's `ProFeature` (written again), the Free and Pro table in
  `docs/CLI.md`, and the README.
- **The command** (`apps/tray/src-tauri/src/share.rs`, `save_share_image` in `lib.rs`): the PNG
  arrives as the raw body of a `tauri::ipc::Request`. The tier is asked first, through the copy's
  `Licensing`, and Free gets the feature's line whatever was sent. Then, read by hand: at most
  4 MB, the eight-byte signature, a first chunk of 13 bytes typed `IHDR` whose CRC-32 matches
  (worked bit by bit; tested against the CRC-32 check value and `IEND`'s), the `IEND` chunk last,
  and exactly 2400 by 1350. A JSON body (Tauri's postMessage fallback would send one) is refused
  as a form QuotaDial cannot read. The account is its settings key in the `quotadial-account`
  header, percent-encoded by the window and decoded here; `history::label` finds it in the
  history's newest snapshots, so the plan is the account's own, and an account the history does
  not know is refused. The file name is Rust's: `QuotaDial <label> <local date>.png`, the label
  kept to letters, figures, spaces and `- _ ( ) + & ' ,` (everything else a space, runs of spaces
  one), at most 80 bytes, cut at a character's edge; files are created with `create_new`, so a
  taken name moves on to ` 2`, ` 3` and so on to 999, and half-written files are removed. The
  answer is "Saved to ~/Downloads/<name>." through `text::tilde`; each refusal is one sentence
  that names what to do, and none carries a path. Property tests (#52) prove no bytes make the PNG
  reader panic, no header makes the decoder panic, every label makes a safe name, and no label
  shows an @ or a control character; each was seen to fail on a module broken on purpose (a dot
  kept in names, an @ let through, the length guard gone, an escape read past its end).
  `docs/SECURITY.md` now lists the Downloads folder and what a window can make the app save.
- **The view:** `HistoryView` gains `dates` (from the period's first day, or the history's first
  day when it began since, to today: "19 to 25 October 2026", "26 September to 25 October 2026")
  and `share_left_out` (the line on Free, `null` on Pro); `AccountHistory` gains `label`, made by
  `share::label`: the provider and the name given the account, or the provider and its plan with
  a capital (`max` is Max), or the provider alone; a name or plan with an @ is passed over, a name
  that starts with the provider is not given it twice, and control characters are spaces.
- **The picture** (`apps/tray/src/share.ts`): QuotaDial and quotadial.com small at the top right;
  the label at 46 px and the period with its dates beneath; a column per window with its name,
  its latest share at up to 84 px (shrinking so "100%" fits any number of columns) in the text,
  warning or high colour by its level, its highest only when that differs from its latest (as for
  a session that reset during the period; otherwise its room is left empty, so the charts in a
  row stay level), and its chart with the share filled beneath the line and the dashed line at
  90%, labelled "90%" in 16 px at its right end, on the picture's ground (the chart gives it the
  room), since the picture is read with nothing around it to say what the line is; then the
  heatmap, every day named over a week and every seventh over a month, with the hours 0, 6, 12
  and 18. Colours come from
  `--background`, `--text`, `--muted`, `--warning`, `--high`, `--bar`, `--bar-normal` and
  `--bar-warning` as the window's theme has them; text uses only the four `check-contrast.mjs`
  holds to 4.5:1. The font is the root's stack. A canvas takes no `font-variant-numeric`, so a
  `Pen` lays each figure out in the room of the widest, centred, as tabular figures are; text too
  wide for its room is cut short with "…". `shareStrings` lists everything drawn, from the same
  words the drawing uses. A picture is about 220 to 270 KB; the window's JavaScript went from
  28.8 KB to 33.7 KB of its 150 KB.
- **The panel:** a Share image button after each account's heatmap, named "Share image of
  <label>"; another press while a picture is being made is ignored. The status line is now always
  in the page (empty until a file is saved), so a screen reader hears each new line in it; the
  alert line shows a refusal. On Free, one `.left-out` line after the accounts, only when there
  are accounts. A rule above Export CSV and Done keeps the last account's button with its
  account.
- **The window check:** the stand-in answers `save_share_image` as the app does (by the key in the
  header, or refusing an account it does not know), records a raw body by its size, and records
  every `fillText` and `fillRect` a canvas makes with its colour and font. `checkSharing` runs in
  both themes: on Pro, each account's button is after the periods and before Export CSV in Tab
  order and named by its label; Tab and Enter send the app a PNG for that account, read in Node
  (signature, header and its CRC, end) and held to 2400 by 1350 as fixed numbers; History says
  where it went; and every string drawn is in `shareStrings` (imported from `src/share.ts`, which
  Node 22 runs with its types stripped), with no @, in one of the stylesheet's text colours, in
  the window's font, figures one by one, on the window's background. It rebuilds the strings the
  canvas wrote (pieces in one font, colour and line that run on) and holds them to `shareStrings`
  both ways, writes "highest" only for a window whose highest differs from its latest and "90%"
  in small type beside every chart, and measures every piece of text at 4.5:1 against the
  canvas's own pixels behind it, read just before it is written and again just above it when the
  picture is encoded (the first reading alone missed a label drawn before the chart that then
  covered it). It shares each account over 7 days and the first over 30 where a copy has a
  month. On Free: no button and the line once.
  `--share-images <folder>` saves each picture. A key's drawing now counts as the key doing
  something, since the save comes after the PNG is encoded. Seen to fail on 11 windows broken on
  purpose: a button named by the address, no button on Pro, a button on Free, the line twice, a
  picture at 1x (which first passed, as the check took the size from the module it checked; it
  now holds 2400 by 1350 fixed), the address as the title, a colour not from the stylesheet,
  figures drawn whole, the picture sent for another account, a ground not the background, and a
  picture that never reaches the app.
- **Fixtures:** both sets carry `label`, `dates` and `share_left_out`, and a session or 5-hour
  window in either fills during each working day and resets, so its highest is above its latest
  while a weekly window's is not. In `window-fixtures.mjs`
  the Claude account's given name is now long ("Work, the team account we share across two
  offices"), so the picture cuts it short, and the Codex account has none, so History names it by
  its very long address, which the @ check needs.
- **The site:** `PRO_ADDS` and the home table's History row list the feature, /pro's ledger has
  its row and counts eight additions, and `test/pro.test.ts` reads `tier.rs` (ALL's order, what's
  words) and fails when `PRO_ADDS` differs, seen to fail before the site listed the feature, with
  two entries swapped and with a word changed; its own control proves it follows ALL's order.
  Captured with the Brain's `finish-shots.mjs` at 1440x900 and 390x844: the new row wraps cleanly.
- **The website's example accounts' pictures**, from `check-window.mjs --fixtures
  ./scripts/site-fixtures.mjs --share-images <folder>`, light and dark, Claude Work and Codex Pro,
  are kept outside the repository for review; none is committed.
- **Left for later:** the site's `public/images/app/history-*.png`, which no page uses, predate the
  button; home's lead sentence and /pro's description summarise Pro without the picture; the
  existing "QuotaDial could not open its history" and "could not read its history" errors do not
  name a fix (#52).
- **Read alone** (the coordinator's review): "highest" repeated the latest share in every column
  of the first pictures, and nothing said the dashed line was at 90%. Both fixed; the new checks
  failed on the earlier `share.ts` (three "highest" lines where one adds something, no 90% label)
  and on five more broken pictures: the label inside a chart's ground, in either theme; the label
  in large type; `shareStrings` without it; and "highest" back under every window.
- **Tests:** 558 in the workspace (2 new) and 77 in the app (17 new, 4 of them property tests); the
  window check passes 3350 checks over 16 views (640 before); the site 46 (2 new).

### M5: the menu bar icon by keyboard and VoiceOver (#64, D-114)

- **The fault:** on macOS, VoiceOver (VO-Space) and Full Keyboard Access (Control-F8, then Space)
  press a menu bar icon's button through the accessibility interface. tray-icon 0.25 answers only
  mouse events, on a view it lays over the button, and leaves the button with no action, so a press
  opened nothing.
- **The fix:** `apps/tray/src-tauri/src/icon_press.rs` defines a small Objective-C class with
  objc2's `define_class!`, whose `press:` runs a closure, and makes it the button's target with
  `press:` as its action; `build_tray` calls `answer_presses` on the status item tray-icon made,
  with `toggle_popover`. The target is kept in a thread-local for as long as the icon, since AppKit
  does not keep a control's target alive. A click still goes to tray-icon's view, and tray-icon's
  menu press (it sets the menu, presses the button, clears the menu) goes to the menu. Before
  QuotaDial has been clicked once, the positioner does not know where the icon is, so a press opens
  the popover in the middle of the screen, as Spotlight does.
- **The exception:** `unsafe_code = "deny"` in the tray app's `Cargo.toml` (the core workspace keeps
  `forbid`) and `#[expect(unsafe_code, reason = "...")]` on `pub mod icon_press;` in `lib.rs`. Its
  blocks: `init` on the new target, setting target and action, and `performClick` in
  `press_as_the_tray_library_does`, which only the test uses (objc2 marks `performClick` `unsafe`,
  and keeping it here keeps the exception to one line). `objc2` 0.6.5 and `objc2-app-kit` 0.3.2,
  already in the tree through tray-icon, are now direct dependencies on macOS; `dispatch2` 0.3.1,
  also in the tree, is a dev-dependency.
- **The test:** `tests/icon_press.rs` runs with `harness = false`, since AppKit needs the main
  thread and a test harness never gives one. It starts the app's event loop, then from the main
  queue: VoiceOver's press (`accessibilityPerformPress`) runs the action once; tray-icon's press with
  no menu runs it again; the same press with a menu set leaves it at two. A watchdog ends the test
  after 30 s, and a session that gives no status item says so and passes. Controls: without
  `setAction` the first assertion fails (0 presses), and `unsafe` added anywhere else in the app
  fails to build ("usage of an `unsafe` block"). The first version of the test used an empty
  `NSMenu` while asserting that the menu tracked; it never did (an empty menu does not open, and a
  menu that opens is not closed by the main queue while it tracks), so the test now compares the
  same press with and without a menu, as #64's probe did with a menu that opened.
- **Docs:** `docs/ACCESSIBILITY.md` gives the VoiceOver and Full Keyboard Access steps and drops the
  entry from Not there yet; `docs/SECURITY.md` names the exception; a line in `CHANGELOG.md`.
- **Not done here:** the app's wiring was not pressed live on this Mac, because a second copy of
  the app would poll the providers beside Matt's running copy, against the polling floor; the test
  covers the module on a real status item, and #64's probe covered a real press on a test copy.
  (b), the upstream PR to tray-icon, waits on Matt's go-ahead on #51.

### M5 queue item 5: the website (D-113)

Item 5 is the website and the shareable image (decision f on #40). This PR is the website; the
shareable image, a picture of an account's history made in the app, is a Pro feature of History
and comes in its own PR.

- **How it was made** (the brain-site-build pipeline): product truth (`site/PRODUCT.md`); a dated
  search study (DataForSEO, Google US desktop; the study and its captures are private, the route
  map and blueprints are in `site/docs/seo/`); seven grounded candidate worlds and a seeded roll,
  which gave the tide table and the almanac (`site/docs/design/candidates.md`); the identity
  (`site/docs/brand/identity.md`) and a Design Contract checked by codus-ui-proof
  (`site/docs/design-contract.json`, `site/docs/DIRECTION.md`); the content bible and image
  manifest; a copy library cast from 28 pre-2022 pages, with no third-party prose committed
  (`site/docs/copy/`). Then the foundation (tokens with Tailwind's defaults cleared, the
  primitives, the header and footer, fonts, the brand files, the app's pictures from typed example
  accounts) and three lanes on worktrees: home; /download, /pro and /key; /privacy, /support,
  /changelog and the 404. Each lane read its skills in its own context, reported the reads, and
  wrote a report the merge train checked against its branch tip before merging.
- **The pages:** home (the hero is a plate: the week's curve meeting the dashed high-water line,
  a tide table of the three windows, and the real window, with example accounts); /download (the
  visitor's file first, files banded by system, the check commands and the unsigned-build steps
  read from `docs/INSTALL.md`); /pro (the seven Pro features in the app's words, what stays free,
  how buying works; "Buying QuotaDial Pro opens soon" until `QUOTADIAL_CHECKOUT_URL` is set);
  /key (reads `#licence=`, posts it to `/api/key`, shows each answer in the function's own
  sentence, and takes the key out of the address); /privacy (`docs/PRIVACY.md` word for word);
  /support (the doctor sample read from `docs/CLI.md`); /changelog (`CHANGELOG.md`, newest first).
- **One source:** `lib/markdown.ts` renders the repository's texts at build time; `installGuide()`
  reads INSTALL.md's file table, check commands and unsigned-build section, and a renamed section
  or a lost table fails the build (a control did). INSTALL.md gained the check commands as a table.
- **No third parties:** the fonts are self-hosted; events stay in `window.quotadialEvents`; the
  Brain's browser check records every request each page makes, and every one is to the site.
- **Supply chain:** `npm audit` for the site and the app's window in CI's supply-chain job, weekly
  too; both lockfiles; the dependencies are next, react, react-dom, marked, four
  `@fontsource-variable` fonts and `@phosphor-icons/react`, with Tailwind and TypeScript to build.
- **Deploy:** `.github/workflows/site-deploy.yml` publishes to the Pages project `quotadial` when
  `CLOUDFLARE_API_TOKEN` is set and the newest published release carries `SHA256SUMS.txt` (the
  site describes the checksums, `quotadial doctor`, licence keys and updates that arrive after
  0.2.0; today's newest, v0.2.0, has none, so the gate holds). Downloads go to `releases/latest`,
  which GitHub sends to the release list while every release is a pre-release. Buttons that start
  the download itself come with item 12, from version-less copies of each release file (the
  manager's check on #49, 05:30 UTC on 2026-10-25).
- **Found and fixed on the way:** commands cut off on a phone (CodeBlock now wraps, and scrolls
  only for aligned output, as a labelled region); JetBrains Mono drawing `--` as one long dash
  (ligatures off in code); a strip under the footer on short pages; the curve's labels touching,
  and unreadable at 390 (TideCurve draws a narrow version under `sm`); `next dev` rewriting
  `site/AGENTS.md` (its block is committed); codus-site-seo's copy check looking for four stock
  phrases (the config now carries the site's ban list), and missing a phrase that opens an element
  (the Brain's own check reads each page's visible text; reported to codus).
- **The reviews:** the first pass of both independent reviewers said "fix". Design: the tide table
  wrapped every time in the hero, tables were striped by position rather than banded by meaning,
  text was right-aligned, /privacy ran in two columns, /pro opened with two headlines, and plates
  were bare rectangles (they now carry a chart plate's neatline). SEO and conversion: the site
  described features only the next release has (the deploy gate and the corrected claims), the
  download buttons landed below the download, /key's "works offline for ever" stated a plan term,
  and home promised every Claude account on Free. Each finding went to the lane that owned it, or
  to the foundation. After the second pass, every running paragraph takes one reading measure,
  52ch; on the built pages at 1440 px the longest full line of every paragraph is 72 characters or
  fewer, but for one on /download at 75 (narrow letters in `docs/INSTALL.md`'s Windows text). The
  reviewers' last verdicts are in PR #68's description.
- **The design system as built:** `site/DESIGN.md`, written from the merged code (with
  `.impeccable/design.json` for design tools). Writing it found the Semi Condensed captions set in a
  slope the browser invents, since only that width's upright face was loaded; its italic is now
  loaded, and `test/fonts.test.ts` fails when a width the site sets lacks its italic file. The
  Brain's browser check then found a command in an italic caption slanted in the code face, which
  has no italic; code is now always upright.

### M5 queue item 11: package managers, completions and manual pages (D-112)

- **Completions and manual pages, from the commands:** `crates/cli/src/packaging.rs`, compiled
  only for tests, writes `packaging/completions/` (`quotadial.bash`, `_quotadial`,
  `quotadial.fish`, `quotadial.ps1`, and the same for `qd`) with clap_complete 4.6.11, and
  `packaging/man/` (17 pages: `quotadial.1` and one per command and subcommand) with clap_mangen
  0.3.3 and roff 1.1.1, all three dev-dependencies of the CLI only. `quotadial(1)` adds EXAMPLES
  (from the examples `quotadial --help` now ends with), EXIT STATUS, ENVIRONMENT
  (`QUOTADIAL_DATA_DIR`, `CLAUDE_CODE_OAUTH_TOKEN`, `CLAUDE_CONFIG_DIR`, `CODEX_HOME`), FILES (the
  data folder on each system) and SEE ALSO; clap's `help` subcommand gets no page. Tests:
  `the_completions_and_manual_pages_are_the_commands` (any difference fails, and
  `QUOTADIAL_WRITE_PACKAGING=1 cargo test -p quotadial packaging` writes them again),
  `every_command_and_argument_says_what_it_is_for` (its first run found `alerts remove`, `on` and
  `off` with an undocumented rule number, now "The rule's number, as `quotadial alerts` lists
  it"), and `there_is_a_manual_page_for_every_command`. `quotadial --help` gains a description and
  five examples.
- **Shipped:** the release's command archives carry `completions/` and `man/`; the deb maps them to
  `/usr/share/man/man1`, `/usr/share/bash-completion/completions`, `/usr/share/zsh/vendor-completions`
  and `/usr/share/fish/vendor_completions.d` (`tauri.linux.conf.json`); the macOS app carries them
  in `Contents/Resources` (`tauri.macos.conf.json`), checked in a local `tauri build`.
- **The packages:** `scripts/packages.mjs <version> <SHA256SUMS.txt> <folder>` writes
  `Formula/quotadial.rb` (both commands, the pages with `qd.1` linked to `quotadial.1`, the
  completions, a test of `--version` and `--help`), `Casks/quotadial-app.rb` (the dmg, the app's
  command, every page and the three completions, `uninstall quit` and a `zap` of the folders
  QuotaDial makes, `auto_updates` only when the release has `latest.json`), `bucket/quotadial.json`
  (the Windows archive, with `checkver` and `autoupdate` from `SHA256SUMS.txt`) and winget's three
  manifests (schema 1.10.0, the msi with its product code). `scripts/packages.test.mjs` (8 tests)
  runs in CI with the other scripts' tests. `.github/workflows/packages.yml` runs on a full
  release, or by hand with a version: it checks the release's files against `SHA256SUMS.txt`, reads
  the msi's product code with msitools, writes everything as the artifact `packages`, commits the
  formula, cask and manifest to `pghmatt/homebrew-quotadial` when `HOMEBREW_TAP_TOKEN` is set, and
  opens winget's pull request with wingetcreate 1.12.13.0, checked against its SHA-256, when
  `WINGET_TOKEN` is set.
- **How it was checked:** in a local tap on this Mac, against archives built from this branch
  exactly as the Release workflow packs them and a dmg of the app built here, with the real
  v0.2.0 checksums for the other files: `brew audit --strict` passed for both after two fixes it
  found (a cask cannot say `conflicts_with formula:`; the formula's `version` was redundant), and
  `brew style` after a third (`depends_on :macos`, since Homebrew supports no macOS older than the
  app's minimum). `brew install` of the formula put `quotadial` and `qd` on the path with 18 pages
  and the bash, zsh and fish completions, and `brew test` passed; the cask, installed into a
  scratch folder, linked the app's command, 17 pages and three completions. Both were uninstalled
  and the tap removed. The Scoop manifest and winget's three manifests pass their published JSON
  schemas, and a manifest broken on purpose fails each. Bash completion offered `alerts` for `al`
  and `--since` for `history --s`. `actionlint` passes on both workflows. A Release run by hand on
  this branch (run 40161006745) built every file: its command archives hold `completions/` and
  `man/`, and its deb installs `quotadial` in `/usr/bin`, the 17 pages in `/usr/share/man/man1`, and
  the completions in `/usr/share/bash-completion/completions`, `/usr/share/zsh/vendor-completions`
  and `/usr/share/fish/vendor_completions.d`.
- **Not done here:** the tap repository and the two tokens are Matt's (#51); until then each full
  release's files are an artifact of the Packages workflow. The changelog gains its line about
  Homebrew, Scoop and winget when the tap is live.

### M5 queue item 10: the OpenAPI description (D-111)

- **What it is:** `docs/openapi.json`, the local API in OpenAPI 3.1: 6 paths, 8 operations and 29
  schemas, with the bearer token as its security scheme, and the answers every request can get
  whatever it asks (403 for a wrong `Host`, 401 for a missing or wrong token). The API serves the
  same bytes at `GET /v1/openapi.json`, behind the token like everything else, on Free as on Pro;
  `include_str!` puts the file in the binary, so the command and the app ship no schema code.
- **Written from the code:** `crates/api/src/openapi.rs`, compiled only for tests, builds the
  description from the types the routes really send and take (`StatusReport`, `HeadroomReport`,
  the history's `Series`, the alert rules' `StoredRule`, `Alerts`, `NewRule`, `RuleChange` and the
  rest), with schemars 1.2.3 behind a `schema` feature in core and the daemon that only the API's
  dev-dependencies turn on (`#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]`). What
  is sent is described by serde's serialising attributes (a field skipped when empty is not
  required) and what is taken by its deserialising ones. The descriptions are the types' doc comments, cleaned for people who use the API: lines
  joined, decision numbers dropped, Rust's link brackets taken off. Doc comments that spoke only to
  the code became `//` comments, and a test fails on a description that still speaks of the code: a Rust
  path, the PRD, an issue number, a link, or a line left unjoined.
- **Kept honest by tests:** `the_committed_spec_is_the_api` fails when `docs/openapi.json` and the
  code disagree (`QUOTADIAL_WRITE_OPENAPI=1 cargo test -p quotadial-api openapi` writes it again);
  `every_route_is_described_and_every_reference_resolves` checks the file describes exactly the
  API's eight operations and every `$ref` resolves; `the_api_answers_as_its_openapi_description_says`
  sends every method to every path on a Pro and a Free copy and fails on any status the file does
  not list for it, and on any method it does not describe that is not a 405;
  `the_api_serves_its_openapi_description` checks the bytes, the type and the token;
  `descriptions_use_quotadials_words` fails on "quota", "auth" or "credential" outside a name in
  backticks or quotes (#52's plain words). Controls: taking the 400 out of `POST /v1/alerts`'
  answers failed the third, a changed doc comment in `model.rs` failed the first, and the words
  test failed on its first run on `ProviderError`'s "its credentials were unreadable", which now
  says "its sign-in could not be read".
- **Checked by two validators:** openapi-spec-validator and Redocly's linter (`@redocly/cli` 2,
  its recommended rules) both report the file valid, with no warnings.
- **Docs:** `docs/CLI.md`'s API table, the agent skill and AGENTS.md name the route and the
  command that writes the file; a line in `CHANGELOG.md`.
- **Not done here:** axum's router cannot list its routes, so a new route is added to the builder
  and to the test's list of operations by hand; the 404's text names the routes as well.

### M5 queue item 9: accessibility (D-110)

- **The window check** (`apps/tray/scripts/check-window.mjs`, `npm run check:window`, a step in
  CI's `tray` job on macOS, Windows and Linux): serves `dist/` on 127.0.0.1 under the app's own
  content security policy, starts Chrome headless with a profile of its own, and drives it over the
  DevTools protocol with Node's WebSocket, so it needs no package. A stand-in for the app answers
  the window's commands from `scripts/window-fixtures.mjs` (made-up accounts on example.com, one
  with a 65-character address; a Pro copy and a Free one), which `tsc` checks against
  `src/meter.ts`; a command the app does not have fails the check. Over the accounts, Settings,
  History and Help, in both themes: Tab order equals the order shown, each control draws the
  window's ring, Shift-Tab goes back; each control's role, name and state as Chrome's accessibility
  tree gives them, failing a field named only by its hint; every chart, meter and picture named;
  the heatmap's words a line per row and its grid hidden; each control, on a window opened
  afresh, does something when pressed with Enter (Space for a box, typing and Enter for a field),
  and the chosen period stays chosen; a panel takes the focus and Done gives it back to its
  button; no text past the window's edge or cut off by a box, at 100% and at 200% (the page as
  wide as the window at that zoom, with every disclosure open, and each field's hint measured
  against its field); nothing animating or in transition with less motion asked for; no error,
  rejected command or refused style on the page. 640 checks over 16 views, about 75 s.
  `-- --screenshots <folder>` saves each view, `-- --only "pro, dark"` runs some.
- **The controls:** a window with eleven faults put in on purpose (no ring on buttons, a field
  named by its hint, History first in Tab order, Export CSV answering only the mouse, History not
  taking the focus, Done not giving it back, buttons that move, the heatmap's words gone, a chart
  with no name, an unknown command, an inline style) failed 154 of 509 checks, naming each fault;
  the two hidden behind History's broken Tab order failed on their own. The first run of the check
  on the real window found 24 failures, all text cut off: the Names fieldset as wide as its longest
  word (a fieldset's minimum is its content's), the long address in History's heading and
  Settings' Names, "Write to support@quotadial.com" at 200%, Remove and Add pushed off the side,
  and the licence field's hint. CI then found one more on all three runners, where a scroll bar
  takes room (a Mac's lies over the page): at 200% Settings' Names fields kept a text field's own
  width and pushed their labels off the side. They now shrink with the window, and the check
  leaves a scroll bar's room on every system, so a Mac finds what Windows and Linux show.
- **Fixes:** `overflow-wrap: anywhere` on the body, `min-width: 0` on fieldsets, `max-width: 100%`
  on buttons, and an alert's Remove and a field's Add wrap below when there is no room; tabular
  figures on the root; in the light theme `--muted` `#67676c` and `--warning` `#a05600`, since
  main's failed three pairs (a warning 4.49:1 on the background and 3.93:1 on a tag, grey 4.30:1 on
  a tag); `--accent` and `--on-accent` (4.70:1) for the chosen period, which was white
  on `#0a84ff` (3.65:1), bold as well; `--field` for text fields' edges (3.33:1 light, 3.24:1 dark;
  they were 1.25:1), both pairs in `check-contrast.mjs`, whose control with the old colours fails
  on three pairs; `usePanelFocus` (`src/focus.ts`) and the footer's `data-panel` buttons.
- **Zoom:** `zoomHotkeysEnabled` and `core:webview:allow-set-webview-zoom`. On macOS and Linux
  Tauri's script zooms in steps of 20% (Command or Control and = or -, and 0); on Windows it is
  WebView2's own. `fit` now takes the page's width in CSS pixels, and `fitted_height` sizes the
  window to the content at that zoom (a unit test). A probe confirmed on this Mac that WebKit's
  page zoom narrows the page in CSS pixels (360 to 180 at 200%), which `fit` relies on.
- **Live, on a test copy of the app** (its own identifier and binary name, a scratch data folder):
  the macOS accessibility tree, which VoiceOver reads, names every control with its role and
  state, gives each bar's words ("20% used, resets in 1 d 20 h"), the charts' descriptions and the
  period buttons as toggles; History and Settings took the focus when opened and Done gave it back
  to its button. Keys could not be sent to the popover from here (an accessory app does not become
  frontmost by automation on this Mac), so zoom was checked by the probe above and the Chrome
  check. The copy's fresh data folder had the default alert rules, and it fired one real
  notification ("Codex Weekly passed 90%"). Opening the app again, as Spotlight does, showed the
  popover.
- **Found, not fixed here (#64):** on macOS, VoiceOver and Full Keyboard Access press the menu bar
  icon through accessibility, and tray-icon answers only clicks, so nothing opens. The fix (the
  button's own action) is ready but needs `unsafe`, which the crate forbids; #64 asks for the
  decision. `docs/ACCESSIBILITY.md` gives Spotlight as the way in meanwhile.

### M5 queue item 8: security (D-109)

- **Supply chain:** `deny.toml` at the root and in `apps/tray/src-tauri`, and
  `apps/tray/src-tauri/.cargo/audit.toml`; a `supply-chain` CI job (cargo-deny 0.20.2 and
  cargo-audit 0.22.2 through `taiki-e/install-action`) on both workspaces, which `ci-success` needs,
  and a weekly run of CI. The core workspace has no advisory; the tray's seven exceptions (three
  `zip` 4 vulnerabilities only on Windows' zipped-update path, which QuotaDial never ships and
  which runs only after the signature checks out, and four unmaintained crates through gtk-rs and
  zbus 5 on Linux) each say why and what ends them. Without the exceptions `cargo audit` fails on
  the three, which is the control.
- **Reporting and the threat model:** `SECURITY.md` (private reports, an answer in 3 working days,
  a fix in 14 for the serious ones) and `docs/SECURITY.md` (what QuotaDial is trusted with, where
  it keeps it, what goes over the network, what each kind of attacker can and cannot do, the supply
  chain and the known limits), each claim checked against the code: files 0600 in a 0700 folder,
  `daemon.json` included; the API's token, address check and lack of cross-site headers; MCP's
  three tools and what they answer.
- **Property tests** (27 new, 21 in the workspace and 6 in the app, with `quotadial_core::testing::json()` and `line()`): core's alert
  grammar, `redact_secrets` and `outside_text`, including that no Anthropic or OpenAI key or JSON
  Web Token anywhere in a text survives redaction; the Claude provider's sign-in files, usage
  answers, profile and error bodies; the Codex provider's `auth.json` with arbitrary JWT claims,
  usage answers, session-log events and error bodies; the cost scan's Claude and Codex log lines,
  with token counts at their extremes, and the price table; the status pages; `config.json`; any
  input to the MCP server, each answer of which must be JSON; and the app's release notes, settings
  and records. They found `Tokens` summing with `+`, which overflows on a log line with impossible
  counts: a panic in a debug build, a silently wrong week's cost in a release one. It saturates
  now.
- **SBOM:** the Release workflow's publish job writes CycloneDX 1.5 JSON for the CLI package and
  the tray app with `cargo cyclonedx --target all` (265 components for the CLI) and attaches both,
  inside the checksums.

### M5: the two notes from the review of PR #61

- **The moved file stays as found** (`alerts::carry_rules`, `read_rules`): the damaged history is
  opened read-only, and the `-shm`, and an empty `-wal`, that SQLite makes beside it even to read
  are removed once the rules are read, unless they were there before. A test compares the folder's
  files, sizes and hashes before and after carrying the rules: unchanged.
- **The app holds no history open:** `AppState::history` is the daemon's own handle while the app
  runs the daemon, and is otherwise opened for each use and closed after it, so a daemon the app
  follows can move a damaged history aside, which Windows refuses while any handle is open, and
  the app's next use reads the new file. The handle opened at setup for what M3 left and what was
  told is closed before the daemon starts; a role that becomes Following drops the daemon's handle.

### M5 queue item 7: resilience (D-107, D-108)

- **The report's `problems`** (`StatusReport.problems`, `Latest::set_problem`, `Problem`): one
  line each, by kind, shown by `status`, the app (popover and tray menu), `doctor`, the API and
  MCP; the agent skill says to pass them on.
- **A damaged history** (`crates/daemon/src/history.rs`): `HistoryError::is_damaged` from SQLite's
  codes; `History::open_or_recover(path, check, now)`, used by `Host::start` under the lock, with
  `check` after an unclean stop or a noted damage; `check_pages` (`quick_check(1)`); `move_aside`
  (WAL and shared memory first) to `history-damaged-<time>[-n].db`; `alerts::carry_rules`, quiet
  at first; `Recovered::line`, also printed by `quotadial daemon`; `note_damage`, written by the
  poller and the CLI's history commands, cleared once dealt with. `user_version` is written only
  when it changes. `CostScan` deletes and remakes a damaged `costs.db`.
- **The tray** reads the daemon's own `History` (`Update::History`) while it runs one, opens its
  own lazily otherwise, and resets its told-firing record when the history's newest firing is
  below it.
- **Full disk** (`crates/daemon/src/poller.rs`): `not_saved` sets the saving problem from any
  failed history write (a reading, a firing, a status page, a prune) and notes damage; a reading
  saved clears it. `Deliverer` remembers webhook sends it could not record for the run.
- **Sleep, clock and network:** `poll_due` compares the wall clock (`WallClock`, which tests move)
  with its own; `WOKE_AFTER` (2 minutes) unseen makes everything due. `RETRY_UNREACHED` (a minute)
  after `Error::Request` or a stale answer. `undelivered` takes a last try in the future as due
  now. The known limit (a reset alert around a clock set back) is in D-108.
- **Tests:** 524 in the workspace (19 new) and 52 in the app. History: a file that is not a
  database, a damaged page found by the check with three rules carried (one still off), the WAL
  and shared memory moved with it, a second recovery in the same second, noted damage dealt with,
  busy and full errors never taken for damage. Alerts: carried rules quiet at first, nothing
  readable leaves the starting rules. Cost: a damaged `costs.db` made again. Deliveries: a send
  that cannot be recorded goes once; a clock set back does not hold up a retry. Poller: a wake
  asks everything at once; a clock set back is not a wake; an unreachable account is asked within
  a minute and back at its floor once answered; stale numbers within a minute; a history that
  cannot be saved says so until a save works; damage seen while running is noted. API: a damaged
  history moved aside at start with the line in the report; after an unclean stop the check finds
  a damaged page. Each poller and delivery test, and the unclean-stop check, fails with its fix
  taken out.

### M5: Free at once, from the review of PR #56

- **The in-flight scan** (`crates/daemon/src/poller.rs`): `Latest` keeps the costs with a
  generation, and clearing them, as a Free copy's round does every time, starts a new one under the
  same lock. A scan takes the generation when it begins and keeps what it finds only if it is still
  current, so a scan that began on Pro and ends on Free keeps nothing, with no window between the
  check and the change.
- **History:** a tier change sets `prune_at` to now beside `discover_at` and `costs_at`, so Free's
  30 days hold from the moment a key goes rather than from the next hourly prune.
- **The nits:** `licence::left_out` asks `Tier::allows` for each part rather than comparing the
  tier; `History::alerts(now, tier)` marks each rule's `waits`, which JSON shows only when true, so
  `quotadial alerts list --json` says what the text says and the CLI's text reads the same field;
  `undelivered` says why a key added sends the last hour's Free alerts to webhooks.
- **Tests:** a scan held in flight while the key is removed keeps nothing; a 60-day-old reading
  goes in the round the key goes; a running poller wakes on a nudge within seconds rather than at
  its next round a minute away; `alerts list --json` on a lapsed copy marks the waiting rule and no
  other; the daemon's Free rule test checks `waits` in JSON both ways. Each of the three poller
  tests fails with its fix taken out (the scan keeps its costs, the reading stays, the costs come
  only after the 5 s limit).

### M5 queue item 6: signing and updates (D-103 to D-106)

- **Updates** (`apps/tray/src-tauri/src/updates.rs`, `tauri-plugin-updater` 2.14): one endpoint,
  the latest full release's `latest.json` on GitHub, with `requireSignedVersion` and the public key
  in `tauri.conf.json`. `Record` (in `updates.json`: when last asked, whether it went unanswered,
  what was offered and told) decides when the daily check is due: two minutes after a start, a day
  after an answer, an hour after none, at once if the clock went back, and soon after a start when
  an offer was not installed. `Updates::checked` keeps an answer and tells each newer version once
  from the daily check; `view` writes the one sentence the window shows. `found` reads the
  updater's answer (no `latest.json` is nothing newer; a manifest without this system is "download
  it"); `problem` names the fix for each way an install fails, with `~` for the home folder.
  `install` downloads with progress, lets the updater check the signature, installs, and restarts
  through `stop_daemon` and `AfterStop` (the feed restarts the app once its daemon has stopped; on
  Windows the daemon stops before the msi runs and the tray icon is taken down first).
- **The app:** Settings' "Check for updates once a day" (`Settings.check_updates`, on for old
  settings files); Help's About shows where updates stand with Check for updates or Update, and
  What's new in the version on offer; the popover offers a newer version under the accounts, with
  a progress bar while it downloads, and a screen reader hears each step rather than each percent;
  the tray's Help menu gains Check for Updates. Help also shows what QuotaDial reads and sends.
- **Privacy** (`docs/PRIVACY.md`, `quotadial_core::privacy`): one text in Help (`help::blocks`
  turns it into paragraphs and a list) and the README (`scripts/privacy.sh`, checked by CI);
  `HOSTS` holds every destination, a core test holds the text to it, and tests in the Claude and
  Codex providers, the status pages and the app hold each address to it.
- **Signing** (`scripts/release-signing.sh`, sourced by the Release workflow's app build): each
  secret arrives as `QD_<name>` and is exported only when set, since Tauri reads a set but empty
  variable as a setting; the updater key turns on `createUpdaterArtifacts`, the Apple certificate
  the hardened runtime and the Developer ID, the Apple ID notarization, and Azure Artifact Signing
  `artifact-signing-cli` as the `signCommand`. The publish job names the macOS update for its
  version, writes `latest.json` with `scripts/updater-manifest.mjs` (which refuses a signature for
  another version, a system with no signed file, a signed file of a kind the updater does not
  install, and a signature without its file), adds `SHA256SUMS.txt`, and opens the notes with what
  is signed. The CLI job's packaging reads the target from the environment, which actionlint asked
  for. `docs/RELEASE.md` names every secret and variable, the keypair command and how to change a
  key.
- **The updater key:** made on Matt's Mac with `tauri signer generate`, in
  `~/Library/Application Support/quotadial-issue/updater.key` (0600) with a random password in his
  login Keychain; never printed. Its public half (key id `621DD8277B9290DA`) is in
  `tauri.conf.json`. Adding the two secrets and the backup are on #51.
- **Live check** (macOS 26.7.1): a test copy (`com.quotadial.updatetest`, its binary named
  `qd-update-test` so it could be driven beside Matt's running copy) at 0.2.0 on a copy of the data
  folder, updating from a local server. Building it with the public key in its config and no
  private key proved the unsigned path. No `latest.json` (404) read as "QuotaDial 0.2.0 is the
  newest version. Checked just now."; with the server stopped, "QuotaDial could not reach GitHub to
  check for updates. Check the connection, then try again.", and `updates.json` marked it
  unanswered. A manifest announcing 0.2.5 with the genuine 0.2.1 package and signature was offered,
  downloaded and refused ("QuotaDial 0.2.5 did not match QuotaDial's signature, so it was not
  installed."), as was 0.2.1 with a genuine signature of another file; the copy stayed 0.2.0 and
  kept running. With the real manifest (written by `updater-manifest.mjs` from files signed with
  the real key, which also refused 0.2.2 for those files), Update in the popover downloaded the
  9.6 MB update, swapped the app, stopped the daemon and opened again as 0.2.1 within a couple of
  seconds: a new process, a new binary, `Info.plist` at 0.2.1, its daemon on the same data copy,
  and Help saying "QuotaDial 0.2.1. Last checked for updates just now." The copy's own daily check
  had fired two minutes after it started and recorded 0.2.1 as told. Turning the setting off
  saved at once and Help said "The daily check for updates is off." Matt's running copy and data
  folder were untouched. The live check found one gap, fixed before this PR: after a failed
  install Help offered only Update, so it now always offers Check for updates too. Windows and
  Linux installs are the updater's own code, built and tested in CI on both but not yet run live;
  the first release made with the updater key is their live check, and until then they are
  experimental. Signing itself waits for the certificates on #51.
- **Tests:** 501 in the workspace (6 new: the privacy text and its hosts, and each address held to
  them), 52 in the app (13 new: when a check is due, the record, telling once, what each answer
  and each failure says, the update endpoint's config, and Help's privacy text), and 6 for the
  manifest script under `node --test`. The failing controls: an edited README fails
  `scripts/privacy.sh --check`, and a host the text stops naming fails the core test.

### M5 PR 4: the key page's function (D-102)

- **Two keys:** `quotadial-issue keygen --key-file .../issuing-key.pem` made the issuing key on
  Matt's Mac beside the root key. Core trusts `QUOTADIAL` and `ISSUER` (and the test key in a debug
  build); `quotadial-issue sign` takes either, and `public` and `check` name which it is.
- **Core:** `KeyError::OrderKey` for a pasted Lemon Squeezy key (a UUID), which says the receipt's
  link gives QuotaDial's key; the ended-licence sentence now names the renewal at
  `quotadial.com/pro` (the review of PR #55).
- **The function** (`site/`): `lib/licence.ts` signs as `crates/core` does (alphabetical JSON with
  no spaces, `QD1.` signed with it, base64url) and reads the PKCS #8 file `keygen` writes;
  `lib/lemonsqueezy.ts` posts to `/v1/licenses/validate` as a form, follows no redirect, and reads
  the documented answer, treating 5xx and a failed fetch as unreachable and an odd body as
  unreadable; `lib/key.ts` checks the configuration, the key's shape (a UUID, before any request),
  `valid`, the status (`inactive` or `active`), the store and product, and the dates, then signs;
  `functions/api/key.ts` takes `{ "licence" }` and answers JSON with `Cache-Control: no-store`.
- **Tests:** 10 under Node's test runner, with Lemon Squeezy's documented answer as the fixture
  (`site/test/fixtures/README.md`) and the test key: the documented keys byte for byte, the same
  key for the same receipt, a new `inactive` key and one that does not end, every refusal, no
  request for text that is not a receipt's key, an unset environment, and the function's headers.
  `tsc` checks the types with TypeScript 7.0.2, as the app's window does.
- **Docs:** `docs/LICENSING.md` (the two keys, the key page, its environment and rate limit, if the
  issuing key leaks, changing the root key), the changelog, AGENTS.md and D-102.

### M5 PR 3: help and feedback (D-101)

- **The text** (`crates/core/src/diagnostics.rs`): `Diagnostics` holds the facts and `text(now,
  home)` writes them: a header (version and system, licence, data folder, daemon, where the
  numbers came from), each account (status, age and source of its newest number, sign-in, error),
  then what Free leaves out, the status pages, the problems, and the settings in effect. The home
  folder becomes `~`, tokens go through `redact_secrets`, and `mask_emails` cuts each address to
  `m***@a***.com` (ASCII names with a dotted domain; a property test checks that no whole address
  is left and that nothing panics). The account's name, the home folder's last part, becomes
  `[user]` wherever it is a whole word, as in a data folder outside the home folder; the live check
  found that case. `Standing::without_id` says what the copy is without the licence's id.
- **Gathering** (`crates/api/src/diagnostics.rs`): `gather(dir, report, numbers_from, now)`, from
  the licensing, `daemon.lock` and `daemon.json`, and `config.json`; `system()` from `sw_vers`,
  `/etc/os-release` or `ver`; `feedback_url(text)`, a new GitHub issue with the text in the body,
  cut at a line past 4,000 characters; and the links, `ISSUES_URL`, `CHANGELOG_URL` and
  `SUPPORT_EMAIL`.
- **The CLI:** `quotadial doctor`, from the daemon or a direct fetch as `status`, or "nowhere"
  with the reason when a daemon does not answer.
- **The app:** a Help panel (What's new from the built-in `CHANGELOG.md`, for this version or
  Unreleased; Send feedback; Write to support; About with the version, the licence and NOTICE's
  credits) beside Settings, and a Help submenu in the tray's menu that opens it, sends feedback or
  writes to support. Links open through `tauri-plugin-opener` from Rust, and only the four Help
  names; a link chosen from the tray's menu that cannot open is told as a notification. The footer's origin line now has a line of its own above the buttons. The window's
  JavaScript is 26.4 KB.
- **The changelog:** `CHANGELOG.md` with Unreleased, 0.2.0 and 0.1.0; `scripts/release-notes.sh
  <version>`; the Release workflow drafts with the version's notes under the unsigned-builds line,
  and fails a tag with none; CI checks that the workspace's version has notes. AGENTS.md asks each
  PR for its changelog lines.

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
- **The review of PR #55:** `docs/CLI.md` now leads with `quotadial licence add` and its prompt,
  then the pipe, then the argument, which stays in the shell's history. Its other note, the
  ended-licence sentence naming where a new key comes from, waits for the key page (PR 4).
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
