import { SITE } from "../lib/data/site.ts";
import { Dial } from "./Dial.tsx";

const COLUMNS = [
  {
    title: "QuotaDial",
    links: [
      { href: "/download", label: "Download" },
      { href: "/pro", label: "QuotaDial Pro" },
      { href: "/changelog", label: "Changelog" },
    ],
  },
  {
    title: "Help",
    links: [
      { href: "/support", label: "Support" },
      { href: "/privacy", label: "What it reads and sends" },
      { href: SITE.securityReport, label: "Report a security problem" },
    ],
  },
  {
    title: "Project",
    links: [
      { href: SITE.repository, label: "Source on GitHub" },
      { href: SITE.installGuide, label: "Install guide" },
      { href: SITE.cliGuide, label: "Command guide" },
    ],
  },
];

/** The deep: the mark again, the site's links, the licence, and what the examples are. */
export function Footer() {
  return (
    <footer className="bg-deep text-on-deep">
      <div className="mx-auto grid w-full max-w-[80rem] gap-10 px-4 py-14 md:grid-cols-[1.4fr_1fr_1fr_1fr] md:px-6">
        <div>
          <a href="/" className="flex items-center gap-2 no-underline" aria-label="QuotaDial, home">
            <Dial size={30} />
            <span className="font-display text-[1.375rem] leading-none font-[750]">QuotaDial</span>
          </a>
          <p className="mt-4 mb-0 max-w-[34ch] text-on-deep-2">
            QuotaDial is free software under the MIT licence, made for people who work in Claude Code
            and the Codex CLI.
          </p>
        </div>
        {COLUMNS.map((column) => (
          <nav key={column.title} aria-label={column.title}>
            <p className="m-0 mb-3 font-display text-[1.125rem] font-[700]">{column.title}</p>
            <ul className="m-0 grid list-none gap-2 p-0">
              {column.links.map((link) => (
                <li key={link.href}>
                  <a href={link.href} className="text-on-deep-2 no-underline hover:text-on-deep hover:underline">
                    {link.label}
                  </a>
                </li>
              ))}
            </ul>
          </nav>
        ))}
      </div>
      <div className="mx-auto w-full max-w-[80rem] border-t border-[color-mix(in_srgb,var(--on-deep)_18%,transparent)] px-4 py-6 text-[0.9375rem] text-on-deep-2 md:px-6">
        <p className="m-0">
          The accounts and numbers in pictures of the app are examples. This site sets no cookies
          and runs no analytics.
        </p>
      </div>
    </footer>
  );
}
