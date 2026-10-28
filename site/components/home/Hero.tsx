import { TIDE_ROWS, WEEKLY_CURVE } from "../../lib/data/examples.ts";
import { AppShot } from "../AppShot.tsx";
import { ButtonLink } from "../Button.tsx";
import { DownloadButton } from "../DownloadButton.tsx";
import { Plate } from "../Plate.tsx";
import { Section } from "../Section.tsx";
import { TideCurve } from "../TideCurve.tsx";
import { TideTable } from "../TideTable.tsx";

const CURVE_TITLE =
  "Claude's weekly window, example accounts: 91% used on Tuesday at 14:10, filling at 2.0% an hour, so it reaches 100% on Tuesday at 18:40, long before it resets on Thursday at 06:00";

/**
 * Under 640 px each tide-table row is set as two short lines, the window and its share used above,
 * its reset and its prediction below, because four columns of times that never break need 430 px
 * and a phone's plate has about 320. The cells are the same ones; only their layout changes.
 */
const TABLE_ON_A_PHONE =
  "max-sm:block max-sm:[&_tbody]:block max-sm:[&_thead]:sr-only max-sm:[&_tr]:grid max-sm:[&_tr]:grid-cols-[1fr_auto] " +
  "max-sm:[&_tr]:gap-x-3 max-sm:[&_td:nth-child(2)]:after:content-['_used'] max-sm:[&_td:nth-child(3)]:text-left " +
  "max-sm:[&_td:nth-child(3)]:before:content-['Resets_']";

/**
 * The first screen (docs/DIRECTION.md, FIRST VIEWPORT): on a wide screen the name, the answer and
 * the action fill the left five columns, their middle level with the plate's middle, and the right seven hold one
 * almanac plate: the week's curve across the top, the tide table under it with every prediction
 * on one line, and the real window layered over the plate's lower edge, beside its caption, its lower part
 * faded out below the Claude account and running a little past the plate's foot. Seven columns give the plate about 650 px; the table needs 430 to hold its
 * times whole and the window 260, so the window cannot share the table's row without covering a
 * prediction. On a phone the order is the H1, the answer, the action and then the window, which
 * is first in the plate's markup for that reason. The curve draws itself once on load, the page's
 * one moment of motion.
 */
export function Hero() {
  return (
    <Section
      tone="chart"
      labelledBy="home-title"
      className="[&>div]:pt-10 md:[&>div]:pt-12 [&>div]:pb-16 md:[&>div]:pb-20"
    >
      <div className="grid grid-cols-[minmax(0,1fr)] gap-x-12 gap-y-10 xl:grid-cols-12">
        <div className="xl:col-span-5 xl:self-start xl:pt-[8rem]">
          <h1
            id="home-title"
            className="m-0 max-w-[20ch] text-[2.75rem] leading-[0.98] font-[800] tracking-[-0.015em] sm:text-[3.75rem] lg:max-w-[16ch] lg:text-[4.25rem] xl:max-w-none xl:text-[3.75rem]"
          >
            Claude and Codex usage tracker for your menu bar or tray
          </h1>
          <p className="m-0 mt-6 max-w-[40ch] text-[1.1875rem] leading-[1.45] text-ink-2 md:text-[1.25rem]">
            See how much of each Claude and ChatGPT/Codex plan limit you have used, and when it resets, from Claude
            Code's status line and the sign-in the Codex CLI already keeps.
          </p>
          <div className="mt-7 flex flex-wrap items-center gap-x-6 gap-y-4">
            <DownloadButton />
            <ButtonLink variant="text" href="/pro">
              See what Pro adds
            </ButtonLink>
          </div>
          {/* Free and open source stays under the action: the keyword map's related searches ask for free and GitHub (docs/seo/blueprints/home.md). */}
          <p className="m-0 mt-5 max-w-[40ch] text-ink-2">
            Free and open source under the MIT licence, for macOS, Windows and Linux.
          </p>
        </div>

        <Plate
          className="xl:col-span-7 lg:mb-[8rem]"
          title="Example accounts, Tuesday 14:10"
          caption={
            <span className="block lg:max-w-[calc(100%-20.5rem)]">
              Claude's weekly window from Thursday 06:00 as a curve, its three windows in the table, and the window as
              QuotaDial draws it.
            </span>
          }
        >
          <div className="grid grid-cols-[minmax(0,1fr)] gap-5 lg:grid-cols-[minmax(0,1fr)_18.5rem]">
            <div className="relative z-10 mx-auto w-full max-w-[18.5rem] lg:col-start-2 lg:row-start-3 lg:-mb-[18rem] lg:max-h-[20rem] lg:max-w-none lg:overflow-hidden lg:[mask-image:linear-gradient(to_bottom,black_82%,transparent)]">
              <AppShot
                name="accounts"
                alt="QuotaDial's window: Claude Work and Codex, each window with its share used, reset and pace"
                width={720}
                height={1412}
                priority
                className="block w-full"
              />
            </div>
            <div className="lg:col-span-2 lg:row-start-1">
              <TideCurve
                title={CURVE_TITLE}
                hours={WEEKLY_CURVE.hours}
                days={WEEKLY_CURVE.days}
                points={WEEKLY_CURVE.points}
                reset={WEEKLY_CURVE.reset}
                runsOut={WEEKLY_CURVE.runsOut}
                animate
              />
            </div>
            <TideTable
              className={`lg:col-span-2 lg:row-start-2 ${TABLE_ON_A_PHONE}`}
              caption={{ text: "Each example window, with its share used, its reset and what its pace says" }}
              columns={["Window", "Used", "Resets", "At this pace"]}
              align={["words", "figures", "figures", "figures"]}
              whole={[true, true, true, true]}
              rows={TIDE_ROWS.map((row) => ({
                key: row.window,
                group: row.window.split(" ")[0],
                cells: [row.window, `${row.used}%`, row.resets, { text: row.pace, caution: row.runsOutFirst }],
              }))}
            />
          </div>
        </Plate>
      </div>
    </Section>
  );
}
