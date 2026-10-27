// What a share picture's pixels must say (D-137), read from the PNG the window sends the app, as
// the person who posts it gets it: the ground is the material of the theme, the card sits on it,
// and nothing is drawn as a plate behind a chart. check-window.mjs runs these over every picture
// it makes; share-picture.test.mjs proves each one fails on a picture broken on purpose, with no
// browser. The PNG is read here with no library: a signature, chunks with their checksums, one
// zlib stream, and the five filters of a scanline.
import { crc32, inflateSync } from "node:zlib";

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
 * is its hairline edge, which is clear in dark; and the empty band above each chart's dashed line,
 * where only a plate could be, is the card's fill too. A verdict names the first place that is not.
 *
 * `want` holds `scale` (pixels to a CSS pixel); `card`, where the card is in CSS pixels, its `x`,
 * `y`, `width`, `height`, `padding` and `edge` (the hairline's width); `ground`, `fill` (the card's
 * fill laid over the ground) and `edge` (the hairline laid over the fill), each `[r, g, b]`; and
 * `charts`, each with its `label`, its `box` in CSS pixels and its readings, `points`.
 */
export function pictureVerdicts(picture, want) {
  const { scale, card, ground, fill, edge, charts } = want;
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
  return verdicts;
}
