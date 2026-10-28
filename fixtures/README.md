# Fixtures

Provider responses that drive the provider crates' tests.

| Path | Committed | Contents |
| --- | --- | --- |
| `fixtures/<provider>/*.json` | Yes | Redacted or synthetic responses, safe to publish |
| `fixtures/<provider>/raw/` | Never (gitignored) | Unredacted captures from a live account |

Synthetic fixtures are hand-written in the real response shape with obviously fake values. They are
named `*.synthetic.json`. Once a redacted capture exists, a synthetic fixture stays only to cover
shapes the live account does not show.

## Claude

| File | Source | Covers |
| --- | --- | --- |
| `status-line.synthetic.json` | The example in Claude Code's docs, "Customize your status line" (code.claude.com/docs/en/statusline), read on 2026-10-28, as written there | What Claude Code's status line hands `quotadial claude status-line`: `rate_limits.five_hour` and `seven_day`, each `used_percentage` and `resets_at` in Unix seconds, beside a gateway's `spend_limit` and every other field, none of which QuotaDial keeps (D-145) |
| `status-line-api-key.synthetic.json` | The same, with no `rate_limits` | A sign-in with an API key, which the docs say gets none, and a session before its first answer: nothing to keep |
| `status-line-one-window.synthetic.json` | The same, with only `rate_limits.seven_day` | A window Claude Code has dropped once its reset passed, as the docs say it does |

## Codex

| File | Source | Covers |
| --- | --- | --- |
| `wham-usage.json` | Live `GET /backend-api/wham/usage` for a Pro account, 2026-10-23, with the email, user id and account id replaced | Today's shape: the weekly window as `primary_window` and no five-hour window, `credits`, `model_usage`, `spend_control`, the reset credits, null `additional_rate_limits` and `code_review_rate_limit` |
| `wham-usage.synthetic.json` | CodexBar's field names | A Plus plan's session and weekly windows, credits turned off |
| `rollout.jsonl` | A live Codex CLI session log, 2026-10-23, reduced to its event shapes with ids, paths and text replaced | `session_meta` naming the creating account and user, `token_count` events with rate limits (one for the `premium` limit), other events after the last one |

## Fake

The example build's accounts (D-117): made up for pictures of the app, never a capture.

| File | Source | Covers |
| --- | --- | --- |
| `fake/example.json` | Hand-written, the numbers of the site's pictures; built into `quotadial-provider-fake`, and the form of the file `QUOTADIAL_FAKE_ACCOUNTS` names | One account like a Claude Max 5x plan (Weekly, Session: the two windows Claude Code's status line hands over, D-145) and one like a ChatGPT Pro plan (Weekly, Session), each window with its share used when the examples start, its reset and how fast it fills |

## Status pages

| File | Source | Covers |
| --- | --- | --- |
| `status/claude-status.json` | Live `GET https://status.claude.com/api/v2/status.json`, 2026-10-23, as sent (a public page) | All systems operational |
| `status/openai-status.json` | Live `GET https://status.openai.com/api/v2/status.json`, 2026-10-23, as sent (a public page) | A minor incident: "Partial System Degradation" |

## Redacting a capture

1. Save the live response under `fixtures/<provider>/raw/`. That folder is gitignored. Use a tool
   that never prints the token, and keep response headers there too: they carry organization and
   workspace IDs, and ChatGPT's set cookies. Leave at least 2 minutes between requests to ChatGPT,
   as QuotaDial does. Claude's numbers are never asked for: its fixtures are made up from Claude
   Code's docs (D-145).
2. Copy it up one level. Replace every token, cookie, API key, email address, account ID and
   organization ID with an obviously fake value of the same type, such as `user@example.com` or
   `00000000-0000-0000-0000-000000000000`. Keep field names, nesting and numbers, because those are
   what the tests check.
3. Before committing, search the redacted file for anything that still looks secret:
   `grep -n -i -E "eyJ|sk-|bearer|refresh|@" fixtures/<provider>/*.json`
