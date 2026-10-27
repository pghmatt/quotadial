// How the window check holds History's heatmap to the charts' period (heatmap-rows.mjs, D-144), and
// the rule the made-up copies' rows are written by (fixture-period.mjs): that rule against rows worked
// out by hand, as the app's own tests work them out; that every copy's rows follow it, one for each
// calendar day its period touches, the check counting those days itself; that the rows' verdicts pass
// a card drawn as it should be and fail one with a square outside the period at either end, as
// `main`'s window draws every hour, a square missing, one out of its column or not under the others,
// the rows not the copy's or not as many as the days, the first line of words not naming the hour its
// row starts from, the hours not over their columns, and marks that name other days than the rows'.
// The browser's part, the boxes themselves, is held by the window check itself.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { Script } from "node:vm";

import { fromOf, rowsOf, saidOf } from "./fixture-period.mjs";
import { ALIGNED, HEAT_ROWS, daysTouched, markDaysVerdict, rowVerdicts } from "./heatmap-rows.mjs";
import { site } from "./site-fixtures.mjs";
import { crowded, free, many, pro } from "./window-fixtures.mjs";

const HOUR = 60 * 60 * 1000;
const DAY = 24 * HOUR;

/** The rows of the `days` before `now`, an ISO time, as `[the day's date, its first held hour, its last]`. */
function rowsBefore(days, now) {
  const end = Date.parse(now);
  return rowsOf(end - days * DAY, end).map(({ day, hours }) => [new Date(day).toISOString().slice(0, 10), hours.indexOf(true), hours.lastIndexOf(true)]);
}

/** The hours of a row the period holds, each held one after another from its first to its last. */
const unbroken = (hours) => hours.slice(hours.indexOf(true), hours.lastIndexOf(true) + 1).every(Boolean);

test("over 7 days, a row for each calendar day the period touches, the first from the hour it starts in and the last to the hour now is in", () => {
  // To Tuesday 27 October 2026 at 14:25, from Tuesday 20 at 14:25: eight rows.
  assert.deepEqual(rowsBefore(7, "2026-10-27T14:25:00Z"), [
    ["2026-10-20", 14, 23], ["2026-10-21", 0, 23], ["2026-10-22", 0, 23], ["2026-10-23", 0, 23],
    ["2026-10-24", 0, 23], ["2026-10-25", 0, 23], ["2026-10-26", 0, 23], ["2026-10-27", 0, 14],
  ]);
  // To noon: the first from 12:00, the last to 11:00's hour, none of noon's.
  assert.deepEqual(rowsBefore(7, "2026-10-27T12:00:00Z").map(([, first, last]) => [first, last]), [[12, 23], [0, 23], [0, 23], [0, 23], [0, 23], [0, 23], [0, 23], [0, 11]]);
  // A minute after midnight: the first row's first hour is held from 00:01, and the last is that minute's hour.
  assert.deepEqual(rowsBefore(7, "2026-10-27T00:01:00Z").map(([, first, last]) => [first, last]), [[0, 23], [0, 23], [0, 23], [0, 23], [0, 23], [0, 23], [0, 23], [0, 0]]);
  // At midnight: seven whole days, none of the day that begins at now.
  assert.deepEqual(rowsBefore(7, "2026-10-28T00:00:00Z").map(([date, first, last]) => [date, first, last]), [
    ["2026-10-21", 0, 23], ["2026-10-22", 0, 23], ["2026-10-23", 0, 23], ["2026-10-24", 0, 23],
    ["2026-10-25", 0, 23], ["2026-10-26", 0, 23], ["2026-10-27", 0, 23],
  ]);
});

