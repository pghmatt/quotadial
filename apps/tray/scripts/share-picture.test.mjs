// The reader of a share picture's pixels and its verdicts (share-picture.mjs, which check-window.mjs
// runs over every picture the window makes). The reader is held to PNGs made here with each of the
// five filters, and the verdicts to pictures made here the way the window draws one and then broken
// on purpose: each verdict passes the right picture in both themes and fails the picture broken
// its way, and only that one (D-137). The geometry and the readings are the window's own, from
// src/share.ts and the website's made-up accounts.
import assert from "node:assert/strict";
import { crc32, deflateSync } from "node:zlib";
import { test } from "node:test";

import { CARD, chartBoxes } from "../src/share.ts";
import { MATERIAL, decodePng, fromHex, pictureVerdicts } from "./share-picture.mjs";
import { site } from "./site-fixtures.mjs";

/** A picture of `width` by `height` pixels to paint on and write as a PNG. */
class Raster {
  constructor(width, height) {
    this.width = width;
    this.height = height;
    this.data = new Uint8Array(width * height * 4);
    // The same pixels a 32-bit word each, so a row is filled at once.
    this.words = new Uint32Array(this.data.buffer);
  }

  /** Fills the pixels from `x0, y0` up to `x1, y1` with `[r, g, b, a]`. */
  fill(x0, y0, x1, y1, [r, g, b, a = 255]) {
    // The word that holds the four bytes in memory's order, whichever order that is.
    const [word] = new Uint32Array(new Uint8Array([r, g, b, a]).buffer);
    for (let y = y0; y < y1; y += 1) {
      this.words.fill(word, y * this.width + x0, y * this.width + x1);
    }
  }
}

/** What each filter predicts from the byte at the left, above and above left, written as PNG's
 * specification words it. */
function predicted(filter, left, up, upLeft) {
  if (filter === 0) return 0;
  if (filter === 1) return left;
  if (filter === 2) return up;
  if (filter === 3) return Math.floor((left + up) / 2);
  const p = left + up - upLeft;
  const [pa, pb, pc] = [Math.abs(p - left), Math.abs(p - up), Math.abs(p - upLeft)];
  if (pa <= pb && pa <= pc) return left;
  if (pb <= pc) return up;
  return upLeft;
}

/** A PNG chunk: its length, type, body and checksum. */
function chunk(type, body) {
  const typed = Buffer.concat([Buffer.from(type, "latin1"), body]);
  const length = Buffer.alloc(4);
  length.writeUInt32BE(body.length);
  const checksum = Buffer.alloc(4);
  checksum.writeUInt32BE(crc32(typed));
  return Buffer.concat([length, typed, checksum]);
}

/**
 * `raster` as a PNG of 8-bit RGBA (or RGB, with `colour: 2`), its rows filtered in turn by each of
 * `filters`, and its header with `header` laid over it.
 */
function png(raster, { colour = 6, filters = [0], header = {} } = {}) {
  const each = colour === 6 ? 4 : 3;
  const rows = [];
  for (let row = 0; row < raster.height; row += 1) {
    const line = Buffer.alloc(raster.width * each);
    for (let x = 0; x < raster.width; x += 1) {
      for (let channel = 0; channel < each; channel += 1) {
        line[x * each + channel] = raster.data[(row * raster.width + x) * 4 + channel];
      }
    }
    rows.push(line);
  }
  const lines = rows.map((line, row) => {
    const filter = filters[row % filters.length];
    const out = Buffer.alloc(line.length + 1);
    out[0] = filter;
    for (let at = 0; at < line.length; at += 1) {
      const left = at >= each ? line[at - each] : 0;
      const up = row > 0 ? rows[row - 1][at] : 0;
      const upLeft = row > 0 && at >= each ? rows[row - 1][at - each] : 0;
      out[at + 1] = (line[at] - predicted(filter, left, up, upLeft)) & 255;
    }
    return out;
  });
  const top = Buffer.alloc(13);
  top.writeUInt32BE(raster.width, 0);
  top.writeUInt32BE(raster.height, 4);
  top.set([8, colour, 0, 0, 0], 8);
  for (const [at, value] of Object.entries(header)) {
    top[Number(at)] = value;
  }
  return Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    chunk("IHDR", top),
    chunk("IDAT", deflateSync(Buffer.concat(lines))),
    chunk("IEND", Buffer.alloc(0)),
  ]);
}

