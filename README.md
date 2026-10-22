# QuotaDial

QuotaDial shows how much of your Claude and ChatGPT/Codex plan limits you have used, and when they
reset, so a long agent run does not stop halfway. It reads the login your Claude Code and Codex CLIs
already store, asks each provider for your current usage, and puts the answer in your menu bar or
system tray, in your terminal, behind a local HTTP API, and in front of your agents through an MCP
server and a SKILL.md. It runs on macOS, Windows and Linux.

**Status: pre-alpha.** Nothing works end to end yet. Progress is tracked in the
[milestones](https://github.com/pghmatt/quotadial/milestones), and the product spec is in
[docs/PRD-public.md](docs/PRD-public.md).

## Provider support

| Provider | Reads the login from | Status |
| --- | --- | --- |
| Claude | Claude Code (`~/.claude/.credentials.json`, or the macOS Keychain) | Planned (M1) |
| ChatGPT / Codex | Codex CLI (`~/.codex/auth.json`) | Planned (M1) |

QuotaDial only reads these credentials. It never writes, refreshes or moves them, and it never logs
them.

## Build from source

You need Rust. [rustup](https://rustup.rs) installs the right toolchain from `rust-toolchain.toml`.

```sh
git clone https://github.com/pghmatt/quotadial
cd quotadial
cargo build --workspace --release
./target/release/quotadial status          # or the short alias: qd status
./target/release/quotadial status --json
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