test("over 30 days, 31 rows when now is not a midnight and 30 when it is; a younger history from its first reading's hour; none for no period", () => {
  const month = rowsBefore(30, "2026-10-27T14:25:00Z");
  assert.equal(month.length, 31);
  assert.deepEqual([month[0], month[30]], [["2026-09-27", 14, 23], ["2026-10-27", 0, 14]]);
  assert.equal(rowsBefore(30, "2026-10-28T00:00:00Z").length, 30);
  // From a first reading at 09:10 to 11:00 the same day: one row, its hours 9 and 10.
  const start = Date.parse("2099-01-07T09:10:00Z");
  assert.deepEqual(rowsOf(start, start + 110 * 60 * 1000).map(({ hours }) => hours.flatMap((held, hour) => (held ? [hour] : []))), [[9, 10]]);
  assert.deepEqual(rowsOf(start, start), []);
  assert.deepEqual(rowsOf(start, start - HOUR), []);
  // Every row's held hours run on unbroken from its first to its last, in UTC.
  for (const now of ["2026-10-27T14:25:00Z", "2026-10-27T00:01:00Z", "2026-10-28T00:00:00Z"]) {
    for (const days of [7, 30]) {
      const end = Date.parse(now);
      assert.ok(rowsOf(end - days * DAY, end).every(({ hours }) => unbroken(hours)), `${days} days to ${now}`);
    }
  }
});

test("the first row's words name the hour it starts from, unless that is its first; a row's words say each run of its use and the earliest busiest hour", () => {
  const end = Date.parse("2026-10-27T14:25:00Z");
  assert.equal(fromOf(rowsOf(end - 7 * DAY, end)), 14);
  assert.equal(fromOf(rowsOf(Date.parse("2026-10-20T00:01:00Z"), Date.parse("2026-10-27T00:01:00Z"))), null);
  assert.equal(fromOf(rowsOf(Date.parse("2026-10-21T00:00:00Z"), Date.parse("2026-10-28T00:00:00Z"))), null);
  assert.equal(fromOf([]), null);
  const cells = Array.from({ length: 24 }, (_, hour) => (hour < 14 ? null : 0));
  assert.equal(saidOf("Sun 19 from 14:00", cells), "Sun 19 from 14:00: not used");
  cells[15] = 2;
  cells[16] = 8;
  assert.equal(saidOf("Sun 19 from 14:00", cells), "Sun 19 from 14:00: used 15:00 to 17:00, most at 16:00 (8%)");
  const day = Array.from({ length: 24 }, () => 0);
  for (const hour of [9, 10, 11]) {
    day[hour] = 2;
  }
  day[14] = 12.4;
  day[16] = 1;
  day[17] = 1;
  assert.equal(saidOf("Mon 20", day), "Mon 20: used 9:00 to 12:00, 14:00 to 15:00 and 16:00 to 18:00, most at 14:00 (12%)");
  assert.equal(saidOf("Thu 23", Array.from({ length: 24 }, () => 1)), "Thu 23: used 0:00 to 24:00, most at 0:00 (1%)");
});

test("the check counts the calendar days a period touches itself, a now at midnight touching none of its day", () => {
  const at = (iso) => Date.parse(iso);
  assert.equal(daysTouched({ start: at("2026-10-20T14:25:00Z"), now: at("2026-10-27T14:25:00Z") }), 8);
  assert.equal(daysTouched({ start: at("2026-10-21T00:00:00Z"), now: at("2026-10-28T00:00:00Z") }), 7);
  assert.equal(daysTouched({ start: at("2026-09-27T14:25:00Z"), now: at("2026-10-27T14:25:00Z") }), 31);
  assert.equal(daysTouched({ start: at("2026-10-27T09:10:00Z"), now: at("2026-10-27T11:00:00Z") }), 1);
  assert.equal(daysTouched({ start: at("2026-10-27T09:10:00Z"), now: at("2026-10-27T09:10:00Z") }), 0);
});

