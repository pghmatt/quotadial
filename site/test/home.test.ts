import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import { site } from "../../apps/tray/scripts/site-fixtures.mjs";
import { ALERT_SAMPLES, READ_ALOUD, STATUS_SAMPLE } from "../components/home/examples.ts";
import { HOME_META } from "../components/home/meta.ts";
import { WEEKLY_CURVE } from "../lib/data/examples.ts";

type Window = (typeof site.meter.accounts)[number]["windows"][number];

const claudeWeekly = (): Window => {
  const window = site.meter.accounts[0].windows.find((candidate) => candidate.label === "Weekly");
  assert.ok(window, "the example Claude account has a weekly window");
  return window;
};

/** The hour, since the week began, at which the curve first reaches `used` percent. */
function hourReaching(used: number): number {
  const points = WEEKLY_CURVE.points;
  for (let index = 1; index < points.length; index += 1) {
    const [before, after] = [points[index - 1], points[index]];
    if (after.used >= used) {
      return before.hour + ((after.hour - before.hour) * (used - before.used)) / (after.used - before.used);
    }
  }
  throw new Error(`the curve never reaches ${used}%`);
}

/** A time left as the app says it: "1 d 21 h", the minutes dropped once a day is there. */
function timeLeft(hours: number): string {
  const days = Math.floor(hours / 24);
  const rest = Math.floor(hours - days * 24);
  return days > 0 ? `${days} d ${rest} h` : `${rest} h`;
}

/** docs/CLI.md's "Text output", written from the pictures' own accounts. */
test("the quotadial status sample is the example accounts in the command's text format", () => {
  const lines: string[] = [];
  for (const account of site.meter.accounts) {
    lines.push(`${account.provider.toLowerCase()} ${account.name} (${(account.plan ?? "").split(" ")[0].toLowerCase()}): ${account.status}`);
    // The window closest to its limit comes first.
    for (const window of [...account.windows].sort((a, b) => b.used_pct - a.used_pct)) {
      lines.push(`  ${window.label}: ${window.used_pct}% used, ${window.resets}, ${window.fetched}`);
      if (window.pace) {
        lines.push(`    ${window.pace}`);
      }
    }
  }
  assert.equal(STATUS_SAMPLE, lines.join("\n"));
});

test("the row read aloud is Claude Work's weekly window in the app", () => {
  const window = claudeWeekly();
  assert.equal(READ_ALOUD.label, window.label);
  assert.equal(READ_ALOUD.used, window.used_pct);
  assert.equal(window.resets, `resets in ${READ_ALOUD.resets}`);
  assert.equal(window.fetched, `from the API ${READ_ALOUD.fetched}`);
  assert.equal(
    window.pace,
    `at ${READ_ALOUD.rate}, runs out in ${READ_ALOUD.runsOut}, ${READ_ALOUD.early} before it resets`,
  );
});

test("the alert notifications follow the example week", () => {
  const [share, runsOut] = ALERT_SAMPLES;
  // "warn me at 80%": told when the curve reaches 80%, with the time then left until the reset.
  const at80 = hourReaching(80);
  assert.match(share.text, new RegExp(`resets in ${timeLeft(WEEKLY_CURVE.reset.hour - at80)}\\.$`));
  assert.match(share.text, /: 8[01]% used,/);
  // "tell me 30 minutes before the weekly window runs out": 30 minutes before it reaches 100%.
  const before = WEEKLY_CURVE.runsOut.hour - 0.5;
  const used = Math.floor(claudeWeekly().used_pct + 2 * (before - WEEKLY_CURVE.points.at(-1)!.hour));
  assert.match(runsOut.text, new RegExp(`: ${used}% used, at ${READ_ALOUD.rate}, runs out in 30 min, ${READ_ALOUD.early} before it resets\\.$`));
  assert.equal(timeLeft(WEEKLY_CURVE.reset.hour - WEEKLY_CURVE.runsOut.hour), READ_ALOUD.early);
  // The rules are two of the three in the Settings picture, word for word.
  const written = site.settings.alerts.map((alert: { written: string }) => alert.written);
  for (const sample of ALERT_SAMPLES) {
    assert.ok(written.includes(sample.rule), `Settings shows "${sample.rule}"`);
  }
});

test("the home page's title and description are the route map's", () => {
  const config = JSON.parse(readFileSync(new URL("../docs/seo/site.config.json", import.meta.url), "utf8"));
  const route = config.routes.find((candidate: { path: string }) => candidate.path === "/");
  assert.equal(`${HOME_META.title} | QuotaDial`, route.title);
  assert.equal(HOME_META.path, route.path);
  assert.equal(HOME_META.description.length, 153, "the blueprint's meta description is 153 characters");
});
