import { AppShot } from "../AppShot.tsx";
import { Plate } from "../Plate.tsx";
import { SectionHeading } from "../Section.tsx";
import { BandedTable } from "./BandedTable.tsx";
import { ALERT_SAMPLES } from "./examples.ts";
import { Part } from "./Part.tsx";
import { Body, Lead, Times } from "./Text.tsx";

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
              rows={ALERT_SAMPLES.map((sample) => ({
                key: sample.key,
                cells: [
                  sample.rule,
                  <>
                    <strong className="font-[650]">{sample.title}</strong>
                    <br />
                    <Times>{sample.text}</Times>
                  </>,
                ],
              }))}
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
              height={2172}
            />
          </div>
        </Plate>
      </div>
    </Part>
  );
}
