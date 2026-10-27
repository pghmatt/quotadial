import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "preact/hooks";

import { AccountHeading } from "./account";
import { usePanelFocus } from "./focus";
import type { AccountHistory, Heatmap, HistoryView, WindowChart } from "./meter";
import { ACCOUNT_HEADER, lookOf, shareImage } from "./share";

/** How far back the view can go, in days. */
const PERIODS = [7, 30];

/**
 * The history view: each account's windows charted over the last 7 or 30 days, when it was used
 * by hour of the day, an export of every reading as CSV, and on QuotaDial Pro a picture of each
 * account's history to share, saved in the Downloads folder as the CSV is (D-115).
 */
export function HistoryPanel() {
  const [days, setDays] = useState(PERIODS[0]);
  const [view, setView] = useState<HistoryView | null>(null);
  const [problem, setProblem] = useState<string | null>(null);
  const [saved, setSaved] = useState<string | null>(null);
  // A picture being drawn and saved; another press meanwhile is the same press.
  const sharing = useRef(false);
  const panel = usePanelFocus();

  useEffect(() => {
    // An answer for a period no longer chosen is dropped, however late it comes.
    let chosen = true;
    invoke<HistoryView>("history", { days })
      .then((next) => {
        if (chosen) {
          setView(next);
          setProblem(null);
        }
      })
      .catch((error: unknown) => {
        if (chosen) {
          setProblem(String(error));
        }
      });
    return () => {
      chosen = false;
    };
  }, [days]);

  /** Says where a file went, or what went wrong. */
  const report = (saving: Promise<string>) => {
    saving
      .then((where) => {
        setSaved(where);
        setProblem(null);
      })
      .catch((error: unknown) => setProblem(String(error)));
  };

  const exportReadings = () => report(invoke<string>("export_history", { days }));

  // The picture is drawn here from the view's own numbers, in the window's theme, and the app
  // saves it; the account goes by its key, and the app chooses the file's name.
  const share = (account: AccountHistory) => {
    if (!view || sharing.current) {
      return;
    }
    sharing.current = true;
    report(
      shareImage(view, account, lookOf(document.documentElement))
        .then((png) =>
          invoke<string>("save_share_image", png, {
            headers: { [ACCOUNT_HEADER]: encodeURIComponent(account.key) },
          }),
        )
        .finally(() => {
          sharing.current = false;
        }),
    );
  };

  const shown = view !== null && view.accounts.length > 0;
  return (
    <section class="history" aria-label="History" tabIndex={-1} ref={panel}>
      <h2>History</h2>
      <div class="period" role="group" aria-label="How far back">
        {PERIODS.map((period) => (
          <button
            type="button"
            key={period}
            aria-pressed={period === days}
            onClick={() => setDays(period)}
          >
            {period} days
          </button>
        ))}
      </div>
      {shown && <p class="detail">{view.span}</p>}
      {view && view.accounts.length === 0 && (
        <p class="detail">No readings in the last {days} days yet.</p>
      )}
      {view?.accounts.map((account, index) => (
        <AccountCharts
          key={account.key}
          account={account}
          index={index}
          onShare={view.share_left_out === null ? () => share(account) : null}
        />
      ))}
      {/* On QuotaDial Free, one line where the buttons would be, as D-100 says elsewhere. */}
      {shown && view.share_left_out !== null && <p class="left-out">{view.share_left_out}</p>}
      {/* Always in the page, so a screen reader hears each new line in it. */}
      <p class="detail" role="status">
        {saved ?? ""}
      </p>
      {problem && (
        <p class="note" role="alert">
          {problem}
        </p>
      )}
      <div class="history-actions">
        <button type="button" onClick={exportReadings}>
          Export CSV
        </button>
      </div>
    </section>
  );
}

/** One account's history, as a card headed as the popover heads the account's card (D-132). */
function AccountCharts({
  account,
  index,
  onShare,
}: {
  account: AccountHistory;
  index: number;
  onShare: (() => void) | null;
}) {
  const heading = `history-account-${index}`;
  return (
    <section class="account-history card" aria-labelledby={heading}>
      <AccountHeading who={account} id={heading} level={3} />
      {account.windows.map((chart) => (
        <Chart key={chart.label} chart={chart} />
      ))}
      <HeatmapGrid heatmap={account.heatmap} />
      {onShare && (
        <button
          type="button"
          class="share"
          aria-label={`Share image of ${account.label}`}
          onClick={onShare}
        >
          Share image
        </button>
      )}
    </section>
  );
}

/**
 * A window's share used over the period, drawn with no plate under it: a hairline at 0%, a faint
 * line at 100% to show the scale, and a dashed line at 90%. Its caption names the window and says
 * the latest and the highest share, each of them one token that never breaks (`nobreak`), so that
 * it is the label, and the words around the shares, that wrap when there is little room.
 */
function Chart({ chart }: { chart: WindowChart }) {
  const line = chart.points
    .map(([along, used]) => `${(along * 100).toFixed(2)},${(100 - used).toFixed(2)}`)
    .join(" ");
  return (
    <figure class="chart">
      <figcaption>
        <span class="label">{chart.label}</span>
        <span class="detail">
          latest <span class="nobreak">{Math.floor(chart.latest)}%</span>, highest{" "}
          <span class="nobreak">{Math.floor(chart.highest)}%</span>
        </span>
      </figcaption>
      <svg viewBox="0 0 100 100" preserveAspectRatio="none" role="img" aria-label={chart.summary}>
        <line class="top" x1="0" y1="0" x2="100" y2="0" />
        <line class="base" x1="0" y1="100" x2="100" y2="100" />
        <line class="limit" x1="0" y1="10" x2="100" y2="10" />
        <polyline class="used" points={line} />
      </svg>
    </figure>
  );
}

/** When the account was used: a row per day, a column per hour, darker for more. */
function HeatmapGrid({ heatmap }: { heatmap: Heatmap }) {
  if (heatmap.most <= 0) {
    return <p class="detail">No use in {heatmap.window.toLowerCase()} readings in this period.</p>;
  }
  return (
    <figure class="heatmap" aria-label={`When it was used, by hour, from ${heatmap.window} readings`}>
      <figcaption class="detail">When it was used, by hour ({heatmap.window.toLowerCase()})</figcaption>
      <ul class="visually-hidden">
        {heatmap.said.map((line) => (
          <li key={line}>{line}</li>
        ))}
      </ul>
      {heatmap.cells.map((hours, day) => (
        <div class="heat-row" key={heatmap.days[day]} aria-hidden="true">
          <span class="heat-day">{heatmap.days[day]}</span>
          {hours.map((used, hour) => (
            <span
              class="heat-cell"
              key={hour}
              style={{ opacity: used > 0 ? 0.15 + (0.85 * used) / heatmap.most : 0.06 }}
              title={`${heatmap.days[day]}, ${hour}:00: ${Math.round(used)}% used`}
            />
          ))}
        </div>
      ))}
      <div class="heat-row heat-hours" aria-hidden="true">
        <span class="heat-day" />
        {[0, 6, 12, 18].map((hour) => (
          <span class="heat-hour" key={hour} style={{ gridColumn: `${hour + 2} / span 6` }}>
            {hour}
          </span>
        ))}
      </div>
    </figure>
  );
}
