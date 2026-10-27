// @ts-check
// The website's pictures of the app: one QuotaDial Pro copy with two made-up accounts on
// example.com, at Tuesday 14:10, the same numbers the home page's tide table and curve show
// (site/lib/data/examples.ts; site/test/examples.test.ts holds the two together). Typed against
// src/meter.ts, so `tsc` fails here when a shape changes there.
//
//   node scripts/check-window.mjs --fixtures ./scripts/site-fixtures.mjs --screenshots <folder>

import { marksOf } from "./fixture-marks.mjs";

/** @typedef {import("../src/meter").Meter} Meter */
/** @typedef {import("../src/meter").Settings} Settings */
/** @typedef {import("../src/meter").HistoryView} HistoryView */
/** @typedef {import("../src/meter").AccountHistory} AccountHistory */
/** @typedef {import("../src/meter").HelpView} HelpView */
/** @typedef {import("../src/meter").UpdateView} UpdateView */

const CLAUDE_SIGN_IN =
  "QuotaDial reads the sign-in Claude Code uses: the CLAUDE_CODE_OAUTH_TOKEN environment " +
  'variable when it is set, then the macOS Keychain item "Claude Code-credentials", then ' +
  ".credentials.json in ~/.claude or the folder CLAUDE_CONFIG_DIR names. It only reads the " +
  "sign-in and never changes it. If it stops working, run claude once.";
const CODEX_SIGN_IN =
  "QuotaDial reads the sign-in the Codex CLI keeps in auth.json, in ~/.codex or the folder " +
  "CODEX_HOME names. It only reads the sign-in and never changes it. If it stops working, run " +
  "codex once.";

/** @type {Meter["accounts"]} */
const ACCOUNTS = [
  {
    key: "claude:dev@example.com",
    alias: "Work",
    summary: "Claude Work: Weekly 91%",
    provider: "Claude",
    name: "dev@example.com",
    plan: "Max 5x",
    status: "ok",
    ok: true,
    // Three windows of one answer: the card says where and when once, and no window does.
    fetched: "from the API 3 min ago",
    windows: [
      {
        label: "Weekly",
        used_pct: 91,
        level: "high",
        resets: "resets in 1 d 15 h",
        fetched: null,
        unconfirmed: false,
        pace: "at 2.0%/h, runs out in 4 h 30 min, 1 d 11 h before it resets",
        runs_out: true,
      },
      {
        label: "Weekly (Opus)",
        used_pct: 74,
        level: "warning",
        resets: "resets in 1 d 15 h",
        fetched: null,
        unconfirmed: false,
        pace: null,
        runs_out: false,
      },
      {
        label: "Session",
        used_pct: 38,
        level: "normal",
        resets: "resets in 2 h 10 min",
        fetched: null,
        unconfirmed: false,
        pace: "at 8.5%/h, lasts until it resets",
        runs_out: false,
      },
    ],
    extra_usage: null,
    note: null,
    sign_in: CLAUDE_SIGN_IN,
  },
  {
    key: "codex:you@example.com",
    alias: null,
    summary: "Codex you@example.com: Weekly 46%",
    provider: "Codex",
    name: "you@example.com",
    plan: "Pro",
    status: "ok",
    ok: true,
    fetched: "from the API 1 min ago",
    windows: [
      {
        label: "Weekly",
        used_pct: 46,
        level: "normal",
        resets: "resets in 5 d 3 h",
        fetched: null,
        unconfirmed: false,
        pace: "at 0.4%/h, lasts until it resets",
        runs_out: false,
      },
      {
        label: "5-hour",
        used_pct: 12,
        level: "normal",
        resets: "resets in 3 h 41 min",
        fetched: null,
        unconfirmed: false,
        pace: null,
        runs_out: false,
      },
    ],
    extra_usage: null,
    note: null,
    sign_in: CODEX_SIGN_IN,
  },
];

/** @type {Meter} */
const METER = {
  percent: 91,
  level: "high",
  headline: "Weekly · Claude Work · resets in 1 d 15 h",
  accounts: ACCOUNTS,
  problems: [],
  incidents: [],
  origin: "Running the daemon, port 47115",
  costs: [],
  left_out: [],
};

