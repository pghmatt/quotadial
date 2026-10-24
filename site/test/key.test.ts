import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import { onRequestPost } from "../functions/api/key.ts";
import { type Answer, type Env, issue } from "../lib/key.ts";
import { type Fetch, VALIDATE_URL } from "../lib/lemonsqueezy.ts";

/** Lemon Squeezy's documented answer for a valid key (test/fixtures/README.md). */
const ACTIVE = JSON.parse(readFileSync(new URL("fixtures/validate-active.json", import.meta.url), "utf8"));
const RECEIPT_KEY = ACTIVE.license_key.key as string;

/** The test key from `crates/core/src/licence.rs`, as the PEM file `quotadial-issue keygen` writes. */
function testPem(): string {
  const der = Uint8Array.of(
    0x30, 0x2e, 0x02, 0x01, 0x00, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x04, 0x22, 0x04,
    0x20, ...new TextEncoder().encode("QuotaDial's licence test key 001"),
  );
  return `-----BEGIN PRIVATE KEY-----\n${btoa(String.fromCharCode(...der))}\n-----END PRIVATE KEY-----\n`;
}

const ENV: Env = { LICENCE_ISSUING_KEY: testPem(), LS_STORE_ID: "1", LS_PRODUCT_ID: "4" };

/** A fetch that answers `body` with `status`, and remembers what it was asked. */
function answering(status: number, body: unknown): Fetch & { asked: { url: string; init: RequestInit }[] } {
  const asked: { url: string; init: RequestInit }[] = [];
  const fetcher = async (url: string, init: RequestInit) => {
    asked.push({ url, init });
    return new Response(typeof body === "string" ? body : JSON.stringify(body), { status });
  };
  return Object.assign(fetcher, { asked });
}

/** The documented answer with `change` applied to it. */
function changed(change: (answer: typeof ACTIVE) => void): typeof ACTIVE {
  const answer = structuredClone(ACTIVE);
  change(answer);
  return answer;
}

function error(answer: Answer): string {
  assert.ok("error" in answer.body, JSON.stringify(answer.body));
  return answer.body.error;
}

test("a paid receipt's key becomes a QuotaDial key, the same each time", async () => {
  const fetcher = answering(200, ACTIVE);

  const first = await issue(RECEIPT_KEY, ENV, fetcher);
  const again = await issue(` ${RECEIPT_KEY}\n`, ENV, fetcher);

  assert.equal(first.status, 200);
  assert.deepEqual(first, again, "signing is deterministic, so the page can be visited again");
  assert.ok("key" in first.body);
  assert.deepEqual(first.body.licence, { id: "ls-1", issued: "2021-01-24", until: "2022-01-24" });
  const [format, body, signature] = first.body.key.split(".");
  assert.equal(format, "QD1");
  const decoded = JSON.parse(atob(body.replaceAll("-", "+").replaceAll("_", "/")));
  assert.deepEqual(decoded, { id: "ls-1", issued: "2021-01-24", plan: "pro", until: "2022-01-24" });
  assert.equal(signature.length, 86, "a 64-byte signature");

  const { url, init } = fetcher.asked[0];
  assert.equal(url, VALIDATE_URL);
  assert.equal(init.method, "POST");
  assert.equal(init.redirect, "error");
  assert.equal(String(init.body), `license_key=${RECEIPT_KEY}`);
});

test("a new key, valid with no activations, holds; a licence that does not end has no until", async () => {
  const fresh = changed((answer) => {
    answer.license_key.status = "inactive";
    answer.license_key.expires_at = null;
  });

  const answer = await issue(RECEIPT_KEY, ENV, answering(200, fresh));

  assert.equal(answer.status, 200);
  assert.ok("licence" in answer.body);
  assert.deepEqual(answer.body.licence, { id: "ls-1", issued: "2021-01-24" });
});

test("each refusal is one sentence that says what to do", async () => {
  const cases: [string, Fetch, number, string][] = [
    ["expired", answering(200, changed((a) => (a.license_key.status = "expired"))), 403, "Renewing it"],
    ["disabled", answering(200, changed((a) => (a.license_key.status = "disabled"))), 403, "turned off"],
    ["another store", answering(200, changed((a) => (a.meta.store_id = 9))), 403, "another product"],
    ["another product", answering(200, changed((a) => (a.meta.product_id = 9))), 403, "another product"],
    [
      "unknown to Lemon Squeezy",
      answering(404, { valid: false, error: "license_key not found.", license_key: null, instance: null, meta: null }),
      403,
      "doesn't know that key",
    ],
    ["down", answering(503, "Service Unavailable"), 502, "couldn't be reached"],
    ["not JSON", answering(200, "<html>"), 502, "couldn't be read"],
    ["a strange date", answering(200, changed((a) => (a.license_key.created_at = "yesterday"))), 502, "couldn't be read"],
    [
      "unreachable",
      async () => {
        throw new TypeError("network down");
      },
      502,
      "couldn't be reached",
    ],
  ];
  for (const [name, fetcher, status, words] of cases) {
    const answer = await issue(RECEIPT_KEY, ENV, fetcher);
    assert.equal(answer.status, status, name);
    assert.ok(error(answer).includes(words), `${name}: ${error(answer)}`);
  }
});

test("text that is not a receipt's key is refused without asking Lemon Squeezy", async () => {
  const fetcher = answering(200, ACTIVE);

  for (const text of ["", "hello", "QD1.abc.def", `${RECEIPT_KEY}x`]) {
    const answer = await issue(text, ENV, fetcher);
    assert.equal(answer.status, 400, text);
  }
  assert.equal(fetcher.asked.length, 0);
});

test("with no issuing key or store set, the page says it isn't set up yet", async () => {
  for (const env of [{}, { ...ENV, LICENCE_ISSUING_KEY: undefined }, { ...ENV, LS_PRODUCT_ID: "" }]) {
    const answer = await issue(RECEIPT_KEY, env, answering(200, ACTIVE));
    assert.equal(answer.status, 503);
    assert.ok(error(answer).includes("isn't set up yet"));
  }
  const broken = await issue(RECEIPT_KEY, { ...ENV, LICENCE_ISSUING_KEY: "not a key" }, answering(200, ACTIVE));
  assert.equal(broken.status, 503);
});

test("the function answers JSON that is never cached", async () => {
  const real = globalThis.fetch;
  globalThis.fetch = (async () => new Response(JSON.stringify(ACTIVE))) as typeof fetch;
  try {
    const request = new Request("https://quotadial.com/api/key", {
      method: "POST",
      body: JSON.stringify({ licence: RECEIPT_KEY }),
    });
    const response = await onRequestPost({ request, env: ENV });

    assert.equal(response.status, 200);
    assert.equal(response.headers.get("Cache-Control"), "no-store");
    assert.equal(response.headers.get("Content-Type"), "application/json");
    const answer = await response.json();
    assert.ok(String(answer.key).startsWith("QD1."));

    const unreadable = await onRequestPost({
      request: new Request("https://quotadial.com/api/key", { method: "POST", body: "not json" }),
      env: ENV,
    });
    assert.equal(unreadable.status, 400);
  } finally {
    globalThis.fetch = real;
  }
});
