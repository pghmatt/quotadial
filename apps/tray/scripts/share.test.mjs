// How the share picture is drawn (src/share.ts, which check-window.mjs also draws in Chrome and
// reads the pixels of): the account's card on the material (D-137), read here from the calls the
// drawing makes on a canvas that records them, so each part of the design is held on its own and a
// change that moves one fails by name. The canvas measures text by a fixed width a character, so
// what is held about where things sit is held for any font as wide, and the real font's is held by
// the window check. The accounts are the window check's own made-up ones.
import assert from "node:assert/strict";
import { test } from "node:test";

import { CARD, MATERIAL, chartBoxes, drawShare, lookOf, shareStrings } from "../src/share.ts";
import { site } from "./site-fixtures.mjs";
import { themes } from "./stylesheet.mjs";
import { pro } from "./window-fixtures.mjs";

/** The theme's colours: a different one for each use, so a use of the wrong one is seen. */
const LOOKS = {
  light: {
    theme: "light",
    font: "Test Sans, sans-serif",
    text: "#101010",
    secondary: "#202020",
    muted: "#303030",
    warning: "#404040",
    high: "#505050",
    card: "rgba(255, 255, 255, 0.7)",
    edge: "rgba(0, 0, 0, 0.07)",
    base: "#606060",
    top: "#707070",
    line: "#0000f0",
    limit: "#f0a000",
  },
  dark: {
    theme: "dark",
    font: "Test Sans, sans-serif",
    text: "#f0f0f0",
    secondary: "#e0e0e0",
    muted: "#d0d0d0",
    warning: "#c0c0c0",
    high: "#b0b0b0",
    card: "rgba(255, 255, 255, 0.06)",
    edge: "rgba(255, 255, 255, 0)",
    base: "#a0a0a0",
    top: "#909090",
    line: "#2020f0",
    limit: "#f0b020",
  },
};

/** The uses of colour that are for text, the picture's five. */
const TEXT_COLOURS = ["text", "secondary", "muted", "warning", "high"];

/** Whether two lengths in pixels are the same, as far as 10/3 and a sum of widths can tell. */
const same = (a, b) => Math.abs(a - b) < 1e-9;

/**
 * A canvas that records what is drawn on it, in the picture's CSS pixels: every filled rectangle,
 * every path filled or stroked with its bounds, and every piece of text with where it starts, its
 * size and weight, and the width it is given: `WIDE` of its size a character, a figure wider.
 */
function recorder() {
  const ops = [];
  const KEYS = ["fillStyle", "strokeStyle", "lineWidth", "lineJoin", "lineCap", "globalAlpha", "font", "textAlign", "textBaseline"];
  const saved = [];
  let dash = [];
  let path = [];
  const context = {
    fillStyle: "#000000",
    strokeStyle: "#000000",
    lineWidth: 1,
    lineJoin: "miter",
    lineCap: "butt",
    globalAlpha: 1,
    font: "10px sans-serif",
    textAlign: "start",
    textBaseline: "alphabetic",
    save() {
      saved.push({ values: Object.fromEntries(KEYS.map((key) => [key, context[key]])), dash });
    },
    restore() {
      const last = saved.pop();
      Object.assign(context, last.values);
      dash = last.dash;
    },
    setTransform() {},
    setLineDash(next) {
      dash = [...next];
    },
    beginPath() {
      path = [];
    },
    moveTo(x, y) {
      path.push([x, y]);
    },
    lineTo(x, y) {
      path.push([x, y]);
    },
    arcTo(x1, y1, x2, y2) {
      path.push([x1, y1], [x2, y2]);
    },
    arc(x, y, r) {
      path.push([x - r, y - r], [x + r, y + r]);
    },
    closePath() {},
    clip() {
      ops.push({ op: "clip" });
    },
    fillRect(...box) {
      ops.push({ op: "fillRect", box, fill: context.fillStyle });
    },
    fill() {
      ops.push({ op: "fill", box: bounds(path), fill: context.fillStyle, alpha: context.globalAlpha });
    },
    stroke() {
      ops.push({
        op: "stroke",
        box: bounds(path),
        stroke: context.strokeStyle,
        width: context.lineWidth,
        dash: [...dash],
        alpha: context.globalAlpha,
      });
    },
    measureText(text) {
      const size = Number(context.font.match(/([0-9.]+)px/)[1]);
      return { width: Array.from(text).length * size * (/^[0-9]+$/.test(text) ? 0.6 : 0.56) };
    },
    fillText(text, x, y) {
      const [, weight, size] = context.font.match(/^(\d+) ([0-9.]+)px/);
      ops.push({
        op: "text",
        text,
        x,
        y,
        size: Number(size),
        weight: Number(weight),
        fill: context.fillStyle,
        width: context.measureText(text).width,
      });
    },
  };
  return { context, ops };
}

