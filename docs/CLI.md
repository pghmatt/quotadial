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
| `quotadial daemon` | Runs the daemon until Ctrl-C: the one poller every consumer shares, with history |
| `quotadial --version` | Prints the version |

## How often providers are asked

No command asks a provider more often than its own limit, counted across every QuotaDial process
on the machine: Anthropic at most once every 5 minutes per account, and ChatGPT at most once every
2 minutes. In between, a command shows the last numbers again, marked as coming from the cache,
with their age. `watch` refreshes every minute, so most refreshes show cached numbers whose age
keeps growing until the provider may be asked again.

When a provider answers 429, QuotaDial waits as long as the `Retry-After` header says, or else 5,
10, 20, 40 and then 60 minutes as 429s repeat, and keeps the last numbers on screen meanwhile.
When ChatGPT cannot be reached and nothing is cached, the newest numbers from the Codex CLI's own
session logs stand in, marked as coming from them.

## The daemon

`quotadial daemon` runs until Ctrl-C. While it runs it is the only thing that asks providers for
usage, and the local API (next) and the MCP server read what it last found.

- It asks each account no more often than its provider's floor, and slows to every 10 minutes once
  the account's numbers have been still for 10 minutes. A change in usage is what counts as
  activity.
- Every snapshot that brings new data goes into a SQLite history (WAL mode), kept for 30 days, in
  `quotadial/history.db` in the OS data directory: `~/Library/Application Support` on macOS,
  `$XDG_DATA_HOME` or `~/.local/share` on Linux, `%APPDATA%` on Windows. Set `QUOTADIAL_DATA_DIR` to
  put it elsewhere. The folder and its files are private to the user.
- Only one daemon runs at a time; a second one says so and exits with status 1.
- A restarted daemon shows the numbers it stored last, marked as coming from the cache, until its
  first poll.

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
        "org": "00000000-0000-0000-0000-000000000002"
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
| `snapshots[]` | One per account. An account is a provider, an organization or workspace (`org`) and an account id, so one person in two organizations is two accounts |
| `account.plan`, `account.org` | Left out when the provider does not say |
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
| `errors[]` | Providers or accounts that produced no snapshot, each with a message that names the fix |

Times are RFC 3339 in UTC. No token, cookie or API key ever appears in the output.

## Exit codes

| Code | `status` | `watch` |
| --- | --- | --- |
| 0 | Every account produced a snapshot, whatever its status. Check each snapshot's `status` | Its output was closed, as when piped into `head` |
| 1 | At least one provider or account produced none (see `errors`), or the output could not be written | The output could not be written |
| 2 | The arguments were not understood | The same |

Stopped with Ctrl-C, `watch` ends the way any program interrupted by Ctrl-C does.

An agent deciding whether to start a long job can read `status --json`, find the tightest window
with `"status": "ok"` and a fresh `fetched_at`, and compare its `used_pct` and `resets_at` with the
job's needs.
