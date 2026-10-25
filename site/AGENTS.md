# The QuotaDial website

The site in this folder is quotadial.com: a static Next.js export served by Cloudflare Pages, with
the key page's Pages Function in `functions/api/key.ts`. Its product truth is `PRODUCT.md`; its
design is `docs/DIRECTION.md` and `docs/design-contract.json`; its routes, titles and required
headings are `docs/seo/site.config.json` and `docs/seo/blueprints/`; its facts are
`docs/CONTENT-BIBLE.md` and `lib/data/`. Who builds which page is `docs/LANES.md`.

## Commands

```sh
npm ci
npm run check      # types (TypeScript 7), everything
npm test           # the key function, the Markdown pages, the examples
npm run build      # the static site in out/
PORT=3082 npm run serve   # out/ as Pages serves it, with a stand-in for /api/key (scripts/serve.mjs)
node scripts/brand.mjs        # favicon, icons, share card, mark sheet
node scripts/app-pictures.mjs # the app's pictures (build ../apps/tray first)
```

## Rules for every page

- Read your lane brief and the skills it names in your own context before you write anything.
- Use the foundation: `Section`, `Plate`, `TideTable`, `TideCurve`, `AppShot`, `Dial`, `ButtonLink`,
  `DownloadButton`, `CodeBlock`, `CopyButton`, `Notice`, `Question`, `Prose`, `pageMetadata`. Never
  restyle or fork them; ask the Brain for a change.
- Colours and fonts only from the tokens (`bg-chart`, `text-ink`, `font-display` and the rest);
  Tailwind's own are cleared. Magenta only where `docs/brand/identity.md` allows it.
- Each page's H1, title and required section headings are exactly as `docs/seo/site.config.json`
  says. One H1. The primary action carries `data-primary-action` and is visible at 390x844.
- Events through `data-event` attributes (or `track()` in a client component), named as the route
  map names them.
- No kicker or eyebrow above a heading, no cards in cards, no gradient text, no em dashes, no
  "Get started" or "Learn more", no new dependencies, no price, no invented proof. Pictures of the
  app only through `AppShot`; every table of example numbers says "Example accounts".
- Copy: `codus-copy`, built from `docs/copy/source-library.md`; the product's words (#52).
- The contract, the seed and review text stay in `docs/`; never in a page, a comment or `public/`.

<!-- BEGIN:nextjs-agent-rules -->

## This is NOT the Next.js you know

This version has breaking changes — APIs, conventions, and file structure may all differ from your training data. Read the relevant guide in `node_modules/next/dist/docs/` (resolved from this file's directory; in monorepos the `next` package may not be visible from the repo root) before writing any code. Heed deprecation notices.

This block is written and re-added by `next dev` — verify at `node_modules/next/dist/server/lib/generate-agent-files.js`. Removing it from a diff only re-creates the uncommitted change; committing it with your work keeps the tree clean.

<!-- END:nextjs-agent-rules -->