/** The smallest box round `points`, as `{ x0, y0, x1, y1 }`. */
function bounds(points) {
  const [xs, ys] = [points.map(([x]) => x), points.map(([, y]) => y)];
  return { x0: Math.min(...xs), y0: Math.min(...ys), x1: Math.max(...xs), y1: Math.max(...ys) };
}

/** What drawing `account` of `view` in `theme` does on a canvas. */
function draw(view, account, theme) {
  const { context, ops } = recorder();
  drawShare(context, view, account, LOOKS[theme]);
  return ops;
}

/** The accounts drawn: the window check's Pro copy over 7 and 30 days, and the website's two. */
const ACCOUNTS = [
  ...pro.history[7].accounts.map((account) => [pro.history[7], account]),
  [pro.history[30], pro.history[30].accounts[0]],
  ...site.history[7].accounts.map((account) => [site.history[7], account]),
];

/** The account of a given number of windows, from the Pro copy. */
const withWindows = (count) => pro.history[7].accounts.find((account) => account.windows.length === count);

/** The writing of a picture: each piece of text. */
const texts = (ops) => ops.filter((op) => op.op === "text");

test("the picture opens with the material of the theme, then the card on it as the window draws one", () => {
  const [view, account] = ACCOUNTS[0];
  for (const theme of ["light", "dark"]) {
    // A page's own background is never the picture's ground, whatever it holds.
    const ops = draw(view, account, theme);
    assert.deepEqual(ops[0], { op: "fillRect", box: [0, 0, 1200, 675], fill: MATERIAL[theme] });
    assert.equal(MATERIAL[theme], { light: "#ececec", dark: "#2c2c2e" }[theme]);
    const [card, edge] = ops.filter((op) => op.op === "fill" || op.op === "stroke");
    assert.deepEqual(card, {
      op: "fill",
      box: { x0: CARD.x, y0: CARD.y, x1: CARD.x + CARD.width, y1: CARD.y + CARD.height },
      fill: LOOKS[theme].card,
      alpha: 1,
    });
    // The hairline edge runs just inside the outline, as the window's inset shadow does.
    const half = CARD.edge / 2;
    assert.deepEqual(edge.box, { x0: CARD.x + half, y0: CARD.y + half, x1: CARD.x + CARD.width - half, y1: CARD.y + CARD.height - half });
    assert.equal(edge.stroke, LOOKS[theme].edge);
    assert.equal(edge.width, CARD.edge);
  }
});

test("the card is the window's, to scale: 10/3 of its corners and padding, inset 40 px as its 12 are", () => {
  assert.deepEqual(
    [CARD.x, CARD.y, CARD.width, CARD.height, CARD.radius, CARD.padding],
    [40, 40, 1120, 595, 33, 40],
  );
  // The window's 0.5 px hairline, drawn three times as thick, 3 device pixels, so it shows when the picture is small.
  assert.equal(CARD.edge, 1.5);
  // The 336 px card of the 360 px window, 12 px in from each side, drawn 10/3 times as large.
  assert.ok(same(CARD.width, 336 * (10 / 3)));
  assert.ok(same(CARD.x, 12 * (10 / 3)));
  assert.equal(Math.round(10 * (10 / 3)), CARD.radius);
  assert.ok(same(CARD.padding, 12 * (10 / 3)));
  // Inset equally on every side, so it sits in the middle of the picture.
  assert.equal(CARD.x, 1200 - (CARD.x + CARD.width));
  assert.equal(CARD.y, 675 - (CARD.y + CARD.height));
});

