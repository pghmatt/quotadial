# Brand assets delivered with the foundation

codus-brand-furniture's manifest for quotadial.com. Every lane uses these and nothing else for
QuotaDial's identity; a missing or wrong asset goes back to the foundation's owner (the Brain).

| Asset | Path | Evidence |
| --- | --- | --- |
| Favicon | `app/favicon.ico` | 16, 32 and 48 px entries, each an RGBA PNG (colour type 6); SHA-256 starts `2f5f55e06ba2c075` |
| App icon | `app/icon.png` | 512x512 RGBA |
| Apple touch icon | `app/apple-icon.png` | 180x180, the tile to the edges |
| Share card | `app/opengraph-image.png` | 1200x630; alt in `app/opengraph-image.alt.txt` and `SHARE_ALT` |
| Header identity | `components/Header.tsx` | The mark (`Dial`, 30 px) and "QuotaDial" in Sofia Sans Condensed 750, linked home, on every page |
| Footer identity | `components/Footer.tsx` | The same lockup on the deep, the MIT line, the example-data line |
| Mark sheet | `docs/brand/mark-sheet.png` | The mark at 16, 32, 64 and 256 px on both grounds |
| Metadata | `lib/metadata.ts` | Canonical, Open Graph and Twitter large-card tags with the share card's size and alt, from one helper that alone adds the brand to titles |

**Scaffold favicon:** none. The site had no web app before the foundation and the Next.js app was
written by hand, so no template icon existed to replace (recorded in `docs/seo/site.config.json`).

**The mark at small sizes (reviewed by the Brain, 2026-10-25, from the mark sheet):** at 16 px the
blue arc and the white needle still read as a dial on the light ground; on the dark ground the tile
is close to the background, and the arc and needle carry it. It is the app's own icon, so the site
keeps it as it is rather than redrawing it for the web.

Render them again with `node scripts/brand.mjs` after the icon or the card changes.
