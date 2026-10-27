// Holds History's time marks (the manager's decision on #73, 2026-10-27, 15:28 UTC, D-143): a tick on
// every chart's baseline where a day begins, or over 30 days a week, and a row of words under each
// card's last chart, the days' initials over 7 days and the period's two dates over 30, all from the
// one `marks` list the app sends with the view, which the window draws and works nothing out from.
// The window check (check-window.mjs) finds, in the page, each card's charts with their ticks and its
// rows of words with the boxes each word is drawn in, and reads the pixels just under each chart's
// baseline; what it makes of them, against the marks the copy has, is here, so that tests which need
// no browser hold it.

import { linesOf } from "./whole-shares.mjs";

/**
 * How far under a chart's baseline its pixels are read for its ticks, in CSS pixels: inside a tick,
 * which hangs 4 px from the baseline (`.chart .tick` in style.css), and clear of the baseline's own
 * line and of the share's line, which can run along the baseline, each drawn across it.
 */
export const BAND = { from: 1.25, to: 3.5 };

/**
 * How far from where its mark puts it a tick may be, in CSS pixels: half a pixel for its line's own
 * place in the page, which is exact but for rounding, and a pixel for where its pixels are inked,
 * which a line between two pixels shares between them.
 */
export const PLACED = 0.5;
export const INKED = 1;

/** How much a pixel must differ from the band's ground, in any channel of 255, to be ink. */
export const INK = 10;

/**
 * How far apart a pixel's channels may be, of 255, for it to be a tick's ink: a tick is drawn in
 * `--chart-base`, black or white laid thinly over a grey card, so a grey, while the share's line,
 * which a sharp turn at 0% pokes under the baseline (a session's reset does), is blue.
 */
export const GREY = 8;

/**
 * The code to run in the page: History's account cards, in order, each as `{ what, charts, rows,
 * caption }`: the card's title; each chart's drawing, its box, its role and whether it has a name,
 * and the place across the page of each tick in it, the left of its box, since a tick's line has no
 * width, with the length its dash draws; each row of words in the card, the chart it is in (-1 for
 * none), whether it is hidden from a screen reader, and each word's text and the boxes it is drawn in,
 * one for each line (a range's client rects); and the box of the heatmap's caption, or of the line
 * said in the heatmap's place for an account not used in the period.
 */
export const MARKS = `(() => {
  const box = (element) => {
    const { top, bottom, left, right } = element.getBoundingClientRect();
    return { top, bottom, left, right };
  };
  return [...document.querySelectorAll('section.history .account-history')].map((card) => {
    const charts = [...card.querySelectorAll('figure.chart')];
    const caption = card.querySelector('.heatmap figcaption') ?? card.querySelector(':scope > p.detail');
    return {
      what: (card.querySelector('.card-head .title')?.textContent ?? 'a card').trim(),
      charts: charts.map((figure) => {
        const svg = figure.querySelector('svg');
        if (!svg) return { top: 0, bottom: 0, left: 0, right: 0, role: null, named: false, ticks: [] };
        return {
          ...box(svg),
          role: svg.getAttribute('role'),
          named: (svg.getAttribute('aria-label') ?? '').trim() !== '',
          ticks: [...svg.querySelectorAll('.tick')].map((tick) => ({
            at: tick.getBoundingClientRect().left,
            length: parseFloat(getComputedStyle(tick).strokeDasharray) || 0,
          })),
        };
      }),
      rows: [...card.querySelectorAll('.marks')].map((row) => ({
        chart: charts.findIndex((figure) => figure.contains(row)),
        hidden: !!row.closest('[aria-hidden="true"]'),
        words: [...row.children].map((word) => {
          const range = document.createRange();
          range.selectNodeContents(word);
          return {
            text: word.textContent,
            boxes: [...range.getClientRects()].map(({ top, bottom, left, right }) => ({ top, bottom, left, right })),
          };
        }),
      })),
      caption: caption ? box(caption) : null,
    };
  });
})()`;

/**
 * The marks a copy's History has over `days`: the places of its ticks along the charts, from 0 to 1,
 * and its words, each in the order along the charts that the app sends them in.
 */
export function expectedMarks(copy, days) {
  const marks = copy.history[days]?.marks ?? [];
  return {
    ticks: marks.filter((mark) => mark.word === "").map((mark) => mark.along),
    words: marks.filter((mark) => mark.word !== "").map((mark) => mark.word),
  };
}

