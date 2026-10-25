import type { ReactNode } from "react";

/** Steps in the order they are done, numbered because the order is the information. */
export function Steps({ children }: { children: ReactNode }) {
  return (
    <ol className="m-0 grid max-w-measure list-decimal gap-3 pl-6 marker:font-table marker:font-[600] marker:text-ink-2">
      {children}
    </ol>
  );
}
