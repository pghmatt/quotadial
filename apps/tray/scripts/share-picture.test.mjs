// The reader of a share picture's pixels and its verdicts (share-picture.mjs, which check-window.mjs
// runs over every picture the window makes). The reader is held to PNGs made here with each of the
// five filters, and the verdicts to pictures made here the way the window draws one and then broken
// on purpose: each verdict passes the right picture in both themes and fails the picture broken
// its way, and only that one (D-137, and the manager on #73, 2026-10-27, 11:26 UTC, for the dashed
// 90% line). The geometry and the readings are the window's own, from src/share.ts, the website's
// made-up accounts and the window check's account of seven windows, and the colours the dashed line
// is judged by are the stylesheet's, read as the window check reads them.
import assert from "node:assert/strict";
import { crc32, deflateSync } from "node:zlib";
import { test } from "node:test";

import { CARD, SHADING, chartBoxes, heatBoxes } from "../src/share.ts";
import { MATERIAL, dashColours, dashPlaces, decodePng, fromHex, heatVerdicts, pictureVerdicts } from "./share-picture.mjs";
import { crowded, pro } from "./window-fixtures.mjs";

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

  /** Lays `[r, g, b]` over the pixel at `x, y` at `alpha`, as a canvas blends what it draws over a pixel, to whole numbers. */
  lay(x, y, colour, alpha) {
    const pixel = this.data.subarray((y * this.width + x) * 4, (y * this.width + x) * 4 + 4);
    pixel.set([...colour, 255].map((channel, at) => Math.round(channel * alpha + pixel[at] * (1 - alpha))));
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

/**
 * A made-up account of the window check's, with its three windows, high, warning and normal. The
 * website's Claude account has two since Claude's windows come from Claude Code's status line
 * (D-145), so the three are the Pro copy's.
 */
const [account] = pro.history[7].accounts;

/**
 * The window check's account of seven windows, over 7 days and over 30, whose pictures it checks:
 * its Weekly window has been full since the middle of the period, so its dashed 90% line runs over
 * the soft fill under its line for half the chart, and the other six cross the card.
 */
const [crowdedWeek] = crowded.history[7].accounts;
const [crowdedMonth] = crowded.history[30].accounts;

/** What a picture of `account` in `theme` must say, as check-window.mjs works it out. */
function wanted(theme, charts = account.windows) {
  const ground = fromHex(MATERIAL[theme]);
  const fill = over(STYLE[theme].fill, ground);
  const { limit, shading } = dashColours(theme, fill);
  return {
    scale: 2,
    card: CARD,
    ground,
    fill,
    edge: over(STYLE[theme].edge, fill),
    limit,
    shading,
    charts: chartBoxes(charts.length).map((box, index) => ({
      label: charts[index].label,
      box,
      points: charts[index].points,
    })),
  };
}

/** How much of `[from, to]` lies between `a` and `b`. */
const overlap = (a, b, from, to) => Math.max(0, Math.min(b, to) - Math.max(a, from));

/** How far `point` is from the line through `line`'s points, or from its one point. */
function fromLine([x, y], line) {
  return Math.min(
    ...line.map(([x0, y0], index) => {
      const [x1, y1] = line[Math.min(index + 1, line.length - 1)];
      const length = (x1 - x0) ** 2 + (y1 - y0) ** 2;
      const t = length === 0 ? 0 : Math.min(1, Math.max(0, ((x - x0) * (x1 - x0) + (y - y0) * (y1 - y0)) / length));
      return Math.hypot(x - (x0 + t * (x1 - x0)), y - (y0 + t * (y1 - y0)));
    }),
  );
}

/** How high `line` is at `x`, between the corners either side of it. */
function heightAt(line, x) {
  const next = line.findIndex(([at]) => at >= x);
  if (next <= 0) {
    return line[Math.max(0, next)][1];
  }
  const [[x0, y0], [x1, y1]] = [line[next - 1], line[next]];
  return x1 === x0 ? Math.min(y0, y1) : y0 + ((y1 - y0) * (x - x0)) / (x1 - x0);
}

/**
 * Draws a chart in `box` with its readings, `points`, as src/share.ts does, with no plate, at two
 * pixels to a CSS pixel: a faint line at 100% and a hairline at 0%; the share used filled softly
 * beneath its line, the line's colour laid at SHADING; the dashed 90% line over that, 4 pixels thick,
 * 12 on and 12 off from the chart's left end, in `limit`; and the line over all, 6 pixels thick with
 * round ends and joins. A pixel any of them covers in part is blended by how much it covers, as a
 * canvas blends an edge. `broken` changes the dashed line, over the card and the soft fill or only
 * `where` it crosses one of them ("card" or "fill"): `dash: false` leaves it out, `pattern` draws it
 * on another pattern, on then off in CSS pixels (`[6, 0]` is solid), `shift` starts the pattern that
 * many pixels further along, and `limit` draws it in another colour; and `fillOver` lays the soft
 * fill after it, as the picture did before #97, so that the fill dulls it.
 */
function drawChart(raster, theme, box, points, card, { limit, line: colour }, broken = {}) {
  const [left, right, top, foot] = [box.x * 2, (box.x + box.width) * 2, box.y * 2, (box.y + box.height) * 2];
  const grey = theme === "light" ? [0, 0, 0] : [255, 255, 255];
  raster.fill(Math.round(left), Math.round(top) - 2, Math.round(right), Math.round(top) + 2, over([...grey, 0.08], card).map(Math.round));
  raster.fill(Math.round(left), Math.round(foot) - 2, Math.round(right), Math.round(foot) + 2, over([...grey, 0.22], card).map(Math.round));
  const held = (value, least, most) => Math.min(most, Math.max(least, value));
  const line = points.map(([along, used]) => [left + held(along, 0, 1) * (right - left), top + ((100 - held(used, 0, 100)) / 100) * (foot - top)]);
  const middle = top + 0.1 * (foot - top);
  const shade = () => {
    if (line.length < 2) {
      return;
    }
    for (let x = Math.floor(line[0][0]); x < Math.ceil(line.at(-1)[0]); x += 1) {
      const across = overlap(x, x + 1, line[0][0], line.at(-1)[0]);
      const height = heightAt(line, x + 0.5);
      for (let y = Math.floor(height); y < Math.ceil(foot); y += 1) {
        raster.lay(x, y, colour, SHADING * across * overlap(y, y + 1, height, foot));
      }
    }
  };
  if (!broken.fillOver) {
    shade();
  }
  // Whether the break is drawn in this column: it is over the soft fill where the line is above the
  // dashed one, and over the card elsewhere.
  const breaks = (x) =>
    broken.where === undefined ||
    broken.where === (line.length > 1 && x + 0.5 > line[0][0] && x + 0.5 < line.at(-1)[0] && heightAt(line, x + 0.5) < middle ? "fill" : "card");
  for (let x = Math.floor(left); x < Math.ceil(right); x += 1) {
    const broke = breaks(x);
    if (broke && broken.dash === false) {
      continue;
    }
    const [on, off] = ((broke && broken.pattern) || [6, 6]).map((length) => length * 2);
    const start = left + ((broke && broken.shift) || 0);
    let across = 0;
    for (let from = start + (Math.floor((x - start) / (on + off)) - 1) * (on + off); from < x + 1; from += on + off) {
      across += overlap(x, x + 1, Math.max(left, from), Math.min(right, from + on));
    }
    for (let y = Math.floor(middle - 2); y < Math.ceil(middle + 2); y += 1) {
      raster.lay(x, y, (broke && broken.limit) || limit, across * overlap(y, y + 1, middle - 2, middle + 2));
    }
  }
  if (broken.fillOver) {
    shade();
  }
  // The line: each pixel within 3 of it is the line's, the last pixel blended, each laid once
  // where two pieces meet. A single reading's ring, 6 across and 6 thick, is drawn as the disc it
  // bounds, 9 each way.
  const reach = line.length === 1 ? 9 : 3;
  const pieces = line.length === 1 ? [[line[0], line[0]]] : line.slice(1).map((end, index) => [line[index], end]);
  const cover = new Map();
  for (const [[x0, y0], [x1, y1]] of pieces) {
    for (let x = Math.floor(Math.min(x0, x1) - reach - 1); x < Math.max(x0, x1) + reach + 1; x += 1) {
      for (let y = Math.floor(Math.min(y0, y1) - reach - 1); y < Math.max(y0, y1) + reach + 1; y += 1) {
        const share = held(reach + 0.5 - fromLine([x + 0.5, y + 0.5], [[x0, y0], [x1, y1]]), 0, 1);
        const at = y * raster.width + x;
        if (share > (cover.get(at) ?? 0)) {
          cover.set(at, share);
        }
      }
    }
  }
  for (const [at, share] of cover) {
    raster.lay(at % raster.width, Math.floor(at / raster.width), colour, share);
  }
}

/**
 * A picture of 2400 by 1350 as the window draws one, written as a PNG and read back: the ground,
 * the card on it with its hairline edge and a chart for each of `charts` on it, and each of the
 * pieces `broken` takes away or changes (and `drawChart` the dashed line's). `stains` are
 * [x, y, colour] marks of 5 pixels square at those device pixels, and `dots` marks of one pixel.
 */
function picture(theme, broken = {}, charts = account.windows) {
  const want = wanted(theme, charts);
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
  const card = round(broken.fill ?? want.fill);
  for (const { box, points } of broken.card === false ? [] : want.charts) {
    drawChart(raster, theme, box, points, card, dashColours(theme, card), broken);
  }
  for (const [x, y, colour] of broken.stains ?? []) {
    raster.fill(Math.max(0, x - 2), Math.max(0, y - 2), Math.min(2400, x + 3), Math.min(1350, y + 3), colour);
  }
  for (const [x, y, colour] of broken.dots ?? []) {
    raster.fill(x, y, x + 1, y + 1, colour);
  }
  return decodePng(png(raster));
}

/** The verdicts a picture of `charts` earns in `theme`, those that are not false. */
function faults(theme, broken = {}, charts = account.windows) {
  return pictureVerdicts(picture(theme, broken, charts), wanted(theme, charts)).filter(Boolean);
}

test("a picture drawn as the window draws it passes every verdict, in both themes", () => {
  for (const theme of ["light", "dark"]) {
    const verdicts = pictureVerdicts(picture(theme), wanted(theme));
    // The corners, the margin, the padding, the edge, and four a chart: its plate, and its dashed
    // line's dashes, gaps and colour.
    assert.equal(verdicts.length, 4 + 4 * account.windows.length);
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

/** The chart a verdict names, by its plate or by its dashed 90% line. */
const chartOf = (verdict) => verdict.match(/behind (.*)'s chart|along (.*)'s 90% line|of (.*)'s 90% line/)?.slice(1).find(Boolean);

test("a plate behind a chart, in either theme, fails for that chart and no other", () => {
  // The plate, and, since it shows in the gaps of the chart's dashed line where the card's fill
  // should, the line's gaps.
  for (const [theme, plate] of [["light", [212, 212, 212, 255]], ["dark", [85, 85, 87, 255]], ["light", [0, 122, 255, 255]]]) {
    const found = faults(theme, { plates: { 1: plate } });
    assert.deepEqual(found.map(chartOf), [account.windows[1].label, account.windows[1].label], `${theme} ${plate}: ${found.join(" | ")}`);
    assert.ok(found[0].includes(`draws a plate behind ${account.windows[1].label}'s chart`), found[0]);
    assert.match(found[1], /in a gap \d+% along .*, where the 6\/6 pattern leaves the card's fill/);
  }
  const both = faults("light", { plates: { 0: [212, 212, 212, 255], 2: [212, 212, 212, 255] } });
  assert.deepEqual(
    both.filter((verdict) => verdict.includes("plate behind")).map(chartOf),
    [account.windows[0].label, account.windows[2].label],
  );
  assert.deepEqual([...new Set(both.map(chartOf))], [account.windows[0].label, account.windows[2].label]);
});

test("a plate over a part of a chart is found wherever along it that part is", () => {
  // A third at the left, a third in the middle and a third at the right, each in turn.
  for (const [from, to] of [[0, 0.34], [0.33, 0.67], [0.66, 1]]) {
    for (const theme of ["light", "dark"]) {
      const found = faults(theme, { partPlates: [[2, from, to, theme === "light" ? [214, 214, 214, 255] : [90, 90, 92, 255]]] });
      assert.deepEqual(found.map(chartOf), [account.windows[2].label, account.windows[2].label], `${theme}, ${from} to ${to}: ${found.join(" | ")}`);
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
  const found = faults("light", {}, full);
  assert.equal(found.length, 1);
  assert.match(found[0], /no empty place on Full's chart/);
});

test("a chart of two readings is looked at between them: a line from or to 100% leaves no empty place, one between 10% and 20% does", () => {
  // The line between two readings reaches the band above 90% if either one is near the top.
  for (const [label, points] of [["Climbing", [[0, 0], [1, 100]]], ["Falling", [[0, 100], [1, 0]]]]) {
    const found = faults("light", {}, [{ label, points }]);
    assert.equal(found.length, 1, label);
    assert.match(found[0], new RegExp(`no empty place on ${label}'s chart`));
  }
  const gentle = [{ label: "Gentle", points: [[0, 10], [1, 20]] }];
  assert.deepEqual(faults("light", {}, gentle), []);
});

test("a chart with no readings is checked for a plate too", () => {
  const empty = [{ label: "Nothing yet", points: [] }];
  assert.deepEqual(faults("dark", {}, empty), []);
  const plated = faults("dark", { plates: { 0: [90, 90, 90, 255] } }, empty);
  assert.match(plated[0], /plate behind Nothing yet's chart/);
});

/** `[r, g, b]` as the verdicts write it, `#rrggbb`, each channel to the nearest whole number. */
const hex = (colour) => `#${colour.map((channel) => Math.round(channel).toString(16).padStart(2, "0")).join("")}`;

/** The labels of the seven windows, in order: a verdict a chart. */
const CROWDED = crowdedWeek.windows.map((window) => window.label);

test("the window check's copy has a chart whose dashed line crosses the soft fill and one whose crosses the card, in what the verdicts read", () => {
  // Each picture the check judges: its accounts over 7 days, and the first over 30.
  for (const [days, charts] of [[7, crowdedWeek.windows], [30, crowdedMonth.windows]]) {
    // A reading above 90% and one below.
    assert.ok(charts.some((chart) => chart.points.some(([, used]) => used > 90)), `${days} days: no reading above 90%`);
    assert.ok(charts.some((chart) => chart.points.some(([, used]) => used < 90)), `${days} days: no reading below 90%`);
    // In what the verdicts read, the dashes and the gaps each judged, in a chart, over the fill and
    // over the card, two or more of each: a dash or a gap is a 12 pixel run of the 6/6 pattern.
    const boxes = chartBoxes(charts.length);
    const runs = charts.map((chart, index) => {
      const { places } = dashPlaces(boxes[index], chart.points, 2);
      const count = (over, dash) =>
        new Set(places.filter((place) => place.over === over && place.dash === dash).map((place) => Math.floor((place.x - boxes[index].x * 2) / 12))).size;
      return { label: chart.label, fill: Math.min(count("fill", true), count("fill", false)), card: Math.min(count("card", true), count("card", false)) };
    });
    assert.ok(runs.some((run) => run.fill >= 2), `${days} days: no chart's dashed line is judged over two dashes and two gaps of the soft fill: ${JSON.stringify(runs)}`);
    assert.ok(runs.some((run) => run.card >= 2), `${days} days: no chart's dashed line is judged over two dashes and two gaps of the card: ${JSON.stringify(runs)}`);
  }
});

test("each chart's dashed 90% line passes, read along the whole of it, over the card and over the soft fill, in both themes", () => {
  for (const charts of [crowdedWeek.windows, crowdedMonth.windows]) {
    for (const theme of ["light", "dark"]) {
      const verdicts = pictureVerdicts(picture(theme, {}, charts), wanted(theme, charts));
      assert.equal(verdicts.length, 4 + 4 * charts.length);
      assert.deepEqual(verdicts.filter(Boolean), [], theme);
    }
  }
});

test("a picture drawn without the dashed line fails, each chart where its first dash should be, and over the soft fill too", () => {
  for (const theme of ["light", "dark"]) {
    const { fill, shading } = wanted(theme, crowdedWeek.windows);
    const found = faults(theme, { dash: false }, crowdedWeek.windows);
    assert.deepEqual(found.map(chartOf), CROWDED, `${theme}: ${found.join(" | ")}`);
    for (const verdict of found) {
      assert.match(verdict, new RegExp(`^has no dash \\d+% along .*'s 90% line, where the 6/6 pattern puts one: #[0-9a-f]{6} there is the card's fill ${hex(fill)}$`));
    }
    // With the dashes left out only where the line is drawn over the fill, the one chart that crosses it fails, there.
    const overFill = faults(theme, { dash: false, where: "fill" }, crowdedWeek.windows);
    assert.equal(overFill.length, 1, `${theme}: ${overFill.join(" | ")}`);
    assert.match(overFill[0], new RegExp(`^has no dash \\d+% along Weekly's 90% line, where the 6/6 pattern puts one: #[0-9a-f]{6} there is the soft fill under its line ${hex(shading)}$`));
  }
});

test("a picture with its 90% line drawn solid fails, each chart in its first gap, and over the soft fill too", () => {
  for (const theme of ["light", "dark"]) {
    const { fill, shading, limit } = wanted(theme, crowdedWeek.windows);
    const found = faults(theme, { pattern: [6, 0] }, crowdedWeek.windows);
    assert.deepEqual(found.map(chartOf), CROWDED, `${theme}: ${found.join(" | ")}`);
    for (const verdict of found) {
      assert.match(verdict, new RegExp(`^has ${hex(limit)} in a gap \\d+% along .*'s 90% line, where the 6/6 pattern leaves the card's fill ${hex(fill)} to show, so the line is not dashed there$`));
    }
    const overFill = faults(theme, { pattern: [6, 0], where: "fill" }, crowdedWeek.windows);
    assert.equal(overFill.length, 1, `${theme}: ${overFill.join(" | ")}`);
    assert.match(overFill[0], new RegExp(`^has ${hex(limit)} in a gap \\d+% along Weekly's 90% line, where the 6/6 pattern leaves the soft fill under its line ${hex(shading)} to show`));
  }
});

test("a picture whose dashed line is #97's old colour, #ff9f0a, fails on the light card and the light fill, and passes in dark, whose colour it still is", () => {
  const old = [255, 159, 10];
  const { fill, shading, limit } = wanted("light", crowdedWeek.windows);
  const found = faults("light", { limit: old }, crowdedWeek.windows);
  assert.deepEqual(found.map(chartOf), CROWDED, found.join(" | "));
  for (const verdict of found) {
    assert.match(verdict, new RegExp(`^draws the dash \\d+% along .*'s 90% line in #ff9f0a over the card's fill ${hex(fill)}, not in --chart-limit ${hex(limit)}$`));
  }
  const overFill = faults("light", { limit: old, where: "fill" }, crowdedWeek.windows);
  assert.equal(overFill.length, 1, overFill.join(" | "));
  assert.match(overFill[0], new RegExp(`^draws the dash \\d+% along Weekly's 90% line in #ff9f0a over the soft fill under its line ${hex(shading)}, not in --chart-limit ${hex(limit)}$`));
  // #97 darkened the light theme's line only; dark's --chart-limit is the old orange still.
  assert.deepEqual(wanted("dark").limit, old);
  assert.deepEqual(faults("dark", { limit: old }, crowdedWeek.windows), []);
});

test("the soft fill laid over the dashed line, as the picture was drawn before #97, fails where a share above 90% dulls the dashes, and there alone", () => {
  for (const theme of ["light", "dark"]) {
    const { shading, limit } = wanted(theme, crowdedWeek.windows);
    const found = faults(theme, { fillOver: true }, crowdedWeek.windows);
    assert.equal(found.length, 1, `${theme}: ${found.join(" | ")}`);
    assert.match(found[0], new RegExp(`^draws the dash \\d+% along Weekly's 90% line in #[0-9a-f]{6} over the soft fill under its line ${hex(shading)}, not in --chart-limit ${hex(limit)}$`));
  }
});

test("a dash a step off --chart-limit fails: it is opaque, so a pixel it covers whole is its colour exactly", () => {
  for (const theme of ["light", "dark"]) {
    const { limit } = wanted(theme, crowdedWeek.windows);
    const step = [limit[0] === 255 ? 254 : limit[0] + 1, limit[1], limit[2]];
    const found = faults(theme, { limit: step }, crowdedWeek.windows);
    assert.deepEqual(found.map(chartOf), CROWDED, `${theme}: ${found.join(" | ")}`);
    assert.ok(found.every((verdict) => verdict.startsWith("draws the dash")), found.join(" | "));
  }
});

test("the dashes are held to the 6/6 pattern from the line's left end, within a pixel of where it puts each end", () => {
  // A pattern a pixel early or late, or dashes a pixel long or short, is anti-aliasing's to blur.
  for (const broken of [{ shift: 1 }, { shift: -1 }, { pattern: [6.5, 5.5] }, { pattern: [5.5, 6.5] }]) {
    assert.deepEqual(faults("light", broken, crowdedWeek.windows), [], JSON.stringify(broken));
  }
  // Two pixels out is not, nor is another pattern: 3/3 (which is 6/6 in the picture's own pixels),
  // 7/5, 8/8, or the gaps where the dashes should be. Each chart fails.
  for (const broken of [{ shift: 2 }, { shift: -2 }, { pattern: [3, 3] }, { pattern: [7, 5] }, { pattern: [8, 8] }, { shift: 12 }]) {
    const found = faults("light", broken, crowdedWeek.windows);
    assert.deepEqual([...new Set(found.map(chartOf))], CROWDED, `${JSON.stringify(broken)}: ${found.join(" | ")}`);
  }
});

test("a mark on the dashed line is found wherever along it the verdicts read, in a dash or a gap, over the card or the fill", () => {
  // Weekly's line climbs to 100% by half way, so its dashed line crosses the card and then the fill.
  const charts = crowdedWeek.windows;
  const { places } = dashPlaces(wanted("light", charts).charts[1].box, charts[1].points, 2);
  for (const over of ["card", "fill"]) {
    for (const dash of [true, false]) {
      const these = places.filter((place) => place.over === over && place.dash === dash);
      // The first of them, the middle one and the last.
      for (const place of [these[0], these[Math.floor(these.length / 2)], these.at(-1)]) {
        const found = faults("light", { dots: [[place.x, place.y, [255, 0, 0]]] }, charts);
        const what = `a mark at ${place.x}, ${place.y}, in a ${dash ? "dash" : "gap"} over the ${over}`;
        assert.equal(found.length, 1, `${what}: ${found.join(" | ")}`);
        assert.equal(chartOf(found[0]), "Weekly", what);
        assert.ok(found[0].startsWith(dash ? "draws the dash" : "has #ff0000 in a gap"), `${what}: ${found[0]}`);
      }
    }
  }
});

test("a chart's own line leaves out only the dashed line's pixels it comes within a pixel of", () => {
  const charts = crowdedWeek.windows;
  const boxes = chartBoxes(charts.length);
  for (const [index, chart] of charts.entries()) {
    const { places, covered, all } = dashPlaces(boxes[index], chart.points, 2);
    // Where the line is drawn, from the readings, as the picture here draws it: each pixel within
    // 3.5 of it, which is how far its last blended pixel reaches.
    const box = boxes[index];
    const line = chart.points.map(([along, used]) => [(box.x + along * box.width) * 2, (box.y + ((100 - used) / 100) * box.height) * 2]);
    const touched = covered.filter(([x, y]) => fromLine([x + 0.5, y + 0.5], line) < 3.5);
    // No pixel the line touches is judged, and every pixel left out is within two of one it touches.
    assert.ok(places.every(({ x, y }) => fromLine([x + 0.5, y + 0.5], line) >= 3.5), chart.label);
    assert.ok(touched.length > 0, `${chart.label}: its line crosses its 90% line`);
    for (const [x, y] of covered) {
      assert.ok(touched.some(([x0, y0]) => Math.abs(x0 - x) <= 2 && Math.abs(y0 - y) <= 2), `${chart.label}: ${x}, ${y} is left out, away from the line`);
    }
    assert.ok(covered.length + places.length <= all);
  }
  // A line that stays under 90% leaves none of it out.
  const [wide] = chartBoxes(1);
  assert.deepEqual(dashPlaces(wide, [[0, 10], [0.5, 60], [1, 20]], 2).covered, []);
});

test("a chart whose own line covers the whole of its 90% line leaves nothing to judge, which fails, saying how much it covers", () => {
  const [wide] = chartBoxes(1);
  const level = [{ label: "Level", points: [[0, 90], [1, 90]] }];
  const { all, covered, places } = dashPlaces(wide, level[0].points, 2);
  assert.deepEqual([covered.length, places.length], [all, 0]);
  // Its line is over 85% all along it, so the verdict on a plate has no empty place to look in either.
  const [plate, ...found] = faults("light", {}, level);
  assert.match(plate, /no empty place on Level's chart/);
  assert.deepEqual(found, [
    `has no dash of Level's 90% line to judge: its own line, drawn over it, covers ${all} of the ${all} pixels the 90% line covers whole`,
    `has no gap of Level's 90% line to judge: its own line, drawn over it, covers ${all} of the ${all} pixels the 90% line covers whole`,
  ]);
  // A single reading on the 90% line is a ring that reaches 9 pixels from it, and covers the line
  // there and no further.
  const middle = (wide.x + 0.5 * wide.width) * 2;
  const dot = dashPlaces(wide, [[0.5, 90]], 2);
  assert.ok(dot.covered.length > 0 && dot.covered.every(([x]) => Math.abs(x + 0.5 - middle) < 9 + 2), `${dot.covered.length} left out`);
  const [none, ...dotted] = faults("dark", {}, [{ label: "Dot", points: [[0.5, 90]] }]);
  assert.match(none, /no empty place on Dot's chart/);
  assert.deepEqual(dotted, []);
});

test("a picture drawn for the other theme fails, whichever way round", () => {
  assert.match(pictureVerdicts(picture("dark"), wanted("light")).find(Boolean), /corner/);
  assert.match(pictureVerdicts(picture("light"), wanted("dark")).find(Boolean), /corner/);
});

/**
 * A picture of the card's fill with `heatmap` drawn on it as the window draws one: a square in each
 * hour's box (`heatBoxes`) with a cell, in the chart line's colour at the alpha its use gives, and
 * none for an hour with no cell (D-144); then a faint square for each of `extra`, `[row, hour]`, and
 * none for each of `skip`.
 */
function heatPicture(theme, heatmap, { extra = [], skip = [] } = {}) {
  const want = wanted(theme);
  const raster = new Raster(2400, 1350);
  raster.fill(0, 0, 2400, 1350, want.fill.map((channel) => Math.round(channel)));
  const { line } = dashColours(theme, want.fill);
  const boxes = heatBoxes(heatmap.cells.length);
  const paint = (row, hour, alpha) => {
    const { x, y, width, height } = boxes[row][hour];
    for (let down = Math.ceil(y * 2); down < Math.floor((y + height) * 2); down += 1) {
      for (let across = Math.ceil(x * 2); across < Math.floor((x + width) * 2); across += 1) {
        raster.lay(across, down, line, alpha);
      }
    }
  };
  heatmap.cells.forEach((hours, row) => {
    hours.forEach((used, hour) => {
      if (used !== null && !skip.some(([at, of]) => at === row && of === hour)) {
        paint(row, hour, used > 0 ? 0.15 + (0.85 * used) / heatmap.most : 0.06);
      }
    });
  });
  for (const [row, hour] of extra) {
    paint(row, hour, 0.06);
  }
  return decodePng(png(raster));
}

/** What a picture's heatmap must hold, as check-window.mjs works it out from `heatmap`. */
const heatWanted = (theme, heatmap) => ({ ...wanted(theme), heat: { days: heatmap.days, cells: heatmap.cells, boxes: heatBoxes(heatmap.cells.length) } });

test("a heatmap drawn with a square for each hour the period holds and none outside it passes, a week and a month, in both themes", () => {
  for (const theme of ["light", "dark"]) {
    for (const heatmap of [account.heatmap, crowdedWeek.heatmap, crowdedMonth.heatmap]) {
      const verdicts = heatVerdicts(heatPicture(theme, heatmap), heatWanted(theme, heatmap));
      assert.deepEqual(verdicts, [false, false], `${theme}, ${heatmap.days.length} rows`);
    }
  }
  // A picture with no heatmap has nothing to hold.
  assert.deepEqual(heatVerdicts(heatPicture("light", account.heatmap), { ...wanted("light"), heat: null }), []);
});

test("a square for an hour outside the period fails, at either end, as one drawn for every hour does", () => {
  const heatmap = crowdedWeek.heatmap;
  for (const theme of ["light", "dark"]) {
    // The first row holds no hour before noon, and the last none from noon.
    const [first] = heatVerdicts(heatPicture(theme, heatmap, { extra: [[0, 3]] }), heatWanted(theme, heatmap));
    assert.match(first, /^draws a square for "Wed 22" at 3:00, an hour the period holds none of: #[0-9a-f]{6} where only the card's fill #[0-9a-f]{6} should be \(1 hour in all\)$/);
    const [last] = heatVerdicts(heatPicture(theme, heatmap, { extra: [[7, 23]] }), heatWanted(theme, heatmap));
    assert.match(last, /draws a square for "Wed 29" at 23:00/);
    // Every hour drawn, as `main`'s picture draws a row: 24 at either end, less those held.
    const every = heatmap.cells.flatMap((hours, row) => hours.flatMap((cell, hour) => (cell === null ? [[row, hour]] : [])));
    const [all] = heatVerdicts(heatPicture(theme, heatmap, { extra: every }), heatWanted(theme, heatmap));
    assert.match(all, /draws a square for "Wed 22" at 0:00, .*\(24 hours in all\)$/);
  }
});

test("a square missing for an hour the period holds fails, however faint the square would be", () => {
  const heatmap = crowdedWeek.heatmap;
  for (const theme of ["light", "dark"]) {
    // Thursday 23's 2:00 had no use: its square is the faintest a heatmap draws.
    assert.equal(heatmap.cells[1][2], 0);
    const verdicts = heatVerdicts(heatPicture(theme, heatmap, { skip: [[1, 2]] }), heatWanted(theme, heatmap));
    assert.deepEqual(verdicts.filter(Boolean), [`draws no square for "Thu 23" at 2:00, an hour the period holds: the card's fill ${verdicts[1].match(/#[0-9a-f]{6}/)[0]} there (1 hour in all)`]);
  }
});
