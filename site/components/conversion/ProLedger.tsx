import { PRO_ROWS, type ProRow } from "./pro-rows.ts";

/** Rows that belong together, in the order the app lists them. */
function bands(): ProRow[][] {
  const out: ProRow[][] = [];
  for (const row of PRO_ROWS) {
    const last = out[out.length - 1];
    if (last && last[0].group === row.group) {
      last.push(row);
    } else {
      out.push([row]);
    }
  }
  return out;
}

/**
 * What QuotaDial Pro adds, a row for each feature in the app's words, with what a Free copy has in
 * its place and what the feature changes. Rows that belong together (what you see, warnings, the
 * week's cost, agents) share a band; the bands alternate, and no row is striped on its own.
 */
export function ProLedger() {
  return (
    <dl className="m-0">
      {bands().map((band, number) => (
        <div
          key={band[0].feature}
          className={`grid grid-cols-[minmax(0,1fr)] gap-x-6 gap-y-1 px-3 py-3 md:grid-cols-[minmax(0,1fr)_minmax(0,1.15fr)] md:gap-y-5 ${number % 2 === 1 ? "bg-[color-mix(in_srgb,var(--ink)_5%,transparent)]" : ""}`}
        >
          {band.map((row) => (
            <Row key={row.feature} row={row} />
          ))}
        </div>
      ))}
    </dl>
  );
}

function Row({ row }: { row: ProRow }) {
  return (
    <>
      <dt className="font-display text-[1.1875rem] leading-[1.2] font-[700] max-md:not-first:mt-4">
        {row.feature}
        <span className="mt-1 block font-table text-[0.9375rem] leading-[1.35] font-[400] text-ink-2">
          On a Free copy: {row.free.charAt(0).toLowerCase()}
          {row.free.slice(1)}
        </span>
      </dt>
      <dd className="m-0 leading-[1.45]">{row.changes}</dd>
    </>
  );
}
