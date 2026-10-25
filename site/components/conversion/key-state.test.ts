import assert from "node:assert/strict";
import { test } from "node:test";

import { longDay, receiptKey, requestKey, UNREACHED, UNREADABLE } from "./key-state.ts";

const RECEIPT = "11111111-1111-4111-8111-111111111111";
const LICENCE = { id: "ls-1", issued: "2026-10-25" };

/** A fetch that answers `body` with `status`, and remembers what it was asked. */
function answering(status: number, body: unknown) {
  const asked: { url: string; init: RequestInit }[] = [];
  const fetcher = async (url: string, init: RequestInit) => {
    asked.push({ url, init });
    return new Response(typeof body === "string" ? body : JSON.stringify(body), { status });
  };
  return Object.assign(fetcher, { asked });
}

test("the receipt's key is read from the fragment, and only from `licence`", () => {
  assert.equal(receiptKey(`#licence=${RECEIPT}`), RECEIPT);
  assert.equal(receiptKey(`licence=${RECEIPT}&other=1`), RECEIPT);
  assert.equal(receiptKey(`#licence=%20${RECEIPT}%20`), RECEIPT, "spaces around a key are not part of it");
  assert.equal(receiptKey(""), null);
  assert.equal(receiptKey("#"), null);
  assert.equal(receiptKey("#key=abc"), null);
  assert.equal(receiptKey("#licence="), "", "an empty key is still a key the function answers for");
});

test("a key is sent as JSON to /api/key, never cached", async () => {
  const fetcher = answering(200, { key: "QD1.a.b", licence: LICENCE });
  await requestKey(RECEIPT, fetcher);
  assert.equal(fetcher.asked.length, 1);
  assert.equal(fetcher.asked[0].url, "/api/key");
  assert.equal(fetcher.asked[0].init.method, "POST");
  assert.equal(fetcher.asked[0].init.body, JSON.stringify({ licence: RECEIPT }));
  assert.equal(fetcher.asked[0].init.cache, "no-store");
});

test("a QuotaDial key and its licence come back as they were sent", async () => {
  const answer = await requestKey(RECEIPT, answering(200, { key: "QD1.abc.def", licence: { ...LICENCE, until: "2027-10-24" } }));
  assert.deepEqual(answer, { ok: true, key: "QD1.abc.def", licence: { ...LICENCE, until: "2027-10-24" } });
});

test("the function's own sentence is shown unchanged, and only Lemon Squeezy's trouble offers another try", async () => {
  const sentences = [
    [400, "That isn't the key from your Lemon Squeezy receipt. Open the receipt's \"Get your QuotaDial key\" link again.", false],
    [403, "That licence has ended. Renewing it at quotadial.com/pro gives a new key.", false],
    [403, "That key is for another product, not QuotaDial Pro.", false],
    [502, "Lemon Squeezy couldn't be reached just now. Try again in a minute.", true],
    [503, "The key page isn't set up yet. Write to support@quotadial.com with your order number, and we'll send your key.", false],
  ] as const;
  for (const [status, error, retry] of sentences) {
    assert.deepEqual(await requestKey(RECEIPT, answering(status, { error })), { ok: false, message: error, retry });
  }
});

test("a dropped connection, a page that is not JSON and a success with no key are each answered", async () => {
  const down = async () => {
    throw new TypeError("fetch failed");
  };
  assert.deepEqual(await requestKey(RECEIPT, down), { ok: false, message: UNREACHED, retry: true });
  assert.deepEqual(await requestKey(RECEIPT, answering(200, "<html>")), { ok: false, message: UNREADABLE, retry: true });
  assert.deepEqual(await requestKey(RECEIPT, answering(200, { key: "not a key", licence: LICENCE })), {
    ok: false,
    message: UNREADABLE,
    retry: true,
  });
  assert.deepEqual(await requestKey(RECEIPT, answering(200, { key: "QD1.a.b", licence: { id: 1 } })), {
    ok: false,
    message: UNREADABLE,
    retry: true,
  });
  assert.deepEqual(await requestKey(RECEIPT, answering(500, { oops: true })), { ok: false, message: UNREADABLE, retry: true });
});

test("a licence's day is written as people write it", () => {
  assert.equal(longDay("2026-10-25"), "25 October 2026");
  assert.equal(longDay("2027-01-01"), "1 January 2027");
  assert.equal(longDay("soon"), "soon");
});
