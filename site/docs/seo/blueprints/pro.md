# Blueprint: /pro (utility)

Job: show what QuotaDial Pro adds and what stays free, then hand a buyer to the checkout. No price
on the page (D-098): Lemon Squeezy's checkout shows it. Source: `README.md` (Free and Pro),
`crates/core/src/tier.rs` (`ProFeature::what`), `docs/LICENSING.md`.

## First phone viewport (390x844)

1. H1: "QuotaDial Pro"
2. Answer: Pro adds every account on the machine, a year of history, warnings before a window runs
   out, webhooks and Slack, the week's cost at API prices, and headroom for agents.
3. Primary action: "Buy QuotaDial Pro" (`data-primary-action`, `checkout_click`), to the Lemon
   Squeezy checkout link in `lib/data/site.ts`. Until Matt adds that link (#51), the button is
   replaced by one sentence saying buying opens soon, and the primary action is "Download
   QuotaDial Free".
4. Proof: the seven features, each in the words the app itself uses.

## Sections

| H2 | Answer |
|---|---|
| What Pro adds | The seven Pro features (`ProFeature::what`), each with what it changes for the reader |
| What stays free | QuotaDial Free: one account per provider, every window with pace and reset, 30 days of history, alerts for a share used, a reset and a sign-in, status pages, read-only status for agents |
| How buying works | Lemon Squeezy is the merchant of record and shows the price and tax; the receipt has a button, "Get your QuotaDial key", that opens /key |
| Adding your key | Settings' Licence field or `quotadial licence add <key>`; checked on the machine, no call home; a Free copy says where each Pro feature would be |

Links: /download, /support. Schema: none (no price can be stated).
Title: "What Pro adds: every account and a year of history | QuotaDial" · Meta: "QuotaDial Pro adds every
account on the machine, a year of history, early warnings, webhooks and Slack, and the week's cost.
QuotaDial Free stays free." (151)