/** @type {Settings} */
const SETTINGS = {
  launch_at_login: true,
  alerts: [
    { id: 1, written: "warn me at 80%", read_as: "when any window reaches 80%", enabled: true, waits: false },
    {
      id: 2,
      written: "tell me 30 minutes before the weekly window runs out",
      read_as: "when a Weekly window will run out within 30 min at its pace",
      enabled: true,
      waits: false,
    },
    {
      id: 3,
      written: "tell me when an account needs to sign in again",
      read_as: "when an account's sign-in stops working",
      enabled: true,
      waits: false,
    },
  ],
  alert_targets: ["hooks.slack.com"],
  alert_problems: [],
  providers: [
    { id: "claude", name: "Claude", shown: true },
    { id: "codex", name: "Codex", shown: true },
  ],
  accounts: ACCOUNTS.map((account) => ({
    key: account.key,
    provider: account.provider,
    name: account.name,
    alias: account.alias ?? "",
  })),
  licence: { text: "QuotaDial Pro, licence 1042.", kept: true, adds: [] },
  check_updates: true,
};

const DAY = 24 * 60 * 60 * 1000;

/** Today, Tuesday 27 October 2026, the last day of every period History shows. */
const TODAY = Date.UTC(2026, 9, 27);

/** Now, at 14:10 today, when every period History shows ends, in UTC, the copy's zone. */
const NOW = TODAY + (14 * 60 + 10) * 60 * 1000;

/** Wednesday 21 October 2026, the first day of the example week, which every period repeats. */
const WEEK_BEGINS = Date.UTC(2026, 9, 21);

const WEEKDAYS = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const MONTHS = [
  "January",
  "February",
  "March",
  "April",
  "May",
  "June",
  "July",
  "August",
  "September",
  "October",
  "November",
  "December",
];

/**
 * The days of the last `days` days, oldest first, each at midnight UTC.
 *
 * @param {number} days
 * @returns {Date[]}
 */
function period(days) {
  return Array.from({ length: days }, (_, day) => new Date(TODAY - (days - 1 - day) * DAY));
}

/**
 * The days a period covers, as the app writes them within a year (`dates` in history.rs): "21 to
 * 27 October 2026", or "28 September to 27 October 2026".
 *
 * @param {Date[]} days
 */
function dates(days) {
  const [first, last] = [days[0], days[days.length - 1]];
  const month = (/** @type {Date} */ day) => MONTHS[day.getUTCMonth()];
  const from = month(first) === month(last) ? `${first.getUTCDate()}` : `${first.getUTCDate()} ${month(first)}`;
  return `${from} to ${last.getUTCDate()} ${month(last)} ${last.getUTCFullYear()}`;
}

/** Windows that reset within a day: a session (Claude) or a 5-hour window (Codex). */
const SESSIONS = ["Session", "5-hour"];

/**
 * The most a session window reaches on each day of the example week. Tuesday, today, fills to the
 * share now, and so does every Tuesday before it.
 *
 * @type {Record<string, number>}
 */
const SESSION_PEAKS = { Wed: 52, Thu: 64, Fri: 47, Sat: 6, Sun: 0, Mon: 58 };

/**
 * A window's readings over the days of a period, four a day, the example week over again: a weekly
 * window climbs through each week to its share now and resets as the next begins, and a session
 * window fills during each working day and resets, so its highest is above its latest.
 *
 * @param {Meter["accounts"][number]["windows"][number]} window
 * @param {Date[]} days
 * @returns {[number, number][]}
 */
function readings(window, days) {
  const count = days.length * 4;
  return Array.from({ length: count }, (_, index) => {
    const along = index / (count - 1);
    if (!SESSIONS.includes(window.label)) {
      // Each week's 28 readings, counted back from now: the last is the share now.
      const step = 27 - ((count - 1 - index) % 28);
      return [along, (window.used_pct * step) / 27];
    }
    const peak = SESSION_PEAKS[WEEKDAYS[days[Math.floor(index / 4)].getUTCDay()]] ?? window.used_pct;
    return [along, Math.round((peak * ((index % 4) + 1)) / 0.4) / 10];
  });
}

