/**
 * The example accounts the site shows, at Tuesday 14:10: the same numbers as the pictures of the
 * app (apps/tray/scripts/site-fixtures.mjs; test/examples.test.ts holds the two together). Every
 * place that shows them says they are examples.
 */

/** The home page's tide table: each window, its share used, its reset, and what its pace says. */
export const TIDE_ROWS = [
  { window: "Claude Weekly", used: 91, resets: "Thu 06:00", pace: "Runs out Tue 18:40", runsOutFirst: true },
  { window: "Claude Session", used: 38, resets: "Today 16:20", pace: "Lasts to the reset", runsOutFirst: false },
  { window: "Codex Weekly", used: 46, resets: "Sun 17:10", pace: "Lasts to the reset", runsOutFirst: false },
] as const;

/**
 * Claude's weekly window as a curve: hours since it began (Thursday 06:00) and the share used. Now
 * is hour 128 (Tuesday 14:10); at 2.0% an hour it reaches 100% at hour 132.5 (Tuesday 18:40),
 * long before it resets at hour 168 (Thursday 06:00).
 */
export const WEEKLY_CURVE = {
  hours: 168,
  days: [
    { hour: 18, label: "Fri" },
    { hour: 42, label: "Sat" },
    { hour: 66, label: "Sun" },
    { hour: 90, label: "Mon" },
    { hour: 114, label: "Tue" },
    { hour: 138, label: "Wed" },
    { hour: 162, label: "Thu" },
  ],
  points: [
    { hour: 0, used: 0 },
    { hour: 10, used: 6 },
    { hour: 18, used: 9 },
    { hour: 30, used: 20 },
    { hour: 42, used: 24 },
    { hour: 66, used: 31 },
    { hour: 80, used: 44 },
    { hour: 90, used: 47 },
    { hour: 104, used: 60 },
    { hour: 114, used: 66 },
    { hour: 120, used: 75 },
    { hour: 124, used: 83 },
    { hour: 128, used: 91 },
  ],
  reset: { hour: 168, label: "Resets Thu 06:00" },
  runsOut: { hour: 132.5, label: "Runs out Tue 18:40" },
} as const;
