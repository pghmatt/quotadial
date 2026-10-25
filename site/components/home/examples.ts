/**
 * The example accounts as the home page writes them out in words: the `quotadial status` text, the
 * row read aloud and the two alert notifications. test/home.test.ts holds every figure here to the
 * pictures of the app (apps/tray/scripts/site-fixtures.mjs) and to the week's curve.
 */

/** `quotadial status` for the example accounts, in the format docs/CLI.md shows under "Text output". */
export const STATUS_SAMPLE = `claude dev@example.com (max): ok
  Weekly: 91% used, resets in 1 d 15 h, from the API 3 min ago
    at 2.0%/h, runs out in 4 h 30 min, 1 d 11 h before it resets
  Weekly (Opus): 74% used, resets in 1 d 15 h, from the API 3 min ago
  Session: 38% used, resets in 2 h 10 min, from the API 3 min ago
    at 8.5%/h, lasts until it resets
codex you@example.com (pro): ok
  Weekly: 46% used, resets in 5 d 3 h, from the API 1 min ago
    at 0.4%/h, lasts until it resets
  5-hour: 12% used, resets in 3 h 41 min, from the API 1 min ago`;

/** One row of Claude Work's weekly window, read aloud: "Weekly 91%, resets in 1 d 15 h, ...". */
export const READ_ALOUD = {
  label: "Weekly",
  used: 91,
  resets: "1 d 15 h",
  fetched: "3 min ago",
  rate: "2.0%/h",
  runsOut: "4 h 30 min",
  /** How long before the reset it runs out, from the pace line. */
  early: "1 d 11 h",
} as const;

/** What the two example alert rules tell, in the words the app uses (crates/core/src/alert.rs). */
export const ALERT_SAMPLES = [
  {
    key: "share",
    rule: "warn me at 80%",
    title: "Claude Weekly passed 80%",
    text: "dev@example.com: 81% used, resets in 1 d 21 h.",
  },
  {
    key: "runs-out",
    rule: "tell me 30 minutes before the weekly window runs out",
    title: "Claude Weekly runs out before it resets",
    text: "dev@example.com: 99% used, at 2.0%/h, runs out in 30 min, 1 d 11 h before it resets.",
  },
] as const;
