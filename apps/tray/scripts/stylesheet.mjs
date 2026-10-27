// The window's colours as src/style.css gives them, for the checks that hold them to something
// (check-contrast.mjs, share.test.mjs): each theme's custom properties, light and dark, read from
// the stylesheet itself so that a colour changed there is the colour checked.
import { readFileSync } from "node:fs";

const css = readFileSync(new URL("../src/style.css", import.meta.url), "utf8");

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

/** Each theme and its custom properties: the dark theme is the light one with its own laid over. */
export function themes() {
  const light = variables(0);
  const darkAt = css.indexOf("prefers-color-scheme: dark");
  return [
    ["light", light],
    ["dark", { ...light, ...variables(darkAt) }],
  ];
}
