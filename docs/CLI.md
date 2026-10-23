# The QuotaDial CLI

`quotadial` (and its short alias `qd`) shows how much of each Claude and ChatGPT/Codex plan limit is
used, when each resets, and how old every number is. It reads the sign-ins Claude Code and the Codex
CLI already store and never writes them.

## Commands

| Command | What it does |
| --- | --- |
| `quotadial status` | Shows every account this machine is signed in to, once, as text |
| `quotadial status --json` | The same, as JSON, for scripts and agents |
| `quotadial watch` | Keeps showing the status, refreshing every minute, until Ctrl-C |
| `quotadial watch --json` | One line of JSON per refresh, the same report as `status --json` |
| `quotadial daemon` | Runs the daemon until Ctrl-C: the one poller every consumer shares, with history and the local API |
| `quotadial daemon --port <port>` | The same, with the API on another port; 0 lets the system choose a free one |
| `quotadial mcp` | Runs the MCP server named `quotadial` on standard input and output, for agents |
| `quotadial --version` | Prints the version |

## How often providers are asked

No command asks a provider more often than its own limit, counted across every QuotaDial process
on the machine: Anthropic at most once every 5 minutes per account, and ChatGPT at most once every
2 minutes. In between, a command shows the last numbers again, marked as coming from the cache,
with their age. `watch` refreshes every minute, so most refreshes show cached numbers whose age
keeps growing until the provider may be asked again. While the daemon runs, `status` and `watch`
ask it instead of asking providers themselves.

When a provider answers 429, QuotaDial waits as long as the `Retry-After` header says, or else 5,
10, 20, 40 and then 60 minutes as 429s repeat, and keeps the last numbers on screen meanwhile.
When ChatGPT cannot be reached and nothing is cached, the newest numbers from the Codex CLI's own
session logs stand in, marked as coming from them.

## Accounts

QuotaDial shows every subscription whose sign-in is stored on the machine, not only the one the
current shell would use. Each account has its own floor, cache and history.

- **Claude.** First the sign-in Claude Code would use here: `CLAUDE_CODE_OAUTH_TOKEN` when it is
  set, or else the one for `CLAUDE_CONFIG_DIR`, or for `~/.claude` when that is not set. Then every
  other Claude Code sign-in on the machine, including the one Claude Code keeps for this folder when
  a token in the environment comes first:
  - On macOS, every Keychain item named `Claude Code-credentials`, or that, a dash and 8 hex
    characters, as Claude Code names them. QuotaDial lists them by name without reading them, and
    reads an item's token only when it asks about that account. If the Keychain cannot be listed,
    QuotaDial looks up each folder's item by its name instead and says so in `errors`. Only an
    item whose folder it does not know is missing until the listing works.
  - Claude Code names each item after the folder it was made for, so QuotaDial matches the items to
    folders: other `~/.claude…` folders, the folders in `~/.codus/accounts`, and those in
    `extra_config_dirs` (see Settings below). A link to a folder counts as the folder.
  - On Linux and Windows, `.credentials.json` in those same folders.
- **Codex.** The sign-in the Codex CLI keeps in `auth.json`, in `~/.codex` or the folder
  `CODEX_HOME` names.
- **One row per subscription.** A subscription found in two places, such as two folders signed in
  to the same account, shows once, and is asked about no more than once every 5 minutes, whichever
  sign-in asks. A sign-in that does not say whose it is, such as `CLAUDE_CODE_OAUTH_TOKEN`, is
  named by Anthropic's profile endpoint, once per token, before its usage is asked.
- **Where each sign-in is.** `account.sign_in` in the JSON says where QuotaDial read it, such as
  `the macOS Keychain item "Claude Code-credentials-a1b2c3d4" (CLAUDE_CONFIG_DIR ~/.claude-work)`.
- **A sign-in that expired or was rejected** shows `sign-in required`, with the command that signs
  that folder in again, such as "Run `CLAUDE_CONFIG_DIR=~/.claude-work claude` once to refresh
  it." On Windows the command is for PowerShell: `$env:CLAUDE_CONFIG_DIR = '…'; claude`. Its last
  good numbers stay on show, marked as coming from the cache. QuotaDial never refreshes or writes a
  sign-in itself.
