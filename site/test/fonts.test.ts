import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

const layout = readFileSync(new URL("../app/layout.tsx", import.meta.url), "utf8");
const styles = readFileSync(new URL("../app/globals.css", import.meta.url), "utf8");

/** The font packages app/layout.tsx imports, each with the files it takes from it. */
function imported(): Map<string, string[]> {
  const packages = new Map<string, string[]>();
  for (const [, name, file] of layout.matchAll(/^import "@fontsource-variable\/([a-z-]+)(?:\/([\w.-]+))?";$/gm)) {
    packages.set(name, [...(packages.get(name) ?? []), file ?? "index"]);
  }
  return packages;
}

test("every width of Sofia Sans the site sets is loaded with its own italic", () => {
  // Captions, plate titles and the curve's notes are italic. A width loaded upright only would
  // have the browser slant its letters instead, which the type's own italic does better.
  const packages = imported();
  const widths = [...packages.keys()].filter((name) => name.startsWith("sofia-sans"));
  assert.deepEqual(widths.sort(), ["sofia-sans", "sofia-sans-condensed", "sofia-sans-semi-condensed"]);
  for (const width of widths) {
    assert.ok(packages.get(width)?.includes("wght-italic.css"), `${width} is loaded with wght-italic.css`);
  }
});

test("code stays upright, so the code face, loaded without an italic, is never slanted", () => {
  // An italic caption that names a command would otherwise slant the command too.
  const rule = styles.match(/code,\s*kbd,\s*pre\s*\{([^}]*)\}/);
  assert.ok(rule, "the stylesheet styles code, kbd and pre together");
  assert.match(rule[1], /font-style:\s*normal;/);
  assert.ok(!imported().get("jetbrains-mono")?.includes("wght-italic.css"), "the code face loads no italic it would never use");
});

test("each typeface the stylesheet names is one the site loads", () => {
  const names = new Map([
    ["Sofia Sans Variable", "sofia-sans"],
    ["Sofia Sans Condensed Variable", "sofia-sans-condensed"],
    ["Sofia Sans Semi Condensed Variable", "sofia-sans-semi-condensed"],
    ["JetBrains Mono Variable", "jetbrains-mono"],
  ]);
  const families = [...styles.matchAll(/--font-[a-z]+:\s*"([^"]+)"/g)].map(([, family]) => family);
  assert.ok(families.length >= 3, "the stylesheet names the site's typefaces");
  const packages = imported();
  for (const family of families) {
    const name = names.get(family);
    assert.ok(name, `${family} is one of the site's typefaces`);
    assert.ok(packages.has(name), `${family} is loaded by app/layout.tsx`);
  }
});