/** `box` as "from x to y" across the page, in CSS pixels, for a verdict. */
const across = (box) => `${box.left.toFixed(1)} to ${box.right.toFixed(1)}`;

/**
 * Verdicts on History's marks as `MARKS` found them, against `wanted`, the copy's (`expectedMarks`),
 * in a view of `cards` accounts: false where each holds, or what is wrong, naming the card.
 *
 * - **The ticks:** each chart has exactly the ticks the marks put, each at its place along the chart,
 *   the chart's left plus its mark's `along` of its width, to half a pixel, and no others, so a window
 *   with no ticks, or one off its place, fails; and each is inside a chart drawn as a named image
 *   (`<svg role="img">` with its summary), which a screen reader hears in its place.
 * - **The words:** each card has one row of them, in its last chart, holding the copy's words in
 *   order, so a check that finds none cannot pass and one missing or out of order fails; hidden from
 *   a screen reader; each word on one line, inside the last chart's width, apart from the next with
 *   nothing of either over the other; and under the last chart and its ticks and above the heatmap's
 *   caption, so the row of the marks and the row of the heatmap's hours, with the caption between
 *   them, are never read as one. A view with no words has no row.
 */
export function markVerdicts(found, wanted, cards) {
  const verdicts = [found.length !== cards && `History shows ${found.length} account cards to mark, not ${cards}`];
  for (const { what, charts, rows, caption } of found) {
    for (const [index, chart] of charts.entries()) {
      const name = `${what}'s chart ${index + 1}`;
      const width = chart.right - chart.left;
      const ticks = chart.ticks.map((tick) => tick.at).toSorted((a, b) => a - b);
      verdicts.push(
        (chart.role !== "img" || !chart.named) &&
          `${name} is not drawn as a named image (role ${JSON.stringify(chart.role)}), so its ticks are not covered by its summary for a screen reader`,
      );
      if (ticks.length !== wanted.ticks.length) {
        verdicts.push(`${name} has ${ticks.length} ticks, where its marks put ${wanted.ticks.length}`);
        continue;
      }
      for (const [at, along] of wanted.ticks.entries()) {
        const place = chart.left + along * width;
        verdicts.push(
          Math.abs(ticks[at] - place) > PLACED &&
            `${name}'s tick ${at + 1} is at ${ticks[at].toFixed(2)}, ${(ticks[at] - place).toFixed(2)} px from ${place.toFixed(2)}, where its mark puts it (${along.toFixed(4)} along)`,
        );
      }
    }
    const last = charts.at(-1);
    if (wanted.words.length === 0) {
      verdicts.push(rows.length > 0 && `${what} has ${rows.length === 1 ? "a row" : `${rows.length} rows`} of words, where its marks have none`);
      continue;
    }
    if (rows.length !== 1) {
      verdicts.push(rows.length === 0 ? `${what} has no row of the marks' words` : `${what} has ${rows.length} rows of the marks' words, not one`);
      continue;
    }
    const [row] = rows;
    const written = row.words.map((word) => word.text);
    verdicts.push(
      written.join(" ") !== wanted.words.join(" ") &&
        `${what}'s row of words is ${JSON.stringify(written)}, where its marks have ${JSON.stringify(wanted.words)}`,
      row.chart !== charts.length - 1 &&
        `${what}'s row of words is ${row.chart < 0 ? "in none of its charts" : `in its chart ${row.chart + 1} of ${charts.length}`}, not its last`,
      !row.hidden && `${what}'s row of words is not hidden from a screen reader, which hears the period in each chart's summary`,
    );
    if (!last) {
      continue;
    }
    // Under the last chart's ticks as well as its drawing.
    const ticked = Math.max(0, ...last.ticks.map((tick) => tick.length));
    for (const [index, { text, boxes }] of row.words.entries()) {
      const word = `"${text}" in ${what}'s row of words`;
      const lines = linesOf(boxes);
      if (lines === 0) {
        verdicts.push(`${word} has no box to measure`);
        continue;
      }
      const [left, right, top, bottom] = [
        Math.min(...boxes.map((one) => one.left)),
        Math.max(...boxes.map((one) => one.right)),
        Math.min(...boxes.map((one) => one.top)),
        Math.max(...boxes.map((one) => one.bottom)),
      ];
      verdicts.push(
        lines > 1 && `${word} is drawn on ${lines} lines`,
        left < last.left - 0.5 && `${word} runs past the left edge of the chart above it (${across({ left, right })}, the chart ${across(last)})`,
        right > last.right + 0.5 && `${word} runs past the right edge of the chart above it (${across({ left, right })}, the chart ${across(last)})`,
        top < last.bottom + ticked - 0.5 &&
          `${word} is not under the card's last chart and its ticks: its top is ${(top - last.bottom).toFixed(1)} px below the chart's foot, whose ticks reach ${ticked} px`,
        caption && bottom > caption.top + 0.5 && `${word} is not above the heatmap's caption: its foot is ${(bottom - caption.top).toFixed(1)} px below the caption's top`,
      );
      const next = row.words[index + 1];
      if (next?.boxes.length) {
        const after = Math.min(...next.boxes.map((one) => one.left));
        verdicts.push(right > after + 0.5 && `${word} overlaps "${next.text}" after it, or is drawn after it (${right.toFixed(1)}, then ${after.toFixed(1)})`);
      }
    }
  }
  return verdicts;
}

