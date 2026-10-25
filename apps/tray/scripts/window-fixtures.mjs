// @ts-check
// What the window check (check-window.mjs) answers in place of the app: the shapes the app's
// commands return, typed against src/meter.ts, so `tsc` fails here when a shape changes there. The
// accounts are made up, on example.com, and one has a very long address and a long name given it,
// so the check proves long text wraps rather than being cut off, and that a picture to share cuts
// a long name short and never shows an address. Two copies of the app: Pro, with the week's cost,
// a key kept and a Share image button on each account in History, and Free, with what Pro adds,
// the line said in the buttons' place, and an update on offer.

/** @typedef {import("../src/meter").Meter} Meter */
/** @typedef {import("../src/meter").Settings} Settings */
/** @typedef {import("../src/meter").HistoryView} HistoryView */
/** @typedef {import("../src/meter").AccountHistory} AccountHistory */
/** @typedef {import("../src/meter").HelpView} HelpView */
/** @typedef {import("../src/meter").UpdateView} UpdateView */
/** @typedef {{ meter: Meter, settings: Settings, history: Record<string, HistoryView>, help: HelpView, update: UpdateView }} Copy */

const LONG_NAME = "a.very.long.account.name.that.keeps.going@engineering.example.com";

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
    alias: "Work, the team account we share across two offices",
    summary: "Claude Work, the team account we share across two offices: Weekly 91%",
    provider: "Claude",
    name: "dev@example.com",
    plan: "Max 5x",
    status: "ok",
    ok: true,
    windows: [
      {
        label: "Weekly",
        used_pct: 91.4,
        level: "high",
        resets: "resets in 2 d 4 h",
        fetched: "from the API 4 min ago",
        unconfirmed: false,
        pace: "at 2.0%/h, runs out in 4 h 18 min, 1 d 23 h before it resets",
        runs_out: true,
      },
      {
        label: "Weekly Opus",
        used_pct: 74,
        level: "warning",
        resets: "resets in 2 d 4 h",
        fetched: "from the API 4 min ago",
        unconfirmed: false,
        pace: null,
        runs_out: false,
      },
      {
        label: "Session",
        used_pct: 38.2,
        level: "normal",
        resets: "resets in 2 h 10 min",
        fetched: "from the API 4 min ago",
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
    key: `codex:${LONG_NAME}`,
    alias: null,
    summary: "Codex: 5-hour 12%",
    provider: "Codex",
    name: LONG_NAME,
    plan: "Pro",
    status: "stale",
    ok: false,
    windows: [
      {
        label: "5-hour",
        used_pct: 12,
        level: "normal",
        resets: "resets in 3 h 41 min",
        fetched: "from the API 14 min ago",
        unconfirmed: false,
        pace: null,
        runs_out: false,
      },
      {
        label: "Weekly",
        used_pct: 0,
        level: "normal",
        resets: "resets in 6 d 1 h",
        fetched: "from the API 14 min ago",
        unconfirmed: true,
        pace: null,
        runs_out: false,
      },
    ],
    extra_usage: null,
    note:
      "The usage API has not answered for 14 min, so these are its last numbers. QuotaDial asks " +
      "again within a minute.",
    sign_in: CODEX_SIGN_IN,
  },
  {
    key: "claude:someone@example.org",
    alias: null,
    summary: "Claude someone@example.org: sign-in required",
    provider: "Claude",
    name: "someone@example.org",
    plan: null,
    status: "sign-in required",
    ok: false,
    windows: [],
    extra_usage: null,
    note: "This sign-in has ended. Run claude once and QuotaDial picks up the new one.",
    sign_in: CLAUDE_SIGN_IN,
  },
];

/** @type {Meter} */
const METER = {
  percent: 91,
  level: "high",
  headline: "Claude Weekly, resets in 2 d 4 h",
  accounts: ACCOUNTS,
  problems: [
    "The history could not be saved: the disk is full. Free some space and QuotaDial carries on " +
      "by itself.",
  ],
  incidents: ["Claude: Elevated errors on Claude Opus"],
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
      written: "tell me 30 minutes before the weekly window runs out at this pace",
      read_as: "when a Weekly window will run out within 30 min at its pace",
      enabled: false,
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
  licence: { text: "QuotaDial Pro, licence 1234.", kept: true, adds: [] },
  check_updates: true,
};

const DAY_NAMES = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

/** The days each period covers, as the app writes them, from Monday 20 April 2026. */
const DATES = { 7: "20 to 26 April 2026", 30: "20 April to 19 May 2026" };

/** On QuotaDial Free, the line History shows where the Share image buttons would be. */
const SHARE_LEFT_OUT = "A picture of an account's history to share is part of QuotaDial Pro: quotadial.com/pro";

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
 * An account's history over `days` days: a working day's use, heavier mid-week, the same each
 * run so a failure can be reproduced.
 *
 * @param {Meter["accounts"][number]} account
 * @param {number} days
 * @returns {AccountHistory}
 */
