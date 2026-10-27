// How the window check holds History's time marks (time-marks.mjs, D-143), and the rule the made-up
// copies' marks are written by (fixture-marks.mjs): that rule against marks worked out by hand, as
// the app's own tests work them out; that every copy has marks, ticks and words, over 7 and 30 days,
// so the check cannot pass by finding none; that the marks' verdicts pass a card drawn as it should be
// and fail one with no marks, a tick off its place, a word missing or out of order, two words over
// each other or one past the chart's edge, a row over the last chart or under the heatmap's caption,
// and a row a screen reader is told; that the ticks read from the pixels under a baseline are the grey
// ones, where the marks put them; and that the stylesheet says a tick's length and the words' type.
// The browser's part, the boxes and pixels themselves, is held by the window check itself.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { Script } from "node:vm";

import { marksOf } from "./fixture-marks.mjs";
import { site } from "./site-fixtures.mjs";
import { rules } from "./stylesheet.mjs";
import { BAND, GREY, INK, MARKS, expectedMarks, heardVerdicts, inkVerdicts, inkedTicks, markVerdicts } from "./time-marks.mjs";
import { crowded, free, many, pro } from "./window-fixtures.mjs";

const HOUR = 60 * 60 * 1000;
const DAY = 24 * HOUR;

/** The marks of the `days` before `now`, an ISO time, for a history older than the period. */
const before = (days, now) => marksOf(days, Date.parse(now) - days * DAY, Date.parse(now));

/** Holds `marks` to `wanted`, each `[along, word]`, in order, each place to a thousand-millionth. */
function same(marks, wanted) {
  assert.deepEqual(
    marks.map((mark) => mark.word),
    wanted.map(([, word]) => word),
    JSON.stringify(marks),
  );
  for (const [index, [along]] of wanted.entries()) {
    assert.ok(Math.abs(marks[index].along - along) < 1e-9, `${JSON.stringify(marks[index])} is not at ${along}`);
  }
}

test("over 7 days, a tick at each midnight inside the period and an initial in each day held half or more, as the app's tests have them", () => {
  const week = (hours) => hours / 168;
  // Tuesday noon: both ends are half a day, and each has its initial.
  same(before(7, "2026-10-27T12:00:00Z"), [
    [week(6), "T"], [week(12), ""], [week(24), "W"], [week(36), ""], [week(48), "T"], [week(60), ""], [week(72), "F"], [week(84), ""],
    [week(96), "S"], [week(108), ""], [week(120), "S"], [week(132), ""], [week(144), "M"], [week(156), ""], [week(162), "T"],
  ]);
  // A minute after midnight: a tick a minute from the right edge, and no initial in today's minute.
  const seconds = (count) => count / 604_800;
  same(before(7, "2026-10-27T00:01:00Z"), [
    [seconds(43_170), "T"], [seconds(86_340), ""], [seconds(129_540), "W"], [seconds(172_740), ""], [seconds(215_940), "T"],
    [seconds(259_140), ""], [seconds(302_340), "F"], [seconds(345_540), ""], [seconds(388_740), "S"], [seconds(431_940), ""],
    [seconds(475_140), "S"], [seconds(518_340), ""], [seconds(561_540), "M"], [seconds(604_740), ""],
  ]);
  // Ending as a day begins: seven whole days and no tick at either edge.
  const days = (count) => count / 7;
  same(before(7, "2026-10-28T00:00:00Z"), [
    [days(0.5), "W"], [days(1), ""], [days(1.5), "T"], [days(2), ""], [days(2.5), "F"], [days(3), ""], [days(3.5), "S"],
    [days(4), ""], [days(4.5), "S"], [days(5), ""], [days(5.5), "M"], [days(6), ""], [days(6.5), "T"],
  ]);
});

test("over 30 days, a tick at each Monday's midnight inside the period, four or five, and the dates of the days its ends are in", () => {
  const month = (hours) => hours / 720;
  same(before(30, "2026-10-27T12:00:00Z"), [[0, "27 Sep"], [month(12), ""], [month(180), ""], [month(348), ""], [month(516), ""], [month(684), ""], [1, "27 Oct"]]);
  same(before(30, "2026-10-28T12:00:00Z"), [[0, "28 Sep"], [month(156), ""], [month(324), ""], [month(492), ""], [month(660), ""], [1, "28 Oct"]]);
  const seconds = (count) => count / 2_592_000;
  same(before(30, "2026-10-27T00:01:00Z"), [
    [0, "27 Sep"], [seconds(86_340), ""], [seconds(691_140), ""], [seconds(1_295_940), ""], [seconds(1_900_740), ""], [seconds(2_505_540), ""], [1, "27 Oct"],
  ]);
  const days = (count) => count / 30;
  same(before(30, "2026-10-28T00:00:00Z"), [[0, "28 Sep"], [days(7), ""], [days(14), ""], [days(21), ""], [days(28), ""], [1, "27 Oct"]]);
});

