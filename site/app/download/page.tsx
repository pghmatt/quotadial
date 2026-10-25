import type { Metadata } from "next";

import { Section } from "../../components/Section.tsx";
import { pageMetadata } from "../../lib/metadata.ts";

// The foundation's placeholder; its lane builds the page from docs/seo/blueprints/download.md.
export const metadata: Metadata = pageMetadata({
  title: "Download for macOS, Windows and Linux",
  description:
    "Download QuotaDial for macOS, Windows or Linux from its GitHub releases, check the file against its SHA-256, and open a build that is not signed yet.",
  path: "/download",
});

export default function Download() {
  return (
    <Section>
      <h1 className="m-0 text-[2.5rem] leading-[1.05] font-[800]">Download QuotaDial</h1>
    </Section>
  );
}
