// @ts-check
// The marks a copy's History view carries, as the app sends them (`marks` in
// src-tauri/src/history.rs, D-143), for the made-up copies the window check and the website's
// pictures answer with (window-fixtures.mjs, site-fixtures.mjs). The rule is written again here, in
// UTC, the copies' zone, and not taken from the app or the window: the window check holds the window
// to these marks, so they must not come from the code it checks. time-marks.test.mjs holds this rule
// to marks worked out by hand.

/** @typedef {import("../src/meter").Mark} Mark */

const DAY = 24 * 60 * 60 * 1000;

/** Each day's initial, by `getUTCDay`, Sunday first. */
const INITIALS = ["S", "M", "T", "W", "T", "F", "S"];

const MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

/**
 * The marks of a period of `days` charted from `start` to `now`, each a time in milliseconds, in
 * the days of UTC. Over 7 days or fewer: a tick (a mark with no word) at each midnight inside the
 * period, and each day's initial centred in the part of it the period holds, for a whole day and for
 * a part at either end at least half a day wide. Over a longer period: a tick at each Monday's
 * midnight inside the period, and the dates of the days its two ends are in, at 0 and at 1, or the
 * one date at 0 when it holds a single day. A place is in whole seconds, as the app's is, and the
 * marks come in order along the charts.
 *
 * @param {number} days
 * @param {number} start
 * @param {number} now
 * @returns {Mark[]}
 */
export function marksOf(days, start, now) {
  if (now <= start) {
    return [];
  }
  const span = Math.max(1, Math.trunc((now - start) / 1000));
  /** @param {number} at */
  const along = (at) => Math.min(1, Math.max(0, Math.trunc((at - start) / 1000) / span));
  const byDay = days <= 7;
  const first = Math.floor(start / DAY) * DAY;
  // The day of the period's last moment: a period that ends at midnight holds none of that day.
  const last = Math.floor((now - 1) / DAY) * DAY;
  /** @type {Mark[]} */
  const marks = [];
  for (let day = first; day <= last; day += DAY) {
    if (day > start && day < now && (byDay || new Date(day).getUTCDay() === 1)) {
      marks.push({ along: along(day), word: "" });
    }
    const [from, to] = [Math.max(day, start), Math.min(day + DAY, now)];
    if (byDay && to > from && 2 * (to - from) >= DAY) {
      marks.push({ along: along(from + (to - from) / 2), word: INITIALS[new Date(day).getUTCDay()] });
    }
  }
  if (!byDay) {
    /** @param {number} day */
    const date = (day) => `${new Date(day).getUTCDate()} ${MONTHS[new Date(day).getUTCMonth()]}`;
    marks.push({ along: 0, word: date(first) });
    if (last > first) {
      marks.push({ along: 1, word: date(last) });
    }
  }
  return marks.sort((a, b) => a.along - b.along);
}
