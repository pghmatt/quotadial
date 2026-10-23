# QuotaDial tray app

The tray app puts the share used of your tightest Claude or ChatGPT/Codex limit in the menu bar or
system tray. On macOS and Linux it shows a dial with the number beside it. On Windows it shows the
number drawn into the icon. Its menu and tooltip list every account. Clicking it opens a popover
beside it with every window: a bar, the reset, where each number came from and how old it is.

When no QuotaDial daemon is running, the app runs one itself, through the same code as
`quotadial daemon`. The `quotadial` command and the MCP server then read the app's numbers, and
nothing polls twice. When a standalone daemon already runs, the app follows it and takes over when
it stops. The app carries the `quotadial` command inside it; see [INSTALL.md](../../docs/INSTALL.md).

## Layout

| Path | Contents |
| --- | --- |
| `src/` | The popover: Preact and TypeScript, plain CSS, under 150 KB of JavaScript |
| `src-tauri/src/feed.rs` | Runs the daemon, or follows one already running |
| `src-tauri/src/meter.rs` | What the tray and the popover show, built from the shared wording |
| `src-tauri/src/icon.rs` | The tray icons, drawn in code: the dial, and the Windows number |
| `src-tauri/src/lib.rs` | The Tauri app: tray, menu, the popover's placing and hiding, and quitting cleanly |
| `scripts/prepare-cli.mjs` | Builds the CLI the app bundles |
| `scripts/check-size.mjs` | Fails the build when the window's JavaScript passes 150 KB |
| `app-icon.svg` | The app icon; `npx tauri icon app-icon.svg -o src-tauri/icons` makes the sizes |

## Develop

You need Rust and Node.js 22 or later. On Linux you also need the libraries listed in
[INSTALL.md](../../docs/INSTALL.md#build-it-yourself).

```sh
npm ci
npx tauri dev       # builds the CLI and the window, then runs the app
npx tauri build     # the app and its installer, in src-tauri/target/release/bundle
```

`src-tauri` is its own Cargo workspace, so run its checks from there after `npm run build` and
`npm run cli`: `cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings` and
`cargo test`.