/**
 * A small raster of pixels that differ from their neighbours, so a wrong filter shows: a gradient
 * that rises along a row and falls down a column, which makes Paeth choose the pixel above and to
 * the left, gradients that rise both ways, and noise from a small generator with a fixed seed.
 */
function varied(width, height) {
  const raster = new Raster(width, height);
  let state = 12345;
  const noise = () => {
    state = (Math.imul(state, 1103515245) + 12345) >>> 0;
    return state >>> 24;
  };
  for (let y = 0; y < height; y += 1) {
    for (let x = 0; x < width; x += 1) {
      raster.data.set(
        [(x * 37 - y * 29) & 255, noise(), (x * x + y * 3) & 255, 255 - ((x + y * 13) & 127)],
        (y * width + x) * 4,
      );
    }
  }
  return raster;
}

test("a PNG is read back pixel for pixel, whichever filter its rows have", () => {
  const raster = varied(13, 9);
  for (const filters of [[0], [1], [2], [3], [4], [0, 1, 2, 3, 4], [4, 3, 2, 1, 0, 4]]) {
    const read = decodePng(png(raster, { filters }));
    assert.equal([read.width, read.height].join("x"), "13x9");
    for (let y = 0; y < raster.height; y += 1) {
      for (let x = 0; x < raster.width; x += 1) {
        const at = (y * raster.width + x) * 4;
        assert.deepEqual(read.pixel(x, y), [...raster.data.subarray(at, at + 4)], `filters ${filters}, pixel ${x}, ${y}`);
      }
    }
  }
});

test("an RGB PNG is read as opaque, and one pixel wide or high is a PNG too", () => {
  const rgb = decodePng(png(varied(7, 5), { colour: 2, filters: [1, 2, 3, 4, 0] }));
  assert.equal(rgb.pixel(3, 2)[3], 255);
  assert.deepEqual(rgb.pixel(0, 0).slice(0, 3), [...varied(7, 5).data.subarray(0, 3)]);
  for (const [width, height] of [[1, 1], [1, 6], [6, 1]]) {
    const raster = varied(width, height);
    const read = decodePng(png(raster, { filters: [4, 3, 2, 1] }));
    assert.deepEqual(read.pixel(width - 1, height - 1), [...raster.data.subarray(-4)]);
  }
});

test("a pixel that is not in the picture is refused, not read from beside it", () => {
  const read = decodePng(png(varied(4, 3)));
  for (const [x, y] of [[4, 0], [0, 3], [-1, 0], [0, -1], [0.5, 0], [Number.NaN, 0]]) {
    assert.throws(() => read.pixel(x, y), RangeError, `${x}, ${y}`);
  }
});

test("what is not a whole PNG of 8-bit RGB or RGBA is refused, saying why", () => {
  const good = png(varied(6, 4), { filters: [0, 4] });
  const refused = (bytes, words) => assert.throws(() => decodePng(bytes), (error) => error instanceof Error && words.test(error.message), String(words));
  refused(Buffer.from("not a picture at all"), /does not begin as a PNG/);
  refused(Buffer.alloc(0), /does not begin as a PNG/);
  refused(good.subarray(0, good.length - 12), /ends before its IEND/);
  refused(png(varied(6, 4), { header: { 8: 16 } }), /16-bit/);
  refused(png(varied(6, 4), { header: { 9: 3 } }), /colour type 3/);
  refused(png(varied(6, 4), { header: { 12: 1 } }), /interlace 1/);
  // A header that says the picture is taller than the pixels that follow it.
  const taller = Buffer.from(good);
  taller.writeUInt32BE(5, 20);
  taller.writeUInt32BE(crc32(taller.subarray(12, 29)), 29);
  refused(taller, /its pixels are .* bytes, not the/);
  // A byte changed inside a chunk no longer matches the checksum that ends it.
  const damaged = Buffer.from(good);
  damaged[good.length - 20] ^= 1;
  refused(damaged, /checksum of its IDAT chunk is wrong/);
  // A row that names a filter PNG does not have.
  refused(png(varied(6, 4), { filters: [5] }), /filter 5/);
  // A header that asks for more pixels than a picture here may have.
  const huge = Buffer.from(good);
  huge.writeUInt32BE(2 ** 30, 16);
  huge.writeUInt32BE(crc32(huge.subarray(12, 29)), 29);
  refused(huge, /will not read/);
});

