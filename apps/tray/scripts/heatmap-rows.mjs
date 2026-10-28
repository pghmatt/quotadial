// Holds History's heatmap to the charts' period (the manager's decision on #73, 2026-10-27, 19:35 UTC,
// D-144): a row for each calendar day the period holds any part of, from the day the charts start in,
// and no square for an hour the period holds none of, before its start in the first row or after now
// in the last, the other squares kept under their hours. The window check (check-window.mjs) finds,
// in the page, each card's heatmap rows with their squares and the columns of their grid, the hours
// written under them, the first line of the heatmap's words and the marks' words; what it makes of
// them, against the rows the copy has, is here, so that tests which need no browser hold it.

/** How far a square may be from its hour's column, or from the squares above and below it, in CSS
 * pixels: a pixel. */
export const ALIGNED = 1;

/**
 * The code to run in the page: History's account cards, in order, each as `{ what, rows, hours,
 * said, words }`: the card's title; each row of its heatmap but the hours' row, its day's label, the
 * left and right of each column of its grid, from the grid's own tracks, its first the day's label
 * and then an hour's each, and each square in it, with its box and whether it is drawn (shown, with
 * room, and a colour that is not clear); the hours written under the rows, each with its text and
 * the left of its box; the first line of the heatmap's words, which a screen reader reads; and the
 * words of the marks' row under the card's last chart.
 */
export const HEAT_ROWS = `(() => {
  const drawn = (element) => {
    const style = getComputedStyle(element);
    const { width, height } = element.getBoundingClientRect();
    const alpha = (style.backgroundColor.match(/[\\d.]+/g) ?? []).map(Number)[3] ?? 1;
    return element.getClientRects().length > 0 && style.visibility !== 'hidden' && Number(style.opacity) > 0 && width > 0 && height > 0 && alpha > 0 && style.backgroundColor !== 'transparent';
  };
  return [...document.querySelectorAll('section.history .account-history')].map((card) => {
    const heatmap = card.querySelector('figure.heatmap');
    const rows = heatmap ? [...heatmap.querySelectorAll('.heat-row:not(.heat-hours)')] : [];
    return {
      what: (card.querySelector('.card-head .title')?.textContent ?? 'a card').trim(),
      rows: rows.map((row) => {
        const style = getComputedStyle(row);
        const gap = parseFloat(style.columnGap) || 0;
        const tracks = style.gridTemplateColumns.split(' ').map(parseFloat).filter((width) => !Number.isNaN(width));
        let left = row.getBoundingClientRect().left + (parseFloat(style.paddingLeft) || 0) + (parseFloat(style.borderLeftWidth) || 0);
        const columns = tracks.map((width) => {
          const column = { left, right: left + width };
          left += width + gap;
          return column;
        });
        return {
          day: row.querySelector('.heat-day')?.textContent ?? '',
          columns,
          squares: [...row.querySelectorAll('.heat-cell')].map((square) => {
            const { left, right, top, bottom } = square.getBoundingClientRect();
            return { left, right, top, bottom, drawn: drawn(square) };
          }),
        };
      }),
      hours: heatmap ? [...heatmap.querySelectorAll('.heat-hours .heat-hour')].map((hour) => ({ text: hour.textContent.trim(), left: hour.getBoundingClientRect().left })) : [],
      said: heatmap?.querySelector('ul.visually-hidden li')?.textContent ?? null,
      words: [...card.querySelectorAll('.marks span')].map((word) => word.textContent),
    };
  });
})()`;

const DAY = 24 * 60 * 60 * 1000;

/**
 * How many calendar days of UTC, the copies' zone, the period from `start` to `now`, each a time in
 * milliseconds, touches: from the day it starts in to the day of its last moment, so a `now` at
 * midnight touches none of its day. The check's own count, not the copies' rows or the app's.
 */
export function daysTouched({ start, now }) {
  if (now <= start) {
    return 0;
  }
  return Math.floor((now - 1) / DAY) - Math.floor(start / DAY) + 1;
}

