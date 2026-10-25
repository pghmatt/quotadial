import type { ReactNode } from "react";

export interface TideCell {
  text: ReactNode;
  /** A prediction that a window runs out before it resets: magenta, and said in words. */
  caution?: boolean;
}

export interface TideRow {
  key: string;
  /**
   * What the row belongs to, such as its system or its account. Rows of one group are read
   * together and share a band, as a tide table bands each port's times; a table whose rows name no
   * group has no bands.
   */
  group?: string;
  cells: (TideCell | string)[];
}

/** How a column sits: words from the left, figures and times from the right. */
export type TideAlign = "words" | "figures";

/**
 * A table of predictions, like a tide table: tabular figures, a plain header, and rows banded by
 * what they belong to, never striped by position or ruled under each.
 */
export function TideTable({
  caption,
  columns,
  rows,
  align = [],
  whole = [],
  className = "",
}: {
  /** Says what the table holds; visible or for screen readers only. */
  caption: { text: string; visible?: boolean };
  columns: string[];
  rows: TideRow[];
  /** Each column's alignment; a column left out holds words. */
  align?: TideAlign[];
  /** Columns whose cells never break across lines, such as times and file names. */
  whole?: boolean[];
  className?: string;
}) {
  const sides = columns.map((_, index) => (align[index] === "figures" ? "text-right" : "text-left"));
  const unbroken = columns.map((_, index) => (whole[index] ? "whitespace-nowrap" : ""));
  const bands: TideRow[][] = [];
  for (const row of rows) {
    const last = bands[bands.length - 1];
    if (last && row.group !== undefined && last[0].group === row.group) {
      last.push(row);
    } else {
      bands.push([row]);
    }
  }
  const banded = rows.some((row) => row.group !== undefined);
  return (
    <table className={`w-full border-collapse font-table text-[1rem] leading-[1.35] ${className}`}>
      <caption className={caption.visible ? "mb-2 text-left font-display text-[1.125rem] font-[700]" : "sr-only"}>
        {caption.text}
      </caption>
      <thead>
        <tr>
          {columns.map((column, index) => (
            <th key={column} scope="col" className={`px-3 pb-2 text-[0.875rem] font-[600] text-ink-2 ${sides[index]}`}>
              {column}
            </th>
          ))}
        </tr>
      </thead>
      {bands.map((band, number) => (
        <tbody
          key={band[0].key}
          className={banded && number % 2 === 1 ? "bg-[color-mix(in_srgb,var(--ink)_5%,transparent)]" : undefined}
        >
          {band.map((row) => (
            <tr key={row.key}>
              {row.cells.map((cell, index) => {
                const value = typeof cell === "string" ? { text: cell } : cell;
                const Tag = index === 0 ? "th" : "td";
                return (
                  <Tag
                    key={index}
                    scope={index === 0 ? "row" : undefined}
                    className={`px-3 py-2 align-top ${sides[index]} ${unbroken[index]} ${index === 0 ? "font-[600]" : ""} ${value.caution ? "font-[650] text-magenta" : ""}`}
                  >
                    {value.text}
                  </Tag>
                );
              })}
            </tr>
          ))}
        </tbody>
      ))}
    </table>
  );
}
