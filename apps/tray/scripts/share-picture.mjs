// What a share picture's pixels must say (D-137), read from the PNG the window sends the app, as
// the person who posts it gets it: the ground is the material of the theme, the card sits on it,
// nothing is drawn as a plate behind a chart, each chart's 90% line is dashed in its own colour
// along the whole of it (the manager on #73, 2026-10-27, 11:26 UTC), and the heatmap draws no square
// for an hour the period holds none of (D-144). check-window.mjs runs these
// over every picture it makes; share-picture.test.mjs proves each one fails on a picture broken on
// purpose, with no browser. The PNG is read here with no library: a signature, chunks with their
// checksums, one zlib stream, and the five filters of a scanline.
import { crc32, inflateSync } from "node:zlib";

import { SHADING } from "../src/share.ts";
import { themes } from "./stylesheet.mjs";

/**
 * The picture's ground in each theme, as the manager decided it on #73 (2026-10-26, 23:25 UTC) and
 * D-137 records it: the macOS popover material's colour, which the window's contrast check also
 * takes for the material. Written here as the check's own text of the decision, not read from
 * src/share.ts or the stylesheet, so a ground changed in either fails the check instead of being
 * followed by it.
 */
export const MATERIAL = { light: "#ececec", dark: "#2c2c2e" };

