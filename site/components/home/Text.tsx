import type { ReactNode } from "react";

/*
 * Reading paragraphs sit a little under the `max-w-measure` token (57ch): Sofia Sans' zero is wider
 * than its average letter, and the lead's larger size and the body's 17 px both measured 76 to 79
 * characters a line at 1440 with the token itself, over the contract's 65 to 72.
 */

/** The paragraph that answers the section's heading: a little larger, in the first ink. */
export function Lead({ children }: { children: ReactNode }) {
  return <p className="m-0 mt-5 max-w-[52ch] text-[1.1875rem] leading-[1.5]">{children}</p>;
}

/** Any paragraph after the answer, in the second ink. */
export function Body({ children, className = "" }: { children: ReactNode; className?: string }) {
  return <p className={`m-0 mt-4 max-w-[52ch] text-ink-2 ${className}`}>{children}</p>;
}

/** A command, a path or an address inside a sentence. */
export function Code({ children }: { children: ReactNode }) {
  return (
    <code className="rounded-chart bg-[color-mix(in_srgb,var(--ink)_9%,transparent)] px-1 font-code text-[0.875em] text-ink whitespace-nowrap">
      {children}
    </code>
  );
}

/** A sentence with each time such as "1 d 21 h" or "4 h 30 min" kept on one line. */
export function Times({ children }: { children: string }) {
  return (
    <>
      {children.split(/(\d+ (?:d|h|min)(?: \d+ (?:h|min))?)/).map((part, index) =>
        index % 2 === 1 ? (
          <span key={index} className="whitespace-nowrap">
            {part}
          </span>
        ) : (
          part
        ),
      )}
    </>
  );
}