- **A Keychain item whose folder QuotaDial cannot find** still shows, named by its item. Its fix
  asks for `claude` with the `CLAUDE_CONFIG_DIR` that made the item. If its sign-in holds no Claude
  account, as when a folder only ever signed in to MCP servers, it is left out until the item
  changes.
- **Order.** Claude's accounts come first, then Codex's, each sorted by name.

The daemon looks for new and removed sign-ins every minute.

## Settings

Settings that every QuotaDial process reads (the CLI, the daemon and the tray app) go in
`config.json` in the data directory (see The daemon below). The file is optional. A missing or
damaged one means the defaults, and a setting of the wrong kind keeps its default while the others
apply. A change applies the next time a command, the daemon or the app starts.

```json
{
  "extra_config_dirs": ["~/work/claude", "~/claude-accounts/*"],
  "max_accounts": 3
}
```

| Setting | Meaning |
| --- | --- |
| `extra_config_dirs` | More folders where Claude Code keeps sign-ins, as `CLAUDE_CONFIG_DIR` would name them: full paths, or paths starting with `~` for the home folder. A folder ending in `/*` adds every folder inside it. An entry that is not a full path is left out, with a warning in `errors`. None by default |
| `max_accounts` | At most this many accounts per provider, 1 or more, the current shell's own first. Two sign-ins of one subscription count once, once each is named. All of them by default |

The tray app keeps its own settings, such as the names given to accounts, in its settings panel
(`docs/INSTALL.md`).

## The daemon

`quotadial daemon` runs until Ctrl-C, or on Linux and macOS until it is sent `SIGTERM`, as a
service manager does. While it runs it is the only thing that asks providers for usage: `status`,
`watch`, the local API (next) and the MCP server read what it last found. The tray app runs the
same daemon itself when none is running, so with the app open there is no need to start one.

- It asks each account no more often than its provider's floor, counted from when its last answer
  came back, and slows to every 10 minutes once the account's numbers have been still for 10
  minutes. A change in usage is what counts as activity.
- Every snapshot that brings new data goes into a SQLite history (WAL mode), kept for 30 days, in
  `quotadial/history.db` in the OS data directory: `~/Library/Application Support` on macOS,
  `$XDG_DATA_HOME` or `~/.local/share` on Linux, `%APPDATA%` on Windows. Set `QUOTADIAL_DATA_DIR` to
  put it elsewhere. The folder and its files are private to the user.
- Only one daemon runs at a time; a second one says so and exits with status 1.
- A restarted daemon shows the numbers it stored last, marked as coming from the cache, until its
  first poll.
- It serves the local API on `127.0.0.1:47115`, or the port given with `--port`, and starts by
  printing where the API is and where its token is kept, never the token itself. If the port is
  taken, it says so and exits with status 1.
- `status` and `watch` find it through `daemon.json` in the same folder, which holds its process id
  and port while it runs. With no daemon running they fetch for themselves. If a daemon is running
  but its API does not answer, they say so and name the fix rather than polling in its place. A
  daemon that stops while being asked, as when the tray app quits, hands over: once its lock is
  released, they fetch for themselves.

## The local API

The daemon answers two requests, on `127.0.0.1` only:

| Request | Answer |
| --- | --- |
| `GET /v1/status` | The same report as `status --json` (see JSON output below) |
| `GET /v1/headroom` | Every account ranked by the room left in its tightest window |

Every request needs the token from the `token` file in the data directory, as
`Authorization: Bearer <token>`. The daemon creates it on its first start, 64 hex characters that
only the user can read, and keeps it across restarts. A request without it, or with another, gets
401. A request whose `Host` is not `127.0.0.1:<port>` or `localhost:<port>` gets 403, so a web page
cannot reach the API by pointing a name of its own at `127.0.0.1`. Errors are JSON, such as
`{ "error": "..." }`, with 404 for any other path and 405 for any other method.

On macOS, for example:

```sh
token="$HOME/Library/Application Support/quotadial/token"
curl -H @<(printf 'Authorization: Bearer %s\n' "$(cat "$token")") http://127.0.0.1:47115/v1/headroom
```

