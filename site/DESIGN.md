---
name: "quotadial.com"
description: "Plan limits read like a tide almanac: predicted times in tables and on curves against a high-water line, in chart colours."
colors:
  magenta: "#a8005f"
  magenta-fill: "#a8005f"
  magenta-fill-hover: "color-mix(in srgb, #a8005f 86%, black)"
  on-magenta: "#ffffff"
  line: "#2f74a6"
  chart: "#f6f9fb"
  ink: "#0c1d2b"
  ink-2: "#3b5466"
  shallow: "#dcedf6"
  mid: "#b3d7ec"
  deep: "#0b2236"
  on-deep: "#e6f1f8"
  on-deep-2: "#a9c3d6"
  rule: "color-mix(in srgb, #0c1d2b 18%, transparent)"
  row-band: "color-mix(in srgb, #0c1d2b 5%, transparent)"
  scroll-shade: "color-mix(in srgb, #0c1d2b 24%, transparent)"
  magenta-night: "#ff6db5"
  magenta-fill-night: "#c2186f"
  magenta-fill-hover-night: "color-mix(in srgb, #c2186f 86%, black)"
  line-night: "#5b9fd0"
  chart-night: "#08131d"
  ink-night: "#dce9f2"
  ink-2-night: "#9fb6c7"
  shallow-night: "#0e2233"
  mid-night: "#143149"
  deep-night: "#040d15"
  rule-night: "color-mix(in srgb, #dce9f2 18%, transparent)"
  row-band-night: "color-mix(in srgb, #dce9f2 5%, transparent)"
  scroll-shade-night: "color-mix(in srgb, #dce9f2 24%, transparent)"
typography:
  scale:
    home-display-wide: "4.25rem"
    display: "4rem"
    home-display: "3.75rem"
    close: "3rem"
    headline: "2.75rem"
    close-phone: "2.25rem"
    headline-phone: "2rem"
    ledger-heading: "1.875rem"
    subhead: "1.75rem"
    read-aloud: "1.5rem"
    title: "1.375rem"
    answer: "1.25rem"
    lead: "1.1875rem"
    label: "1.125rem"
    body: "1.0625rem"
    table: "1rem"
    caption: "0.9375rem"
    table-head: "0.875rem"
  display:
    fontFamily: "'Sofia Sans Condensed Variable', 'Arial Narrow', sans-serif"
    fontSize: "4rem"
    fontWeight: 800
    lineHeight: 1.02
    letterSpacing: "-0.01em"
  headline:
    fontFamily: "'Sofia Sans Condensed Variable', 'Arial Narrow', sans-serif"
    fontSize: "2.75rem"
    fontWeight: 750
    lineHeight: 1.05
    letterSpacing: "-0.01em"
  side-head:
    fontFamily: "'Sofia Sans Condensed Variable', 'Arial Narrow', sans-serif"
    fontSize: "2rem"
    fontWeight: 750
    lineHeight: 1.05
  subhead:
    fontFamily: "'Sofia Sans Condensed Variable', 'Arial Narrow', sans-serif"
    fontSize: "1.75rem"
    fontWeight: 750
    lineHeight: 1.1
  title:
    fontFamily: "'Sofia Sans Condensed Variable', 'Arial Narrow', sans-serif"
    fontSize: "1.375rem"
    fontWeight: 700
    lineHeight: 1.15
  answer:
    fontFamily: "'Sofia Sans Variable', 'Helvetica Neue', Arial, sans-serif"
    fontSize: "1.25rem"
    fontWeight: 400
    lineHeight: 1.45
  lead:
    fontFamily: "'Sofia Sans Variable', 'Helvetica Neue', Arial, sans-serif"
    fontSize: "1.1875rem"
    fontWeight: 400
    lineHeight: 1.5
  body:
    fontFamily: "'Sofia Sans Variable', 'Helvetica Neue', Arial, sans-serif"
    fontSize: "1.0625rem"
    fontWeight: 400
    lineHeight: 1.55
  table:
    fontFamily: "'Sofia Sans Semi Condensed Variable', 'Arial Narrow', sans-serif"
    fontSize: "1rem"
    fontWeight: 400
    lineHeight: 1.35
    fontFeature: "'tnum' 1"
  table-head:
    fontFamily: "'Sofia Sans Semi Condensed Variable', 'Arial Narrow', sans-serif"
    fontSize: "0.875rem"
    fontWeight: 600
    lineHeight: 1.35
    fontFeature: "'tnum' 1"
  caption:
    fontFamily: "'Sofia Sans Semi Condensed Variable', 'Arial Narrow', sans-serif"
    fontSize: "0.9375rem"
    fontWeight: 400
    lineHeight: 1.55
  plate-title:
    fontFamily: "'Sofia Sans Condensed Variable', 'Arial Narrow', sans-serif"
    fontSize: "1rem"
    fontWeight: 650
    lineHeight: 1.3
  label:
    fontFamily: "'Sofia Sans Condensed Variable', 'Arial Narrow', sans-serif"
    fontSize: "1.125rem"
    fontWeight: 650
    lineHeight: 1
  code:
    fontFamily: "'JetBrains Mono Variable', ui-monospace, Menlo, Consolas, monospace"
    fontSize: "0.9375rem"
    fontWeight: 400
    lineHeight: 1.6
    fontFeature: "'liga' 0, 'calt' 0"
