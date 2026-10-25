# QuotaDial

QuotaDial shows how much of your Claude and ChatGPT/Codex plan limits you have used, and when they
reset, so a long agent run does not stop halfway. It reads the login your Claude Code and Codex CLIs
already store, asks each provider for your current usage, and puts the answer in your menu bar or
system tray, in your terminal, behind a local HTTP API, and in front of your agents through an MCP
server and a SKILL.md. It runs on macOS, Windows and Linux.

**Status: pre-alpha.** `quotadial status` and `quotadial watch` show Claude and ChatGPT/Codex
usage. `quotadial daemon` polls for every consumer, keeps the history that gives each window its
pace (how fast it fills, and whether it runs out before it resets), checks alert rules written in
plain words (`quotadial alerts add "warn me at 80%"`), keeps 30 days of history (`quotadial
history`), watches the providers' status pages for incidents, works out what the week's local use
would have cost at API prices, and serves a local HTTP API, and
`quotadial mcp` is an MCP server for agents, with [a skill](skills/quotadial/SKILL.md) that tells
them how to use it, and `quotadial doctor` prints what support asks for. See
[docs/CLI.md](docs/CLI.md) for the commands, output and exit codes, and
[CHANGELOG.md](CHANGELOG.md) for what changed in each version. The tray app
([apps/tray](apps/tray)) shows the tightest limit in the menu bar or tray and runs the daemon
itself; see [docs/INSTALL.md](docs/INSTALL.md) to install or build it, and
[docs/ACCESSIBILITY.md](docs/ACCESSIBILITY.md) to use it by keyboard or screen reader.
Progress is tracked in the [milestones](https://github.com/pghmatt/quotadial/milestones), and the
product spec is in [docs/PRD-public.md](docs/PRD-public.md).

## QuotaDial Free and Pro

QuotaDial Free needs no key: one account per provider, every window with its pace and reset, 30
days of history, alerts for a share used, a reset and a sign-in, the providers' status pages, and
read-only status for agents through the CLI, the local API and MCP. QuotaDial Pro adds every
account on the machine, a year of history, a picture of an account's history to share, alerts
before a window runs out, before a reset and for incidents, alerts to webhooks and Slack, the
week's cost at API prices, and headroom and alert rules through the API and MCP. A licence key is
checked on your machine, with no call home, and a Free copy says in one line where each Pro
feature would be. See [docs/CLI.md](docs/CLI.md#quotadial-free-and-pro).

## Provider support

| Provider | Reads the login from | Status |
| --- | --- | --- |
| Claude | Claude Code (the macOS Keychain, `~/.claude/.credentials.json`, or `CLAUDE_CODE_OAUTH_TOKEN`), for every `CLAUDE_CONFIG_DIR` it finds | Works, with every stored subscription |
| ChatGPT / Codex | Codex CLI (`auth.json` in `$CODEX_HOME` or `~/.codex`) | Works |

If you keep several Claude subscriptions, each signed in through its own `CLAUDE_CONFIG_DIR`,
QuotaDial shows all of them, not only the one Claude Code uses now. See
[docs/CLI.md](docs/CLI.md#accounts) for where it looks.

QuotaDial only reads these credentials. It never writes, refreshes or moves them, and it never logs
them. When a sign-in has expired, run `claude` or `codex` once and QuotaDial picks up the refreshed
one.

Anthropic rate-limits its usage endpoint, so QuotaDial asks it at most once every 5 minutes per
account, and ChatGPT at most once every 2 minutes, and backs off when told to. In between, it shows
the last numbers and says how old they are. When ChatGPT cannot be reached, QuotaDial shows the
newest numbers from the Codex CLI's own session logs, marked as coming from them.

<!-- privacy: copied from docs/PRIVACY.md by scripts/privacy.sh; edit it there -->

## What QuotaDial reads and sends

QuotaDial runs on your machine. It reads the sign-ins that Claude Code and the Codex CLI already
keep, and never writes, refreshes, moves or logs them. From the CLIs' session logs it reads only
numbers: token counts and model names for the week's cost, and the Codex CLI's own record of your
limits. What you and the model wrote never leaves the machine, and QuotaDial never reads it.

QuotaDial connects to these, and nothing else:

- **api.anthropic.com**, with your Claude Code sign-in, for each Claude account's limits, at most
  once every 5 minutes per account.
- **chatgpt.com**, with your Codex CLI sign-in, for your ChatGPT/Codex limits, at most once every
  2 minutes.
- **status.claude.com** and **status.openai.com**, the providers' public status pages, every 5
  minutes while `status_pages` is on. These carry no sign-in.
- **github.com**, where QuotaDial's releases are, once a day while the app's "Check for updates"
  is on, or when you check yourself; and **release-assets.githubusercontent.com**, where GitHub
  keeps the files, when you choose Update. These carry no sign-in and nothing about you or your
  usage. GitHub sees your IP address, as it does for any download.
- The **webhook and Slack addresses you add** to `config.json`, if you add any. Each gets every
  alert: what fired and its numbers, never a token or where a sign-in is kept.

Help's links, to Send feedback, the changelog and the support address, open in your own browser or
mail app, and only when you click them. Send feedback fills in a GitHub issue that you read before
you send it.

The local API answers only on this machine, at 127.0.0.1, and asks for QuotaDial's own token. The
MCP server talks only to the agent that starts it. QuotaDial's history, settings and licence key
stay on your machine, readable only by you. A picture made with History's Share image is drawn on
your machine and saved in your Downloads folder, and QuotaDial sends it nowhere. There are no
analytics, no crash reports and no account to make.

QuotaDial Pro is sold through Lemon Squeezy, which keeps the order. The link in your receipt opens
quotadial.com/key, which asks Lemon Squeezy whether the receipt's key is valid and gives you a
QuotaDial key, keeping and logging nothing. QuotaDial checks that key on your machine, with no
network call.

<!-- /privacy -->

## Build from source

You need Rust. [rustup](https://rustup.rs) installs the right toolchain from `rust-toolchain.toml`.

```sh
git clone https://github.com/pghmatt/quotadial
cd quotadial
cargo build --workspace --release
./target/release/quotadial status          # or the short alias: qd status
./target/release/quotadial status --json
./target/release/quotadial watch           # refreshes every minute; Ctrl-C to stop
./target/release/quotadial daemon          # one poller for everything, and the local API
```

To give Claude Code or Codex the MCP tools, with `quotadial` on your `PATH`:

```sh
claude mcp add quotadial -- quotadial mcp
codex mcp add quotadial -- quotadial mcp
```

Run the checks CI runs:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Attribution

QuotaDial's provider integrations are ported from
[CodexBar](https://github.com/steipete/CodexBar) by Peter Steinberger, used under the MIT License.
See [NOTICE](NOTICE).

## License

MIT. See [LICENSE](LICENSE).
