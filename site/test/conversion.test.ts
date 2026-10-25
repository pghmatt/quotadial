import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";

import { longDay, receiptKey, requestKey, UNREACHED, UNREADABLE } from "../components/conversion/key-state.ts";
import { PRO_ROWS } from "../components/conversion/pro-rows.ts";
import { PRO_ADDS } from "../lib/data/pro.ts";

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

/** The words the app lists, down to where its own "not" clause begins. */
const lead = (what: string) => what.split(", not ")[0].toLowerCase();

test("the Pro table has the app's seven features, in its order, in its words", () => {
  assert.equal(PRO_ROWS.length, PRO_ADDS.length);
  PRO_ROWS.forEach((row, index) => {
    assert.equal(row.feature.toLowerCase(), lead(PRO_ADDS[index]));
  });
});

test("no row is empty, and none carries a price or a dash", () => {
  for (const row of PRO_ROWS) {
    for (const text of [row.feature, row.free, row.changes]) {
      assert.ok(text.trim().length > 0);
      assert.doesNotMatch(text, /[$£€]|per month|lifetime/i);
      assert.doesNotMatch(text, /[–—]/);
    }
  }
});

/** The conversion lane's own files: three pages and their components. */
function conversionFiles(): { name: string; text: string }[] {
  const root = new URL("../", import.meta.url).pathname;
  const files: string[] = [];
  const walk = (dir: string) => {
    for (const entry of readdirSync(dir, { withFileTypes: true })) {
      const path = join(dir, entry.name);
      if (entry.isDirectory()) {
        walk(path);
      } else if (/\.(tsx?|css)$/.test(entry.name)) {
        files.push(path);
      }
    }
  };
  for (const dir of ["app/download", "app/pro", "app/key", "components/conversion"]) {
    walk(join(root, dir));
  }
  return files.map((path) => ({ name: path.slice(root.length), text: readFileSync(path, "utf8") }));
}

test("the three pages carry no price, no dash and no empty call to action", () => {
  const files = conversionFiles().filter((file) => !file.name.endsWith(".test.ts"));
  assert.ok(files.length >= 8, "the lane's files were found");
  for (const { name, text } of files) {
    assert.doesNotMatch(text, /[\u2013\u2014]/, `${name} has a dash`);
    assert.doesNotMatch(text, /[$£€]\s?\d|\d\s?(?:dollars|pounds|euros)|per month|lifetime|for ever|forever|Pro price/i, `${name} has a price`);
    assert.doesNotMatch(text, /get started|learn more/i, `${name} has a call to action that names nothing`);
    assert.doesNotMatch(text, /\b(?:quota|auth|credentials?|endpoints?|polls?)\b(?!dial)/i, `${name} uses a word the product avoids`);
  }
});

test("the key page keeps the receipt's key out of the address and the log", () => {
  const flow = readFileSync(new URL("../components/conversion/KeyFlow.tsx", import.meta.url), "utf8");
  assert.match(flow, /history\.replaceState\(null, "", window\.location\.pathname \+ window\.location\.search\)/);
  assert.doesNotMatch(flow, /console\./);
  assert.doesNotMatch(flow, /localStorage|sessionStorage|document\.cookie/);
});

test("the key page is not indexed and /pro offers a checkout only through the checked link", () => {
  const key = readFileSync(new URL("../app/key/page.tsx", import.meta.url), "utf8");
  assert.match(key, /noindex: true/);
  const pro = readFileSync(new URL("../app/pro/page.tsx", import.meta.url), "utf8");
  assert.match(pro, /SITE\.checkout/);
  assert.doesNotMatch(pro, /process\.env/);
});

test("/download renders docs/INSTALL.md's steps and check commands, and copies none of them", () => {
  const page = readFileSync(new URL("../app/download/page.tsx", import.meta.url), "utf8");
  assert.match(page, /installGuide\(\)/);
  assert.match(page, /GUIDE\.unsigned/);
  assert.match(page, /GUIDE\.checks/);
  for (const copied of ["Move to Trash", "Keep anyway", "Run anyway", "--ignore-missing", "Get-FileHash", "chmod +x", "apt install ./"]) {
    assert.ok(!page.includes(copied), `the page has "${copied}" written out: it belongs to INSTALL.md`);
  }
  assert.match(page, /DownloadButton|SystemDownload/);
  const download = readFileSync(new URL("../components/conversion/SystemDownload.tsx", import.meta.url), "utf8");
  assert.match(download, /<DownloadButton href=\{DOWNLOAD_PAGE\} \/>/);
});

test("each system's part of /download has its own secondary download action, and only the hero's is primary", () => {
  const page = readFileSync(new URL("../app/download/page.tsx", import.meta.url), "utf8");
  for (const system of ["macos", "windows", "linux"]) {
    assert.ok(page.includes(`<SystemAction system="${system}" />`), `${system} has its own action`);
  }
  const action = readFileSync(new URL("../components/conversion/SystemAction.tsx", import.meta.url), "utf8");
  assert.match(action, /data-event="download_click"/);
  assert.match(action, /data-event-system=\{system\}/);
  assert.match(action, /href=\{DOWNLOAD_PAGE\}/);
  assert.doesNotMatch(action, /data-primary-action/);
});
