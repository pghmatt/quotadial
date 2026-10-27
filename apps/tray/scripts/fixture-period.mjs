// @ts-check
// The heatmap's rows a copy's History view carries, as the app sends them (`held_days`, `heatmap`
// and `said` in src-tauri/src/history.rs, D-144), for the made-up copies the window check and the
// website's pictures answer with (window-fixtures.mjs, site-fixtures.mjs). The rule is written again
// here, in UTC, the copies' zone, and not taken from the app or the window: the window check holds
// the window to these rows, so they must not come from the code it checks. heatmap-rows.test.mjs
// holds this rule to rows worked out by hand, as time-marks.test.mjs holds fixture-marks.mjs.

const HOUR = 60 * 60 * 1000;
const DAY = 24 * HOUR;

/**
 * The calendar days of UTC the period from `start` to `now`, each a time in milliseconds, holds any
 * part of, oldest first, each as `{ day, hours }`: the day's midnight, and for each of its 24 hours
 * whether the period holds any part of it. The period holds none of `now` itself, so a `now` at
 * midnight holds none of its day and a `now` on the hour none of that hour; the hour the period
 * starts in is held from the start.
 *
 * @param {number} start
 * @param {number} now
 * @returns {{ day: number, hours: boolean[] }[]}
 */
export function rowsOf(start, now) {
  if (now <= start) {
    return [];
  }
  const first = Math.floor(start / DAY) * DAY;
  const last = Math.floor((now - 1) / DAY) * DAY;
  const rows = [];
  for (let day = first; day <= last; day += DAY) {
    rows.push({ day, hours: Array.from({ length: 24 }, (_, hour) => day + hour * HOUR < now && day + (hour + 1) * HOUR > start) });
  }
  return rows;
}

/**
 * The hour a heatmap's first row starts from, as its words name it, or null for a row held from
 * its day's first hour: the first hour `rows`' first row holds, when that is not its first.
 *
 * @param {{ hours: boolean[] }[]} rows
 * @returns {number | null}
 */
export function fromOf(rows) {
  const first = rows[0]?.hours.indexOf(true) ?? -1;
  return first > 0 ? first : null;
}

/**
 * A row's words, as the app says them (`said`): the hours it was used, run together, and the
 * earliest of its busiest, or that it was not used. `label` names the row, with the hour it starts
 * from for a first row that starts part way through its day, such as "Sun 19 from 14:00". A null
 * cell, an hour the period holds none of, was not used in it.
 *
 * @param {string} label
 * @param {(number | null)[]} cells
 * @returns {string}
 */
export function saidOf(label, cells) {
  const used = cells.map((cell) => cell ?? 0);
  /** @type {[number, number][]} */
  const runs = [];
  used.forEach((share, hour) => {
    if (share <= 0) {
      return;
    }
    const last = runs.at(-1);
    if (last && last[1] === hour) {
      last[1] = hour + 1;
    } else {
      runs.push([hour, hour + 1]);
    }
  });
  if (runs.length === 0) {
    return `${label}: not used`;
  }
  const most = Math.max(...used);
  const spans = runs.map(([from, to]) => `${from}:00 to ${to}:00`);
  const last = spans.pop();
  const hours = spans.length === 0 ? last : `${spans.join(", ")} and ${last}`;
  return `${label}: used ${hours}, most at ${used.indexOf(most)}:00 (${Math.round(most)}%)`;
}
