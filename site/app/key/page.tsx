import type { Metadata } from "next";

import { KeyFlow } from "../../components/conversion/KeyFlow.tsx";
import { Section } from "../../components/Section.tsx";
import { pageMetadata } from "../../lib/metadata.ts";

export const metadata: Metadata = pageMetadata({
  title: "Your QuotaDial Pro key",
  description:
    "Collect the QuotaDial Pro key from your Lemon Squeezy receipt link, then paste it into QuotaDial's Settings or run quotadial licence add on your machine.",
  path: "/key",
  noindex: true,
});

/**
 * A buyer arrives with the receipt's key in the address. Before the page's script has run, the
 * steps for a visitor with no link would flash on the screen, so the page marks itself as soon as
 * it sees the key and hides them for those moments.
 */
const ARRIVING = `if(/[#&]licence=/.test(location.hash))document.documentElement.setAttribute("data-arriving","")`;
const HIDE_UNTIL_READ = "html[data-arriving] [data-none-state]{visibility:hidden}";

export default function Key() {
  return (
    <Section className="md:min-h-[40rem]">
      <style dangerouslySetInnerHTML={{ __html: HIDE_UNTIL_READ }} />
      <script dangerouslySetInnerHTML={{ __html: ARRIVING }} />
      <KeyFlow heading={<h1 className="m-0 text-[2.75rem] leading-[1.02] font-[800] md:text-[4rem]">Your QuotaDial key</h1>} />
    </Section>
  );
}
