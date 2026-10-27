// Checks the window's colours against WCAG AA in both themes (#52), from src/style.css itself, so
// a colour changed there is checked here: 4.5:1 for every text colour against every ground it can
// sit on, and 3:1 for the focus ring, a text field's edge, a chart's line and its dashed 90% line
// (WCAG 1.4.11). A ground is a stack: the page, then what is laid over it, such as a card and a
// capsule on the card, each see-through colour laid over what is under it, as the window shows it.
// Run by `npm run build`; fails, naming the pair, when one falls short.
//
// On macOS the system draws the popover material behind a transparent page; the page's own colour,
// which Windows and Linux show, is a grey like it, and stands in for it here. The picture to share
// is drawn on a material of its own choosing, not the page's (D-137), so the text and the chart
// lines it uses are held on that ground, `--picture` below, taken from src/share.ts. It also fills
// the share used softly under a chart's line, the line's colour at SHADING (src/share.ts), and
// draws the dashed 90% line over that fill, so the dashed line is held on it too, as `--shading`.
// History draws no fill. The bars' fill against their track is left out: each bar has its
// percentage beside it in text, which is checked, so the bar is never the only way to tell.
import { MATERIAL, SHADING } from "../src/share.ts";
import { themes } from "./stylesheet.mjs";

/** Each pair: the colour, the grounds under it from the bottom up, the ratio it needs, and where
 * the window uses it. */
const PAIRS = [
  ["--text", ["--background"], 4.5, "text outside a card, such as a panel's"],
  ["--text", ["--background", "--card"], 4.5, "an account's name and its windows, on its card"],
  ["--text", ["--background", "--tag"], 4.5, "a button, such as the footer's, or a command"],
  ["--text", ["--background", "--card", "--tag"], 4.5, "a button or a command on a card"],
  ["--text", ["--background", "--tag", "--segment"], 4.5, "the chosen period in History"],
  ["--secondary", ["--background"], 4.5, "the line under the summary's number"],
  ["--secondary", ["--background", "--card"], 4.5, "an account's address and what it does with its sign-in"],
  ["--secondary", ["--background", "--card", "--tag"], 4.5, "a capsule, such as a plan, on a card"],
  ["--secondary", ["--background", "--tag"], 4.5, "a period not chosen, in History"],
  ["--muted", ["--background"], 4.5, "words outside a card, such as History's span, Free's lines and the origin line"],
  ["--muted", ["--background", "--card"], 4.5, "a window's reset and pace, a card's line for where its windows were read, the provider after an address, and a field's hint, on a card"],
  ["--warning", ["--background"], 4.5, "an incident, and a problem in a panel"],
  ["--warning", ["--background", "--card"], 4.5, "a window past 70%, an account's note and a problem"],
  ["--warning", ["--background", "--card", "--attention"], 4.5, "the capsule of an account that is not ok"],
  ["--high", ["--background"], 4.5, "the summary's number past 90%"],
  ["--high", ["--background", "--card"], 4.5, "a window past 90%, on its card"],
  ["--focus", ["--background"], 3, "the focus ring"],
  ["--focus", ["--background", "--card"], 3, "the focus ring on a card"],
  ["--field", ["--background"], 3, "the edge of a text field"],
  ["--field", ["--background", "--card"], 3, "the edge of a text field in a card, in Settings"],
  ["--bar-normal", ["--background", "--card"], 3, "a chart's line in History, on its card"],
  ["--chart-limit", ["--background", "--card"], 3, "the dashed 90% line in History's charts, on its card"],
  // The picture to share (D-137): the account's card on the picture's material, which is its own
  // choice. It writes only in these five text colours, and draws its chart's line in the bars' blue.
  ["--text", ["--picture", "--card"], 4.5, "the share picture's title, window names, shares and QuotaDial, on its card"],
  ["--secondary", ["--picture", "--card"], 4.5, "the share picture's period and its dates, on its card"],
  ["--muted", ["--picture", "--card"], 4.5, "the share picture's quotadial.com, highest share, 90% label and heatmap's words, on its card"],
  ["--warning", ["--picture", "--card"], 4.5, "a share past 70% in the share picture, on its card"],
  ["--high", ["--picture", "--card"], 4.5, "a share past 90% in the share picture, on its card"],
  ["--bar-normal", ["--picture", "--card"], 3, "the share picture's chart line, on its card"],
  ["--chart-limit", ["--picture", "--card"], 3, "the share picture's dashed 90% line, on its card"],
  ["--chart-limit", ["--picture", "--card", "--shading"], 3, "the share picture's dashed 90% line where a share above 90% fills the chart under its line"],
];

