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
}
