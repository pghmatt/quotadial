# Fixtures

Provider responses that drive the provider crates' tests.

| Path | Committed | Contents |
| --- | --- | --- |
| `fixtures/<provider>/*.json` | Yes | Redacted or synthetic responses, safe to publish |
| `fixtures/<provider>/raw/` | Never (gitignored) | Unredacted captures from a live account |

Synthetic fixtures are hand-written in the real response shape with obviously fake values. They are
named `*.synthetic.json` and are replaced by redacted captures once those exist.

## Redacting a capture

1. Save the live response under `fixtures/<provider>/raw/`. That folder is gitignored.
2. Copy it up one level. Replace every token, cookie, API key, email address, account ID and
   organization ID with an obviously fake value of the same type, such as `user@example.com` or
   `00000000-0000-0000-0000-000000000000`. Keep field names, nesting and numbers, because those are
   what the tests check.
3. Before committing, search the redacted file for anything that still looks secret:
   `grep -n -i -E "eyJ|sk-|bearer|refresh|@" fixtures/<provider>/*.json`