/** The hours of a row of `cells` that the period holds, those with a cell. */
const heldHours = (cells) => cells.flatMap((cell, hour) => (cell === null ? [] : [hour]));

/** A day's number, from a row's label, "Sun 19", or a date, "19 Apr". */
const dayNumber = (text) => Number(text.match(/\d+/)?.[0]);

/**
 * Verdicts on History's heatmaps as `HEAT_ROWS` found them, in a view of `days` days of `copy`, whose
 * accounts and period, `copy.periods[days]`, they are held to: false where each holds, or what is
 * wrong, naming the card.
 *
 * - **The rows** are the copy's, labelled as it labels them, in order, so a check that finds none
 *   cannot pass, and as many as the calendar days the period touches.
 * - **The squares:** in each row a square is drawn for each hour the copy's row holds and none for
 *   the others, before the period's start and after now; each square sits in its hour's column of
 *   the grid, to a pixel, and under the squares of that hour in every other row, so the squares not
 *   drawn keep their place; and the hours written under the rows stand at their columns.
 * - **The words:** the first line the heatmap says names its row's day and, when the row starts part
 *   way through it, the hour it starts from, its first square's.
 * - **The marks name the rows' days:** over 30 days, the left date is the first row's day and the
 *   right the last's; over 7, the initials are the rows' days', in order, from the first row or the
 *   next, to the last or the one before, since a row held too little of its day has none.
 */