rounded:
  chart: "2px"
spacing:
  gutter: "24px"
  gutter-phone: "16px"
  band-y: "96px"
  band-y-phone: "64px"
  part-gap: "128px"
  part-gap-phone: "96px"
  plate: "28px"
  plate-phone: "20px"
  neatline-inset: "8px"
  cell-x: "12px"
  cell-y: "8px"
  cell-y-words: "12px"
  side-head-gap: "40px"
  section-top: "56px"
  section-top-phone: "40px"
  block-gap: "20px"
components:
  button-primary:
    backgroundColor: "{colors.magenta-fill}"
    textColor: "{colors.on-magenta}"
    typography: "{typography.label}"
    rounded: "{rounded.chart}"
    padding: "12px 20px"
  button-primary-hover:
    backgroundColor: "{colors.magenta-fill-hover}"
    textColor: "{colors.on-magenta}"
  button-secondary:
    backgroundColor: "transparent"
    textColor: "{colors.ink}"
    typography: "{typography.label}"
    rounded: "{rounded.chart}"
    padding: "11px 20px"
  button-text:
    textColor: "{colors.ink}"
    typography: "{typography.label}"
  button-copy:
    textColor: "{colors.ink}"
    rounded: "{rounded.chart}"
    padding: "8px 16px"
  header:
    backgroundColor: "{colors.chart}"
    textColor: "{colors.ink}"
    height: "64px"
  footer:
    backgroundColor: "{colors.deep}"
    textColor: "{colors.on-deep}"
  section-chart:
    backgroundColor: "{colors.chart}"
    textColor: "{colors.ink}"
    padding: "96px 24px"
  section-shallow:
    backgroundColor: "{colors.shallow}"
    textColor: "{colors.ink}"
    padding: "96px 24px"
  section-mid:
    backgroundColor: "{colors.mid}"
    textColor: "{colors.ink}"
    padding: "96px 24px"
  section-deep:
    backgroundColor: "{colors.deep}"
    textColor: "{colors.on-deep}"
    padding: "96px 24px"
  plate:
    backgroundColor: "{colors.shallow}"
    textColor: "{colors.ink}"
    rounded: "{rounded.chart}"
    padding: "28px"
  plate-chart:
    backgroundColor: "{colors.chart}"
    textColor: "{colors.ink}"
    rounded: "{rounded.chart}"
    padding: "28px"
  plate-mid:
    backgroundColor: "{colors.mid}"
    textColor: "{colors.ink}"
    rounded: "{rounded.chart}"
    padding: "28px"
  tide-table:
    textColor: "{colors.ink}"
    typography: "{typography.table}"
    padding: "8px 12px"
  tide-table-band:
    backgroundColor: "{colors.row-band}"
  tide-table-caution:
    textColor: "{colors.magenta}"
  code-block:
    backgroundColor: "{colors.mid}"
    textColor: "{colors.ink}"
    typography: "{typography.code}"
    rounded: "{rounded.chart}"
    padding: "16px"
  code-block-output:
    backgroundColor: "{colors.mid}"
    textColor: "{colors.ink}"
    typography: "{typography.code}"
    rounded: "{rounded.chart}"
    padding: "16px"
  side-head:
    textColor: "{colors.ink}"
    typography: "{typography.side-head}"
    width: "224px"
  contents-label:
    textColor: "{colors.ink-2}"
    typography: "{typography.plate-title}"
  file-name:
    textColor: "{colors.ink}"
    typography: "{typography.table}"
  file-name-versioned:
    textColor: "{colors.ink-2}"
  inline-code:
    backgroundColor: "{colors.shallow}"
    textColor: "{colors.ink}"
    rounded: "{rounded.chart}"
    padding: "0 4px"
  notice:
    textColor: "{colors.ink}"
    rounded: "{rounded.chart}"
    padding: "24px"
