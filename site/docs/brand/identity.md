# QuotaDial's identity on the web

Written 2026-10-25 for the website (M5 item 5), after PRODUCT.md, the search study and the roll
(`docs/design/candidates.md`: the tide table and the almanac).

## The five answers

1. **What it does, for whom.** Shows developers who pay for Claude or ChatGPT and work in Claude
   Code or the Codex CLI how much of each plan limit they have used and when it resets, from the
   sign-in they already have, in the menu bar or tray and for their agents.
2. **The single idea.** You can see a limit coming: when it resets, and whether you will reach it
   first at the pace you are going.
3. **The category's conventions, and the chosen world.** Usage trackers ship a centred Mac-app hero
   with a floating popover and a price button, or a dark docs template with icon cards. The chosen
   world is the tide almanac: predicted times in tables, curves against a high-water line, chart
   colours.
4. **Where it must work.** The 16 px favicon and the macOS menu bar (the app's own icon), a 390 px
   phone header, the 1200x630 share card, light and dark themes.
5. **What would misrepresent it.** Nautical kitsch (anchors, rope, ship's wheels, waves as
   decoration); a speedometer or fuel-gauge reading of "Dial" that suggests speed; anything that
   implies QuotaDial raises a limit; a price or a user count.

## Fixed assets, and what may change

- **Fixed:** the name QuotaDial; the app icon, `apps/tray/app-icon.svg`: a 240-degree dial on a dark
  rounded tile, the used part in blue `#0a84ff`, the rest grey, a white needle and hub. The site
  uses it unchanged as the mark and the favicon, so the site and the menu bar show the same thing.
- **Free to choose here:** the wordmark's setting, the site's type and colours.

## The mark and the name

- **Mark:** the app icon. Tested at 16, 32 and 48 px for the favicon and 180 and 512 px for the
  touch and app icons when the foundation renders them; the tile keeps the dial readable on light
  and dark grounds alike (16:1 against the light ground).
- **Wordmark:** "QuotaDial" set in Sofia Sans Condensed, weight 700, in the ink colour, beside the
  mark; never generated lettering.

## Type as a system

| Role | Face | Licence and loading |
| --- | --- | --- |
| Display and headings | Sofia Sans Condensed (variable), 650 to 800; its italic for labels on charts, as charts set water names in sloping letters | SIL Open Font Licence 1.1, `@fontsource-variable/sofia-sans-condensed` 5.3.0, self-hosted |
| Reading text | Sofia Sans (variable), 400 and 600 | OFL 1.1, `@fontsource-variable/sofia-sans` 5.3.0 |
| Tables and figures | Sofia Sans Semi Condensed with tabular figures | OFL 1.1, `@fontsource-variable/sofia-sans-semi-condensed` 5.3.0 |
| Code only | JetBrains Mono (variable) | OFL 1.1, `@fontsource-variable/jetbrains-mono` 5.3.0 |

One superfamily for every word a visitor reads, in three widths, and a monospace only where the
text is code. Scale, weight and width carry the hierarchy before any other device.

## Colour as a system

Strategy: **full palette with chart roles**. Chart blues own whole regions (the page descends from
the white of deep water to the navy of the deep as it scrolls); one reserved accent, chart magenta,
which on a nautical chart marks lights and cautions.

| Role | Light | Dark (the night chart) |
| --- | --- | --- |
| Ground | `#f6f9fb` chart white | `#08131d` |
| Ink (text) | `#0c1d2b` | `#dce9f2` |
| Second ink | `#3b5466` | `#9fb6c7` |
| Shallow field | `#dcedf6` | `#0e2233` |
| Mid field | `#b3d7ec` | `#143149` |
| Line blue (curves, rules; never text) | `#2f74a6` | `#5b9fd0` |
| The deep (footer) | `#0b2236`, text `#e6f1f8` and `#a9c3d6` | `#040d15` |
| Magenta, reserved | `#a8005f` | `#ff6db5` as text, `#c2186f` as a fill |

**Where magenta may appear, and nowhere else:** the primary action's fill; a prediction that a
window runs out before its reset; the focus ring; the one caution on the not-signed-yet steps. Never
on headings, names, links in running text, or decoration.

### Measured contrast (WCAG 2.2 AA)

| Pair | Light | Dark | Needs |
| --- | --- | --- | --- |
| Ink on ground | 16.20 | 15.15 | 4.5 |
| Second ink on ground | 7.51 | 8.91 | 4.5 |
| Ink on shallow field | 14.26 | 13.11 | 4.5 |
| Second ink on shallow field | 6.61 | 7.71 | 4.5 |
| Ink on mid field | 11.29 | 10.85 | 4.5 |
| Second ink on mid field | 5.23 | 6.38 | 4.5 |
| Magenta text on ground | 7.02 | 7.23 | 4.5 |
| Magenta text on shallow field | 6.18 | 6.26 | 4.5 |
| White on the magenta fill | 7.42 | 5.75 | 4.5 |
| Line blue on ground, shallow, mid | 4.76, 4.19, 3.32 | 6.52, 5.65, - | 3 (graphics) |
| Text on the deep | 14.12 and 8.84 | - | 4.5 |

The foundation repeats these from the built tokens and the lanes measure them on the rendered
pages.

## Rejected directions

- The timing tower, the departure board and the EV planner (candidates 1 to 3): strong, and not
  what the roll assigned; nothing factual stopped the tide table.
- The oscilloscope bench (the strongest challenger): its phosphor on black is the category's own
  costume.
- A cream almanac with a serif display and hairlines: the generic rendering of "almanac", refused
  before it started; the chart's own colours are blue, white and magenta.
