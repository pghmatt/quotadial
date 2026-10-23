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

## Redacting a capture

1. Save the live response under `fixtures/<provider>/raw/`. That folder is gitignored. Use a tool
   that never prints the token, and keep response headers there too: they carry organization and
   workspace IDs. Leave at least 5 minutes between requests to Anthropic, as QuotaDial does.
2. Copy it up one level. Replace every token, cookie, API key, email address, account ID and
   organization ID with an obviously fake value of the same type, such as `user@example.com` or
   `00000000-0000-0000-0000-000000000000`. Keep field names, nesting and numbers, because those are
   what the tests check.
3. Before committing, search the redacted file for anything that still looks secret:
   `grep -n -i -E "eyJ|sk-|bearer|refresh|@" fixtures/<provider>/*.json`
