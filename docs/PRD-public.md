<!-- Generated from the private docs/PRD.md by scripts/public-prd.sh. Do not edit by hand; see docs/PUBLIC_SCOPE.md. -->

# QuotaDial — Research & PRD

Oct 21–22, 2026 · Matt Fowkes · Manager: Claude (claude.ai session). Builder: Claude Code in Codus.

> Private sections: "Competitive teardown" and all pricing figures stay out of the public repo. See docs/PUBLIC_SCOPE.md.

## Summary

Build a cross-platform usage tracker for AI coding subscriptions that reads limits from the tokens the CLIs already store, instead of forcing a webview login. It covers Claude and ChatGPT/Codex in v1 (Gemini and others as plugins after), exposes a local API plus an agent skill, and fixes the four complaints users have about [Usage for Claude](https://apps.apple.com/us/app/usage-for-claude/id6755173244): stale numbers, forced sign-in, no way to reach the developer, and single-provider lock-in.

The core is a provider-plugin service with its own store, alert engine and loopback API; UI shells (tray app, TUI, web dashboard, later iOS) are thin consumers of that API. Claude Code builds it from the plan in the last section.

Decisions so far: the product is QuotaDial (quotadial.com, quotadial.app); v1 providers are Anthropic (Claude) and OpenAI (ChatGPT/Codex); this is a product to use and to sell, so licensing, a free tier and a paid tier are in scope from the start. First shell is the Tauri tray app. Status Oct 22 evening: repo pghmatt/quotadial is live; the Claude provider (M1 #1) is in PR #8 with CI green on all three platforms, reviewed, merging after three small fixes; the Codex provider (M1 #2) is next.

## Provider data sources

Both v1 providers expose a server-side usage endpoint that the official CLI already calls, and both leave a token on disk after `claude login` or `codex login`. Read that token, call that endpoint, and no webview login is needed. Every endpoint below is undocumented and may change; the architecture treats each as a plugin with a local-log fallback. All details are from open-source implementations, not official docs, so Claude Code should verify each against a live account on day one.

| Provider | Credential on disk | Endpoint | Headers | Response shape | Fallback | Known fragility |
| --- | --- | --- | --- | --- | --- | --- |
| Anthropic Claude (Pro, Max, Team) | `~/.claude/.credentials.json` → `claudeAiOauth.accessToken` (+ `refreshToken`, `expiresAt`); on macOS the Keychain item "Claude Code-credentials"; env `CLAUDE_CODE_OAUTH_TOKEN` | `GET https://api.anthropic.com/api/oauth/usage` | `Authorization: Bearer <token>`, `anthropic-beta: oauth-2025-04-20`, `User-Agent: claude-code/<version>` (required per [Claude-Code-Usage-Monitor#202](https://github.com/Maciek-roboblog/Claude-Code-Usage-Monitor/issues/202)) | `five_hour{utilization, resets_at}`, `seven_day{…}`, per-model weekly buckets keyed by model name, `extra_usage{utilization}`; unknown buckets appear without an app update ([wakamex/ccusage](https://github.com/wakamex/ccusage)) | `anthropic-ratelimit-unified-*` headers on Messages API responses; Claude Code session JSONL under `~/.claude/projects` for token totals | Returns 429 aggressively, so poll no faster than ~5 min and back off on 429 ([claude-code#31637](https://github.com/anthropics/claude-code/issues/31637)); Free accounts report nothing since Aug 2026; Enterprise returns no usage; token refresh is normally done by `claude` itself |
| OpenAI ChatGPT / Codex (Plus, Pro, Team) | `~/.codex/auth.json` (or `$CODEX_HOME/auth.json`) → access token, refresh token, account id in JWT claims | `GET https://chatgpt.com/backend-api/wham/usage` | `Authorization: Bearer <access_token>`, `ChatGPT-Account-Id: <account_id>` | `rate_limits{primary{used_percent, window_minutes, resets_at}, secondary{…}, plan_type}` plus `credits`; classify windows by `limit_window_seconds` or `window_minutes`, never by slot ([m4fn3/Codex-Usage-Tracker](https://github.com/m4fn3/Codex-Usage-Tracker), [vscode-codex-usage](https://github.com/Ganymede404/vscode-codex-usage/releases)) | `~/.codex/sessions/YYYY/MM/DD/rollout-*.jsonl`, each `token_count` event carries the last `rate_limits` snapshot; also `codex` app-server JSON-RPC | Windows are rolling and only re-anchor on the first request after expiry, so reset drifts; 5-hour window exists on Plus but not Pro as of Aug 25 2026 ([SessionWatcher](https://sessionwatcher.com/guides/codex-rate-limits-explained)); let the Codex CLI refresh tokens rather than writing `auth.json` |

**Deferred providers (v2+)**

| Provider | Credential | Endpoint or source | Notes |
| --- | --- | --- | --- |
| Google Gemini CLI / Code Assist | `~/.gemini/oauth_creds.json` | `cloudcode-pa.googleapis.com/v1internal:retrieveUserQuota` ([methol/usage-bar](https://github.com/methol/usage-bar)) | Daily request quota (1,000 free, 1,500 AI Pro, 2,000 Ultra per [Google quotas](https://developers.google.com/gemini-code-assist/resources/quotas)); no official in-CLI check; session files at `~/.gemini/tmp/*/chats/` as fallback |
| Antigravity | Local app state | Requires the Antigravity app running (CodexBar) | Same Google account family as Gemini |
| Cursor | Browser cookie, pasted or read from the browser | Cursor web dashboard endpoints | Cookie-based; needs Full Disk Access to read Safari cookies on Mac |
| GitHub Copilot | Device-flow OAuth | Premium request and chat quota endpoints | AI Limits & Reset Tracker ships it; cleanest auth of the deferred set |
| Anthropic API, OpenAI API, OpenRouter | API or Admin keys | Documented billing and usage endpoints | Cost tracking, not subscription limits; a different product surface |

**Normalized model** (what every plugin returns, so UI and alerts never see provider shapes):

```
Account { provider, account_id, display_name, plan, org }
Window  { kind: session | weekly | model_weekly | daily | credits, label, used_pct, resets_at, window_seconds, scope_model?, source: api | local_log | cache, fetched_at }
Snapshot { account, windows[], extra_usage?, status: ok | stale | auth_required | rate_limited | unsupported, error? }
```

Every snapshot carries `fetched_at` and `source`, so the UI can always say "fetched 4 min ago from the API" or "from Codex's last session log" rather than showing a number that silently stopped moving.

**Corrections from the build (Oct 22, from the builder's P-001 to P-009, accepted)**

- `Window.resets_at`, `Account.plan` and `Account.org` are optional. Credits windows have no reset; Claude's credentials blob and profile, and Codex's `plan_type`, can all omit these.
- **Codex response shape is `rate_limit` (singular)** with `primary_window` and `secondary_window`, each `{used_percent, reset_at (Unix seconds), limit_window_seconds}`, plus `account_id`, `plan_type` and `credits{has_credits, unlimited, balance}`. The `rate_limits{primary{…window_minutes…}}` form in the table above is wrong. CodexBar stopped reading `~/.codex/sessions` in Nov 2025; its fallback is `codex app-server` (`account/rateLimits/read`), returning `rateLimits.primary{usedPercent, windowDurationMins, resetsAt}`. The live check (issue #5) settles the rollout-log shape.
- Codex `account_id` comes from `tokens.account_id` in `auth.json`; the `chatgpt_account_id` JWT claim is only a fallback.
- On macOS, Claude Code keeps one Keychain item per `CLAUDE_CONFIG_DIR`: `Claude Code-credentials-` + first 8 hex of SHA-256 of the directory path. `Claude Code-credentials` alone is only the default profile. Claude Code also honours `CLAUDE_SECURESTORAGE_CONFIG_DIR`.
- Claude Code rewrites the Keychain item's access list on every token refresh, so "Always Allow" does not stick for a third-party reader. M1 reads via `/usr/bin/security find-generic-password` (the same binary Claude Code trusts, so no prompt in practice, verified on Matt's Mac); the first-run experience is an M3 item.
- On Windows, Claude Code 2.1.286 can store the sign-in in Credential Manager (`tengu_windows_credman` flag or `CLAUDE_CODE_FORCE_WINDOWS_CREDMAN=1`); M1 reads only `.credentials.json` there, so Credential Manager support is an M1 follow-up issue.
- Claude account identity comes from `oauthAccount` in Claude Code's account config (`.config.json` or `.claude.json`); `GET /api/oauth/profile` is the fallback and returns `{ account: { uuid, email }, organization: { uuid, plan_display_name } }`.
- The Claude usage response also carries `iguana_necktie` (cloud credits), `cedar_ember` (reset credits) and `limits[]` (scoped weekly limits, kind `weekly_scoped`, by display name); `extra_usage` amounts are in cents. Claude Code's own plain read sends `GET /api/oauth/usage` with no query and `User-Agent: claude-code/<version>`.
- The unconfirmed-0% rule needs a field; the model gains `unconfirmed: bool` on `Window` (M1 follow-up).

## Product requirements

The product wins on four things the field lacks: no login, honest freshness, works on Windows and Linux, and usable by agents. Everything else matches the best existing feature and is table stakes.

**Fixes for the complaints**

| Complaint | Requirement |
| --- | --- |
| Numbers stale or snap to 0% | Every number shows its age and source; refresh with adaptive interval (2 min while a CLI is active, 10 min idle, never under 5 min for Anthropic) and exponential backoff on 429; a 0% that follows a non-zero reading is held as "unconfirmed" until a second fetch agrees |
| Forced sign-in to Claude in a third-party app | Read CLI credentials first; webview or cookie login is an opt-in fallback, never the default; show exactly which file was read |
| Can't reach the developer | Help → Send feedback opens a prefilled issue with version, OS and redacted diagnostics; a visible email and changelog in the app |
| Accounts in multiple orgs blend | Account identity = provider + org id + account id; the org picker appears when the token spans more than one org |
| Free accounts show empty cards | Detect `unsupported` and say why in one sentence instead of 0% |
| Mac-only, or needs a Mac helper on the LAN | One codebase for Mac, Windows and Linux; no companion device required |

**Core features (free tier)**

- Live session, weekly and per-model windows for Claude and Codex, with reset countdowns and absolute times
- Tray or menu bar icon with a compact meter; popover with all accounts
- Burn rate and "runs out at" projection per window
- Provider status feed (status.claude.com, status.openai.com) with an incident badge
- Threshold and reset notifications (native OS notifications), one account per provider
- Local history, 30 days

**Pro features (paid)**

- Unlimited accounts per provider, named, with org selection
- History to 365 days, activity heatmap, CSV export, shareable image
- Alert rules: threshold, reset, pace-to-limit, time-before-reset, status incidents; webhook and Slack delivery in addition to OS notifications
- Cross-provider "what's open": which subscription has the most headroom now, and API-equivalent cost of the week's local token usage
- Local HTTP API on 127.0.0.1 with a bearer token, a CLI (`quotadial status --json`), an MCP server, and a bundled SKILL.md so Claude Code, Codex and other agents can read budget before starting long jobs
- Optional auto-anchor: one throwaway low-cost request when a rolling window has expired, so resets stop drifting (off by default, exactly-once per window, as [Codex-Usage-Tracker](https://github.com/m4fn3/Codex-Usage-Tracker) implements it)

**Team tier (design in, build after v1)**

- Opt-in reporting from each seat to a small hosted rollup: who is near limits, which plan sizes fit, weekly cost-equivalent per person
- Admin-only; no prompts or content ever leave the machine, only window percentages and token counts

**UX principles**

- Lead with the single number that matters now (the tightest window), then everything else
- Never show a number without its age
- First run to first number under 60 seconds, zero typing, if a CLI is already logged in
- Every error names the fix ("run `claude` to sign in again")
- Keyboard and screen-reader support from day one

**Out of scope for v1**

- iOS, watchOS and widgets (a later native shell over the same local API)
- Gemini, Cursor, Copilot, API-key cost tracking
- Any cloud account for the user

## Architecture

Flow: credential sources (`~/.claude` or Keychain, `~/.codex/auth.json`, status feeds and local session logs) → provider plugins (claude, codex in v1; gemini, cursor, copilot later; each returns the normalized Snapshot) → core service in Rust (poller with adaptive interval and backoff, SQLite store, alert engine, licensing) → local API on 127.0.0.1 (HTTP + bearer token, CLI `quotadial status --json`, MCP server, SKILL.md) → shells and agents (Tauri tray app on Mac/Windows/Linux, web dashboard, Claude Code / Codex / other agents, later iOS and Watch reading the same API).

The core service owns all data and logic; every shell, including the tray app, talks to it only through the local API, so a web dashboard, a TUI and a later iOS app are the same work as the first UI.

**Stack**

- Core: Rust (single static binary, low idle CPU, runs as a user service or launch agent). Crates: `reqwest`, `tokio`, `rusqlite`, `axum` for the local API, `keyring` for secrets, `notify-rust` for notifications.
- Tray app: Tauri 2 with a small TypeScript/React UI; Tauri bundles for macOS (dmg), Windows (msi), Linux (AppImage, deb). The tray process embeds the core or supervises it.
- CLI: a thin binary over the same API (`quotadial status --json`, `quotadial watch`, `quotadial alerts add "warn me at 80%"`); `qd` is the short alias.
- MCP server: exposes `get_usage`, `get_account_headroom`, `list_alerts` as tools; the SKILL.md documents the HTTP shape for agents that prefer curl.
- Licensing: offline-verifiable signed license keys (Ed25519), issued by a small Lemon Squeezy or Paddle checkout; free tier needs no key.
- Secrets: tokens are never copied; the plugin reads the CLI's own file on each poll and keeps only the derived snapshot. If a provider needs an app-owned credential later, it goes in the OS keychain.

**Alternative considered:** TypeScript end to end (Node core + Electron). Faster for the UI, but Electron's memory and the always-on daemon argue for Rust; CodexBar and SessionWatcher both ship native-sized binaries and users notice.

## Leveraging CodexBar

CodexBar's MIT core already encodes every provider contract v1 needs, so Claude Code should port from it rather than rediscover endpoints; the time saved is roughly the first two milestones. Its core (`CodexBarCore`) builds and tests on Linux without AppKit or Keychain, which proves the Rust port is feasible and gives a reference to diff against. Attribution: keep the MIT notice and credit Peter Steinberger in the repo and the About screen.

**What to port directly** (paths in [steipete/CodexBar](https://github.com/steipete/CodexBar), all under `Sources/CodexBarCore/`)

| Need | CodexBar source | What it gives us |
| --- | --- | --- |
| Claude fetch | `Providers/Claude/ClaudeOAuth/ClaudeOAuthUsageFetcher.swift` | `GET /api/oauth/usage?cedar_ember=1`, retry without the query on 400/403, `/api/oauth/profile` for account and org ids, exact `User-Agent` and `anthropic-beta` headers, every response field (`five_hour`, `seven_day`, `seven_day_opus`, `seven_day_sonnet`, `seven_day_routines`, `seven_day_oauth_apps`, `extra_usage`) |
| Claude credentials and refresh | `Providers/Claude/ClaudeOAuth/ClaudeOAuthCredentials.swift`, `ClaudeConfigPaths.swift` | Keychain item `Claude Code-credentials`, `~/.claude/.credentials.json`, `$CLAUDE_CONFIG_DIR`; refresh via `POST platform.claude.com/v1/oauth/token` with Claude Code's client id |
| Claude 429 gate | `Providers/Claude/ClaudeOAuth/ClaudeOAuthUsageRateLimitGate.swift` | Honors `Retry-After`, else a 5-minute per-token cooldown that short-circuits further calls |
| Codex fetch, credentials, refresh | `Providers/Codex/CodexOAuth/{CodexOAuthUsageFetcher, CodexOAuthCredentials, CodexTokenRefresher}.swift` | `wham/usage` and `wham/rate-limit-reset-credits`, `auth.json` field names, `chatgpt_account_id` JWT claim, refresh via `auth.openai.com/oauth/token`, refresh timing rules, `invalid_grant` handling |
| Codex window classification | `Providers/Codex/CodexRateWindowNormalizer.swift` | 300 min = session, 10080 min = weekly, swap primary/secondary when needed |
| Codex local fallback | `UsageFetcher.swift` | Reads the newest rate-limit event from `~/.codex/sessions` |
| Provider status | `ProviderStatusFetcher.swift` | Statuspage.io `api/v2/status.json` and `summary.json` for status.claude.com; incident.io proxy path for status.openai.com |
| Local cost scan | `Vendored/CostUsage/CostUsageScanner*.swift`, `CostUsagePricing.swift`, `ModelsDevPricing.swift` | Token counts from Claude and Codex session logs; pricing from `models.dev/api.json` with a hardcoded fallback table |
| Provider plugin shape | `Providers/ProviderDescriptor.swift`, `ProviderFetchPlan.swift`, `docs/provider.md` | Ordered fetch strategies (oauth, web, cli, apiToken, localProbe); rules: descriptor is the source of truth, identity silo per provider, timeouts on every wait, parse on device, cached data over flapping |
| Tests | `TestsLinux/`, `Tests/CodexBarTests/Fixtures/Providers/` | Linux tests for credentials, rate gate, JWT and cost scan; note there are no Codex fixtures and the Claude fixtures are CLI screen captures, so record our own redacted API responses on day one |

**What not to copy:** the 40+ web-cookie providers, the PTY driver that scrapes the CLI's screen, and the AppKit UI. Those are where CodexBar's complexity lives and none of it is on the v1 path.

**Fastest route:** point Claude Code at the cloned repo and have it write the Rust `Provider` implementations as a translation of the two OAuth fetchers, keeping CodexBar's field names in the fixture tests so a diff against upstream catches endpoint drift later.

**Table corrections (Oct 22, checked by the builder against CodexBar `4654968e9`):** `ClaudeConfigPaths.swift` is at `Providers/Claude/`, not `ClaudeOAuth/`. The Claude retry fires on 400, and on 403 only when the body lacks `user:profile`; the first attempt sends `User-Agent: claude-cli/<version> (external, cli)`. CodexBar's rate gate exempts user-initiated refreshes; QuotaDial's floor exempts nothing. CodexBar no longer refreshes CLI-owned tokens in production (`CodexTokenRefresher.swift` is dead code); it hands them back to `claude` and `codex`. The status fetcher tries the incident.io proxy for every host, then `summary.json` + `components.json`. There are six fetch kinds including `webDashboard`. Codex fixtures do exist at `Tests/CodexBarTests/Fixtures/` (one level above `Providers/`); the Claude folder holds 7 terminal captures and 2 JSON files, with OAuth usage responses only inline in tests.

## Handling every complaint and gap

Each item maps to a concrete mechanism and a milestone; nothing is left as a principle without an implementation.

| Complaint or gap | Mechanism | Milestone |
| --- | --- | --- |
| Stale numbers, snap to 0% | Snapshot carries `fetched_at` and `source`; UI renders age on every number; 0% after non-zero is `unconfirmed` until a second fetch agrees; cached snapshot shown with a grey badge during backoff instead of blanking | M1 model, M3 UI |
| Aggressive 429 from Anthropic | Port CodexBar's rate-limit gate; 5-minute floor; one poller shared by tray, CLI, API and MCP so four consumers never mean four polls | M1 |
| Forced login in a third-party app | Credential discovery from CLI files and Keychain first; a "how we read this" panel naming the file; webview login hidden behind an advanced toggle | M1, M3 |
| Google sign-in fails in webviews | Moot when credentials come from the CLI; fallback uses the system browser (`ASWebAuthenticationSession` / default browser), never an embedded webview | M3 |
| Can't reach the developer | Help menu: Send feedback (prefilled GitHub issue or email with version, OS, redacted diagnostics), Changelog, email address visible in About | M5 |
| Accounts in multiple orgs blend | Account key = provider + org id + account id, taken from `/api/oauth/profile` for Claude and the JWT for Codex; org picker when a token spans orgs; identity silo rule from CodexBar's guide | M1 |
| Mac helper and phone fail to sync | No companion device in v1; later mobile shells read the local API over the LAN with a pairing token, with the daemon as the only source of truth | Design now, build later |
| Free Claude accounts show empty cards | Plan detection → `unsupported` status with one-sentence explanation and a link to upgrade | M1 |
| Reset times drift (Codex rolling windows) | Optional auto-anchor, off by default, exactly-once per expired window, cheapest model, capped retries, as Codex-Usage-Tracker implements it | M4 |
| Apple-only / needs a Mac | Rust core + Tauri shell; CI builds dmg, msi, AppImage and deb from one repo | M3 |
| No Linux | Same build; Linux tray via Tauri's system-tray plugin; CLI and daemon work headless | M3 |
| Single provider lock-in | `Provider` trait with fixture tests; Claude and Codex in v1, each later provider is one crate behind a feature flag | M1, M6 |
| No team or org rollups | Opt-in reporter in the daemon posts window percentages only to a hosted rollup; admin dashboard; Enterprise support by reading org-level data where the API exposes it | Post-v1 |
| Agents can't read budget | Local API, CLI `--json`, MCP server and SKILL.md ship together in M2, free tier includes read-only status, Pro adds rules and headroom queries | M2 |
| No cross-provider decisions | `GET /v1/headroom` ranks accounts by tightest window and time to reset; weekly API-equivalent cost from the ported cost scan | M4 |
| Updates needed for new limit windows | Unknown buckets in the Claude response are surfaced by name without a release, plus a signed remote schema file for labels and window lengths | M1 |
| Silent token expiry | Daemon detects 401, marks the account `auth_required`, shows "run `claude` / `codex login`", never writes the provider's credential file | M1 |
| Battery and background load | Adaptive interval tied to CLI process activity; cache-first widgets; single daemon; SQLite with WAL | M2 |

## Build plan for Claude Code

Milestone 1 is a CLI that prints both providers' limits from local tokens; everything after that is UI and polish on a working core. Each milestone ends with something Matt can run.

| Milestone | Deliverable | Done when |
| --- | --- | --- |
| M1 Core + CLI (week 1) | Rust workspace: `core`, `providers/claude`, `providers/codex`, `cli` | `quotadial status --json` prints normalized snapshots for both providers from a logged-in machine; 429 backoff and stale flags work; unit tests on fixture responses |
| M2 Daemon + local API (week 2) | Core runs as a background service with SQLite history and the HTTP API; `SKILL.md` and MCP server | `curl 127.0.0.1:PORT/v1/status` returns live data; Claude Code can call the skill and read headroom |
| M3 Tray app (weeks 3–4) | Tauri 2 shell: tray meter, popover, settings, notifications | Installable builds for macOS, Windows, Linux; first run shows a number in under 60 s with no typing |
| M4 Alerts + history (week 5) | Alert rules, pace projection, 30/365-day history, heatmap, CSV | Rules fire correctly across sleep/wake and clock changes |
| M5 Licensing + release (week 6) | Free/Pro gating, signed license keys, checkout, auto-update, website, feedback flow | A stranger can buy, install, and reach support without help |
| M6 Next providers | Gemini, Copilot, Cursor as plugins using the same trait | Each ships behind a feature flag with its own fixture tests |

**Repo layout**

```
quotadial/
  Cargo.toml                  # workspace
  crates/core/                # model, store, poller, alerts, licensing
  crates/providers/           # one crate per provider, implements `Provider` trait
  crates/api/                 # axum server, OpenAPI spec, MCP server
  crates/cli/                 # `quotadial` binary (+ `qd` alias)
  apps/tray/                  # Tauri 2 app (Rust shell + TS/React UI)
  skills/quotadial/SKILL.md   # agent-facing instructions
  fixtures/                   # recorded provider responses, redacted
  docs/                       # this PRD, ADRs, provider notes
```

**Provider trait (the contract every plugin meets)**

```
trait Provider {
    fn id(&self) -> &'static str;
    fn discover_accounts(&self) -> Result<Vec<Account>>;   // read local credentials
    async fn fetch(&self, account: &Account) -> Result<Snapshot>;
    fn fallback(&self, account: &Account) -> Option<Snapshot>; // local logs
    fn min_poll_interval(&self) -> Duration;
}
```

**First prompts for Claude Code**

1. "Create the Cargo workspace per the repo layout in this doc. Implement the `Provider` trait and the normalized `Snapshot` model. Add the Claude provider: read `~/.claude/.credentials.json` (and the macOS Keychain item), call `/api/oauth/usage` with the headers listed, map the response, handle 429 with backoff, and write fixture tests from a redacted real response."
2. "Add the Codex provider: read `~/.codex/auth.json`, decode the JWT for the account id, call `chatgpt.com/backend-api/wham/usage`, classify windows by length, and fall back to the newest `rate_limits` in `~/.codex/sessions`."
3. "Build the `quotadial` CLI with `status`, `status --json`, and `watch`. Every line shows the number, its age and its source."
4. "Add the daemon: SQLite store, adaptive poller, axum API on 127.0.0.1 with a generated bearer token, and an MCP server exposing `get_usage`. Write `skills/quotadial/SKILL.md`."
5. "Scaffold the Tauri 2 tray app that supervises the daemon and renders the popover from the API."

**Open questions**

- [ ] Product name: QuotaDial (decided Oct 22, 2026; quotadial.com and quotadial.app registered through Oct 2027). App Store title "QuotaDial: AI Usage & Limits", subtitle "Claude, Codex & ChatGPT quotas". Still to do: USPTO TESS search, reserve @quotadial on GitHub and social
- [ ] Terms-of-service review for calling undocumented endpoints with the user's own token, same exposure as every competitor but worth a read before selling
- [ ] Auto-anchor default: off (safer) or on (better resets)

**Risks**

| Risk | Mitigation |
| --- | --- |
| Provider changes or blocks the endpoint | Plugin isolation, local-log fallback, remote schema file for new windows, fixtures to detect drift in CI |
| Anthropic rate-limits polling | 5-minute floor, backoff, cache-first widgets, one poll shared by all shells |
| CodexBar adds Windows | Move faster on alerts, history, team tier and agent API, which it does not have |
| Token refresh ownership | Never write provider credential files; prompt the user to run the CLI when a token expires |

**Sources**

[Usage for Claude App Store](https://apps.apple.com/us/app/usage-for-claude/id6755173244) · [Usage for Claude privacy policy](https://amirhayek.dev/ClaudeUsage/privacy/) · [Usage for Claude version history](https://amirhayek.dev/ClaudeUsage/version-history.html) · [AI Limits & Reset Tracker](https://apps.apple.com/us/app/ai-limits-reset-tracker/id6758946226) · [iCodex](https://apps.apple.com/us/app/icodex-claude-codex-usage/id6792240003) · [AI Usage](https://apps.apple.com/us/app/ai-usage/id6753959146) · [CodexBar](https://github.com/steipete/Codexbar) · [SessionWatcher](https://sessionwatcher.com/guides/codex-rate-limits-explained) · [wakamex/ccusage](https://github.com/wakamex/ccusage) · [claude-code#31637](https://github.com/anthropics/claude-code/issues/31637) · [Claude-Code-Usage-Monitor#202](https://github.com/Maciek-roboblog/Claude-Code-Usage-Monitor/issues/202) · [m4fn3/Codex-Usage-Tracker](https://github.com/m4fn3/Codex-Usage-Tracker) · [vscode-codex-usage](https://github.com/Ganymede404/vscode-codex-usage/releases) · [methol/usage-bar](https://github.com/methol/usage-bar) · [codex-island](https://github.com/ericjypark/codex-island) · [Gemini Code Assist quotas](https://developers.google.com/gemini-code-assist/resources/quotas)
