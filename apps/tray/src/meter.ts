/** The meter the app sends: the same shape as `Meter` in `src-tauri/src/meter.rs`. */
export interface Meter {
  /** The tightest window's share used, rounded down, if any account has numbers. */
  percent: number | null;
  level: Level;
  /** What that number is: its window, the account it belongs to and its reset, such as "Weekly ·
   * Claude Work · resets in 1 d 15 h". */
  headline: string | null;
  accounts: Account[];
  /** What needs the user's attention and belongs to no account, each naming the fix. */
  problems: string[];
  /** The providers whose status pages report an incident, such as "Codex: Partial System
   * Degradation". */
  incidents: string[];
  /** Where the numbers come from, such as "Running the daemon, port 47115". */
  origin: string | null;
  /** What each provider's week would have cost at API prices, while the daemon reads the logs. */
  costs: Cost[];
  /** What this copy leaves out because it is QuotaDial Free, a line each. */
  left_out: string[];
}

/** What a provider's week would have cost at API prices. */
export interface Cost {
  /** Such as "Claude". */
  provider: string;
  /** Such as "$1,234 at API prices over the last 7 days, from 51,234 requests". */
  summary: string;
  /** When the logs were read, and the prices' date. */
  detail: string;
  /** Each model's part, the costliest first, when there is more than one model. */
  models: string[];
  /** What the cost leaves out. */
  notes: string[];
}

export type Level = "unknown" | "normal" | "warning" | "high";

export interface Account {
  /** As the settings name the account. */
  key: string;
  /** The name the user gave the account, shown in place of `name`. */
  alias: string | null;
  summary: string;
  /** Such as "Claude". */
  provider: string;
  /** Such as the account's email address; empty when it says nothing beyond `provider`, as for an
   * account called Example under the provider Example, so the provider shows once. */
  name: string;
  plan: string | null;
  /** Such as "ok", "stale" or "sign-in required". */
  status: string;
  ok: boolean;
  /** Tightest first. */
  windows: Window[];
  extra_usage: string | null;
  /** Why the status is not ok, naming the fix. */
  note: string | null;
  /** What QuotaDial does with this account's sign-in, in its provider's words; empty for an account
   * from an older daemon, which says none, and then the panel is left out. */
  sign_in: string;
}

export interface Window {
  label: string;
  /** From 0 to 100. */
  used_pct: number;
  level: Level;
  /** Such as "resets in 2 h 10 min". */
  resets: string;
  /** Such as "from the API 4 min ago". */
  fetched: string;
  unconfirmed: boolean;
  /** Such as "at 8.5%/h, runs out in 1 h 30 min, 40 min before it resets", once there is enough
   * history to go on. */
  pace: string | null;
  /** Whether, at that pace, the window runs out before it resets. */
  runs_out: boolean;
}

/** The settings, as `settings` returns them and `set_settings` takes them. */
export interface Settings {
  /** Whether QuotaDial opens when you log in, as the system says. */
  launch_at_login: boolean;
  /** The alert rules, oldest first; they change through `add_alert`, `set_alert` and
   * `remove_alert`. */
  alerts: AlertChoice[];
  /** The hosts of the webhooks alerts are also sent to. */
  alert_targets: string[];
  /** What `config.json` could not be used for, each naming the fix. */
  alert_problems: string[];
  providers: ProviderChoice[];
  /** Every account QuotaDial knows now, with the name the user gave it. */
  accounts: AccountChoice[];
  /** Whether this copy is QuotaDial Free or Pro; it changes through `add_licence` and
   * `remove_licence`. */
  licence: LicenceView;
  /** Whether the app asks GitHub once a day for a newer version. */
  check_updates: boolean;
}

export interface LicenceView {
  /** One sentence, such as "QuotaDial Pro, licence 1234." */
  text: string;
  /** Whether a key is kept, which Settings then offers to remove. */
  kept: boolean;
  /** On QuotaDial Free, what Pro adds, one feature each; empty on Pro. */
  adds: string[];
}

export interface AlertChoice {
  id: number;
  /** The words it was written in. */
  written: string;
  /** What QuotaDial read it as, such as "when any window reaches 80%". */
  read_as: string;
  enabled: boolean;
  /** Whether it waits for QuotaDial Pro: a kind of rule only Pro checks, on QuotaDial Free. */
  waits: boolean;
}

export interface AccountChoice {
  key: string;
  /** Such as "Claude user@example.com". */
  label: string;
  /** The name given, or empty. */
  alias: string;
}

export interface ProviderChoice {
  id: string;
  name: string;
  shown: boolean;
}

/** The history view, as `history` returns it. */
export interface HistoryView {
  /** How many days back it goes. */
  days: number;
  /** What the charts span, such as "The last 7 days." */
  span: string;
  /** The days it covers, such as "19 to 25 October 2026". */
  dates: string;
  accounts: AccountHistory[];
  /** On QuotaDial Free, the line saying a picture of an account's history to share is part of
   * QuotaDial Pro, shown once where the Share image buttons would be; null on Pro. */
  share_left_out: string | null;
}

export interface AccountHistory {
  key: string;
  /** Such as "Claude". */
  provider: string;
  /** The name the user gave the account, or its email address; empty when that says nothing beyond
   * `provider`. */
  name: string;
  /** What a picture of this account's history calls it, such as "Claude Work" or "Claude Max":
   * its provider and the name given it, or its plan, and never its address. */
  label: string;
  windows: WindowChart[];
  heatmap: Heatmap;
}

export interface WindowChart {
  label: string;
  /** Each reading's place along the period, from 0 to 1, and its share used. */
  points: [number, number][];
  latest: number;
  highest: number;
  /** For screen readers, such as "Session: latest 8%, highest 11% in the last 7 days". */
  summary: string;
}

export interface Heatmap {
  /** The window the use is counted in, such as "Session". */
  window: string;
  /** A label per day, oldest first, such as "Mon 20". */
  days: string[];
  /** Per day, the share used in each of its 24 hours, local time. */
  cells: number[][];
  /** The largest cell, which the others are shaded against. */
  most: number;
  /** The same facts in words, a line per day, which a screen reader reads in place of the grid. */
  said: string[];
}

/** What the app's Help shows. */
export interface HelpView {
  /** The app's version, such as "0.3.0". */
  version: string;
  /** What changed in it, from the changelog built into the app. */
  whats_new: WhatsNew;
  /** What this copy is, such as "QuotaDial Pro, licence 1234." */
  licence: string;
  /** Where people write for help. */
  support: string;
  /** Whose work QuotaDial uses, and under which licence. */
  credits: string[];
  /** What QuotaDial reads and sends, the same text as the README's and the website's. */
  privacy: Block[];
}

/** A paragraph or a list of a text Help shows, in plain words. */
export type Block = { kind: "paragraph"; text: string } | { kind: "list"; items: string[] };

/** Where updates stand, as `updates` returns them and the `update` event sends them. */
export interface UpdateView {
  /** One sentence, such as "QuotaDial 0.2.0 is the newest version. Checked 3 min ago." */
  text: string;
  /** The newer version on offer, which `install_update` installs. */
  available: string | null;
  /** What changed in it, a line each. */
  notes: string[];
  /** Whether a check, a download or an install is under way. */
  busy: boolean;
  /** How far a download has got, from 0 to 100, once its size is known. */
  progress: number | null;
}

export interface WhatsNew {
  /** Such as "0.3.0", or "Unreleased" for a build between releases. */
  version: string;
  sections: ChangeSection[];
}

export interface ChangeSection {
  /** Such as "Added". */
  heading: string;
  items: string[];
}
