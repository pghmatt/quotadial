# Seven candidate worlds for quotadial.com

Written before the roll, 2026-10-25, from PRODUCT.md and the search study.

## The mechanism, the scene, the home

- **Mechanism, in one sentence:** QuotaDial reads the sign-in Claude Code and the Codex CLI already
  keep and keeps every plan limit in view, with its share used, when it resets, how old the number
  is and whether it runs out before the reset, in the menu bar and for agents.
- **Scene:** a developer two hours into an agent run, terminal full height, the menu bar along the
  top of the screen, and the dread of "limit reached, resets at 3:00 PM" arriving mid-task.
- **Cultural home:** terminals, menu-bar utilities, open source on GitHub, Claude Code and Codex
  themselves, the culture of watching a long job finish.
- **What the first surface must prove:** the number is honest and always in view, needs no new
  sign-in, and works on any system.

## The rut (excluded)

- **The page this category always ships:** a centred indie Mac-app hero, a heavy sans headline
  over a soft gradient, a floating screenshot of the menu-bar popover, a blue pill "Download" or a
  price button, then avatars, stars and a feature grid.
- **Its predictable opposite:** the dark developer-docs template (VitePress or Docusaurus dark,
  icon cards, "Get Started" and "View on GitHub"), or the terminal costume (near-black, one neon
  green, monospace everywhere).
- **The name's literal reading:** a dial or gauge. At most one candidate spends it (number 4).

## The seven, ordered by resonance

1. **Pit-wall live timing.** The timing tower and strategy screens a Formula 1 team watches during a
   stint: every car ranked by gap, tyre life counting down, the pit window open or closed. A long
   agent run is a stint; each account is a car; headroom is the gap; "runs out before the reset" is
   the tyre that will not last to the pit window. Developers watch this sport, and the timing tower
   is already a ranked list of the same facts QuotaDial ranks.
2. **The departure board and the printed timetable.** The station board and the pocket timetable:
   services in rows, each with its next departure and its status. "When does my limit reset" is the
   question people search most after hitting one; a timetable answers "when" better than any other
   object, and its status column carries "on time" or "runs out first".
3. **The EV trip planner.** The car's range display and route planner: range left, the charge you
   arrive with, the stop you need before you run dry. Range anxiety is the exact feeling of watching
   a weekly limit drain mid-run, and "arrive with 12%" is the same sentence as "runs out 40 min before
   it resets".
4. **The watch's power-reserve complication.** The small sub-dial on a mechanical watch that shows
   how many hours remain before the spring stops it, until it is wound again. It is the dial in
   QuotaDial's name and in its icon, made precise: a reserve that empties and is wound at a known
   time. This candidate spends the literal reading.
5. **The RFC and the man page.** The plain-text specification developers read and quote: numbered
   sections, MUST and MUST NOT, examples set apart. QuotaDial's promises are spec-like ("MUST NOT
   write, refresh, move or log a sign-in"), and a page written as a specification makes privacy a
   testable statement rather than a slogan.
6. **The airport wayfinding system.** A complete information system in the tradition of Swiss
   International Style signage and the great transport identity programmes: one grid, flat colour
   fields that each mean one state, pictograms, nothing decorative. QuotaDial's three states
   (normal, past 70%, past 90%) and three systems map onto signage's colour codes and symbols.
7. **The tide table and the almanac.** Printed predictions of high and low water with their times,
   and the curve between them. QuotaDial predicts too: when a window runs out at its pace against
   when it resets; a table of predicted times with a curve is how sailors have read that for
   centuries.

## Material families

Motorsport broadcast screens (1), transit signage and timetables (2), automotive interfaces (3),
horology (4), technical documentation (5), wayfinding and identity programmes (6), nautical print and
data graphics (7): seven families, no more than one candidate in any.

## The roll (2026-10-25)

`concept-seed.mjs --scope direction --mode persuade --grain product --platform web --candidate-count 7`
(seed key in `docs/seo/site.config.json`) assigned **candidate 7, the tide table and the almanac**.

The dealt challengers, each fused with QuotaDial's facts and weighed against the assignment on
audience identification and product clarity:

| Challenger | Fused with QuotaDial | Verdict |
| --- | --- | --- |
| A Miura-fold sheet unfolding from a packet | The menu-bar dial unfolding into the window and its history | Loses both: folding names nothing QuotaDial does beyond "open the popover", and developers do not read deployable sheets |
| A tensegrity column under tension | Limits as forces in balance | Loses both: no fact of the product is a force |
| A pulp paperback rack | "The run that stopped halfway" as a lurid cover line | Loses both: drama undercuts a tool whose case is trust and privacy |
| A chaekgeori bookshelf in reverse perspective | Accounts as shelf boxes | Loses both: an inventory of objects carries no limit, reset or pace |
| A CRT oscilloscope on a signal bench | Usage as a sawtooth that drops at each reset, the trigger level as an alert threshold, channels as accounts | The strongest: it matches the assignment on product clarity and beats it on identification, but its grammar (phosphor green on near-black glass, instrument capitals) is the terminal costume this list excluded as the category's predictable opposite; built, it would be guessable from the category. Named alternate. |
| A risograph web system | Spot-ink sections over paper | Loses both: a print texture says nothing about limits, and its warm paper ground is a default this build refuses |

The assignment stands: the tide table carries QuotaDial's distinct fact, prediction (when a window
runs out at its pace against when it resets), in the form people have read predicted times in for
centuries, and it lands nowhere near the category's pages.
