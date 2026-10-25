// @ts-check
// The website's pictures of the app: one QuotaDial Pro copy with two made-up accounts on
// example.com, at Tuesday 14:10, the same numbers the home page's tide table and curve show
// (site/lib/data/examples.ts; site/test/examples.test.ts holds the two together). Typed against
// src/meter.ts, so `tsc` fails here when a shape changes there.
//
//   node scripts/check-window.mjs --fixtures ./scripts/site-fixtures.mjs --screenshots <folder>

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
    windows: [
      {
        label: "Weekly",
        used_pct: 91,
        level: "high",
        resets: "resets in 1 d 15 h",
        fetched: "from the API 3 min ago",
        unconfirmed: false,
        pace: "at 2.0%/h, runs out in 4 h 30 min, 1 d 11 h before it resets",
        runs_out: true,
      },
      {
        label: "Weekly (Opus)",
        used_pct: 74,
        level: "warning",
        resets: "resets in 1 d 15 h",
        fetched: "from the API 3 min ago",
        unconfirmed: false,
        pace: null,
        runs_out: false,
      },
      {
        label: "Session",
        used_pct: 38,
        level: "normal",
        resets: "resets in 2 h 10 min",
        fetched: "from the API 3 min ago",
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
    windows: [
      {
        label: "Weekly",
        used_pct: 46,
        level: "normal",
        resets: "resets in 5 d 3 h",
        fetched: "from the API 1 min ago",
        unconfirmed: false,
        pace: "at 0.4%/h, lasts until it resets",
        runs_out: false,
      },
      {
        label: "5-hour",
        used_pct: 12,
        level: "normal",
        resets: "resets in 3 h 41 min",
        fetched: "from the API 1 min ago",
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
  headline: "Claude Weekly, resets in 1 d 15 h",
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
    label: `${account.provider} ${account.name}`,
    alias: account.alias ?? "",
  })),
  licence: { text: "QuotaDial Pro, licence 1042.", kept: true, adds: [] },
  check_updates: true,
};

const DAYS = ["Wed 21", "Thu 22", "Fri 23", "Sat 24", "Sun 25", "Mon 26", "Tue 27"];

/** Windows that reset within a day: a session (Claude) or a 5-hour window (Codex). */
const SESSIONS = ["Session", "5-hour"];

/** The most a session window reached on each day of the week, from Wednesday 21. */
const SESSION_PEAKS = [52, 64, 47, 6, 0, 58];

/**
 * A window's week of readings, four a day: a weekly window climbs to its share now, and a session
 * window fills during each working day and resets, so its highest is above its latest.
 *
 * @param {Meter["accounts"][number]["windows"][number]} window
 * @returns {[number, number][]}
 */
function readings(window) {
  return Array.from({ length: 28 }, (_, index) => {
    if (!SESSIONS.includes(window.label)) {
      return [index / 27, (window.used_pct * index) / 27];
    }
    // Tuesday, the last day, fills to the share now.
    const peak = index >= 24 ? window.used_pct : SESSION_PEAKS[Math.floor(index / 4)];
    return [index / 27, Math.round((peak * ((index % 4) + 1)) / 0.4) / 10];
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
 * A week of one account's use: working days busy from 9:00 to 19:00, weekends quiet, the same on
 * every run.
 *
 * @param {Meter["accounts"][number]} account
 * @param {number} days
 * @returns {AccountHistory}
 */
function accountHistory(account, days) {
  /** @type {number[][]} */
  const cells = [];
  /** @type {string[]} */
  const said = [];
  /** @type {string[]} */
  const labels = [];
  for (let day = 0; day < days; day += 1) {
    const label = DAYS[day % 7];
    const weekend = label.startsWith("Sat") || label.startsWith("Sun");
    const hours = Array.from({ length: 24 }, (_, hour) =>
      !weekend && hour >= 9 && hour < 19 ? 2 + ((day * 5 + hour * 3) % 7) : 0,
    );
    const most = hours.indexOf(Math.max(...hours));
    labels.push(label);
    cells.push(hours);
    said.push(weekend ? `${label}: not used` : `${label}: used 9:00 to 19:00, most at ${most}:00 (${hours[most]}%)`);
  }
  return {
    key: account.key,
    provider: account.provider,
    name: account.alias ?? account.name,
    label: label(account),
    windows: account.windows.map((window) => {
      const points = readings(window);
      const highest = Math.max(window.used_pct, ...points.map(([, used]) => used));
      return {
        label: window.label,
        points,
        latest: window.used_pct,
        highest,
        summary: `${window.label}: latest ${window.used_pct}%, highest ${highest}% in the last ${days} days`,
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
  return {
    days,
    span: `The last ${days} days.`,
    dates: "21 to 27 October 2026",
    accounts: ACCOUNTS.map((account) => accountHistory(account, days)),
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
  history: { 7: history(7), 30: history(7) },
  help: HELP,
  update: {
    text: "QuotaDial 0.3.0 is the newest version. Checked 5 min ago.",
    available: null,
    notes: [],
    busy: false,
    progress: null,
  },
};
