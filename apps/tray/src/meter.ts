/** The meter the app sends: the same shape as `Meter` in `src-tauri/src/meter.rs`. */
export interface Meter {
  /** The tightest window's share used, rounded down, if any account has numbers. */
  percent: number | null;
  level: Level;
  /** What that number is, such as "Codex Weekly, resets in 6 d 1 h". */
  headline: string | null;
  accounts: Account[];
  /** What needs the user's attention and belongs to no account, each naming the fix. */
  problems: string[];
  /** Where the numbers come from, such as "Running the daemon, port 47115". */
  origin: string | null;
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
  /** Such as the account's email address. */
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
  /** Where QuotaDial reads this account's sign-in. */
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
  providers: ProviderChoice[];
  /** Every account QuotaDial knows now, with the name the user gave it. */
  accounts: AccountChoice[];
}

export interface AlertChoice {
  id: number;
  /** The words it was written in. */
  written: string;
  /** What QuotaDial read it as, such as "when any window reaches 80%". */
  read_as: string;
  enabled: boolean;
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
