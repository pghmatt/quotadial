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
  check and #52's figures from Matt's copy, merged (PR #74); the example build's figures, merged
  (PR D, #80); and the first-run recording, PR E, merged (PR #91);
- then M6. The fake provider #52 asks for is done: the manager decided on #49 (14:25 UTC)
  to build it with item 12 (D-117). The M6 plan is #73, approved by the manager at 20:29 UTC with
  its decisions (Gemini CLI detected and explained, then Antigravity through its status line, then
  Copilot; Cursor out of M6); its PRs are in the M6 notes below.
The manager's 03:30 UTC check on 2026-10-25 set the order: 5, then #64's fix, 10, 11, 12, then M6.
Items 10 and 11 merged while 5 was being built, ahead of that order (the check landed a minute after
the builder last read #49; said so on #49 at 04:45 UTC). Item 5 merged (PRs #68 and #70) and
#64's fix (PR #69); from here: 12 (#71, #72, #74, #80 and #91, the recording,
all merged),
then M6 (#73).
The manager's review of #68, posted after it merged (point 1 repeated on #70 and #49 as blocking
the 0.3.0 bump), is answered on `docs/review-fixes`, from `feat/docs`: every point, wrangler with
an undici override (notes below).

M6, the next providers: the manager approved the plan on #73 on 2026-10-25 with decisions (a) to
(d): Gemini CLI detected and explained, never read (D-118); Antigravity through its CLI's status
line only (D-119); no Cursor provider (D-120); and the order 1, 3, 2, each provider behind its own
feature (D-121). PR 1, the groundwork, merged as #75 (D-122): P-018's hook for a provider's own
commands, Google's and GitHub's tokens redacted by their shape, and the Gemini provider behind a
`gemini` feature, held off by default since the app could not show its account correctly without a
change for one provider. The manager accepted P-020 on #73 for that, one change for every provider,
as its own PR: PR 1b, merged as #77, where each provider names itself and says what QuotaDial
does with each sign-in, and which turns Gemini CLI on (D-124). The manager then decided what #77
left open (#73, 2026-10-26): PR 1c, merged as #78, has each provider say whom it asks
(P-021) and offers the week's cost only where logs could give it (D-125). The order is 1, 1b, 1c,
3, 2: PR 3, Antigravity through its CLI's status line, is in review as #79 (D-126),
with the manager's answers of 2026-10-26 (02:25 UTC) on #73 in it; then PR 2, Copilot, its merge
held on its live check (#51).

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
- P-020 (providers name themselves and say what QuotaDial does with each sign-in) is accepted by the
  manager on #73 (2026-10-25) and in the code (D-124); the PRD edit is the manager's, with P-015,
  P-018 and P-019.
- P-021 (each provider says whom it asks) is accepted by the manager on #73 (2026-10-26) and in the
  code (D-125); the PRD edit is the manager's, with P-020.
- P-022 (`Window.source` `status_line`) is accepted by the manager on #73 (rule (b)(4)) and in the
  code (D-126); the PRD edit is the manager's.
- P-023 (a window with no share) is proposed; the manager deferred it to #51's live check of
  Antigravity, where a real amount-only limit would bring it back with the capture as its reason.

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
| 12, PR D: the example's figures | Done (PR #80) | From QuotaDial Example built at 0bd845f: first run to the number in 702.8 and 370.2 ms; the popover drawn in a median of 42.1 ms; the tray app idle with the popover closed at 0.163% of one core and 22.4 MiB of footprint; the daemon alone at 0.002% and 4.0 MiB in its hour, and 0.003% and 3.7 to 6.4 MiB over ten hours; the recording split into PR E by the manager on #80; the Graphics and Media helper takes about 245 MiB while the window is drawn and gives it back within 10 seconds (#91's run) |
| 12, PR E: the first-run recording | Done (PR #91) | The README's recording of QuotaDial Example's first run, cropped to its icon and popover, under rule 4 on #49: `docs/images/first-run.gif` (376x600, 5.5 s) in the README and the same as an MP4, recorded on 2026-10-26 at 21:02 UTC from the example built at 485d0cf (#86) once Matt's menu bar stayed visible; the Graphics and Media helper's third reading says its 245 MiB is the window being drawn, given back within 10 seconds |
| 6: signing and updates | Done (PR #59) | Tauri's updater from the latest full release, checked daily while Settings' switch is on and by hand from Help, installing only on Update, with the version bound into each signature; signing for macOS (Developer ID, notarization), Windows (Azure Artifact Signing) and updates, each a workflow step that runs only when its secrets exist; `latest.json` and `SHA256SUMS.txt` on each release; `docs/RELEASE.md`; one privacy text, `docs/PRIVACY.md`, in Help and the README, with every destination held to it by tests (D-103 to D-106) |

## M6

| Step | Status | Notes |
| --- | --- | --- |
| 1: the groundwork, and Gemini CLI | Done (PR #75); Gemini held off by default | P-018: `Provider::commands`, run by the CLI as `quotadial <provider id> <name>` and listed in its help, completions and pages; Google's (`ya29.`, `1//`) and GitHub's (`ghp_`, `gho_`, `ghu_`, `ghs_`, `ghr_`, `github_pat_`) tokens redacted by shape, with every fixture held to it; `quotadial-provider-gemini`: an account when Gemini CLI is signed in, from the sign-in file's presence and one settings field, `unsupported` with one line naming `/model`, nothing read inside the sign-in and nothing sent; the `gemini` feature off until the manager settles what the app lacks; CI lints and tests the Gemini build; P-019 and D-118 to D-122 |
| 1b: providers name themselves, and Gemini CLI on | Done (PR #77) | P-020 by the manager's five rules on #73 (D-124): `Provider::name`, `Account.provider_name` and `sign_in_use`, the report's `providers`, every surface showing them as given; the panel "What QuotaDial does with this sign-in"; an older account's id shown until the next poll; #52's plain words held over every name and sentence; Gemini CLI on in the release binaries and the app, with its docs, GEMINI_CLI_HOME in the help and pages, and the release check |
| 1c: whom QuotaDial asks, and no cost offer where nothing could fill it | Done (PR #78) | P-021 (D-125): `Provider::asks()`, required, the company a provider asks or none, held by one test to `privacy::HOSTS` or to a manifest with no HTTP client; the daemon's and `watch`'s line names whom QuotaDial asks and says it asks no one about Gemini CLI; the week's cost offered only for an account of a provider that keeps logs; error lines named through the report left for a later PR, being three places |
| 3: Antigravity, through its status line | Done (PR #79) | D-126: `quotadial-provider-antigravity`, on in the release binaries and the app; `quotadial antigravity status-line` keeps each limit's share or amount and reset, the plan, the address and the time in `antigravity.json` (0600, temporary file and rename) before it runs any status line named after it, unchanged, and prints nothing of its own; an account from that file, or from Antigravity CLI's folder being there, with the block to add; windows from the status line (P-022), stale after two polls with the handover's time, a limit turned off or reset since left out, an amount-only limit named in words (P-023, deferred to #51); never a reader of Antigravity's sign-in or its app's server |
| 2: Copilot | After 3 | Built from GitHub's docs and fixtures under the manager's Copilot rules on #73; `api.github.com` in `privacy::HOSTS` and `docs/PRIVACY.md` first; its merge held on its live check (#51) |
| Cursor | Out of M6 (D-120) | Until Cursor says in writing that reading one's own usage with the editor's sign-in is allowed; asking is a #51 checkbox |
| Error lines named through the report | Done (PR #82, D-128) | `StatusReport::error_lines()`, "{the provider's own name}: {what happened and what to do}", the one line `status` and `watch`, doctor and the app's problems show; `errors[].message` leaves the provider out, which `provider` and `providers` give, and an older daemon's id is taken off so the provider shows once; "could not read the sign-in from" in place of "credentials", every `Error` held to #52's plain words by a test; the home folder written `~` in every provider's error by one pass where it is made; the OpenAPI descriptions, the CLI guide, the skill and the MCP tools say so |
| The home folder written `~` in QuotaDial's own messages | Done (PR #83, D-129) | D-128's leftovers and the manager's two notes on #82: `text::tilde_in` once where each message's text is made, for `config.json`'s warnings, every history error, the daemon's start errors, why a running daemon gave no answer, the CLI's history errors and `quotadial daemon`'s first two lines, and the app's Export CSV; doctor through it in place of its own replace, so `/Users/anna` keeps its name; `~` and then the system's own separator, `~\` on Windows, in `tilde` and `tilde_in` (the manager on #83); a property test of `tilde_in` beside `unplain_words`'s |
| `config.json`'s `chmod 600` command, working as written | Done (PR #84, D-130) | D-129's leftover: `shell_word` writes the place for a POSIX shell, `~/` unquoted and the rest in single quotes (`'\''` for a quote), or the whole place quoted outside the home folder, so `chmod 600 ~/'Library/Application Support/quotadial/config.json'` works when pasted on macOS; the warning's `tilde_in` pass leaves it as written, which a comment says and the tests hold; never shown on Windows, whose file mode check is Unix-only; a test runs the warning's own command with `sh -c` and finds the file 0600 |
| Claude's sign-in command, working as written and naming no user | Done (PR #85, D-131) | D-130's leftover, ahead of the next row by the manager's note on #84: `shell_word` moved to core's `text` module and used for both commands, so a folder in the home folder is `CLAUDE_CONFIG_DIR=~/'My Claude' claude`, where it was the whole path with the user's name, which doctor's pass then wrote `'~/My Claude'`; `text::powershell_string` for PowerShell, `$env:CLAUDE_CONFIG_DIR = "$HOME\My Claude"; claude`, a backtick before what a double-quoted string reads, and outside the home folder the single-quoted path with every single quotation mark PowerShell takes written twice; doctor's `tilde_in` pass leaves both shells' commands as written, which tests hold; a test runs the command with `sh -c` and a stand-in `claude` and finds the folder's own path |
| A place holding the home folder's path beside a space or a quote | Done (PR #87, D-133) | D-130's other leftover, done in `text::tilde_in` alone, as the manager's review of #85 asked: a folder whose name only starts as the home folder's does, such as `/Users/ann (old)` beside `/Users/ann`, is left as written in a sentence and in both commands, where it was written `~ (old)`, so `config.json`'s `chmod 600` and Claude's sign-in command failed as written; a word in single quotes is read whole as a POSIX shell and PowerShell read it, so every place outside the home folder that `shell_word` and `powershell_string` write comes through unchanged, whatever its name holds; a sentence is read by what follows the home folder: it is `~` at a separator, the end of the text or of a line, or marks and then a word that does not carry a name on (one that opens with a bracket or has a separator after a name does), and any other character joined to it is more of the name; where it must guess it leaves the name, and D-133 says what it still gets wrong; no folder QuotaDial makes is one of these, and only `QUOTADIAL_DATA_DIR`, `QUOTADIAL_FAKE_DATA_DIR`, `XDG_DATA_HOME`, `CLAUDE_CONFIG_DIR`, `CLAUDE_SECURESTORAGE_CONFIG_DIR` or an `extra_config_dirs` entry can name one |
| The popover redesigned (Matt's ask, 2026-10-26) | Done (PR #86, D-132) | Each account a card raised on the page, headed by the name given it or its provider, its address leading the line under it; plan and status as capsules; the summary and the tray's tooltip name the account whose window sets the number; commands shown as code that one click selects (the manager's note on #85); one quiet toolbar at the foot of every view; History, Settings and Help in the same cards, History's charts without plates and its period a segmented control; colours retuned for the macOS material and field hints readable; the window check's new checks of the cards, the address, the summary, the commands, the headings and the toolbar; the site's and README's pictures captured again |
| Doctor's `[user]` mask and a user named `claude` | Done (PR #93, D-134) | Doctor hid every whole word that was the user's name, a command's own words and `~/.claude` included, so for a Mac user called `claude` it wrote `CLAUDE_CONFIG_DIR=~/'.[user]-work' [user]` and `~/.[user]`, and the same for `codex`, `status`, `history`, `daemon` and any other word it writes (queued by the manager's review of #85, and put next by the review of #87). `text::hide_user_in`, beside `tilde_in`, writes `[user]` only where the name is a whole name inside a place outside the home folder (a root, `/`, `\` or a drive, read as `tilde_in` reads a path; a word in single quotes whole; a web address no place), so a command, a sentence and a place that starts at `~` keep every word for any user; where it must guess it hides, and D-134 lists where; hidden again, a text is what it was, for any name; 21 new tests and five properties among them, 18 failing first against the old rule, and 28 of 31 hand-made breaks failing a test, the other three having broken code no test could tell from its absence, or repeated another |
| A line once per card for a window's source and time, when every window agrees | Done (PR #89, D-135) | The manager's review of #86 (2026-10-26, 19:28 UTC) put it on the M6 queue, keeping each window's own line until then (#52: each window carries its own time and origin). The view model gives an account a `fetched`, such as "from the API 3 min ago", when every window was read from the same source at exactly the same moment, as the windows of one answer are, and then each window says only its reset and pace; when they differ, each says its own, as before. A card with one window says it for the card too, so the line does not jump when a limit leaves a card after its reset. The rule is `fetched_together` in `meter.rs`, not the window's script. The line is a quiet caption under the windows and above the sign-in, which opens below it, chosen from three placements shown in pictures; the window check's new `checkSources` holds it, and the site's test and pictures follow. #52 holds: the data, the API, the CLI, `doctor` and the MCP server still say it for each window |
| A property test for the window's `codeSpans` | Done (PR #90) | The manager's note 1 on #86: `codeSpans` parses text that can carry words from outside (a provider's error reaches `problems` through `outside_text`), and CLAUDE.md asks of every such parser a test that no input breaks it, while `code.test.mjs` held its shape with three cases. A fourth feeds it 5,000 random strings a run of backticks, spaces, brackets and punctuation and checks only that the spans joined give back the text, from a small seeded generator with no dependency, its seed in every failure (`QUOTADIAL_TEST_SEED=<seed>` runs the same strings again), and failing if too few strings reach a command. Note 2 on #86, checked: `.window .detail::first-letter` and `.window .pace::first-letter` capitalise nothing from outside, since every detail line begins with `text::resets`' words and every pace line with `text::pace`'s, QuotaDial's own; it stays to keep in mind if a detail line ever starts with an address or a command |
| The share picture in the cards' language | In review (PR #95, D-137) | History's Share image redrawn as the account's card on the material (the manager's decision on #73, 23:25 UTC): the ground is the picture's own choice, `#ececec` in light and `#2c2c2e` in dark (`MATERIAL` in `share.ts`, no longer whatever the page's `--background` is), with the card on it as the window draws one (its fill, its corners and padding drawn 10/3 times as large, its hairline edge in light), the title in the card's type, the charts with no plate (a faint line at 100%, a hairline at 0%, the dashed 90% line and its label, and the soft fill under the line, kept by the coordinator's call) and the heatmap's rows spaced for a week's names. The window check reads the PNG's own pixels (the corners and the margin are the material, the card's padding its fill, its edge the hairline, no plate behind a chart) and the contrast check holds the picture's text and chart line on its ground, each failing first on a picture broken on purpose; before and after pictures in both themes go in the pull request. No Rust changed |
| The toolbar stays at the window's foot while the accounts scroll | In review (PR #?, D-136) | The toolbar and the origin line, which scrolled away with the accounts once the window reached its 640 px, stay at its foot while the summary and the accounts or a panel scroll above them: layout and not overlap, so nothing is drawn under the toolbar and the material needs no backdrop, with a hairline over the toolbar only while content is hidden below it. Chosen from two layouts in pictures (the footer alone, or with the summary pinned too). Held by the manager on #73 to 200% zoom, and pinned in any window 300 px tall or more; below that the page scrolls as one, since the app zooms on to 500% and past. The app is told what the scrolling part holds beyond its edge (`neededHeight`, which the new check found a pixel short at first), each view opens at its top, and the part is a Tab stop only when nothing in it is. The window check gains a copy of nine accounts and checks of the frame at every place along the scroll, the keys, the wheel, the reading order and what the app is told, each first failed on a window broken on purpose; `--window-pictures` takes pictures of the window as it is shown |
| Queued: the daemon's footprint over a day | Decided after the 24-hour run (ends 2026-10-27, 06:53 UTC) | Still rising at ten hours in PR D's run (#80): the heap's allocated bytes from 973 KiB at 1 hour 51 minutes to 2,835 KiB at 9 hours 51 minutes, mostly SQLite's page buffers, which its default cache of about 2 MB per connection should stop; if the day's run does not level off, find what else grows, or set `cache_size` |

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

The notes below run newest first; the setup notes above them are the oldest.

### Pictures of the window, and when each is captured again

The pictures of the app's window, and what makes each one out of date. Each is captured again in the PR
that changes the window enough to change the picture, and not before; a change that leaves a picture
as it was, as D-136 left the accounts pictures, leaves the file alone.

- **The website's six pictures,** `site/public/images/app/{accounts,settings,history}-{light,dark}.png`, of which
  the README shows `accounts-light` and `accounts-dark`: `node scripts/app-pictures.mjs` in `site/`, from
  the window check answering with the website's two example accounts (`site-fixtures.mjs`), each the whole
  of the window's content as tall as it needs, at 2x. Captured again when the window's drawing of those two
  accounts changes (D-132 did; D-135 and D-136 did not change the two accounts pictures).
- **The README's first-run recording,** `docs/images/first-run.gif` (and the same as `first-run.mp4`): QuotaDial
  Example's first run, recorded through ScreenCaptureKit with Matt's menu bar set to stay visible, so it
  waits for a change worth his menu bar again (the manager's note on #91, 2026-10-26): it is captured again
  when the window next changes materially, not before. It shows the window as it was at #86, each window with
  its own source line, so #89's one line per card is not in it; D-136 would change nothing in it, since its
  two example accounts do not reach the 640 px at which a window scrolls, and no toolbar is pinned.
- **Pictures of the window as it is shown,** from `node scripts/check-window.mjs --window-pictures <folder>`
  (`--window-zoom 2` for 200%): each view no taller than 640 px, at its top and half way down what scrolls.
  Not kept in the repository: they are the before and after of a PR that changes how the window scrolls, such
  as D-136's, which `--screenshots` cannot show.

### M6: the toolbar stays at the window's foot while the accounts scroll (D-136)

- **The ask:** Matt's of 2026-10-26, "make the OS X UX more professional and make it easier to see separation between
  accounts and what the account email addresses are. make the ux something Steve Jobs would love", which #86
  answered with the cards, one heading and one toolbar, and left this in its #52 self-review: "a footer that stays
  put when many accounts scroll". The window caps at 640 px and scrolls, and with many accounts the toolbar
  (History, Settings, Help, Quit QuotaDial) and the origin line scrolled away with them. The manager's plan is on #73
  (22:28 UTC) and its approval, at 23:25 UTC, holds the PR to: the toolbar and the origin line pinned to the window's
  foot on the material with the cards scrolling above them; the footer's tab order unchanged, `usePanelFocus` and
  the `data-panel` buttons still working; the window check running the nine-account case at 100% and 200% zoom and
  failing if the footer leaves the viewport; one CHANGELOG line; a D number for the layout rule; and the two notes
  from #91 in the record. Built on `feat/pinned-footer` from `main` at 4b8757c, in the commits `cd28415`, `01fb147`
  and the records, to the coordinator's brief.
- **What it is:** the popover is a column of two parts, `.scroll`, the summary with the accounts or a panel under it,
  and `footer`. While the window is 300 px tall or more, `.scroll` is the one part that scrolls and the footer is
  never in it; `.scroll` ends where the footer begins, so nothing is drawn under the toolbar and the material, a
  transparent page on macOS, needs no backdrop. A hairline over the toolbar (an inset shadow in `--rule`, half a CSS
  pixel, a whole one at 1x) shows only while content is hidden below, fades in over 150 ms and is cut by the
  reduced-motion rule; it is a decorative separator, so it is not held to 3:1 and no colour pair is new.
- **Two layouts, in pictures:** with nine accounts, light and dark, at the top and half way down, the footer alone
  against the footer with the summary pinned too, which is Wi-Fi's and Bluetooth's frame. The coordinator chose the
  footer alone, on three reasons the D records. By the check's browser, at its narrowed width: the toolbar and origin
  line are 51 px at 100% and 110%, 75 at 125% and 150%, 90 at 175% and 114 at 200%; the summary, with the longest
  headline and one incident, 109 px at 100% and 142, 160 and 194 at 125%, 150% and 200%, and unbounded; the accounts
  keep 587 px against 478 at 100%, 435 against 292 at 125% and 350 against about 190 at 150%. The pictures are kept
  outside the repository.
- **200% and past it:** my first threshold was 400 px, from a rule that the footer take under a quarter of the window, and
  it dropped 200% (320 px) to the old scrolling page. The manager's hold put 200% inside it. The app is not limited to
  200%: Tauri's zoom keys (`webview/scripts/zoom-hotkey.js` in tauri 2.12.5, off on Windows) step 20% from 20% to
  1000% on macOS and Linux, and WebView2 has its own, to 500%. So the pinning stops where the footer takes the window,
  and the check's widths show where that is: at 200%, 114 px of 320 (36%), leaving 204 px (228 at a Mac's width); at
  220%, the next step of the keys, 129 px of 291 (44%), leaving 160; at 250%, Windows' next step, 169 px of 256 (66%),
  leaving 85; at 300%, 215 px of 213, leaving the part's own 26 px; at 400% the footer is taller than the window. The
  line is a 300 px window (213% zoom), which leaves 169 px at the check's width and 20 px of margin under 200%; the
  check holds 120 px as the least a pinned window leaves, below that so that a wider font on Windows or Linux still meets it.
- **The window's height:** with the window capped, the popover's own height stops growing with its content, so `fit`,
  which was told that height, would never grow the window again. It is now told what `.scroll` holds beyond its
  edge as well (`neededHeight` in `src/scroll.ts`). The first version added `scrollHeight` minus `clientHeight`,
  which round to a pixel, and the check's first run told it 2,795 px for a window that holds 2,796, which would
  leave the window a pixel short with a hairline up at rest; it now measures the content's box and the part's
  padding to a fraction of a pixel, with the content a formatting context of its own so that its last card's margin
  is in it. The Rust side, `fit` and `fitted_height` are unchanged.
- **Keyboard, focus and VoiceOver:** the markup's order is the page's order, so Tab and a screen reader meet the
  summary, the accounts or the panel, the toolbar and the origin line as before; `usePanelFocus` and the
  `data-panel` buttons are as they were, and the check's panel-takes-the-focus and Done-gives-it-back checks, which
  run over Pro and Free in the light theme, held through every run. With the focus in the scrolling part the arrows,
  Page Up and Page Down, Home and End scroll it, and Space where the focus is on nothing Space presses; with nothing
  focused, as in a window just opened, none of them scrolled anything, since the page no longer scrolls (Chrome showed
  it, until a click or a Tab), so the window's key handler gives the part the focus on the first of them
  (`tabindex="-1"`, no ring, not a Tab stop) and Tab then reaches the first control as before; Tab scrolls a
  control into view with room for its ring (`scroll-padding: 8px 0`), found necessary by a broken-on-purpose run. Each view opens
  at its top (a layout effect on the panel), which Done needs, since the accounts would come back where the panel left
  them. The part is a Tab stop of its own, a region named for its view, only when it scrolls and holds nothing Tab
  reaches, as Chrome does from 130 and WebKit does not. With less motion asked for, scrolling is not animated.
- **The window check:** a third copy, `many`, Pro with nine accounts (`forScrolling`, so it is not pressed control
  by control or made to draw nine pictures to share, which Pro does). In every view and both themes: the toolbar at
  five places along the scroll, at 100% and at 200% (the buttons and the origin line inside the window, a click at a button's middle reaching it, the
  toolbar not moving, with the scrolling part ending where it begins, the page not scrolling, the line exactly while content is
  hidden, 120 px left, a pixel short of the end drawing no line, no animated scrolling), and Tab bringing each control
  into view with its ring. In every copy, in the light theme: the order Tab and the accessibility tree meet, and what
  `fit` is told, with 123 px put in and taken out. On `many`, in the light theme: the keys, with nothing focused too, the
  wheel over the accounts and over the toolbar, each view at its top, and the part as a Tab stop with nothing else to reach; and, for
  the accounts, 125%, 150%, 210%, the 300 px window (213%), 220% and 250%, and windows of 299, 300 and 4,000 px. The
  cut-off check no longer counts text beyond the end of a box that scrolls, and still sees it cut off by one that
  does not. The structure the old checks address moved from `main > section` to `main .content > section`.
  `--screenshots` frames the whole of the content from the page's own height; `--window-pictures` is new (see the list
  of pictures above). The check's run is 164 s over 24 views, from 113 s over 16.
- **Broken on purpose,** each new check failing first, in twenty-four ways: the page scrolling as one at every height
  (the toolbar not inside the window, not reached by a click, the page scrolling, no part scrolling); pinned in every
  window (a toolbar kept in a window under 300 px); pinned only from 400 px (the manager's hold: at 200% the accounts do
  not scroll above the toolbar); the scrolling part 40 px under the toolbar, with and without being drawn over it (a
  click would not reach the buttons; 40.0 px under the toolbar); the line always drawn; the line never drawn; the line for a
  pixel of rounding, which the check first did not see and now does by scrolling to a pixel short of the end; the app
  told the capped height (640 px for 2,796, and not told when 123 px is put in); the app told `scrollHeight`'s rounded
  height (2,795 for 2,796); the toolbar before the accounts in the markup (Tab reaches the toolbar first, a screen
  reader reads the last account after it); the part hiding what overflows (the keys and the wheel scroll nothing); scrolling
  animated with less motion asked for; no room for a focus ring (the ring cut off at the edge); no Tab stop of its own when
  nothing else is one (the first try did not compile, then five checks failed); a new view keeping the last one's
  scroll (Done shows the accounts 1,985 px down); the key handler no longer giving the part the focus, so that no key scrolls a window just opened; the app not told when what it
  holds grows; a box that clips text and does not
  scroll (the relaxed cut-off check still sees it); and the window as it was before this PR, where the check first threw on
  the missing content part and now reports it, with 175 lines of failures; the page scrolling under the wheel; the footer hanging 40 px below the window, which cuts off
  the origin line under the toolbar; and, for the manager's hold on `usePanelFocus` and the `data-panel` buttons, which the check's older
  checks already hold, a panel not taking the focus when it opens ("opening history left the focus on the page") and Done not
  giving it back ("Done gave the focus to the page, not the history button").
- **Pictures:** `--screenshots` shows the window as tall as what it holds, so the website's accounts pictures, light
  and dark, come out byte for byte the same as committed (the two-account pictures should not change, and they do not).
  The history and settings pictures differ from the committed ones by 210 and 316 and by 12 antialiased pixels, at a card's
  corner and a chart's hairline, with their sizes unchanged: the sub-pixel snapping of a scroll box. They were not
  captured again, as the coordinator decided; a regeneration with `node scripts/app-pictures.mjs` would write those pixels.
  The README's first-run recording shows two example accounts that do not reach 640 px, so it is unchanged, and it is
  on the list above for the window's next material change. The first geometry comparison, byte by byte against the
  baseline, found a real slip: the footer's text 12 px too far left, since the popover's side padding had gone.
- **Counts:** in the workspace, 780 passed (1 ignored) and in the example build 777 (2 ignored), both as on `main` at 4b8757c, since no crate changed; the app's `--lib` tests 99 in both builds, as on `main`; the window's `npm test` 9, as on `main`; the window check 11,166 checks over 24 views in 164 s, up from 6,721 over 16 views in 113 s; the contrast check 21 pairs a theme, as before; the window's JavaScript 37.2 KB, up from 35.5 KB, within its 150 KB; the site's tests 90; the release scripts' tests 40, none failing.
- **Checks run here:** one run after another with the data folders in the scratchpad, on a warm build: `cargo fmt --check` in both workspaces; clippy with `-D warnings` and `--locked` on the workspace and the tray app, default and `fake`; the workspace's tests, default and `fake`; the tray app's `--lib` tests, default and `fake`, the latter with the example's CLI bundled (`npm run cli -- --fake`, then `npm run cli` again), and the default once more, since they read `CHANGELOG.md`; `scripts/privacy.sh --check` and `scripts/public-prd.sh --check`; the release scripts' tests; the tray app's `npm run build` (types, size, contrast), `npm test` and `npm run check:window`; and the site's `npm run check`, `npm test` and `npm run build`, which the changelog line makes necessary, the site rendering it. All passed. The Rust is as on `main`, which nothing here changed.
- **Not run:** a Windows or Linux build, which no target here makes (CI's); WebKit, which draws the app on macOS and
  Linux (the window check drives Chrome, which is WebView2's engine on Windows), so macOS's overlay scroll bars,
  its elastic scrolling and how WebKit scrolls by key with nothing focused are unseen; VoiceOver by hand, and the
  transparent page over the real material (the pictures are the painted page, and the check holds that nothing is drawn
  under the toolbar by geometry); the app itself, never run, as briefed, so the popover's draw time and the Graphics and Media
  helper's figure with one more scroll box are unmeasured; `tests/icon_press.rs`; `cargo deny`, `cargo audit` and
  actionlint, not installed here, with no dependency, lockfile or workflow changed.
- **Safety on this Mac:** every build, test and npm run had `QUOTADIAL_DATA_DIR`, `QUOTADIAL_FAKE_DATA_DIR`,
  `CLAUDE_CONFIG_DIR`, `CODEX_HOME` and `GEMINI_CLI_HOME` in the scratchpad and no token in its environment; no vendor
  CLI ran, no sign-in or keychain item was read, and nothing touched `~/Library/Application Support/quotadial`.
- **#52's list, what it found:** *never waits or guesses:* every number still shows its age and source, and the
  summary scrolling away loses none; *fast and light:* the window's JavaScript is 37.2 KB of its 150 KB (35.5 KB
  before), the listeners are events and observers with no polling, the popover's draw time and the helper's memory are
  unmeasured here; *usable by everyone:* keyboard, screen-reader order, reduced motion and 200% are checked, no
  colour pair is new, nothing is cut off at 200%, and nothing shifts as numbers change, since the toolbar never moves;
  *trustworthy:* no dependency, no network address and no change to what is read or sent; *installable:* the
  CHANGELOG line, and the two #91 notes here; *the code:* module docs on `src/scroll.ts`, one concern per commit, the
  record in D-136. What it left for a later PR: how WebKit and VoiceOver take the key handler that gives the part the
  focus on the first scrolling key, which only Chrome has shown here; at 200% a percentage can break over two lines, "64" above "%", where a long label takes the room (seen in the 200% picture; the row's CSS and `overflow-wrap: anywhere` are as before this PR); past about 213% zoom the toolbar scrolls away with the page again, which
  the D weighs; the history and settings pictures' antialiasing; and measuring the popover's draw time and the helper's
  memory on QuotaDial Example at the next real run.

### M6: the share picture in the cards' language (D-137)

- **The ask:** the manager's decision on #73 (2026-10-26, 23:25 UTC) on the builder's second pull request
  of that day, and the coordinator's brief of the same day: redraw History's Share image in the window's
  card language (#86, D-132, D-135), on a background that is chosen and named, not whatever the page
  paints; the strings `shareStrings` lists, the size, the label's no-address rule and Rust's checks on
  the PNG unchanged; before and after pictures in both themes; and the window check holding the new
  ground and no plate behind a chart, each failing first on a picture broken on purpose. Built on
  `feat/share-picture-cards` from `main` at 705286f (#93), in five commits: the check's copies gain an
  account with one window (0336979); the stylesheet reader becomes one script (cb12d47); the redraw with
  its drawing test, contrast pairs and ground verdict (294ae25); the pixel reader and its verdicts
  (a2f4c58); and these records. In review as #95.
- **The ground (D-137):** `#ececec` in light and `#2c2c2e` in dark, `MATERIAL` in `share.ts`, chosen for
  the picture and read from nowhere. #86 had made the page's `--background` a grey like the material,
  and the picture had taken it for its ground unchosen; the two are the same values today and are two
  decisions now. The theme is `prefers-color-scheme`, the stylesheet's own test. The contrast check takes
  the ground from `share.ts`; the window check keeps its own text of the decision in `share-picture.mjs`.
- **What the picture is now** (`apps/tray/src/share.ts`, in CSS pixels of the 1200 by 675 picture, drawn
  twice over): the account's card, 1120 by 595 at 40, 40, 33 px corners, 40 px of padding, filled with
  `--card` over the ground, its 1.5 px hairline just inside the outline (`--card-edge`, clear in dark).
  The heading: the label at 47 px semibold (the baseline at 118), the period and its dates under it at
  25 px in `--secondary` (154), QuotaDial and quotadial.com at 24 and 20 px at the card's right on those
  baselines. Each window: its name at 25 px semibold (204), its latest share at up to 84 px semibold (282),
  its highest at 21 px (311, said only where it differs), its chart 58 px tall from 326, with a faint line
  at 100%, a hairline at 0%, the dashed line at 90% and its label in the 48 px kept at the chart's right,
  the line 3 px in `--bar-normal` and the soft fill under it at 16%. The heatmap: its words at 430, its
  rows from 445, 18.9 px apart over a week, its hours at 598, 37 px above the card's foot. `chartBoxes`
  and `CARD` are exported, so the window check looks where the drawing puts things.
- **The first pictures found a flaw, and the check said so before I saw it:** with the card's padding the
  week's rows were 15.3 px apart, and the check's contrast reading, which looks 15 px above each word,
  found a day's name touching the one above it ("6" at 2.88:1 in light, "Mon " at 3.33:1 in dark). The
  vertical rhythm was re-balanced (the heading and the windows 6 to 10 px higher, the chart 6 px shorter,
  the heatmap's words 18 px higher and its foot 6 px lower) so that the rows are 18.9 px apart, and each
  day's name is level with its row and held inside the grid, so a month's thin rows no longer push the
  last name down towards the hours.
- **Calls where the brief left room:**
  - *No plan capsule.* The brief asked for it "if the label does not already say it". `AccountHistory`
    carries no plan, and the label says it whenever no name is given; drawing it beside a name would add a
    field to the view and a string to `shareStrings`, which the manager's decision keeps. Left for a later
    pull request if it is wanted.
  - *The mark stays inside the card,* at its top right on the title's and the period's baselines, as it
    was at the top right of the picture, not on the material under the card; the card is the account, and
    nothing is louder than the numbers.
  - *Semibold, not bold,* for the title and the figures: the window's weights (`.percent` is 600) and the
    card title the brief names. The period takes `--secondary`, the colour of the window's address line;
    the picture's allowed text colours are now five, and the contrast check holds each.
  - *The soft fill under each line stays* (the coordinator's call on the first pictures): History's window
    draws the line alone, but a picture seen small in a feed needs the weight. D-137 names it as the
    picture's own choice, with that reason. The faint line at 100% and the dashed line at 90% sit close at
    each chart's top, as they do in History; the 30-day pictures do not make them crowded, so they stay.
  - *The scale is 10/3:* the window's 336 px card in its 360 px popover, 12 px in from each side, drawn at
    1120 px and 40 px in from the picture's sides. The 10 px corners are 33 px (33.3), and the 0.5 px
    hairline is 1.5 px, three device pixels, so it lands on whole pixels and shows when the picture is
    small.
  - *A card with no margin is a verdict, not a crash:* the first broken picture in Chrome, a card out to
    the picture's edges, stopped the check with a range error, since the margin it reads was outside the
    picture. A place off the picture is now read from the nearest pixel on it, and a card closer than 8 px
    to an edge says so, naming the edge; a test holds each edge in turn.
  - *One more account in the check's copies,* a Codex Pro with a single weekly window, so that the picture
    of one column (the widest chart, the largest figure) is checked in both themes; it is in the Free
    copy too, which lists every account. The check passes with it on the unchanged picture.
  - *The before pictures come from `main` itself:* `git archive origin/main apps/tray` into the
    scratchpad (not a worktree), its dependencies linked, built and run there with
    `check-window.mjs --share-images`, with the new fixtures file laid over its own (data only) so that
    both pictures show the same accounts. The website's accounts (`--fixtures ./scripts/site-fixtures.mjs
    --only ", history"`) give Claude Work, the README's; the check's own copy gives the account with one
    window. All 2400 by 1350, kept outside the repository and attached to the pull request.
  - *`scripts/stylesheet.mjs`* reads the stylesheet's two themes for `check-contrast.mjs`, which did it
    itself, and for the drawing test, so that the reading is written once.
- **Proof:**
  - The window check now reads the PNG (`scripts/share-picture.mjs`): the four corners and the middle of
    each margin are the material exactly; the card's padding is its fill laid over it; its outline is the
    hairline; the empty band above each chart's dashed line is the card's fill, at up to six places where
    its line is not; and the first thing drawn is the material over the whole picture.
  - *Each failed first on a picture broken on purpose, in Chrome* (`--only "pro, light, history"`, then
    dark where it differs): a ground one step off (`#ededed`: "has #ededed at its top left corner, not the
    material #ececec", 18 of 2,704 checks failed); a ground never painted (44, "has #00000000 at its top
    left corner"); a ground in the card's fill (44); no card (26, "has #ececec in the card's left padding,
    not the card's fill #f9f9f9"); a card out to the picture's edges (6, "has the card 0 px from its left
    edge, with no margin for the material to show in", after the crash above); a card of the wrong fill
    (26); no hairline (6, "has #fdfdfd on the card's left edge, not its hairline #e8e8e8"); a plate behind
    each chart (14, "draws a plate behind Weekly's chart: #e2e2e2 4% along its empty band above 90%,
    where only the card's fill #f9f9f9 should be"); a plate over a chart's left half (6); and a text
    colour the stylesheet does not have (50). In dark: the ground one step off (`#2d2c2e`, 18), no card
    (26), a card out to the edges (6), a wrong fill (202) and a plate (14). The contrast check fails the
    picture's pairs when its ground is chosen badly: with `#000000` for light, five pairs, `--secondary`
    4.31:1 to the chart line 1.91:1; with `#808080`, `--high` and the line.
  - `share-picture.test.mjs`, 22 tests with no browser: the reader gives back pixels made with each of the
    five filters, RGB and RGBA, a PNG cut short anywhere or changed at any byte is refused, and each
    verdict passes a picture drawn the way the window draws one in both themes and fails the one broken its
    way, and only that one: every place a verdict reads is marked in turn and found by name. 32 hand-made
    breaks of `share-picture.mjs` each failed a test; the first round of 18 left five standing, and the
    tests were made to reach them (a Paeth predictor that never chose the pixel above and to the left, a
    corner or padding spot not read, a chart's line never drawn, a ground of the right colour with no
    opacity, a plate over only part of a chart).
  - `share.test.mjs`, 14 tests on a canvas that records its calls: the material first, then the card, no
    fill that could be a plate, the scale of each chart, the heading's size, weight and colour, the figures'
    colours, five text colours and no others, nothing outside the card's padding or over anything else
    (against a fixed width a character, and against the charts and the heatmap's squares), a long label
    cut clear of the mark, no `@`, and the eleven properties `lookOf` asks the real stylesheet for, never
    `--background`. 50 hand-made breaks of `share.ts` each failed a test; the first run left five
    standing, and the tests were made to reach them (the hairline's width, which share got which colour, a
    chart over writing, a heatmap past the padding, and the label's room next to the wider of the mark's
    two lines).
- **Counts:** the tray's `npm test` 45, up from 9 (22 and 14 new); the window check 7,781 checks over 16
  views, up from 6,721 (984 for the new account on the picture as it was, 76 for the new checks: four
  verdicts, and one more for each chart, in each of 12 pictures), and the website's accounts 2,858, up
  from 2,818; the
  contrast check 27 pairs a theme, up from 21; the window's JavaScript 36.4 KB, up from 35.5, of its 150.
  No Rust changed, so the Rust counts are `main`'s own: the workspace 801 passed (1 ignored) and the
  example build 798 (2 ignored), the app's `--lib` tests 99 in both builds.
- **#52's list, gone down:** *found and fixed:* the crowded day names the contrast check caught on the
  first pictures (above), and the check that stopped instead of failing on a card with no margin. *Held:*
  every colour pair in both themes, the picture's among them (27); tabular figures; no address anywhere
  (`shareStrings`, and a test that no drawn text of the made-up accounts holds an `@`); the privacy text,
  "drawn on your machine and saved in your Downloads folder, and QuotaDial sends it nowhere", still true;
  no new dependency, the PNG reader being a script on Node's `zlib`; the window's JavaScript within its
  budget. *Left for later:* an account with readings and no use in the period shows its sentence ("No use
  in session readings in this period.") over an empty foot of the card, as D-115's picture left that room
  on the ground; the dashed 90% line's colour is 1.96:1 on the light card (5.64:1 in dark), as it is in the
  window's History since before this, and the "90%" beside it says in text what it marks; the heatmap's
  caption for a window named in brackets reads "(weekly (gemini))", D-115's words; the saved PNG has no
  text alternative, as before; and the M6 row for #93 still says "In review" though #93 is on `main`.
- **Checks run here:** `cargo fmt --check` in the workspace and the app; clippy with `-D warnings`
  and `--locked` on the workspace and the app, default and `fake`; the workspace tests, default and
  `fake`; the app's `--lib` tests, default and `fake`; `scripts/privacy.sh --check` and
  `scripts/public-prd.sh --check`, all passing, run after the changelog line was in, since the app's
  tests read it. The tray's `npm run build` (types, size, contrast), `npm test` and `npm run check:window`
  on the final tree, and each of those on a clean export of the third commit's tree, which passes alone
  (23 tests, 7,705 window checks). Before the go-ahead I ran only Node: the two new test files and
  `check-contrast.mjs`, and the hand-made breaks.
- **Not run:** Windows and Linux builds (CI's), `cargo deny`, `cargo audit` and actionlint (not on the
  brief's list, and nothing they cover changed), `tests/icon_press.rs` (as briefed), the app itself, and
  VoiceOver by hand.
- **Safety on this Mac:** every cargo and npm run and every run in Chrome went through the
  scratchpad's `safe.sh`: `QUOTADIAL_DATA_DIR`, `QUOTADIAL_FAKE_DATA_DIR`, `CLAUDE_CONFIG_DIR`,
  `CODEX_HOME` and `GEMINI_CLI_HOME` in the scratchpad, no token in the environment, four build jobs,
  one heavy job at a time; the Node-only unit tests and hand-made breaks, which read and write only the
  repository and the scratchpad, were run directly. No provider's CLI ran, no sign-in or keychain item was read, and nothing touched
  `~/Library/Application Support/quotadial`, the example's folders or the owner's Downloads folder; the
  window check's stand-in saved the pictures in the scratchpad. The one other command line was `gh`,
  reading #73's comment, #52 and the open pull requests.

### M6: doctor hides the user's name only where it names the user (D-134)

- **The ask:** the coordinator's brief of 2026-10-26: M6's queued row
  "Doctor's `[user]` mask and a user named `claude`", left queued by the manager's review of #85
  (2026-10-26, 13:40 UTC, "doctor's masking is its own rule and the case is rare") and put next by
  the review of #87 (20:22 UTC). Doctor's text hides the user's name wherever it is a whole word, so
  for a Mac user called `claude` it broke Claude's sign-in command and wrote `~/.[user]` for
  `~/.claude`. Hide the name where it names the user and nowhere else, with tests and a property
  that fail first, and record it as D-134. Built on `fix/doctor-user-mask` from `main` at 82038b0
  (#86), moved onto 72bc175 (#87) before the first commit, as the coordinator asked, and onto
  f786132 (#91) since. In review as #93.
- **The bug, as the old rule wrote it** (what the new tests printed against it, with the home
  folder `/Users/claude` unless said): `CLAUDE_CONFIG_DIR=~/'.[user]-work' [user]` for Claude's
  sign-in command, `~/.[user]/.credentials.json` for the sign-in's place and "run [user] once" for
  "run claude once"; for a user called `status`, the daemon's address `http://127.0.0.1:47115/v1/[user]`
  and `[user]_pages: on`; and the properties' shrunk inputs: `~/'[user]'` for a user called `a-0`
  whose home folder holds a folder of the same name, and `[[user]]` for a user called `user`, who
  was hidden again each time the text was. `claudeclaude`, a name right after a name the rule had
  kept, became `claude[user]`, since it read the character before a name from the text it had left
  to scan.
- **Where the fix is:** `crates/core/src/text.rs`: `hide_user_in(text, home)` beside `tilde_in`,
  public like it, and its private helpers (`user_name`, `is_whole_name`, `outside_the_home_folder`,
  `starts_at_a_root`, `continues_a_path`, `is_slash`, `is_web_address`, `home_string`,
  `place_goes_on`); `crates/core/src/diagnostics.rs` calls it where it called `without_word`,
  which is gone. No provider, no command writer and no other caller changed, and no file system is
  read. Only doctor and Send feedback hide the name, as before (D-129's "what can still show a
  name" stands for every other surface).
- **The audit the brief asked for** (D-134 has it in full): what `Diagnostics::text` writes holds
  the user's name only in a place: the data folder, "Numbers: from nowhere: ...", each account's
  sign-in sentence, errors and problems, and the settings (`extra_config_dirs` as typed, the
  warnings). A place in the home folder is `~` (or `$HOME` in PowerShell's command) by `tilde_in`
  and the commands, so only a place outside it can hold the name. The version, the system, the
  licence, the daemon's line, account names (emails cut to their first letters), status pages and the
  Keychain's account (passed to `security`, never shown) hold none.
- **The rule, in short** (D-134 has it in full): the name is written `[user]` where it is a whole
  name inside a place outside the home folder. A place starts at a root (`/`, `\`, `C:\`, `C:/`) where
  a path can start, goes on to the end of its word and past a space into a next word that holds a
  separator, and ends at a line break or a plain word; a word in single quotes is read whole, as
  `tilde_in` reads one; PowerShell's string from `$HOME` is read whole as `powershell_string` writes
  it; a word that starts with `~`, `$` or `%` is the home folder's (`%USERPROFILE%\x` and `${HOME}/x`
  in one piece); a relative path is no place; a `//` after `scheme:` is a web address, except after
  `file:`. Each name and each `[user]` already there is read as one character, so hiding a name
  cannot change what is read after it (a name with a space, such as `Ann Smith`, included), and a
  name that touches a `[user]` is left: hidden again, a text is what it was hidden as.
- **How it was built:** a prototype of the rule in Python, in the session's scratchpad and not
  committed, ported `tilde_in`'s readers and ran the rule over random text before any Rust was written.
  It found, before the Rust did, that PowerShell's string from `$HOME` must be read whole (a folder
  whose name ends in a space before a separator made the next word look like a place), that
  `claudeclaude` needs the character before a name read from the whole text, and, from the test for
  the `%` anchor, that `%USERPROFILE%/x` was read as a variable and then a place of its own.
- **Calls the brief left to me, and why** (D-134 weighs the alternatives and says what each costs):
  - *Names in places only,* not commands marked where written, not a list of the words doctor
    writes, not the whole-word pass with `~` places and backticked commands kept out of it: the
    brief's own list of users (`codex`, `gemini`, `daemon`, `history`, `status`) are words in
    sentences, settings and an address, which only a rule about places leaves alone.
  - *Where the rule guesses, it hides,* as the brief said: a component only spelled as the name
    (`/usr/bin/security` for `security`, Claude Code's own `claude-501`, a backup's `.claude`), a
    word that reads as a place (`/model`, a bare `/v1/status`), a word with a separator after a
    space (`docs/CLI.md` after a place), and a folder in the home folder that ends with a space
    before a word that starts like a place. D-134 lists them and I pinned none by a test, since a
    test would hold a wrong answer in place (as D-133 did for its own).
  - *A web address is no place,* except `file:`: `status` is on the brief's own list, and doctor
    writes "its API at http://127.0.0.1:47115/v1/status could not be reached" when the daemon does
    not answer.
  - *A place goes on past a space only into a word with a separator,* not into a plain word: the
    macOS volume `/Volumes/My Passport/Users/ann` is read whole, `/Volumes/Time Machine Backups/Users/ann`
    is not (the place ends at `Machine`), and a sentence after a place is never part of it.
  - *Both separators on every system,* so a place written on another system is found, and the tests
    run the same on this Mac as on Windows.
  - *PowerShell's `"$HOME..."` string is read whole,* which fuzzing the rule found it needs: a folder
    whose name ends in a space before a separator made the next word look like a place.
  - *`%` and `}` go on into a path,* so `%USERPROFILE%\x` and `${HOME}/x` are the home folder's in
    one piece: found when the example for the `%` anchor failed in the prototype.
  - *Names are read as one character,* so that a name with a space cannot change what is read after
    it, which makes the text idempotent for any name, `user` and `Ann Smith` included.
  - *Case and form are as before:* matching is by the characters, so `/users/ANN` and a decomposed
    accent show the name, as they did.
  - *Public, in `text`, beside `tilde_in`:* the one rule for the user's name next to the one rule
    for the home folder (the manager on #82), usable by another surface later; the brief put the
    property beside `tilde_in`'s.
  - *The tracker:* the existing queued row is flipped to "In review (PR #93, D-134)" as the
    coordinator said, and the rows of #87, #90 and #91, all on `main` now, to Done, as each PR has
    done for the one before; the number comes when the coordinator gives it. `docs/CLI.md`'s Doctor paragraph is not changed (below).
- **What it still gets wrong** (D-134 has it in full): the guesses above, which hide a word; and
  what it leaves, since the text cannot tell: a name after a plain word in a folder's name, in a
  relative path (the `extra_config_dirs` entry shown as typed in "... is not a full path"), in a
  sentence, in another case or form, and in the user's folder encoded under `~`.
- **Tests,** each new one failing first (below): 21 new tests.
  - core, `text.rs`, thirteen tests: `a_users_name_is_hidden_in_a_place_outside_the_home_folder`
    (another disk's copy of the home folder, Claude Code's temporary folder, a drive, a share and a
    folder named after the name, for users called `ann`, `claude` and `codex`, in five sentences
    each, the name kept in the prose around the place);
    `a_word_that_is_only_spelled_as_the_users_name_stays_as_it_is`;
    `a_place_in_the_home_folder_keeps_the_words_it_has_whoever_the_user_is` (`claude`, `codex`,
    `gemini`, `quotadial`, `Support`); `a_command_for_a_place_in_the_home_folder_comes_through_whole_for_any_user`
    (both shells, `chmod 600` and the warning's own words, five users, seven folders, one with a
    space before a separator); `a_web_address_is_not_a_place` (`status`, `headroom`, `anthropic`,
    `api`, `wham`; `file:` and `D://` are places);
    `the_words_around_a_quoted_place_outside_the_home_folder_stay` (three folders, and PowerShell's
    quotes on a drive); `a_quoted_word_that_goes_on_from_a_path_is_part_of_that_path`;
    `a_place_goes_on_past_a_space_only_into_a_word_with_a_separator`;
    `a_place_outside_the_home_folder_ends_where_one_in_it_starts_in_the_same_word` (`:`, `=`,
    `$HOME`, `%USERPROFILE%`, `${HOME}`); `a_name_with_a_space_in_it_is_hidden_as_one_name`;
    `a_name_that_is_not_whole_does_not_hide_a_whole_one_that_overlaps_it`;
    `a_name_that_is_a_word_of_the_marker_is_hidden_once`; and
    `nothing_is_hidden_without_a_name_of_three_characters_in_the_text`.
  - core, five properties beside `tilde_in`'s, with the brief's three:
    `no_text_or_home_folder_makes_hiding_the_user_panic` (any text, any home folder or none),
    `hiding_the_user_again_changes_nothing` (any name, `user` and `Ann Smith` among them, over
    every shape of text around it), `hiding_the_user_hides_nothing_but_the_name`,
    `the_users_name_never_survives_in_a_place_outside_the_home_folder` (every root, separator,
    shape of the name (`ann`, `ann-OLD`, `-Users-ann--WORK`, `OLD.ann`) and form of sentence, with
    the name kept in the sentence around the place; every expected place made from its parts) and
    `the_commands_for_a_place_in_the_home_folder_come_through_whole_for_any_user` (any name, any
    folder, in a POSIX shell and PowerShell).
  - doctor, `diagnostics.rs`, two:
    `a_user_called_as_a_word_in_the_text_sees_every_command_and_place_as_written` (the text for
    each of nine users called by a word doctor writes is the text any other user sees, built
    from `shell_word`, `powershell_string`, `sign_in_read_from` and places in the home folder) and
    `the_users_name_is_hidden_in_a_place_outside_the_home_folder_whoever_the_user_is`.
  - Claude's provider, `advice.rs`, one:
    `doctor_shows_the_command_and_places_as_written_for_a_user_called_claude_or_codex`, with the
    real `claude_in`, `signed_out` and the account's sign-in sentence, both shells, four folders,
    and a place outside the home folder with the name hidden in it and the command's own words
    around it kept.
  - Changed: `the_account_s_name_is_hidden_outside_the_home_folder_too` lost its two assertions about
    `without_word`, which is gone: "someones [user]_x xsomeone" is in
    `a_word_that_is_only_spelled_as_the_users_name_stays_as_it_is` and "too short to be sure" in
    `nothing_is_hidden_without_a_name_of_three_characters_in_the_text`.
- **First failures:** against the old rule put behind the new signature (a stub, not committed), 18 of the
  21 failed, with the outputs in "The bug, as the old rule wrote it". The three that passed pin what
  both rules do, and each fails under a break of the new one (a name of any length; a hidden name
  taking the character after it; the scan running a byte past the text):
  `nothing_is_hidden_without_a_name_of_three_characters_in_the_text`,
  `no_text_or_home_folder_makes_hiding_the_user_panic` and `hiding_the_user_hides_nothing_but_the_name`.
  An earlier set, run first, passed its "the name is still hidden" tests against the old rule too,
  since that hid the name everywhere; each of them now also holds the name left in the sentence
  around the place, which the old rule hid, so each fails first.
- **Mutations:** thirty-one hand-made breaks, each applied alone to the finished file and run against
  core's `text::` and `diagnostics::` tests (a harness in the session's scratchpad, not committed,
  which copied the files back from copies after each): no guard for a web address's `//`; `file:`
  a web address too; a word at `~` not ending a place; no continuation past a space; a place going on
  into any word; a line break not ending a place; `\` not a separator; a drive not a root; the
  character before a name, and the one after it, not looked at; a name that touches a `[user]`
  taken for a name; a `[user]` not read as one character; names not read as one character; a word
  in single quotes not read whole; a quoted word that goes on from a path taken for a place of its
  own; any quoted word a place; `$HOME`'s string not read whole, leaving a place going, and with no
  backtick escape; a name of any length; an occurrence that is not whole hiding one that overlaps
  it; a root inside a word starting a place; the name hidden wherever it is; doctor not calling the
  rule; a hidden name taking the character after it; the scan running a byte past the text; `%` not
  an anchor; the end of a variable not going on into a path; and the three that no longer apply.
  The final tests fail under the other 28. Of those three, two broke code no test could tell from
  its absence, which is removed (the check for `~`, `$` and `%` in `place_goes_on`, which the scan
  makes again where it reaches the word, and a scheme's length, since a drive written `D://` has
  started a place at its letter), and one repeated another. Six survived the first set of tests: five got a
  test each (the `~` that ends a place in the same word, names read as one character, a quoted word
  that goes on from a path, `$HOME`'s string leaving a place going, an overlapping name) and the
  sixth was the redundant check; two more that only a property caught, a drive and `$HOME`'s string,
  got an example too.
- **Stress:** core's `text::` and `diagnostics::` tests at 20,000 cases per property
  (`PROPTEST_CASES`, with failure persistence off), 64 tests passing in 6 seconds; the five new
  properties at 150,000 cases, three runs of 34 seconds, passing. The version before the last three
  edits (`%` and `}` going on into a path, `place_goes_on` and the scheme's length made simpler)
  passed the same, at 150,000 over every `text::` test in 107 seconds and in three more runs of
  the new properties. A prototype of the rule in Python (in the scratchpad, not committed) ran 600,000
  random cases per check over four seeds, and over a million more of any characters, names holding
  quotes, brackets, separators and the marker's own letters included.
- **Windows:** no `#[cfg(unix)]` item is added, no import, helper or constant is used by Unix tests
  alone, and every place a test writes is made from its parts (`slashed`, `backslashed`, `path`,
  `MAIN_SEPARATOR_STR`); the rule reads both separators on every system, so a Windows-style place
  (`D:\Backup\Ann Smith`, `\\server\home\ann`) is tested here as text. No new test runs
  `sh`, so none needs `cfg(unix)`, and each runs the same on Windows; no Windows target is installed here, so Windows is CI's to build, and `pwsh` is
  not installed: no PowerShell command ran, only its text.
- **Counts:** the default workspace 778 passed (1 ignored) on `main` at 72bc175, measured first
  here, and 799 (1 ignored) with this change; the example build 775 (2 ignored), measured first, and
  796 (2 ignored). By crate: core 150 to 170 (168 in the library, up from 148, and 2 in
  `fixtures`: eighteen new in `text` and two in doctor); Claude's provider 141 to 142 (1 ignored);
  the CLI 67 (64, 1 ignored, in the example build), the API 59, the daemon 109, Antigravity 64, Codex
  63, cost 30, fake 19, Gemini CLI 29, issue 6, poll 37 and `quotadial-providers` 4 unchanged.
  Rebased onto `main` at f2a7a1e, where #88 adds one daemon test (110), the default workspace is
  779 there and 800 (1 ignored) with this change, measured again. The tray app's
  `--lib` tests 91 before (the default build, measured first) and 91 after in both builds. The
  site's tests 90 and the release scripts' tests 40 (39 passed, 1 skipped) unchanged.
- **Checks run here:** `cargo fmt --check` in both workspaces; clippy with `-D warnings` and `--locked`
  on the workspace and the tray app, default and `fake` (the app's `fake` run with the example's CLI
  bundled by `npm run cli -- --fake`, then `npm run cli` to bundle the default build's again); the
  workspace's tests, default and `fake`; the tray app's `--lib` tests, default and `fake`,
  `tests/icon_press.rs` not run, as briefed; the release build's licence key test
  (`only_a_debug_build_trusts_the_test_key`), core having changed; the tray app's `npm run build`
  (35.5 KB of 150); the site's `npm run check`, `npm run build` and `npm test` (90), since it renders
  `CHANGELOG.md`, whose built `/changelog` page shows the new line; `scripts/privacy.sh --check`,
  `scripts/public-prd.sh --check`, the 0.2.0 release notes, the release scripts' tests (40, 39 passed,
  1 skipped) and shellcheck on `scripts/`. The changelog line was held to the site's jargon list by
  hand, as no word outside a code span is on it. **Not run:** a Windows or Linux build, which no
  target here makes (CI's); PowerShell, not installed; `cargo deny`, `cargo audit` and actionlint,
  not installed here (no dependency, lockfile or workflow changed); `npm ci` and `npm audit`, no npm
  dependency changed; the tray app's `npm test` and `npm run check:window`, its window being
  unchanged; and the jargon engine on the built changelog, which lives under `~/.claude`, where the
  brief says to touch nothing.
- **Found going down #52, left for the manager or a later PR:**
  - Every other surface (`status`, `watch`, the API, the MCP server and the app's window) still shows
    the user's name in a place outside the home folder (D-129); only doctor and Send feedback hide
    it. `text::hide_user_in` is there if a later PR wants it on those surfaces, though a pasteable
    command cannot be both private and exact outside the home folder (D-130, D-131).
  - Doctor's copy of a command that names a place outside the home folder has the name hidden in its
    place (`chmod 600 '/Volumes/Data/[user]/quotadial/config.json'`), so it is for support to read.
  - `docs/CLI.md`'s Doctor paragraph lists what doctor leaves out (tokens, email addresses, the home
    folder, the licence id, webhooks) and not the user's name; adding it is one sentence, left out
    because the brief's records do not name that file and the site renders it.
  - A server's or the system's words that name the local account outside a place would show it; the
    audit found none, and a provider's sentence that does should name a place.
  - `/model` for a user called `model` and the other guesses in D-134: rare, and hiding is the
    private side of the error.
- **Safety on this Mac:** every build, test and npm run went through a wrapper that points
  `QUOTADIAL_DATA_DIR`, `QUOTADIAL_FAKE_DATA_DIR`, `CLAUDE_CONFIG_DIR`,
  `CLAUDE_SECURESTORAGE_CONFIG_DIR`, `CODEX_HOME`, `GEMINI_CLI_HOME` and `XDG_DATA_HOME` into the
  session's scratchpad, strips every token-like variable (the session's own `CLAUDE_CODE_*` and the
  like) from the environment, and turns off proptest's failure file; no vendor CLI ran, no sign-in or
  Keychain item was read, and nothing touched `~/Library/Application Support/quotadial`, `~/.claude`,
  the other worktrees or QuotaDial Example (whose daemon, running from `QuotaDial-fake`, was left
  alone). No script wrote `rm`; the mutation and first-failure runs copied the files back from
  copies in the scratchpad, and the tree holds only this change (`git status`), with no
  `proptest-regressions` file and no changed lockfile.

### Main's CI, a second flake: a place where nothing answers, on a port no test is given

- **What happened:** #90's CI at d248b46 failed once in `fake provider (macos-latest)`, in a test
  #90 does not touch: `deliveries::tests::a_webhook_that_cannot_be_reached_says_so_by_its_host`
  got "not sent to 127.0.0.1: the connection failed" where it expects "... it could not be
  reached". The job passed when run again.
- **Why:** the test made its unreachable webhook by binding a port and letting it go. A test
  running beside it can be given that same port next, and then the delivery reaches that test's
  server, which fails it otherwise.
- **The fix:** `test_support::unreachable()`, port 1 on this machine, where nothing listens, so a
  connection is refused at once, and which no test is ever given, since a port asked for as 0 comes
  from the system's ephemeral range, far above it. A port held bound without listening will not do:
  macOS lets a connection to it hang until it times out (7.8 s here, where port 1 refuses at once).
  A new test, `an_unreachable_place_refuses_and_is_never_given_to_a_test`, failed first against the
  old way (a port let go is in the ephemeral range). Three more tests made a place nothing answers
  the same racy way, and take port 1 now too: the CLI's
  `a_running_daemon_that_does_not_answer_is_shown_instead_of_polling`, and Claude's and Codex's
  `an_unreachable_..._shows_stale_numbers_without_using_up_the_floor`. Checked here: `cargo fmt
  --check`; clippy with `-D warnings` for the daemon's, the CLI's and both providers' targets; the
  daemon's `deliveries`, `test_support` and `status_pages` tests (14 pass) and the three others; CI
  runs the rest.

### M6: where and when an account's windows were read, said once for the card (D-135)

- **The ask:** the manager's review of #86 (2026-10-26, 19:28 UTC) confirmed D-132's call to keep each
  window's own source line for the time being (#52: each window carries its own time and origin) and put
  the once-per-card line, for a card whose windows all agree, on the M6 queue. The coordinator's brief of
  2026-10-26: say it once for the card when every window has the same source and was fetched at the same
  moment, each window's line then saying only its reset and pace; the rule in the view model, with Rust
  tests; the line quiet, to the bar Matt set ("make the ux something Steve Jobs would love"); the
  placements shown in pictures before one is built. Built on `feat/card-source-line` from `main` at
  82038b0 (#86), then rebased onto 72bc175 (#87). The brief's second item, a property test for
  `codeSpans`, became a pull request of its own (`test/code-spans-property`) after the manager's review
  of #87 (2026-10-26, 20:22 UTC). In review as #89.
- **The rule** (`fetched_together`, `apps/tray/src-tauri/src/meter.rs`): an account's `fetched`, such as
  "from the API 3 min ago", is set when every window of the account has the same `source` and exactly the
  same `fetched_at`, and then each window's `fetched` is none; otherwise the account's is none and each
  window's says its own, as before. A card with one window says it for the card, so the line does not jump
  to the window when a limit leaves a card after its reset, as Antigravity's do; a card with no window says
  nothing. `AccountView::fetched` and `WindowView::fetched` are both `Option<String>`, sent as `null`, and
  `src/meter.ts` types them `string | null`. The window's script has no rule: it draws what it is sent.
- **What the providers really do** (read before the rule was chosen): Claude's `usage::read` and Codex's
  give every window of one response the one `fetched_at` they are called with, `Source::Api`; Codex's log
  fallback (`sessions::log_windows`) stamps every window with its one event's time, `Source::LocalLog`, and
  a snapshot is the API's or the log's, never both; Antigravity's `windows::shown` stamps every window with
  the handover's time, `Source::StatusLine`; the fake provider takes one clock reading a fetch,
  `Source::Example`; and every replay, stale answer and sign-in waiting for its owner takes the last good
  snapshot's windows whole through `Snapshot::served_from_cache`, which changes the windows that came from
  the API to the cache together and keeps `fetched_at`. So a real card's windows agree today, always; the
  case where they differ is a guard, and a card with windows from two readings, which no provider makes, is
  in the check's fixtures.
- **Exact times, not the words:** two windows read 40 seconds apart say "4 min ago" at some moments and
  "4 min ago" and "3 min ago" at others; a rule on the words would put the line up and take it down every
  minute. A test holds that the line does not come and go across four moments, 3:30, 4:10, 4:50 and 5:20
  after the first reading, at one of which the two lines read the same.
- **Where it sits:** three placements drawn and shown to the coordinator, light and dark, over the README's
  two example accounts and the window check's made-up ones (the pictures are kept outside the repository):
  a footnote at the card's foot, which cannot sit beside the sign-in disclosure in a 360 px window (the
  label is about 212 px, the line about 122 px, a card gives 312 px) and so took a row under it; a third
  heading line under the address, which competes with it; and a caption under the windows and above the
  disclosure, which the coordinator chose: the line is about the numbers, so it sits beside them, and the
  disclosure, about the sign-in, stays the card's last thing and does not push the line away from the
  numbers when it opens. `.source`: 11 px `--muted`, right-aligned, sentence case by `::first-letter` as the
  windows' lines are, 10 px above it (the space between windows, so it is not read as a window's third
  line) and the disclosure's own 10 px below it, which the coordinator kept. It costs a row a card, 25 px:
  the two example accounts' popover is 706 px high, from 656 px.
- **VoiceOver:** the card's line is read in order, as a paragraph after the windows and before the
  sign-in; the card is still named by its heading, since the line is outside it, and each window is read as
  before but for its detail line, which no longer repeats the source (its meter's words never held it).
  Chosen over each window's accessible text carrying the line, which would repeat it as often as before.
  Read from Chrome's accessibility tree, as the window check reads the rest; macOS's VoiceOver itself was
  not run.
- **The check** (`checkSources`, `check-window.mjs`): over every account of both copies in both themes at
  100% and 200%: the line is shown once when the app sends one for the card and under each window when it
  does not, never both and never neither; the only one in its card, inside it, outside the heading and the
  panel, not hidden from a screen reader; no larger, bolder or louder than a window's lines and smaller
  than the address; under the windows with the space between windows above it; above the disclosure, which
  is the card's last thing, and still there, at the same place in its card, with the panel below it, when
  the panel is open; and the accessibility tree has it. The copies gain a card whose windows really differ,
  a lone window (the example account's), a card with no windows and an extra-usage line under the windows.
- **Failed first,** against the window broken on purpose in eleven ways, by a script kept outside the
  repository that breaks the window's markup or stylesheet, builds it, runs the check over a view of each
  copy and puts the files back, byte for byte: no line drawn (21 checks failed); every window saying a
  source of its own as well (34); the line inside the card's heading (70); hidden from screen readers (21);
  under the disclosure, which is then not the card's last thing (56); above the windows (28); loud, 14 px
  bold in the text colour (26); with no space above it (14); inside the sign-in panel (97); a window never
  saying its own source (8); and a line drawn for a card the app sent none for (4). The check's first run
  also showed two mistakes in the check itself, page positions that scrolling and other cards' panels move,
  and a match on the accessibility tree's words that took no account of the stylesheet's capital; the check
  was mended, not the window. The view model's eight new tests each failed first against a stub of the
  rule: one that never says it (6 tests failed), one that takes every card with a window to agree (4), one
  that needs two windows (2), one that compares the words (1) and one that speaks for a card with no window
  (1).
- **Counts,** `main` at 72bc175 and then this branch: the app's `--lib` tests, 91 and 99 in each build; the
  window check, 5,019 checks over 16 views and 6,721; the tray's `npm test`, 8 and 8, the property test
  being its own pull request; contrast pairs a theme, 21 and 21, the caption using a pair already held; the
  window's JavaScript, 35.5 KB of 150 KB and 35.5 KB; the workspace tests, 778 passed and 1 ignored in the
  default build and 775 and 2 in the example build, both as they were, since no crate changed; the site's
  tests, 90 passed of 90, as they were.
- **The notes of the review of #86:** note 1, the property test, is its own pull request. Note 2,
  `.window .detail::first-letter`: after this change no detail line starts with text from outside, a
  window's beginning with `text::resets`' words and the caption with "from" and one of
  `text::source_label`'s five names; the same for the lines already there is in that pull request's note.
- **Calls the brief left to me, and why** (D-135 weighs them):
  - *"Agree"* is the same source and exactly the same `fetched_at`, as the brief suggested, after reading
    what the four providers do; not the same words, which would make the line come and go as minutes tick.
  - *A card with one window* says it for the card, which I first built the other way (nothing is said
    twice under one window); the stress picture showed the lone-window card differing from every other, and
    Antigravity can drop a card from two windows to one while the popover is open, so the rule has no
    special case and the line has one place.
  - *The words* are the view model's own, "from the API 3 min ago", the first letter capitalised by CSS as
    the windows' lines are, not a new sentence: one place makes the words (`text::source_label`,
    `text::ago`), the CLI's among them.
  - *Both fields are `Option<String>`,* sent as `null`, so the window's script has no rule and cannot
    disagree with the view model.
  - *VoiceOver* hears the card's line in order, after the windows, and not each window's accessible text
    carrying it, which would say it as many times as before.
  - *Where the line sits:* the coordinator's choice, the caption above the disclosure, right-aligned. The
    gap below it is the sign-in's own 10 px, kept at the coordinator's word where the picture shown had 4 px
    (6 px a card more), so it is no nearer the disclosure than its numbers. It follows the extra-usage
    line, which is one of the numbers, and comes before the sign-in.
  - *The check's copies:* a made-up Codex account whose windows really differ, since no provider makes one;
    an extra-usage line on Claude Work; and the stale Codex account's source corrected to what a replayed
    snapshot says, "from cache 14 min ago", where the fixture said the API.
  - *The site:* its home test takes the source from the card where the card says it, and holds that the
    app says it once; the home page's picture is recorded 100 px taller (`Hero.tsx`,
    `docs/IMAGE-MANIFEST.md`); the page's copy is unchanged (see below).
  - *No new contrast pair:* the caption is `--muted` on a card, a pair already checked; its description
    names it now. No dependency, image, icon font or logo was added.
- **Left for later:**
  - VoiceOver by hand on a Mac; the accessibility tree read here is Chrome's.
  - The website's home page ("One row of the window, read aloud", and "Every number says where it came
    from") describes one window's row with its age and source in one sentence, as `quotadial status`
    writes it; the popover now says it once for the card. For the manager to say whether the page should
    follow.
  - A card whose windows differ has no real provider yet; the first that merges two readings will show it.
  - The real material on Matt's Mac, as #86 left it.
- **Checks:** `cargo fmt --check` and clippy with `-D warnings --locked`, for the workspace and the app, in
  the default and example builds; the workspace tests and the app's `--lib` tests in both builds; the
  tray's `npm run build` (types, size, contrast), `npm test` and `check:window`; the site's check, build
  and tests; `scripts/privacy.sh --check`, `scripts/public-prd.sh --check`, shellcheck and the release
  scripts' tests. Not run: Windows and Linux builds (CI), `cargo deny` and `cargo audit` and actionlint
  (not installed here, and nothing they cover changed), `tests/icon_press.rs`, the real app, and VoiceOver
  by hand.

### M5 queue item 12, PR E: the README's first-run recording

- **The ask:** #52: "`README.md` shows the app: a screenshot of the popover in both themes and a
  short recording of the first run". The manager's rule 4 on #49 (2026-10-25, 14:25 UTC): the
  recording comes from the example build, cropped to QuotaDial's icon and popover, with the account
  labelled as an example in the picture itself. The manager split it from PR D on #80. Matt set his
  menu bar to stay visible for it on 2026-10-26 (#51).
- **What it shows:** `docs/images/first-run.gif` (376x600, 5.5 s, 196 KB), in the README under the
  window's picture, and the same as an MP4 (`docs/images/first-run.mp4`, H.264, 60 KB), linked
  under it. QuotaDial Example, built at 485d0cf (#86's head, the same tree as 82038b0 on `main`),
  starts as a first run with an empty data folder of its own. Its icon shows 91% in the menu bar, a
  click opens the window, whose two accounts are named "Example" in the picture itself (D-117), and
  the window stays open for 3 seconds. First run to the number while recording: 391.9 ms, with the
  icon in the menu bar at 331.5 ms.
- **Only the example in it:** recorded through ScreenCaptureKit with a filter that takes the
  example's windows and nothing else, so the menu bar, other apps' icons (Matt's own QuotaDial among
  them), their windows and the desktop are in neither the picture nor the raw file. Every frame was
  checked to be the plain background outside the icon and the window (297 frames; the largest
  spread was 3, where 8 is allowed), then cropped to the two.
- **On macOS 26, Control Center draws the icon.** The first take recorded nothing: macOS 26 draws
  every app's menu bar icon in a window of Control Center's own, so a filter of the example's
  windows held no icon, and the recorder waited for one in vain. The recorder, kept outside the
  repository with the builder's notes, now waits until the icon shows its number, then records the
  example's windows and the one Control Center window that lies within the icon's width, so a
  neighbour's icon is never in it.
- **The Graphics and Media helper's third reading** (#80 left open whether the 245 MiB it held in its
  first minutes was the app or macOS): a 6-minute run of the same build, sampled every 10 seconds
  from 21:04 to 21:11 UTC. The helper held 244.7 MiB at the launch, then 11.5 to 14.5 MiB from 10
  seconds later, except 240.9 MiB at the one sample with the window open, and 14.5 MiB again at the
  next. So it takes about 245 MiB while the window is drawn and gives it back within 10 seconds;
  #80's samples, a minute apart, could not see it go. The app's process: 22.1 MiB of footprint on
  average, 24.9 at most; its helpers' footprints add 48.4 MiB on average.
- **Left for later:** the recording shows each window's source under it, as the window did at #86.
  #89, in review, says it once for a card whose windows agree, so a recording made after it would
  differ in that line. A new recording needs Matt's menu bar again, so it waits for a change worth
  one.
- **Checked:** `scripts/privacy.sh --check` and `scripts/public-prd.sh --check`. No code changed.

### Test: a property test for `codeSpans` (the manager's note 1 on #86)

- **The ask:** the manager's review of #86 (2026-10-26, 19:28 UTC), note 1: `codeSpans` "is a parser of
  text that can carry words from outside", CLAUDE.md's rule is "a test that no input breaks a
  parser", and `code.test.mjs` held its shape with three cases, so "add one that feeds it random
  strings of backticks, spaces, brackets and punctuation and checks only that the spans joined give
  back the text". Made a pull request of its own by the coordinator after the manager's review of #87
  (2026-10-26, 20:22 UTC): one pull request each, the `[user]` mask, then the once-per-card source
  line, then this. Built on `test/code-spans-property` from `main` at 72bc175 (#87). In review as #90.
- **The test** (`apps/tray/scripts/code.test.mjs`, the only code changed): 5,000 strings a run, half of
  up to 11 characters and half of up to 59, each character one of 31 pieces: a backtick (four of the
  31), a space (three), the brackets `([{}])`, a phrase's punctuation `.,;:!?`, quotes, `$`, a
  backslash, a few letters, a line break, a tab, a no-break space, an accented letter and an emoji,
  which takes two UTF-16 units. Each goes through `codeSpans`; the spans joined, each command between
  its backticks again (`written`, as the three cases above use it), must be the string, and a throw
  fails the case too. Only the join is checked, as asked: where commands are found is held by the
  cases above it.
- **The seed:** a new one each run, so over time the strings are many, from a small generator
  (mulberry32, no dependency). It is in every failure with the case's number and the string as JSON,
  and `QUOTADIAL_TEST_SEED=<seed>` runs the same strings again; a value that is not a whole number
  below 2 ** 32 is refused.
- **Not vacuous:** about 18% of the strings hold a command (16% at the least, over 200 seeds of 3,000
  strings), and the test fails if fewer than 5% do, so a change to the pieces cannot leave the code
  that finds a command unreached.
- **Failed first,** against five copies of `codeSpans` broken on purpose, kept outside the repository:
  a stray backtick dropped from the words; a command's text taking its closing backtick; the character
  after a command skipped; words of only spaces between two commands dropped; and a throw on a
  backtick at the end. Each failed, naming its seed, and the seed printed for the first ran the same
  string again and failed again; the real file passes with it.
- **The manager's note 2, checked:** `.window .detail::first-letter` and `.window .pace::first-letter`
  (`text-transform: uppercase`) capitalise nothing from outside today. `WindowRow` (`src/app.tsx`) is
  the only place either line is drawn. The detail line begins with the window's `resets`, which
  `text::resets` writes from a time alone, as "resets in ...", "reset ... ago" or "no reset time
  reported", then the source's words; the pace line is `text::pace`'s, and each of its five shapes
  begins "at {rate}". A provider's own words reach a window only in its label (`Weekly (Opus)`), a
  line of its own. Every other `.detail` (History's span, Help, the week's cost, Settings) is outside
  `.window`, which the selector does not reach. Nothing to change; it stays something to keep in mind
  if a detail line ever starts with an address or a command.
- **Checks:** `npm test` in the tray, 9 tests, up from 8 (Chrome's five, the three of `codeSpans` and
  this one). Nothing else changes: no window code, no Rust, no picture. No D, since a test settles
  nothing the PRD left open, and no CHANGELOG line, since it changes nothing people see.

### Main's CI: the daemon's stand-in servers say they close each connection

- **What happened:** CI on `main` at 72bc175 (#87) failed once, in `fake provider (macos-latest)`:
  `deliveries::tests::a_webhook_that_fails_is_tried_again_later_and_its_address_never_shown` saw
  one delivery where it expected two ("again a minute later"). The job passed when run again, and
  #87 changed nothing the test touches.
- **Why:** the daemon's tests' stand-in webhook (`test_support::receiver`) and status page
  (`test_support::status_page`) take one request on each connection and then close it, but their
  answers did not say so. A `reqwest` client keeps a connection for its next request unless told
  otherwise, so the retry could be sent down the connection the stand-in had just closed and never
  arrive. Whether it did turned on timing, which a busy CI runner changes.
- **The fix:** both answers say `connection: close`, so the client opens a new connection for each
  request. A new test, `the_stand_ins_say_they_close_each_connection`, holds both stand-ins to it
  and sends two requests through one client; it failed first, with the header taken out. The retry
  test then passed 300 runs in a row. Nothing outside the tests changed. Checked here: `cargo fmt
  --check`, clippy with `-D warnings` for the daemon's targets, and the daemon's `test_support`,
  `deliveries` and `status_pages` tests; CI runs the rest.

### M6: a folder whose name only starts as the home folder's is never written `~` (D-133)

- **The ask:** the coordinator's brief of 2026-10-26: D-130's other leftover, now its own queued row,
  "a place holding the home folder's path beside a space or a quote", fixed in `text::tilde_in` as
  the manager's review of #85 (2026-10-26, 13:40 UTC) said, "the fix belongs in `tilde_in`, not in
  either command": a folder whose name only starts as the home folder's does is left as written in
  a sentence and in both commands, with properties that fail first and each case of the row as a
  test. Built on `fix/home-folder-longer-names` from `main` at 284a97c (#85), then rebased onto
  82038b0 (#86) with the code commit unchanged (`git range-diff`). In review as #87.
- **The bug, as the old rule wrote it** (what the new tests printed against it, with the home
  folder `/Users/ann`): "could not open the history at ~ (old)/quotadial/history.db: permission
  denied"; `'~ (old)/quotadial/config.json'` for the `chmod 600` command (the daemon's own test:
  "run `chmod 600 '~ (old)/quotadial/config.json'`"); `CLAUDE_CONFIG_DIR='~ (old)/.claude' claude`
  from doctor's pass; and, run by `sh` with a stand-in `claude` that prints its folder, `~ (old)/claude`
  where the folder's own path was meant. The properties' shrunk inputs showed two more: the home
  folder's text inside a quoted word after a character no path holds, `'/aA\0/aA'` for a place
  outside the home folder `/aA`, came out `'/aA\0~'`; and `/AA:-` came out `~:-`, a mark taken for
  the end of a path though a name character follows it.
- **Where the fix is** (`0e3f5a4`): the code in `crates/core/src/text.rs` alone: `tilde_in` and its
  private helpers. Neither command's writer, no caller and no file-system read changed. Four comments are
  brought in line: `shell_word`'s and `powershell_string`'s docs, the one in
  `crates/daemon/src/config.rs` where `config.json`'s warning goes through the pass, which named
  D-130's exception, and `claude_in`'s doc.
- **A word in single quotes is read whole** (`single_quoted`, `posix_quoted`, `powershell_quoted`):
  a quote opens a word at the start of the text or after a character that is not a letter, a digit,
  a quote or a backslash, so the apostrophe in `Ann's` opens none and the quotes inside `'\''` and
  `''` are not openers. The word is read by each shell's rules: POSIX, to the next quote unless it
  is `'\''`; PowerShell, to the next of the five single quotation marks unless another follows it
  at once, the same five `powershell_string` writes twice. The text does not say which shell wrote
  it, so the longer of the two readings is taken, which never cuts a place short; a word that
  neither reading closes is not a word and is read as the sentence it may be. A word whose text is
  the home folder's path, or starts with it and a separator, and starts a place (a word that goes on
  from a path, as `'Library'` does after `~/`, does not), is written `'~` and the rest as it was;
  every other word is copied exactly, so nothing inside it is rewritten, which also takes in the
  home folder's text after a space inside a place outside it (`'/Volumes/Backup /Users/ann/x'` was
  written `'/Volumes/Backup ~/x'`).
- **A sentence is read by what follows the home folder's text** (`ends_path`, `name_goes_on`,
  `starts_a_place`, `is_line_break`): `~` at a separator, as before; at the end of the text or of a
  line; and at the seven marks (`:`, `,`, `;`, `)`, a quote, a backtick, and a full stop before a
  space or the end) when nothing but marks joins it to the next space, unless the next word on the
  line carries the name on: it opens with `(` or `[`, or holds a separator after a name character
  (a letter, a digit, `_` or `-`), and is not a place of its own (from a root, `~` or a drive,
  `C:` and a separator). Any other character joined to the home folder is more of the name, where
  each of the seven marks used to end the path whatever followed it. A line break ends a path.
  The home folder's own text inside the next word is skipped when that word is read, so that
  writing a second mention of it `~` cannot change the answer for the first (`H x(H) y`, which
  `skip_home_in_word_off` below showed needs it).
- **Calls the brief left to me, and why** (D-133 weighs the alternatives and says what each costs):
  - *The sentence rule:* a look-ahead for a separator, narrowed to the next word on the line. The
    places-passed-in alternative changes nine call sites in eight files and cannot name the places
    in SQLite's, the operating system's or a server's words, so each would keep the user's name; it
    stays the next step if a place that matters turns out to be one no text can tell, beside the
    rule and not in its place. A wider window finds names of several words and keeps the home
    folder's name wherever a later word looks like a path.
  - *Which error is worse:* a wrong place named, which makes the sentence false and the pasted
    command fail, over a name shown, which is what every place outside the home folder is already.
    So the guess leaves the name where the text offers a reason to think it goes on, and writes `~`
    only where the home folder's end is clear.
  - *The bracket:* a next word opening with `(` or `[` keeps the name, so a leaf such as
    `/Users/ann (old)`, with no separator after it, is read right: the shape people give an old
    copy, `(old)`, and Windows a second one, `(2)`. Not `-`, which prose puts between a path and its
    reason.
  - *Marks:* the seven are the old set; they end a path only when nothing but marks is joined to
    the next space, so `ann;x`, `ann's` and `ann)/x` are longer names (the brief's "`ann, old`,
    `ann;x`, `ann)` and the rest of `ends_path`'s characters"), a full stop only before a space or
    the end as before (`ann.old`, `ann..`).
  - *Single quotes only:* the only quoting `shell_word` and `powershell_string` write around a
    place outside the home folder. Double quotes and backticks are read as a sentence reads them.
  - *Opacity:* a quoted word that does not start in the home folder is left whole even where the
    home folder's text sits inside it after a space, since "no exceptions" is about every place
    outside the home folder. The cost is a single-quoted phrase in a sentence, which keeps the home
    folder's name after a space (D-133).
  - *Not pinned by a test:* what the rule still gets wrong. A test that held `~ old stuff/x` would
    hold a wrong answer in place; D-133 lists them.
- **What it still gets wrong** (D-133 has them in full): a name of several plain words whose
  separator comes after the second, and a folder that ends the place after a plain word, written
  `~ old stuff/x` and "could not use ~ old: ..."; the home folder itself followed by a word with a
  separator in it or by a bracket, left as written; its path followed by a mark and then
  something that is not a mark or a space, such as `"}` at the end of a JSON string, left as
  written (before this it was `~`); a name that ends in `)` or `.`, read as the home folder and a
  mark; a name that holds a line break; a single-quoted phrase in a sentence that holds the home
  folder after a space, and whatever follows an apostrophe used as an opening quote up to the next
  apostrophe, both left as written; and PowerShell's `"$HOME\..."` string, read as a sentence,
  which cannot hold the home folder's path after a space on Windows, where a name holds no `:`.
- **Tests,** each new one failing first (below): in core, five tests and four properties.
  - `a_folder_whose_name_goes_on_from_the_home_folders_is_left_as_written_in_a_sentence`: thirteen
    names after the home folder's own (` (old)`, ` old`, a tab and a word, `'s`, a curly `’s`,
    `, old`, `;x`, `)`, `:x`, `"x"`, a backtick pair, `. old`, `'`), each beside `/Users/ann` and
    `C:\Users\ann`, in four wordings of a sentence, every expected place built from its parts.
  - `a_folder_whose_name_goes_on_from_the_home_folders_is_quoted_whole_in_a_posix_command` and
    `..._is_a_single_quoted_powershell_string`: the brief's own examples (`chmod 600
    '/Users/ann (old)/quotadial/config.json'`, `'/Users/ann'\''s/...'`, PowerShell's `'...ann''s...'`
    and its curly `’’`), then each name for the folder itself and a file in it, in `chmod 600`,
    `CLAUDE_CONFIG_DIR=`, `$env:CLAUDE_CONFIG_DIR =` and `config.json`'s whole warning.
  - `the_home_folder_on_its_own_is_told_from_a_longer_name_beside_it`: both in one sentence, the
    home folder alone after seventeen endings (the end, a stop, a colon, a comma, a bracket, a space
    and a word, another path, a relative path, a drive path, a line break) and the `H x(H) y` idempotence case.
  - `a_word_in_single_quotes_is_read_whole`: `'~'`, `'~/a b'`; ten words that go on (with `'\''`, `''`,
    `’’`, `.`, `)`); four that do not start in the home folder; an apostrophe and a quote that
    never closes.
  - Properties: `a_place_outside_the_home_folder_is_left_as_written_in_both_commands` (the brief's
    property 1, with the home folder's text and every quote among the rest, in five places a command
    is written); `a_posix_command_for_a_place_in_the_home_folder_is_left_as_written`;
    `what_follows_the_home_folder_tells_it_from_a_longer_name` (property 3: the sentence rule over
    its marks, spaces, brackets, line breaks and separators); and
    `a_word_in_single_quotes_is_read_whole_before_it_is_taken_for_the_home_folder`.
  - Existing, widened (property 2): `no_text_or_home_folder_makes_writing_the_home_folder_panic` takes
    quotes and a bracket in its short alphabets, and `writing_the_home_folder_again_changes_nothing`
    takes marks, quotes, brackets, tabs, line breaks, words with a separator, ` (old)`, `'\''` and
    `''` among the home folder's own text. They pass against the old rule too, which is
    idempotent, so they are the one place a new shape has no first failure.
  - Extended, each failing first: in the daemon, `a_warning_names_config_json_with_the_home_folder_written_with_a_tilde`,
    `a_file_with_webhooks_that_others_can_read_gets_a_warning` and
    `the_warnings_chmod_command_works_as_written_in_a_shell` (a folder `ann (old)`, `ann's` and
    `ann, old` beside the home folder; `sh` runs the warning's own `chmod 600` and finds the file
    0600); in Claude's provider,
    `the_fix_command_for_a_folder_outside_the_home_folder_quotes_the_whole_folder` and
    `the_fix_command_gives_claude_code_the_folder_when_a_shell_runs_it` (`ann (old)/claude` and
    `ann's Claude`, with the home folder `ann` and `Ann Smith`, as a sentence shows the command and
    as doctor's pass writes it).
  - *First failures:* against the original `tilde_in` block with the final tests (core), all nine
    new tests failed and the two widened properties passed; the extended tests failed in the daemon
    (3 of 3) and Claude's provider (2 of 2). An earlier version of the tests, run first, failed the
    same way and printed the shrunk inputs above.
  - *Mutations:* fifteen hand-made breaks of the new rule, each applied alone to the finished
    file and run against core's `text::` tests: no `'\''` continuation, no PowerShell reading, a
    word not left whole, no bracket rule, no line-break rule, no name before a separator, a mark
    ending a path anywhere, no rule on what a quote opens after, no drive rule, no skip of the home
    folder's text in the next word, a quoted word that is the home folder alone dropped, a quoted
    word in the home folder with a separator dropped, `_` and `-` not name characters, a different
    set of line breaks, and a quoted word after a path character taken for a place. Two survived the first set of tests: the drive, which a backslash
    cannot show on Unix (`C:/Windows` does), and the skip of the home folder's text, which the
    widened existing property found only in some runs and the new `H x(H) y` example now fails
    every time. Each of the fifteen now fails a test.
  - *Stress:* core's `text::` tests at 20,000 cases per property (`PROPTEST_CASES`) with failure
    persistence off, passing in 6 seconds, then at 150,000. The first 150,000-case run found one
    more case, `~/'C:\/A0'` (a place in a home folder `C:\/A0` as a Unix-built Windows-style home
    makes it, whose rest has the home folder's own text): the quoted word after `~/` has the home
    folder's text, but goes on from a path and is not a place of its own. Only a word that starts
    a place is now checked against the home folder; `a_word_in_single_quotes_is_read_whole` pins it
    (`cd ~/'/Users/ann'` stays as written, and the old rule wrote `~/'~'`). Three more runs at
    150,000 cases each pass, in about 42 seconds. No `proptest-regressions` file is added.
- **Windows:** no `#[cfg(unix)]` item is added to a test that runs there, no import, helper or
  constant is used by Unix tests alone, and every expected place is built from its parts
  (`MAIN_SEPARATOR`, `longer`, `home_folders`); an expected `~/` after a home folder is in a test
  whose input names the separator itself, as D-129's does. The `sh` tests are `cfg(unix)` already.
  No Windows target is installed here, so Windows is CI's to build, and `pwsh` is not installed
  either: no PowerShell command ran.
- **Counts:** the default workspace 769 passed (1 ignored) on `main` at 284a97c, measured first
  here, and 778 (1 ignored) now. By crate: core 141 to 150, the nine new tests; the daemon 109, Claude
  141 (1 ignored), the CLI 67, the API 59, cost 30, issue 6, poll 37, Antigravity 64, Codex 63, fake 19,
  Gemini CLI 29 and `quotadial-providers` 4 unchanged, their extended tests the same tests. The
  example build 766 (2 ignored) by D-131's notes, not measured again, and 775 (2 ignored) now,
  core's nine and nothing else. The tray app's `--lib` tests 87 in both builds, before by D-131's
  notes and after here. The site's tests 90, the release scripts' tests 39 (1 skipped).
- **Checks run here:** `cargo fmt --check` in both workspaces; clippy with `-D warnings` and
  `--locked` on the workspace and the tray app, default and `fake` (the app's `fake` run with the
  example's CLI bundled by `npm run cli -- --fake`, then `npm run cli` to bundle the default build's
  again); the workspace's tests, default and `fake`; the tray app's `--lib` tests, default and
  `fake`, `tests/icon_press.rs` not run, as briefed; the release build's licence key test
  (`only_a_debug_build_trusts_the_test_key`), core having changed; the tray app's `npm run build`
  (34.0 KB of 150, contrast passing); the site's `npm run check`, `npm run build` and `npm test`
  (90), since it renders `CHANGELOG.md`, whose built `/changelog` page shows the new line;
  `scripts/privacy.sh --check`, `scripts/public-prd.sh --check`, the 0.2.0 release notes, the
  release scripts' tests (39, 1 skipped) and shellcheck on `scripts/`. **Not run:** a Windows or
  Linux build, which no target here makes (CI's); PowerShell, not installed; `cargo deny`, `cargo
  audit` and actionlint, not installed here (no dependency, lockfile or workflow changed); `npm ci`
  and `npm audit`, no npm dependency changed; the tray app's `npm test` and `npm run check:window`,
  its window being unchanged.
- **Found going down #52, left for the manager or a later PR:**
  - `tilde` writes a place from its parts and is exact; `tilde_in` can only read text, so it
    guesses where a name ends, and always will. Wherever a message knows the place it names, writing
    it with `tilde` and leaving `tilde_in` to the outside text in it keeps the guess to what
    cannot be known; D-129 chose one pass where a message is made, and D-133 keeps it. If a place
    that matters turns out to be a leaf the look-ahead cannot read, such as a data folder
    `/Users/ann old`, passing the places QuotaDial knows in is the fix, beside the rule.
  - Doctor's `[user]` pass and a user named `claude` is now a queued row of its own (the manager's
    review of #85 left it queued).
- **Safety on this Mac:** every build, test and npm run had `QUOTADIAL_DATA_DIR`,
  `QUOTADIAL_FAKE_DATA_DIR`, `CLAUDE_CONFIG_DIR`, `CLAUDE_SECURESTORAGE_CONFIG_DIR`, `CODEX_HOME`,
  `GEMINI_CLI_HOME` and `XDG_DATA_HOME` in the session's scratchpad and no token in its environment;
  the `sh` tests make their home folders and stand-in in a temporary folder and run with `PATH`
  holding the stand-in alone. No vendor CLI ran, no sign-in or keychain item was read, and nothing
  touched `~/Library/Application Support/quotadial` or `~/.claude`. No script wrote `rm`; the
  mutation runs copied the file back from a copy in the scratchpad, and the tree holds only this
  change (`git status`), with no `proptest-regressions` file and no changed lockfile.

### Matt's ask: the popover redesigned (D-132)

- **The ask:** Matt, on 2026-10-26 at about 12:50 UTC: "make the OS X UX more professional and make
  it easier to see separation between accounts and what the account email addresses are. make the
  ux something Steve Jobs would love." Built on `feat/popover-design` from `main` at c7dedc8 to the
  coordinator's brief, in steps the coordinator pushed and reviewed from pictures as they came: the
  cards and their heading (0e847f0); commands as code, the manager's note on #85 (e78fa63); a
  card's title a step larger and a hairline edge in light, from the review of the first pictures
  (c3d6721); the summary naming the account (8a3621f); the footer, to the coordinator's spec
  (699f56e); History, Settings and Help (7f0cf7b); the coordinator's three findings on the final
  pictures (12bddd4); an independent design review's findings, as the coordinator decided them
  (2363064); then the pictures and these records. In review as #86, rebased onto 284a97c (#85)
  with only these records' three conflicts to settle, both sides kept; the commits are named here
  as they were built, and `git range-diff` shows each of them unchanged by the rebase.
- **Each account a card** (`style.css`, `app.tsx`): a rounded platter, 10 px corners and 12 px of
  padding, 8 px from the next, raised on the page in both themes: `--card` white at 70% in light,
  with a 0.5 px hairline (`--card-edge`, an inset shadow), and white at 6% in dark. The page Windows,
  Linux and the pictures paint is a grey like the macOS popover material (`--background` `#ececec`
  and `#2c2c2e`, were `#fbfbfd` and `#1f1f21`); on macOS the page stays transparent over the
  material (`html.mac`). Space and the cards' grounds set accounts apart, with no rules. Problems,
  the week's cost and an update offer take the same card; Free's lines stay in words under the
  accounts, in line with the cards' text.
- **One heading for an account** (`src/account.tsx`, `AccountHeading`), in the popover and History:
  the name the user gave it or its provider's (P-020), 14 px semibold, the plan a capsule on the
  right and a tinted status capsule beside it when it is not ok; under it the address, 12 px in
  `--secondary`, then under a name given the provider in `--muted`: "dev@example.com · Claude", so
  the addresses start at one edge down the cards (the review's finding; the brief had the provider
  first). Nothing for an account whose name adds nothing (D-124). The address alone is selectable;
  it sits in the heading, so a screen reader names the section by both, and the dot is hidden from
  it. Settings' names are headed by the address with the provider under it, or the provider alone.
  `AccountHistory` carries `alias` and the account's own `name` apart (`history.rs`), and
  `AccountChoice` the `provider` and `name` in place of `label` (`lib.rs`).
- **Windows:** the reset, source and pace at 11 px in `--muted`, sentence case by CSS
  (`::first-letter`), so the view model's words are unchanged; the unconfirmed capsule after a real
  space, so a screen reader does not run it into the line; every disclosure's chevron drawn in CSS,
  turning in 150 ms, and not at all with less motion.
- **The summary** (`meter.rs`): `Meter` keeps the window that sets the number (`Tightest`, skipped
  by serde) and builds the headline from the account as listed, `AccountView::label`, in `new` and
  again in `with_aliases`: "Weekly · Claude Work · resets in 1 d 15 h", or the provider and the
  address, or the provider alone for an example account. The tooltip has "88% · {headline}" after
  "QuotaDial"; `lines()`, which the tray's menu shows, is unchanged.
- **Commands** (`src/code.ts`, `src/said.tsx`): `codeSpans` splits a message into words and
  commands; `Said` shows each command as code (`ui-monospace`, the text colour on `--tag`,
  `user-select: all`) in an account's note and sign-in panel, the problems and Settings' warnings.
- **One toolbar at every view's foot:** History, Settings and Help under the accounts, Done under a
  panel (the panels lose their `onDone`), borderless with a fill only on `:hover` and
  `:focus-visible`, and Quit pushed to the far end; the toolbar's negative 8 px margin puts each
  label, and the origin line under them, at the cards' text edge (measured at 24 and 336 px in the
  360 px window). Export CSV stays in History, under its last card.
- **Panels:** title 15 px; words outside a card padded to the cards' text; a button outside a card
  at the cards' edge with a 12 px label inset (all buttons now 12 px, were 10). History: each
  account a card; each chart with no plate, a hairline at 0% (`--chart-base`) and a faint line at
  100% (`--chart-top`) drawn whole at the edges (`overflow: visible`), the dashed line at 90% kept;
  the period a segmented control, a `--tag` track with the chosen segment raised (`--segment`,
  `--segment-shadow`) and bold. "The last 7 days." under it stays: it is the charts' span from the
  app, which reads "Since Mon 5 at 09:10, when the history begins." when the history is younger
  than the period, and is not wired to the control. Settings: each group a card under its name.
  Help: its four sections cards.
- **Colours:** new `--secondary` (`#48484d`, `#c7c7cc`), `--card`, `--card-edge`, `--attention`,
  `--tag-pressed`, `--segment`, `--segment-shadow`, `--chart-base` and `--chart-top`; in light
  `--muted` `#58585d`, `--focus` `#0071e3`, `--field` `#78787d` and the bars `#007aff` (were
  `#67676c`, `#0a84ff`, `#8a8a8e` and `#0a84ff`), `--warning` `#8f4b00` (was `#a05600`); in dark
  `--muted` `#b0b0b5`, `--warning` `#ffb340`, `--high` `#ff7f78` and `--field` `#8e8e93` (were
  `#a1a1a6`, `#ff9f0a`, `#ff453a` and `#6e6e73`); `--accent` and `--on-accent` gone; a field's hint
  in `--muted`, where the browser's grey was 4.08:1 and 3.02:1 on a card; off the Mac a box takes
  the bars' blue. Ratios on a card: the address 8.66 (light) and 6.88 (dark), a window's details
  6.74 and 5.36, a window past 90% 5.65 and 4.72, a status capsule 5.57 and 4.68, a chart's line
  3.83 and 3.18. `check-contrast.mjs` lays each colour over a stack of grounds; 21 pairs a theme,
  up from 10.
- **Calls where the brief left room:** the title at 14 px (the coordinator's, after the first
  pictures); a hairline for light cards' edge; the details at 11 px, below the address's 12 px;
  sentence case by CSS; a tinted status capsule; the address alone selectable; the headline naming
  the account by provider and address when it has no name given, since one address can have
  accounts at two providers; the tooltip's new line, the menu unchanged; Settings' group names over
  their cards; Help's sections as cards too; the costs card's words unchanged.
- **The independent review** (a fresh-context reviewer, from the pictures alone): "ship with
  fixes". Taken: the field hints, colour to spare on the material, the checkboxes off the Mac, the
  "Read as:" and chevron offsets, and, by the coordinator's calls, the address first, raised cards
  and Settings' names headed by the address. Left, by the coordinator's calls, and in D-132: a
  17 px title, a line once per card for the source and sign-in panel, and the origin line in Help.
- **Proof:** the window check adds five checks (`checkCards` and `checkSummary`, each at 100% and
  200%, `checkCommands`, `checkHeadings` and `checkToolbar`); each failed first on a window broken
  on purpose: cards 3 px apart; the address moved out of its card and heading and made
  unselectable; the note as plain text and the code selectable like any text; the headline naming
  no account, and hidden; History headed as before, and Settings headed by the provider; a panel
  with a Done of its own, and Quit beside the others. The contrast check failed on a pale chart
  line (2.17:1). The four new meter tests and the two changed ones failed against the old headline
  and tooltip; `code.test.mjs`'s two tests of hard cases failed against a split at every backtick.
  The design detector (`impeccable detect`) found nothing in the changed sources, after it was
  shown to fire on a planted side-tab border and gradient text.
- **Pictures:** `site/scripts/app-pictures.mjs` wrote the six pictures again: accounts 720 x 1312
  (was 1214), History 720 x 2078 (was 1776) and Settings 720 x 2172 (was 1826), with the site's
  `AppShot` sizes and `docs/IMAGE-MANIFEST.md` to match. Main's committed History picture was older
  than main's window, which captured at 1918 px.
- **Counts:** in the workspace, 763 passed (1 ignored) and in the example build 760 (2 ignored),
  both as on `main` at c7dedc8, since no crate changed; the app's `--lib` tests 91 in both builds,
  up from 87 (the four new meter tests); the window's `npm test` 8, up from 5 (the three of
  `code.test.mjs`); the window check 5,019 checks over 16 views, up from 4,371; the contrast check
  21 pairs a theme, up from 10; the window's JavaScript 35.5 KB, up from 34.0 KB, within its 150 KB;
  the site's tests 90.
- **Checks run here:** the heavy run, from 18:26:58 to 18:33:51 UTC with the app's Rust as
  committed in 7f0cf7b, which nothing after it changed: `cargo fmt --check` in both workspaces;
  clippy with `-D warnings` and `--locked` on the workspace and the tray app, default and `fake`;
  the workspace's tests, default and `fake`; the tray app's `--lib` tests, default and `fake`, the
  latter with the example's CLI bundled (`npm run cli -- --fake`, then `npm run cli` again);
  `scripts/privacy.sh --check` and `scripts/public-prd.sh --check`. Then, on the final tree: the
  tray app's `npm run build` (its budget and contrast), `npm test` and `npm run check:window`; the
  app's `--lib` tests in both builds again, since they read `CHANGELOG.md`; and the site's
  `npm run check`, `npm run build` and `npm test`, after `site/scripts/app-pictures.mjs`. All
  passed.
- **Not run:** a Windows or Linux build, which no target here makes (CI's); `cargo deny`,
  `cargo audit` and actionlint, not installed here, and no dependency, lockfile or workflow
  changed; the app itself, never run, as briefed: the window check renders the built window in
  headless Chrome, and a scratch preview drew it as macOS does, a transparent page over a stand-in
  for the material; `tests/icon_press.rs`, as briefed (clippy compiles it, nothing runs it);
  VoiceOver by hand: what a screen reader is told was read from Chrome's accessibility tree.
- **Safety on this Mac:** every build, test and npm run had `QUOTADIAL_DATA_DIR`,
  `QUOTADIAL_FAKE_DATA_DIR`, `CLAUDE_CONFIG_DIR`, `CODEX_HOME` and `GEMINI_CLI_HOME` in the
  scratchpad and no token in its environment; no vendor CLI ran, no sign-in or keychain item was
  read, and nothing touched `~/Library/Application Support/quotadial`.

### M6: Claude's sign-in command works as written and names no user (D-131)

- **The ask:** the coordinator's brief of 2026-10-26: the command QuotaDial shows to sign a Claude
  sign-in folder in again works when pasted, in a POSIX shell and in PowerShell, and never names
  the user, even after doctor writes the home folder `~`. It is D-130's leftover, which the
  manager's note on #84 (11:30 UTC) called "a real bug a user can hit" and put ahead of D-130's
  other one, now a queued row of its own. Built on `fix/claude-sign-in-command` from `main` at
  b144911 (#84). In review as #85.
- **The bug, as the old code wrote it** (the new doctor test's failure against it printed both):
  for a sign-in kept for `/Users/ann/My Claude`, the sentence said "Run
  `CLAUDE_CONFIG_DIR='/Users/ann/My Claude' claude` to sign in again.", the user's name in it,
  and doctor's text, after its `tilde_in` pass, "Run `CLAUDE_CONFIG_DIR='~/My Claude' claude`
  ...". Run by sh, dash, bash, zsh and ksh with a stand-in `claude`, the second handed over
  `~/My Claude` as written.
- **One rule for a POSIX shell word** (`ba0d8ad`, a move with no change of behaviour):
  `shell_word` went from `crates/daemon/src/config.rs` to `crates/core/src/text.rs`, public, with
  its doc comment, its three tests and their `word` helper; `config.json`'s warning calls
  `text::shell_word`, and the comment at its `tilde_in` pass names it.
- **PowerShell** (`d0931fc`): `text::powershell_string(place, home)` beside it, with PowerShell's
  quotation marks as two private constants, `POWERSHELL_DOUBLE_QUOTES` (`"`, `“`, `”`, `„`) and
  `POWERSHELL_SINGLE_QUOTES` (`'`, `‘`, `’`, `‚`, `‛`). `shell_word` and it find the home folder
  through one private helper, `after_home`: whole names, by `Path::strip_prefix`, and never a home
  folder that is a whole disk, as `tilde_in` takes it. In the home folder `"$HOME`, the system's own
  separator and the rest with a backtick before each backtick, `$` and double quotation mark, then
  `"`; `"$HOME"` for the home folder itself; elsewhere the whole place in single quotes with each
  single quotation mark written twice.
- **Microsoft's rules, read on Microsoft Learn on 2026-10-26:** about_Quoting_Rules (PowerShell
  7.6): a double-quoted string replaces "Variable names preceded by a dollar sign", the backtick is
  "the PowerShell escape character", a single-quoted string is passed "exactly as you type it",
  and PowerShell "treats smart quotation marks ... as normal quotation marks";
  about_Automatic_Variables (7.6, and 5.1's page): `$HOME` takes `USERPROFILE` on Windows and
  `HOME` on Unix;
  about_Special_Characters (7.6): the escape sequences are `` `0 `` and letters, `` `e `` and
  `` `u{x} `` added in PowerShell 6, none of which this writes; and the PowerShell Language
  Specification 3.0, chapter 2, kept "for historical reference": 2.3.5.2 lists the four double
  and five single quotation marks and a delimiter written twice, and 2.3.7 takes any other
  character after a backtick as written. Rust 1.99.1's docs for `std::env::home_dir()`: on Windows
  `USERPROFILE` when it is set and not empty, else `GetUserProfileDirectory`, so `$HOME` is the
  folder QuotaDial stripped.
- **The command** (`82c26a7`, `crates/providers/claude/src/login.rs`): `claude_in(dir, home,
  windows)` takes the home folder where it took the folder already written with `~`, and is
  `CLAUDE_CONFIG_DIR={text::shell_word} claude` or
  `$env:CLAUDE_CONFIG_DIR = {text::powershell_string}; claude`. `Login::sign_in_command` passes its
  environment's home folder, the one `env.tilde` writes the sentence's place from. A folder that
  needed no quotes is quoted now too, `CLAUDE_CONFIG_DIR=~/'.claude-work' claude`, by D-130's
  rule; `docs/CLI.md`'s example and the changelog show the new forms.
- **The shells, by hand** (scratch scripts in the session's scratchpad, not committed), each
  command run with `HOME` set to `ann` and then `Ann Smith` in the scratchpad and a stand-in
  `claude` first on `PATH` that prints `CLAUDE_CONFIG_DIR`: sh (bash 3.2 as sh), dash, bash 3.2,
  zsh 5.9 and ksh 93u+ gave the folder's own path for `~/'My Claude'`, `~/'.claude-work'`,
  `~/'it'\''s Claude'`, `~` alone, `~/'a b/c d'`, a name holding `$HOME`, backticks, `"`, `\`, `*`
  and `~`, and `'/Volumes/Data/My Claude'`. tcsh 6.21, having no `NAME=value command` form,
  answered "Command not found." to every form, the old ones included, such as
  `CLAUDE_CONFIG_DIR=~/My Claude: Command not found.` fish is not installed here.
- **Not run in PowerShell:** neither `pwsh` nor `powershell` is installed on this Mac, so no
  PowerShell command ran here; CI's Windows jobs run the Rust tests, which hold the PowerShell
  form as text, and run no PowerShell either.
- **Windows:** the tests that run there build every expected path from its parts
  (`MAIN_SEPARATOR`, `MAIN_SEPARATOR_STR`); the `sh` test is `cfg(unix)`, with its two imports
  inside it; the provider test expects `"$HOME\.claude-work"` on Windows, where the harness's
  folder is in its home folder, and `~/'.claude-work'` elsewhere. No Windows target is installed
  here, so Windows is CI's to build.
- **Tests,** each new one failing first against a stub that kept the old command behind the new
  signature (in core, a stub that wrote the old single-quoted string); no stub was committed:
  - core: `a_place_in_the_home_folder_is_a_powershell_string_from_home`,
    `what_a_double_quoted_powershell_string_reads_is_written_after_a_backtick` and
    `a_place_outside_the_home_folder_is_a_single_quoted_powershell_string`, each string held to
    `tilde_in` leaving it as written. The stub failed them on the pass (`'/Users/ann/.claude-work'`
    came out `'~/.claude-work'`) and on the curly quotes it did not double.
  - Claude's provider: `the_fix_command_for_a_folder_in_the_home_folder_never_names_the_user` and
    `the_fix_command_for_a_folder_outside_the_home_folder_quotes_the_whole_folder` (both shells,
    a space, a quote, PowerShell's special characters, each command held to `tilde_in`);
    `doctor_shows_the_fix_command_as_written_in_both_shells` (doctor's text from a `Diagnostics`
    with one account, holding `signed_out`'s sentence and each shell's command as written); and,
    on Unix, `the_fix_command_gives_claude_code_the_folder_when_a_shell_runs_it`: the command from
    `sign_in_command` and as doctor's pass writes it, run by `/bin/sh -c` with the environment
    cleared, `HOME` set and `PATH` holding only a stand-in `claude` that prints the variable, for
    five folders in the home folder and one outside it, with the home folder `ann` and then
    `Ann Smith`, each giving the folder's own path. Against the stub the `sh` test got
    `~/My Claude` for `CLAUDE_CONFIG_DIR='~/My Claude' claude`.
  - Changed: `a_sign_in_from_another_folder_gets_that_folders_command` and
    `a_folders_sign_in_is_polled_with_its_token_and_gets_its_own_fix` expect the new forms, and
    failed on the old; removed: `the_fix_command_suits_the_shell_and_the_folder`, whose cases the
    two new command tests hold for both shells.
- **Counts:** the workspace 763 passed (1 ignored) on `main` at b144911 and 769 (1 ignored) now;
  the example build 760 (2 ignored) and 766 (2 ignored). By crate: core 135 to 141 (the three
  moved tests and three new ones), the daemon 112 to 109 (the three moved), Claude 138 to 141 (1
  ignored; four new, one removed); the CLI 67 (64, 1 ignored, in the example build), the API 59,
  Antigravity 64, Codex 63, cost 30, fake 19, Gemini CLI 29, issue 6, poll 37 and
  `quotadial-providers` 4 unchanged. The tray app's `--lib` tests 87 in both builds, before and
  after.
- **Checks run here:** `cargo fmt --check` in both workspaces; clippy with `-D warnings` and
  `--locked` on the workspace and the tray app, default and `fake` (the app's `fake` run with the
  example's CLI bundled by `npm run cli -- --fake`, then `npm run cli` to bundle the default
  build's again); the workspace's tests, default and `fake`; the tray app's `--lib` tests, default
  and `fake`, `tests/icon_press.rs` not run, as briefed; the release build's licence key test
  (`only_a_debug_build_trusts_the_test_key`), core having changed; the tray app's `npm run build`;
  the site's `npm run check`, `npm run build` and `npm test` (90), since it renders `CHANGELOG.md`
  and `docs/CLI.md`, whose built `/changelog` and `/cli` pages show the new text;
  `scripts/privacy.sh --check`, `scripts/public-prd.sh --check`, the 0.2.0 release notes, the
  release scripts' tests (39, 1 skipped) and shellcheck on `scripts/`. **Not run:** a Windows or
  Linux build, which no target here makes (CI's); PowerShell, not installed; `cargo deny`, `cargo
  audit` and actionlint, not installed here (no dependency, lockfile or workflow changed); `npm
  ci` and `npm audit`, no npm dependency changed; the tray app's `npm test` and
  `npm run check:window`, its window being unchanged.
- **Found going down #52, left for the manager or a later PR:**
  - A form tcsh takes too, `env CLAUDE_CONFIG_DIR="$HOME"/'My Claude' claude`, gave the folder's
    path in sh, dash, bash, zsh, ksh and tcsh alike here (fish not tried), and holds no home
    folder path either. This PR keeps D-130's form, as briefed; switching is the manager's call.
  - Doctor's `[user]` pass replaces every whole word that is the user's name, the command
    included, so for a Mac user called `claude` doctor would show
    `CLAUDE_CONFIG_DIR=~/'.[user]-work' [user]`, and `~/.[user]` for `~/.claude` elsewhere in its
    text. Doctor's copy is for support to read (D-130); a later PR could keep the command's own
    words, such as `claude`, out of that pass.
- **Safety on this Mac:** every build, test and npm run had `QUOTADIAL_DATA_DIR`,
  `QUOTADIAL_FAKE_DATA_DIR`, `CLAUDE_CONFIG_DIR`, `CODEX_HOME` and `GEMINI_CLI_HOME` in the
  scratchpad and no token in its environment; the new `sh` test makes its home folders and
  stand-in in a temporary folder and runs with `PATH` holding the stand-in alone, so no other
  `claude` can run, and the shells by hand ran the same way in the scratchpad. No vendor CLI ran,
  no sign-in or keychain item was read, and nothing touched
  `~/Library/Application Support/quotadial` or `~/.claude`.

### M5 queue item 12, PR D: QuotaDial Example's figures for #52

- **The go-ahead:** Matt's, in Chat on 2026-10-25, for QuotaDial Example in his menu bar for the
  recording and the timings (#51). Measured on 2026-10-25 and 26 on his Mac: macOS 26.7.1, a
  Mac16,5 with 16 cores and 128 GB, busy with other work throughout (load averages from 10 to 64).
- **The build:** QuotaDial Example (D-117) built from `main` at 0bd845f (#78) gives the figures
  below. An earlier build at a5cda9e, the same app code as ddcb456, gives those in brackets, as a
  check. Every run used a new, empty data folder, and none touched Matt's QuotaDial, its data folder
  or its sign-ins: its process was the same before and after each run, and only read.
- **First run to the number** (#52: under 5 s; the PRD: 60 s): 702.8 and 370.2 ms, from the launch
  call to the menu bar item showing 91% (664.8, 290.9 and 343.4 ms, and 267.0, 485.3 and 317.9 ms
  in the recording's attempts). Most of the slowest was macOS starting the app: the process reported
  in after 430.5 ms, against about 100 ms in the others.
- **The popover** (#52: open and drawn under 100 ms from a warm daemon): drawn in a median of 42.1
  ms over five presses, 73.5, 41.9, 56.6, 42.1 and 42.0 ms (44.0 ms: 43.8, 55.5, 37.7, 44.0 and
  50.3; one first press alone, 90.9), from the press through accessibility to the popover's last
  change, as ScreenCaptureKit saw the example's windows and nothing else. Its window was listed on
  screen only after about 102 ms in four presses, later than its pixels: that time waits on the
  accessibility call returning, so it does not measure the popover.
- **The tray app idle with the popover closed** (#52: under 80 MB, judged on the footprint, D-123),
  over 60.1 minutes, 61 samples, the popover closed in all of them. The app's process, the tray with
  the daemon inside it: 5.88 s of CPU, 0.163% of one core (0.141%); footprint 22.4 MiB on average,
  25.6 at most (22.4, 24.3 at most); resident 79.4 MiB on average, 96.1 at most (82.6). Its
  helpers' footprints on average: Web Content 19.5 MiB, Networking 6.4 MiB, Graphics and Media 29.8
  MiB, and an audio helper, there for 19 minutes, 3.2 MiB. The Graphics and Media helper held about
  245 MiB for the first three minutes after the launch, then 10 to 21 MiB; the earlier build's
  started at 6.4 MiB, so two runs do not say whether that is the app or macOS. PR E's third reading
  settled it: the helper takes about 245 MiB while the window is drawn and gives it back within 10
  seconds (#91's run).
- **The daemon alone** (#52: under 0.5% of one core averaged over an hour, and under 40 MB, judged
  on its footprint, D-123): the example build's `quotadial daemon`, as the command-line install runs
  it, with a home and data folder of its own, on a port the system chose, listening only on
  127.0.0.1. Over 60.0 minutes, 0.07 s of CPU, 0.002% of one core (0.002%); footprint 4.0 MiB on
  average, 3.7 to 4.1 (3.8, 3.5 to 4.0); resident 11.0 MiB (10.9).
- **Growth:** the daemon's footprint rose steadily in every run: 3.7 to 4.1 MiB in its hour, and
  3.5 to 4.3 MiB over 2 hours 25 minutes in a longer run of the earlier build, whose heap's
  allocated bytes went from 527 KiB to 968 KiB (`vmmap --summary`). SQLite's page cache, about 2 MB
  per connection by default, grows like that and then stops. Matt's copy's tray process, read only:
  39.0, 39.9 and 37.7 MiB of footprint at 21:07, 00:01 and 03:33 UTC, after 1 day 20 hours to 2 days
  2 hours of running, so no growth is seen there.
- **The ten-hour run** of this build, from 02:32 to 12:32 UTC, sampled every 5 minutes: 1.05 s of
  CPU, 0.003% of one core; footprint 5.1 MiB on average, 3.7 to 6.4 MiB; resident 11.6 MiB. The
  footprint rose the whole time, by about a quarter of a MiB an hour, and the heap's allocated
  bytes with it, from 973 KiB at 1 hour 51 minutes to 2,835 KiB at 9 hours 51 minutes. `heap` at
  4 hours 20 minutes found 227 blocks of the 5 KB size, SQLite's page buffers, about 1.1 MB of
  the 1.6 MB in use; the history database ended at 1,974,272 bytes, about the size at which
  SQLite's default cache stops taking pages. So, still rising at ten hours, the manager's rule on
  #80 puts it in M6's queue (its row there) rather than in this PR. A 24-hour run of the same
  build, from 06:53 UTC with the heap at 538 KiB, says whether it levels off.
- **The recording** is PR E, by the manager's split on #80, since it was not in when the ten-hour
  run reported: Matt's menu bar hides itself, and the recording needs it set to stay visible for
  about two minutes. Its run is the third for the Graphics and Media helper's start. A frame from a
  test recording of the earlier build showed its accounts as "fake Example", which is why the
  figures were taken again from a build with #77 and #78 in it, which names them "Example" and
  offers no week's cost.
- **The scripts** (kept outside the repository with the builder's notes, as above): a long run's
  summary can now be written again from its saved samples, after one failed on an awk slip with the
  hour's samples intact, and the idle summary words #52's targets as D-123 does.

### M6: the `chmod 600` command in `config.json`'s warning works as written (D-130)

- **The ask:** the coordinator's brief of 2026-10-26: the command `config.json`'s warning tells a
  user to run works when pasted into the user's shell as written, with a space in the place and `~`
  for the home folder, which D-129 left for a later PR (its first "Found going down #52" item).
  Built on `fix/chmod-command` from `main` at 6063ad1 (#83), in review as #84.
- **The command** (`crates/daemon/src/config.rs`): `shell_word(place, home)`, a private helper with
  a doc comment, called where the warning is made. In the home folder, `~/` unquoted and the rest in
  single quotes; outside it, or with no home folder or one with no parent (a whole disk, as
  `text::tilde_in` takes it), the whole place in single quotes; a single quote as `'\''`; always
  quoted. In the home folder by `Path::strip_prefix`, whole names, as `text::tilde` finds it. On
  macOS the warning's command was
  `chmod 600 ~/Library/Application Support/quotadial/config.json`, which chmod ran on
  `~/Library/Application` and `Support/quotadial/config.json`, and is now
  `chmod 600 ~/'Library/Application Support/quotadial/config.json'`. By hand on this Mac the form
  worked in sh, dash, bash, zsh, ksh and tcsh, on a file in the scratchpad with a space and a quote
  in its folder's name (fish is not installed here).
- **The pass:** the warnings' one `text::tilde_in` loop is unchanged; its comment now says why it
  leaves the command as `shell_word` wrote it, and points to D-130's one exception, a place whose
  text holds the home folder's path beside a space, a quote or the like. A scratch program against
  the real `tilde_in` (not committed) showed it: `'/Users/ann (old)/quotadial/config.json'` comes
  out `'~ (old)/quotadial/config.json'`, and `~/'Backup (old)/Users/ann/quotadial/config.json'`
  comes out `~/'Backup (old)~/quotadial/config.json'`, while macOS's command and a place in
  `/Users/anna` come out as they went in. The tests' `word` helper holds every word they make to
  `tilde_in` leaving it as written, on every system.
- **Windows:** `readable_by_others` reads the file's mode only under `cfg(unix)` and says no
  elsewhere, so the warning and its command never occur on Windows (D-130). `shell_word` and its
  three tests still build and run there: their expected places are built from their parts with
  `MAIN_SEPARATOR`, and the two tests that set a file's mode or run `sh` are `cfg(unix)`, with their
  imports inside them. No Windows target is installed here, so Windows is CI's to build.
- **Tests** (`config.rs`): new, `a_place_in_the_home_folder_is_quoted_after_a_bare_tilde`,
  `a_place_outside_the_home_folder_is_quoted_whole`,
  `a_single_quote_in_a_place_is_written_as_a_shell_reads_it` and, on Unix,
  `the_warnings_chmod_command_works_as_written_in_a_shell`: the command taken from the warning
  `Config::loaded` makes, run by `sh -c` with `HOME` set to the test's home folder and the
  temporary folder as its working folder, on files in `ann/Library/Application Support/quotadial`,
  `ann/Ann's QuotaDial` and, outside the home folder, `O'Brien's data/quotadial`, each then 0600
  with its warning gone. Changed: `a_file_with_webhooks_that_others_can_read_gets_a_warning`, now in
  macOS's `Library/Application Support`, with the command's own expected text. The new tests and the
  changed one all failed first against a stub that wrote the place as before, which chmod split at
  the space (".../ann/Library/Application: No such file or directory" and
  "Support/quotadial/config.json: No such file or directory"); all but the outside test, which is
  right to pass it, failed against a second stub that quoted the whole place, which the pass then
  wrote `'~/Library/...'`, a place chmod could not find. Neither stub was committed.
- **Counts:** in the workspace, 759 passed (1 ignored) on `main` at 6063ad1 and 763 (1 ignored)
  now; the example build 756 (2 ignored) and 760 (2 ignored). By crate: the daemon 108 to 112, the
  four new tests (the test of a file others can read is changed, not added); the CLI 67 (64, 1
  ignored, in the example build), the API 59, core 135, Antigravity 64, Claude 138 (and 1
  ignored), Codex 63, cost 30, fake 19, Gemini CLI 29, issue 6, poll 37 and `quotadial-providers` 4
  unchanged. The tray app's `--lib` tests 87 in both builds, unchanged.
- **Checks run here:** `cargo fmt --check` in both workspaces; clippy with `-D warnings` and
  `--locked` on the workspace and the tray app, default and `fake`; the workspace's tests, default
  and `fake`; the tray app's `--lib` tests, default and `fake`, the latter with the example's CLI
  bundled (`npm run cli -- --fake`, then `npm run cli` to bundle the default build's again), and
  `tests/icon_press.rs` not run, as briefed; the tray app's `npm run build`; the site's
  `npm run check`, `npm run build` and `npm test` (90), since it renders `CHANGELOG.md`, whose built
  `/changelog` page shows the new line; `scripts/privacy.sh --check`,
  `scripts/public-prd.sh --check`, the 0.2.0 release notes, the release scripts' tests (39, 1
  skipped for GNU sha256sum) and shellcheck on `scripts/`. **Not run:** a Windows or Linux build,
  which no target here makes (CI's); `cargo deny`, `cargo audit` and actionlint, not installed here
  (no dependency, lockfile or workflow changed); `npm audit`, no npm dependency changed; the tray
  app's `npm test` and `npm run check:window`, its window being unchanged; and the release-only
  licence key test, which nothing here reaches.
- **Found going down #52, left for a later PR:** Claude's sign-in command, `claude_in`, writes a
  folder that needs quotes as the whole place, `CLAUDE_CONFIG_DIR='/Users/ann/My Claude' claude`,
  with the user's name in it, and doctor's pass then writes `~` inside the quotes,
  `'~/My Claude'`, which no shell expands (the scratch program showed it). D-130's form would mend
  both: `CLAUDE_CONFIG_DIR=~/'My Claude' claude` works in sh, dash, bash and zsh, which expand `~`
  after an assignment's `=`.
- **Safety on this Mac:** every build, test and npm run had `QUOTADIAL_DATA_DIR`,
  `QUOTADIAL_FAKE_DATA_DIR`, `CLAUDE_CONFIG_DIR`, `CODEX_HOME` and `GEMINI_CLI_HOME` in the
  scratchpad and no token in its environment; the new tests make their own home and data folders in
  a temporary folder and run `sh` only there, and the shells by hand ran on files in the scratchpad
  with `HOME` set there. No vendor CLI ran, no sign-in or keychain item was read, and nothing
  touched `~/Library/Application Support/quotadial`.

### M6: the home folder written `~` in QuotaDial's own messages (D-129)

- **The ask:** what D-128 left, QuotaDial's own messages naming its data folder and `config.json`'s
  warnings, with the manager's two notes on #82 (2026-10-26, 07:27 UTC): a property test of
  `text::tilde_in`, which takes a server's words, and doctor through it, so the rule has one
  implementation. Built on `fix/home-folder-in-messages` from `main` at 52265b6 (#82), in review as
  #83.
- **One pass where each message is made** (D-129), with the process's home folder, as
  `ProviderError::warning` does for a provider's error. Each place that makes the text takes the
  home folder as an argument, which its tests give and its callers read from the process:
  - `config.json`'s three warnings, in `Config::load`, through `Config::loaded(dir, home)`
    (`crates/daemon/src/config.rs`);
  - every history error, through `HistoryError::said`, which `new`, SQLite's errors and the page
    check use, and `made(message, damaged, home)` (`crates/daemon/src/history.rs`). SQLite's own
    words for a file it cannot open name the file ("unable to open database file: ..."), which the
    test takes from a real `Connection::open`;
  - the daemon's start errors, in `HostError::line(home)`, which `Display` calls
    (`crates/api/src/host.rs`);
  - why a running daemon gave no answer, in `Unanswered::new(said, home)`; `status` and `headroom`
    read the home folder once and hand it through `get` and `announced`
    (`crates/api/src/client.rs`);
  - the CLI's own, in `history_in(dir, home)`, which `open_history` calls, and
    `started(asking, dir, port, home)`, `quotadial daemon`'s first two lines
    (`crates/cli/src/lib.rs`). `fix_for` now takes the folder whose history it notes as damaged,
    the one `history_in` opened, which in use is the same folder, so a test of a damaged history
    could never note it in the real data folder;
  - the app's own, `history::export(..., home)`, with Tauri's `home_dir()`, read once in
    `export_history` for the error and the "Saved to ..." line (`apps/tray/src-tauri`).
- **Doctor** (`crates/core/src/diagnostics.rs`): `Diagnostics::text` writes its whole text through
  `tilde_in`; its plain replace is gone. Its new test failed first against the replace, which wrote
  `/Users/anna/.codex/auth.json` as `~a/.codex/auth.json` and `/Volumes/Backup/Users/ann/...` as
  `/Volumes/Backup~/...`. On Windows it writes `~\data`, as its replace did and #57 wanted, now by
  the rule every message follows (next).
- **The separator** (the manager's decision on #83, 2026-10-26, 09:22 UTC, asked for in the first
  round, where doctor would have written `~/data` on Windows): the home folder is written `~` and
  then the system's own separator, so `~\AppData\Roaming\quotadial\history.db` on Windows and
  `~/.codex/auth.json` elsewhere. `tilde` writes `std::path::MAIN_SEPARATOR` after `~`; `tilde_in`
  writes the separator it found after the home folder, which for a path the system wrote is the
  same, so a path written with `/` on Windows keeps it (an assertion in the Windows home folder's
  test holds that, and only Windows can fail it). Before, both wrote `~/`, so Windows showed
  `~/quotadial\history.db`. Every test that wrote `~/` into an expectation of either function now
  builds it as `Path::new("~").join(...)`, the platform's separator with no `cfg(windows)` branch:
  in core `text.rs`, `report.rs` and `diagnostics.rs`; in the daemon `config.rs`, `history.rs` (the
  recovery line's test too) and `poller.rs`; in the API `host.rs`, `diagnostics.rs` and `tests.rs`;
  in the CLI `lib.rs` and `tests/status.rs` (Gemini CLI's sign-in and doctor's data folder too); the
  tray's `history.rs`; Claude's advice tests (a folder's place and a file's); and Gemini CLI's
  `in_home`. To find them all, the whole workspace's and the app's tests ran once on this Mac with
  both functions made to write `~\`: 33 failed, these 30 and three that pass on Windows as they are,
  doctor's two whose places are written with `/` and so keep it, and Claude's command test, whose
  Unix command is checked off Windows only. The window check's fixtures and the site show no Windows
  path, and Antigravity's settings place is written as its docs write it, not by `tilde`, so they
  stay.
- **The property test** (`crates/core/src/text.rs`, beside `unplain_words`'s test): for any text and
  any home folder or none, with short home folders of a path's characters mixed in, `tilde_in` never
  panics and never makes the text longer, and with no home folder or a whole disk leaves it as it
  is; for a home folder as a system names one (`/` or `C:\` and one to three names), written again
  is written once, a place in it after a space is written as `tilde` writes it with the text before
  it as it would be alone, and a place beside it whose name only starts as the home folder's, or
  around its name, stays as it is. Each property failed first against a stub: one that cut a
  character in two (the first, shrunk to the home folder `೦`), the plain replace (the fourth), one
  that left every text as it was (the third), one that took a path to start anywhere (the second
  and the fourth), and one that wrote the home folder itself as `~/` (the first and the third; the
  first only once its strategy had short home folders, which it was given for that). 100,000 cases
  of each passed on the real code; the stubs' failure seeds were removed, not committed.
- **Found going down #52, left for a later PR:**
  - `config.json`'s warning for a file others can read says to run `chmod 600 <the file>`,
    unquoted, so on macOS, where the data folder is in `Application Support`, the command as written
    stops at the space. Quoting it needs care, since a quoted `~` is not expanded; this PR writes
    the home folder `~` and leaves the command as it was.
  - A Claude sign-in in a file whose account Anthropic has not named yet is called by the file's
    full path (`LoginSource::File`'s stand-in) in `status`, the app and the API until an answer
    names it: an account's name, not a message, so not this pass.
  - Antigravity's `status-line` writes "quotadial: <the command after it> could not be started:
    ..." on standard error, as a shell does, with the command as written; Antigravity CLI does not
    show it.
- **Tests:** in the workspace, 745 passed (1 ignored) on `main` at 52265b6 and 759 (1 ignored) now;
  the example build 742 (2 ignored) and 756 (2 ignored). By crate: core 130 to 135 (`tilde_in`'s
  four properties and doctor's `/Users/anna`); the daemon 105 to 108 (`config.json`'s warnings with
  a home folder given, the history errors with one given and with this process's; the test of a
  file others can read now gives the home folder too); the API 56 to 59 (the start errors with a
  home folder given and with this process's, and why a daemon gave no answer); the CLI 64 to 67
  (61 to 64, 1 ignored, in the example build): `history_in`, `started`, and `alerts` run on a
  history it cannot open in a data folder in the home folder and beside it, with the daemon's two
  first lines and a refused token's message now held whole; Antigravity 64, Claude 138 (and 1
  ignored), Codex 63, cost 30, fake 19, Gemini CLI 29, issue 6, poll 37 and `quotadial-providers` 4
  unchanged. The tray app 86 to 87 in both builds, the export. Every new test failed first: the
  properties against the stubs above, doctor's against its plain replace, the rest against a stub
  that left their text as it was, and the tests of the built binaries with the CLI's and the
  client's passes undone. The API's and the CLI's doctor tests, and the test of a file others can
  read, are changed for the new text. After the manager's review on #83 the counts are the same,
  every test passing: the separator added no test, only the assertion above.
- **Checks run here:** `cargo fmt --check` in both workspaces; clippy with `-D warnings` and
  `--locked` on the workspace and the tray app, default and `fake`; the workspace's tests, default
  and `fake`; the tray app's `--lib` tests, default and `fake` (`tests/icon_press.rs` not run, as
  briefed); the tray app's `npm run build` (34.0 KB of 150, contrast), `npm test` (5) and
  `npm run check:window` (4371 checks over 16 views, headless Chrome with a profile of its own,
  removed after); the site's `npm run check`, `npm run build` and `npm test` (90), since it renders
  `CHANGELOG.md` and `docs/CLI.md`; `scripts/privacy.sh --check`, `scripts/public-prd.sh --check`,
  shellcheck on `scripts/`, actionlint 1.7.12 on every workflow, the release scripts' tests (39, 1
  skipped for GNU sha256sum) and the 0.2.0 release notes; and `scripts/no-example-provider.sh` on
  the example build's CLI, bundled by `npm run cli -- --fake` (it fails, naming `status --json` and
  doctor), then, once `npm run cli` had bundled the default build's CLI again, on that and on the
  workspace's CLI (both pass). By hand, in scratch homes and data folders: `quotadial alerts` says
  "could not open the history at ~/data/history.db: Is a directory (os error 21)" where a folder
  stands in the history's place, and, with a `config.json` holding `history_days` 0 and an `http`
  webhook, "warning: history_days in ~/data/config.json ..." and "warning: A webhook in
  ~/data/config.json ...". **Not run:** `cargo deny` and `cargo audit`, not installed here (no
  dependency or lockfile changed); the release-only licence key test, which no change here reaches;
  and a release build of the CLI or the app. After the review, every check above ran again with the
  same results, the site's because `docs/CLI.md` changed: its built `/cli` page shows `~\` in each
  code span that names Windows.
- **Safety on this Mac:** every build, test and npm run had `QUOTADIAL_DATA_DIR`,
  `QUOTADIAL_FAKE_DATA_DIR`, `CLAUDE_CONFIG_DIR`, `CODEX_HOME` and `GEMINI_CLI_HOME` in the
  scratchpad and no token in its environment; every test made up its own home and data folder, and
  the tests that take this process's home folder only build a path's text from it; the built CLIs
  ran only in scratch homes, the runs by hand and the example-provider check included, where they
  found no sign-in, as the empty-home tests' empty reports show. No vendor CLI ran, no sign-in or
  keychain item was read, and no app or daemon ran against Matt's data.

### M6: error lines named through the report (D-128)

- **The decision:** the manager's item 3 on #73 (2026-10-26, 01:26 UTC): a provider's error is named
  through the report, `StatusReport::provider_name(id)`, where the CLI and the app render it, not
  in each provider, by P-020's rule 1. 1c found three places (D-125), so it is this PR, on
  `fix/provider-error-names` from `main` at 1789d07 (#79), in review as #82.
- **Where the id goes:** out of the message (D-128). `quotadial_core::Error`'s text no longer
  begins with `{provider}: `, and `ProviderError::warning`, which `new` uses, takes a leading
  `<id>: ` off whatever a provider writes; Claude's two discovery warnings no longer write it. The
  API's `errors[].message` is the sentence alone: `errors[].provider` is the id, and the report's
  `providers` the name.
- **One helper** (`crates/core/src/report.rs`): `StatusReport::error_lines()`, "{the provider's own
  name}: {what happened and what to do}", the name from `provider_name`. `status` and `watch` print
  it after `error: ` (`write_text`), doctor under Problems (`Diagnostics::text`), and the app among
  its problems (`Meter::new`), which the popover, the tray's menu and its tooltip show. A provider
  the report does not list keeps its id. An older daemon's message, which began with its id, loses
  exactly `<id>: `, so the provider shows once: by its name where the report lists providers, by its
  id from 0.2.0, which lists none.
- **#52's plain words:** "could not read credentials from" is "could not read the sign-in from".
  Claude's Keychain location gained "the", and Claude's and the daemon's lookups that stop
  unexpectedly read the sign-in "from this machine", so each sentence reads after "from". A test
  over one of each `Error` variant holds its text to `unplain_words` and to not beginning with its
  provider, and its match makes a new variant join it. The README's "these credentials" is "these
  sign-ins". Going down the repository's text found no other "quota", "auth" or "credential"
  outside a name in anything a user reads, but in `docs/PRD-public.md`, which is the manager's.
- **Docs:** `docs/openapi.json` (`ProviderError`'s two fields and `providers`), `docs/CLI.md`
  (the error line, `errors[]` and its JSON example, `providers[]`, doctor),
  `skills/quotadial/SKILL.md` and the MCP tools `get_usage` and `get_account_headroom`; D-128; one
  changelog line.
- **The home folder written `~`** (#52, no path with a username in it; the coordinator's word on
  this PR, the same concern): a sign-in or cache error's location was a full path. One pass where
  a provider's error text is made, `ProviderError::warning`, which `new` uses, now writes the home
  folder `~` after `redact_secrets`, through `text::tilde_in`, `text::tilde`'s rule for any text:
  a path that starts in the home folder becomes `~/` and the rest as written, or `~` for the folder
  itself; a path starts at the start of the text or after a character no path holds, and is in the
  home folder only where the folder's name ends, so with `/Users/ann` as the home, `/Users/anna`
  and `/Volumes/Backup/Users/ann` stay as they are. The home folder is the process's own,
  `std::env::home_dir()`, the one the providers in the CLI and in the daemon build these paths
  from, and doctor and the daemon's recovery line already use; the two do not find it differently.
  So every provider's error is written so in the report, the API, `status`, doctor and the app.
  `docs/openapi.json`, `docs/CLI.md`, the skill and the changelog line say so.
- **Found going down #52, left for a later PR:** a place outside the home folder is written as it
  is, so an error still holds a user's name where a variable or setting puts a provider's folder
  there (`CLAUDE_CONFIG_DIR`, `CODEX_HOME`, `extra_config_dirs`, `XDG_CACHE_HOME`, the example
  build's `QUOTADIAL_FAKE_ACCOUNTS`), where Windows redirects AppData, or where a path reaches the
  home folder by another spelling (D-128); a daemon older than this sends full paths until it is
  updated; QuotaDial's own messages naming its data folder, such as "could not open the history
  at ...", and config.json's warnings are not a provider's error and still write a place in full;
  doctor's own plain replace of the home folder would also cut into `/Users/anna`; and
  `/v1/headroom` sends no `providers`, so an agent names its errors from `/v1/status`'s.
- **#81 (Copilot), built before this:** its crate's two discovery warnings begin "copilot: ", and
  its tests hold the old `Error` text, its id in front and the old words for a sign-in. Whichever
  of the two merges second drops the prefix from those warnings (the report drops it anyway) and
  moves those tests to the new text. #81's queued row "error lines named through the report" is
  this PR's row in the M6 table.
- **Tests:** in the workspace, 725 passed (1 ignored) on `main` at 1789d07 and 745 (1 ignored) now;
  the example build 722 (2 ignored) and 742 (2 ignored). By crate, from `main`: core 118 to 130
  (the helper's two, doctor's, the report keeping a message without the id, `Error`'s three: plain
  words, not beginning with its provider, the sign-in sentence; and the home folder's five: a place
  in it in any text, places outside it, a Windows home folder, and the report's message with a
  home folder given and with this process's own); API 54 to 56 (`/v1/status`'s and
  `/v1/headroom`'s JSON, and the MCP tools' text); the CLI 62 to 64, `write_text` and a place in the
  home folder in `collect`'s report and `status` (59 to 61, 1 ignored, in the example build); the
  daemon 103 to 105, a lookup that stops and the home folder in its report; Claude 136 to 138 (and
  1 ignored), its warnings and its stopped lookup; Antigravity 64, Codex 63, fake 19, Gemini CLI 29,
  cost 30, issue 6, poll 37 and `quotadial-providers` 4 unchanged in number, their error texts moved
  to the new words, and the example provider's test of an unusable file expecting its path as
  `text::tilde` writes it, since Windows keeps temporary folders in the home folder. The tray app
  85 to 86 in both builds, the problems list; its scripts' tests 5; the site 90; the window check
  4371 checks over 16 views, as before. Every new test failed first: core's helper against a stub
  that gave each message as it is, the CLI's and the app's before they used the helper, the
  message's nine (eight new, and the Keychain test's new words) before its text changed, the MCP
  tools' on their old text, the home folder's six against a stub that left the text as it was, and
  its test of places outside the home folder against a plain replace, which wrote `/Users/anna` as
  `~a`.
- **Checks run here:** `cargo fmt --check` in both workspaces; clippy with `-D warnings` and
  `--locked` on the workspace and the tray app, default and `fake`; the workspace's tests, default
  and `fake`, with the OpenAPI and packaging checks in them; the tray app's tests, default and
  `fake`, as CI runs them, so `tests/icon_press.rs` too, which put a status item in the menu bar for
  under a second each run and passed; the release-only licence key test; the tray app's `npm ci`,
  `npm run build` (34.0 KB of 150, contrast), `npm test`, `npm run check:window` (headless Chrome
  with a profile of its own, removed after), `npm run cli` and `npm run cli -- --fake`; the site's
  `npm ci`, wrangler's start, `npm run check`, `npm run build` and `npm test`;
  `scripts/privacy.sh` (no change) and `--check`, `scripts/public-prd.sh --check`, shellcheck on
  `scripts/`, actionlint 1.7.12 (its release tarball checked against the release's checksums) on
  every workflow, the release scripts' tests (39, 1 skipped for GNU sha256sum) and the 0.2.0
  release notes; and `scripts/no-example-provider.sh` on the CLI and the app's bundled CLI (both
  pass) and the example CLI (it fails, naming both commands), the bundled CLI built again after. By
  hand, in a scratch home with a made-up `auth.json` that is not JSON: `status` says "error: Codex:
  could not read the sign-in from ...", doctor's Problems "Codex: could not read the sign-in from
  ~/.codex/auth.json: ...", and `status --json`'s message leaves the provider out. After the home
  folder pass: `cargo fmt --check` in both workspaces; clippy with `-D warnings` on the workspace
  and the tray app, default and `fake`; the workspace's tests, default and `fake`; the tray app's
  tests, default and `fake`, with `--lib` only from here on, so `tests/icon_press.rs` no longer
  runs; the site's `npm run check`, `npm run build` and `npm test`; `scripts/privacy.sh --check`
  and the 0.2.0 release notes; and by hand, in the same scratch home, `status`, `status --json` and
  doctor each write "could not read the sign-in from ~/.codex/auth.json". **Not run:** `cargo deny`
  and `cargo audit`, not installed here (no dependency or lockfile changed); a release build of the
  CLI or the app (the release check ran on debug builds of the same code); and, after the home
  folder pass, the window check and the release check, which it does not touch.
- **Safety on this Mac:** every cargo run after the first had QuotaDial's data folders and every
  sign-in folder in the scratchpad and no token in its environment; the first, the count on `main`,
  ran the tests as they are, each making up its own home and data folder or faking the Keychain.
  The release check and the runs by hand used scratch homes, where `security` sees no login
  keychain. No vendor CLI ran, no sign-in or keychain item was read, and no app or daemon ran
  against Matt's data.

### M6 PR 3: Antigravity through its CLI's status line (D-126, P-022, P-023)

- **The decisions:** the manager's (b) on #73 (2026-10-25, 20:29 UTC; D-119) and its answers of
  2026-10-26, 02:25 UTC: the design stands, with an amount-only limit named in the account's line
  or sign-in sentence, P-023 deferred to #51's live check, the folder's path in one constant, the
  file written before a chained command, QuotaDial's failures shown nowhere, disabled limits left
  out with a line in the module's docs, and one sentence in `docs/PRIVACY.md`. P-022 is accepted.
  The branch is `feat/antigravity`, from 1b's a7dc15f, moved onto `main` at 0bd845f (#78) with
  `git rebase --onto origin/main a7dc15f` before any commit; in review as #79.
- **The crate** (`crates/providers/antigravity`): `handover` (one reader for Antigravity CLI's
  JSON and QuotaDial's file, which keeps only each limit's `remaining_fraction` or
  `remaining_amount`, `reset_time`, `reset_in_seconds` and `disabled`, the `plan_tier`, the `email`
  and `written_at`, each checked, at most 32 limits), `store` (`antigravity.json`, 0600, a
  temporary file of the process's own and a rename, read only as a plain file of at most a
  megabyte, handed over at the earlier of its recorded time and its modified time), `chain` (the
  status line already there, run as a program and its arguments with the same input, its output
  and status unchanged, 127 or 126 as a shell ends, stopped at 4.5 s), `command` (keep first, then
  chain; nothing of its own printed), `windows` (CodexBar's names for the limits; disabled and reset
  since left out; amounts named, never windows) and `advice` (every sentence, times as RFC 3339 so
  a line stays the same while its numbers do). No HTTP client, which the P-021 test reads.
- **The account:** the file's address, in lower case, as its id, or `antigravity-cli`; before any
  handover, the `antigravity-cli` account, `unsupported`, when `~/.gemini/antigravity-cli` is a
  folder, opening nothing in it. `name()` is "Antigravity". The block to add is in its
  `sign_in_use` and in the line before any numbers.
- **On:** `antigravity` in `quotadial-providers`' default features, after Gemini CLI, its data
  folder from `DataDir::from_env()` in `with_config`; D-117's list test names it; the app's
  lockfile gains only the new crate. The CLI's help and quotadial(1) name it, `history --provider`
  takes `antigravity`, and `packaging/` gains `quotadial-antigravity(1)` and
  `quotadial-antigravity-status-line(1)`. The daemon's and `watch`'s line ends "and no one about
  Gemini CLI or Antigravity" (P-021).
- **Docs:** `docs/PRIVACY.md` (one sentence) and the README's copy; the README's opening, provider
  table and a paragraph; `docs/INSTALL.md`'s new Antigravity section with the block, chaining onto
  a status line already there and the `PATH` it needs; `docs/CLI.md`'s new "Antigravity's status
  line" section, the account, the source, MCP's `provider` and the exit codes, and the review of
  #78's two notes (the two lines past 100 columns rewrapped, and OpenAI for ChatGPT where it says
  whom QuotaDial asks, in the README too); `docs/SECURITY.md`; the skill; `docs/openapi.json`; one
  changelog line. The site's guide test lists the two new sections; Antigravity's docs page is named
  in words, not linked, so no list of addresses gains `antigravity.google`. The window check's
  fixtures gain an Antigravity account whose panel carries the block and an amount-only limit, the
  longest text the popover shows, checked at 200% zoom.
- **Tests:** in the workspace, 659 passed (1 ignored) on `main` at 0bd845f, as 1c recorded it, and
  725 (1 ignored) now; the example build 658 (2 ignored) and 722 (2 ignored). By crate, from
  `main`: `quotadial-provider-antigravity` 64 (handover 11 with its property test, windows 7, store
  7, chain 7, command 10, advice 5, the provider 17); the CLI 60 to 62, two real-binary tests of the
  command, `status`, `status --json` and `doctor`, and of a chained status line (59, 1 ignored, in
  the example build, unchanged); core 118 and `quotadial-providers` 4, unchanged in number, with the
  source's value and words and the list's fourth provider asserted in their tests; daemon 103, API
  54, cost 30, issue 6, poll 37, Claude 136 (and 1 ignored), Codex 63, fake 19 and Gemini CLI 29
  unchanged. The tray app 85 in both builds, its scripts' tests 5, the site 90, the window check
  4371 checks over 16 views (3394 before, with the new account). Every new test failed first: the
  core assertions and the list test against a stub or the old list, the CLI's two with the
  `antigravity` feature off, the crate's first 63 under 77 mutations, each restored after, which
  between them failed every one of them, and its Debug test against the Debug that named the file;
  the one mutation that survived, a megabyte cap removed from the file's reader, survives as it
  should, since the parser's own cap holds.
- **Checks run here:** `cargo fmt --check` in both workspaces; clippy with `-D warnings` on the
  workspace, default and `fake`, and on the tray app, default and `fake`, with `--locked`; the
  workspace's tests, default and `fake`, with the packaging and OpenAPI checks in them; the
  release-only licence key test; `cargo metadata --locked` for both lockfiles; the tray app's `npm
  ci`, `npm run build` (34.0 KB of 150, contrast), `npm test`, `npm run cli` and `npm run cli --
  --fake`, its tests (`--lib`), default and `fake`, and `npm run check:window` in headless Chrome;
  the site's `npm ci`, `npm run check`, `npm run build` and `npm test`; `scripts/privacy.sh` and
  `--check`, `scripts/public-prd.sh --check`, shellcheck on `scripts/`, actionlint 1.7.12 (fetched
  into the scratch folder and checked against its release's checksums) on every workflow, and
  `scripts/no-example-provider.sh` on the CLI and the app's bundled CLI (both pass) and the example
  CLI (it fails, naming both commands). By hand, in a scratch home: the command's help, the line
  before any numbers, a handover in 0.01 s into a 0600 file, then `status` and `doctor`.
  **Not run:** the tray's `tests/icon_press.rs`, which puts a real status item in Matt's menu bar;
  `cargo deny` and `cargo audit`, not installed here (no third-party crate was added); a release
  build of the app; and a real Antigravity CLI, which is #51's check.
- **Safety on this Mac:** every cargo and npm run had `QUOTADIAL_DATA_DIR`,
  `QUOTADIAL_FAKE_DATA_DIR`, `CLAUDE_CONFIG_DIR`, `CODEX_HOME` and `GEMINI_CLI_HOME` in the
  scratchpad and no token in its environment; every test made up its own home, data folder and
  Antigravity folder; the release check and the runs by hand used scratch homes. No vendor CLI ran,
  `agy` included; no `~/.gemini` file, keychain item or sign-in was read; no app or daemon ran
  against Matt's data. Antigravity's facts come from its public docs and CodexBar's source.

### M6 PR 1c: whom QuotaDial asks, and no cost offer where nothing could fill it (P-021, D-125)

- **The decision:** the manager's on #73 (2026-10-26, 01:26 UTC), on what #77 left open: P-021
  accepted as proposed, with rules (a) to (d), in a small PR of its own before PR 3; the cost offer
  fixed there as its own commit; error lines named through the report if that is one place. The
  branch is `feat/provider-asks`, from `main` at f67a7c1 (#77), in review as #78.
- **P-021** (`crates/core/src/provider.rs`, each provider): `Provider::asks()`, required with no
  default, the company a provider's `fetch` sends usage requests to, never a host: "Anthropic" for
  Claude, "OpenAI" for Codex, none for Gemini CLI and the fake. `docs/PRIVACY.md` (and the README's
  copy) now says "**api.anthropic.com**, Anthropic's" and "**chatgpt.com**, OpenAI's", so rule (a)'s
  words are in the privacy text, beside their hosts; the app's Help test and the window check's
  fixtures follow.
- **The test** (`crates/providers/all`, rule (b)): for every provider a build lists, one that asks a
  company has a host in `privacy::HOSTS` whose line in the privacy text names it, and one that asks
  no one has none of the HTTP clients #75 listed in its manifest, `crates/providers/<its id>`. The
  default build checks Claude, Codex and Gemini CLI, the example build the fake. Gemini CLI's own
  manifest test moved into it. A client is found written as a line (`reqwest.workspace = true`) or
  as a section header whose last dotted part names it (`[dependencies.reqwest]`,
  `[target.'cfg(unix)'.dependencies.reqwest]`), held by a test over literal manifest lines, which
  failed on the header form before it was read; only the provider's own manifest is read, so a
  client reached through another crate is not seen (core has none today). Each direction failed under a mutation: Claude saying it asks no one
  ("claude asks no one, but depends on reqwest"), Gemini CLI saying it asks Google.
- **The line** (rule (c)): `asking()` in the CLI, one sentence: "It asks Anthropic at most every 5
  min and OpenAI at most every 2 min, and no one about Gemini CLI." The daemon says it when it starts
  and `watch` after each refresh; the example build says "It asks no one about Example." Its test
  covers a provider that asks, one that does not and both, each held to `unplain_words`; the
  binary tests hold both builds' lines. `docs/CLI.md` describes it.
- **The week's cost:** `licence::keeps_logs` (from `Provider::log_folders()`, no change to
  `Provider`). The daemon asks each provider once, when its poller is made, and offers the cost on
  Free only for an account of a provider that keeps logs; asking each round would wait on a
  provider's own lock while a scan holds it, as a test that holds the logs to keep a scan reading
  showed. The CLI's direct fetch follows the same rule, its "Pace needs the daemon" line names the
  cost only where the daemon would add one, and `quotadial daemon` says nothing of the cost when
  none of its providers keeps logs. Tests: a copy whose provider keeps no logs makes no offer (it
  failed first, with the offer); the pace line with and without the cost; the example build's
  daemon prints no cost line; a Gemini CLI-only home's `status` and `--json` carry no offer.
- **Error lines, left for a later PR:** a provider's error is shown in three places, `status` and
  `watch` (the CLI's `write_text`), doctor (`Diagnostics::text`) and the app's problems
  (`meter.rs`), and its `<id>: ` comes from `quotadial_core::Error`'s text when the report is made.
  Naming it through `StatusReport::provider_name` means each of the three cutting the id off the
  message and putting the name in its place, or the report keeping the message without it, which
  changes the API's `errors[].message`. More than one place, so not this PR.
- **The release check** (the manager's note on #77): it still fails on the word `Example` anywhere in
  `status` or `doctor` output; this PR does not touch it, so the match is to be narrowed to the
  provider's column when it next changes.
- **Tests:** in the workspace, 656 passed (1 ignored) on `main` and 659 (1 ignored) now; the example
  build 655 (2 ignored) and 658 (2 ignored). By crate, from `main`: the CLI 59 to 60, the daemon 102
  to 103, `quotadial-providers` 2 to 4, Gemini CLI 30 to 29 (its manifest test now the providers'
  rule); core 118, API 54, cost 30, issue 6, poll 37, Claude 136 (and 1 ignored), Codex 63 and fake
  19 unchanged. The tray app 85 in both builds, its scripts' tests 5, the site 90, the window check
  3394 checks over 16 views.
- **Checks run here:** `cargo fmt --check` in both workspaces; clippy with `-D warnings` on the
  workspace and the tray app, default and `fake`; the workspace's tests, default and `fake`, with
  the packaging and OpenAPI checks in them; the release-only licence key test; `cargo metadata
  --locked` for both lockfiles; the tray app's `npm ci`, `npm run build` (34.0 KB of 150, contrast),
  `npm test`, `npm run cli`, its tests (`--lib`), default and `fake`, and `npm run check:window`; the
  site's `npm ci`, wrangler's start, `npm run check`, `npm run build` and `npm test`; shellcheck,
  `scripts/public-prd.sh --check`, `scripts/privacy.sh --check`, the 0.2.0 release notes and the
  scripts' tests (39, 1 skipped for GNU sha256sum); actionlint on every workflow; and
  `scripts/no-example-provider.sh` on the release CLI and the app's bundled CLI (both pass) and the
  example CLI (it fails, naming both commands). **Not run:** the tray's `tests/icon_press.rs`, which
  puts a real status item in Matt's menu bar; `cargo deny` and `cargo audit`, not installed here (no
  crate was added); and a release build of the app.
- **Safety on this Mac:** as for 1b: every cargo run with QuotaDial's folders and every sign-in
  folder in the scratchpad and no token set; release CLIs only in empty homes, and the release check
  on them under `sandbox-exec`; the window check's Chrome headless with a profile of its own, removed
  after. No vendor CLI ran, no sign-in or keychain item was read, and no app or daemon ran against
  Matt's data.

### M6 PR 1b: providers name themselves, and Gemini CLI on (P-020, D-124)

- **The decision:** the manager accepted P-020 on #73 (2026-10-25, 22:25 UTC) as its own PR, with
  five rules, merged #75 with Gemini held off, and gave 1b the turn-on too, so D-121's "turned on in
  the PR that merges it" holds one PR later for Gemini (D-122). The P-020 commits first built on
  #75's branch were moved onto `main` with `git rebase --onto origin/main 0261b78`, then onto
  4d81a8a (#76); the branch is `feat/provider-names`, in review as #77.
- **Rule 1, the name:** `Provider::name()`, required, beside `id()`: "Claude", "Codex", "Gemini CLI",
  and "Example" for the example build. Each provider sets `Account.provider_name` from the same
  `NAME` on every account it builds. `ProviderName { id, name }` and `StatusReport.providers`, filled
  by `collect` and by the daemon's report from its providers before any account is found, name
  status pages, costs, incidents and doctor's status pages (`StatusReport::provider_name`, the id
  for a provider it does not list). `Rule::read_as(&[ProviderName])` replaces the rule's Display, so
  a rule narrowed to a provider reads back with the provider's own name in `quotadial alerts` and
  Settings. Free's line for held-back accounts takes `Provider::name`; MCP's `provider` argument is
  described from the names ("claude for Claude, codex for Codex, gemini for Gemini CLI"), where it
  kept words of its own for Claude and Codex; a provider's own commands are listed under its name.
  `text::provider_name` is gone, and the app's strings name no provider.
- **Rule 2, the sentence:** `Account.sign_in_use`, in the provider's words: Claude's and Codex's are
  the popover's old sentences (`text::sign_in_read_from`, or each one's fallback); Gemini CLI's say
  it only checks the sign-in file is there, or that it reads one field of the settings, which says
  how Gemini CLI signs in, and then, in a sentence of its own so it cannot be read as what the
  field says, "It never reads the key." ("It never reads a key or sign-in." for Vertex AI); the
  example's says it has no sign-in. `Snapshot::without_secrets` now blanks a token out
  of `sign_in` and `sign_in_use` too. The panel is "What QuotaDial does with this sign-in", and the
  site's two pictures of the accounts were captured again with it (`node scripts/app-pictures.mjs`;
  the settings pictures came out byte for byte the same, and the unused history pictures were left).
- **Rule 3, older data:** both fields are always sent and so required in `docs/openapi.json`; read
  back from before P-020 they take serde's empty default. `Account::provider_label` shows the id
  until the next poll, doctor leaves out the sign-in line and the popover the panel, and
  `Snapshot::completed_from` fills a replayed snapshot's empty fields from the account discovery
  found. History names a provider from its newest snapshot that names it.
- **Rule 4, docs:** `docs/openapi.json` rewritten (`QUOTADIAL_WRITE_OPENAPI=1`), its descriptions
  saying both fields come from the provider as given; `docs/CLI.md` and `skills/quotadial/SKILL.md`
  show both fields and `providers[]`; `docs/ACCESSIBILITY.md` names the panel; one changelog line.
- **Rule 5, tests:** `text::unplain_words` is the one check for #52's plain words ("quota", "auth",
  "credential" outside a name; a name is anything in backticks or quotes, or a path, file or
  variable). Each provider's tests hold its name and every sentence it can produce to it, and each
  fails when one of those words goes in; the OpenAPI and Gemini CLI tests use it too. D-117's list
  test holds each build's names as its ids. The window check's fixtures gain a Gemini CLI account
  with its own sentence and, in the Free copy, a Codex account with none, as an older daemon sends
  it.
- **Gemini on:** `gemini` in `quotadial-providers`' defaults, which the CLI and the app take; the
  CLI's own pass-through feature and CI's separate Gemini steps go; the app's lockfile gains only
  the Gemini crate. The CLI's help and quotadial(1) say a Gemini CLI account is listed with why it
  has no numbers; `history --provider` names gemini; ENVIRONMENT names GEMINI_CLI_HOME, which a test
  holds. `scripts/no-example-provider.sh` unsets GEMINI_CLI_HOME (checked: with GEMINI_CLI_HOME on a
  made-up signed-in folder, the CLI finds a Gemini account there only without the unset).
  `docs/PRIVACY.md` and the README's copy, the README's provider table, `docs/CLI.md`,
  `docs/INSTALL.md` and `docs/SECURITY.md` name it.
- **One name** (asked for by the coordinator, for PR D's recording from the example build): an
  account whose display name says nothing beyond its provider's label, being empty or the label
  itself whatever the case, shows the label once, by one rule for every account
  (`Account::name_beside_provider`, `text::name_beside`, `text::labelled`): `status`, `doctor` and
  `history`, the popover's heading (its name span drawn only when there is a name), its summary and
  tooltip, Settings and History. The example build's accounts read "Example (Claude Max 5x)", where
  `main` shows "fake Example" and this branch had shown "Example Example"; a name given in Settings
  still shows after the label. The window check's Free copy gains an example account: its heading
  reads "Example" and the plan, captured and looked at here.
- **Found on the way and fixed:** doctor's status pages still showed the id; once doctor named
  providers, the release check's doctor half no longer saw `fake` on an example build (run here:
  the old script let it through), so it now fails on `Example` too; the site's `quotadial status`
  sample wrote ids, as its test held it to.
- **Left open, for the manager:** a copy without a licence still offers the week's cost to a
  machine with only a Gemini CLI account, which the cost scan cannot give (D-122); `watch` and the
  daemon say they ask "Gemini CLI at most every 5 min", though it is never asked, which needs a way
  for a provider to say it asks no one, a change in core, proposed as P-021 (`Provider::asks`,
  required, whom a provider's fetch asks or none), to settle before PR 3, since Antigravity asks
  no one either; error lines keep the provider's id
  (`claude: request failed`), as each provider writes its errors; and the site's home page speaks
  of Claude and Codex only, still true, so naming Gemini CLI there is for the site's review.
- **Tests:** in the workspace, 625 passed (1 ignored) on `main` and 656 (1 ignored) now, Gemini
  included by default; the example build 625 (2 ignored) and 655 (2 ignored). By crate, from `main`:
  core 104 to 118, the CLI 51 to 59 (with the Gemini binary test now in every release), the daemon
  100 to 102, Claude 134 to 136 (and 1 ignored), Codex 61 to 63, fake 18 to 19, Gemini CLI 29 to 30,
  `quotadial-providers` 1 to 2; API 54, cost 30, issue 6 and poll 37 unchanged. The tray app 81 to 85
  in both builds, its scripts' tests 5, the site 90, and the window check 3394 checks over 16
  views. Every new test failed first, against a stub or
  with its change left out, and each that passed at once because the code was already there failed
  under a mutation, restored after.
- **Checks run here:** `cargo fmt --check` in both workspaces; clippy with `-D warnings` on the
  workspace, default and `fake`, and on the tray app, default and `fake`; the workspace's tests,
  default and `fake`, with the packaging and OpenAPI checks in them; the release-only licence key
  test; `cargo metadata --locked` for both lockfiles; the tray app's `npm ci`, `npm run build` (34.0
  KB of 150, contrast), `npm test`, `npm run cli`, its tests (`--lib`), default and `fake`, and
  `npm run check:window` in headless Chrome with a profile of its own, removed after; the site's
  `npm ci`, wrangler's start, `npm run check`, `npm run build` and `npm test`; shellcheck,
  `scripts/public-prd.sh --check`, `scripts/privacy.sh --check`, the 0.2.0 release notes and the
  scripts' tests (39, 1 skipped for GNU sha256sum); actionlint on every workflow; and
  `scripts/no-example-provider.sh` on the release CLI and the app's bundled CLI (both pass) and the
  example CLI (it fails, naming both commands). **Not run:** the tray's `tests/icon_press.rs`, which
  puts a real status item in Matt's menu bar; `cargo deny` and `cargo audit`, not installed here
  (no third-party crate was added); and a release build of the app.
- **Safety on this Mac:** every cargo run had `QUOTADIAL_DATA_DIR`, `QUOTADIAL_FAKE_DATA_DIR`,
  `CLAUDE_CONFIG_DIR`, `CODEX_HOME` and `GEMINI_CLI_HOME` in the scratchpad and no token in its
  environment; the release CLIs ran only in empty homes, and the release check on them under
  `sandbox-exec` with a profile that refuses `/usr/bin/security`, `~/Library/Keychains`, every
  `~/.claude*`, `~/.codex*` and `~/.gemini*` path, Matt's QuotaDial folders and the network. No
  vendor CLI ran, no sign-in or keychain item was read, and no app or daemon ran against Matt's data.

### CI: the window check gives Chrome a minute to start

- **What happened:** main's CI failed on #74's merge (run 40236269486, 21:38 UTC on 2026-10-25) in
  the tray job on Linux, at the window check: "Chrome did not start". The check waited 20 s for
  headless Chrome to say where its DevTools listen, and that runner's Chrome was still starting at
  17 to 20 s. #74 changed nothing in the window, the same job had passed on the PR, and a rerun of
  the failed job passed, so main was green again at 21:48 UTC without a change.
- **The fix, at the cause:** the launcher moves from `apps/tray/scripts/check-window.mjs` to
  `chrome.mjs`, unchanged but for the wait, now a minute (`START_WAIT_MS`), and saying how long it
  waited when it gives up. Chrome usually answers within a few seconds, so a minute only matters on
  a slow runner, and still stops a Chrome that never answers. On failure the check still stops
  Chrome and removes its profile, as before.
- **Tests:** `apps/tray/scripts/chrome.test.mjs`, the tray scripts' first, run by `npm test` in the
  tray job before the window check: a script standing in for Chrome gives the address it announces,
  a quiet one fails within its wait saying what it said, one that exits fails with its status, the
  sandbox is turned off on Linux only, and the wait is at least a minute (that test failed with the
  old 20 s). The stand-in needs a shebang, so on Windows only the last runs. The real check passed
  here with the new launcher: 3,350 checks over 16 views.

### M6 PR 1: the groundwork (P-018, P-019, D-118 to D-122)

- **The decisions:** the manager's on #73 (2026-10-25, 20:29 UTC), taken as decided at 20:32 UTC:
  (a) to (d) are D-118 to D-121, the deferred-provider rows are P-019, and P-018 is the one change
  M6 makes in core. The branch is `feat/m6-groundwork`, from `main` at a5cda9e, rebased on `main` at
  ddcb456 once #74 merged, and merged as #75 (ec3163e) with Gemini held off, as the manager asked.
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

- **Measured** on 2026-10-25 and 26: the figures are in "PR D: QuotaDial Example's figures for
  #52", among the newest notes above; the recording is PR E, waiting on Matt.
- **Waited on** Matt's go-ahead for a few minutes with a second QuotaDial icon in his menu bar, for
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