test("no plate is drawn behind a chart: after the card, only the line's own shading and the heatmap's squares are filled", () => {
  for (const theme of ["light", "dark"]) {
    for (const [view, account] of ACCOUNTS) {
      const ops = draw(view, account, theme);
      const fills = ops.filter((op) => op.op === "fill").slice(1);
      assert.equal(ops.filter((op) => op.op === "fillRect").length, 1, "only the ground is a rectangle");
      for (const fill of fills) {
        assert.equal(fill.fill, LOOKS[theme].line, `${account.label}: a fill in ${fill.fill}`);
      }
      // The shading under a line is faint; a plate is not.
      const boxes = chartBoxes(account.windows.length);
      const shading = fills.filter((fill) => fill.alpha === 0.16);
      assert.equal(shading.length, account.windows.length, account.label);
      for (const fill of fills) {
        const covers = boxes.some(
          (box) => fill.box.x0 <= box.x + 0.5 && fill.box.x1 >= box.x + box.width - 0.5 && fill.box.y0 <= box.y + 0.5 && fill.box.y1 >= box.y + box.height - 0.5,
        );
        assert.equal(covers, false, `${account.label}: a fill with alpha ${fill.alpha} covers a whole chart`);
      }
    }
  }
});

test("each chart has the scale History draws: a faint line at 100%, a hairline at 0%, a dashed line at 90%, and its line", () => {
  for (const theme of ["light", "dark"]) {
    const look = LOOKS[theme];
    for (const [view, account] of ACCOUNTS) {
      const strokes = draw(view, account, theme).filter((op) => op.op === "stroke").slice(1);
      const boxes = chartBoxes(account.windows.length);
      boxes.forEach((box, index) => {
        const along = (colour) => strokes.filter((op) => op.stroke === colour && op.box.x0 === box.x && op.box.x1 === box.x + box.width);
        const [top] = along(look.top);
        const [base] = along(look.base);
        const [limit] = along(look.limit);
        assert.ok(top && base && limit, `${account.label}, chart ${index + 1}: a line is missing`);
        assert.deepEqual([top.box.y0, top.box.y1, top.dash], [box.y, box.y, []], "the faint line is at 100%");
        assert.deepEqual([base.box.y0, base.box.y1, base.dash], [box.y + box.height, box.y + box.height, []], "the hairline is at 0%");
        assert.deepEqual([limit.box.y0, limit.dash], [box.y + 0.1 * box.height, [6, 6]], "the dashed line is at 90%");
        assert.equal(top.width, 2);
      });
      // Each window's own line, thicker than the scale and in the line's colour, drawn last.
      const lines = strokes.filter((op) => op.stroke === look.line);
      assert.equal(lines.length, account.windows.length, account.label);
      assert.ok(lines.every((line) => line.width === 3), "the line is 3 px");
    }
  }
});

test("a chart runs for the column's width less the room of its 90% label, which is written after it", () => {
  const widest = chartBoxes(1)[0];
  // One window has the whole room: the card's contents, 80 px to 1120 px, less 48 px for the label.
  assert.deepEqual(widest, { x: 80, y: widest.y, width: 1040 - 48, height: widest.height });
  const three = chartBoxes(3);
  assert.equal(three.length, 3);
  for (const [index, box] of three.entries()) {
    assert.equal(box.y, widest.y);
    assert.ok(box.x + box.width + 48 <= 1120 + 0.001, "the label's room ends inside the card's padding");
    if (index > 0) {
      assert.ok(box.x >= three[index - 1].x + three[index - 1].width + 48, "a column starts after the last one's label");
    }
  }
  assert.deepEqual(chartBoxes(0), []);
  // The label sits at the right end of each chart's column, level with its dashed line.
  const account = withWindows(3);
  const ops = draw(pro.history[7], account, "light");
  const limits = texts(ops).filter((piece) => piece.text === "%" && piece.size === 16);
  assert.equal(limits.length, 3, "a 90% label for each of three charts");
});

