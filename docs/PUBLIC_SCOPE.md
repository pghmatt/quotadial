# What goes in the public repo

The GitHub repo `pghmatt/quotadial` is public. The PRD in `docs/PRD.md` contains competitor research and pricing that stay private.

## Rule

`docs/PRD.md` is gitignored. The public repo carries `docs/PRD-public.md`, generated from the PRD with these sections removed:

- "Competitive teardown" (whole section, including the complaints and gaps lists)
- Any line containing a dollar amount or the words "Pro price", "lifetime", "per month"
- The "Open questions" checklist items about price and open-sourcing

Everything else is public: Summary, Provider data sources, Product requirements (minus the pricing lines), Architecture, Leveraging CodexBar, Handling every complaint and gap, Build plan, Risks, Sources.

## How to regenerate

A small script at `scripts/public-prd.sh` strips the sections above and writes `docs/PRD-public.md`. Run it after any PRD edit and commit the output. CI fails if `docs/PRD.md` is ever tracked.

## Why

The research took a day; competitors should not get it for free. The requirements and architecture are fine to publish because the product's edge is execution, not the idea.