test("a period held from a younger history's first reading is marked as its charts span it, and an empty one has no marks", () => {
  // Monday 5 January 2099 at 09:10 to Wednesday 7 at midnight, 38 h 50 min.
  const [start, now] = [Date.parse("2099-01-05T09:10:00Z"), Date.parse("2099-01-07T00:00:00Z")];
  const seconds = (count) => count / 139_800;
  same(marksOf(7, start, now), [[seconds(26_700), "M"], [seconds(53_400), ""], [seconds(96_600), "T"]]);
  same(marksOf(30, start, now), [[0, "5 Jan"], [1, "6 Jan"]]);
  // Within one day: one date, and over a week no initial unless it is half a day.
  const nine = Date.parse("2099-01-07T09:00:00Z");
  same(marksOf(30, nine, nine + 2 * HOUR), [[0, "7 Jan"]]);
  same(marksOf(7, nine, nine + 2 * HOUR), []);
  same(marksOf(7, nine, nine + 12 * HOUR), [[0.5, "W"]]);
  same(marksOf(7, nine, nine), []);
});

test("every copy's History has ticks and words over 7 and 30 days, so a check that finds none cannot pass", () => {
  for (const [name, copy] of Object.entries({ pro, free, many, crowded, site })) {
    for (const days of [7, 30]) {
      const { ticks, words } = expectedMarks(copy, days);
      assert.ok(ticks.length >= 4, `${name} over ${days} days has ${ticks.length} ticks`);
      assert.ok(words.length >= 2, `${name} over ${days} days has ${words.length} words`);
      assert.ok(copy.history[days].accounts.length > 0, `${name} over ${days} days has no account to mark`);
    }
  }
});

test("the copies' marks: a week to Sunday 14:00 and a month to a Tuesday, and Crowded's to a Wednesday noon with a W at each end", () => {
  // Pro, Free and Many: Sunday 19 April at 14:00 to Sunday 26: no initial in the first ten hours,
  // and Sunday's fourteen have theirs.
  const week = (hours) => hours / 168;
  const sunday = [
    [week(10), ""], [week(22), "M"], [week(34), ""], [week(46), "T"], [week(58), ""], [week(70), "W"], [week(82), ""], [week(94), "T"],
    [week(106), ""], [week(118), "F"], [week(130), ""], [week(142), "S"], [week(154), ""], [week(161), "S"],
  ];
  for (const copy of [pro, free, many]) {
    same(copy.history[7].marks, sunday);
    same(copy.history[30].marks, [[0, "19 Apr"], [10 / 720, ""], [178 / 720, ""], [346 / 720, ""], [514 / 720, ""], [682 / 720, ""], [1, "19 May"]]);
  }
  // Crowded: Wednesday 22 April at noon to Wednesday 29: eight initials, a W in the half day at each
  // end, a twenty-eighth of the chart from its edge; and over a month, from Monday 30 March at noon,
  // four ticks.
  same(crowded.history[7].marks, [
    [week(6), "W"], [week(12), ""], [week(24), "T"], [week(36), ""], [week(48), "F"], [week(60), ""], [week(72), "S"], [week(84), ""],
    [week(96), "S"], [week(108), ""], [week(120), "M"], [week(132), ""], [week(144), "T"], [week(156), ""], [week(162), "W"],
  ]);
  same(crowded.history[30].marks, [[0, "30 Mar"], [156 / 720, ""], [324 / 720, ""], [492 / 720, ""], [660 / 720, ""], [1, "29 Apr"]]);
  // The heatmap's days are the calendar's, in the same days as the marks.
  assert.deepEqual([crowded.history[7].accounts[0].heatmap.days[0], crowded.history[7].accounts[0].heatmap.days.at(-1)], ["Thu 23", "Wed 29"]);
  assert.equal(crowded.history[7].dates, "23 to 29 April 2026");
  assert.deepEqual([pro.history[30].accounts[0].heatmap.days[11], pro.history[30].accounts[0].heatmap.days.at(-1)], ["Fri 1", "Tue 19"]);
  assert.equal(pro.history[30].dates, "20 April to 19 May 2026");
  // The website's: Tuesday 20 October at 14:10 to Tuesday 27.
  assert.deepEqual(expectedMarks(site, 7).words, ["W", "T", "F", "S", "S", "M", "T"]);
  assert.deepEqual(expectedMarks(site, 30).words, ["27 Sep", "27 Oct"]);
  assert.equal(expectedMarks(site, 30).ticks.length, 5);
});