test("a PNG cut short anywhere, or changed at any byte, is refused rather than misread", () => {
  const good = png(varied(5, 4), { filters: [0, 1, 2, 3, 4] });
  for (let length = 0; length < good.length; length += 1) {
    assert.throws(() => decodePng(good.subarray(0, length)), Error, `cut at ${length} bytes`);
  }
  for (let at = 0; at < good.length; at += 1) {
    const changed = Buffer.from(good);
    changed[at] ^= 0x55;
    assert.throws(() => decodePng(changed), Error, `byte ${at} changed`);
  }
});

/** `top` as `[r, g, b, a]` laid over the opaque `under`, as the canvas composites it. */
function over([r, g, b, a], under) {
  return [r, g, b].map((channel, at) => channel * a + under[at] * (1 - a));
}

/** The card's fill and edge in each theme, as the stylesheet gives them (D-132). */
const STYLE = {
  light: { fill: [255, 255, 255, 0.7], edge: [0, 0, 0, 0.07] },
  dark: { fill: [255, 255, 255, 0.06], edge: [255, 255, 255, 0] },
};

/** A made-up account of the website's, with its three windows. */
const [account] = site.history[7].accounts;

/** What a picture of `account` in `theme` must say, as check-window.mjs works it out. */
function wanted(theme, charts = account.windows) {
  const ground = fromHex(MATERIAL[theme]);
  const fill = over(STYLE[theme].fill, ground);
  return {
    scale: 2,
    card: CARD,
    ground,
    fill,
    edge: over(STYLE[theme].edge, fill),
    charts: chartBoxes(charts.length).map((box, index) => ({
      label: charts[index].label,
      box,
      points: charts[index].points,
    })),
  };
}

/** A chart's share used at `along` (0 to 1), between the readings either side of it. */
function usedAlong(points, along) {
  const next = points.findIndex(([at]) => at >= along);
  if (next <= 0) {
    return points[Math.max(0, next)][1];
  }
  const [[from, before], [to, after]] = [points[next - 1], points[next]];
  return before + ((after - before) * (along - from)) / (to - from);
}

/**
 * Draws `chart` in `box` as the window does, with no plate: a faint line at 100%, a hairline at 0%,
 * a dashed line at 90%, the share used filled softly beneath its line, and the line, 3 px thick.
 */
function drawChart(raster, theme, box, points, fill) {
  const device = (css) => Math.round(css * 2);
  const [left, right, top, bottom] = [device(box.x), device(box.x + box.width), device(box.y), device(box.y + box.height)];
  const row = (used) => top + Math.round(((100 - used) / 100) * (bottom - top));
  const grey = theme === "light" ? [0, 0, 0] : [255, 255, 255];
  raster.fill(left, top - 2, right, top + 2, over([...grey, 0.08], fill).map(Math.round));
  raster.fill(left, bottom - 2, right, bottom + 2, over([...grey, 0.22], fill).map(Math.round));
  for (let x = left; x < right; x += 12) {
    raster.fill(x, row(90) - 2, Math.min(right, x + 12 - 6), row(90) + 2, [255, 159, 10]);
  }
  const blue = theme === "light" ? [0, 122, 255] : [10, 132, 255];
  for (let x = left; x < right; x += 1) {
    const at = row(usedAlong(points, (x / 2 - box.x) / box.width));
    raster.fill(x, at, x + 1, bottom, over([...blue, 0.16], fill).map(Math.round));
    raster.fill(x, at - 3, x + 1, at + 3, blue);
  }
}

/**
 * A picture of 2400 by 1350 as the window draws one, written as a PNG and read back: the ground,
 * the card on it with its hairline edge and each chart on it, and each of the pieces `broken` takes
 * away or changes. `stains` are [x, y, colour] marks of 5 pixels square at those device pixels.
 */
