// @ts-check
// What the window check (check-window.mjs) answers in place of the app: the shapes the app's
// commands return, typed against src/meter.ts, so `tsc` fails here when a shape changes there. The
// accounts are made up, on example.com, and one has a very long address and a long name given it,
// so the check proves long text wraps rather than being cut off, and that a picture to share cuts
// a long name short and never shows an address. One is Gemini CLI's, whose sign-in panel says what
// Gemini CLI says rather than Claude's sentence (P-020), and one Antigravity's, whose windows come
// from its CLI's status line and whose panel carries the block to add to Antigravity CLI's
// settings and a limit with an amount and no share, the longest text the popover shows (D-126).
// One is a Codex Pro account with a single window, whose picture to share has one column (D-137).
// Commands sit between backticks, as QuotaDial writes them: Claude's sign-in command in an
// account's note, Gemini CLI's `/model`, Antigravity's block in its sign-in panel and, in Free's
// Settings, config.json's chmod; the window shows each as code (the manager's note on #85).
// Where and when the windows were read is said once for a card whose windows all agree (Claude Work's,
// the stale Codex account's from the cache, Antigravity's, the example account's one window) and under
// each window of the one card whose windows do not (a Codex account's, which no provider makes today),
// as the app's view model sends it (D-135).
// Four copies of the app: Pro, with the
// week's cost, a key kept and a Share image button on each account in History; Free, with what
// Pro adds, the line said in the buttons' place, an update on offer, Codex's account as an older
// daemon sends it, with no panel, and an example account, its provider shown once; Many, Pro
// with ten accounts, which carry every view far past the window's tallest height (D-136) and, in a
// place of its own each, text from outside as long as it gets, with no spaces in it, and the changelog
// and privacy lines and the update's notes with their real file names and hosts, so that the check
// proves each of them breaks where it must and is never cut off (D-139); and Crowded, Pro with
// one Claude account of seven windows, whose History alone is checked, for the picture to share of
// an account with many windows, where each share must be written whole in every system's own font,
// and where one window, full since the middle of the period, puts its dashed 90% line over the soft
// fill under its line for half the chart, to be judged there.

/** @typedef {import("../src/meter").Meter} Meter */
/** @typedef {import("../src/meter").Settings} Settings */
/** @typedef {import("../src/meter").HistoryView} HistoryView */
/** @typedef {import("../src/meter").AccountHistory} AccountHistory */
/** @typedef {import("../src/meter").WindowChart} WindowChart */
/** @typedef {import("../src/meter").HelpView} HelpView */
/** @typedef {import("../src/meter").UpdateView} UpdateView */
/**
 * @typedef {{ meter: Meter, settings: Settings, history: Record<string, HistoryView>, help: HelpView, update: UpdateView, forScrolling?: boolean, panels?: ("accounts" | "history" | "settings" | "help")[] }} Copy
 * `forScrolling` marks a copy made to scroll: the check puts the window's scrolling through its
 * tests over this copy, once, in the light theme (the keys, the wheel, each view's top, the other
 * sizes), and, since what it holds is checked on the other copies, does not press each of its
 * controls from a window opened afresh or draw each account's picture to share over again.
 * `panels` names the only views of a copy that are checked, where the copy is made for one.
 */

const LONG_NAME = "a.very.long.account.name.that.keeps.going@engineering.example.com";