test("the marks a check expects are the copy's ticks and words, each in the order the app sends them", () => {
  const copy = { history: { 7: { marks: [{ along: 0.1, word: "M" }, { along: 0.2, word: "" }, { along: 0.3, word: "T" }, { along: 0.4, word: "" }] } } };
  assert.deepEqual(expectedMarks(copy, 7), { ticks: [0.2, 0.4], words: ["M", "T"] });
  assert.deepEqual(expectedMarks(copy, 30), { ticks: [], words: [] });
});

/**
 * What `MARKS` finds of a card drawn as it should be: `charts` charts of `width` from x 40, each 40 px
 * tall with 30 px between, its ticks at their places and hanging 4 px; and under the last, 5 px below
 * its foot, the row of words, each 7 px wide a letter and placed as the stylesheet places it: a date
 * at an end flush with it, anything else centred on its place.
 */
function drawn(marks, { charts = 2, width = 280 } = {}) {
  const all = Array.from({ length: charts }, (_, index) => ({
    top: 100 + 70 * index,
    bottom: 140 + 70 * index,
    left: 40,
    right: 40 + width,
    role: "img",
    named: true,
    ticks: marks.filter((mark) => mark.word === "").map((mark) => ({ at: 40 + mark.along * width, length: 4 })),
  }));
  const last = all.at(-1);
  return {
    what: "Claude Work",
    charts: all,
    rows: [
      {
        chart: charts - 1,
        hidden: true,
        words: marks
          .filter((mark) => mark.word !== "")
          .map(({ along, word }) => {
            const wide = 7 * word.length;
            const left = along <= 0 ? last.left : along >= 1 ? last.right - wide : last.left + along * width - wide / 2;
            return { text: word, boxes: [{ top: last.bottom + 5, bottom: last.bottom + 17, left, right: left + wide }] };
          }),
      },
    ],
    caption: { top: last.bottom + 26, bottom: last.bottom + 40, left: 40, right: 40 + width },
  };
}

/** The verdicts that are not false. */
const failed = (verdicts) => verdicts.filter(Boolean);

test("a card drawn with its marks where they go passes, over a week and over a month", () => {
  for (const days of [7, 30]) {
    const marks = pro.history[days].marks;
    assert.deepEqual(failed(markVerdicts([drawn(marks)], expectedMarks(pro, days), 1)), []);
    assert.deepEqual(failed(markVerdicts([drawn(marks), drawn(marks, { charts: 3 })], expectedMarks(pro, days), 2)), []);
  }
});

test("a window with no marks, as main's, fails: no ticks on any chart and no row of words", () => {
  const card = drawn(pro.history[7].marks);
  const bare = { ...card, charts: card.charts.map((chart) => ({ ...chart, ticks: [] })), rows: [] };
  assert.deepEqual(failed(markVerdicts([bare], expectedMarks(pro, 7), 1)), [
    "Claude Work's chart 1 has 0 ticks, where its marks put 7",
    "Claude Work's chart 2 has 0 ticks, where its marks put 7",
    "Claude Work has no row of the marks' words",
  ]);
  // Nor does a History with fewer cards than the copy's accounts pass.
  assert.deepEqual(failed(markVerdicts([], expectedMarks(pro, 7), 2)), ["History shows 0 account cards to mark, not 2"]);
});

test("a tick off its place by more than half a pixel fails, and one more than the marks put", () => {
  const card = drawn(pro.history[7].marks);
  const off = (by) => ({ ...card, charts: [card.charts[0], { ...card.charts[1], ticks: card.charts[1].ticks.map((tick, index) => (index === 2 ? { ...tick, at: tick.at + by } : tick)) }] });
  assert.deepEqual(failed(markVerdicts([off(0.4)], expectedMarks(pro, 7), 1)), []);
  assert.deepEqual(failed(markVerdicts([off(-0.6)], expectedMarks(pro, 7), 1)), [
    "Claude Work's chart 2's tick 3 is at 136.07, -0.60 px from 136.67, where its mark puts it (0.3452 along)",
  ]);
  const extra = { ...card, charts: [{ ...card.charts[0], ticks: [...card.charts[0].ticks, { at: 200, length: 4 }] }, card.charts[1]] };
  assert.deepEqual(failed(markVerdicts([extra], expectedMarks(pro, 7), 1)), ["Claude Work's chart 1 has 8 ticks, where its marks put 7"]);
});