export function rowVerdicts(found, copy, days) {
  const view = copy.history[days];
  const accounts = view?.accounts ?? [];
  const verdicts = [found.length !== accounts.length && `History shows ${found.length} account cards, not the ${accounts.length} the copy has`];
  const period = copy.periods?.[days];
  verdicts.push(!period && `the copy says no period for its ${days} days, to count the heatmap's rows against`);
  for (const [index, card] of found.entries()) {
    const account = accounts[index];
    if (!account) {
      break;
    }
    const { what } = card;
    const heatmap = account.heatmap;
    if (heatmap.most <= 0) {
      verdicts.push(card.rows.length > 0 && `${what} draws ${card.rows.length} heatmap rows for a heatmap with no use, which has none`);
      continue;
    }
    const labels = card.rows.map((row) => row.day);
    verdicts.push(
      labels.join("|") !== heatmap.days.join("|") &&
        `${what}'s heatmap rows are ${JSON.stringify(labels)}, where its copy has ${JSON.stringify(heatmap.days)}`,
    );
    if (period) {
      const touched = daysTouched(period);
      verdicts.push(
        card.rows.length !== touched &&
          `${what}'s heatmap has ${card.rows.length} rows, where its period of ${days} days touches ${touched} calendar days`,
      );
    }
    // Each hour's squares, across the rows, to hold them under one another.
    const columns = new Map();
    for (const [at, row] of card.rows.entries()) {
      const cells = heatmap.cells[at];
      if (!cells) {
        continue;
      }
      const name = `${what}'s row "${row.day}"`;
      const hourColumns = row.columns.slice(1);
      if (hourColumns.length !== 24) {
        verdicts.push(`${name} has ${hourColumns.length} hour columns in its grid, not 24`);
        continue;
      }
      const shown = new Map();
      for (const square of row.squares.filter((one) => one.drawn)) {
        const middle = (square.left + square.right) / 2;
        const hour = hourColumns.findIndex((column) => middle >= column.left - ALIGNED && middle <= column.right + ALIGNED);
        if (hour < 0) {
          verdicts.push(`${name} draws a square at ${square.left.toFixed(1)}, under no hour's column`);
          continue;
        }
        const column = hourColumns[hour];
        verdicts.push(
          (Math.abs(square.left - column.left) > ALIGNED || Math.abs(square.right - column.right) > ALIGNED) &&
            `${name}'s square for ${hour}:00 runs from ${square.left.toFixed(1)} to ${square.right.toFixed(1)}, not in its column, ${column.left.toFixed(1)} to ${column.right.toFixed(1)}`,
        );
        shown.set(hour, (shown.get(hour) ?? 0) + 1);
        columns.set(hour, [...(columns.get(hour) ?? []), { name, left: square.left }]);
      }
      const held = heldHours(cells);
      const outside = [...shown.keys()].filter((hour) => cells[hour] === null).sort((a, b) => a - b);
      const missing = held.filter((hour) => !shown.has(hour));
      const twice = [...shown.entries()].filter(([, count]) => count > 1).map(([hour]) => hour);
      verdicts.push(
        outside.length > 0 && `${name} draws a square for ${outside.map((hour) => `${hour}:00`).join(", ")}, outside the period, where its copy has no cell`,
        missing.length > 0 && `${name} draws no square for ${missing.map((hour) => `${hour}:00`).join(", ")}, which the period holds`,
        twice.length > 0 && `${name} draws more than one square for ${twice.map((hour) => `${hour}:00`).join(", ")}`,
      );
    }
    for (const [hour, squares] of columns) {
      const lefts = squares.map((square) => square.left);
      const [least, most] = [Math.min(...lefts), Math.max(...lefts)];
      verdicts.push(
        most - least > ALIGNED &&
          `${what}'s squares for ${hour}:00 are not under one another: from ${least.toFixed(1)} in ${squares[lefts.indexOf(least)].name} to ${most.toFixed(1)} in ${squares[lefts.indexOf(most)].name}`,
      );
    }
    const reference = card.rows.find((row) => row.columns.length === 25);
    for (const { text, left } of card.hours) {
      const column = reference?.columns[Number(text) + 1];
      verdicts.push(
        (!column || Math.abs(left - column.left) > ALIGNED) &&
          `${what}'s hour "${text}" is written at ${left.toFixed(1)}, not at its column, ${column ? column.left.toFixed(1) : "which there is none of"}`,
      );
    }
    // The first line of the words: the row's day, and the hour it starts from when it starts late.
    const first = heldHours(heatmap.cells[0] ?? [])[0];
    const opening = first > 0 ? `${heatmap.days[0]} from ${first}:00: ` : `${heatmap.days[0]}: `;
    verdicts.push(
      !card.said?.startsWith(opening) &&
        `${what}'s heatmap says ${JSON.stringify(card.said)} first, not a line that begins ${JSON.stringify(opening)}`,
    );
    verdicts.push(markDaysVerdict(card.words, heatmap.days, days, what));
  }
  return verdicts;
}

/**
 * Whether the marks' words, `words`, name the days of the heatmap's rows, `rows`' labels, in a view of
 * `days`: false, or what is wrong. Over 30 days, the left date is the first row's day and the right
 * date the last's; over 7, the initials are those of the rows' days, in order, from the first row or
 * the next to the last or the one before.
 */
export function markDaysVerdict(words, rows, days, what) {
  if (words.length === 0 || rows.length === 0) {
    return `${what} has ${words.length} marks' words and ${rows.length} heatmap rows to hold to each other`;
  }
  if (days > 7) {
    const ends = [words[0], words.at(-1)].map(dayNumber);
    const wanted = [rows[0], rows.at(-1)].map(dayNumber);
    return (
      ends.join() !== wanted.join() &&
      `${what}'s dates under its charts, ${JSON.stringify(words)}, are not its heatmap's first and last days, ${JSON.stringify([rows[0], rows.at(-1)])}`
    );
  }
  const initials = rows.map((row) => row[0]);
  const fits = [0, 1].some((from) => [initials.length, initials.length - 1].some((to) => initials.slice(from, to).join(" ") === words.join(" ")));
  return (
    !fits &&
    `${what}'s initials under its charts, ${JSON.stringify(words.join(" "))}, are not its heatmap's days', ${JSON.stringify(initials.join(" "))}, from the first row or the next`
  );
}
