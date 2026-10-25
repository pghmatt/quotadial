import type { Metadata } from "next";

import { Section } from "../components/Section.tsx";
import { pageMetadata } from "../lib/metadata.ts";

// The foundation's placeholder; the home lane builds the page from docs/seo/blueprints/home.md.
export const metadata: Metadata = pageMetadata({
  title: "Claude and Codex usage tracker for the menu bar",
  description:
    "See how much of each Claude and ChatGPT/Codex plan limit you have used and when it resets, from the sign-in your CLI keeps. For macOS, Windows and Linux.",
  path: "/",
});

export default function Home() {
  return (
    <Section>
      <h1 className="m-0 text-[2.5rem] leading-[1.05] font-[800]">Claude and Codex usage tracker for your menu bar or tray</h1>
    </Section>
  );
}
