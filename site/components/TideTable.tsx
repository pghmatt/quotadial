import type { ReactNode } from "react";

export interface TideCell {
  text: ReactNode;
  /** A prediction that a window runs out before it resets: magenta, and said in words. */
  caution?: boolean;
}

/**
 * A table of predictions, like a tide table: tabular figures, a plain header, rows grouped by
 * shading rather than a rule under each.
 */
export function TideTable({
  caption,
  columns,
  rows,
  className = "",
}: {
  /** Says what the table holds; visible or for screen readers only. */
  caption: { text: string; visible?: boolean };
  columns: string[];
  rows: { key: string; cells: (TideCell | string)[] }[];
  className?: string;
}) {
  return (
    <table className={`w-full border-collapse font-table text-[1rem] leading-[1.35] ${className}`}>
      <caption className={caption.visible ? "mb-2 text-left font-display text-[1.125rem] font-[700]" : "sr-only"}>
        {caption.text}
      </caption>
      <thead>
        <tr>
          {columns.map((column, index) => (
            <th
              key={column}
              scope="col"
              className={`px-3 pb-2 text-[0.875rem] font-[600] text-ink-2 ${index === 0 ? "text-left" : "text-right"}`}
            >
              {column}
            </th>
          ))}
        </tr>
      </thead>
      <tbody>
        {rows.map((row) => (
          <tr key={row.key} className="even:bg-[color-mix(in_srgb,var(--ink)_5%,transparent)]">
            {row.cells.map((cell, index) => {
              const value = typeof cell === "string" ? { text: cell } : cell;
              const Tag = index === 0 ? "th" : "td";
              return (
                <Tag
                  key={index}
                  scope={index === 0 ? "row" : undefined}
                  className={`px-3 py-2 align-top ${index === 0 ? "text-left font-[600]" : "text-right"} ${value.caution ? "text-magenta font-[650]" : ""}`}
                >
                  {value.text}
                </Tag>
              );
            })}
          </tr>
        ))}
      </tbody>
    </table>
  );
}
