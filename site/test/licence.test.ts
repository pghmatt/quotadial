import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import { base64url, sign, signingKey, written } from "../lib/licence.ts";

/** The test key from `crates/core/src/licence.rs`, whose private half is public on purpose. */
const TEST_SEED = new TextEncoder().encode("QuotaDial's licence test key 001");

/** The test key as the PKCS #8 PEM file `quotadial-issue keygen` writes. */
function testPem(): string {
  const der = Uint8Array.of(
    0x30, 0x2e, 0x02, 0x01, 0x00, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x04, 0x22, 0x04,
    0x20, ...TEST_SEED,
  );
  let binary = "";
  for (const byte of der) {
    binary += String.fromCharCode(byte);
  }
  return `-----BEGIN PRIVATE KEY-----\n${btoa(binary)}\n-----END PRIVATE KEY-----\n`;
}

/** The two keys `docs/LICENSING.md` gives, which QuotaDial signs byte for byte the same. */
const DOCUMENTED = [
  {
    licence: { id: "test-0001", issued: "2026-10-24" },
    key: "QD1.eyJpZCI6InRlc3QtMDAwMSIsImlzc3VlZCI6IjIwMjYtMTAtMjQiLCJwbGFuIjoicHJvIn0.TCnQdA2XJTPnuGVV280SOgfy_BH4maU7Nj8s11TSw6nWbpPT_icIfL-nf155_ZQXnAnvGsKDQ14V3X-72YchDw",
  },
  {
    licence: { id: "test-0002", issued: "2026-10-24", until: "2027-10-24" },
    key: "QD1.eyJpZCI6InRlc3QtMDAwMiIsImlzc3VlZCI6IjIwMjYtMTAtMjQiLCJwbGFuIjoicHJvIiwidW50aWwiOiIyMDI3LTEwLTI0In0.tlTkpHCVT6JLE_iVJEanK4PgzTLBxrwz9IApnRFCHLPO87jnrYkVKrNZvnEAWKvfK_NG_az9TK3xkY9fa3k3Dw",
  },
];

test("the page signs a licence to the same key QuotaDial does", async () => {
  const key = await signingKey(testPem());
  const documented = readFileSync(new URL("../../docs/LICENSING.md", import.meta.url), "utf8");

  for (const { licence, key: expected } of DOCUMENTED) {
    assert.equal(await sign(licence, key), expected);
    assert.ok(documented.includes(`\n${expected}\n`), "the key is in docs/LICENSING.md");
  }
});

test("a licence is written with its fields in order and no spaces", () => {
  assert.equal(
    written({ id: "ls-1", issued: "2026-10-24", until: "2027-10-24" }),
    '{"id":"ls-1","issued":"2026-10-24","plan":"pro","until":"2027-10-24"}',
  );
  assert.equal(written({ id: "ls-1", issued: "2026-10-24" }), '{"id":"ls-1","issued":"2026-10-24","plan":"pro"}');
});

test("base64url has no padding and the URL-safe letters", () => {
  assert.equal(base64url(Uint8Array.of(0xfb, 0xff)), "-_8");
  assert.equal(base64url(new Uint8Array()), "");
});

test("only an Ed25519 key in PKCS #8 signs", async () => {
  await assert.rejects(signingKey("-----BEGIN PRIVATE KEY-----\nAAAA\n-----END PRIVATE KEY-----\n"));
  await assert.rejects(signingKey("not a key"));
  const crlf = testPem().replaceAll("\n", "\r\n");
  assert.ok(await signingKey(crlf), "a file with Windows line ends reads too");
});
