/**
 * The key page's one job (#54, D-102): turn the key from a paid Lemon Squeezy receipt into a
 * QuotaDial licence key, signed with the issuing key. The page never stores anything: signing is
 * deterministic, so the same receipt always gives the same key, and the person can come back for it.
 */

import { type Fetch, validate } from "./lemonsqueezy.ts";
import { type Licence, sign, signingKey } from "./licence.ts";

/** What the page needs from its environment, set as Cloudflare secrets and variables. */
export interface Env {
  /** The issuing key's private half, as the PKCS #8 PEM file `quotadial-issue keygen` wrote. */
  LICENCE_ISSUING_KEY?: string;
  /** QuotaDial's store at Lemon Squeezy. */
  LS_STORE_ID?: string;
  /** QuotaDial Pro's product at Lemon Squeezy. */
  LS_PRODUCT_ID?: string;
}

/** The page's answer: a status and its JSON. */
export interface Answer {
  status: number;
  body: { key: string; licence: Licence } | { error: string };
}

const SUPPORT = "support@quotadial.com";

/** Lemon Squeezy's licence keys: a UUID, 8, 4, 4, 4 and 12 hex digits joined by dashes. */
const ORDER_KEY = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

/** A licence key's status that holds: `inactive` is valid with no activations, as every new key is. */
const HOLDS = new Set(["inactive", "active"]);

function refused(status: number, error: string): Answer {
  return { status, body: { error } };
}

/** The QuotaDial licence key for the receipt key `orderKey`, or why there is none. */
export async function issue(orderKey: string, env: Env, fetcher: Fetch): Promise<Answer> {
  if (!env.LICENCE_ISSUING_KEY || !env.LS_STORE_ID || !env.LS_PRODUCT_ID) {
    return refused(
      503,
      `The key page isn't set up yet. Write to ${SUPPORT} with your order number, and we'll send your key.`,
    );
  }
  const key = orderKey.trim();
  if (!ORDER_KEY.test(key)) {
    return refused(
      400,
      'That isn\'t the key from your Lemon Squeezy receipt. Open the receipt\'s "Get your QuotaDial key" link again.',
    );
  }
  const answer = await validate(key, fetcher);
  if (answer === "unreachable") {
    return refused(502, "Lemon Squeezy couldn't be reached just now. Try again in a minute.");
  }
  if (answer === "unreadable") {
    return refused(502, "Lemon Squeezy's answer couldn't be read. Try again in a minute.");
  }
  const licence = answer.license_key;
  if (!answer.valid || !licence || !answer.meta) {
    return refused(
      403,
      `Lemon Squeezy doesn't know that key. Open the link in your receipt again, or write to ${SUPPORT}.`,
    );
  }
  if (
    String(answer.meta.store_id) !== env.LS_STORE_ID.trim() ||
    String(answer.meta.product_id) !== env.LS_PRODUCT_ID.trim()
  ) {
    return refused(403, "That key is for another product, not QuotaDial Pro.");
  }
  if (licence.status === "expired") {
    return refused(403, "That licence has ended. Renewing it at quotadial.com/pro gives a new key.");
  }
  if (!HOLDS.has(licence.status)) {
    return refused(
      403,
      `That licence has been turned off, as after a refund. Write to ${SUPPORT} if that's a mistake.`,
    );
  }
  const issued = day(licence.created_at);
  const until = licence.expires_at === null ? undefined : day(licence.expires_at);
  if (!issued || until === null) {
    return refused(502, "Lemon Squeezy's answer couldn't be read. Try again in a minute.");
  }
  const signed: Licence = { id: `ls-${licence.id}`, issued, ...(until ? { until } : {}) };
  try {
    return {
      status: 200,
      body: { key: await sign(signed, await signingKey(env.LICENCE_ISSUING_KEY)), licence: signed },
    };
  } catch {
    return refused(503, `The key page can't sign keys just now. Write to ${SUPPORT}, and we'll send your key.`);
  }
}

/** The UTC day of an ISO 8601 time such as `2021-01-24T14:15:07.000000Z`, or null. */
function day(time: string): string | null {
  return /^\d{4}-\d{2}-\d{2}T/.test(time) ? time.slice(0, 10) : null;
}
