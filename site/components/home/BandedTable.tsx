import type { ReactNode } from "react";

interface Row {
  key: string;
  /** What the row belongs to. Rows of one group share a band, as in TideTable; a row with no group is a band of its own. */
  group?: string;
  cells: ReactNode[];
}

/**
 * A table of words rather than predictions (TideTable's figures sit right; these sit left), banded
 * by what the rows belong to and never striped by position. Under 768 px each row is set as a
 * short block that names its columns, so nothing needs sideways scrolling.
 */
export function BandedTable({
  caption,
  columns,
  rows,
  wholeFirst = false,
  className = "",
}: {
  /** Says what the table holds, for screen readers. */
  caption: string;
  columns: string[];
  rows: Row[];
  /** Keeps the first column's words on one line, such as a tool's name. */
  wholeFirst?: boolean;
  className?: string;
}) {
  const bands: Row[][] = [];
  for (const row of rows) {
    const last = bands[bands.length - 1];
    if (last && row.group !== undefined && last[0].group === row.group) {
      last.push(row);
    } else {
      bands.push([row]);
    }
  }
  return (
    <table className={`block w-full border-collapse font-body text-[1rem] leading-[1.45] md:table ${className}`}>
      <caption className="sr-only">{caption}</caption>
      <thead className="hidden md:table-header-group">
        <tr>
          {columns.map((column) => (
            <th key={column} scope="col" className="px-3 pb-2 text-left font-table text-[0.875rem] font-[600] text-ink-2">
              {column}
            </th>
          ))}
        </tr>
      </thead>
      {bands.map((band, number) => (
        <tbody
          key={band[0].key}
          className={`block md:table-row-group ${number % 2 === 1 ? "bg-[color-mix(in_srgb,var(--ink)_5%,transparent)]" : ""}`}
        >
          {band.map((row) => (
            <tr key={row.key} className="block py-3 md:table-row md:py-0">
              {row.cells.map((cell, index) => {
                const Tag = index === 0 ? "th" : "td";
                const empty = cell === null || cell === "";
                return (
                  <Tag
                    key={index}
                    scope={index === 0 ? "row" : undefined}
                    data-label={columns[index]}
                    aria-hidden={empty ? true : undefined}
                    className={`block px-3 py-1 text-left align-top text-pretty md:table-cell md:py-3 ${empty ? "max-md:hidden" : ""} ${
                      index === 0
                        ? `font-table text-[1.0625rem] font-[650] ${wholeFirst ? "md:whitespace-nowrap" : ""}`
                        : "before:mb-0.5 before:block before:font-table before:text-[0.875rem] before:font-[600] before:text-ink-2 before:content-[attr(data-label)] md:before:hidden"
                    }`}
                  >
                    {cell}
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
