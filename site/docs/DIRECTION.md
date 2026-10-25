# Direction: quotadial.com

The Design Contract in words. Private process text: it lives in this file and
`docs/design-contract.json`, never in served HTML, comments, scripts or public assets.

## The contract

**THESIS.** QuotaDial's site reads like a tide almanac for plan limits: each limit is a predicted
time (when it resets, and when it would run out at the pace it is going), set in tables and curves
against a high-water line. It refuses the category's centred Mac-app hero with a floating popover
and a price button, and the dark docs template with icon cards.

**OWN-WORLD.** Chart white descending through shallow and mid blue fields to a navy deep at the
footer; navy ink; one chart magenta for the primary action and for cautions only. Sofia Sans in
three widths: condensed for headings, regular for reading, semi-condensed with tabular figures for
every table. Line-blue curves on a fine grid with a dashed high-water line at 100% and italic chart
labels. Tables grouped in bands, never a rule under every row. Corners 2 px everywhere.

**STORY.** A developer who hit a limit mid-run sees, in the first screen, limits like theirs laid
out the way an almanac lays out tides: the share used, the reset, and "at this pace it runs out
Tue 18:40, before it resets". They learn it reads the sign-in they already have, and download it for
their system. Lower down: alerts, agents, every system, every address it connects to, Free and Pro,
questions.

**FIRST VIEWPORT.** From `docs/seo/blueprints/home.md`. Desktop 1440x900: the left five columns
hold the H1 "Claude and Codex usage tracker for your menu bar or tray" (Sofia Sans Condensed 700,
about 64 px, at most three lines), the answer sentence (about 20 px, second ink) and the primary
action, "Download for macOS" or the visitor's system, in magenta, with "See what Pro adds" beside it
as a text link, and under them one line that QuotaDial is free and open source under the MIT
licence, for macOS, Windows and Linux (the searches ask for free and GitHub; see the home
blueprint). The right seven columns hold the almanac plate on the shallow field: the real
QuotaDial window with example accounts, overlapping a three-row tide table (window, used, resets,
at this pace) under the week's curve rising to the high-water line, its dashed prediction meeting
100% before the reset mark. Where seven columns cannot hold the table's times whole beside the
window, the window is layered over the plate's lower edge instead, and `components/home/Hero.tsx`
says why. Phone 390x844: H1, answer, action, and the plate's window image starting inside the
fold.

**FORM.** The tide table and the almanac: candidate 7 of 7 on the grounded list
(`docs/design/candidates.md`), assigned by the concept seed recorded in
`docs/seo/site.config.json`.

**FINISH.** unreviewed and undocumented is unfinished; this build ends with the finish review, the
verdict, and DESIGN.md

## Component vocabulary (the foundation builds these; lanes compose them)

| Component | What it is | Rules |
| --- | --- | --- |
| `Section` | A full-width band with a depth tone: `chart`, `shallow`, `mid`, `deep` | Tones descend down the page; one tone per band; more space above a heading than below |
| `Plate` | An almanac plate: a figure, its table and its caption, on a field tone | Replaces cards; never nested; the caption says what the figure shows and that accounts are examples |
| `TideTable` | Rows of predictions with tabular figures | Header in Sofia Sans Semi Condensed 600, normal case; rows grouped in bands of shading; a "runs out first" cell is magenta with the words, never colour alone |
| `TideCurve` | SVG: time along, share used up, the high-water line at 100% dashed, a reset mark, the prediction dashed | Line blue on the field; labels in condensed italic; an accessible table equivalent beside it |
| `AppShot` | A capture of the real QuotaDial window (light or dark to match the theme) | From `check-window.mjs` with the site's example accounts; alt says what it shows and "example accounts" |
| `Dial` | The app icon, `app-icon.svg` | The only pictorial mark; its needle may sweep once on the home page |
| `Button` | Primary (magenta fill, white text), secondary (1 px ink border), text link | One primary per view; labels name the action; one line at desktop |
| `CodeBlock` | JetBrains Mono on the mid field, a copy button | Code and commands only |
| `Notice` | A 1 px bordered block for steps with a caution | No side rail; the caution sentence may be magenta |
| `Question` | An H3 and its answer, always open | No accordion: every answer readable and indexable |
| `Header` | Mark and wordmark, nav (Download, Pro, Privacy, Changelog), GitHub | One line at desktop, 64 px tall; a menu button under 768 px |
| `Footer` | On the deep: links, the MIT line, the privacy line, the example-data note | |

Icons: Phosphor (`@phosphor-icons/react`, MIT), regular weight, only for download, copy, external
link and the three systems' logos.

## Form fields

The site has no form today (the key page reads its link; support is a mail link). If one is added:
label above, helper text below the label, error below the field naming the problem and the fix;
1 px border in second ink, 2 px corners, magenta focus ring 2 px with a 2 px offset.

## Motion

One authored moment per page, and only on the home page: on load, the week's curve draws to its
prediction while the dial's needle sweeps to the share used, about 900 ms, ease-out from a visible
state. Every other change is a state change (a copied key, a menu opening). `prefers-reduced-motion`
shows the final state with no movement. No reveal-on-scroll.

## Anti-defaults

No kicker or eyebrow above a heading. No cream, parchment or paper grounds. No icon-in-a-square
grids, no three equal cards. No gradient text. No photograph under a dark overlay. No "Get started"
or "Learn more". No centred hero. No neon on black, no monospace outside code. No nautical clip-art:
anchors, ropes, ship's wheels, waves as ornament. No interface drawn from divs to look like the app:
the app's real window only. No price, rating, user count, countdown or testimonial. No em dashes.

## Copy voice

Plain, specific and numerate: times, percentages, file names. Second person. The product's words
(#52): a window, a reset, a sign-in; never quota, auth or credential. The almanac is the look; the
words stay the product's own, with at most one tide image per page. British English. Built from
`docs/copy/source-library.md` under codus-copy's rules.
