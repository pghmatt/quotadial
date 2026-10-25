import type { ReactNode } from "react";

/**
 * A table of words rather than predictions (TideTable's figures sit right; these sit left): rows
 * grouped by shading, a plain header, and, under 768 px, each row set as a short block that names
 * its columns, so nothing needs sideways scrolling.
 */
export function BandedTable({
  caption,
  columns,
  rows,
  className = "",
}: {
  /** Says what the table holds, for screen readers. */
  caption: string;
  columns: string[];
  rows: { key: string; cells: ReactNode[] }[];
  className?: string;
}) {
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
      <tbody className="block md:table-row-group">
        {rows.map((row) => (
          <tr key={row.key} className="block py-3 even:bg-[color-mix(in_srgb,var(--ink)_5%,transparent)] md:table-row md:py-0">
            {row.cells.map((cell, index) => {
              const Tag = index === 0 ? "th" : "td";
              return (
                <Tag
                  key={index}
                  scope={index === 0 ? "row" : undefined}
                  data-label={columns[index]}
                  className={`block px-3 py-1 text-left align-top text-pretty md:table-cell md:py-3 ${
                    index === 0
                      ? "font-table text-[1.0625rem] font-[650]"
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
    </table>
  );
}