// Text from outside as long as QuotaDial lets it be, each with nothing in it a browser breaks a line
// at, one for each place the window shows some that is not an address (D-139): a name the person gave
// an account, a plan the provider names, a model's name in a window's label (QuotaDial keeps 64
// characters of it), a folder the person's machine names, a command that holds it, a provider's
// incident and the host of a webhook. The Many copy has them all, as its header says, and an address
// whose name before the @ is as long as an address's may be, 64 characters with no dot, a part wider
// than the window that only the stylesheet's last resort can break (D-141).
const LONG_LOCAL_PART = "continuousintegrationsandboxfortheengineeringplatformteams2026ci";
const LONG_ALIAS = "EngineeringPlatformTeamSharedAccount2026";
const LONG_PLAN = "Enterprise_Annual_Prepaid_Seats_2026";
const LONG_MODEL = "ClaudeSonnetFourPointFiveExtendedContextPreview";
const LONG_FOLDER = "/Users/someone.with.a.long.name/Library/Application Support/quotadial/history.2026-10-27T09-12-00Z.damaged.sqlite3";
const LONG_COMMAND = `chmod 600 '${LONG_FOLDER}'`;
const LONG_INCIDENT = "ElevatedErrorsOnClaudeOpusAndSonnetAcrossAllRegionsPartialOutage";
const LONG_HOST = "alerts.engineering.platform.internal.corp.example.co.uk";

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
    // Three windows of one answer, so the card says where and when once and no window does.
    fetched: "from the API 4 min ago",
    windows: [
      {
        label: "Weekly",
        used_pct: 91.4,
        level: "high",
        resets: "resets in 2 d 4 h",
        fetched: null,
        unconfirmed: false,
        pace: "at 2.0%/h, runs out in 4 h 18 min, 1 d 23 h before it resets",
        runs_out: true,
      },
      {
        label: "Weekly Opus",
        used_pct: 74,
        level: "warning",
        resets: "resets in 2 d 4 h",
        fetched: null,
        unconfirmed: false,
        pace: null,
        runs_out: false,
      },
      {
        label: "Session",
        used_pct: 38.2,
        level: "normal",
        resets: "resets in 2 h 10 min",
        fetched: null,
        unconfirmed: false,
        pace: "at 8.5%/h, lasts until it resets",
        runs_out: false,
      },
    ],
    // A line of its own under the windows, so the caption is checked under it too.
    extra_usage: "on, 12.50 of 50.00 USD used (25%)",
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
    // Its last numbers, served again from the cache with the time of the fetch.
    fetched: "from cache 14 min ago",
    windows: [
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
      {
        label: "Weekly",
        used_pct: 0,
        level: "normal",
        resets: "resets in 6 d 1 h",
        fetched: null,
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
    fetched: null,
    windows: [],
    extra_usage: null,
    // Claude's sentence for a sign-in that expired, with the command for its folder (D-131).
    note:
      "This Claude Code sign-in expired at 2026-10-26T09:12:00Z. Run " +
      "`CLAUDE_CONFIG_DIR=~/'My Claude' claude` once to refresh it.",
    sign_in: CLAUDE_SIGN_IN,
  },
  {
    key: "gemini:google",
    alias: null,
    summary: "Gemini CLI Google account: unsupported",
    provider: "Gemini CLI",
    name: "Google account",
    plan: null,
    status: "unsupported",
    ok: false,
    fetched: null,
    windows: [],
    extra_usage: null,
    note:
      "Google's terms do not let other tools use Gemini CLI's sign-in, so QuotaDial shows no " +
      "numbers for it. In Gemini CLI, `/model` shows each model's share used and when it resets.",
    // Not Claude's sentence: the panel shows what each provider says, and its heading is true of
    // every account (P-020).
    sign_in:
      "QuotaDial only checks that Gemini CLI's sign-in is at ~/.gemini/oauth_creds.json. It never " +
      "opens it.",
  },
  {
    key: "antigravity ag@example.com -",
    alias: null,
    summary: "Antigravity ag@example.com (Pro): Weekly (Claude and GPT) 64%, resets in 3 d 2 h",
    provider: "Antigravity",
    name: "ag@example.com",
    plan: "Pro",
    status: "ok",
    ok: true,
    // One handover from Antigravity CLI's status line, the longest source the card says.
    fetched: "from the CLI's status line 2 min ago",
    windows: [
      {
        label: "Weekly (Claude and GPT)",
        used_pct: 64,
        level: "normal",
        resets: "resets in 3 d 2 h",
        fetched: null,
        unconfirmed: false,
        pace: null,
        runs_out: false,
      },
      {
        label: "5-hour (Gemini)",
        used_pct: 25,
        level: "normal",
        resets: "resets in 3 h 30 min",
        fetched: null,
        unconfirmed: false,
        pace: null,
        runs_out: false,
      },
      {
        label: "Weekly (Gemini)",
        used_pct: 6.22,
        level: "normal",
        resets: "resets in 5 d 9 h",
        fetched: null,
        unconfirmed: false,
        pace: null,
        runs_out: false,
      },
    ],
    extra_usage: null,
    note: null,
    // Its own words: QuotaDial never reads Antigravity's sign-in, the block to add, and a limit
    // Antigravity CLI gives an amount for and no share of (D-126).
    sign_in:
      "QuotaDial never reads Antigravity's sign-in. It keeps only the numbers Antigravity CLI's " +
      'status line hands it once `"statusLine": { "type": "command", "command": "quotadial ' +
      'antigravity status-line", "stack_with_default": true }` is in ' +
      "~/.gemini/antigravity-cli/settings.json. Antigravity CLI gives an amount left and no share " +
      'for one more limit, which QuotaDial leaves out of its windows: "Credits", 120 left, ' +
      "resetting at 2026-05-01T00:00:00Z.",
  },
  // An account whose windows were not read together: the 5-hour window from the API a minute ago,
  // the weekly one from the Codex CLI's local logs nine minutes ago. No provider makes a card like
  // it today, since each snapshot is one answer and its windows share their source and their
  // time; this one holds the layout for the day one does: the card says nothing once, and each
  // window says its own (D-135).
  {
    key: "codex:both@example.com",
    alias: null,
    summary: "Codex both@example.com (Plus): 5-hour 31%, resets in 2 h 20 min",
    provider: "Codex",
    name: "both@example.com",
    plan: "Plus",
    status: "ok",
    ok: true,
    fetched: null,
    windows: [
      {
        label: "5-hour",
        used_pct: 31,
        level: "normal",
        resets: "resets in 2 h 20 min",
        fetched: "from the API 1 min ago",
        unconfirmed: false,
        pace: null,
        runs_out: false,
      },
      {
        label: "Weekly",
        used_pct: 18,
        level: "normal",
        resets: "resets in 4 d 6 h",
        fetched: "from local logs 9 min ago",
        unconfirmed: false,
        pace: null,
        runs_out: false,
      },
    ],
    extra_usage: null,
    note: null,
    sign_in: CODEX_SIGN_IN,
  },
  // An account with one window, as a Codex Pro sign-in has (its single weekly window, D-039): its
  // picture to share has one column, the widest chart and the largest number, which no other
  // account of the copy shows (D-137).
  {
    key: "codex:solo@example.com",
    alias: "Personal",
    summary: "Codex Personal: Weekly 63%",
    provider: "Codex",
    name: "solo@example.com",
    plan: "Pro",
    status: "ok",
    ok: true,
    fetched: "from the API 2 min ago",
    windows: [
      {
        label: "Weekly",
        used_pct: 63,
        level: "normal",
        resets: "resets in 3 d 5 h",
        fetched: null,
        unconfirmed: false,
        pace: "at 0.4%/h, lasts until it resets",
        runs_out: false,
      },
    ],
    extra_usage: null,
    note: null,
    sign_in: CODEX_SIGN_IN,
  },
];

