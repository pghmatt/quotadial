import { Plate } from "../Plate.tsx";
import { SectionHeading } from "../Section.tsx";
import { READ_ALOUD } from "./examples.ts";
import { Part } from "./Part.tsx";
import { Body, Lead, Times } from "./Text.tsx";

const PARTS = [
  { term: "Share used", says: "How much of the window is gone." },
  { term: "Reset", says: "When the window starts again at zero." },
  { term: "Age and source", says: "Where the number came from, and how long ago." },
  {
    term: "Pace",
    says: `How fast the window fills, and whether it reaches 100% before it resets. Here it does, ${READ_ALOUD.early} early.`,
  },
];

/** What it tracks: every window, and one row of the app read aloud, part by part. */
export function Limits() {
  return (
    <Part id="limits" headingId="limits-title">
      <div className="grid grid-cols-[minmax(0,1fr)] gap-10 lg:grid-cols-[minmax(0,1fr)_minmax(0,34rem)] lg:items-start lg:gap-20">
        <div>
          <SectionHeading id="limits-title">Each limit, its reset and its pace</SectionHeading>
          <Lead>
            QuotaDial lists every window you have: Claude's five-hour session, its weekly limits and the weekly limit
            for each model, and each of Codex's windows. Each one shows the share used, when it resets, how old the
            number is and whether, at the pace you are going, it runs out before it resets.
          </Lead>
          <Body>
            In the menu bar or tray you see the tightest window at a glance, and one click opens the rest. Every number
            says where it came from: the API, the cache or the Codex CLI's own logs. When a window suddenly reads 0%,
            QuotaDial holds it until a second fetch agrees, because a provider sometimes answers 0% by mistake.
          </Body>
        </div>
        <Plate
          tone="chart"
          caption="Example accounts. One row from Claude Work's weekly window, Tuesday 14:10."
        >
          <p className="m-0 font-table text-[1.5rem] leading-[1.25] font-[600] md:text-[1.75rem]">
            {READ_ALOUD.label} {READ_ALOUD.used}%, resets in <span className="whitespace-nowrap">{READ_ALOUD.resets}</span>,
            from the API <span className="whitespace-nowrap">{READ_ALOUD.fetched}</span>;{" "}
            <span className="text-magenta">
              at {READ_ALOUD.rate}, runs out in <span className="whitespace-nowrap">{READ_ALOUD.runsOut}</span>
            </span>
          </p>
          <dl className="m-0 mt-6 font-table">
            {PARTS.map((part) => (
              <div
                key={part.term}
                className="grid gap-x-4 px-3 py-2.5 even:bg-[color-mix(in_srgb,var(--ink)_5%,transparent)] sm:grid-cols-[8rem_1fr]"
              >
                <dt className="font-[650]">{part.term}</dt>
                <dd className="m-0 text-ink-2">
                  <Times>{part.says}</Times>
                </dd>
              </div>
            ))}
          </dl>
        </Plate>
      </div>
    </Part>
  );
}
