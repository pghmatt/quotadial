import type { Metadata } from "next";

import { Section } from "../components/Section.tsx";
import { PageIntro } from "../components/reference/PageIntro.tsx";

export const metadata: Metadata = {
  title: { absolute: "Page not found | QuotaDial" },
};

const WAYS = [
  { href: "/", name: "The home page", what: "Every Claude and Codex limit and its reset, in the menu bar or tray." },
  { href: "/download", name: "Download", what: "QuotaDial for macOS, Windows and Linux." },
  { href: "/support", name: "Support", what: "Write to us, open an issue or report a problem in private." },
] as const;

/** Any address the site does not have. */
export default function NotFound() {
  return (
    <Section>
      <div className="grid gap-14 lg:grid-cols-12 lg:gap-x-12">
        <div className="lg:col-span-6">
          <PageIntro
            title="This page is not on the chart"
            answer="The address may be mistyped, or the page may have moved. Start from one of these."
          />
        </div>
        <nav aria-label="Ways on" className="lg:col-span-5 lg:col-start-8">
          <ul className="m-0 grid list-none gap-8 p-0">
            {WAYS.map((way) => (
              <li key={way.href}>
                <a
                  href={way.href}
                  className="font-display text-[1.75rem] leading-[1.1] font-[700] underline decoration-2 underline-offset-[0.2em]"
                >
                  {way.name}
                </a>
                <p className="mt-1 mb-0 max-w-[40ch] text-ink-2">{way.what}</p>
              </li>
            ))}
          </ul>
        </nav>
      </div>
    </Section>
  );
}