test("the heading is the card's: the label in its title type, the period under it in the second line's colour, the brand at its right", () => {
  const [view, account] = ACCOUNTS.find(([, one]) => one.label === "Claude Work");
  for (const theme of ["light", "dark"]) {
    const look = LOOKS[theme];
    const pieces = texts(draw(view, account, theme));
    const title = pieces.filter((piece) => piece.size === 47);
    // 14 px semibold, 10/3 over, in the text colour, from the card's padding.
    assert.equal(title.map((piece) => piece.text).join(""), "Claude Work");
    assert.ok(title.every((piece) => piece.weight === 600 && piece.fill === look.text), "semibold, in the text colour");
    assert.equal(title[0].x, CARD.x + CARD.padding);
    const period = pieces.filter((piece) => piece.size === 25 && piece.weight === 400);
    assert.equal(period.map((piece) => piece.text).join(""), "The last 7 days, 21 to 27 October 2026");
    assert.ok(period.every((piece) => piece.fill === look.secondary), "the period is the second line's colour");
    assert.equal(period[0].x, CARD.x + CARD.padding);
    assert.ok(period[0].y > title[0].y, "the period is under the title");
    // QuotaDial and its address, small, ending at the card's right padding, on the title's and period's lines.
    const brand = pieces.find((piece) => piece.text === "QuotaDial");
    const address = pieces.find((piece) => piece.text === "quotadial.com");
    assert.ok(same(brand.x + brand.width, CARD.x + CARD.width - CARD.padding));
    assert.ok(same(address.x + address.width, CARD.x + CARD.width - CARD.padding));
    assert.equal(brand.y, title[0].y);
    assert.equal(address.y, period[0].y);
    assert.equal(brand.fill, look.text);
    assert.equal(address.fill, look.muted);
    assert.ok(brand.size < 28 && address.size < 28, "nothing louder than the numbers or the title");
  }
});

test("the figures are the largest thing drawn, semibold, and their colour follows how full the window is", () => {
  const [view, account] = ACCOUNTS.find(([, one]) => one.label === "Claude Work");
  const look = LOOKS.light;
  const pieces = texts(draw(view, account, "light"));
  const biggest = Math.max(...pieces.map((piece) => piece.size));
  const figures = pieces.filter((piece) => piece.size === biggest);
  assert.equal(biggest, 84, "one window or three, the figures reach 84 px");
  assert.ok(figures.every((piece) => piece.weight === 600));
  // 91%, 74% and 38% are each three pieces, two figures and a sign: high, warning and the text colour, in turn.
  assert.deepEqual(figures.map((piece) => piece.text).join(""), "91%74%38%");
  assert.deepEqual(
    figures.map((piece) => piece.fill),
    [...Array(3).fill(look.high), ...Array(3).fill(look.warning), ...Array(3).fill(look.text)],
  );
});

test("everything is written in the stylesheet's text colours, and in them alone", () => {
  for (const theme of ["light", "dark"]) {
    const allowed = new Set(TEXT_COLOURS.map((use) => LOOKS[theme][use]));
    for (const [view, account] of ACCOUNTS) {
      for (const piece of texts(draw(view, account, theme))) {
        assert.ok(allowed.has(piece.fill), `${account.label}: "${piece.text}" in ${piece.fill}`);
      }
    }
  }
});

