import type { ReactNode } from "react";

/** The paragraph that answers the section's heading: a little larger, in the first ink. */
export function Lead({ children }: { children: ReactNode }) {
  return <p className="m-0 mt-5 max-w-[62ch] text-[1.1875rem] leading-[1.5]">{children}</p>;
}

/** Any paragraph after the answer, in the second ink. */
export function Body({ children, className = "" }: { children: ReactNode; className?: string }) {
  return <p className={`m-0 mt-4 max-w-[62ch] text-ink-2 ${className}`}>{children}</p>;
}

/** A command, a path or an address inside a sentence. */
export function Code({ children }: { children: ReactNode }) {
  return (
    <code className="rounded-chart bg-[color-mix(in_srgb,var(--ink)_9%,transparent)] px-1 font-code text-[0.875em] text-ink break-words">
      {children}
    </code>
  );
}