/**
 * The ticks a picture of a chart's foot shows, as the place across the page of each, in CSS pixels:
 * each run of columns with a tick's ink in the band under the baseline (`BAND`), a grey (`GREY`) at
 * least `INK` from the ground, at the middle of its ink. `picture` is the capture, with `width`,
 * `height` and `pixel(x, y)` as `[r, g, b, a]` (share-picture.mjs's `decodePng`); `frame` is where it
 * was taken, `{ x, y, zoom }`, its top left corner in CSS pixels and how many of its pixels a CSS pixel
 * is; and `baseline` is the chart's foot, in the same CSS pixels. The ground is the band's commonest
 * colour, the card's.
 */
export function inkedTicks(picture, { x, y, zoom }, baseline) {
  const rows = [];
  for (let row = 0; row < picture.height; row += 1) {
    const [top, bottom] = [y + row / zoom, y + (row + 1) / zoom];
    if (top >= baseline + BAND.from - 1e-9 && bottom <= baseline + BAND.to + 1e-9) {
      rows.push(row);
    }
  }
  if (rows.length === 0) {
    return [];
  }
  const counts = new Map();
  for (const row of rows) {
    for (let column = 0; column < picture.width; column += 1) {
      const key = picture.pixel(column, row).slice(0, 3).join(",");
      counts.set(key, (counts.get(key) ?? 0) + 1);
    }
  }
  const ground = [...counts.entries()].reduce((most, entry) => (entry[1] > most[1] ? entry : most))[0].split(",").map(Number);
  const ink = Array.from({ length: picture.width }, (_, column) =>
    Math.max(
      ...rows.map((row) => {
        const colour = picture.pixel(column, row).slice(0, 3);
        const grey = Math.max(...colour) - Math.min(...colour) <= GREY;
        return grey ? Math.max(...ground.map((channel, at) => Math.abs(colour[at] - channel))) : 0;
      }),
    ),
  );
  const ticks = [];
  for (let column = 0; column < ink.length; column += 1) {
    if (ink[column] < INK) {
      continue;
    }
    let [weight, sum] = [0, 0];
    for (; column < ink.length && ink[column] >= INK; column += 1) {
      weight += ink[column];
      sum += ink[column] * (column + 0.5);
    }
    ticks.push(x + sum / weight / zoom);
  }
  return ticks;
}

/**
 * Verdicts on the ticks `inkedTicks` found under a chart (`found`, places across the page) against
 * where its marks put them (`wanted`): each drawn within a pixel of its place, and nothing else drawn
 * there, named as `what`.
 */
export function inkVerdicts(found, wanted, what) {
  const near = (a, b) => Math.abs(a - b) <= INKED;
  return [
    ...wanted.map((place) => !found.some((at) => near(at, place)) && `${what} has no tick drawn within a pixel of ${place.toFixed(2)}, where its mark puts one`),
    ...found.map((at) => !wanted.some((place) => near(at, place)) && `${what} has something drawn under its baseline at ${at.toFixed(2)}, where no mark puts a tick`),
  ];
}

/**
 * Verdicts on what a screen reader is told of the marks: `told`, each `{ what, ignored }`, the
 * accessibility tree's node for a row of words, a word or a tick, which must each be left out of
 * what is read, the row by `aria-hidden` and a tick by the named image it is drawn in.
 */
export function heardVerdicts(told) {
  return told.map(({ what, ignored }) => ignored !== true && `a screen reader is told ${what}, which says again what the charts' summaries and the heatmap's lines say`);
}
