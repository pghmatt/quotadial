/**
 * Free and Pro side by side, row by row: each row is a topic, with what Free has beside what Pro
 * adds, so the two lists stay in register wherever a line wraps. The numbers index FREE_HAS and
 * PRO_FEATURES (lib/data/pro.ts), so the words stay the app's own; test/home.test.ts holds every
 * line of both lists to exactly one row.
 */
export const PLAN_ROWS: { topic: string; free: number[]; pro: number[] }[] = [
  { topic: "Accounts", free: [0], pro: [0] },
  { topic: "Windows", free: [1], pro: [] },
  { topic: "History", free: [2], pro: [1, 2] },
  { topic: "Alerts", free: [3], pro: [3, 4] },
  { topic: "Status pages", free: [4], pro: [] },
  { topic: "Agents", free: [5], pro: [6, 7] },
  { topic: "Cost", free: [], pro: [5] },
];
