# Changelog

Every change people will notice in QuotaDial, newest first, in the form of
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/). QuotaDial follows
[Semantic Versioning](https://semver.org/). Each pull request adds its lines under Unreleased, the
app's Help shows the version it is, and a release's notes come from its section here.

## [Unreleased]

### Added

- Licence keys for QuotaDial Pro, checked on your machine with no network call: `quotadial licence add`, or the Licence field in Settings.
- QuotaDial Free and QuotaDial Pro. Free has one account per provider, 30 days of history, alerts for a share used, a reset and a sign-in, the status pages, and read-only status for agents. Pro adds every account, a year of history, the other alert rules, webhooks and Slack, the week's cost, and headroom and alert rules through the API.
- A Free copy says in one line where each Pro feature would be, and a key added or removed counts at once, with no restart.
- `quotadial doctor` prints what support asks for, with nothing secret in it.
- Help in the app: Send feedback opens a GitHub issue with the same diagnostics filled in, What's new shows this list, and About names the version, the licence and the support address.
- Updates: once a day the app asks GitHub for a newer QuotaDial, offers it in the popover and in Help, and installs it only when you choose Update, after checking its signature. Turn the daily check off in Settings; Help's Check for updates asks at any time.
- What QuotaDial reads and sends, in Help and the README: every address it connects to, and nothing else.
- Each release lists every file's SHA-256 in `SHA256SUMS.txt`, and its notes say what is signed.
- The website's key page: the "Get your QuotaDial key" link in a Lemon Squeezy receipt gives the QuotaDial licence key, the same each time it is opened.
- Pasting the key from a Lemon Squeezy receipt into QuotaDial says where QuotaDial's own key is, and a licence that has ended says where to renew it.
- `quotadial alerts list --json` marks a rule that waits for QuotaDial Pro with `"waits": true`, as the text says "(waits for Pro)".
- When something is wrong with QuotaDial itself, such as a history it cannot save, `quotadial status`, the app, `doctor` and the API say so in one line.
- A damaged history is moved aside under a dated name and a new one started, keeping your alert rules.
- After the machine wakes from sleep, every account is asked again at once rather than at its next turn.
- When a provider cannot be reached, QuotaDial asks again within a minute, so the numbers are current soon after the network is back.
- Each release carries a software bill of materials for the app and the command, and `SECURITY.md` says how to report a vulnerability.
- Command and = (Control and = on Windows and Linux) makes the app's window larger, and Command and 0 sets it back; the window grows with its text.
- History, Settings and Help take the keyboard's focus when they open, and Done gives it back to the button that opened them.
- A screen reader reads when an account was used as a line a day, in place of History's grid of squares.
- `docs/ACCESSIBILITY.md` says how to use QuotaDial by keyboard and screen reader, and what is not there yet.
- `GET /v1/openapi.json` describes the local API in OpenAPI 3.1, the same as `docs/openapi.json`: every request, its parameters, and the shape of each answer and error.
- Tab completion for bash, zsh, fish and PowerShell, and a manual page for every command (`man quotadial`), in each release's archives and in the app; the deb installs both.
- `quotadial --help` describes QuotaDial with examples, and every command's `--help` says what each of its arguments is for.
- On macOS, VoiceOver and Full Keyboard Access open QuotaDial from its menu bar icon, as a click does.
- On QuotaDial Pro, each account in History has Share image, which saves a picture of its last 7 or 30 days in your Downloads folder, ready to post. The picture shows the plan or the name you gave the account, never its email address.
- Each release's files also come under a name without the version, such as `QuotaDial_universal.dmg`, so one link always fetches the newest full release.
- Gemini CLI: an account QuotaDial finds signed in shows with why it has no numbers, and its sign-in is never opened. The sign-in panel is now "What QuotaDial does with this sign-in", in each provider's own words, and `status`, `doctor` and the JSON name each provider as it names itself (`provider_name`, `sign_in_use`).
- Antigravity: add `quotadial antigravity status-line` to Antigravity CLI's status line, as the install guide shows, and each of your Antigravity limits shows with its share used and reset while Antigravity CLI runs, marked stale with the time it is from once it stops. QuotaDial never reads Antigravity's sign-in, asks no one about it, and keeps your status line as it was.

### Changed

- `quotadial watch` and `quotadial daemon` say whom QuotaDial asks and how often, Anthropic and OpenAI, and that it asks no one about Gemini CLI; QuotaDial Free offers the week's cost only where Claude Code's or the Codex CLI's logs could give it.
- A provider's error now begins with the provider's own name and writes your home folder as `~`, such as "Codex: could not read the sign-in from ~/.codex/auth.json: ...", the same in `quotadial status`, `doctor`, the app and the local API, whose `errors[].message` no longer begins with the provider's id.
- QuotaDial's own messages, such as a history it could not open, a warning about `config.json` or where `quotadial daemon` keeps its history, write your home folder as `~` too, and `doctor` no longer cuts into a folder whose name only starts like it, such as `/Users/anna` beside `/Users/ann`.
- The app's window says where and when an account's numbers were read once for the account, such as "From the API 3 min ago", in a line under its windows, when all of them were read from the same source at the same moment, where it said it under every window before; when they differ, each window still says its own.
- The app's window is redesigned: each account is a card of its own, headed by the name you gave it or its provider, with its email address on a line under it; the line under the number says which account it is, as the tooltip of QuotaDial's icon now does; a command QuotaDial asks you to run shows as code that one click selects; and History, Settings and Help use the same cards.
- The picture History's Share image saves now draws the account as the window does, as a card on a plain grey ground that is light or dark to match the window, with no grey plate behind its charts. It still shows the plan or the name you gave the account, never its email address.
- The app's window keeps its toolbar, History, Settings and Help (Done in a panel) and Quit QuotaDial, and the line saying where the numbers come from at its foot while the accounts or a panel scroll above it, with a faint line over it while there is more below, as far as 200% zoom and a little past it; zoomed in further, where the toolbar would take most of the window, the whole window scrolls as one, as it did before. Where the system's scroll bars take room, the window keeps room for one on each side, so its cards do not change width when it starts to scroll.

### Fixed

- On a full disk, an alert sent to a webhook is no longer sent again every minute, and the daemon still starts.
- The install guide's command for the CLI now fetches `SHA256SUMS.txt` and checks the archive against it before anything is unpacked or installed, and makes `/usr/local/bin` on a Mac that has none.
- A clock set back no longer holds up a webhook's next try.
- A log line with an impossible token count no longer makes the week's cost wrong.
- Grey text on tags, warnings, the chosen period in History and the edges of text fields now have enough contrast to read, in both themes.
- A long account name or address no longer runs off the side of Settings or History, and nothing in the window is cut off at 200%.
- The command in the warning about a `config.json` other users can read now works when pasted into a terminal, such as `chmod 600 ~/'Library/Application Support/quotadial/config.json'` on macOS, where the space in `Application Support` used to split it.
- The command that signs a Claude Code folder in again now works when pasted, from `doctor` too, and writes a folder in your home folder from `~` or `$HOME`, never with your user name: `CLAUDE_CONFIG_DIR=~/'My Claude' claude`, or on Windows `$env:CLAUDE_CONFIG_DIR = "$HOME\My Claude"; claude`. A folder with a space in its name used to be written in full, and `doctor` showed it as `'~/My Claude'`, which no shell expands.
- A folder whose name only starts as your home folder's does, such as `/Users/ann (old)` beside `/Users/ann`, is no longer written `~ (old)` in errors and in `doctor`, and the `chmod 600` command in the `config.json` warning and the command that signs a Claude Code folder in again work when pasted for such a folder, where the `~` made them fail.
- `doctor` no longer hides a word just because it is spelled like your user name. For a user called `claude` it showed the command that signs a Claude Code folder in again as `CLAUDE_CONFIG_DIR=~/'.[user]-work' [user]` and `~/.claude` as `~/.[user]`, which is not what to type and not a folder that exists, and it did the same for `codex`, `status` and any other word it writes. It now hides your user name only where it is part of a place outside your home folder, such as `/Volumes/Backup/Users/claude`.

## [0.2.0] - 2026-10-24

### Added

- Pace: each window shows how fast it is filling and, when that comes first, when it runs out before it resets.
- Alert rules written in plain words, such as `quotadial alerts add "warn me at 80%"`, or added in Settings. A rule can watch a share used, a time before a reset, running out before the reset, a reset, a sign-in that needs renewing, or a provider's incident, and each alert is told once.
- Alerts sent to webhooks and Slack as well as the system's notifications, set in `config.json`.
- History kept for 30 days, or up to 365 with `history_days`: `quotadial history`, its CSV and JSON, and the app's History panel with charts, a heatmap and Export CSV.
- The providers' status pages, read every 5 minutes, with ⚠ beside the tray's number during an incident.
- The week's cost at each provider's API prices, from the token counts in Claude Code's and the Codex CLI's logs, never their content.

## [0.1.0] - 2026-10-23

### Added

- Claude and ChatGPT/Codex usage limits, read with the sign-ins Claude Code and the Codex CLI already store, never written or moved.
- `quotadial status` and `quotadial watch`, as text or JSON, with every number's age and source.
- Every Claude subscription stored on the machine as its own account, named by where its sign-in is.
- The daemon: one poller for everything, a local history, and a local HTTP API on 127.0.0.1 with a bearer token.
- `quotadial mcp`, an MCP server for agents, and a SKILL.md that tells them how to check their usage.
- The tray app for macOS, Windows and Linux: the tightest limit in the menu bar or tray, a popover with every account and window, Settings, and notifications when a limit passes 90% or a sign-in needs renewing.

[Unreleased]: https://github.com/pghmatt/quotadial/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/pghmatt/quotadial/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/pghmatt/quotadial/releases/tag/v0.1.0