`printf` is built into the shell, so the token never appears in a command line that other
processes can list.

```json
{
  "accounts": [
    {
      "account": { "provider": "claude", "account_id": "...", "display_name": "user@example.com", "plan": "max" },
      "status": "ok",
      "headroom_pct": 92.0,
      "tightest": {
        "kind": "session",
        "label": "Session",
        "used_pct": 8.0,
        "resets_at": "2099-01-01T05:00:00Z",
        "window_seconds": 18000,
        "source": "api",
        "fetched_at": "2099-01-01T00:00:00Z"
      }
    }
  ],
  "errors": []
}
```

| Field | Meaning |
| --- | --- |
| `accounts[]` | Every account with a snapshot, the most headroom first. Equal headroom goes to the account whose tightest window resets sooner. Accounts without windows come last, and so do accounts that cannot be used now, whose status is `auth_required` or `unsupported` |
| `headroom_pct` | 100 minus the tightest window's `used_pct`, from 0 to 100. Left out when the account has no windows or cannot be used now: a sign-in that expired keeps its last numbers on show, but their room is not there to use |
| `tightest` | The window closest to its limit, as in `status --json`: when it resets, its length and when it was fetched. For an account that cannot be used now, the last one known. Left out when the account has no windows |
| `status`, `error` | As in `status --json` |
| `errors[]` | As in `status --json` |

Headroom uses the numbers as last fetched. A `tightest.resets_at` in the past means the window has
reset since, and the daemon fetches again within 10 minutes.

## The MCP server

`quotadial mcp` is an MCP server named `quotadial` that an agent starts and talks to over standard
input and output. To add it, with `quotadial` on your `PATH`:

```sh
claude mcp add quotadial -- quotadial mcp     # Claude Code
codex mcp add quotadial -- quotadial mcp      # Codex
```

Its three tools only read:

| Tool | Answer |
| --- | --- |
| `get_usage` | The same report as `status --json` |
| `get_account_headroom` | The same ranking as `GET /v1/headroom` |
| `list_alerts` | `{ "alerts": [] }`: QuotaDial has no alert rules yet, so the list stays empty until they arrive |

`get_usage` and `get_account_headroom` take an optional `provider`, `claude` or `codex`, that limits
the answer to that provider's accounts. Each answer comes as JSON text and, for agents that speak
MCP 2025-06-18 or later, as structured content too. Like `status`, the server asks the daemon while
one runs and otherwise fetches for itself, keeping every provider's floor. A wrong argument, or a
running daemon that does not answer, comes back as a tool error that names the fix.

[`skills/quotadial/SKILL.md`](../skills/quotadial/SKILL.md) tells agents when to check their usage
and how, including the HTTP API's shape. Copy its folder into `~/.claude/skills/` for Claude Code or
`~/.codex/skills/` for Codex.

## Text output

```text
claude user@example.com (max): ok
  Session: 42% used, resets in 2 h 10 min, from the API 4 min ago
  Weekly: 7% used, resets in 3 d 10 h, from the API 4 min ago
  Extra usage: off
codex user@example.com (pro): stale
  Weekly: 54% used, resets in 6 d 5 h, from local logs 2 min ago
  Extra usage: on
  From the Codex CLI's session log at 2026-10-23T12:14:11Z, because ChatGPT's usage endpoint could not be read just now.
```

- The first line of each account names the provider, the account, its plan and the snapshot's
  status: `ok`, `stale`, `rate limited`, `sign-in required` or `unsupported`.
- Each window shows the share used, when it resets, where the number came from (`the API`, `cache`
  or `local logs`) and how long ago it was fetched. The window closest to its limit comes first. A
  window whose numbers predate its reset says `reset 3 min ago`.
- A window that suddenly reads 0% before its reset, where it read more last time, says
  `unconfirmed until the next fetch agrees`. A provider sometimes answers 0% by mistake, so
  QuotaDial waits for a second fetch before trusting it.
- A status other than `ok` is followed by one sentence that says why and names the fix, such as
  "Run `claude` to sign in again."
- A provider that produced nothing at all shows as `error: <provider>: <what happened>`, which also
  names the fix or when QuotaDial tries again.

