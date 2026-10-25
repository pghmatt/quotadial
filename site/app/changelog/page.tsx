import type { Metadata } from "next";

import { Section } from "../../components/Section.tsx";
import { pageMetadata } from "../../lib/metadata.ts";

// The foundation's placeholder; its lane builds the page from docs/seo/blueprints/changelog.md.
export const metadata: Metadata = pageMetadata({
  title: "Changelog: what changed in each version",
  description:
    "Every change people will notice in QuotaDial, newest first: what each version added, changed and fixed, from the changelog the app itself shows.",
  path: "/changelog",
});

export default function Changelog() {
  return (
    <Section>
      <h1 className="m-0 text-[2.5rem] leading-[1.05] font-[800]">What changed in QuotaDial</h1>
    </Section>
  );
}
