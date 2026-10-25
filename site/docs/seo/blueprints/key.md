# Blueprint: /key (utility, not indexed)

Job: turn the key in a Lemon Squeezy receipt link (`/key#licence=<key>`) into the QuotaDial key,
show it with where to paste it, and explain every failure in one sentence. The page posts the key to
`/api/key` (D-102); the fragment never reaches a server log. `robots: noindex`, not in the sitemap.

## States (each designed and captured)

| State | What shows | Primary action (`data-primary-action`) |
|---|---|---|
| No fragment | How to get here: the receipt's "Get your QuotaDial key" button; where else to find the receipt | "Write to support" |
| Checking | The key from the receipt, being checked, without a spinner-only screen | none (disabled) |
| Success | The QuotaDial key in full, a copy button, and the two ways to add it (Settings' Licence field, `quotadial licence add`) | "Copy the key" (`key_copied`; `key_shown` when it appears) |
| Error | The function's own sentence for the failure (a key that is not one, a refund, another product, Lemon Squeezy unreachable), and what to do next | "Try again" or "Write to support", as the error says |

H1: "Your QuotaDial key" in every state. Links: /download, /support.
Title: "Your QuotaDial Pro key | QuotaDial" · Meta: "Collect the QuotaDial Pro key from your Lemon
Squeezy receipt link, then paste it into QuotaDial's Settings or run quotadial licence add on your
machine." (153)