---

# Design System: quotadial.com

## Overview

**Creative North Star: "The Tide Almanac"**

The site reads every plan limit the way an almanac reads the tide: as a predicted time. When a window resets, and when it would run out at the pace it is going, are set in tables of tabular figures and on curves that climb towards a dashed high-water line at 100%. The look is the nautical chart's own. Flat fields of chart blue, fine vector line work and navy ink carry everything, and one chart magenta is held back for the action and for cautions, much as a chart keeps magenta for its lights and warnings.

Each page descends as it scrolls, from chart white through the shallow and mid blue fields to the navy deep of the closing band and footer, one tone to a band. Sofia Sans carries every word in three widths: condensed for headings, actions and chart labels, regular for reading, semi-condensed with tabular figures for tables and captions. Density is moderate: tables carry real numbers, reading blocks stay short, and generous space separates the bands. The only pictures are captures of the real QuotaDial window with example accounts. The pages read for reference, the changelog and the install and command guides, keep one quiet family throughout: each section's name stands in a margin column beside its text, as a printed reference book sets its side heads. Light and dark follow the visitor's system as the day chart and the night chart, and every colour token has its night pair.

The system rejects the usage-tracker category's centred app hero with a floating popover, the dark documentation template with icon cards, the cream almanac with a serif display and hairlines, and nautical ornament of every kind.

**Key Characteristics:**
- Predicted times in tables and curves, read against a dashed high-water line at 100%.
- Depth bands that descend each page from chart white to the navy deep.
- One reserved accent, chart magenta, in four places and nowhere else.
- Sofia Sans in three widths for every word; JetBrains Mono only for code, commands, file names and addresses.
- Flat fields and hairline line work: no shadows, and a 2 px corner on every box.
- Plates framed with a neatline and a graduated scale, in place of cards.
- Reference pages that set every section as a side head on a 14rem column, one family from top to bottom.

## Colors

A full palette with chart roles: the blues own whole regions of the page, line blue draws, and a single reserved magenta marks what to do and what to watch. Every colour has a night value (the `-night` tokens); the custom properties in `app/globals.css` switch with `prefers-color-scheme`, and there is no theme toggle.

