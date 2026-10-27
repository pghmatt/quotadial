// What src/style.css says, for the checks that hold it to something: the window's colours
// (check-contrast.mjs, share.test.mjs), each theme's custom properties, light and dark, and the
// rules themselves (whole-shares.test.mjs, whole-words.test.mjs), read from the stylesheet itself so
// that what is changed there is what is checked.
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

/**
 * The stylesheet as its rules, each with the selectors it is for and the declarations it makes, in
 * the order it has them, comments left out. A rule inside a media query is one of them, with no
 * trace of its condition, so a test that needs one reads the stylesheet's text.
 */
export function rules() {
  return [...css.replace(/\/\*[\s\S]*?\*\//g, "").matchAll(/([^{}]+)\{([^{}]*)\}/g)].map(([, selector, body]) => ({
    selectors: selector.split(",").map((name) => name.trim()),
    declarations: body.split(";").map((declaration) => declaration.replace(/\s+/g, " ").trim()).filter(Boolean),
  }));
}
