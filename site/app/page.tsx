import type { Metadata } from "next";

import { Agents } from "../components/home/Agents.tsx";
import { Addresses } from "../components/home/Addresses.tsx";
import { Alerts } from "../components/home/Alerts.tsx";
import { Close } from "../components/home/Close.tsx";
import { Hero } from "../components/home/Hero.tsx";
import { Limits } from "../components/home/Limits.tsx";
import { Plans } from "../components/home/Plans.tsx";
import { Questions } from "../components/home/Questions.tsx";
import { SignIn } from "../components/home/SignIn.tsx";
import { Systems } from "../components/home/Systems.tsx";
import { Section } from "../components/Section.tsx";
import { jsonLd, pageMetadata, softwareJsonLd } from "../lib/metadata.ts";

export const metadata: Metadata = pageMetadata({
  title: "Claude and Codex usage tracker for the menu bar",
  description:
    "See how much of each Claude and ChatGPT/Codex plan limit you have used and when it resets, from the sign-in your CLI keeps. For macOS, Windows and Linux.",
  path: "/",
});

/**
 * The home page, from docs/seo/blueprints/home.md: the first screen, then the sections in the
 * blueprint's order, in bands that go from the shallow to the mid and end on the deep.
 */
export default function Home() {
  return (
    <>
      <script type="application/ld+json" dangerouslySetInnerHTML={{ __html: jsonLd(softwareJsonLd()) }} />
      <Hero />
      <Section tone="shallow">
        <Limits />
        <SignIn />
        <Alerts />
        <Agents />
      </Section>
      <Section tone="mid">
        <Systems />
        <Addresses />
        <Plans />
        <Questions />
      </Section>
      <Close />
    </>
  );
}
