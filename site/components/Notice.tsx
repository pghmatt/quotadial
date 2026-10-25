import type { ReactNode } from "react";

/** A bordered block for steps that carry a caution. A border all round; never a side rail. */
export function Notice({ title, children }: { title: string; children: ReactNode }) {
  return (
    <aside className="rounded-chart border border-rule p-4 md:p-6" aria-label={title}>
      <p className="m-0 mb-2 font-display text-[1.25rem] font-[700]">{title}</p>
      {children}
    </aside>
  );
}
