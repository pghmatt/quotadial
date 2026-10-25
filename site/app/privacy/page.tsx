import type { Metadata } from "next";

import { Section } from "../../components/Section.tsx";
import { pageMetadata } from "../../lib/metadata.ts";

// The foundation's placeholder; its lane builds the page from docs/seo/blueprints/privacy.md.
export const metadata: Metadata = pageMetadata({
  title: "What QuotaDial reads and sends",
  description:
    "Every address QuotaDial connects to, and nothing else: Anthropic and ChatGPT for your limits, their status pages, GitHub for updates, and webhooks you add.",
  path: "/privacy",
});

export default function Privacy() {
  return (
    <Section>
      <h1 className="m-0 text-[2.5rem] leading-[1.05] font-[800]">What QuotaDial reads and sends</h1>
    </Section>
  );
}
