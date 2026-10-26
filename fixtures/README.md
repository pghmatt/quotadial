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

## Copilot

Made up in the shape of GitHub's `GET https://api.github.com/copilot_internal/user`, with the field
names GitHub's own Copilot clients read, as CodexBar ports them (`CopilotUsageModels.swift` at
4654968e9), and every value invented (D-127). No real answer has been captured yet: #51's live check
saves one, redacted, under these, and says which of these shapes it confirms.

| File | Covers |
| --- | --- |
| `user-pro-credits.synthetic.json` | A plan billed in AI credits (`token_based_billing`): the month's credits in `premium_interactions`, 375 of 1,500 used, with `quota_reset_at`; chat and completions unlimited |
| `user-annual-premium.synthetic.json` | An annual plan still billed by request: the month's premium requests, 40% left; chat and completions unlimited |
| `user-free.synthetic.json` | Copilot Free: `monthly_quotas` and `limited_user_quotas` for chat and completions, and `limited_user_reset_date`, with no `quota_snapshots` |
| `user-business.synthetic.json` | A seat billed elsewhere: `premium_interactions` with no allowance and nothing left beside `percent_remaining: 100`, which is no meter, not 0% |
| `user-unlimited.synthetic.json` | Every allowance unlimited |
| `unauthorized.synthetic.json` | GitHub's documented error body for a rejected token (`Bad credentials`) |

GitHub CLI's `hosts.yml` is made up in the provider's own tests (`crates/providers/copilot/src/
hosts.rs`) rather than kept here, since the real file holds GitHub CLI's token.

## Fake

The example build's accounts (D-117): made up for pictures of the app, never a capture.

| File | Source | Covers |
| --- | --- | --- |
| `fake/example.json` | Hand-written, the numbers of the site's pictures; built into `quotadial-provider-fake`, and the form of the file `QUOTADIAL_FAKE_ACCOUNTS` names | One account like a Claude Max 5x plan (Weekly, Weekly (Opus), Session) and one like a ChatGPT Pro plan (Weekly, Session), each window with its share used when the examples start, its reset and how fast it fills |

## Status pages

| File | Source | Covers |
| --- | --- | --- |
| `status/claude-status.json` | Live `GET https://status.claude.com/api/v2/status.json`, 2026-10-23, as sent (a public page) | All systems operational |
| `status/openai-status.json` | Live `GET https://status.openai.com/api/v2/status.json`, 2026-10-23, as sent (a public page) | A minor incident: "Partial System Degradation" |

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