test("a window with one column, two or three: all of it inside the card's padding, and nothing written over anything else", () => {
  for (const theme of ["light", "dark"]) {
    for (const [view, account] of ACCOUNTS) {
      const pieces = texts(draw(view, account, theme));
      const boxes = pieces.map((piece) => ({
        piece,
        left: piece.x,
        right: piece.x + piece.width,
        // Capitals and figures rise 0.72 of their size; a letter with a tail falls 0.22 below the line, and figures and signs do not.
        top: piece.y - 0.72 * piece.size,
        bottom: piece.y + (/[gjpqy,;()]/.test(piece.text) ? 0.22 : 0.02) * piece.size,
      }));
      const [left, right] = [CARD.x + CARD.padding, CARD.x + CARD.width - CARD.padding];
      for (const box of boxes) {
        const where = `${account.label}, "${box.piece.text}"`;
        assert.ok(box.left >= left - 0.5 && box.right <= right + 0.5, `${where} is outside the card's padding at the sides`);
        assert.ok(box.top >= CARD.y + CARD.padding / 2, `${where} is too near the card's top`);
        assert.ok(box.bottom <= CARD.y + CARD.height - CARD.padding / 2, `${where} is too near the card's foot`);
      }
      // What is drawn other than writing: each chart with the room kept for its label, and the heatmap's squares.
      const ops = draw(view, account, theme);
      const squares = ops.filter((op) => op.op === "fill" && op.fill === LOOKS[theme].line && op.alpha !== 0.16).slice(0);
      const drawn = [
        ...chartBoxes(account.windows.length).map((box) => ({ what: "a chart", x0: box.x, y0: box.y, x1: box.x + box.width, y1: box.y + box.height })),
        ...squares.map((square) => ({ what: "a heatmap square", ...square.box })),
      ];
      for (const shape of drawn) {
        assert.ok(shape.x0 >= left - 0.5 && shape.x1 <= right + 0.5, `${account.label}: ${shape.what} is outside the card's padding at the sides`);
        assert.ok(shape.y0 >= CARD.y + CARD.padding / 2 && shape.y1 <= CARD.y + CARD.height - CARD.padding / 2, `${account.label}: ${shape.what} is outside the card's padding above or below`);
        for (const box of boxes) {
          const covers = box.left < shape.x1 - 0.5 && shape.x0 < box.right - 0.5 && box.top < shape.y1 - 0.5 && shape.y0 < box.bottom - 0.5;
          assert.equal(covers, false, `${account.label}: "${box.piece.text}" is written over ${shape.what}`);
        }
      }
      for (const [at, one] of boxes.entries()) {
        for (const other of boxes.slice(at + 1)) {
          const overlap =
            one.left < other.right - 0.5 && other.left < one.right - 0.5 && one.top < other.bottom - 0.5 && other.top < one.bottom - 0.5;
          assert.equal(overlap, false, `${account.label}: "${one.piece.text}" is written over "${other.piece.text}"`);
        }
      }
    }
  }
});

test("a long label is cut short before it reaches the brand, and a short one is whole", () => {
  const long = pro.history[7].accounts[0];
  assert.ok(long.label.length > 50);
  const pieces = texts(draw(pro.history[7], long, "light"));
  const title = pieces.filter((piece) => piece.size === 47);
  const written = title.map((piece) => piece.text).join("");
  assert.ok(written.endsWith("…") && long.label.startsWith(written.slice(0, -1)), written);
  const brand = pieces.find((piece) => piece.text === "QuotaDial");
  const address = pieces.find((piece) => piece.text === "quotadial.com");
  const end = title.at(-1).x + title.at(-1).width;
  // The brand and the address are right-aligned, so the wider of them is the nearer to the label.
  const nearest = CARD.x + CARD.width - CARD.padding - Math.max(brand.width, address.width);
  assert.ok(end <= nearest - 48 + 0.5, "at least 48 px between the label and the brand or the address");
  const [view, short] = ACCOUNTS.find(([, one]) => one.label === "Claude Work");
  assert.equal(texts(draw(view, short, "dark")).filter((piece) => piece.size === 47).map((piece) => piece.text).join(""), "Claude Work");
});

