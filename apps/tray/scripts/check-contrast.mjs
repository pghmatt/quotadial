// Checks the window's colours against WCAG AA in both themes (#52), from src/style.css itself, so
// a colour changed there is checked here: 4.5:1 for every pair of text and what is behind it, and
// 3:1 for the focus ring and a text field's edge against the background. A see-through colour is laid over the background
// first, as the window shows it. Run by `npm run build`; fails, naming the pair, when one falls short.
//
// The bars' fill against their track is left out: each bar has its percentage beside it in text,
// which is checked, so the bar is never the only way to tell.
import { readFileSync } from "node:fs";

const css = readFileSync(new URL("../src/style.css", import.meta.url), "utf8");

/** Each pair: the colour, what it is on, the ratio it needs, and where the window uses it. */
const PAIRS = [
  ["--text", "--background", 4.5, "body text"],
  ["--muted", "--background", 4.5, "details, the origin line and the legend"],
  ["--warning", "--background", 4.5, "notes, a window past 70% and incidents"],
  ["--high", "--background", 4.5, "a window past 90%"],
  ["--text", "--tag", 4.5, "buttons"],
  ["--muted", "--tag", 4.5, "tags, such as a plan"],
  ["--warning", "--tag", 4.5, "a tag that needs attention"],
  ["--on-accent", "--accent", 4.5, "the chosen period in History"],
  ["--focus", "--background", 3, "the focus ring"],
  ["--field", "--background", 3, "the edge of a text field"],
];

/** The custom properties in the first `:root` block after `from`. */
function variables(from) {
  const start = css.indexOf(":root", from);
  const open = css.indexOf("{", start);
  const close = css.indexOf("}", open);
  const found = {};
  for (const [, name, value] of css.slice(open + 1, close).matchAll(/(--[\w-]+)\s*:\s*([^;]+);/g)) {
    found[name] = value.trim();
  }
  return found;
}

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

const light = variables(0);
const darkAt = css.indexOf("prefers-color-scheme: dark");
const themes = [
  ["light", light],
  ["dark", { ...light, ...variables(darkAt) }],
];

let failed = false;
for (const [theme, colours] of themes) {
  const background = parse(colours["--background"]);
  const solid = (name) => over(parse(colours[name]), background);
  for (const [foreground, behind, needed, where] of PAIRS) {
    const missing = [foreground, behind].find((name) => colours[name] === undefined);
    if (missing) {
      failed = true;
      console.error(`${theme}: style.css does not define ${missing}, used for ${where}.`);
      continue;
    }
    const back = behind === "--background" ? background : solid(behind);
    const found = ratio(over(parse(colours[foreground]), back), back);
    if (found < needed) {
      failed = true;
      console.error(
        `${theme}: ${foreground} on ${behind} (${where}) is ${found.toFixed(2)}:1, under ${needed}:1.`,
      );
    }
  }
}
if (failed) {
  console.error("Change the colour in src/style.css until each pair reaches its ratio.");
  process.exit(1);
}
console.log(`Every colour pair meets WCAG AA in both themes (${PAIRS.length} pairs each).`);
