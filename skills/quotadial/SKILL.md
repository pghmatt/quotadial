---
name: quotadial
description: Check how much of the user's Claude and ChatGPT/Codex plan limits is left, and when each limit resets, before starting a long or expensive job, or when the user asks about their usage. Uses QuotaDial's MCP tools, its CLI, or its local HTTP API on 127.0.0.1.
---

# QuotaDial

QuotaDial tracks the usage limits of the Claude and ChatGPT/Codex plans that this machine is
signed in to. It reports each limit's share used, when it resets and how old each number is. Check
it before you start something long, such as a multi-hour run, a large refactor or many subagents,
so the job does not stop halfway. Also check it when the user asks how much they have left.

Claude Code runs on the `claude` accounts; the Codex CLI runs on the `codex` ones.

## How to ask

Use the first of these that is available:

1. **The MCP tools**, if the `quotadial` MCP server is connected:
   - `get_account_headroom` ranks the accounts by the room left in their tightest limit.
   - `get_usage` returns every limit.
   - `list_alerts` stays empty until QuotaDial has alert rules.
   - The first two take an optional `provider`, `claude` or `codex`.
2. **The CLI**: `quotadial status --json`, the same report as `get_usage`. It works whether or not
   the QuotaDial daemon is running.
3. **The HTTP API**, while the daemon runs (`quotadial daemon`). See below.

Ask once before a job, not in a loop. QuotaDial asks Anthropic at most every 5 minutes per account
and ChatGPT at most every 2 minutes, so asking more often returns the same numbers.

## The HTTP API

The daemon listens on `127.0.0.1` only, on port 47115 unless it was started with `--port`. While it
runs, `daemon.json` in QuotaDial's data folder holds the port it chose. Two read-only requests:

| Request | Answer |
| --- | --- |
| `GET /v1/status` | The usage report (see below) |
| `GET /v1/headroom` | The accounts ranked by headroom (see below) |

Every request needs `Authorization: Bearer <token>`, with the token from the `token` file in
QuotaDial's data folder:

| System | Data folder |
| --- | --- |
| macOS | `~/Library/Application Support/quotadial` |
| Linux | `$XDG_DATA_HOME/quotadial`, or `~/.local/share/quotadial` |
| Windows | `%APPDATA%\quotadial` |

`QUOTADIAL_DATA_DIR` replaces the data folder when it is set.

**Never print, log, echo or `cat` the token, and never put it in a command's arguments.** Pass it
the way this macOS example does, so the value never appears in your output or in the process list:

```sh
token="$HOME/Library/Application Support/quotadial/token"
curl -s -H @<(printf 'Authorization: Bearer %s\n' "$(cat "$token")") http://127.0.0.1:47115/v1/headroom
```

Status codes:

| Code | Meaning |
| --- | --- |
| 200 | The answer, as JSON |
| 401 | The token is missing or wrong |
| 403 | The `Host` header is not `127.0.0.1:<port>` or `localhost:<port>` |
| 404, 405 | Another path or method. The API has only the two `GET` requests above |

A refused connection means the daemon is not running. Use `quotadial status --json` instead. Every
error body is `{ "error": "..." }`.

## The usage report

The report from `GET /v1/status`, `get_usage` and `quotadial status --json`:

```json
{
  "snapshots": [
    {
      "account": { "provider": "claude", "account_id": "...", "display_name": "user@example.com", "plan": "max" },
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
  "errors": []
}
```

| Field | Meaning |
| --- | --- |
| `snapshots[]` | One per account: a provider, its organization or workspace (`org`), and an account id |
| `windows[].kind` | `session` (5 hours), `weekly`, `model_weekly` (one model's weekly limit, named in `scope_model`), `daily`, `credits`, or `other` for a limit QuotaDial does not know yet |
| `windows[].used_pct` | The share of the limit used, from 0 to 100 |
| `windows[].resets_at` | When the window resets, in UTC. Left out when the provider does not say |
| `windows[].source` | `api`, `cache` (an earlier answer shown again) or `local_log` (the Codex CLI's own logs) |
| `windows[].fetched_at` | When the number was fetched, in UTC |
| `windows[].unconfirmed` | `true` on a sudden 0% that a second fetch has not confirmed yet. Do not trust that 0% |
| `extra_usage` | Pay-as-you-go use beyond the plan: `enabled`, and when known `used_pct`, `used`, `limit`, `currency`, or `balance` (credits left) |
| `status` | `ok`; `stale` or `rate_limited` (older numbers shown again); `auth_required` (the user must sign in again); `unsupported` |
| `error` | Why the status is not `ok`, naming the fix, such as "Run `claude` to sign in again." |
| `errors[]` | Providers or accounts that produced nothing, each with a message naming the fix |

## Headroom

The ranking from `GET /v1/headroom` and `get_account_headroom`:

```json
{
  "accounts": [
    {
      "account": { "provider": "claude", "account_id": "...", "display_name": "user@example.com", "plan": "max" },
      "status": "ok",
      "headroom_pct": 58.0,
      "tightest": { "kind": "session", "label": "Session", "used_pct": 42.0, "resets_at": "2099-01-01T05:00:00Z", "window_seconds": 18000, "source": "api", "fetched_at": "2099-01-01T00:00:00Z" }
    }
  ],
  "errors": []
}
```

- `headroom_pct` is 100 minus the `used_pct` of the account's tightest window, the one closest to
  its limit.
- The account with the most headroom comes first. Equal headroom goes to the account whose tightest
  window resets sooner. Accounts without windows come last, with their `status` and `error`.
- A `tightest.resets_at` in the past means that window has reset since the numbers were fetched.

## Deciding

- Look at the accounts for the provider you run on. Trust `status: "ok"` most. Treat `stale` and
  `rate_limited` as a few minutes old. For `auth_required`, tell the user the fix in `error`.
- Suppose the tightest window is above about 80% used and resets hours from now. Tell the user
  before starting a long job, with when it resets (`resets_at`), so they can choose to wait, narrow
  the job, or use another account. A near-empty window needs no comment.
- Say how old the numbers are when it matters. `fetched_at` can be minutes ago.
- QuotaDial only reads. Never write to its files, and never touch the Claude Code or Codex sign-ins
  it reads.
