import type { Metadata } from "next";

import { Section } from "../../components/Section.tsx";
import { pageMetadata } from "../../lib/metadata.ts";

// The foundation's placeholder; its lane builds the page from docs/seo/blueprints/support.md.
export const metadata: Metadata = pageMetadata({
  title: "Support, feedback and security reports",
  description:
    "Write to support@quotadial.com, open an issue on GitHub with the details support needs, or report a security problem privately, as SECURITY.md describes.",
  path: "/support",
});

export default function Support() {
  return (
    <Section>
      <h1 className="m-0 text-[2.5rem] leading-[1.05] font-[800]">Get help with QuotaDial</h1>
    </Section>
  );
}
