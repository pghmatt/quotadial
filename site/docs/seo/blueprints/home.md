# Blueprint: /

Job: show a developer who pays for Claude or ChatGPT and works in Claude Code or the Codex CLI that
QuotaDial keeps every plan limit and its reset in view, with no new sign-in, on their system, and
get them to download it.
Primary phrase: claude usage tracker · Secondaries: claude code usage monitor, codex usage tracker,
claude usage tracker windows · Intent verdict: looking for a tool (Google US desktop, 2026-10-25
02:51 UTC) · Segment: product pages of usage trackers.
Study: private (the Brain's workspace, `quotadial-site-private/seo/intent/home/study.md`).

## First phone viewport (390x844), in this order

1. H1: "Claude and Codex usage tracker for your menu bar or tray"
2. Answer sentence: "See how much of each Claude and ChatGPT/Codex plan limit you have used,
   and when it resets, from the sign-in Claude Code and the Codex CLI already keep."
3. Primary action: "Download for macOS" (or Windows, or Linux, by the visitor's system; "Download
   QuotaDial" when it cannot tell), a link to `/download#<system>`, marked `data-primary-action`,
   firing `download_click` with the system.
4. Proof: the real QuotaDial window, as the app draws it, with made-up example accounts, labelled
   "Example accounts" where it is shown. No price, no rating, no user count.

## Sections, top to bottom

| # | Job | H2 | First-paragraph answer | Required elements | Evidence asset | Links out |
|---|---|---|---|---|---|---|
| 1 | What it tracks | Each limit, its reset and its pace | Claude's five-hour session, its weekly limits and per-model weeks, and each Codex window, each with the share used, when it resets, how old the number is, and whether it runs out before the reset | the real window (light or dark), a window row read aloud: "Weekly 91%, resets in 2 d 4 h, from the API 4 min ago; at 2.0%/h, runs out in 4 h 18 min" | app captures with example accounts | |
| 2 | No sign-in | Uses the sign-in you already have | It reads the sign-in Claude Code and the Codex CLI keep, and only reads it: it never writes, refreshes, moves or logs it; every Claude subscription you keep under its own `CLAUDE_CONFIG_DIR` shows up | where it looks, per provider, in plain words | `README.md`, provider support | |
| 3 | Alerts | A warning before a run stops | Rules in plain words, such as "warn me at 80%" or "tell me 30 minutes before the weekly window runs out", as the system's notifications | two example rules as typed, the notification they give | `docs/CLI.md` Alerts | |
| 4 | Agents | Your agents can check before they start | The same numbers through `quotadial status`, a local API on 127.0.0.1 and an MCP server with a SKILL.md, so an agent can check its headroom first | a short terminal sample of `qd status` output with example accounts | `docs/CLI.md` | |
| 5 | Systems | On macOS, Windows and Linux | One app for all three, in the menu bar, the notification area or the panel's tray | three systems named with their file types | `docs/INSTALL.md` | /download |
| 6 | Privacy | Everything it connects to | Anthropic and ChatGPT for your limits, their public status pages, GitHub for updates, and webhooks you add: nothing else, no account, no analytics | the address list, in short | `docs/PRIVACY.md` | /privacy |
| 7 | Free and Pro | QuotaDial Free and Pro | Free needs no key; Pro adds every account, a year of history, early warnings, webhooks and Slack, the week's cost and headroom for agents | the two lists, no price | `README.md` Free and Pro | /pro |
| 8 | Questions | Questions before you install | (each question an H3 with its answer) | the demanded questions below | | /support |
| 9 | Close | (no heading, or the action named) | the download action again | | | /download |

## Demanded questions

| Question | Answered in section |
|---|---|
| Why use a tracker when Claude Code has /usage and Codex has /status? | 8 (and 1) |
| Does it read my conversations? | 6 and 8 |
| Does it need an API key or a new sign-in? | 2 and 8 |
| Will it warn me before I hit a limit? | 3 |
| Does it work on Windows and Linux? | 5 |
| Does it work with Codex? | 1, 2 |
| Can it show several Claude subscriptions? | 2 and 7 |
| Can my agent check its limits? | 4 |
| Is it open source? | 8 |

## Information gain

Shows the real app at work, with every number's age and pace; lists every address it connects to;
covers Linux; several Claude subscriptions; agents through a local API and MCP. None of the compared
pages does more than two of these.

## Not copied

A countdown, avatars, a star rating or a user count (none exist; none are invented); a price in the
first viewport (the site shows no price, D-098); comparisons naming other products.

## Media

The app's window from `apps/tray/scripts/check-window.mjs --screenshots`: the real interface with
the made-up accounts in `window-fixtures.mjs`, light and dark. Labelled "Example accounts". Alt
pattern: "QuotaDial's window: <what it shows>, example accounts".

## Schema

`SoftwareApplication`: name, operatingSystem (macOS, Windows, Linux), applicationCategory
`DeveloperApplication`, license (MIT), downloadUrl `/download`, isAccessibleForFree. No offers
price, no rating.

## Conversions

Primary: the download action (`download_click`, with the system). Secondary: "See what Pro adds"
(to /pro). Events go through the site's event adapter; no analytics service is chosen (an owner
decision), so the adapter records to the page only.

## Title and meta

Title: "Claude and Codex usage tracker for the menu bar | QuotaDial" (59) · Meta: "See how much of each
Claude and ChatGPT/Codex plan limit you have used and when it resets, from the sign-in your CLI
keeps. For macOS, Windows and Linux." (153)

## Reviewer checks

Fold set present at 390 · section order matches · every H2 answered in its first paragraph · the
example label beside the app captures · phrase in H1 and title · no claim the app does not keep ·
action above the fold · `download_click` fires with the system.