/**
 * Three more accounts, which with those above make ten, as many as a person with a few
 * subscriptions each for work and for play has, and enough to carry the popover well past its
 * tallest, so that its toolbar has to stay put while they scroll (D-136). The copy of them was
 * called Nine, before an account with one window was added to those above (D-137). The first has,
 * in a place of its own, text from outside with no spaces in it that a window must break rather
 * than cut off (D-139): a window named for a model (its label, in the popover and in History's
 * caption and heatmap) and a sign-in panel that names a folder; the second a plan (its capsule);
 * the third an address whose name before the @ is a part wider than the window (D-141).
 * @type {Meter["accounts"]}
 */
const MORE_ACCOUNTS = [
  {
    key: "claude:research@example.com",
    alias: "Research",
    summary: "Claude Research: Weekly 22%",
    provider: "Claude",
    name: "research@example.com",
    plan: "Max 20x",
    status: "ok",
    ok: true,
    fetched: "from the API 4 min ago",
    windows: [
      {
        label: "Weekly",
        used_pct: 22.4,
        level: "normal",
        resets: "resets in 5 d 7 h",
        fetched: null,
        unconfirmed: false,
        pace: "at 0.2%/h, lasts until it resets",
        runs_out: false,
      },
      {
        label: "Session",
        used_pct: 4.1,
        level: "normal",
        resets: "resets in 3 h 52 min",
        fetched: null,
        unconfirmed: false,
        pace: null,
        runs_out: false,
      },
      {
        label: `Weekly (${LONG_MODEL})`,
        used_pct: 12.5,
        level: "normal",
        resets: "resets in 5 d 7 h",
        fetched: null,
        unconfirmed: false,
        pace: null,
        runs_out: false,
      },
    ],
    extra_usage: null,
    note: null,
    sign_in: `${CLAUDE_SIGN_IN} Its folder is ${LONG_FOLDER}.`,
  },
  {
    key: "codex:ops@example.com",
    alias: "Ops",
    summary: "Codex Ops: Weekly 41%",
    provider: "Codex",
    name: "ops@example.com",
    plan: LONG_PLAN,
    status: "ok",
    ok: true,
    fetched: "from the API 2 min ago",
    windows: [
      {
        label: "5-hour",
        used_pct: 8,
        level: "normal",
        resets: "resets in 4 h 12 min",
        fetched: null,
        unconfirmed: false,
        pace: null,
        runs_out: false,
      },
      {
        label: "Weekly",
        used_pct: 41,
        level: "normal",
        resets: "resets in 3 d 20 h",
        fetched: null,
        unconfirmed: false,
        pace: "at 0.4%/h, lasts until it resets",
        runs_out: false,
      },
    ],
    extra_usage: null,
    note: null,
    sign_in: CODEX_SIGN_IN,
  },
  {
    key: `claude:${LONG_LOCAL_PART}@example.com`,
    alias: null,
    summary: `Claude ${LONG_LOCAL_PART}@example.com: Weekly 63%`,
    provider: "Claude",
    name: `${LONG_LOCAL_PART}@example.com`,
    plan: "Pro",
    status: "ok",
    ok: true,
    fetched: "from the API 3 min ago",
    windows: [
      {
        label: "Weekly",
        used_pct: 63,
        level: "normal",
        resets: "resets in 1 d 2 h",
        fetched: null,
        unconfirmed: false,
        pace: "at 0.9%/h, lasts until it resets",
        runs_out: false,
      },
      {
        label: "Session",
        used_pct: 17.5,
        level: "normal",
        resets: "resets in 1 h 5 min",
        fetched: null,
        unconfirmed: false,
        pace: null,
        runs_out: false,
      },
    ],
    extra_usage: null,
    note: null,
    sign_in: CLAUDE_SIGN_IN,
  },
];

