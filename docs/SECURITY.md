# QuotaDial's threat model

What QuotaDial is trusted with, where it keeps it, what it sends, and what an attacker in each
position can and cannot do. [`SECURITY.md`](../SECURITY.md) says how to report a vulnerability;
[`PRIVACY.md`](PRIVACY.md) lists every address QuotaDial connects to.

## What QuotaDial is trusted with

- **The sign-ins Claude Code and the Codex CLI keep.** QuotaDial reads them from the macOS Keychain,
  `~/.claude/.credentials.json`, `CLAUDE_CODE_OAUTH_TOKEN` and the Codex CLI's `auth.json`. It never
  writes, refreshes or moves them, and never logs them. A token is held in memory only, in a type
  that wipes it when dropped (`quotadial_core::Secret`), and sent only to the provider it belongs
  to, over TLS. Every error and every text from outside goes through `redact_secrets` before it is
  shown or saved, and a property test proves no token survives it.
- **Usage numbers and history**, which are the user's, kept on the machine.
- **Webhook and Slack addresses** in `config.json`, which work like passwords: never printed or
  logged, shown by their host only, and QuotaDial warns when other users can read the file.
- **The licence key**, checked on the machine against public keys built into QuotaDial
  ([`LICENSING.md`](LICENSING.md)).

## Where QuotaDial keeps things