function accountHistory(account, days) {
  /** @type {string[]} */
  const labels = [];
  /** @type {number[][]} */
  const cells = [];
  /** @type {string[]} */
  const said = [];
  for (let day = 0; day < days; day += 1) {
    const label = `${DAY_NAMES[day % 7]} ${((day + 19) % 31) + 1}`;
    const weekend = day % 7 >= 5;
    const hours = Array.from({ length: 24 }, (_, hour) =>
      !weekend && hour >= 9 && hour < 18 ? Math.round((3 + ((day * 7 + hour * 3) % 9)) * 10) / 10 : 0,
    );
    const most = hours.indexOf(Math.max(...hours));
    labels.push(label);
    cells.push(hours);
    said.push(
      weekend
        ? `${label}: not used`
        : `${label}: used 9:00 to 18:00, most at ${most}:00 (${Math.round(hours[most])}%)`,
    );
  }
  return {
    key: account.key,
    provider: account.provider,
    name: account.alias ?? account.name,
    label: label(account),
    windows: account.windows.map((window) => {
      /** @type {[number, number][]} */
      const points = Array.from({ length: days * 4 }, (_, index) => [
        index / (days * 4 - 1),
        Math.min(100, (index * window.used_pct) / (days * 4 - 1)),
      ]);
      const latest = window.used_pct;
      return {
        label: window.label,
        points,
        latest,
        highest: latest,
        summary: `${window.label}: latest ${Math.floor(latest)}%, highest ${Math.floor(latest)}% in the last ${days} days`,
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

/**
 * @param {7 | 30} days
 * @param {string | null} shareLeftOut
 * @returns {HistoryView}
 */
function history(days, shareLeftOut) {
  return {
    days,
    span: `The last ${days} days.`,
    dates: DATES[days],
    accounts: ACCOUNTS.filter((account) => account.windows.length > 0).map((account) =>
      accountHistory(account, days),
    ),
    share_left_out: shareLeftOut,
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
          "Updates: once a day the app asks GitHub for a newer QuotaDial and installs it only when you choose Update.",
        ],
      },
      { heading: "Fixed", items: ["A sudden 0% is held until the next fetch agrees."] },
    ],
  },
  licence: "QuotaDial Pro, licence 1234.",
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
    { kind: "paragraph", text: "QuotaDial connects to these, and nothing else:" },
    {
      kind: "list",
      items: [
        "api.anthropic.com, with your Claude Code sign-in, for each Claude account's limits, at most once every 5 minutes per account.",
        "chatgpt.com, with your Codex CLI sign-in, for your ChatGPT/Codex limits, at most once every 2 minutes.",
        "status.claude.com and status.openai.com, the providers' public status pages.",
      ],
    },
  ],
};

/** @type {Copy} */
export const pro = {
  meter: {
    ...METER,
    costs: [
      {
        provider: "Claude",
        summary: "$1,234 at API prices over the last 7 days, from 51,234 requests",
        detail: "From the logs read 2 min ago, at the prices of 1 October 2026.",
        models: ["Claude Opus: $1,010", "Claude Sonnet: $224"],
        notes: ["Leaves out requests made outside Claude Code."],
      },
    ],
  },
  settings: SETTINGS,
  history: { 7: history(7, null), 30: history(30, null) },
  help: HELP,
  update: {
    text: "QuotaDial 0.3.0 is the newest version. Checked 5 min ago.",
    available: null,
    notes: [],
    busy: false,
    progress: null,
  },
};

/** @type {Copy} */
export const free = {
  meter: {
    ...METER,
    left_out: [
      "Showing every account is part of QuotaDial Pro: quotadial.com/pro",
      "The week's cost at API prices is part of QuotaDial Pro: quotadial.com/pro",
    ],
  },
  settings: {
    ...SETTINGS,
    alerts: SETTINGS.alerts.map((alert) => ({ ...alert, waits: alert.id === 2 })),
    alert_targets: [],
    alert_problems: ["config.json names a webhook that is not https; QuotaDial leaves it out."],
    licence: {
      text: "QuotaDial Free, with no licence key.",
      kept: false,
      adds: [
        "every account on this machine, not one per provider",
        "a year of history, not 30 days",
        "a picture of an account's history to share",
        "alerts before a window runs out, before a reset, and for provider incidents",
        "alerts sent to webhooks and Slack",
        "the week's cost at API prices",
        "headroom through the local API and MCP",
        "alert rules through the local API and MCP",
      ],
    },
  },
  history: { 7: history(7, SHARE_LEFT_OUT), 30: history(30, SHARE_LEFT_OUT) },
  help: { ...HELP, licence: "QuotaDial Free, with no licence key." },
  update: {
    text: "QuotaDial 0.4.0 is available. This copy is 0.3.0.",
    available: "0.4.0",
    notes: ["Accessibility: the window works by keyboard and screen reader throughout."],
    busy: false,
    progress: null,
  },
};