/**
 * Every account above and three more: the Many copy's. Its first account, whose window sets the
 * number under the summary, has a name the person gave it with no spaces in it, which the summary's
 * line, the card's heading and History's card each carry, and Gemini CLI's account has a note that
 * names a folder (D-139).
 * @type {Meter["accounts"]}
 */
const MANY_ACCOUNTS = [
  { ...ACCOUNTS[0], alias: LONG_ALIAS, summary: `Claude ${LONG_ALIAS}: Weekly 91%` },
  ...ACCOUNTS.slice(1, 3),
  { ...ACCOUNTS[3], note: `${ACCOUNTS[3].note ?? ""} QuotaDial looked for its sign-in in ${LONG_FOLDER}.` },
  ...ACCOUNTS.slice(4),
  ...MORE_ACCOUNTS,
];

/**
 * An account as the example build sends it (D-117): called Example under the provider Example, so
 * the app leaves its name out and the provider shows once in its heading and summary.
 * @type {Meter["accounts"][number]}
 */
const EXAMPLE_ACCOUNT = {
  key: "fake example-1 -",
  alias: null,
  summary: "Example (Claude Max 5x): Weekly 91%, resets in 2 d 4 h",
  provider: "Example",
  name: "",
  plan: "Claude Max 5x",
  status: "ok",
  ok: true,
  // One window, which the card says it for as it does for any window.
  fetched: "from the example build 1 min ago",
  windows: [
    {
      label: "Weekly",
      used_pct: 91,
      level: "high",
      resets: "resets in 2 d 4 h",
      fetched: null,
      unconfirmed: false,
      pace: null,
      runs_out: false,
    },
  ],
  extra_usage: null,
  note: null,
  sign_in:
    "An example account, made up by QuotaDial's example build for pictures of the app. It has no " +
    "sign-in, and QuotaDial asks no one for its numbers.",
};

