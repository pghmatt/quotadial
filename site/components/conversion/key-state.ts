/**
 * What the key page does with its address and with the key function's answers, apart from how it
 * looks, so the rules can be tested without a browser (lib/key.ts and functions/api/key.ts hold the
 * other side).
 */

import type { Licence } from "../../lib/licence.ts";

/** The receipt's key from an address fragment such as `#licence=<key>`, or null when there is none. */
export function receiptKey(fragment: string): string | null {
  const value = new URLSearchParams(fragment.startsWith("#") ? fragment.slice(1) : fragment).get("licence");
  return value === null ? null : value.trim();
}

export type KeyAnswer =
  | { ok: true; key: string; licence: Licence }
  | {
      ok: false;
      /** The sentence to show: the function's own when it gave one. */
      message: string;
      /** Whether asking again could help: Lemon Squeezy or the connection, not the key. */
      retry: boolean;
    };

export const UNREACHED = "The key page couldn't be reached. Check your connection and try again.";
export const UNREADABLE = "The key page's answer couldn't be read. Try again in a minute.";

/** The statuses `lib/key.ts` gives when Lemon Squeezy, not the key, is the problem. */
const TRY_AGAIN = new Set([502]);

/** Asks `/api/key` to turn the receipt's key into a QuotaDial key. */
export async function requestKey(
  receipt: string,
  fetcher: (url: string, init: RequestInit) => Promise<Response>,
  signal?: AbortSignal,
): Promise<KeyAnswer> {
  let response: Response;
  try {
    response = await fetcher("/api/key", {
      method: "POST",
      headers: { "Content-Type": "application/json", Accept: "application/json" },
      body: JSON.stringify({ licence: receipt }),
      cache: "no-store",
      signal,
    });
  } catch {
    return { ok: false, message: UNREACHED, retry: true };
  }
  let body: unknown;
  try {
    body = await response.json();
  } catch {
    return { ok: false, message: UNREADABLE, retry: true };
  }
  const answer = typeof body === "object" && body !== null ? (body as Record<string, unknown>) : {};
  if (response.ok && typeof answer.key === "string" && answer.key.startsWith("QD1.") && isLicence(answer.licence)) {
    return { ok: true, key: answer.key, licence: answer.licence };
  }
  if (!response.ok && typeof answer.error === "string" && answer.error.trim() !== "") {
    return { ok: false, message: answer.error, retry: TRY_AGAIN.has(response.status) };
  }
  return { ok: false, message: UNREADABLE, retry: true };
}

function isLicence(value: unknown): value is Licence {
  if (typeof value !== "object" || value === null) {
    return false;
  }
  const { id, issued, until } = value as Record<string, unknown>;
  return typeof id === "string" && typeof issued === "string" && (until === undefined || typeof until === "string");
}

/** A licence's day, `2026-10-25`, as people write it: `25 October 2026`. */
export function longDay(day: string): string {
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(day);
  if (!match) {
    return day;
  }
  const date = new Date(Date.UTC(Number(match[1]), Number(match[2]) - 1, Number(match[3])));
  return date.toLocaleDateString("en-GB", { day: "numeric", month: "long", year: "numeric", timeZone: "UTC" });
}