`watch` clears the terminal before each refresh and ends with when it was updated and how often
each provider is asked. Piped into another program, and on Windows, it prints each refresh after
the last, without escape codes.

## JSON output

`status --json` prints one object; `watch --json` prints one such object per line.

```json
{
  "snapshots": [
    {
      "account": {
        "provider": "claude",
        "account_id": "00000000-0000-0000-0000-000000000001",
        "display_name": "user@example.com",
        "plan": "max",
        "org": "00000000-0000-0000-0000-000000000002",
        "sign_in": "the macOS Keychain item \"Claude Code-credentials\""
      },
      "windows": [
        {
          "kind": "session",
          "label": "Session",
          "used_pct": 42.0,
          "resets_at": "2099-01-01T05:00:00Z",
          "window_seconds": 18000,
          "source": "api",
          "fetched_at": "2099-01-01T00:00:00Z"
        }
      ],
      "extra_usage": { "enabled": false },
      "status": "ok"
    }
  ],
  "errors": [
    { "provider": "codex", "message": "codex: could not read credentials from ..." }
  ]
}
```

| Field | Meaning |
| --- | --- |
| `snapshots[]` | One per account, Claude's first, each provider's sorted by name. An account is a provider, an organization or workspace (`org`) and an account id, so one person in two organizations is two accounts |
| `account.plan`, `account.org` | Left out when the provider does not say |
| `account.sign_in` | Where QuotaDial read the account's sign-in, written for people with `~` for the home folder, such as `the macOS Keychain item "Claude Code-credentials-a1b2c3d4" (CLAUDE_CONFIG_DIR ~/.claude-work)` or `~/.codex/auth.json`. Never a token. Left out when unknown |
| `windows[].kind` | `session`, `weekly`, `model_weekly`, `daily`, `credits`, or `other` for a limit QuotaDial does not know yet, shown by its provider's name with `window_seconds` 0 |
| `windows[].used_pct` | The share of the limit used, from 0 to 100 |
| `windows[].resets_at` | When the window resets, in UTC. Left out when the provider does not say |
| `windows[].window_seconds` | The window's length, such as 18000 for five hours or 604800 for a week |
| `windows[].scope_model` | For a per-model limit, the model it covers. Left out otherwise |
| `windows[].source` | `api`, `cache` (an earlier answer shown again) or `local_log` (the CLI's own logs) |
| `windows[].fetched_at` | When the number was fetched, in UTC. Every number carries its age |
| `windows[].unconfirmed` | `true` when the window reads 0% where it read more on the last fetch, before its reset, until a second fetch agrees. Left out otherwise |
| `extra_usage` | Pay-as-you-go usage beyond the plan: `enabled`, and when known `used_pct`, `used`, `limit` and `currency`, or `balance`, the credits left in the provider's own units. Left out when the provider does not say |
| `status` | `ok`, `stale`, `auth_required`, `rate_limited` or `unsupported` |
| `error` | One sentence that explains a status other than `ok` and names the fix. Left out otherwise |
| `errors[]` | Providers or accounts that produced no snapshot, and places a provider could not look in while finding its accounts, each with a message that names the fix |

Times are RFC 3339 in UTC. No token, cookie or API key ever appears in the output.

## Exit codes

| Code | `status` | `watch` |
| --- | --- | --- |
| 0 | Every account produced a snapshot, whatever its status. Check each snapshot's `status` | Its output was closed, as when piped into `head` |
| 1 | At least one provider or account produced none, or a provider could not look everywhere for accounts (see `errors`), the daemon is running but its API did not answer, or the output could not be written | The output could not be written |
| 2 | The arguments were not understood | The same |

`mcp` exits with 0 when the agent closes its input, and 1 if it cannot read its input or write
its answers. Stopped with Ctrl-C, `watch` ends the way any program interrupted by Ctrl-C does. When a running
daemon does not answer, `status` prints why on standard error; `watch` shows it in place of the
status, or with `--json` prints it on standard error and keeps refreshing.

An agent deciding whether to start a long job can read `status --json`, find the tightest window
with `"status": "ok"` and a fresh `fetched_at`, and compare its `used_pct` and `resets_at` with the
job's needs. With the daemon running, `GET /v1/headroom` does that ranking for it.