/** @type {Meter} */
const METER = {
  percent: 91,
  level: "high",
  // As the app names the account whose window sets the number: by the name given it (D-132).
  headline: "Weekly · Claude Work, the team account we share across two offices · resets in 2 d 4 h",
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

/**
 * What Settings lists to name, for each of `accounts`.
 *
 * @param {Meter["accounts"]} accounts
 * @returns {Settings["accounts"]}
 */
function choices(accounts) {
  return accounts.map((account) => ({
    key: account.key,
    provider: account.provider,
    name: account.name,
    alias: account.alias ?? "",
  }));
}

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
    { id: "gemini", name: "Gemini CLI", shown: true },
    { id: "antigravity", name: "Antigravity", shown: true },
  ],
  accounts: choices(ACCOUNTS),
  licence: { text: "QuotaDial Pro, licence 1234.", kept: true, adds: [] },
  check_updates: true,
};

const DAY_NAMES = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

/** Windows that reset within a day: a session (Claude) or a 5-hour window (Codex). */
const SESSIONS = ["Session", "5-hour"];

/** The most a session window reached on each day of the week, Monday first. */
const SESSION_PEAKS = [56, 64, 71, 58, 49, 6, 0];

/**
 * A window's readings over `days` days, four a day: a weekly window climbs to its share now, and
 * a session window fills during each day and resets, so its highest is above its latest.
 *
 * @param {Meter["accounts"][number]["windows"][number]} window
 * @param {number} days
 * @returns {[number, number][]}
 */