| Where | What | Protection |
| --- | --- | --- |
| The data folder (`~/Library/Application Support/quotadial`, `$XDG_DATA_HOME/quotadial`, `%APPDATA%\quotadial`) | `history.db` (readings, snapshots, alert rules, firings), `costs.db` (token counts from the logs, never their content), `token` (the local API's bearer token), `licence`, `daemon.json`, `daemon.lock` | The folder is the user's only (0700 on macOS and Linux; a per-user folder on Windows), and each file QuotaDial writes is 0600 |
| `config.json` in the data folder | Settings, including webhook addresses | Written by the user; QuotaDial warns when others can read it |
| The app's config folder | `settings.json`, `alerts.json`, `updates.json` | 0600 |
| The cache folder (`quotadial/` in the OS cache directory) | When each account may be asked next, and its last answer | Private permissions; derived data only, never a token |
| The Downloads folder | Only what the user saves from History: Export CSV's readings, and on QuotaDial Pro a picture of an account's history (D-115) | The folder's own permissions, as for any download; never a token, a picture never shows an address, and a picture is never written over another file |

A damaged history is moved aside under a dated name, never deleted, so it stays for support
(D-108).

## What goes over the network

Only what [`PRIVACY.md`](PRIVACY.md) lists, each over TLS: the two providers' usage endpoints, their
public status pages, GitHub for updates, and the webhooks the user adds, which must be `https`,
or `http` to the machine itself. Tests hold every address in the code to that list (D-106).

QuotaDial's example build (D-117), made with the `fake` feature for pictures of the app and never
released, polls no provider and reads no sign-in, so the 5-minute floor for Anthropic is untouched;
its accounts are made up, and it keeps folders, a lock, a token and a port of its own.

Its Gemini CLI provider (D-118, D-122), built with the `gemini` feature and not yet in a release,
has no network client and sends nothing. It checks only that Gemini CLI's sign-in file is there,
never opening it, and reads one field of Gemini CLI's `settings.json`, how it signs in; the rest of
that file, which can hold other tools' secrets, is skipped without a copy, and the bytes read are
wiped. Google's and GitHub's tokens are blanked out of any text QuotaDial shows or saves, by their
shape, before any provider can read one (D-122).

Updates are checked twice before they are installed: against the public key built into the app,
and against the version that key's signature names, so neither a changed file nor an older release
passed off as newer is installed (D-103). Nothing downloads until the user chooses Update.

## Who could attack, and what they can do

### Another program running as the same user

It can do what the user can: read the CLIs' sign-ins where they keep them, and QuotaDial's data
and API token. The operating system does not separate programs one user runs, and QuotaDial does
not pretend to. It does not make things worse: it adds no copy of a sign-in to disk, logs no token,
and its API token unlocks only what QuotaDial shows, which holds no sign-in.

### Another user on the same machine

They cannot read QuotaDial's folders or files. They can connect to the local API on `127.0.0.1`,
but every request needs the bearer token, which is in the user's own `token` file, so they get
401 and nothing else.

### A website in the user's browser

It cannot read the local API. The API needs the bearer token, which a page cannot read. It answers
only requests addressed to `127.0.0.1` or `localhost`, so a site that points its own name at
`127.0.0.1` is refused. It sends no headers that would let a browser show a cross-site page its
answers. Tests cover the token, the address and the refusal.

### Someone on the same network

They cannot reach the local API, which listens on `127.0.0.1` only, nor the MCP server, which talks
over the standard input and output of the agent that starts it. They see TLS connections to the
addresses in [`PRIVACY.md`](PRIVACY.md), and can block them; QuotaDial then shows the last numbers,
marked stale with their age, and asks again within a minute (D-108).

### Someone in the path of an update

They cannot install code. A changed file fails the signature check, and an older release passed off
as newer fails the version check, so either is refused with one sentence and the app stays as it
is; both were checked live (D-103). They can delay an update by blocking it.

### Someone holding one of QuotaDial's keys

- **The updater key** signs updates, but a signed update also has to be published as the latest
  release of this repository to reach anyone. The key lives in the repository's secrets and Matt's
  password manager; [`RELEASE.md`](RELEASE.md) says how to change it.
- **The licence keys** can only make Pro licences; they reach no one's data. The root key stays on
  Matt's Mac and the issuing key in the website's settings;
  [`LICENSING.md`](LICENSING.md#if-the-issuing-key-leaks) says what to do if one leaks.

### Anything outside that QuotaDial reads

Provider answers, status pages, the CLIs' logs, `config.json`, licence keys, release notes and what
an agent sends the MCP server are all read as data, never run. A property test over each parser
proves no input can make it panic, and found one overflow, now fixed (#52). Text from outside is
cut to one line and redacted before it is shown. The app's window runs only its own code, under a
content security policy (`default-src 'self'`), can call only the commands the app's own panels
use, and can open only the links Help names. A picture it sends to save is written only if the
copy is QuotaDial Pro and the bytes are a whole PNG of exactly 2400 by 1350 under 4 MB, read by
hand, and only into the Downloads folder, under a name the app makes from what it knows of the
account, never over another file.

### Agents through MCP

The MCP server answers three questions: usage, headroom and alert rules. On QuotaDial Free it
answers only the first. None of them writes a file or sends a request anywhere, and none answers
with a token: the report names where a sign-in is kept, as `quotadial status` does, never what it
holds.

## Supply chain

`cargo deny` (licences, advisories, sources) and `cargo audit` run in CI on both Rust workspaces.
Each advisory either fails the build or is excepted in `deny.toml` with why it cannot reach a user.
Releases are built only by the Release workflow, and each carries `SHA256SUMS.txt` and a software
bill of materials for the command and the app ([`RELEASE.md`](RELEASE.md)). Each download's copy
named without the version is the same signed file under a second name, listed in `SHA256SUMS.txt`
under both; the updater and the package managers name only the versioned file (D-116).

## The one `unsafe` exception

QuotaDial's code is free of `unsafe`: the core workspace forbids it, and the tray app denies it
with one exception, the module `apps/tray/src-tauri/src/icon_press.rs` (#64, D-114). On macOS,
VoiceOver and Full Keyboard Access press the menu bar icon rather than click it, and the tray
library answers only clicks, so the module gives the icon's button an action of its own through
AppKit's target and action, which objc2 marks `unsafe`. Its `unsafe` blocks make the target
object, set it as the button's target and action, and, for the press test alone, press the button
as the tray library does. A press can only run the same toggle a click runs, opening or closing the
popover; it carries no data. The exception is one `#[expect(unsafe_code, ...)]` on the module's
line in `lib.rs`, so any other `unsafe` in the app still fails to build.

## Known limits

- A program running as the same user can read what the user can, as above.
- The local API's token is a bearer token in a file; whatever can read the file can use it.
- `latest.json`, which says what the newest version is, is not signed itself; the version bound
  into each update's signature is what stops an older release being passed off as newer.
- A clock set back can misplace one reset alert (D-108).