/** `#rrggbb` as `[r, g, b]`. */
export function fromHex(hex) {
  const found = hex.match(/^#([0-9a-f]{6})$/i);
  if (!found) {
    throw new Error(`${hex} is not a colour written #rrggbb`);
  }
  const number = Number.parseInt(found[1], 16);
  return [(number >> 16) & 255, (number >> 8) & 255, number & 255];
}

/** The PNG's predictor for a pixel from its neighbours at the left, above and above left. */
function paeth(left, up, upLeft) {
  const estimate = left + up - upLeft;
  const [fromLeft, fromUp, fromUpLeft] = [left, up, upLeft].map((near) => Math.abs(estimate - near));
  if (fromLeft <= fromUp && fromLeft <= fromUpLeft) {
    return left;
  }
  return fromUp <= fromUpLeft ? up : upLeft;
}

/**
 * Undoes one scanline's filter: `line` holds the filtered bytes, and the row begins at `at` in
 * `raw`, which holds the rows above it, `stride` bytes each, `each` bytes to a pixel.
 */
function unfilter(filter, line, raw, at, stride, each, row) {
  const left = (index) => (index >= each ? raw[at + index - each] : 0);
  const up = (index) => (row > 0 ? raw[at - stride + index] : 0);
  const upLeft = (index) => (row > 0 && index >= each ? raw[at - stride + index - each] : 0);
  // A Uint8Array keeps the low eight bits of what it is given, which is PNG's arithmetic.
  switch (filter) {
    case 0:
      raw.set(line, at);
      break;
    case 1:
      for (let index = 0; index < stride; index += 1) {
        raw[at + index] = line[index] + left(index);
      }
      break;
    case 2:
      for (let index = 0; index < stride; index += 1) {
        raw[at + index] = line[index] + up(index);
      }
      break;
    case 3:
      for (let index = 0; index < stride; index += 1) {
        raw[at + index] = line[index] + ((left(index) + up(index)) >> 1);
      }
      break;
    case 4:
      for (let index = 0; index < stride; index += 1) {
        raw[at + index] = line[index] + paeth(left(index), up(index), upLeft(index));
      }
      break;
    default:
      throw new Error(`row ${row} has the filter ${filter}, which PNG does not have`);
  }
}

/** The most pixels a picture may have here, so a damaged header cannot ask for all the memory. */
const MOST_PIXELS = 2 ** 26;

/**
 * Reads a PNG of 8-bit RGB or RGBA, not interlaced, as Chrome writes a canvas: its width and
 * height, and a `pixel(x, y)` that answers `[r, g, b, a]`. Throws an Error that says what it cannot
 * read, in a sentence that fits after "its pixels cannot be read: ".
 */
export function decodePng(bytes) {
  const data = Buffer.isBuffer(bytes) ? bytes : Buffer.from(bytes);
  const signature = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
  if (data.length < signature.length || signature.some((byte, at) => data[at] !== byte)) {
    throw new Error("it does not begin as a PNG");
  }
  let header = null;
  const compressed = [];
  for (let at = signature.length, ended = false; !ended; ) {
    if (at + 12 > data.length) {
      throw new Error("it ends before its IEND chunk");
    }
    const length = data.readUInt32BE(at);
    const type = data.toString("latin1", at + 4, at + 8);
    if (at + 12 + length > data.length) {
      throw new Error(`its ${type} chunk runs past the end`);
    }
    const typed = data.subarray(at + 4, at + 8 + length);
    if (data.readUInt32BE(at + 8 + length) !== crc32(typed)) {
      throw new Error(`the checksum of its ${type} chunk is wrong`);
    }
    const body = typed.subarray(4);
    if (type === "IHDR") {
      if (length !== 13) {
        throw new Error("its header is not 13 bytes");
      }
      header = {
        width: body.readUInt32BE(0),
        height: body.readUInt32BE(4),
        depth: body[8],
        colour: body[9],
        interlace: body[12],
      };
    } else if (type === "IDAT") {
      compressed.push(body);
    } else if (type === "IEND") {
      ended = true;
    }
    at += 12 + length;
  }
  if (!header) {
    throw new Error("it has no header");
  }
  const { width, height, depth, colour, interlace } = header;
  if (width === 0 || height === 0 || width * height > MOST_PIXELS) {
    throw new Error(`it is ${width} by ${height}, which it will not read`);
  }
  if (depth !== 8 || (colour !== 2 && colour !== 6) || interlace !== 0) {
    throw new Error(
      `it is ${depth}-bit, colour type ${colour}, interlace ${interlace}, not 8-bit RGB or RGBA in one pass`,
    );
  }
  const each = colour === 6 ? 4 : 3;
  const stride = width * each;
  const inflated = inflateSync(Buffer.concat(compressed));
  if (inflated.length !== height * (stride + 1)) {
    throw new Error(`its pixels are ${inflated.length} bytes, not the ${height * (stride + 1)} its header gives`);
  }
  const raw = new Uint8Array(height * stride);
  for (let row = 0; row < height; row += 1) {
    const start = row * (stride + 1);
    unfilter(inflated[start], inflated.subarray(start + 1, start + 1 + stride), raw, row * stride, stride, each, row);
  }
  return {
    width,
    height,
    pixel(x, y) {
      if (!Number.isInteger(x) || !Number.isInteger(y) || x < 0 || y < 0 || x >= width || y >= height) {
        throw new RangeError(`there is no pixel at ${x}, ${y} in a picture of ${width} by ${height}`);
      }
      const at = y * stride + x * each;
      return [raw[at], raw[at + 1], raw[at + 2], each === 4 ? raw[at + 3] : 255];
    },
  };
}

/** `[r, g, b, a]` or `[r, g, b]` as `#rrggbb`, or `#rrggbbaa` when it is not opaque. */
function show(colour) {
  const [r, g, b, a = 255] = colour.map((channel) => Math.round(channel));
  const hex = (channel) => channel.toString(16).padStart(2, "0");
  return `#${hex(r)}${hex(g)}${hex(b)}${a === 255 ? "" : hex(a)}`;
}

/** Whether `found` is opaque and within `tolerance` of `wanted` in every channel. */
function near(found, wanted, tolerance) {
  return found[3] === 255 && wanted.every((channel, at) => Math.abs(found[at] - channel) <= tolerance);
}

/** The first of `samples`, each `[where, pixel]`, that is not `wanted`, or null when all are. */
function firstWrong(samples, wanted, tolerance) {
  return samples.find(([, found]) => !near(found, wanted, tolerance)) ?? null;
}

/** How much a colour the canvas composites may differ from the one worked out in full. */
const BLEND = 2;

/** `[r, g, b, a]` laid over the opaque `under`, as the canvas composites it, worked out in full. */
function over([r, g, b, a], under) {
  return [r, g, b].map((channel, at) => channel * a + under[at] * (1 - a));
}

/**
 * The colours a chart's dashed 90% line is judged by in `theme`, over the card's `fill`: `limit`,
 * the line's own, `--chart-limit`; `line`, a chart's own line, `--bar-normal`; and `shading`, the
 * soft fill under a chart's line, `--bar-normal` at SHADING laid over `fill`. They are read from
 * src/style.css and src/share.ts as check-contrast.mjs reads them, and not written here as MATERIAL
 * is: the contrast check holds each of them to its ratio, so a colour or SHADING moved there is
 * judged by that one check while this one follows it, rather than two copies of one number agreeing
 * with each other (the manager on #73, 2026-10-27, 11:26 UTC).
 */
export function dashColours(theme, fill) {
  const stylesheet = new Map(themes()).get(theme);
  const colour = (name) => {
    if (stylesheet?.[name] === undefined) {
      throw new Error(`style.css does not define ${name} in the ${theme} theme`);
    }
    return fromHex(stylesheet[name]);
  };
  const line = colour("--bar-normal");
  return { limit: colour("--chart-limit"), line, shading: over([...line, SHADING], fill) };
}

/**
 * A chart's dashed 90% line as src/share.ts draws it, in CSS pixels, written here as the check's
 * own text of what the picture draws, as MATERIAL is: at LIMIT_PCT of the chart's height up from its
 * foot, LIMIT_WIDTH thick (`RULE`), dashed LIMIT_DASH, on then off from its left end
 * (`setLineDash([6, 6])`, the manager's 6/6), as the canvas begins a pattern where its line begins.
 * Chrome's pictures show it so: a chart that begins 233.71 CSS pixels in begins with a dash from
 * device pixel 467.43 to 479.43, which covers pixels 468 to 478 whole and 467 and 479 in part.
 */
const LIMIT_PCT = 90;
const LIMIT_WIDTH = 2;
const LIMIT_DASH = [6, 6];
/**
 * A chart's own line, drawn over its dashed one, in CSS pixels: LINE_WIDTH thick (`LINE`), with
 * round joins and ends, or, for a single reading, a ring of DOT_RADIUS round it drawn as thick.
 */
const LINE_WIDTH = 3;
const DOT_RADIUS = 3;
/**
 * How far, in pixels, an edge must be from a pixel of the dashed line for the pixel to be judged:
 * more than this from its square, the manager's "within a pixel of anti-aliasing". The ends of each
 * dash and gap, the ends of the line, the sides of the soft fill and the edge of the chart's own
 * line are each blended into the pixels they cross, so the verdicts judge a pixel only when each of
 * them is more than a pixel from it: an end of a dash may fall anywhere within a pixel of where the
 * 6/6 pattern puts it, and a pixel judged is one a dash or a gap covers whole.
 */
const ANTI_ALIASING = 1;
/**
 * How far a pixel the dash covers whole may be from `--chart-limit`: 0, since nothing is blended
 * there. The dash is opaque, so the canvas writes a pixel it covers whole in the dash's own colour,
 * whatever is under it, over the card and over the soft fill alike, as it writes the material at
 * the corners, which is held to 0 too. A pixel the dash covers in part is a blend, and is not
 * judged (ANTI_ALIASING). What a gap shows, the card's fill or the soft fill, is a see-through
 * colour laid over another, which the canvas rounds, and is held within BLEND. Chrome's pictures
 * bear this out: every pixel the verdicts judge in a dash is --chart-limit exactly, in both themes,
 * and every one in a gap is within 1 of what is worked out for it.
 */
const DASH_ROUNDING = 0;

/** How far `point` is from the straight line from `from` to `to`, its ends included. */
function fromSegment([x, y], [x0, y0], [x1, y1]) {
  const [dx, dy] = [x1 - x0, y1 - y0];
  const length = dx * dx + dy * dy;
  const t = length === 0 ? 0 : Math.min(1, Math.max(0, ((x - x0) * dx + (y - y0) * dy) / length));
  return Math.hypot(x - (x0 + t * dx), y - (y0 + t * dy));
}

/** How far `point` is from the line through `line`'s points, or from its point when it has one. */
function fromLine(point, line) {
  if (line.length === 1) {
    return fromSegment(point, line[0], line[0]);
  }
  let least = Number.POSITIVE_INFINITY;
  for (let index = 1; index < line.length; index += 1) {
    least = Math.min(least, fromSegment(point, line[index - 1], line[index]));
  }
  return least;
}

/** Whether `point` is inside `outline`, closed from its last corner to its first, by the nonzero rule a canvas fills by. */
function inside([x, y], outline) {
  let winding = 0;
  outline.forEach(([x0, y0], index) => {
    const [x1, y1] = outline[(index + 1) % outline.length];
    const side = (x1 - x0) * (y - y0) - (x - x0) * (y1 - y0);
    if (y0 <= y && y1 > y && side > 0) {
      winding += 1;
    } else if (y0 > y && y1 <= y && side < 0) {
      winding -= 1;
    }
  });
  return winding !== 0;
}

/**
 * The pixels of a chart's dashed 90% line that the verdicts judge, at `scale` pixels to a CSS
 * pixel, worked out from the chart's `box` and its readings, `points`, as src/share.ts draws them:
 * `all`, how many pixels the line covers whole, in each of its rows, from its left end to its
 * right; `covered`, those of them left out because the chart's own line, drawn over the dashed one,
 * comes within a pixel of them, worked out from the readings as `emptyPlaces` works out where the
 * line runs, each `[x, y]`; and `places`, the rest, less those within a pixel of the end of a dash,
 * a gap or the line or of a side of the soft fill, left to anti-aliasing. Each place is `{ x, y,
 * along, dash, over }`: the pixel, how far along the line it is (0 to 1), whether the 6/6 pattern
 * puts a dash there or a gap, and what the line is drawn over there: "fill", the soft fill under
 * the chart's line, where the line is above 90%, or "card". Places run from the left end, a column
 * at a time.
 */
export function dashPlaces(box, points, scale) {
  const held = (value, least, most) => Math.min(most, Math.max(least, value));
  const [left, right] = [box.x * scale, (box.x + box.width) * scale];
  const [top, foot] = [box.y * scale, (box.y + box.height) * scale];
  const middle = top + ((100 - LIMIT_PCT) / 100) * (foot - top);
  const half = (LIMIT_WIDTH * scale) / 2;
  const [on, off] = LIMIT_DASH.map((length) => length * scale);
  // The chart's own line, each reading held to the chart as share.ts holds it, and how far from
  // it the line is drawn: half its width, or out to the far side of a single reading's ring.
  const line = points.map(([along, used]) => [
    left + held(along, 0, 1) * (right - left),
    top + ((100 - held(used, 0, 100)) / 100) * (foot - top),
  ]);
  const reach = (line.length === 1 ? DOT_RADIUS + LINE_WIDTH / 2 : LINE_WIDTH / 2) * scale;
  // The soft fill, from the chart's foot under its first reading, along the line to its last and
  // down to the foot again; a single reading fills nothing.
  const outline = line.length > 1 ? [[line[0][0], foot], ...line, [line.at(-1)[0], foot]] : [];
  const ends = [left, right, ...(line.length > 1 ? [line[0][0], line.at(-1)[0]] : [])];
  const places = [];
  const covered = [];
  let all = 0;
  for (let x = Math.ceil(left); x + 1 <= right; x += 1) {
    for (let y = Math.ceil(middle - half); y + 1 <= middle + half; y += 1) {
      all += 1;
      const centre = [x + 0.5, y + 0.5];
      // A square's farthest point is half its diagonal from its middle.
      if (line.length > 0 && fromLine(centre, line) < reach + Math.SQRT1_2 + ANTI_ALIASING) {
        covered.push([x, y]);
        continue;
      }
      // The ends of the dashes and gaps, of the line and of the fill each cross it upright, so a
      // pixel's square reaches half a pixel from its middle towards each.
      const phase = (centre[0] - left) % (on + off);
      const edge = Math.min(phase, Math.abs(phase - on), on + off - phase, ...ends.map((end) => Math.abs(centre[0] - end)));
      if (edge < 0.5 + ANTI_ALIASING) {
        continue;
      }
      places.push({
        x,
        y,
        along: (centre[0] - left) / (right - left),
        dash: phase < on,
        over: outline.length > 0 && inside(centre, outline) ? "fill" : "card",
      });
    }
  }
  return { places, all, covered };
}

/** The least room, in CSS pixels, between the card and an edge of the picture for the material to show in. */
const LEAST_MARGIN = 8;

/** How far down between its faint line at 100% and its dashed line at 90% a chart is looked at. */
const BAND_PCT = 95;
/** A chart's line is at least this far below the band where the plate is looked for. */
const CLEAR_PCT = 85;
/** The most places along a chart looked at. */
const MOST_PLACES = 6;

/**
 * Where along a chart (0 to 1) the band above its dashed line is empty of its line and of the
 * share filled beneath it, so that only a plate could colour it: the places where the two readings
 * between which its line runs are both under CLEAR_PCT, a line between two readings being never
 * higher than the higher of them.
 */
function emptyPlaces(points) {
  const along = Array.from({ length: 12 }, (_, index) => 0.04 + index * 0.08);
  const empty = along.filter((place) => {
    const before = points.filter(([at]) => at <= place).at(-1);
    const after = points.find(([at]) => at >= place);
    return [before, after].every((reading) => !reading || reading[1] <= CLEAR_PCT);
  });
  const every = Math.max(1, Math.ceil(empty.length / MOST_PLACES));
  return empty.filter((_, index) => index % every === 0);
}

/**
 * A verdict on each thing a picture's pixels must say, false where it passed or what is wrong:
 * the picture's corners and the margin between its edges and the card are the material; the card's
 * padding, where nothing is written, is the card's fill laid over the material; the card's outline
 * is its hairline edge, which is clear in dark; the empty band above each chart's dashed line,
 * where only a plate could be, is the card's fill too; and each chart's 90% line, read along the
 * whole of it (`dashPlaces`), is dashed 6/6 in `--chart-limit` over the card and over the soft fill,
 * three verdicts a chart: a dash wherever the pattern puts one, what the line is drawn over in each
 * gap, so that it is not solid, and each dash in `--chart-limit`. A verdict names the first place
 * that is not, along the line from its left end, and a chart whose own line leaves no dash or no
 * gap of its 90% line to judge fails, saying how much it covers.
 *
 * `want` holds `scale` (pixels to a CSS pixel); `card`, where the card is in CSS pixels, its `x`,
 * `y`, `width`, `height`, `padding` and `edge` (the hairline's width); `ground`, `fill` (the card's
 * fill laid over the ground), `edge` (the hairline laid over the fill), `limit` (`--chart-limit`)
 * and `shading` (the soft fill laid over the card's fill), each `[r, g, b]`, the last two as
 * `dashColours` gives them; and `charts`, each with its `label`, its `box` in CSS pixels and its
 * readings, `points`.
 */
export function pictureVerdicts(picture, want) {
  const { scale, card, ground, fill, edge, limit, shading, charts } = want;
  // A place off the picture is read from the nearest one on it, so a card put where the check does
  // not expect it fails a verdict, which names what it found, and does not end the check.
  const clamp = (value, last) => Math.min(Math.max(Math.floor(value), 0), last);
  const at = (x, y) => picture.pixel(clamp(x * scale, picture.width - 1), clamp(y * scale, picture.height - 1));
  const [wide, high] = [picture.width / scale, picture.height / scale];
  const verdict = (wrong, said) => (wrong ? said(wrong) : false);
  const verdicts = [];

  verdicts.push(
    verdict(
      firstWrong(
        [
          ["top left corner", picture.pixel(0, 0)],
          ["top right corner", picture.pixel(picture.width - 1, 0)],
          ["bottom left corner", picture.pixel(0, picture.height - 1)],
          ["bottom right corner", picture.pixel(picture.width - 1, picture.height - 1)],
        ],
        ground,
        0,
      ),
      ([where, found]) => `has ${show(found)} at its ${where}, not the material ${show(ground)}`,
    ),
  );

  // Between each edge of the picture and the card, which is inset from every side: the middle of
  // each margin is the material, unless a margin is too narrow to show it.
  const margins = [
    ["left", card.x, at(card.x / 2, high / 2)],
    ["right", wide - card.x - card.width, at(wide - (wide - card.x - card.width) / 2, high / 2)],
    ["top", card.y, at(wide / 2, card.y / 2)],
    ["bottom", high - card.y - card.height, at(wide / 2, high - (high - card.y - card.height) / 2)],
  ];
  const narrow = margins.find(([, room]) => room < LEAST_MARGIN);
  verdicts.push(
    narrow
      ? `has the card ${narrow[1].toFixed(0)} px from its ${narrow[0]} edge, with no margin for the material to show in, so the card does not sit on it`
      : verdict(
          firstWrong(
            margins.map(([side, , found]) => [`${side} margin`, found]),
            ground,
            0,
          ),
          ([where, found]) =>
            `has ${show(found)} in its ${where}, not the material ${show(ground)}, so the card does not sit on it`,
        ),
  );

  // In the middle of each side of the card's padding, where nothing is written: the fill.
  const half = card.padding / 2;
  verdicts.push(
    verdict(
      firstWrong(
        [
          ["left padding", at(card.x + half, high / 2)],
          ["right padding", at(card.x + card.width - half, high / 2)],
          ["top padding", at(wide / 2, card.y + half)],
          ["bottom padding", at(wide / 2, card.y + card.height - half)],
        ],
        fill,
        BLEND,
      ),
      ([where, found]) => `has ${show(found)} in the card's ${where}, not the card's fill ${show(fill)}`,
    ),
  );

  // Across the middle of the hairline, just inside the card's outline.
  const middle = (edgeAt) => Math.floor(edgeAt * scale + (card.edge * scale) / 2);
  verdicts.push(
    verdict(
      firstWrong(
        [
          ["left edge", at(middle(card.x) / scale, high / 2)],
          ["top edge", at(wide / 2, middle(card.y) / scale)],
        ],
        edge,
        BLEND,
      ),
      ([where, found]) => `has ${show(found)} on the card's ${where}, not its hairline ${show(edge)}`,
    ),
  );

  // Behind each chart: no plate, so the empty band above its dashed line is the card's fill.
  for (const { label, box, points } of charts) {
    const places = emptyPlaces(points);
    if (places.length === 0) {
      verdicts.push(`has no empty place on ${label}'s chart to look for a plate, its line being over ${CLEAR_PCT}% all along it`);
      continue;
    }
    const y = box.y + ((100 - BAND_PCT) / 100) * box.height;
    verdicts.push(
      verdict(
        firstWrong(
          places.map((along) => [along, at(box.x + along * box.width, y)]),
          fill,
          BLEND,
        ),
        ([along, found]) =>
          `draws a plate behind ${label}'s chart: ${show(found)} ${(along * 100).toFixed(0)}% along its empty band above 90%, where only the card's fill ${show(fill)} should be`,
      ),
    );
  }

  // Each chart's dashed 90% line, every pixel of it but those its own line covers and those within a
  // pixel of an end: a dash where the 6/6 pattern puts one, what the line is drawn over in each gap,
  // and each dash in its own colour, whatever it is drawn over.
  for (const { label, box, points } of charts) {
    const { places, all, covered } = dashPlaces(box, points, scale);
    const read = places.map((place) => ({
      ...place,
      found: picture.pixel(place.x, place.y),
      under: place.over === "fill" ? shading : fill,
    }));
    // A pixel that is what the line is drawn over there shows nothing of the line.
    const bare = (place) => near(place.found, place.under, BLEND);
    const along = (place) => `${(place.along * 100).toFixed(0)}% along ${label}'s 90% line`;
    const under = (place) =>
      `${place.over === "fill" ? "the soft fill under its line" : "the card's fill"} ${show(place.under)}`;
    const nothing = (what) =>
      `has no ${what} of ${label}'s 90% line to judge: its own line, drawn over it, covers ${covered.length} of the ${all} pixels the 90% line covers whole`;
    const dashes = read.filter((place) => place.dash);
    const gaps = read.filter((place) => !place.dash);
    const missing = dashes.find(bare);
    verdicts.push(
      dashes.length === 0
        ? nothing("dash")
        : verdict(
            missing,
            (place) => `has no dash ${along(place)}, where the 6/6 pattern puts one: ${show(place.found)} there is ${under(place)}`,
          ),
    );
    const filled = gaps.find((place) => !bare(place));
    verdicts.push(
      gaps.length === 0
        ? nothing("gap")
        : verdict(
            filled,
            (place) =>
              `has ${show(place.found)} in a gap ${along(place)}, where the 6/6 pattern leaves ${under(place)} to show, so the line is not dashed there`,
          ),
    );
    const wrong = dashes.find((place) => !bare(place) && !near(place.found, limit, DASH_ROUNDING));
    verdicts.push(
      verdict(
        wrong,
        (place) =>
          `draws the dash ${along(place)} in ${show(place.found)} over ${under(place)}, not in --chart-limit ${show(limit)}`,
      ),
    );
  }
  return verdicts;
}

/**
 * How near the card's fill the middle of an hour's box may be and still have no square: the faintest
 * square, the chart line's colour at 6% for an hour the period holds with no use, is 11 or more away
 * in a channel in both themes.
 */
const SQUARE = 4;

/**
 * Verdicts on the heatmap a picture draws (D-144), read from its own pixels in the middle of each
 * hour's box: the card's fill where the copy has no cell, an hour the period holds none of, so that no
 * square is drawn outside the period at either end; and a square where it has one, so that none is
 * missing inside it. `want.heat` holds the rows' labels, `days`, their `cells`, and `boxes`, each
 * row's 24 boxes in CSS pixels (`heatBoxes` in src/share.ts), or is null for a picture with no
 * heatmap; `want.scale` and `want.fill` are as `pictureVerdicts` takes them. Each verdict names the
 * first hour that is wrong and how many are.
 */
export function heatVerdicts(picture, want) {
  const { scale, fill, heat } = want;
  if (!heat) {
    return [];
  }
  const clamp = (value, last) => Math.min(Math.max(Math.floor(value), 0), last);
  const middle = ({ x, y, width, height }) =>
    picture.pixel(clamp((x + width / 2) * scale, picture.width - 1), clamp((y + height / 2) * scale, picture.height - 1));
  const outside = [];
  const missing = [];
  heat.cells.forEach((hours, row) => {
    hours.forEach((cell, hour) => {
      const box = heat.boxes[row]?.[hour];
      if (!box) {
        return;
      }
      const found = middle(box);
      const where = { day: heat.days[row], hour, found };
      if (cell === null && !near(found, fill, BLEND)) {
        outside.push(where);
      }
      if (cell !== null && near(found, fill, SQUARE)) {
        missing.push(where);
      }
    });
  });
  const name = ({ day, hour }) => `"${day}" at ${hour}:00`;
  const count = (hours) => (hours.length === 1 ? "1 hour" : `${hours.length} hours`);
  return [
    outside.length > 0 &&
      `draws a square for ${name(outside[0])}, an hour the period holds none of: ${show(outside[0].found)} where only the card's fill ${show(fill)} should be (${count(outside)} in all)`,
    missing.length > 0 &&
      `draws no square for ${name(missing[0])}, an hour the period holds: the card's fill ${show(missing[0].found)} there (${count(missing)} in all)`,
  ];
}