function readings(window, days) {
  const count = days * 4;
  return Array.from({ length: count }, (_, index) => {
    const along = index / (count - 1);
    if (!SESSIONS.includes(window.label)) {
      return [along, Math.min(100, (index * window.used_pct) / (count - 1))];
    }
    // The last day fills to the share now.
    const peak = index >= count - 4 ? window.used_pct : SESSION_PEAKS[Math.floor(index / 4) % 7];
    return [along, Math.round((peak * ((index % 4) + 1)) / 0.4) / 10];
  });
}

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
    alias: account.alias,
    name: account.name,
    label: label(account),
    windows: account.windows.map((window) => {
      const points = readings(window, days);
      const latest = window.used_pct;
      const highest = Math.max(latest, ...points.map(([, used]) => used));
      return {
        label: window.label,
        points,
        latest,
        highest,
        summary: `${window.label}: latest ${Math.floor(latest)}%, highest ${Math.floor(highest)}% in the last ${days} days`,
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
 * @param {Meter["accounts"]} [accounts] the accounts it charts, those with windows
 * @returns {HistoryView}
 */
function history(days, shareLeftOut, accounts = ACCOUNTS) {
  return {
    days,
    span: `The last ${days} days.`,
    dates: DATES[days],
    accounts: accounts.filter((account) => account.windows.length > 0).map((account) =>
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
        "api.anthropic.com, Anthropic's, with your Claude Code sign-in, for each Claude account's limits, at most once every 5 minutes per account.",
        "chatgpt.com, OpenAI's, with your Codex CLI sign-in, for your ChatGPT/Codex limits, at most once every 2 minutes.",
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

/**
 * QuotaDial Pro with ten accounts (D-136): the popover's accounts, and each panel's, run far past
 * the window's tallest height, so the check sees the toolbar stay at the window's foot while they
 * scroll, from the top to the last card. It is also the copy whose text from outside is as long as
 * it gets in every place the window shows some (D-139), so that the check's rule that nothing is cut
 * off holds each of them: an address (the Codex account's, above), a name the person gave an
 * account, a plan, a window's label, a folder and a command in a problem, a provider's incident, a
 * webhook's host and an alert the person wrote.
 *
 * @type {Copy}
 */
export const many = {
  ...pro,
  meter: {
    ...pro.meter,
    headline: `Weekly · ${LONG_ALIAS} · resets in 2 d 4 h`,
    accounts: MANY_ACCOUNTS,
    problems: [
      ...pro.meter.problems,
      `The history could not be opened at ${LONG_FOLDER}: it was damaged, so a new one was started. ` +
        `If other users can read it, run \`${LONG_COMMAND}\`.`,
    ],
    incidents: [...pro.meter.incidents, `Codex: ${LONG_INCIDENT}`],
    costs: [
      ...pro.meter.costs,
      {
        provider: "Codex",
        summary: "$312 at API prices over the last 7 days, from 8,120 requests",
        detail: "From the logs read 2 min ago, at the prices of 1 October 2026.",
        models: [`${LONG_MODEL}: $201`, "Codex Mini: $111"],
        notes: [],
      },
    ],
  },
  settings: {
    ...SETTINGS,
    alerts: [
      ...SETTINGS.alerts,
      {
        id: 3,
        written: `warn me about ${LONG_ALIAS}`,
        read_as: `when ${LONG_ALIAS} reaches 80%`,
        enabled: true,
        waits: false,
      },
    ],
    alert_targets: [...SETTINGS.alert_targets, LONG_HOST],
    alert_problems: [
      `${LONG_FOLDER} holds webhook addresses, which work like passwords, and other users can read it; ` +
        `run \`${LONG_COMMAND}\`.`,
    ],
    accounts: choices(MANY_ACCOUNTS),
  },
  history: { 7: history(7, null, MANY_ACCOUNTS), 30: history(30, null, MANY_ACCOUNTS) },
  // What Help and an update on offer say that is longer than a window's words: lines of the real
  // changelog, which name files and commands, and the real privacy text's longest host, in a copy
  // whose update is on offer, so that the popover's card for it and About's notes carry them too.
  help: {
    ...HELP,
    whats_new: {
      ...HELP.whats_new,
      sections: [
        ...HELP.whats_new.sections,
        {
          heading: "Changed",
          items: [
            "Each release's files also come under a name without the version, such as QuotaDial_universal.dmg, so one link always fetches the newest full release.",
            "The command that signs a Claude Code folder in again works when pasted, from doctor too: CLAUDE_CONFIG_DIR=~/'.claude-work' claude, as docs/INSTALL.md says.",
          ],
        },
      ],
    },
    privacy: [
      ...HELP.privacy,
      {
        kind: "list",
        items: [
          "release-assets.githubusercontent.com, GitHub's, where an update's files are fetched from, only when you choose Update.",
        ],
      },
    ],
  },
  update: {
    text: "QuotaDial 0.4.0 is available. This copy is 0.3.0.",
    available: "0.4.0",
    notes: [
      "Each release's files also come under a name without the version, such as QuotaDial_universal.dmg, so one link always fetches the newest full release.",
    ],
    busy: false,
    progress: null,
  },
  forScrolling: true,
};

/**
 * A Claude account with seven windows, as one has when Anthropic scopes weekly limits to a model or
 * a feature: the session, the week, Opus, Sonnet, OAuth apps, Cowork and one limit more, three of
 * them at 90% or more and one at 100%. Its picture to share has seven narrow columns, where each
 * share must be written whole, never cut short, in each system's own font, whose widths are not in
 * proportion to its size (the manager on #73, 2026-10-27).
 * @type {Meter["accounts"][number]}
 */
const CROWDED_ACCOUNT = {
  key: "claude:max@example.com",
  alias: null,
  summary: "Claude max@example.com (Max 20x): Weekly 100%, resets in 2 d 6 h",
  provider: "Claude",
  name: "max@example.com",
  plan: "Max 20x",
  status: "ok",
  ok: true,
  fetched: "from the API 1 min ago",
  windows: [
    ["Session", 64, "normal", "resets in 1 h 40 min"],
    ["Weekly", 100, "high", "resets in 2 d 6 h"],
    ["Weekly (Opus)", 7, "normal", "resets in 2 d 6 h"],
    ["Weekly (Sonnet)", 99, "high", "resets in 2 d 6 h"],
    ["Weekly (OAuth apps)", 12, "normal", "resets in 2 d 6 h"],
    ["Weekly (Cowork)", 38, "normal", "resets in 2 d 6 h"],
    ["Weekly (Example model)", 91, "high", "resets in 2 d 6 h"],
  ].map(([label, used_pct, level, resets]) => ({
    label: String(label),
    used_pct: Number(used_pct),
    level: /** @type {"normal" | "high"} */ (level),
    resets: String(resets),
    fetched: null,
    unconfirmed: false,
    pace: null,
    runs_out: false,
  })),
  extra_usage: null,
  note: null,
  sign_in: CLAUDE_SIGN_IN,
};

/**
 * `chart` as a window reads that was full in the middle of the period and has reset since: its
 * readings climb to 100%, fall to nothing and climb to its share now, so that its highest share,
 * 100%, is more than its latest, which makes the widest "highest" line a picture to share draws.
 * A window at 100% now has no higher share.
 *
 * @param {number} days
 * @returns {(chart: WindowChart) => WindowChart}
 */
const fullBefore = (days) => (chart) => {
  if (Math.floor(chart.latest) >= 100) {
    return chart;
  }
  /** @type {[number, number][]} */
  const points = Array.from({ length: 28 }, (_, index) => [
    index / 27,
    index <= 15 ? (100 * index) / 15 : (chart.latest * (index - 16)) / 11,
  ]);
  return {
    ...chart,
    points,
    highest: 100,
    summary: `${chart.label}: latest ${Math.floor(chart.latest)}%, highest 100% in the last ${days} days`,
  };
};

/**
 * `chart` as a window reads that has been full since the middle of the period, when it is at 100%
 * now: its readings climb to 100% by half way and stay there. A picture to share draws the soft
 * fill under a chart's line up to the line, so for the second half of this chart its dashed 90%
 * line runs over that fill, which the window check judges it on as well as on the card (the manager
 * on #73, 2026-10-27, 11:26 UTC); a window that only passes 90% crosses the fill too briefly for its
 * dashes to be judged there. A window below 100% now is `fullBefore`'s.
 *
 * @param {WindowChart} chart
 * @returns {WindowChart}
 */
const fullSince = (chart) => {
  if (Math.floor(chart.latest) < 100) {
    return chart;
  }
  const last = chart.points.length - 1;
  /** @type {[number, number][]} */
  const points = chart.points.map(([along], index) => [along, Math.min(100, (200 * index) / last)]);
  return { ...chart, points };
};

/**
 * The crowded account's History over `days`.
 *
 * @param {7 | 30} days
 * @returns {HistoryView}
 */
function crowdedHistory(days) {
  const view = history(days, null, [CROWDED_ACCOUNT]);
  return {
    ...view,
    accounts: view.accounts.map((account) => ({
      ...account,
      windows: account.windows.map(fullBefore(days)).map(fullSince),
    })),
  };
}

/**
 * Pro with the crowded account alone, over History alone: its pictures to share, each with seven
 * columns, in both themes, one of them full since the middle of the period (`fullSince`).
 *
 * @type {Copy}
 */
export const crowded = {
  ...pro,
  meter: {
    ...pro.meter,
    percent: 100,
    level: "high",
    headline: "Weekly · Claude max@example.com · resets in 2 d 6 h",
    accounts: [CROWDED_ACCOUNT],
  },
  settings: { ...SETTINGS, accounts: choices([CROWDED_ACCOUNT]) },
  history: { 7: crowdedHistory(7), 30: crowdedHistory(30) },
  panels: ["history"],
};

/** @type {Copy} */
export const free = {
  meter: {
    ...METER,
    // Codex's account as an older daemon sends it, saying nothing of its sign-in, so no panel; and
    // an example account, whose provider shows once.
    accounts: [
      ...ACCOUNTS.map((account) => (account.provider === "Codex" ? { ...account, sign_in: "" } : account)),
      EXAMPLE_ACCOUNT,
    ],
    left_out: [
      "Showing every account is part of QuotaDial Pro: quotadial.com/pro",
      "The week's cost at API prices is part of QuotaDial Pro: quotadial.com/pro",
    ],
  },
  settings: {
    ...SETTINGS,
    alerts: SETTINGS.alerts.map((alert) => ({ ...alert, waits: alert.id === 2 })),
    alert_targets: [],
    alert_problems: [
      "config.json names a webhook that is not https; QuotaDial leaves it out.",
      "~/Library/Application Support/quotadial/config.json holds webhook addresses, which work like " +
        "passwords, and other users can read it; run " +
        "`chmod 600 ~/'Library/Application Support/quotadial/config.json'`.",
    ],
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
