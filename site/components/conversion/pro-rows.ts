/**
 * What QuotaDial Pro adds, one row per feature, in the order the app lists them. `feature` is the
 * app's own phrase (crates/core/src/tier.rs, `ProFeature::what`) up to its "not" half, which `free`
 * says as what a Free copy has (docs/CLI.md, QuotaDial Free and Pro). `changes` is what the feature
 * does for the reader. test/conversion.test.ts holds the seven to lib/data/pro.ts.
 */

export interface ProRow {
  /** What the row belongs to; neighbouring rows of one group share a band. */
  group: "seen" | "warnings" | "cost" | "agents";
  feature: string;
  free: string;
  changes: string;
}

export const PRO_ROWS: readonly ProRow[] = [
  {
    group: "seen",
    feature: "Every account on this machine",
    free: "One account per provider",
    changes: "Every Claude subscription stored here shows in the window, so you can keep an eye on each one.",
  },
  {
    group: "seen",
    feature: "A year of history",
    free: "30 days of history",
    changes: "How a window filled over many weeks stays on your machine for a year.",
  },
  {
    group: "warnings",
    feature: "Alerts before a window runs out, before a reset, and for provider incidents",
    free: "Refused when you add one",
    changes:
      "Write “tell me 30 minutes before the weekly window runs out” and QuotaDial tells you, at the pace the window is going.",
  },
  {
    group: "warnings",
    feature: "Alerts sent to webhooks and Slack",
    free: "System notifications only",
    changes: "An alert can also go to a webhook or a Slack channel, as well as to your system's notifications.",
  },
  {
    group: "cost",
    feature: "The week's cost at API prices",
    free: "The session logs are not read",
    changes:
      "Under your accounts, QuotaDial shows what the last 7 days of Claude Code and Codex use would have cost at each provider's API prices.",
  },
  {
    group: "agents",
    feature: "Headroom through the local API and MCP",
    free: "The API answers 403 and the MCP tool returns an error",
    changes: "An agent can ask which account has the most room left before it starts a long job.",
  },
  {
    group: "agents",
    feature: "Alert rules through the local API and MCP",
    free: "Refused the same way",
    changes: "An agent can read your alert rules, add one, switch one off or remove it.",
  },
];
