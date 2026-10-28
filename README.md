# QuotaDial

QuotaDial shows how much of your Claude, ChatGPT/Codex and Antigravity plan limits you have used,
and when they reset, so a long agent run does not stop halfway. It keeps the numbers Claude Code's
and Antigravity CLI's status lines hand it, reads the login your Codex CLI already stores and asks
ChatGPT for your current usage, and puts the answer in your menu bar or system tray, in
your terminal, behind a local HTTP API, and in front of your agents through an MCP server and a
SKILL.md. It runs on macOS, Windows and Linux.

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="site/public/images/app/accounts-dark.png">
    <img src="site/public/images/app/accounts-light.png" width="360" alt="QuotaDial's window with example accounts: Claude Work and Codex, each window with its share used, its reset and, where there is one, its pace">
  </picture>
</p>

<p align="center">
  <img src="docs/images/first-run.gif" width="376" alt="QuotaDial's first run: its icon shows 91% in the menu bar, and a click opens the window with two accounts named Example">
  <br>
  <sub>The first run of QuotaDial Example, a build with made-up accounts: its number in the menu bar, then the window a click opens (<a href="docs/images/first-run.mp4">as an MP4</a>).</sub>
</p>

**Status: pre-alpha.** `quotadial status` and `quotadial watch` show Claude, ChatGPT/Codex and
Antigravity usage. `quotadial daemon` polls for every consumer, keeps the history that gives each window its
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
| Claude | No sign-in: what Claude Code's status line hands `quotadial claude status-line`, once you add it to Claude Code's settings, for every `CLAUDE_CONFIG_DIR` it finds | Works while Claude Code runs, for Pro and Max |
| ChatGPT / Codex | Codex CLI (`auth.json` in `$CODEX_HOME` or `~/.codex`) | Works |
| Gemini CLI | Nothing inside the sign-in: only that Gemini CLI's `oauth_creds.json` is in `.gemini` in `$GEMINI_CLI_HOME` or your home folder, and one field of its `settings.json` | Shown, with why it has no numbers |
| Antigravity | No sign-in: what Antigravity CLI's status line hands `quotadial antigravity status-line`, once you add it to Antigravity CLI's settings | Works while Antigravity CLI runs |

Gemini CLI's account shows with no numbers. Signed in with Google, Google's terms do not let other
tools use Gemini CLI's sign-in, so QuotaDial never opens it and says where Gemini CLI shows its
limits (`/model`). A Gemini API key or Vertex AI has no plan limits to show.

Antigravity's terms do not let other tools use its sign-in either, so QuotaDial never reads it.
Antigravity CLI hands each limit's share and reset to a command named in its settings: add the
line [docs/INSTALL.md](docs/INSTALL.md#antigravity) gives, and its five-hour and weekly windows
show while Antigravity CLI runs, marked stale with the time they are from once it stops.

Claude's numbers come only from Claude Code's status line, never from Claude Code's sign-in, which
QuotaDial never reads. Claude Code hands its status line each Pro or Max plan's five-hour and weekly
windows: add the line [docs/INSTALL.md](docs/INSTALL.md#claude) gives to Claude Code's settings and
start Claude Code again, and they show while Claude Code runs, marked stale with the time they are
from once it stops. If you keep several Claude subscriptions, each signed in through its own
`CLAUDE_CONFIG_DIR`, add the line in each folder and QuotaDial shows all of them. See
[docs/CLI.md](docs/CLI.md#accounts) for where it looks.

QuotaDial only reads the Codex CLI's sign-in. It never writes, refreshes or moves it, and it never
logs it. When it has expired, run `codex` once and QuotaDial picks up the refreshed one.

QuotaDial asks OpenAI at most once every 2 minutes, and backs off when told to. In between, it
shows the last numbers and says how old they are. When ChatGPT cannot be reached, QuotaDial shows the
newest numbers from the Codex CLI's own session logs, marked as coming from them.

<!-- privacy: copied from docs/PRIVACY.md by scripts/privacy.sh; edit it there -->

## What QuotaDial reads and sends

QuotaDial runs on your machine. It reads the sign-in that the Codex CLI already keeps, and never
writes, refreshes, moves or logs it. From the CLIs' session logs it reads only numbers: token counts
and model names for the week's cost, and the Codex CLI's own record of your limits. What you and the
model wrote never leaves the machine, and QuotaDial never reads it.

For Claude, QuotaDial reads no sign-in and asks no one. It reads only the account that Claude Code's
own account file, `.claude.json`, names: its id, address, organisation and plan. Once you add
`quotadial claude status-line` to Claude Code's status line, it keeps what the status line hands
that command, each window's share used and its reset, with the time, in `claude.json` in
QuotaDial's data folder, or in a file of its own for each folder `CLAUDE_CONFIG_DIR` names, readable
only by you.

For Gemini CLI, QuotaDial reads no sign-in. It only checks that Gemini CLI's sign-in file,
`oauth_creds.json` in `~/.gemini`, is there, without opening it, and reads one field of Gemini CLI's
`settings.json` beside it: the one that says whether it signs in with Google, a Gemini API key or
Vertex AI. It asks no one about Gemini CLI and sends nothing about it.

For Antigravity, QuotaDial reads no sign-in and asks no one, and only checks whether Antigravity
CLI's folder, `~/.gemini/antigravity-cli`, is there, without opening anything in it; once you add
`quotadial antigravity status-line` to Antigravity CLI's status line, it keeps what the status line
hands that command, each limit's share or amount left and its reset, the plan and the account's
address, with the time, in `antigravity.json` in QuotaDial's data folder, readable only by you.

QuotaDial connects to these, and nothing else:

- **chatgpt.com**, OpenAI's, with your Codex CLI sign-in, for your ChatGPT/Codex limits, at most
  once every 2 minutes.
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
