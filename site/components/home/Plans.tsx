import { FREE_HAS, PRO_FEATURES } from "../../lib/data/pro.ts";
import { ButtonLink } from "../Button.tsx";
import { Plate } from "../Plate.tsx";
import { SectionHeading } from "../Section.tsx";
import { Part } from "./Part.tsx";
import { Body, Lead } from "./Text.tsx";

function List({ title, items }: { title: string; items: readonly string[] }) {
  return (
    <div>
      <h3 className="m-0 text-[1.75rem] leading-none font-[750]">{title}</h3>
      <ul className="m-0 mt-4 list-none p-0">
        {items.map((item) => (
          <li key={item} className="px-3 py-2.5 even:bg-[color-mix(in_srgb,var(--ink)_5%,transparent)]">
            {item}
          </li>
        ))}
      </ul>
    </div>
  );
}

/** Free and Pro, side by side, with no figure beside either. */
export function Plans() {
  return (
    <Part id="plans" headingId="plans-title">
      <SectionHeading id="plans-title">QuotaDial Free and Pro</SectionHeading>
      <Lead>
        Free needs no key. Pro adds every account, a year of history, early warnings, webhooks and Slack, the week's
        cost and headroom for agents.
      </Lead>
      <div className="mt-8">
        <Plate tone="chart" className="md:p-8">
          <div className="grid gap-10 md:grid-cols-[5fr_6fr] md:gap-12">
            <List title="QuotaDial Free" items={FREE_HAS} />
            <List title="QuotaDial Pro adds" items={PRO_FEATURES} />
          </div>
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