test("every copy's rows follow the rule: as many as the days its period touches, no cell outside the period, the first line naming its hour", () => {
  for (const [name, copy] of Object.entries({ pro, free, many, crowded, site })) {
    for (const days of [7, 30]) {
      const period = copy.periods[days];
      const rows = rowsOf(period.start, period.now);
      const view = copy.history[days];
      const what = `${name} over ${days} days`;
      assert.equal(rows.length, daysTouched(period), what);
      assert.ok(rows.length > days, `${what}: a partial row at each end, the copies' periods not ending at midnight`);
      for (const account of view.accounts) {
        const { heatmap } = account;
        assert.equal(heatmap.days.length, rows.length, `${what}: ${account.label}`);
        heatmap.cells.forEach((cells, row) => {
          assert.deepEqual(cells.map((cell) => cell !== null), rows[row].hours, `${what}: ${account.label}'s row ${row + 1}`);
        });
        const from = fromOf(rows);
        assert.ok(heatmap.said[0].startsWith(from === null ? `${heatmap.days[0]}: ` : `${heatmap.days[0]} from ${from}:00: `), heatmap.said[0]);
        assert.equal(heatmap.said.length, heatmap.days.length);
      }
      // The dates name the first and last rows.
      const [first, last] = [rows[0], rows.at(-1)].map(({ day }) => String(new Date(day).getUTCDate()));
      assert.ok(view.dates.startsWith(`${first} `) && view.dates.includes(` to ${last} `), `${what}: ${view.dates}`);
    }
  }
});

/**
 * What `HEAT_ROWS` finds of a card drawn as it should be from `account`'s heatmap: each row's grid, a
 * 44 px label and 24 columns of 10 px with 1 px between, from x 40, and a square drawn in the column
 * of each hour the row holds; the hours written at their columns; the first line of the heatmap's
 * words; and `words`, the marks'.
 */
function drawnCard(account, words) {
  const columns = [{ left: 40, right: 84 }, ...Array.from({ length: 24 }, (_, hour) => ({ left: 85 + hour * 11, right: 95 + hour * 11 }))];
  const { heatmap } = account;
  return {
    what: account.alias ?? account.provider,
    rows: heatmap.days.map((day, row) => ({
      day,
      columns,
      squares: heatmap.cells[row].flatMap((cell, hour) => (cell === null ? [] : [{ left: columns[hour + 1].left, right: columns[hour + 1].right, top: row * 11, bottom: row * 11 + 10, drawn: true }])),
    })),
    hours: [0, 6, 12, 18].map((hour) => ({ text: String(hour), left: columns[hour + 1].left })),
    said: heatmap.said[0],
    words,
  };
}

/** A copy of one account, Pro's first, over `days`, with its period. */
const one = (days) => ({ history: { [days]: { ...pro.history[days], accounts: [pro.history[days].accounts[0]] } }, periods: pro.periods });
const [week, month] = [7, 30].map((days) => pro.history[days].accounts[0]);
const wordsOf = (days) => pro.history[days].marks.filter((mark) => mark.word !== "").map((mark) => mark.word);

/** The verdicts that are not false. */
const failed = (verdicts) => verdicts.filter(Boolean);

test("a card drawn with the copy's rows, squares only where the period holds them, passes, over a week and a month", () => {
  assert.deepEqual(failed(rowVerdicts([drawnCard(week, wordsOf(7))], one(7), 7)), []);
  assert.deepEqual(failed(rowVerdicts([drawnCard(month, wordsOf(30))], one(30), 30)), []);
  // A square drawn but not shown is no square.
  const card = drawnCard(week, wordsOf(7));
  card.rows[0].squares.push({ left: 85, right: 95, top: 0, bottom: 10, drawn: false });
  assert.deepEqual(failed(rowVerdicts([card], one(7), 7)), []);
});

test("a window that draws every hour, as main's does, fails at both ends: squares outside the period", () => {
  const card = drawnCard(week, wordsOf(7));
  const every = card.rows[1].squares;
  const all = { ...card, rows: card.rows.map((row) => ({ ...row, squares: every.map((square, hour) => ({ ...square, left: row.columns[hour + 1].left, right: row.columns[hour + 1].right })) })) };
  assert.deepEqual(failed(rowVerdicts([all], one(7), 7)), [
    `Work, the team account we share across two offices's row "Sun 19" draws a square for 0:00, 1:00, 2:00, 3:00, 4:00, 5:00, 6:00, 7:00, 8:00, 9:00, 10:00, 11:00, 12:00, 13:00, outside the period, where its copy has no cell`,
    `Work, the team account we share across two offices's row "Sun 26" draws a square for 15:00, 16:00, 17:00, 18:00, 19:00, 20:00, 21:00, 22:00, 23:00, outside the period, where its copy has no cell`,
  ]);
});

