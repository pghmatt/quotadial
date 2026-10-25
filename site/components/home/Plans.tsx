import { FREE_HAS, PRO_FEATURES } from "../../lib/data/pro.ts";
import { ButtonLink } from "../Button.tsx";
import { Plate } from "../Plate.tsx";
import { SectionHeading } from "../Section.tsx";
import { BandedTable } from "./BandedTable.tsx";
import { Part } from "./Part.tsx";
import { PLAN_ROWS } from "./plan-rows.ts";
import { Body, Lead } from "./Text.tsx";

const lines = (source: readonly string[], indexes: number[]) =>
  indexes.length ? (
    <>
      {indexes.map((index) => (
        <span key={index} className="block [&+&]:mt-2">
          {source[index]}
        </span>
      ))}
    </>
  ) : null;

/** Free and Pro, in one plate. */
export function Plans() {
  return (
    <Part id="plans" headingId="plans-title">
      <SectionHeading id="plans-title">QuotaDial Free and Pro</SectionHeading>
      <Lead>
        Free needs no key. Pro adds every account, a year of history, early warnings, webhooks and Slack, the week's
        cost and headroom for agents.
      </Lead>
      <div className="mt-8">
        <Plate tone="chart" title="QuotaDial Free beside QuotaDial Pro">
          <BandedTable
            caption="What QuotaDial Free has, and what QuotaDial Pro adds, topic by topic"
            columns={["Topic", "QuotaDial Free", "QuotaDial Pro adds"]}
            rows={PLAN_ROWS.map((row) => ({
              key: row.topic,
              cells: [row.topic, lines(FREE_HAS, row.free), lines(PRO_FEATURES, row.pro)],
            }))}
          />
        </Plate>
      </div>
      <Body>
        Pro is a key you paste into the app, checked on your machine, so there is one download for both.
      </Body>
      <p className="m-0 mt-6">
        <ButtonLink variant="secondary" href="/pro">
          See what Pro adds
        </ButtonLink>
      </p>
    </Part>
  );
}
