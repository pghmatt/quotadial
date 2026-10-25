# Product

<!-- impeccable:product-schema 1 -->

Facts below come from the repository (README, `docs/`, `CHANGELOG.md`, the code) and the decisions
on #49 (D-098, D-102). Facts marked *(inferred)* were proposed to Matt on 2026-10-24 at 23:33 UTC
and not yet answered; the manager asked on #49 (02:30 UTC) for the site to be built without
waiting, so they stand until Matt changes them.

## Platform

web

## Stack

Delegated: Next.js App Router exported as static files, Tailwind v4, self-hosted type, served by
Cloudflare Pages beside the existing Pages Function in `functions/api/key.ts` (D-098, D-102).
Static export because every page is the same for every visitor and the one dynamic step, the key,
is already a function. *(inferred: proposed to Matt, unanswered)*

## Users

Developers who pay for Claude (Pro, Max, Team) or ChatGPT (Plus, Pro, Team) and work in Claude Code
or the Codex CLI, often with long agent runs, on macOS, Windows or Linux. Their job: know how much
of each plan limit is left and when it resets, so a run does not stop halfway, without signing in
again or keeping a browser tab open. Some run several Claude subscriptions under different
`CLAUDE_CONFIG_DIR`s. *(The audience was proposed to Matt and is unanswered; the product itself is
built for exactly these people.)*

Two more visitors with a narrow job each: a buyer arriving from a Lemon Squeezy receipt to collect
a licence key (`/key`), and anyone checking what QuotaDial reads and sends before installing it.

## Product Purpose

QuotaDial shows how much of your Claude and ChatGPT/Codex plan limits you have used, and when they
reset, so a long agent run does not stop halfway. It puts the tightest limit in the menu bar or
system tray, and the same numbers in a terminal command (`quotadial`, `qd`), a local HTTP API on
127.0.0.1 and an MCP server with a SKILL.md, so agents can check before a long job.

Success for the site: a developer who has hit a limit mid-run understands in one screen what
QuotaDial does and why it needs no sign-in, downloads it for their system, and later finds where
Pro is and how to get their key.

## Positioning

- **No sign-in.** QuotaDial reads the sign-in Claude Code and the Codex CLI already keep, and only
  reads it: it never writes, refreshes, moves or logs it.
- **Honest numbers.** Every number says how old it is and where it came from (the API, the cache,
  the Codex CLI's own logs); a sudden 0% is held until a second fetch agrees.
- **Pace.** Each window says how fast it is filling and whether it runs out before it resets.
- **Built for agents too.** The same numbers through a command, a local API and MCP, so an agent can
  check headroom before starting.
- **Every system.** macOS, Windows and Linux, from one codebase.
- **On your machine.** It connects only to the providers' usage endpoints, their status pages,
  GitHub for updates, and webhooks you add; no account, no analytics, no crash reports.

## Operating Context

The menu bar or tray; a terminal beside Claude Code or Codex; agent runs that last an hour or a
night; limits that reset every five hours and every week; several subscriptions on one machine.
Providers rate-limit their usage endpoints: QuotaDial asks Anthropic at most once every 5 minutes
per account and ChatGPT at most once every 2 minutes.

## Capabilities and Constraints

- **Downloads** come from GitHub Releases: a universal macOS `.dmg`, a Windows `.msi`, a Debian
  `.deb`, a Linux `.AppImage`, and the command alone for each system; each release lists every
  file's SHA-256. Builds are not yet signed: until Matt's certificates exist, each system warns on
  first open, and `docs/INSTALL.md` says how to open it anyway.
- **QuotaDial Free** needs no key: one account per provider, every window with its pace and reset,
  30 days of history, alerts for a share used, a reset and a sign-in, the providers' status pages,
  and read-only status for agents. **QuotaDial Pro** adds every account on the machine, a year of
  history, alerts before a window runs out, before a reset and for incidents, alerts to webhooks
  and Slack, the week's cost at API prices, and headroom and alert rules through the API and MCP.
- **Buying** goes through Lemon Squeezy, the merchant of record, whose checkout shows the price.
  The price is never on the site or in the repository (D-098). The receipt's button opens
  `quotadial.com/key#licence=<key>`; the page posts the key to `/api/key`, which checks it with
  Lemon Squeezy and answers with the QuotaDial key, keeping and logging nothing (D-102). The key is
  then pasted into Settings or `quotadial licence add`, and checked on the machine.
- **Open source** under the MIT licence; Pro is a key, checked offline, for features in the same
  build.
- **Not live yet:** the site is built, not deployed, until Matt says. The checkout link, the
  support mailbox and the domain are his to provide (#51). Providers beyond Claude and
  ChatGPT/Codex (Gemini CLI first) are planned for M6 and must not be promised as shipped.
- **Terminology** (#52): a "window" (a limit with its own reset), a "reset", a "sign-in"; never
  "quota", "auth" or "credential" in anything a visitor reads.

## Brand Commitments

- The name **QuotaDial**, and the app's own icon: a dial in the menu bar or tray, beside the share
  used (`apps/tray/src-tauri/icons/`, drawn by `icon.rs`).
- Plain words, the same in the app, the command, the API's errors and the docs (#52).
- Support address `support@quotadial.com` (proposed on #49; the mailbox is Matt's to make).
- The privacy text is one text, `docs/PRIVACY.md`, shown in the app, the README and on the site
  (D-106).

## Evidence on Hand

- The real app, which can show its window with made-up accounts through
  `apps/tray/scripts/check-window.mjs --screenshots` (light and dark, Free and Pro). Anything shown
  that way is example data and is labelled so.
- `CHANGELOG.md`, `docs/PRIVACY.md`, `docs/INSTALL.md`, `docs/CLI.md`, `SECURITY.md`,
  `docs/ACCESSIBILITY.md`: real text the site renders or links.
- Absent, and never to be invented: customers, testimonials, download counts, star counts, ratings,
  benchmarks, press, a price, a signed build, providers not yet built.

## Product Principles

1. Say what QuotaDial does in the reader's own situation (a run that stopped at a limit), then how.
2. Every claim on the site is one the app keeps today; future work is labelled future.
3. Show the real product at work, with example data labelled as example.
4. Privacy is a fact to show, with the list of addresses, not a slogan.
5. The site works for everyone the app works for: keyboard, screen readers, both themes, any size.

## Accessibility & Inclusion

WCAG 2.2 AA, as the app holds itself to (`docs/ACCESSIBILITY.md`): contrast in both themes,
keyboard and screen readers throughout, reduced motion respected, nothing cut off at 200% zoom.