test("squares placed one after another from the first column, as a grid does with nothing to keep their hours, fail as outside and missing", () => {
  const card = drawnCard(week, wordsOf(7));
  const first = card.rows[0];
  const shifted = { ...first, squares: first.squares.map((square, index) => ({ ...square, left: first.columns[index + 1].left, right: first.columns[index + 1].right })) };
  const verdicts = failed(rowVerdicts([{ ...card, rows: [shifted, ...card.rows.slice(1)] }], one(7), 7));
  assert.match(verdicts[0], /row "Sun 19" draws a square for 0:00, .*, 9:00, outside the period/);
  // 10:00 to 13:00 are outside the period too, so only the held hours are missing.
  assert.match(verdicts[1], /row "Sun 19" draws no square for 14:00, .*, 23:00, which the period holds/);
});

test("a square out of its column, or not under the squares of its hour, fails to a pixel", () => {
  const card = drawnCard(week, wordsOf(7));
  const moved = (by) => {
    const rows = card.rows.map((row) => ({ ...row, squares: [...row.squares] }));
    rows[3].squares[5] = { ...rows[3].squares[5], left: rows[3].squares[5].left + by, right: rows[3].squares[5].right + by };
    return failed(rowVerdicts([{ ...card, rows }], one(7), 7));
  };
  assert.deepEqual(moved(ALIGNED), []);
  assert.deepEqual(moved(-2), [
    `Work, the team account we share across two offices's row "Wed 22"'s square for 5:00 runs from 138.0 to 148.0, not in its column, 140.0 to 150.0`,
    `Work, the team account we share across two offices's squares for 5:00 are not under one another: from 138.0 in Work, the team account we share across two offices's row "Wed 22" to 140.0 in Work, the team account we share across two offices's row "Mon 20"`,
  ]);
  // Under the day's label, out of every hour's column; and then its own hour has none.
  const out = moved(-90);
  assert.match(out[0], /row "Wed 22" draws a square at 50\.0, under no hour's column/);
  assert.match(out[1], /row "Wed 22" draws no square for 5:00, which the period holds/);
});

test("rows not the copy's, in order, or not as many as the days the period touches, fail", () => {
  const card = drawnCard(week, wordsOf(7));
  const verdicts = failed(rowVerdicts([{ ...card, rows: card.rows.slice(1) }], one(7), 7));
  assert.match(verdicts[0], /heatmap rows are \["Mon 20",.*"Sun 26"\], where its copy has \["Sun 19","Mon 20",/);
  assert.match(verdicts[1], /heatmap has 7 rows, where its period of 7 days touches 8 calendar days/);
  const swapped = [card.rows[1], card.rows[0], ...card.rows.slice(2)];
  assert.match(failed(rowVerdicts([{ ...card, rows: swapped }], one(7), 7))[0], /heatmap rows are \["Mon 20","Sun 19",/);
  // A copy whose rows are not the period's days, as a copy written by another rule would be.
  const sevenRows = { ...one(7), periods: { 7: { start: Date.parse("2026-04-19T14:25:00Z"), now: Date.parse("2026-04-26T00:00:00Z") } } };
  assert.match(failed(rowVerdicts([card], sevenRows, 7))[0], /heatmap has 8 rows, where its period of 7 days touches 7 calendar days/);
  assert.deepEqual(failed(rowVerdicts([card], { ...one(7), periods: {} }, 7)), ["the copy says no period for its 7 days, to count the heatmap's rows against"]);
  assert.deepEqual(failed(rowVerdicts([], one(7), 7)), ["History shows 0 account cards, not the 1 the copy has"]);
});

test("the first line of the words must name the hour its row starts from, and the hours must stand over their columns", () => {
  const card = drawnCard(week, wordsOf(7));
  assert.deepEqual(failed(rowVerdicts([{ ...card, said: "Sun 19: not used" }], one(7), 7)), [
    `Work, the team account we share across two offices's heatmap says "Sun 19: not used" first, not a line that begins "Sun 19 from 14:00: "`,
  ]);
  assert.match(failed(rowVerdicts([{ ...card, said: null }], one(7), 7))[0], /says null first/);
  const hours = card.hours.map((hour) => (hour.text === "12" ? { ...hour, left: hour.left + 3 } : hour));
  assert.deepEqual(failed(rowVerdicts([{ ...card, hours }], one(7), 7)), [
    `Work, the team account we share across two offices's hour "12" is written at 220.0, not at its column, 217.0`,
  ]);
});

test("the marks must name the rows' days: over a month its two ends, over a week the days' initials from the first row or the next", () => {
  const rows = week.heatmap.days;
  assert.equal(markDaysVerdict(["M", "T", "W", "T", "F", "S", "S"], rows, 7, "Work"), false);
  assert.equal(markDaysVerdict(["S", "M", "T", "W", "T", "F", "S", "S"], rows, 7, "Work"), false);
  assert.equal(markDaysVerdict(["S", "M", "T", "W", "T", "F", "S"], rows, 7, "Work"), false);
  assert.match(markDaysVerdict(["T", "W", "T", "F", "S", "S", "M"], rows, 7, "Work"), /initials under its charts, "T W T F S S M", are not its heatmap's days', "S M T W T F S S"/);
  assert.equal(markDaysVerdict(["19 Apr", "19 May"], month.heatmap.days, 30, "Work"), false);
  assert.match(markDaysVerdict(["20 Apr", "19 May"], month.heatmap.days, 30, "Work"), /dates under its charts, \["20 Apr","19 May"\], are not its heatmap's first and last days, \["Sun 19","Tue 19"\]/);
  assert.match(markDaysVerdict([], rows, 7, "Work"), /has 0 marks' words and 8 heatmap rows/);
  // In the card's verdicts, as the window would draw marks of another first day.
  assert.match(failed(rowVerdicts([drawnCard(month, ["20 Apr", "19 May"])], one(30), 30))[0], /are not its heatmap's first and last days/);
});

test("a heatmap with no use has no rows, and drawing some fails", () => {
  const quiet = { ...week, heatmap: { ...week.heatmap, most: 0 } };
  const copy = { history: { 7: { accounts: [quiet] } }, periods: pro.periods };
  assert.deepEqual(failed(rowVerdicts([{ ...drawnCard(week, []), rows: [] }], copy, 7)), []);
  assert.match(failed(rowVerdicts([drawnCard(week, [])], copy, 7))[0], /draws 8 heatmap rows for a heatmap with no use/);
});

test("the code for the page is valid and reads the rows, their grid's columns, their squares, the hours, the words and the marks", () => {
  assert.doesNotThrow(() => new Script(HEAT_ROWS));
  for (const part of ["section.history .account-history", "figure.heatmap", ".heat-row:not(.heat-hours)", "gridTemplateColumns", "columnGap", ".heat-cell", ".heat-hours .heat-hour", "ul.visually-hidden li", ".marks span"]) {
    assert.ok(HEAT_ROWS.includes(part), part);
  }
});

test("the window draws no square for an hour with no cell and puts each square in its hour's column", () => {
  const source = readFileSync(new URL("../src/history.tsx", import.meta.url), "utf8");
  assert.match(source, /used === null \? null : \(/);
  assert.match(source, /gridColumn: String\(hour \+ 2\)/);
  const share = readFileSync(new URL("../src/share.ts", import.meta.url), "utf8");
  assert.match(share, /if \(used === null\) \{\s*return;\s*\}/);
});