function picture(theme, broken = {}) {
  const want = wanted(theme);
  const round = (colour) => colour.map((channel) => Math.round(channel));
  const raster = new Raster(2400, 1350);
  const ground = broken.ground === undefined ? want.ground : broken.ground;
  if (ground !== null) {
    raster.fill(0, 0, 2400, 1350, ground);
  }
  const [x, y] = [CARD.x * 2, CARD.y * 2];
  const [x1, y1] = [x + CARD.width * 2, y + CARD.height * 2];
  const inset = broken.margin ?? 0;
  if (broken.card !== false) {
    // A card to the picture's edges still has rounded corners, so the corners stay the ground.
    raster.fill(x - inset, y - inset, x1 + inset, y1 + inset, round(broken.fill ?? want.fill));
    if (inset > 0) {
      for (const [cx, cy] of [[0, 0], [2400 - 66, 0], [0, 1350 - 66], [2400 - 66, 1350 - 66]]) {
        raster.fill(cx, cy, cx + 66, cy + 66, want.ground);
      }
    }
    if (broken.edge !== false) {
      const ring = Math.round(CARD.edge * 2);
      const edge = round(want.edge);
      raster.fill(x, y, x1, y + ring, edge);
      raster.fill(x, y1 - ring, x1, y1, edge);
      raster.fill(x, y, x + ring, y1, edge);
      raster.fill(x1 - ring, y, x1, y1, edge);
    }
  }
  for (const [at, colour] of Object.entries(broken.plates ?? {})) {
    const { box } = want.charts[Number(at)];
    raster.fill(Math.round(box.x * 2), Math.round(box.y * 2), Math.round((box.x + box.width) * 2), Math.round((box.y + box.height) * 2), colour);
  }
  // A plate over only the part of a chart from `from` to `to` along it.
  for (const [at, from, to, colour] of broken.partPlates ?? []) {
    const { box } = want.charts[at];
    raster.fill(Math.round((box.x + from * box.width) * 2), Math.round(box.y * 2), Math.round((box.x + to * box.width) * 2), Math.round((box.y + box.height) * 2), colour);
  }
  // Each chart's lines and the share used are drawn over any plate, as they are over the card.
  for (const { box, points } of broken.card === false ? [] : want.charts) {
    drawChart(raster, theme, box, points, round(broken.fill ?? want.fill));
  }
  for (const [x, y, colour] of broken.stains ?? []) {
    raster.fill(Math.max(0, x - 2), Math.max(0, y - 2), Math.min(2400, x + 3), Math.min(1350, y + 3), colour);
  }
  return decodePng(png(raster));
}

/** The verdicts a picture earns in `theme`, those that are not false. */
function faults(theme, broken = {}, charts = account.windows) {
  return pictureVerdicts(picture(theme, broken), wanted(theme, charts)).filter(Boolean);
}

test("a picture drawn as the window draws it passes every verdict, in both themes", () => {
  for (const theme of ["light", "dark"]) {
    const verdicts = pictureVerdicts(picture(theme), wanted(theme));
    // The corners, the margin, the padding, the edge, and a chart each.
    assert.equal(verdicts.length, 4 + account.windows.length);
    assert.deepEqual(verdicts.filter(Boolean), [], theme);
  }
});

test("a ground that is not the theme's material fails at the corners, whatever the card is", () => {
  for (const [theme, ground] of [["light", [255, 255, 255]], ["light", [251, 251, 253]], ["dark", [31, 31, 33]], ["dark", [236, 236, 236]], ["light", [44, 44, 46]]]) {
    const found = faults(theme, { ground });
    assert.equal(found.length >= 1, true, `${theme} ground ${ground}`);
    assert.match(found[0], /corner/, `${theme} ground ${ground}`);
    assert.match(found[0], new RegExp(`not the material ${MATERIAL[theme]}`));
  }
  // A ground one step off is as wrong as one far off: it is a choice, not a nearby grey.
  assert.match(faults("light", { ground: [237, 236, 236] })[0], /corner/);
});

