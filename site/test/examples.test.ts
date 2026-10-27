import assert from "node:assert/strict";
import { test } from "node:test";

import { site } from "../../apps/tray/scripts/site-fixtures.mjs";
import { TIDE_ROWS, WEEKLY_CURVE } from "../lib/data/examples.ts";

/** The pictures of the app and the home page's table and curve show the same example week. */
test("the site's examples match the pictures of the app", () => {
  const windows = new Map<string, { used_pct: number; pace: string | null; runs_out: boolean }>();
  for (const account of site.meter.accounts) {
    for (const window of account.windows) {
      windows.set(`${account.provider} ${window.label}`, window);
    }
  }
  for (const row of TIDE_ROWS) {
    const window = windows.get(row.window);
    assert.ok(window, `the app shows ${row.window}`);
    assert.equal(window.used_pct, row.used, `${row.window}'s share used`);
    assert.equal(window.runs_out, row.runsOutFirst, `${row.window} runs out first or not`);
  }
  const weekly = windows.get("Claude Weekly");
  assert.equal(WEEKLY_CURVE.points.at(-1)?.used, weekly?.used_pct, "the curve ends where the app reads");
  // At 2.0% an hour from 91%, 100% comes 4.5 hours later: the curve's prediction and the app's pace.
  assert.match(weekly?.pace ?? "", /at 2\.0%\/h, runs out in 4 h 30 min/);
  assert.equal(WEEKLY_CURVE.runsOut.hour - (WEEKLY_CURVE.points.at(-1)?.hour ?? 0), 4.5);
});

/**
 * Each period the pictures' History offers is as long as it is named, so its picture to share is,
 * and its heatmap has a row for each calendar day the period touches (D-144): one more than its
 * days, since it ends at 14:10 and not at midnight.
 */
test("each period of the pictures' History covers the days it is named for, a row for each day it touches and four readings a day", () => {
  const DAY = 24 * 60 * 60 * 1000;
  for (const [key, view] of Object.entries(site.history)) {
    const days = Number(key);
    assert.equal(view.days, days, `the ${key}-day period says it covers ${view.days} days`);
    assert.equal(view.span, `The last ${days} days.`);
    const { start, now } = site.periods[key];
    const touched = Math.floor((now - 1) / DAY) - Math.floor(start / DAY) + 1;
    assert.equal(touched, days + 1, `the ${key}-day period ends part way through a day`);
    for (const account of view.accounts) {
      const labels = account.heatmap.days;
      assert.equal(labels.length, touched, `${account.label} over ${key} days: a heatmap row for each day the period touches`);
      assert.equal(new Set(labels).size, touched, `${account.label} over ${key} days: each day named once`);
      const [first, last] = [labels[0], labels[labels.length - 1]].map((label) => label.split(" ")[1]);
      assert.ok(view.dates.startsWith(`${first} `) && view.dates.includes(` to ${last} `), `${view.dates} runs from ${first} to ${last}`);
      for (const window of account.windows) {
        assert.equal(window.points.length, days * 4, `${account.label} ${window.label} over ${key} days: four readings a day`);
        assert.ok(window.summary.endsWith(`in the last ${days} days`), window.summary);
      }
    }
  }
  assert.deepEqual(Object.keys(site.history), ["7", "30"], "History's two periods");
});

test("the Buy button takes only an https link to Lemon Squeezy", async () => {
  const { checkoutLink } = await import("../lib/data/site.ts");
  assert.equal(checkoutLink(undefined), null);
  assert.equal(checkoutLink(""), null);
  assert.equal(checkoutLink("https://quotadial.lemonsqueezy.com/buy/abc"), "https://quotadial.lemonsqueezy.com/buy/abc");
  assert.equal(checkoutLink("http://quotadial.lemonsqueezy.com/buy/abc"), null, "not https");
  assert.equal(checkoutLink("https://lemonsqueezy.com.example.net/buy"), null, "another host");
  assert.equal(checkoutLink("not a link"), null);
});
