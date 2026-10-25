import type { Metadata } from "next";

import { Section } from "../../components/Section.tsx";
import { pageMetadata } from "../../lib/metadata.ts";

// The foundation's placeholder; its lane builds the page from docs/seo/blueprints/pro.md.
export const metadata: Metadata = pageMetadata({
  title: "What Pro adds: every account and a year of history",
  description:
    "QuotaDial Pro adds every account on the machine, a year of history, early warnings, webhooks and Slack, and the week's cost. QuotaDial Free stays free.",
  path: "/pro",
});

export default function Pro() {
  return (
    <Section>
      <h1 className="m-0 text-[2.5rem] leading-[1.05] font-[800]">QuotaDial Pro</h1>
    </Section>
  );
}
