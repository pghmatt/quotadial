import assert from "node:assert/strict";
import { test } from "node:test";

import { PRO_ADDS } from "../../lib/data/pro.ts";
import { PRO_ROWS } from "./pro-rows.ts";

/** The words the app lists, down to where its own "not" clause begins. */
const lead = (what: string) => what.split(", not ")[0].toLowerCase();

test("the Pro table has the app's seven features, in its order, in its words", () => {
  assert.equal(PRO_ROWS.length, PRO_ADDS.length);
  PRO_ROWS.forEach((row, index) => {
    assert.equal(row.feature.toLowerCase(), lead(PRO_ADDS[index]));
  });
});

test("no row is empty, and none carries a price or a dash", () => {
  for (const row of PRO_ROWS) {
    for (const text of [row.feature, row.free, row.changes]) {
      assert.ok(text.trim().length > 0);
      assert.doesNotMatch(text, /[$£€]|per month|lifetime/i);
      assert.doesNotMatch(text, /[–—]/);
    }
  }
});
