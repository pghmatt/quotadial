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
