import type { Metadata } from "next";

import { ButtonLink } from "../components/Button.tsx";
import { Section } from "../components/Section.tsx";

export const metadata: Metadata = {
  title: { absolute: "Page not found | QuotaDial" },
  robots: { index: false, follow: true },
};

/** Any address the site does not have. */
export default function NotFound() {
  return (
    <Section>
      <h1 className="m-0 text-[2.5rem] leading-[1.05] font-[800]">This page is not on the chart</h1>
      <p className="mt-4 max-w-[60ch] text-ink-2">
        The address may be mistyped, or the page may have moved. Everything QuotaDial offers is a
        link or two from the home page.
      </p>
      <p className="mt-6">
        <ButtonLink href="/" variant="secondary">
          Go to the home page
        </ButtonLink>
      </p>
    </Section>
  );
}