test("a picture with no ground, left clear, fails at the corners", () => {
  const found = faults("dark", { ground: null });
  assert.match(found[0], /has #00000000 at its top left corner/);
});

test("a ground of the material's colour that is not opaque is not the material", () => {
  for (const [theme, ground] of [["light", [236, 236, 236, 0]], ["dark", [44, 44, 46, 254]], ["light", [236, 236, 236, 128]]]) {
    assert.match(faults(theme, { ground })[0], /corner/, `${theme} ${ground}`);
  }
});

/** Every pixel a verdict reads, as device pixels, and the words its verdict says when it is not right. */
const SPOTS = [
  ["top left corner", 0, 0],
  ["top right corner", 2399, 0],
  ["bottom left corner", 0, 1349],
  ["bottom right corner", 2399, 1349],
  ["left margin", 40, 675],
  ["right margin", 2360, 675],
  ["top margin", 1200, 40],
  ["bottom margin", 1200, 1310],
  ["left padding", 120, 675],
  ["right padding", 2280, 675],
  ["top padding", 1200, 120],
  ["bottom padding", 1200, 1230],
  ["left edge", 81, 675],
  ["top edge", 1200, 81],
];

test("every place a verdict reads is read: a mark on any one of them is found, by name", () => {
  for (const theme of ["light", "dark"]) {
    for (const [name, x, y] of SPOTS) {
      // The dark card's edge is clear, so a mark on it is a mark on the fill, which is also what is wanted there.
      const found = faults(theme, { stains: [[x, y, [255, 0, 0]]] });
      assert.equal(found.length, 1, `${theme}: a mark on the ${name} found ${found.length} faults: ${found.join(" | ")}`);
      assert.ok(found[0].includes(name), `${theme}: a mark on the ${name} was reported as: ${found[0]}`);
    }
  }
});

test("a card with no margin, drawn out to the picture's edges, fails where the ground should show", () => {
  const found = faults("light", { margin: 80 });
  // The corners are still the ground, since the card's corners are rounded; the margin is not.
  assert.doesNotMatch(found.join("\n"), /corner/);
  assert.match(found[0], /margin.*so the card does not sit on it/);
});

test("a card with no room round it at all is a verdict that names the edge, not a check that stops", () => {
  const bare = { ...CARD, x: 0, y: 0, width: 1200, height: 675 };
  for (const [theme, side] of [["light", "left"], ["dark", "left"]]) {
    const found = pictureVerdicts(picture(theme), { ...wanted(theme), card: bare }).filter(Boolean);
    assert.ok(found.some((verdict) => verdict.includes(`0 px from its ${side} edge, with no margin for the material to show in`)), found.join(" | "));
  }
  // Each edge in turn: a card 6 px from it is too near for the material to show, one 10 px from it is not.
  for (const [side, close, apart] of [
    ["left", { x: 6, width: 1114 }, { x: 10, width: 1110 }],
    ["right", { width: 1154 }, { width: 1150 }],
    ["top", { y: 6, height: 629 }, { y: 10, height: 625 }],
    ["bottom", { height: 629 }, { height: 625 }],
  ]) {
    // A card of this geometry is not the picture's, so it is the verdict on the margin that is read.
    const [narrow] = pictureVerdicts(picture("light"), { ...wanted("light"), card: { ...CARD, ...close } }).slice(1, 2);
    assert.match(narrow, new RegExp(`px from its ${side} edge, with no margin`), side);
    const [roomy] = pictureVerdicts(picture("light"), { ...wanted("light"), card: { ...CARD, ...apart } }).slice(1, 2);
    assert.doesNotMatch(String(roomy), /with no margin/, side);
  }
});

test("a picture with no card, the ground only, fails in the card's padding", () => {
  for (const theme of ["light", "dark"]) {
    const found = faults(theme, { card: false });
    assert.equal(found.some((verdict) => /card's left padding.*not the card's fill/.test(verdict)), true, theme);
    assert.doesNotMatch(found.join("\n"), /corner|margin/);
  }
});

test("a card that is the wrong fill fails, and one a blend's rounding off does not", () => {
  const { fill } = wanted("light");
  assert.deepEqual(faults("light", { fill: fill.map((channel) => channel + 2) }), []);
  assert.match(faults("light", { fill: fill.map((channel) => channel - 8) })[0], /padding/);
  // The card of the other theme is the wrong fill for this one.
  assert.match(faults("dark", { fill: wanted("light").fill })[0], /padding/);
  assert.match(faults("light", { fill: [255, 255, 255] })[0], /padding/);
});

test("the hairline edge is there in light, and in dark, where it is clear, is not needed", () => {
  const found = faults("light", { edge: false });
  assert.equal(found.length, 1);
  assert.match(found[0], /card's left edge.*not its hairline/);
  assert.deepEqual(faults("dark", { edge: false }), []);
});

test("a plate behind a chart, in either theme, fails for that chart and no other", () => {
  for (const [theme, plate] of [["light", [212, 212, 212, 255]], ["dark", [85, 85, 87, 255]], ["light", [0, 122, 255, 255]]]) {
    const found = faults(theme, { plates: { 1: plate } });
    assert.equal(found.length, 1, `${theme} ${plate}`);
    assert.ok(found[0].includes(`draws a plate behind ${account.windows[1].label}'s chart`), found[0]);
  }
  const both = faults("light", { plates: { 0: [212, 212, 212, 255], 2: [212, 212, 212, 255] } });
  assert.deepEqual(
    both.map((verdict) => verdict.match(/behind (.*)'s chart/)[1]),
    [account.windows[0].label, account.windows[2].label],
  );
});

test("a plate over a part of a chart is found wherever along it that part is", () => {
  // A third at the left, a third in the middle and a third at the right, each in turn.
  for (const [from, to] of [[0, 0.34], [0.33, 0.67], [0.66, 1]]) {
    for (const theme of ["light", "dark"]) {
      const found = faults(theme, { partPlates: [[2, from, to, theme === "light" ? [214, 214, 214, 255] : [90, 90, 92, 255]]] });
      assert.equal(found.length, 1, `${theme}, ${from} to ${to}: ${found.join(" | ")}`);
      assert.ok(found[0].includes(`draws a plate behind ${account.windows[2].label}'s chart`), found[0]);
    }
  }
});

test("a plate is found along the whole chart, not only where the line is low", () => {
  // The first window climbs to 91%: the plate must be seen at its left, where the line is near 0%.
  const found = faults("light", { plates: { 0: [200, 200, 200, 255] } });
  assert.match(found[0], /plate behind Weekly's chart.*% along/);
});

test("a chart whose line is high all along it has no empty place to look, and says so", () => {
  const full = [{ label: "Full", points: [[0, 100], [0.5, 99], [1, 100]] }];
  const found = pictureVerdicts(picture("light"), wanted("light", full)).filter(Boolean);
  assert.equal(found.length, 1);
  assert.match(found[0], /no empty place on Full's chart/);
});

test("a chart of two readings is looked at between them: a line from or to 100% leaves no empty place, one between 10% and 20% does", () => {
  // The line between two readings reaches the band above 90% if either one is near the top.
  for (const [label, points] of [["Climbing", [[0, 0], [1, 100]]], ["Falling", [[0, 100], [1, 0]]]]) {
    const found = pictureVerdicts(picture("light"), wanted("light", [{ label, points }])).filter(Boolean);
    assert.equal(found.length, 1, label);
    assert.match(found[0], new RegExp(`no empty place on ${label}'s chart`));
  }
  const gentle = [{ label: "Gentle", points: [[0, 10], [1, 20]] }];
  assert.deepEqual(pictureVerdicts(picture("light"), wanted("light", gentle)).filter(Boolean), []);
});

test("a chart with no readings is checked for a plate too", () => {
  const empty = [{ label: "Nothing yet", points: [] }];
  assert.deepEqual(pictureVerdicts(picture("dark"), wanted("dark", empty)).filter(Boolean), []);
  const plated = pictureVerdicts(picture("dark", { plates: { 0: [90, 90, 90, 255] } }), wanted("dark", empty)).filter(Boolean);
  assert.match(plated[0], /plate behind Nothing yet's chart/);
});

test("a picture drawn for the other theme fails, whichever way round", () => {
  assert.match(pictureVerdicts(picture("dark"), wanted("light")).find(Boolean), /corner/);
  assert.match(pictureVerdicts(picture("light"), wanted("dark")).find(Boolean), /corner/);
});