/**
 * What a picture of the account's history calls it, as the app does (`share::label`): its provider
 * and the name given it, or its plan, or the provider alone.
 *
 * @param {Meter["accounts"][number]} account
 */
function label(account) {
  const named = account.alias ?? account.plan;
  return named ? `${account.provider} ${named}` : account.provider;
}

/**
 * One account's use over the days of a period: working days busy from 9:00 to 19:00, weekends
 * quiet, each day the same in every period that shows it and on every run.
 *
 * @param {Meter["accounts"][number]} account
 * @param {Date[]} days
 * @returns {AccountHistory}
 */
function accountHistory(account, days) {
  /** @type {number[][]} */
  const cells = [];
  /** @type {string[]} */
  const said = [];
  /** @type {string[]} */
  const labels = [];
  for (const date of days) {
    // As the app labels a day (`day_label`): "Wed 21".
    const label = `${WEEKDAYS[date.getUTCDay()]} ${date.getUTCDate()}`;
    const weekend = label.startsWith("Sat") || label.startsWith("Sun");
    const day = (date.getTime() - WEEK_BEGINS) / DAY;
    const hours = Array.from({ length: 24 }, (_, hour) =>
      !weekend && hour >= 9 && hour < 19 ? 2 + ((((day * 5 + hour * 3) % 7) + 7) % 7) : 0,
    );
    const most = hours.indexOf(Math.max(...hours));
    labels.push(label);
    cells.push(hours);
    said.push(weekend ? `${label}: not used` : `${label}: used 9:00 to 19:00, most at ${most}:00 (${hours[most]}%)`);
  }
  return {
    key: account.key,
    provider: account.provider,
    alias: account.alias,
    name: account.name,
    label: label(account),
    windows: account.windows.map((window) => {
      const points = readings(window, days);
      const highest = Math.max(window.used_pct, ...points.map(([, used]) => used));
      return {
        label: window.label,
        points,
        latest: window.used_pct,
        highest,
        summary: `${window.label}: latest ${window.used_pct}%, highest ${highest}% in the last ${days.length} days`,
      };
    }),
    heatmap: {
      window: account.windows[account.windows.length - 1]?.label ?? "Session",
      days: labels,
      cells,
      most: Math.max(...cells.flat()),
      said,
    },
  };
}

/** @param {number} days @returns {HistoryView} */
function history(days) {
  const covered = period(days);
  return {
    days,
    span: `The last ${days} days.`,
    dates: dates(covered),
    // As the app marks the period it charts (D-143).
    marks: marksOf(days, NOW - days * DAY, NOW),
    accounts: ACCOUNTS.map((account) => accountHistory(account, covered)),
    share_left_out: null,
  };
}

/** @type {HelpView} */
const HELP = {
  version: "0.3.0",
  whats_new: {
    version: "0.3.0",
    sections: [
      {
        heading: "Added",
        items: [
          "Licence keys for QuotaDial Pro, checked on your machine with no network call.",
          "Command or Control and = makes the window larger.",
        ],
      },
    ],
  },
  licence: "QuotaDial Pro, licence 1042.",
  support: "support@quotadial.com",
  credits: [
    "QuotaDial is free software under the MIT licence.",
    "Its provider integrations and cost scan are ported from CodexBar by Peter Steinberger, used under the MIT licence.",
    "Model prices come from models.dev, used under the MIT licence.",
  ],
  privacy: [
    {
      kind: "paragraph",
      text:
        "QuotaDial runs on your machine. It reads the sign-ins that Claude Code and the Codex CLI " +
        "already keep, and never writes, refreshes, moves or logs them.",
    },
  ],
};

/** @type {{ meter: Meter, settings: Settings, history: Record<string, HistoryView>, help: HelpView, update: UpdateView }} */
export const site = {
  meter: METER,
  settings: SETTINGS,
  history: { 7: history(7), 30: history(30) },
  help: HELP,
  update: {
    text: "QuotaDial 0.3.0 is the newest version. Checked 5 min ago.",
    available: null,
    notes: [],
    busy: false,
    progress: null,
  },
};
