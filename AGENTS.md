# QuotaDial agent guide

QuotaDial tracks AI coding-subscription usage limits on macOS, Windows and Linux. Version 1 covers
Claude and ChatGPT/Codex. It reads the OAuth credentials the official CLIs already store, calls each
provider's usage endpoint, normalizes the result, and serves it through a tray app, a CLI
(`quotadial`, alias `qd`), a local HTTP API, an MCP server and a SKILL.md. It is sold with a free
and a Pro tier. Domains: quotadial.com and quotadial.app.

Stack: Rust core (reqwest, tokio, rusqlite, axum, keyring), a Tauri 2 + TypeScript/React tray app,
and a thin CLI.

## Source of truth

- `docs/PRD.md` is the product spec: provider endpoints, the normalized model, architecture and
  milestones. Follow it. Do not re-research or re-decide anything it settles.
- `docs/DECISIONS.md` records every choice the PRD left open. Add an entry when you make one.
- `docs/TASK_TRACKER.md` records progress and implementation notes. Update it with every change.
- CodexBar (MIT, Peter Steinberger, https://github.com/steipete/CodexBar) is the reference
  implementation we port provider logic from. Keep a clone at `../codexbar-ref`. Port the logic,
  name the CodexBar source file in the module docs, and never vendor its UI. Credit lives in
  `NOTICE`.

## Layout

| Path | Contents |
| --- | --- |
| `crates/core` (`quotadial-core`) | Normalized model (`Account`, `Window`, `Snapshot`, `Status`) and the `Provider` trait |
| `crates/providers/claude` (`quotadial-provider-claude`) | Claude provider |
| `crates/providers/codex` (`quotadial-provider-codex`) | ChatGPT/Codex provider |
| `crates/api` (`quotadial-api`) | Local HTTP API |
| `crates/cli` (package `quotadial`) | The `quotadial` and `qd` binaries |
| `apps/tray` | Tauri 2 + TypeScript/React tray app (placeholder) |
| `skills/quotadial/SKILL.md` | Agent skill (placeholder) |
| `fixtures/<provider>/` | Redacted provider responses used by tests |
| `docs/` | PRD, decisions log, task tracker |

## Conventions

- Conventional Commits: `feat:`, `fix:`, `docs:`, `test:`, `refactor:`, `ci:`, `chore:`, with a
  scope where it helps (`feat(claude): parse weekly window`). Keep commits small, one concern each.
- One crate per provider under `crates/providers/<name>`, each implementing
  `quotadial_core::Provider`. Provider-specific code stays out of core, the CLI, the API and the
  tray app.
- Every `Snapshot` carries `fetched_at` and `source`. Nothing reaches a user without both.
- Never write provider credential files or keychain entries. QuotaDial only reads what the official
  CLIs store. Refreshing or re-authenticating is the CLI's job.
- Never print, log or commit tokens, cookies or API keys. That includes error messages, `Debug`
  output, test snapshots and fixtures. Redact before a value leaves the function that read it.
- Unredacted captures live under `fixtures/**/raw/`, which is gitignored. Only redacted fixtures
  are committed. See `fixtures/README.md`.
- Poll Anthropic usage endpoints no more than once every 5 minutes per account. Enforce the floor in
  code.
- `cargo fmt`, `cargo clippy -D warnings` and `cargo test` must pass on macOS, Windows and Linux
  before anything merges to `main`. CI enforces this and `main` is protected.

## Commands

```sh
cargo build --workspace                                 # build everything
cargo test --workspace                                  # run every test
cargo clippy --workspace --all-targets -- -D warnings   # lint, exactly as CI runs it
cargo fmt --all                                         # format (CI runs it with -- --check)
cargo run -p quotadial -- status                        # human-readable usage status
cargo run -p quotadial -- status --json                 # the same as JSON
cargo run -p quotadial --bin qd -- status               # via the qd alias
```
