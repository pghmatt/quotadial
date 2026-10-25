import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import { PRO_ADDS } from "../lib/data/pro.ts";

/**
 * What QuotaDial Pro adds, as the app lists it: `ProFeature::ALL`'s order, in `ProFeature::what`'s
 * words, read from the source of `crates/core/src/tier.rs`. Fails, saying what it could not find,
 * rather than read a changed file as an empty list.
 */
function appList(source: string): string[] {
  const all = source.match(/pub const ALL: \[Self; (\d+)\] = \[([\s\S]*?)\];/);
  assert.ok(all, "tier.rs declares ProFeature::ALL");
  const order = [...all[2].matchAll(/Self::(\w+)/g)].map((found) => found[1]);
  assert.equal(order.length, Number(all[1]), "ALL lists as many features as its type says");
  const start = source.indexOf("pub fn what(self)");
  const end = source.indexOf("pub fn line(self)", start);
  assert.ok(start >= 0 && end > start, "tier.rs has ProFeature::what, then ProFeature::line");
  const words = new Map(
    [...source.slice(start, end).matchAll(/Self::(\w+) => \{?\s*"((?:[^"\\]|\\.)*)"/g)].map((found) => [
      found[1],
      JSON.parse(`"${found[2]}"`) as string,
    ]),
  );
  return order.map((feature) => {
    const said = words.get(feature);
    assert.ok(said, `ProFeature::what has words for ${feature}`);
    return said;
  });
}

test("the site's Pro list is the app's, word for word and in its order", () => {
  const tier = readFileSync(new URL("../../crates/core/src/tier.rs", import.meta.url), "utf8");
  assert.deepEqual([...PRO_ADDS], appList(tier));
});

test("the app's list is read in ALL's order, whatever order the words are written in", () => {
  const source = `
    pub const ALL: [Self; 2] = [Self::Second, Self::First];
    pub fn what(self) -> &'static str {
        match self {
            Self::First => "the first",
            Self::Second => {
                "the second, \\"quoted\\""
            }
        }
    }
    pub fn line(self) -> String {
        let what = match self { Self::First => "not this" };
    }`;
  assert.deepEqual(appList(source), ['the second, "quoted"', "the first"]);
  assert.throws(() => appList(source.replace("[Self; 2]", "[Self; 3]")), /as many features/);
  assert.throws(() => appList(source.replace('Self::First => "the first",', "")), /words for First/);
});