/** `value` as `[r, g, b, a]`, from `#rrggbb` or `rgb(r g b / a%)`. */
function parse(value) {
  const hex = value.match(/^#([0-9a-f]{6})$/i);
  if (hex) {
    const number = Number.parseInt(hex[1], 16);
    return [(number >> 16) & 255, (number >> 8) & 255, number & 255, 1];
  }
  const rgb = value.match(/^rgb\(\s*(\d+)\s+(\d+)\s+(\d+)\s*(?:\/\s*([\d.]+)(%?))?\s*\)$/);
  if (rgb) {
    const alpha = rgb[4] === undefined ? 1 : Number(rgb[4]) / (rgb[5] ? 100 : 1);
    return [Number(rgb[1]), Number(rgb[2]), Number(rgb[3]), alpha];
  }
  throw new Error(`style.css has a colour the contrast check cannot read: ${value}`);
}

/** `top` laid over the opaque `under`. */
function over(top, under) {
  const [r, g, b, a] = top;
  return [0, 1, 2].map((index) => [r, g, b][index] * a + under[index] * (1 - a));
}

function luminance(rgb) {
  const [r, g, b] = rgb.map((channel) => {
    const value = channel / 255;
    return value <= 0.039_28 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

function ratio(a, b) {
  const [light, dark] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (light + 0.05) / (dark + 0.05);
}

/** The picture's soft fill under a chart's line: the line's colour at SHADING, as `parse` reads it. */
function shading(line) {
  const [r, g, b] = parse(line);
  return `rgb(${r} ${g} ${b} / ${SHADING * 100}%)`;
}

let failed = false;
for (const [theme, stylesheet] of themes()) {
  // The picture's ground is its own, in the theme's name, and so is its soft fill, the chart line's
  // colour at SHADING; the rest are the stylesheet's. A missing chart line is said by its own pairs.
  const line = stylesheet["--bar-normal"];
  const colours = {
    ...stylesheet,
    "--picture": MATERIAL[theme],
    ...(line === undefined ? {} : { "--shading": shading(line) }),
  };
  for (const [foreground, grounds, needed, where] of PAIRS) {
    const missing = [foreground, ...grounds].find((name) => colours[name] === undefined);
    if (missing) {
      failed = true;
      console.error(`${theme}: style.css does not define ${missing}, used for ${where}.`);
      continue;
    }
    // The bottom ground is laid on white in case it is see-through, then each one over it in turn.
    const back = grounds.reduce((under, name) => over(parse(colours[name]), under), [255, 255, 255]);
    const found = ratio(over(parse(colours[foreground]), back), back);
    if (found < needed) {
      failed = true;
      console.error(
        `${theme}: ${foreground} on ${[...grounds].reverse().join(" on ")} (${where}) is ${found.toFixed(2)}:1, under ${needed}:1.`,
      );
    }
  }
}
if (failed) {
  console.error(
    "Change the colour in src/style.css, or the share picture's ground, MATERIAL, or its soft fill, SHADING, in src/share.ts, until each pair reaches its ratio.",
  );
  process.exit(1);
}
console.log(`Every colour pair meets WCAG AA in both themes (${PAIRS.length} pairs each).`);
