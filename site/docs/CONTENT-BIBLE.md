# Content bible: quotadial.com

Every fact the site states, where it comes from, and how it changes. Facts change here and in
`lib/data/`, never by redesigning a page (brain-site-build, step 9). Nothing on the site is invented
except the example accounts, which are labelled wherever they appear.

## What is real, and where it lives

| Fact | Value | Source of truth | In code |
| --- | --- | --- | --- |
| Name | QuotaDial | the repository | `SITE.name` |
| What it does | Shows how much of each Claude and ChatGPT/Codex plan limit you have used and when it resets, in the menu bar or tray, the `quotadial` command, a local API and an MCP server | `README.md` | page copy |
| Systems | macOS (Apple silicon and Intel), Windows, Linux (Debian/Ubuntu `.deb`, other `.AppImage`) | `docs/INSTALL.md` | `lib/data/release.ts` |
| Where downloads come from | QuotaDial's GitHub releases; every release so far is a pre-release, so links go to the releases page | GitHub | `DOWNLOAD_PAGE` |
| Signing | Builds are not signed yet; each system warns on first open; `docs/INSTALL.md` has the steps | `docs/INSTALL.md`, #51 | /download copy |
| Checksums, doctor, licence keys, updates | From the release after 0.2.0, which carries CHANGELOG's Unreleased section: `SHA256SUMS.txt`, `quotadial doctor`, Settings' Licence field and the daily update check. 0.1.0 and 0.2.0 have none of them, so the site goes live only after that release is published (the deploy workflow checks for `SHA256SUMS.txt`) | `CHANGELOG.md`, `docs/INSTALL.md` | /download, /support, /pro, /key, /changelog |
| Sign-in | Reads the sign-in the Codex CLI keeps; never writes, refreshes, moves or logs it. Claude's numbers come from Claude Code's status line, once its settings have one line, and Claude Code's sign-in is never read (D-145) | `README.md`, `docs/PRIVACY.md`, `docs/INSTALL.md` | page copy |
| How often it asks | ChatGPT at most once every 2 minutes; Anthropic never, since Claude's numbers are read from QuotaDial's own files (D-145) | `README.md` | page copy |
| Addresses it connects to | The list in `docs/PRIVACY.md`, and nothing else | `docs/PRIVACY.md` (D-106) | /privacy renders the file |
| Free and Pro | The lists in `README.md` and `ProFeature::what` | `crates/core/src/tier.rs` | `lib/data/pro.ts` |
| Price | Not on the site, not in the repository; Lemon Squeezy's checkout shows it | D-098 | none |
| Checkout link | Matt's to add as `QUOTADIAL_CHECKOUT_URL`; until then /pro says buying opens soon | #51 | `SITE.checkout` |
| The key page | `/key#licence=<key>` posts to `/api/key`; the answers are the function's own sentences | D-102, `lib/key.ts` | /key |
| Licence | MIT, free software | `LICENSE`, `NOTICE` | footer, `softwareJsonLd` |
| Support | support@quotadial.com (the mailbox is Matt's to make, #51) | #49 | `SITE.support` |
| Security reports | GitHub private vulnerability reporting, or support with "Security" in the subject; an answer within 3 working days | `SECURITY.md` | /support |
| Changelog | `CHANGELOG.md`, newest first | `CHANGELOG.md` | /changelog renders the file |
| The site itself | No cookies, no analytics, no third-party scripts | this build | footer, /privacy |

## The example accounts (labelled wherever they appear)

Tuesday 14:10, QuotaDial Pro, two accounts on example.com, in `lib/data/examples.ts` and
`apps/tray/scripts/site-fixtures.mjs` (a test holds them together):

| Account | Window | Used | Resets | Pace |
| --- | --- | --- | --- | --- |
| Claude, "Work", dev@example.com, Max 5x | Weekly | 91% | in 1 d 15 h (Thu 06:00) | 2.0%/h, runs out in 4 h 30 min (Tue 18:40), 1 d 11 h before it resets |
| | Session | 38% | in 2 h 10 min (16:20) | 8.5%/h, lasts until it resets |
| Codex, you@example.com, Pro | Weekly | 46% | in 5 d 3 h (Sun 17:10) | 0.4%/h, lasts until it resets |
| | 5-hour | 12% | in 3 h 41 min | |

Alert rules in the Settings picture: "warn me at 80%", "tell me 30 minutes before the weekly window
runs out", "tell me when an account needs to sign in again", sent to hooks.slack.com as well.

**The label**, beside every picture of the app and every table built from these numbers:
"Example accounts". The footer says: "The accounts and numbers in pictures of the app are examples."

## What may not appear

QuotaDial's price, or any dollar amount that could be read as one (a person's own spend in an
example, such as the week's cost /cli renders from `docs/CLI.md`, is not one); "Pro price",
"lifetime", "per month"; ratings, reviews, testimonials, user or download counts; a claim that
builds are signed; providers beyond Claude and ChatGPT/Codex as if shipped (Gemini CLI is planned
for M6 and may be named only as planned); a promise of a reply time beyond `SECURITY.md`'s.

## Words

The product's own (#52): a window, a reset, a sign-in, the share used, the pace. Never quota, auth,
credential, token (as a thing the reader handles), endpoint, poll. The copy library is
`docs/copy/source-library.md`; the jargon gate is `docs/copy/jargon-terms.json`.
