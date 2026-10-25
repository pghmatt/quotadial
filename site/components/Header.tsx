import { GithubLogo } from "@phosphor-icons/react/dist/ssr";

import { SITE } from "../lib/data/site.ts";
import { Dial } from "./Dial.tsx";
import { NavLinks } from "./NavLinks.tsx";

/** The mark and name, linked home, then the site's links and GitHub; one line, 64 px tall. */
export function Header() {
  return (
    <header className="relative border-b border-rule bg-chart text-ink">
      <div className="mx-auto flex h-16 w-full max-w-[80rem] items-center justify-between gap-4 px-4 md:px-6">
        <a href="/" className="flex items-center gap-2 no-underline" aria-label="QuotaDial, home">
          <Dial size={30} />
          <span className="font-display text-[1.375rem] leading-none font-[750] tracking-[-0.01em]">QuotaDial</span>
        </a>
        <div className="flex items-center gap-4 md:gap-6">
          <NavLinks />
          <a href={SITE.repository} className="inline-flex items-center gap-2 no-underline hover:underline" aria-label="QuotaDial on GitHub">
            <GithubLogo size={24} aria-hidden />
            <span className="hidden font-display text-[1.125rem] font-[600] lg:inline">GitHub</span>
          </a>
        </div>
      </div>
    </header>
  );
}