test("what is written is what shareStrings lists, in its order, and an address is never among it", () => {
  for (const theme of ["light", "dark"]) {
    for (const [view, account] of ACCOUNTS) {
      const written = texts(draw(view, account, theme)).map((piece) => piece.text).join("");
      assert.equal(written.includes("@"), false, `${account.label} draws an @`);
      assert.equal(shareStrings(view, account).join("").includes("@"), false, `${account.label} lists an @`);
    }
  }
  // Where nothing is cut short, the pieces joined are the listed strings joined: the website's accounts.
  for (const [view, account] of ACCOUNTS.filter(([view]) => site.history[7] === view)) {
    const written = texts(draw(view, account, "light")).map((piece) => piece.text).join("");
    assert.equal(written, shareStrings(view, account).join(""), account.label);
  }
});

test("a heatmap with no use draws no squares and no words under its caption", () => {
  const unused = structuredClone(withWindows(3));
  unused.heatmap.cells = unused.heatmap.cells.map((row) => row.map(() => 0));
  unused.heatmap.most = 0;
  const ops = draw(pro.history[7], unused, "light");
  assert.ok(texts(ops).some((piece) => piece.text.startsWith("No use in")), "it says there was no use");
  assert.equal(texts(ops).some((piece) => /^(Mon|Tue|Wed|Thu|Fri|Sat|Sun)/.test(piece.text)), false);
  // Its fills are the card and the charts' shading, no heatmap squares.
  assert.equal(ops.filter((op) => op.op === "fill").length, 1 + 3);
});

test("a reading outside 0 to 100 is held to its chart's edges, now that no plate clips it", () => {
  const wild = structuredClone(withWindows(1));
  wild.windows[0].points = [[-0.2, -30], [0.5, 140], [1.4, 50]];
  const ops = draw(pro.history[7], wild, "light");
  const [box] = chartBoxes(1);
  const line = ops.find((op) => op.op === "stroke" && op.stroke === LOOKS.light.line);
  assert.ok(line.box.x0 >= box.x - 0.001 && line.box.x1 <= box.x + box.width + 0.001);
  assert.ok(line.box.y0 >= box.y - 0.001 && line.box.y1 <= box.y + box.height + 0.001);
});

test("the picture's look is read from the stylesheet's own properties in each theme, and never from --background", () => {
  const asked = new Set();
  // What the picture takes from the stylesheet, for each use, written here as the design says it.
  const FROM = {
    text: "--text",
    secondary: "--secondary",
    muted: "--muted",
    warning: "--warning",
    high: "--high",
    card: "--card",
    edge: "--card-edge",
    base: "--chart-base",
    top: "--chart-top",
    line: "--bar-normal",
    limit: "--bar-warning",
  };
  try {
    for (const [theme, colours] of themes()) {
      globalThis.window = { matchMedia: (query) => ({ matches: theme === "dark" && query === "(prefers-color-scheme: dark)" }) };
      globalThis.getComputedStyle = () => ({
        fontFamily: "The window's font, sans-serif",
        getPropertyValue(name) {
          asked.add(name);
          // The browser gives a custom property as written, with the space it was written after.
          return ` ${colours[name] ?? ""}`;
        },
      });
      const look = lookOf({});
      assert.equal(look.theme, theme);
      assert.equal(look.font, "The window's font, sans-serif");
      for (const [use, property] of Object.entries(FROM)) {
        assert.notEqual(colours[property], undefined, `style.css has no ${property}`);
        assert.equal(look[use], colours[property], `${theme}: ${use} is not ${property}`);
      }
      assert.deepEqual(Object.keys(look).sort(), [...Object.keys(FROM), "font", "theme"].sort());
    }
  } finally {
    delete globalThis.window;
    delete globalThis.getComputedStyle;
  }
  assert.deepEqual([...asked].sort(), Object.values(FROM).sort());
  assert.equal(asked.has("--background"), false);
});
