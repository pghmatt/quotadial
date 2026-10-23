# QuotaDial

QuotaDial shows how much of your Claude and ChatGPT/Codex plan limits you have used, and when they
reset, so a long agent run does not stop halfway. It reads the login your Claude Code and Codex CLIs
already store, asks each provider for your current usage, and puts the answer in your menu bar or
system tray, in your terminal, behind a local HTTP API, and in front of your agents through an MCP
server and a SKILL.md. It runs on macOS, Windows and Linux.

**Status: pre-alpha.** `quotadial status` and `quotadial watch` show Claude and ChatGPT/Codex
usage. `quotadial daemon` polls for every consumer, keeps the history that gives each window its
pace (how fast it fills, and whether it runs out before it resets), and serves a local HTTP API, and
`quotadial mcp` is an MCP server for agents, with [a skill](skills/quotadial/SKILL.md) that tells
them how to use it. See [docs/CLI.md](docs/CLI.md) for the commands, output and exit codes. The tray app
([apps/tray](apps/tray)) shows the tightest limit in the menu bar or tray and runs the daemon
itself; see [docs/INSTALL.md](docs/INSTALL.md) to install or build it.
Progress is tracked in the [milestones](https://github.com/pghmatt/quotadial/milestones), and the
product spec is in [docs/PRD-public.md](docs/PRD-public.md).

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
