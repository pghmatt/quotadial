/**
 * QuotaDial Free and Pro, in the words the app itself uses (crates/core/src/tier.rs,
 * `ProFeature::what`, and README.md). No price: Lemon Squeezy's checkout shows it (D-098).
 */

/**
 * What QuotaDial Pro adds, one feature each, as Settings lists it on a Free copy: `ProFeature::ALL`
 * in `ProFeature::what`'s words, which test/pro.test.ts reads from tier.rs and holds this to.
 */
export const PRO_ADDS = [
  "every account on this machine, not one per provider",
  "a year of history, not 30 days",
  "a picture of an account's history to share",
  "alerts before a window runs out, before a reset, and for provider incidents",
  "alerts sent to webhooks and Slack",
  "the week's cost at API prices",
  "headroom through the local API and MCP",
  "alert rules through the local API and MCP",
] as const;

/**
 * Each Pro feature alone, without what Free has instead: "every account on this machine". For a
 * page that shows FREE_HAS beside it, where "not one per provider" would say the same thing twice.
 */
export const PRO_FEATURES = PRO_ADDS.map((what) => what.split(", not ")[0]);

/** What QuotaDial Free has, with no key. */
export const FREE_HAS = [
  "one account per provider",
  "every window with its pace and reset",
  "30 days of history",
  "alerts for a share used, a reset and a sign-in",
  "the providers' status pages",
  "read-only status for agents through the command, the local API and MCP",
] as const;
