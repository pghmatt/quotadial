import { PRO_ROWS } from "./pro-rows.ts";

/**
 * What QuotaDial Pro adds, a row for each feature in the app's words, with what a Free copy has in
 * its place and what the feature changes. Rows are banded, never ruled.
 */
export function ProLedger() {
  return (
    <dl className="m-0">
      {PRO_ROWS.map((row) => (
        <div
          key={row.feature}
          className="grid gap-x-6 gap-y-1 px-3 py-3 md:grid-cols-[minmax(0,1fr)_minmax(0,1.15fr)] even:bg-[color-mix(in_srgb,var(--ink)_5%,transparent)]"
        >
          <dt className="font-display text-[1.1875rem] leading-[1.2] font-[700]">
            {row.feature}
            <span className="mt-1 block font-table text-[0.9375rem] leading-[1.35] font-[400] text-ink-2">
              On a Free copy: {row.free.charAt(0).toLowerCase()}
              {row.free.slice(1)}
            </span>
          </dt>
          <dd className="m-0 leading-[1.45]">{row.changes}</dd>
        </div>
      ))}
    </dl>
  );
}
