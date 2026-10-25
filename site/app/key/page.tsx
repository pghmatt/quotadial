import type { Metadata } from "next";

import { Section } from "../../components/Section.tsx";
import { pageMetadata } from "../../lib/metadata.ts";

// The foundation's placeholder; its lane builds the page from docs/seo/blueprints/key.md.
export const metadata: Metadata = pageMetadata({
  title: "Your QuotaDial Pro key",
  description:
    "Collect the QuotaDial Pro key from your Lemon Squeezy receipt link, then paste it into QuotaDial's Settings or run quotadial licence add on your machine.",
  path: "/key",
  noindex: true,
});

export default function Key() {
  return (
    <Section>
      <h1 className="m-0 text-[2.5rem] leading-[1.05] font-[800]">Your QuotaDial key</h1>
    </Section>
  );
}
