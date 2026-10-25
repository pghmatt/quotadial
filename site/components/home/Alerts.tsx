import { AppShot } from "../AppShot.tsx";
import { Plate } from "../Plate.tsx";
import { SectionHeading } from "../Section.tsx";
import { BandedTable } from "./BandedTable.tsx";
import { Part } from "./Part.tsx";
import { Body, Lead } from "./Text.tsx";

/** Alerts: two rules as typed, what each one tells you, and the real Settings window. */
export function Alerts() {
  return (
    <Part id="alerts" headingId="alerts-title">
      <div className="grid grid-cols-[minmax(0,1fr)] gap-10 lg:grid-cols-[21.5rem_minmax(0,1fr)] lg:items-start lg:gap-20">
        <div className="lg:order-last">
          <SectionHeading id="alerts-title">A warning before a run stops</SectionHeading>
          <Lead>
            Write a rule in plain words, such as “warn me at 80%” or “tell me 30 minutes before the weekly window
            runs out”, and QuotaDial reads it back to you and sends it as a notification from your system.
          </Lead>
          <div className="mt-8">
            <BandedTable
              caption="Two example alert rules as typed, and the notification each one gives"
              columns={["You type", "QuotaDial tells you"]}
              rows={[
                {
                  key: "share",
                  cells: [
                    "warn me at 80%",
                    <>
                      <strong className="font-[650]">Claude Weekly passed 80%</strong>
                      <br />
                      dev@example.com: 81% used, resets in <span className="whitespace-nowrap">1 d 21 h</span>.
                    </>,
                  ],
                },
                {
                  key: "runs-out",
                  cells: [
                    "tell me 30 minutes before the weekly window runs out",
                    <>
                      <strong className="font-[650]">Claude Weekly runs out before it resets</strong>
                      <br />
                      dev@example.com: 99% used, at 2.0%/h, runs out in <span className="whitespace-nowrap">30 min</span>, <span className="whitespace-nowrap">1 d 11 h</span> before it resets.
                    </>,
                  ],
                },
              ]}
            />
            <p className="m-0 mt-2 px-3 font-table text-[0.9375rem] text-ink-2 italic">Example accounts.</p>
          </div>
          <Body>
            Warnings for a share used, a reset and a sign-in come with QuotaDial Free. The ones that look ahead, such as
            the 30 minute rule, are part of QuotaDial Pro, because they use the pace.
          </Body>
        </div>
        <Plate caption="Example accounts. Settings, with three alert rules and how QuotaDial read each one.">
          <div className="mx-auto max-h-[30rem] w-full max-w-[18.5rem] overflow-hidden [mask-image:linear-gradient(to_bottom,black_88%,transparent)]">
            <AppShot
              name="settings"
              alt="QuotaDial's settings: three alert rules in plain words, each with how QuotaDial read it"
              width={720}
              height={1826}
            />
          </div>
        </Plate>
      </div>
    </Part>
  );
}
