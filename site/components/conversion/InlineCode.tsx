import type { ReactNode } from "react";

import { Breakable } from "../Breakable.tsx";

/** The field the code sits on, so its own field shows against it. */
type Ground = "chart" | "shallow" | "mid";

const FIELD: Record<Ground, string> = { chart: "bg-shallow", shallow: "bg-mid", mid: "bg-chart" };

/** A file name or a command inside a sentence, in the code face. */
export function InlineCode({ on = "chart", children }: { on?: Ground; children: ReactNode }) {
  return <code className={`rounded-chart px-1 font-code [overflow-wrap:break-word] text-[0.875em] ${FIELD[on]}`}>{typeof children === "string" ? <Breakable text={children} /> : children}</code>;
}
