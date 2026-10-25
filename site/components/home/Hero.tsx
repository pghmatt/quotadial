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
 * The first screen (docs/DIRECTION.md, FIRST VIEWPORT): on a wide screen the name, the answer and
 * the action fill the left five columns, and the right seven hold one almanac plate: the week's
 * curve across the top, the tide table under it, and the real window overlapping the table's
 * right-hand margin and running past the plate's foot. On a phone the order is the H1, the answer,
 * the action and then the window, which is first in the plate's markup for that reason. The curve
 * draws itself once on load, the page's one moment of motion.
 */
export function Hero() {
  return (
    <Section
      tone="chart"
      labelledBy="home-title"
      className="[&>div]:pt-10 md:[&>div]:pt-12 [&>div]:pb-16 md:[&>div]:pb-20"
    >
      <div className="grid grid-cols-[minmax(0,1fr)] gap-x-12 gap-y-10 xl:grid-cols-12">
        <div className="xl:col-span-5 xl:self-start xl:pt-14">
          <h1
            id="home-title"
            className="m-0 max-w-[20ch] text-[2.75rem] leading-[0.98] font-[800] tracking-[-0.015em] sm:text-[3.75rem] lg:max-w-[16ch] lg:text-[4.25rem] xl:max-w-none xl:text-[3.75rem]"
          >
            Claude and Codex usage tracker for your menu bar or tray
          </h1>
          <p className="m-0 mt-6 max-w-[40ch] text-[1.1875rem] leading-[1.45] text-ink-2 md:text-[1.25rem]">
            See how much of each Claude and ChatGPT/Codex plan limit you have used, and when it resets, from the
            sign-in Claude Code and the Codex CLI already keep.
          </p>
          <div className="mt-7 flex flex-wrap items-center gap-x-6 gap-y-4">
            <DownloadButton />
            <ButtonLink variant="text" href="/pro">
              See what Pro adds
            </ButtonLink>
          </div>
        </div>

        <Plate
          className="xl:col-span-7 lg:mb-[12.5rem] xl:mb-[9rem]"
          caption={
            <span className="block lg:max-w-[calc(100%-22rem)]">
              Example accounts. Claude's weekly window from Thursday 06:00 as a curve, its three windows in the
              table, and the window as QuotaDial draws it on a Tuesday at 14:10.
            </span>
          }
        >
          <div className="grid grid-cols-[minmax(0,1fr)] gap-5 lg:grid-cols-[minmax(0,1fr)_17.5rem] lg:gap-x-0 lg:gap-y-5">
            <AppShot
              name="accounts"
              alt="QuotaDial's window: Claude Work and Codex, each window with its share used, reset and pace"
              width={720}
              height={1214}
              priority
              className="relative z-10 mx-auto block w-full max-w-[18.5rem] lg:col-start-2 lg:row-start-2 lg:-mb-[20rem] lg:-ml-6 lg:max-w-none"
            />
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
              className="lg:col-start-1 lg:row-start-2 lg:[&_td:last-child]:pr-10 lg:[&_th:last-child]:pr-10"
              caption={{ text: "Each example window, with its share used, its reset and what its pace says" }}
              columns={["Window", "Used", "Resets", "At this pace"]}
              rows={TIDE_ROWS.map((row) => ({
                key: row.window,
                cells: [row.window, `${row.used}%`, row.resets, { text: row.pace, caution: row.runsOutFirst }],
              }))}
            />
          </div>
        </Plate>
      </div>
    </Section>
  );
}
