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
| `oauth-usage.json` | Live `GET /api/oauth/usage`, 2026-10-23, committed as sent (it holds no identifiers) | Today's shape: `five_hour`, `seven_day`, `limits[]` with a scoped weekly limit, `extra_usage`, `spend`, `seven_day_breakdown`, null per-model buckets |
| `oauth-usage.synthetic.json` | CodexBar's field names | Flat per-model buckets (`seven_day_opus`, `seven_day_sonnet`), a bucket with no reset time |
| `oauth-usage-scoped.synthetic.json` | Hand-written | Enabled extra usage, the Cowork bucket, an unknown `seven_day_*` bucket, `limits[]` |
| `oauth-profile.json` | Live `GET /api/oauth/profile`, 2026-10-23, redacted | The profile response QuotaDial reads identity from |
| `oauth-profile.synthetic.json` | Claude Code's validation schema | The fields QuotaDial reads |

## Codex

| File | Source | Covers |
| --- | --- | --- |
| `wham-usage.json` | Live `GET /backend-api/wham/usage` for a Pro account, 2026-10-23, with the email, user id and account id replaced | Today's shape: the weekly window as `primary_window` and no five-hour window, `credits`, `model_usage`, `spend_control`, the reset credits, null `additional_rate_limits` and `code_review_rate_limit` |
| `wham-usage.synthetic.json` | CodexBar's field names | A Plus plan's session and weekly windows, credits turned off |
| `rollout.jsonl` | A live Codex CLI session log, 2026-10-23, reduced to its event shapes with ids, paths and text replaced | `session_meta` naming the creating account and user, `token_count` events with rate limits (one for the `premium` limit), other events after the last one |

## Redacting a capture

1. Save the live response under `fixtures/<provider>/raw/`. That folder is gitignored. Use a tool
   that never prints the token, and keep response headers there too: they carry organization and
   workspace IDs, and ChatGPT's set cookies. Leave at least 5 minutes between requests to Anthropic
   and 2 minutes between requests to ChatGPT, as QuotaDial does.
2. Copy it up one level. Replace every token, cookie, API key, email address, account ID and
   organization ID with an obviously fake value of the same type, such as `user@example.com` or
   `00000000-0000-0000-0000-000000000000`. Keep field names, nesting and numbers, because those are
   what the tests check.
3. Before committing, search the redacted file for anything that still looks secret:
   `grep -n -i -E "eyJ|sk-|bearer|refresh|@" fixtures/<provider>/*.json`
