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
 * The first screen: the name of the thing across the top, what it does with the action beside it,
 * then the almanac plates: the real window, and the week as a table and a curve. The curve draws
 * itself once on load, the page's one moment of motion.
 */
export function Hero() {
  return (
    <Section
      tone="chart"
      labelledBy="home-title"
      className="[&>div]:pt-10 md:[&>div]:pt-12 [&>div]:pb-16 md:[&>div]:pb-20"
    >
      <div className="grid grid-cols-[minmax(0,1fr)] gap-x-12 gap-y-8 lg:grid-cols-[minmax(0,1fr)_23rem] lg:items-end">
        <h1
          id="home-title"
          className="m-0 max-w-[20ch] text-[2.75rem] leading-[0.98] font-[800] tracking-[-0.015em] sm:text-[3.75rem] lg:max-w-none lg:text-[4.25rem] xl:text-[5rem]"
        >
          Claude and Codex usage tracker for your menu bar or tray
        </h1>
        <div>
          <p className="m-0 max-w-[40ch] text-[1.1875rem] leading-[1.45] text-ink-2 md:text-[1.25rem]">
            See how much of each Claude and ChatGPT/Codex plan limit you have used, and when it resets, from the
            sign-in Claude Code and the Codex CLI already keep.
          </p>
          <div className="mt-6 flex flex-wrap items-center gap-x-6 gap-y-4">
            <DownloadButton />
            <ButtonLink variant="text" href="/pro">
              See what Pro adds
            </ButtonLink>
          </div>
        </div>
      </div>

      <div className="mt-10 grid grid-cols-[minmax(0,1fr)] items-stretch gap-6 md:mt-12 lg:grid-cols-[21.5rem_minmax(0,1fr)]">
        <Plate
          caption={
            <>
              Example accounts. The window as QuotaDial draws it on a Tuesday at 14:10: the weekly limit at 91%, the
              first to run out.
            </>
          }
        >
          <AppShot
            name="accounts"
            alt="QuotaDial's window: Claude Work and Codex, each window with its share used, reset and pace"
            width={720}
            height={1214}
            priority
            className="mx-auto block w-full max-w-[18.5rem]"
          />
        </Plate>
        <Plate
          caption={
            <>
              Example accounts. Claude's weekly window from Thursday 06:00: the line is the share used so far, the
              dashed line is where it goes at this pace.
            </>
          }
        >
          <div className="mx-auto max-w-[46rem]">
            <TideCurve
              title={CURVE_TITLE}
              hours={WEEKLY_CURVE.hours}
              days={WEEKLY_CURVE.days}
              points={WEEKLY_CURVE.points}
              reset={WEEKLY_CURVE.reset}
              runsOut={WEEKLY_CURVE.runsOut}
              animate
            />
            <TideTable
              className="mt-5"
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