### Primary
- **Chart Magenta** (#a8005f; at night #ff6db5 as text and #c2186f as a fill): the primary action's fill, a prediction that a window runs out before it resets, the focus ring, and the one caution on the steps for opening a build that is not signed. On hover the fill darkens to Magenta Fill Hover, the fill mixed 86% with black. White on the fill measures 7.42:1 by day and 5.75:1 at night; magenta text measures 7.02:1 on chart white and stays at 4.89:1 or more on every field it is used on.

### Secondary
- **Line Blue** (#2f74a6; at night #5b9fd0): graphics only. The tide curve, the plate's neatline and its graduated scale. It measures 3.32:1 to 4.76:1 on the light fields, enough for line work and never enough for words.

### Neutral
- **Chart White** (#f6f9fb; night #08131d): the ground, the header, the first band of every page, and the field of a plate set on a blue band.
- **Navy Ink** (#0c1d2b; night #dce9f2): headings, reading text, row heads and the high-water line, at 16.2:1 on chart white.
- **Second Ink** (#3b5466; night #9fb6c7): the answer under an H1, paragraphs after a lead, table heads, captions, plate titles, axis labels and the reset mark, at 5.23:1 or more on every field.
- **Shallow Field** (#dcedf6; night #0e2233): the second band, the default plate, and inline code set on chart white.
- **Mid Field** (#b3d7ec; night #143149): the third band, code blocks, the water under the curve at 70% opacity, and selected text.
- **The Deep** (#0b2236; night #040d15): the closing band and the footer, with text in On Deep (#e6f1f8) and On Deep Second (#a9c3d6) in both themes.
- **Rule** (Navy Ink at 18%): every hairline: the header's foot, the line between groups of content, the notice's border and the curve's grid.
- **Row Band** (Navy Ink at 5%): the tint laid over a group of table rows. Navy Ink keeps 9.5:1 or more on it, and Second Ink 4.7:1 or more.
- **Scroll Shade** (Navy Ink at 24%): the sign that output scrolls, laid over the last characters on the side with more text and fading to nothing over 2rem. At night it is the night ink at 24%, so it pales the edge instead of darkening it.

### Named Rules
**The Four Places Rule.** Magenta appears in exactly four places: the primary action's fill, a runs-out-first prediction, the focus ring and the unsigned-build caution. Never on a heading, a name, a link in running text or decoration, and never as text on the Row Band over Mid Field, where it falls to 4.47:1.

**The Words First Rule.** Magenta never carries a meaning alone. A runs-out-first cell says "Runs out Tue 18:40" in words, and the curve's magenta prediction ends in its label.

**The Line Blue Rule.** Line blue draws and never writes.

## Typography

**Display Font:** Sofia Sans Condensed (with Arial Narrow, sans-serif)
**Body Font:** Sofia Sans (with Helvetica Neue, Arial, sans-serif)
**Table Font:** Sofia Sans Semi Condensed (with Arial Narrow, sans-serif)
**Mono Font:** JetBrains Mono (with ui-monospace, Menlo, Consolas, monospace)

**Character:** One superfamily in three widths. The condensed headings are compact and evenly stroked, the regular width reads easily at length, and the semi-condensed width keeps columns of times aligned on tabular figures. Scale, weight and width carry the hierarchy before colour does. Every face is a self-hosted variable font loaded in `app/layout.tsx`; the fallbacks cover loading and nothing else.

### Hierarchy
- **Display** (800, 4rem from 768 px and 2.75rem below, line height 1.02, tracking -0.01em): the one H1 of every page. The home H1 steps on its own scale because it shares the first screen with the almanac plate: 2.75rem on a phone, 3.75rem from 640 px, 4.25rem from 1024 px and 3.75rem again from 1280 px, where it holds five columns, at line height 0.98 and tracking -0.015em.
- **Headline** (750, 2.75rem from 768 px and 2rem below, line height 1.05, tracking -0.01em): section headings, at most 24ch wide.
- **Close** (Condensed 750, 3rem from 768 px and 2.25rem below, line height 1.02): the closing band's one line on the home page, at most 22ch wide.
- **Side head** (Condensed 750, 2rem at every width, line height 1.05): the reference pages' section names, each version on /changelog and each section of /install and /cli, set at the headline's phone size. The three systems' names on the home page take the same size.
- **Ledger heading** (Condensed 750, 1.875rem, line height 1.1): the heading inside /pro's ledger plate.
- **Read aloud** (Semi Condensed 600, 1.5rem, and 1.75rem from 768 px, line height 1.25): the one row of a window the home page reads aloud in its plate ("Weekly 91%, resets in 1 d 15 h, ...").
- **Subhead** (750, 1.75rem, line height 1.1): a heading inside a plate or on the key page, and the second level of rendered Markdown.
- **Title** (700, 1.375rem, line height 1.15): the H3s: questions, the per-system checks, and the changelog's Added, Changed and Fixed.
- **Answer** (Sofia Sans 400, 1.25rem, line height 1.45, Second Ink): the sentence under an H1 that answers it, 34ch to 40ch wide. A guide's own opening paragraph takes the same size in Navy Ink at line height 1.5, at the reading measure.
- **Lead** (Sofia Sans 400, 1.1875rem, line height 1.5, Navy Ink): the paragraph that answers a section heading.
- **Body** (Sofia Sans 400, 1.0625rem, line height 1.55, and 1.6 in rendered Markdown): reading text at the reading measure.
- **Table** (Semi Condensed 400, 1rem, line height 1.35, tabular figures): every cell of a table of figures and every row head; row heads at 600, a caution at 650.
- **Table head** (Semi Condensed 600, 0.875rem, Second Ink, normal case): column heads.
- **Caption** (Semi Condensed italic 400, 0.9375rem, Second Ink): what a figure shows; beside any picture or sample of the app it says the accounts are examples.
- **Plate title** (Condensed italic 650, 1rem, line height 1.3, Second Ink): a plate's name at its head, sloped the way a chart letters the names of water. The curve's notes ("High water: 100%", the reset, the prediction) share this condensed italic, and so does the one label set without a plate: "In this guide", over a guide's contents.
- **Label** (Condensed 650, 1.125rem, line height 1): button labels; navigation links take the same size at 600.
- **Code** (JetBrains Mono 400, 0.9375rem in blocks and 0.875em inline, line height 1.6): code, commands, file names and network addresses, with ligatures off so a double hyphen never joins into one long dash. Code is always upright, even inside an italic caption, and breaks only at its marks.

### Named Rules
**The Three Widths Rule.** Condensed speaks, regular reads, semi-condensed counts. Headings, actions, labels and chart notes are condensed; paragraphs and the sentences in a table of words are regular; figures, times, table heads, row heads and captions are semi-condensed. A width never takes another width's job.

**The True Italic Rule.** A width that slopes loads its own italic file beside its upright, from the same package. The browser never invents a slope, and code, which has no italic, never slopes.

**The Unbroken Time Rule.** A time or a duration never breaks across two lines ("Tue 18:40", "1 d 15 h"), and a file name in a table stays whole wherever its column has the room: every name on /download is whole at 1440 and 1920 px. When a column cannot hold them, the layout changes first: on a phone the home table sets each row as two lines, and the files table stacks each file under its system. A name still wider than a phone's column breaks only at a mark.

**The Break Marks Rule.** Code, file names and addresses break only at their marks: after a slash (never between the two of `https://`), after an underscore, and after a hyphen inside a word, so a release address breaks after `releases/`, `latest/` or `download/` and never inside "latest". A flag (`--ignore-missing`) and a version (`0.2.0`) never break. `lib/breaks.ts` finds the marks and `components/Breakable.tsx` sets a `<wbr>` at each, with `overflow-wrap: break-word`, never `anywhere`, so a name stays whole wherever it fits and breaks inside a piece only when that piece is wider than the whole line.

**The Reading Measure Rule.** Reading text holds the measure token, 52ch (`--container-measure`). A ch is the width of a zero, wider than Sofia Sans' average letter, so 52ch holds a full line of body text to 74 characters at most, measured on every built page at 1440 px.

## Layout

A 12-column grid inside a 1280 px container, with 24 px side gutters from 768 px and 16 px below. Breakpoints sit at 640, 768, 1024 and 1280 px. Each band pads 96 px above and below from 768 px, and 64 px on a phone. Within one band of the home page, sections are set apart by space alone, 128 px from 768 px and 96 px on a phone, because the tone does not change between them. Headings take more space above than below.

The first screen splits five columns and seven from 1280 px, never centred. The left holds the H1, the answer, the download action with a text link beside it, and one line on the licence and the systems, their middle level with the plate's middle; the right holds the almanac plate. Below 1280 px the plate follows the text, and on a phone the order is the H1, the answer, the action and then the real window, starting inside the first screen.

Down the home page each section changes layout family: text beside a plate, a full-width table, a plate beside text, text beside code, three systems in columns of unequal width (5:4:6), text beside the address table, one full-width plate, then questions in two columns. The reference pages share one opening: the H1, its answer and its actions in five to seven columns, with the page's first figure or list beside them; on the guides that list is the contents, in the other seven columns from 1024 px.

The long reference pages, /changelog and the guides /install and /cli, set every section the same way. A Rule runs across the content width, 56 px of space follows it (40 px on a phone), and from 1024 px the section's name stands as a side head in a 14rem (224 px) column, held 24 px from the top of the window while its section scrolls past, with the section's text 40 px to its right. In a guide that text column runs the rest of the container (968 px at 1440 px): reading text holds the measure inside it, and tables and code span it whole. /changelog's text column is one measure wide, so a plate around a version closes on what it holds. Sections stand 56 px to 80 px apart, and the blocks inside a guide's section 20 px apart. Under 1024 px the side head sits above its text.

Responsive behaviour changes form rather than shrinking it. Under 768 px a table of words becomes a stack of blocks with each cell named by its column; under 640 px a guide's contents fold from two columns to one; the curve has its own narrow drawing so its words never fall below about 12 px; the navigation folds behind a Menu button. Nothing scrolls sideways at 390 px except output whose columns must stay aligned, and that scrolls inside a labelled region the keyboard can reach, shading its last characters on the side with more text.

### Named Rules
**The Descent Rule.** Bands descend: chart white first, then shallow, then mid, then the deep for the close and the footer. One tone to a band, and no page climbs back towards white.

**The Unequal Columns Rule.** Things that come in threes sit in columns of unequal width (5:4:6 on the home page, 4:3:5 on the download page), never in three equal cards.

**The Changing Family Rule.** The home page, the one page that persuades at length, changes layout family from section to section: no two of its sections in a row share one. A page read for reference does the opposite, under the Side Head Rule.

**The Side Head Rule.** A long reference page keeps one family from top to bottom: every section is a side head on the 14rem column beside its text, after a Rule, so a reader finds each section's name in the same place on every screen. /changelog and the guides read this way, and a new reference page joins them rather than inventing a layout of its own.

## Elevation & Depth

Flat. Nothing casts a shadow: the theme clears Tailwind's shadow and drop-shadow scales, so none can be reached for. Depth is tonal and drawn, the way a printed chart shows it: the bands descend from chart white to the deep, a plate sits on a field that differs from its band where it can, and its neatline frames it either way. There is no blur and no glass. The one overlap is deliberate: in the first screen the real QuotaDial window lies over the plate's lower edge, its foot faded out. The one shade is not depth either: output that scrolls carries the Scroll Shade over its last characters on the side with more text, a sign that the line goes on, and a block that fits shows none.

### Named Rules
**The Flat Chart Rule.** Depth comes from field tone and line work, never from shadow. A surface that needs to stand forward takes a different field or a neatline.

## Shapes

One corner, 2 px (`--radius-chart`), on buttons, plates, code blocks, inline code, notices, the menu button and the skip link; Tailwind's own radii are cleared. Borders are hairlines: 1 px in Rule for separators (the Rule above each reference section among them) and the notice's border, and 1 px in the current text colour for a secondary button and the menu button. Nothing has a side rail.

The plate's neatline is a 1 px rectangle in Line Blue at 55%, inset 8 px from the plate's edge. A graduated scale hangs from it into that margin on all four sides: a 3 px tick every 12 px at 45% and a 6 px tick every 60 px at 60%, drawn as SVG strokes with crisp edges.

The tide curve keeps fixed line weights: grid lines in Rule at 1 px, one for each day and at 25%, 50% and 75%; the axis in Second Ink at 1 px; the high-water line in Navy Ink at 1.5 px, dashed 6 and 5; the reset mark in Second Ink at 1.5 px; the share used in Line Blue at 3 px with round joins, over Mid Field at 70%; and the prediction in magenta at 2.5 px, dashed 5 and 5, ending in a 5 px dot on the high-water line.

The app icon keeps its own large rounded tile. It belongs to the mark, not to the system.

### Named Rules
**The Two Pixel Rule.** Every box (a button, a plate, a code block, a notice) takes a 2 px corner. No pills and no soft rounded cards; the only round forms are the curve's prediction dot and the mark.

**The Neatline Rule.** Every plate is framed as a chart plate is: a hairline neatline 8 px in, with a graduated scale in the margin, drawn as line work and never as texture or a bevel.

## Components

### Buttons
A small, firm set: one magenta action per view and quiet outlines and links around it.
- **Shape:** 2 px corners, one line at every width, label in Label type.
- **Primary:** Chart Magenta fill with a white label, 12 px by 20 px, one per view. A download action leads with a 20 px download icon and names the visitor's system once the page knows it ("Download for macOS"). An action that moves down the page leads with a 20 px down arrow instead ("Choose your system").
- **Hover / Focus:** the fill darkens to Magenta Fill Hover over 150 ms; pressing nudges the button 1 px down; focus draws a 2 px magenta outline 2 px outside the edge, on every control.
- **Secondary:** transparent with a 1 px border in the current text colour, 11 px by 20 px, so it stands 42 px tall, as the primary does without an icon (its 20 px icon makes it 44 px); hover lays 8% of the text colour over it.
- **Text:** an underlined link (1 px, 0.2em below) in Label type; hover thickens the underline to 2 px.
- **Copy:** compact (8 px by 16 px, 1.0625rem) and outlined; magenta only where copying is the page's one action, as with the key. It swaps to a check and "Copied" for two seconds, and says in a sentence when the browser refuses.
- **Disabled:** 60% opacity, with the reason beside it ("Ready when the check is done").
- Every label names its action: "Download for macOS", "Copy the key", "Write to support".

### Plates
The site's container, in place of cards: a figure, what goes with it, and a caption.
- **Corner Style:** 2 px.
- **Background:** Shallow by default, Chart White or Mid as variants, chosen to differ from the band where it can.
- **Shadow Strategy:** none; the neatline frames it (see Shapes and Elevation & Depth).
- **Internal Padding:** 28 px, and 20 px on a phone; the neatline sits 8 px in, inside that padding.
- **Width:** a plate closes on what it holds. A plate of reading text ends at the measure, so its frame never encloses empty space.
- **Title and caption:** a title at the head in Plate title type names the plate ("Files in each release", "Versions"); a caption under the figure in Caption type says what it shows.
- Plates are never nested, and a plate's title never sits above a heading.

### Tide Table
The signature table: predictions set the way a tide table sets its times.
- **Type:** Table type throughout, column heads in Table head type, and the first cell of each row a row head at 600.
- **Alignment:** words from the left, figures and times from the right.
- **Bands:** rows that belong to one thing (a provider, a system, an account) share one group, and alternate groups take the Row Band tint. A table whose rows name no group carries no tint at all. No rule runs under any row.
- **Cells:** 12 px by 8 px; times never break, and a file name breaks only at its marks, and only where its column cannot hold it whole.
- **Caution:** a runs-out-first cell sets in magenta at 650 and says it in words.
- **Caption:** always present, visible or for screen readers.

### Banded Table
The Tide Table's banding for rows of sentences rather than figures.
- Every column sits from the left; sentences in Sofia Sans at 1rem and line height 1.45; row heads in Semi Condensed 650 at 1.0625rem; 12 px of padding above and below each cell from 768 px.
- A row that stands alone is a band of its own, so long rows of several lines still read apart.
- Under 768 px each row becomes a block, and each cell carries its column's name above it in Table head type.

### Tide Curve
The signature drawing: a window's share used over its period, time along and share up, in the line weights listed under Shapes.
- **Notes:** "High water: 100%" above the line at the left in Navy Ink; the prediction above it at the right in magenta at 650; the reset under the axis in Second Ink. All three are condensed italic; the axis's days and percentages are semi-condensed and upright.
- **Sizes:** drawn twice, a wide drawing (640 by 314) and a narrow one for phones (360 by 266), so its words keep their size.
- **Behaviour:** static, since every time is given, so the drawing is the same on every build. It is an image to assistive technology, with a label that says the whole prediction in one sentence.
- **Motion:** on the home page the curve draws itself once on load, 900 ms on `cubic-bezier(0.16, 1, 0.3, 1)`, from a visible state. It is the site's one authored motion, and reduced motion shows the finished drawing at once.

### Code Block
Commands and output on the mid field.
- Mid Field, Navy Ink, Code type, 16 px padding and 2 px corners. A command carries a copy button under the code ("Copy the command").
- A long command wraps, so a phone shows it whole, and breaks only at its marks.
- **Output region:** example output whose columns must align never wraps and carries no copy button. It scrolls sideways inside a region the keyboard can reach, labelled for what it shows ("Settings, example output"). While more text lies to one side, the Scroll Shade covers the last 2rem on that side; covers in the block's own Mid Field travel with the text and hide each shade once the text reaches that end, so a region that fits shows nothing. It is drawn by CSS alone and adds nothing for a screen reader.

### Inline Code
- Code type at 0.875em, 4 px side padding and 2 px corners, always on a field that differs from the ground it sits on (Shallow on chart white, Mid on shallow), or on a 9% tint of Navy Ink. It breaks only at its marks.
- Network addresses take the code face too: in the address table without a field, and in running text like any inline code.

### File Names
One grammar for every file /download offers, in the files plate and in the command's table alike.
- The name the button saves comes first, as an underlined link in Table type and Navy Ink that fetches the file (`QuotaDial_universal.dmg`).
- Under it, in Second Ink at 0.9375rem, is the name a release holds it by (`QuotaDial_<version>_universal.dmg`), linked to the Releases page in the files plate.
- Both stay whole wherever their column has the room, and break only at their marks where it has not.

### Download Actions
How /download hands each visitor a file.
- **First screen:** once the page knows the system, the primary fetches that system's file itself ("Download for macOS"); Linux's two files stand as a primary and a secondary ("Download the .deb", "Download the .AppImage"). When the page cannot tell, on a phone or in an unknown browser, the primary is "Choose your system", led by the down arrow, and moves to the systems band. A text link to the install guide sits beside it.
- **Each system's part:** the three systems' parts of the shallow band (4:3:5) each end with their own secondary download, so a visitor who arrives at /download#macos finds the file in view. Under the buttons, a line in Second Ink at the reading measure names what each saves ("Saves as `QuotaDial_universal.dmg`.").

### Guide Page
A repository guide (/install, /cli) as a reference page, its words unchanged.
- **Opening:** the H1, the answer, the magenta download and a text link to the other guide in five columns. Beside them in seven, never on a card: the contents, under the label "In this guide" in Plate title type, with each section as an underlined link in Sofia Sans at 1.0625rem and line height 1.35, in two columns 40 px apart from 640 px. Under the list, a line in Caption type says the page is the repository's file, word for word.
- **The guide's own opening paragraph** follows in the column its sections' text uses, at the Answer size in Navy Ink.
- **Sections:** side heads, under the Side Head Rule. Inside a section, reading text holds the measure; tables of words are Banded Tables, each row its own band, with 8 px of side padding per cell from 768 px so the longest file names stay whole in a 1280 px window; commands are Code Blocks; example output sits in output regions. A block that belongs to a list item is indented 24 px from 768 px.
- **Close:** a shallow band with an H2 in Headline type, one line, the download again and the other guide's link, then the deep footer.

### Notice
A box for steps that carry a caution.
- A 1 px Rule border all round, 2 px corners, 24 px padding (16 px on a phone), its title in condensed 700 at 1.25rem.
- The one caution sentence may be magenta. Never a side rail. It also frames the key page's refusals.

### Question
- An H3 in Title type with its answer always open beneath it, in Second Ink at the reading measure. No accordion: every answer can be read and found. On the home page the questions run in two columns.

### Navigation
- **Header:** one line, 64 px tall, Chart White with a Rule hairline at its foot. The 30 px mark and the wordmark (condensed 750, 1.375rem, tracking -0.01em) link home.
- **Links:** Download, Pro, Privacy and Changelog in Label size at 600; hover and the current page underline at 2 px, 0.3em below. The GitHub logo, at 24 px, gains its word from 1024 px.
- **Mobile:** under 768 px a Menu button (1 px border, 2 px corners, a list icon that turns into a close icon) opens the links in a panel under the header, with a Rule at its foot.
- **Footer:** on the deep: the mark and wordmark, a line on the licence, and three columns of links in a 1.4:1:1:1 grid. Column titles are condensed 700 at 1.125rem in On Deep; links are On Deep Second, turning On Deep and underlined on hover; the closing note sits under an On Deep hairline at 18%.

### App Captures
- Real captures of the QuotaDial window, light or dark to match the visitor's theme, with alt text ending "example accounts". A tall capture is cut by a fade to transparent over its last 12% to 18%. In the first screen the window lies over the plate's lower edge.

### Mark and Icons
- **Mark:** the app's own icon, a 240-degree dial on a dark rounded tile, the used part in the app's blue, with a white needle and hub. It stands still at 30 px in the header and the footer; its colours belong to the icon, not to the site's palette.
- **Icons:** Phosphor, regular weight, 18 px to 30 px, for download, a down arrow on an action that moves down the page ("Choose your system"), copy and copied, menu and close, GitHub, and the three systems' logos. Nothing else is an icon.

## Do's and Don'ts

### Do:
- **Do** keep magenta to its four places: the primary fill, a runs-out-first prediction, the focus ring and the unsigned-build caution.
- **Do** band table rows by what they belong to (a provider, a system, an account), alternating the Row Band tint group by group.
- **Do** align words left and figures and times right in every table, on tabular figures.
- **Do** frame every plate with its neatline and graduated scale, and name a titled plate at its head in condensed italic.
- **Do** descend the bands down the page: chart white, shallow, mid, then the deep for the close and the footer.
- **Do** say every caution in words as well as in magenta.
- **Do** keep a time on one line, and change the layout when a column cannot hold it.
- **Do** break code, file names and addresses only at their marks (after a slash, an underscore or a word's hyphen), and keep every flag and version whole.
- **Do** name each file on /download as its button saves it, with the release's versioned name under it in Second Ink.
- **Do** set a long reference page's sections as side heads on the 14rem column, each after a Rule, one family from top to bottom.
- **Do** keep reading text at the reading measure, 52ch (a full line of 74 characters at most).
- **Do** show the app only as a real capture matched to the theme, captioned as example accounts.
- **Do** keep motion to one authored moment, the home curve drawing once in 900 ms, with reduced motion showing the end state.

### Don't:
- **Don't** stripe rows by position or rule a line under every row.
- **Don't** set a label, a kicker or a plate title above a heading; a heading carries itself.
- **Don't** use magenta on a heading, a name, a link in running text or decoration.
- **Don't** set Line Blue as text.
- **Don't** cast a shadow, blur a surface or use glass; the Scroll Shade on output is a sign of more text, not depth.
- **Don't** let code break inside a word, a flag or a version, and never set it with `overflow-wrap: anywhere`.
- **Don't** round a box past 2 px: no pill buttons and no soft cards.
- **Don't** use a cream, parchment or warm paper ground; the ground is a cool chart white.
- **Don't** draw nautical ornament (anchors, rope, ship's wheels, waves as decoration); a curve on this site always plots real numbers.
- **Don't** centre the first screen, or lay content out as three equal cards or icon-in-a-square grids.
- **Don't** use gradient text, neon on black, or monospace outside code, commands, file names and addresses.
- **Don't** draw an interface out of boxes to look like the app.
- **Don't** reveal sections on scroll or add a second moving moment.
- **Don't** fold answers into an accordion.
- **Don't** recolour or redraw the mark.