test("a word missing, out of order or one more than the marks have fails", () => {
  const card = drawn(pro.history[7].marks);
  const [row] = card.rows;
  const withWords = (words) => ({ ...card, rows: [{ ...row, words }] });
  assert.match(failed(markVerdicts([withWords(row.words.slice(1))], expectedMarks(pro, 7), 1))[0], /^Claude Work's row of words is \["T","W","T","F","S","S"\], where its marks have \["M","T","W","T","F","S","S"\]$/);
  const swapped = [row.words[1], row.words[0], ...row.words.slice(2)];
  const verdicts = failed(markVerdicts([withWords(swapped)], expectedMarks(pro, 7), 1));
  assert.match(verdicts[0], /row of words is \["T","M",/);
  // Drawn where each belongs but written in the other's place, the first is after the second.
  assert.ok(verdicts.some((verdict) => /^"T" in Claude Work's row of words overlaps "M" after it, or is drawn after it/.test(verdict)), verdicts.join("\n"));
  assert.match(failed(markVerdicts([withWords([...row.words, row.words[0]])], expectedMarks(pro, 7), 1))[0], /row of words is \[.*"S","M"\]/);
});

test("two words over each other, or one past the chart's left or right edge, fail", () => {
  const card = drawn(pro.history[7].marks);
  const [row] = card.rows;
  const moved = (index, by) => ({
    ...card,
    rows: [{ ...row, words: row.words.map((word, at) => (at === index ? { ...word, boxes: word.boxes.map((box) => ({ ...box, left: box.left + by, right: box.right + by })) } : word)) }],
  });
  // "T" (Tuesday) moved 34 px left, over "M".
  assert.deepEqual(failed(markVerdicts([moved(1, -34)], expectedMarks(pro, 7), 1)), [
    `"M" in Claude Work's row of words overlaps "T" after it, or is drawn after it (80.2, then 79.2)`,
  ]);
  // The first word moved past the chart's left edge, the last past its right.
  assert.deepEqual(failed(markVerdicts([moved(0, -40)], expectedMarks(pro, 7), 1)), [
    `"M" in Claude Work's row of words runs past the left edge of the chart above it (33.2 to 40.2, the chart 40.0 to 320.0)`,
  ]);
  assert.deepEqual(failed(markVerdicts([moved(6, 10)], expectedMarks(pro, 7), 1)), [
    `"S" in Claude Work's row of words runs past the right edge of the chart above it (314.8 to 321.8, the chart 40.0 to 320.0)`,
  ]);
  // Half a pixel is rounding: the last word 0.33 px past the edge.
  assert.deepEqual(failed(markVerdicts([moved(6, 8.5)], expectedMarks(pro, 7), 1)), []);
  // A date broken over two lines, and a word with no box.
  const month = drawn(pro.history[30].marks);
  const [date] = month.rows[0].words;
  const broken = { ...month, rows: [{ ...month.rows[0], words: [{ ...date, boxes: [date.boxes[0], { ...date.boxes[0], top: date.boxes[0].bottom, bottom: date.boxes[0].bottom + 12 }] }, ...month.rows[0].words.slice(1)] }] };
  assert.deepEqual(failed(markVerdicts([broken], expectedMarks(pro, 30), 1)), [
    `"19 Apr" in Claude Work's row of words is drawn on 2 lines`,
    `"19 Apr" in Claude Work's row of words is not above the heatmap's caption: its foot is 3.0 px below the caption's top`,
  ]);
  const unboxed = { ...month, rows: [{ ...month.rows[0], words: [{ ...date, boxes: [] }, ...month.rows[0].words.slice(1)] }] };
  assert.deepEqual(failed(markVerdicts([unboxed], expectedMarks(pro, 30), 1)), [`"19 Apr" in Claude Work's row of words has no box to measure`]);
});

test("a row over the last chart or its ticks, under the heatmap's caption, in another chart or not hidden from a screen reader fails", () => {
  const card = drawn(pro.history[7].marks);
  const [row] = card.rows;
  const lowered = (by) => ({ ...card, rows: [{ ...row, words: row.words.map((word) => ({ ...word, boxes: word.boxes.map((box) => ({ ...box, top: box.top + by, bottom: box.bottom + by })) })) }] });
  // Raised into the ticks, which reach 4 px under the chart's foot, then over the chart itself.
  assert.match(failed(markVerdicts([lowered(-2)], expectedMarks(pro, 7), 1))[0], /^"M" in Claude Work's row of words is not under the card's last chart and its ticks: its top is 3\.0 px below the chart's foot, whose ticks reach 4 px$/);
  assert.equal(failed(markVerdicts([lowered(-30)], expectedMarks(pro, 7), 1)).length, 7);
  // Lowered past the heatmap's caption's top.
  assert.match(failed(markVerdicts([lowered(10)], expectedMarks(pro, 7), 1))[0], /^"M" in Claude Work's row of words is not above the heatmap's caption: its foot is 1\.0 px below the caption's top$/);
  // A card with no heatmap's caption is held under its last chart only.
  assert.deepEqual(failed(markVerdicts([{ ...lowered(10), caption: null }], expectedMarks(pro, 7), 1)), []);
  assert.deepEqual(failed(markVerdicts([{ ...card, rows: [{ ...row, chart: 0 }] }], expectedMarks(pro, 7), 1)), ["Claude Work's row of words is in its chart 1 of 2, not its last"]);
  assert.deepEqual(failed(markVerdicts([{ ...card, rows: [{ ...row, chart: -1 }] }], expectedMarks(pro, 7), 1)), ["Claude Work's row of words is in none of its charts, not its last"]);
  assert.deepEqual(failed(markVerdicts([{ ...card, rows: [row, row] }], expectedMarks(pro, 7), 1)), ["Claude Work has 2 rows of the marks' words, not one"]);
  assert.deepEqual(failed(markVerdicts([{ ...card, rows: [{ ...row, hidden: false }] }], expectedMarks(pro, 7), 1)), [
    "Claude Work's row of words is not hidden from a screen reader, which hears the period in each chart's summary",
  ]);
  // Ticks drawn in a chart a screen reader does not hear as a named image.
  const unnamed = { ...card, charts: [{ ...card.charts[0], role: null }, { ...card.charts[1], named: false }] };
  assert.deepEqual(failed(markVerdicts([unnamed], expectedMarks(pro, 7), 1)), [
    "Claude Work's chart 1 is not drawn as a named image (role null), so its ticks are not covered by its summary for a screen reader",
    'Claude Work\'s chart 2 is not drawn as a named image (role "img"), so its ticks are not covered by its summary for a screen reader',
  ]);
  // A view whose marks have no words has no row.
  const ticksOnly = { ticks: expectedMarks(pro, 7).ticks, words: [] };
  assert.deepEqual(failed(markVerdicts([{ ...card, rows: [] }], ticksOnly, 1)), []);
  assert.deepEqual(failed(markVerdicts([card], ticksOnly, 1)), ["Claude Work has a row of words, where its marks have none"]);
});

/**
 * A picture of the band under a chart's foot, as `decodePng` gives one: `width` by `height` pixels of
 * the light card's ground, with `inks`, each `{ column, colour, rows }`, painted in.
 */
function band(width, height, inks = []) {
  return {
    width,
    height,
    pixel(x, y) {
      const ink = inks.find(({ column, rows = [0, height] }) => column === x && y >= rows[0] && y < rows[1]);
      return [...(ink ? ink.colour : [249, 249, 249]), 255];
    },
  };
}

test("the ticks under a baseline are the grey runs in the band under it, each at the middle of its ink, at 100% and 200%", () => {
  // At 100%, a picture from x 38 and y 139 of a chart whose foot is at 140: the band is its row 3.
  const frame = { x: 38, y: 139, zoom: 1 };
  const grey = [194, 194, 194];
  assert.deepEqual(inkedTicks(band(300, 6, [{ column: 62, colour: grey }]), frame, 140), [100.5]);
  // A line between two pixels shares its ink between them, and its middle is between them.
  assert.deepEqual(inkedTicks(band(300, 6, [{ column: 62, colour: [222, 222, 222] }, { column: 63, colour: [222, 222, 222] }]), frame, 140), [101]);
  // Ink only above the band, the baseline's own, is not a tick; nor is the share's blue line poking under it.
  assert.deepEqual(inkedTicks(band(300, 6, [{ column: 70, colour: grey, rows: [0, 2] }, { column: 80, colour: [0, 122, 255] }, { column: 81, colour: [187, 220, 250] }]), frame, 140), []);
  // Too faint to be ink, or not quite grey.
  assert.deepEqual(inkedTicks(band(300, 6, [{ column: 90, colour: [249 - INK + 1, 249 - INK + 1, 249 - INK + 1] }, { column: 91, colour: [200, 200, 200 + GREY + 1] }]), frame, 140), []);
  // At 200%, two of the picture's pixels a CSS pixel: a picture from y 139 holds the band in rows 5 to 8.
  const twice = { x: 38, y: 139, zoom: 2 };
  assert.deepEqual(inkedTicks(band(600, 12, [{ column: 124, colour: grey, rows: [5, 9] }, { column: 125, colour: grey, rows: [5, 9] }]), twice, 140), [100.5]);
  assert.deepEqual(inkedTicks(band(600, 12, [{ column: 124, colour: grey, rows: [0, 5] }]), twice, 140), []);
  assert.deepEqual(BAND, { from: 1.25, to: 3.5 });
});

test("a tick not drawn within a pixel of its place, or ink where no mark puts one, fails", () => {
  assert.deepEqual(failed(inkVerdicts([100.5, 160.9], [100, 160], "Claude Work's chart 1")), []);
  assert.deepEqual(failed(inkVerdicts([100.5], [100, 160], "Claude Work's chart 1")), [
    "Claude Work's chart 1 has no tick drawn within a pixel of 160.00, where its mark puts one",
  ]);
  assert.deepEqual(failed(inkVerdicts([100.5, 161.2], [100, 160], "Claude Work's chart 1")), [
    "Claude Work's chart 1 has no tick drawn within a pixel of 160.00, where its mark puts one",
    "Claude Work's chart 1 has something drawn under its baseline at 161.20, where no mark puts a tick",
  ]);
  assert.deepEqual(failed(inkVerdicts([], [], "Claude Work's chart 1")), []);
});

test("a screen reader is told nothing of the marks: a row, a word or a tick it is told of fails", () => {
  assert.deepEqual(failed(heardVerdicts([{ what: "Claude Work's row of words", ignored: true }])), []);
  assert.deepEqual(failed(heardVerdicts([{ what: "Claude Work's row of words", ignored: false }, { what: '"M" under Claude Work\'s charts', ignored: null }])), [
    "a screen reader is told Claude Work's row of words, which says again what the charts' summaries and the heatmap's lines say",
    "a screen reader is told \"M\" under Claude Work's charts, which says again what the charts' summaries and the heatmap's lines say",
  ]);
});

test("the code for the page is valid and reads History's cards, charts, ticks, rows of words and captions", () => {
  assert.doesNotThrow(() => new Script(MARKS));
  for (const part of ["section.history .account-history", "figure.chart", ".tick", ".marks", "aria-hidden", ".heatmap figcaption", "strokeDasharray"]) {
    assert.ok(MARKS.includes(part), part);
  }
});

test("the stylesheet says a tick's length, hangs it in the baseline's colour, and writes the words in the heatmap's hours' type", () => {
  const declared = (selector) => rules().filter((rule) => rule.selectors.includes(selector)).flatMap((rule) => rule.declarations);
  const tick = declared(".chart .tick");
  for (const declaration of ["stroke: var(--chart-base)", "stroke-width: 1", "stroke-dasharray: 4 1000", "vector-effect: non-scaling-stroke"]) {
    assert.ok(tick.includes(declaration), declaration);
  }
  // The words are in the one rule with the heatmap's day names and hours.
  const hours = rules().find((rule) => rule.selectors.includes(".heat-hour"));
  assert.deepEqual(hours.selectors.toSorted(), [".heat-day", ".heat-hour", ".marks"]);
  assert.ok(hours.declarations.includes("color: var(--muted)"));
  // Clear of the ticks, and a word on one line, kept inside the chart.
  assert.ok(declared(".marks").includes("margin-top: 6px"));
  assert.ok(declared(".marks span").includes("white-space: nowrap"));
  assert.ok(declared(".marks span").includes("left: clamp(0px, calc(var(--along) * 100% - 0.5em), calc(100% - 1em))"));
  // The window hides the row from a screen reader, and draws a tick from the baseline down.
  const source = readFileSync(new URL("../src/history.tsx", import.meta.url), "utf8");
  assert.match(source, /<div class="marks" aria-hidden="true">/);
  assert.match(source, /class="tick"[\s\S]*?y1="100"[\s\S]*?y2="200"/);
});
