// Serves the built site (out/) the way Cloudflare Pages does, for checks and captures on this
// machine: `/pro` is `pro.html`, an address the site does not have is `404.html` with status 404.
//
// `POST /api/key` runs the real key function's logic (lib/key.ts) against a stand-in for Lemon
// Squeezy, so every state of /key can be seen and captured without a store: the receipt key picks
// Lemon Squeezy's answer (below). It signs with the test key the tests use, never a real one. This
// file is never deployed; Pages runs functions/api/key.ts.
//
//   npm run build && npm run serve                 # http://127.0.0.1:4321
//   PORT=4400 npm run serve
//
// Receipt keys for /key#licence=<key>:
//   11111111-1111-4111-8111-111111111111   a valid key: the QuotaDial key is shown
//   22222222-2222-4222-8222-222222222222   refunded: turned off
//   33333333-3333-4333-8333-333333333333   ended
//   44444444-4444-4444-8444-444444444444   another product
//   55555555-5555-4555-8555-555555555555   Lemon Squeezy cannot be reached
//   any other key in the right shape       Lemon Squeezy does not know it
import { existsSync, readFileSync, statSync } from "node:fs";
import { createServer } from "node:http";
import { extname, join, normalize } from "node:path";
import { fileURLToPath } from "node:url";

import { issue } from "../lib/key.ts";

const OUT = fileURLToPath(new URL("../out/", import.meta.url));
const PORT = Number(process.env.PORT ?? 4321);

const TYPES = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript",
  ".css": "text/css",
  ".json": "application/json",
  ".txt": "text/plain; charset=utf-8",
  ".xml": "application/xml",
  ".svg": "image/svg+xml",
  ".png": "image/png",
  ".webp": "image/webp",
  ".ico": "image/x-icon",
  ".woff2": "font/woff2",
};

/** The test key from crates/core/src/licence.rs, as the PEM file `quotadial-issue keygen` writes. */
function testPem() {
  const der = Uint8Array.of(
    0x30, 0x2e, 0x02, 0x01, 0x00, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x04, 0x22, 0x04, 0x20,
    ...new TextEncoder().encode("QuotaDial's licence test key 001"),
  );
  return `-----BEGIN PRIVATE KEY-----\n${btoa(String.fromCharCode(...der))}\n-----END PRIVATE KEY-----\n`;
}

const ENV = { LICENCE_ISSUING_KEY: testPem(), LS_STORE_ID: "1", LS_PRODUCT_ID: "4" };

/** Lemon Squeezy's answer for a receipt key, as its validate endpoint gives it. */
function lemonSqueezy(key) {
  const licence = (status, extra = {}) => ({
    valid: true,
    error: null,
    license_key: {
      id: 1,
      status,
      key,
      activation_limit: 1,
      activation_usage: 0,
      created_at: "2026-10-25T09:00:00.000000Z",
      expires_at: null,
      ...extra,
    },
    meta: { store_id: 1, product_id: 4, product_name: "QuotaDial Pro" },
  });
  switch (key.slice(0, 8)) {
    case "11111111":
      return licence("inactive");
    case "22222222":
      return licence("disabled");
    case "33333333":
      return licence("expired", { expires_at: "2026-09-30T00:00:00.000000Z" });
    case "44444444":
      return { ...licence("active"), meta: { store_id: 1, product_id: 9, product_name: "Another product" } };
    case "55555555":
      return null;
    default:
      return { valid: false, error: "license_key not found.", license_key: null, meta: null };
  }
}

async function keyAnswer(request) {
  let body = "";
  for await (const chunk of request) {
    body += chunk;
  }
  let orderKey = "";
  try {
    const sent = JSON.parse(body);
    if (typeof sent?.licence === "string") {
      orderKey = sent.licence;
    }
  } catch {
    // Answered as a key that is not one, as the function does.
  }
  const answer = await issue(orderKey, ENV, async (_url, init) => {
    const key = new URLSearchParams(String(init.body)).get("license_key") ?? "";
    const reply = lemonSqueezy(key);
    if (!reply) {
      throw new TypeError("fetch failed");
    }
    return new Response(JSON.stringify(reply), { status: reply.valid ? 200 : 404 });
  });
  return answer;
}

createServer(async (request, response) => {
  const url = new URL(request.url ?? "/", "http://127.0.0.1");
  if (url.pathname === "/api/key" && request.method === "POST") {
    const answer = await keyAnswer(request);
    response
      .writeHead(answer.status, { "content-type": "application/json", "cache-control": "no-store" })
      .end(JSON.stringify(answer.body));
    return;
  }
  const path = decodeURIComponent(url.pathname);
  const candidates = path.endsWith("/")
    ? [join(path, "index.html")]
    : [path, `${path}.html`, join(path, "index.html")];
  for (const candidate of candidates) {
    const file = normalize(join(OUT, candidate));
    if (file.startsWith(OUT) && existsSync(file) && statSync(file).isFile()) {
      response
        .writeHead(200, { "content-type": TYPES[extname(file)] ?? "application/octet-stream" })
        .end(readFileSync(file));
      return;
    }
  }
  response.writeHead(404, { "content-type": TYPES[".html"] }).end(readFileSync(join(OUT, "404.html")));
}).listen(PORT, "127.0.0.1", () => {
  console.log(`The built site is at http://127.0.0.1:${PORT}/`);
});
