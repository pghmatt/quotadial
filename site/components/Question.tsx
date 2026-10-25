import type { ReactNode } from "react";

/** A question and its answer, always open, so every answer can be read and found. */
export function Question({ question, children }: { question: string; children: ReactNode }) {
  return (
    <div className="break-inside-avoid">
      <h3 className="m-0 text-[1.375rem] leading-[1.15] font-[700]">{question}</h3>
      <div className="mt-2 max-w-measure text-ink-2 [&_p]:m-0 [&_p+p]:mt-3">{children}</div>
    </div>
  );
}
