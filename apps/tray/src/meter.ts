/** The meter the app sends: the same shape as `Meter` in `src-tauri/src/meter.rs`. */
export interface Meter {
  /** The tightest window's share used, rounded down, if any account has numbers. */
  percent: number | null;
  level: "unknown" | "normal" | "warning" | "high";
  /** One line per account, such as "Claude user@example.com (max): Session 42%, resets in 2 h 10 min". */
  accounts: string[];
  /** What needs the user's attention, each naming the fix. */
  problems: string[];
}
