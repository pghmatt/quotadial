import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "preact/hooks";

import { AccountHeading } from "./account";
import { usePanelFocus } from "./focus";
import type { AccountHistory, Heatmap, HistoryView, Mark, WindowChart } from "./meter";
import { ACCOUNT_HEADER, heatmapWords, lookOf, shareImage } from "./share";

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
          marks={view.marks}
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

/**
 * One account's history, as a card headed as the popover heads the account's card (D-132). Every
 * chart has the view's ticks, and the last has the marks' words under it, once for the card, since
 * every chart in it spans the same period (D-143).
 */
function AccountCharts({
  account,
  index,
  marks,
  onShare,
}: {
  account: AccountHistory;
  index: number;
  marks: Mark[];
  onShare: (() => void) | null;
}) {
  const heading = `history-account-${index}`;
  const last = account.windows.length - 1;
  return (
    <section class="account-history card" aria-labelledby={heading}>
      <AccountHeading who={account} id={heading} level={3} />
      {account.windows.map((chart, at) => (
        <Chart key={chart.label} chart={chart} marks={marks} worded={at === last} />
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
 * it is the label, and the words around the shares, that wrap when there is little room. A tick
 * hangs from the baseline at each mark with no word, where a day, or over 30 days a week, begins,
 * inside the figure the screen reader hears by its summary; and under a card's last chart (`worded`)
 * are the marks' words (D-143).
 */
function Chart({ chart, marks, worded }: { chart: WindowChart; marks: Mark[]; worded: boolean }) {
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
        {marks.map(
          (mark) =>
            mark.word === "" && (
              // From the baseline down the chart's own height, of which the stylesheet draws the
              // tick's length.
              <line
                class="tick"
                key={mark.along}
                x1={mark.along * 100}
                y1="100"
                x2={mark.along * 100}
                y2="200"
              />
            ),
        )}
        <line class="limit" x1="0" y1="10" x2="100" y2="10" />
        <polyline class="used" points={line} />
      </svg>
      {worded && <MarkWords marks={marks} />}
    </figure>
  );
}

/**
 * The marks' words under a card's last chart (D-143), each at its place along the period: over 7
 * days each day's initial, centred in the day, and over 30 the dates of the period's two ends, each
 * flush with its end of the chart. Hidden from a screen reader, which hears the period in each
 * chart's summary and each day in the heatmap's lines.
 */
function MarkWords({ marks }: { marks: Mark[] }) {
  const words = marks.filter((mark) => mark.word !== "");
  if (words.length === 0) {
    return null;
  }
  return (
    <div class="marks" aria-hidden="true">
      {words.map((mark) => (
        <span
          key={mark.along}
          class={mark.along <= 0 ? "start" : mark.along >= 1 ? "end" : undefined}
          style={{ "--along": mark.along }}
        >
          {mark.word}
        </span>
      ))}
    </div>
  );
}

/**
 * When the account was used: a row per day the period holds, a column per hour, darker for more,
 * and no square for an hour the period holds none of, before its start in the first row and after
 * now in the last, each square kept in its hour's column (D-144).
 */
function HeatmapGrid({ heatmap }: { heatmap: Heatmap }) {
  // One sentence, seen in the caption and heard as the figure's name, from the one place the
  // picture to share takes it too.
  const said = heatmapWords(heatmap);
  if (heatmap.most <= 0) {
    return <p class="detail">{said}</p>;
  }
  return (
    <figure class="heatmap" aria-label={said}>
      <figcaption class="detail">{said}</figcaption>
      <ul class="visually-hidden">
        {heatmap.said.map((line) => (
          <li key={line}>{line}</li>
        ))}
      </ul>
      {heatmap.cells.map((hours, day) => (
        <div class="heat-row" key={heatmap.days[day]} aria-hidden="true">
          <span class="heat-day">{heatmap.days[day]}</span>
          {hours.map((used, hour) =>
            used === null ? null : (
              <span
                class="heat-cell"
                key={hour}
                style={{
                  gridColumn: String(hour + 2),
                  opacity: used > 0 ? 0.15 + (0.85 * used) / heatmap.most : 0.06,
                }}
                title={`${heatmap.days[day]}, ${hour}:00: ${Math.round(used)}% used`}
              />
            ),
          )}
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
