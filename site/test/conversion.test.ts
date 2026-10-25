import assert from "node:assert/strict";
import { existsSync, readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";

import {
  appFiles,
  buttonLabel,
  clickDetail,
  commandFiles,
  commandSystemOf,
  filesFor,
  systemOf,
} from "../components/conversion/download-links.ts";
import { longDay, receiptKey, requestKey, UNREACHED, UNREADABLE } from "../components/conversion/key-state.ts";
import { PRO_ROWS } from "../components/conversion/pro-rows.ts";
import { PRO_ADDS } from "../lib/data/pro.ts";
import { fenced, installGuide, repositoryFile } from "../lib/markdown.ts";
import { breakPieces } from "../lib/breaks.ts";
import { NOT_BUILT } from "./built.ts";

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

test("the Pro table has the app's features, in its order, in its words", () => {
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

const LATEST = "https://github.com/pghmatt/quotadial/releases/latest/download/";
const RELEASES = "https://github.com/pghmatt/quotadial/releases";

/** The version-less names docs/INSTALL.md links, read from the file and not from the site's code. */
function listedNames(): string[] {
  return [...repositoryFile("docs/INSTALL.md").matchAll(/\]\(https:\/\/github\.com\/pghmatt\/quotadial\/releases\/latest\/download\/([^)\s]+)\)/g)].map(
    (found) => found[1],
  );
}

/** A file of the lane, as text. */
function source(path: string): string {
  return readFileSync(new URL(`../${path}`, import.meta.url), "utf8");
}

test("docs/INSTALL.md links eight version-less copies, and installGuide gives each its link", () => {
  const listed = listedNames();
  assert.equal(listed.length, 8, "four app files and four command archives");
  assert.equal(new Set(listed).size, 8);
  const guide = installGuide();
  const given = [...guide.files.map((row) => row.latest), ...guide.commands.map((row) => row.latest)];
  assert.deepEqual(given.map((link) => link.name).sort(), [...listed].sort());
  for (const link of given) {
    assert.equal(link.href, `${LATEST}${link.name}`);
  }
});

test("a version-less link that is not under releases/latest/download fails the build", () => {
  const file = repositoryFile("docs/INSTALL.md");
  const moved = file.replace("releases/latest/download/QuotaDial_universal.dmg", "releases/download/v1/QuotaDial_universal.dmg");
  assert.notEqual(moved, file);
  assert.throws(() => installGuide(moved), /not a link to a file under/);
  const renamed = file.replace("](https://github.com/pghmatt/quotadial/releases/latest/download/QuotaDial_universal.dmg)", "](https://github.com/pghmatt/quotadial/releases/latest/download/QuotaDial.dmg)");
  assert.throws(() => installGuide(renamed), /not a link to a file under/);
  const bare = file.replace(/\[QuotaDial_amd64\.deb\]\([^)]+\)/, "QuotaDial_amd64.deb");
  assert.throws(() => installGuide(bare), /not a link to a file under/);
  assert.throws(() => installGuide(file.replace("### The command alone", "### Elsewhere")), /no section "The command alone"/);
});

test("the install line is the first fenced block of INSTALL.md's command section, and names an archive the table lists", () => {
  const guide = installGuide();
  assert.equal(guide.install.split("\n").length, 2, "a fetch and an install");
  assert.ok(guide.install.includes(`${LATEST}quotadial-aarch64-apple-darwin.tar.gz`), "the page says the first line names Apple silicon");
  assert.ok(listedNames().some((name) => guide.install.includes(`/${name} `)), "an archive the table links");
  assert.equal(fenced("text\n\n```sh\none\ntwo\n```\n\n```\nthree\n```\n"), "one\ntwo");
  assert.throws(() => fenced("no code here"), /no fenced code block/);
});

test("each app file belongs to one system, and Linux has the .deb first and the .AppImage second", () => {
  const files = appFiles(installGuide().files);
  assert.deepEqual(
    files.map((file) => systemOf(file.name)),
    ["macos", "windows", "linux", "linux"],
  );
  assert.deepEqual(
    filesFor(files, "linux").map((file) => file.name),
    ["QuotaDial_amd64.deb", "QuotaDial_amd64.AppImage"],
  );
  assert.deepEqual(
    commandFiles(installGuide().commands).map((file) => commandSystemOf(file.name)),
    ["macos", "macos", "linux", "windows"],
  );
  assert.throws(() => systemOf("QuotaDial.rpm"), /cannot place/);
  assert.throws(() => commandSystemOf("quotadial-riscv.tar.gz"), /cannot place/);
});

test("a button names its file's kind where a system has two, and its system where it has one", () => {
  const files = appFiles(installGuide().files);
  assert.deepEqual(
    ["macos", "windows", "linux"].flatMap((system) => {
      const own = filesFor(files, system as "macos" | "windows" | "linux");
      return own.map((file) => buttonLabel(file, own));
    }),
    ["Download for macOS", "Download for Windows", "Download the .deb", "Download the .AppImage"],
  );
});

test("a download click reports its system, its file and where it was", () => {
  const [mac] = appFiles(installGuide().files);
  assert.deepEqual(clickDetail(mac, "hero"), {
    "data-event": "download_click",
    "data-event-system": "macos",
    "data-event-file": "QuotaDial_universal.dmg",
    "data-event-from": "hero",
  });
  assert.equal(clickDetail({ name: "quotadial-x86_64-pc-windows-msvc.zip" }, "command", "windows")["data-event-system"], "windows");
});

test("every download action goes to the file's own link, and none to the releases page", () => {
  for (const path of ["components/conversion/SystemDownload.tsx", "components/conversion/SystemAction.tsx"]) {
    const text = source(path);
    assert.match(text, /href=\{file\.href\}/, `${path} links the file`);
    assert.doesNotMatch(text, /DOWNLOAD_PAGE|releases\/|https?:\/\//, `${path} has no address of its own`);
  }
  assert.match(source("components/conversion/FileName.tsx"), /href=\{file\.href\}/, "both tables link the file through FileName");
  const release = source("lib/data/release.ts");
  assert.doesNotMatch(release, /DOWNLOAD_PAGE|COMMAND_ONLY/, "the lists that named wrong files and a page are gone");
});

test("the page takes its files, archives and install line from INSTALL.md, and links the install guide", () => {
  const page = source("app/download/page.tsx");
  assert.match(page, /installGuide\(\)/);
  assert.match(page, /appFiles\(GUIDE\.files\)/);
  assert.match(page, /commandFiles\(GUIDE\.commands\)/);
  assert.match(page, /code=\{GUIDE\.install\}/);
  assert.match(page, /GUIDE\.unsigned/);
  assert.match(page, /GUIDE\.checks/);
  assert.match(page, /href="\/install"/);
  assert.doesNotMatch(page, /SITE\.installGuide|COMMAND_ONLY/);
  for (const copied of ["Move to Trash", "Keep anyway", "Run anyway", "--ignore-missing", "Get-FileHash", "chmod +x", "apt install ./", "releases/latest"]) {
    assert.ok(!page.includes(copied), `the page has "${copied}" written out: it belongs to INSTALL.md`);
  }
  for (const system of ["macos", "windows", "linux"]) {
    assert.ok(page.includes(`<SystemAction system="${system}" files={FILES} />`), `${system} has its own action`);
  }
  assert.doesNotMatch(source("components/conversion/SystemAction.tsx"), /data-primary-action/);
  assert.match(source("components/conversion/SystemDownload.tsx"), /data-primary-action=\{index === 0 \? "" : undefined\}/);
});

test("nothing that makes the page fetches an address when the site is built", () => {
  for (const path of [
    "lib/markdown.ts",
    "lib/data/release.ts",
    "app/download/page.tsx",
    "components/conversion/download-links.ts",
    "components/conversion/CommandTable.tsx",
    "components/conversion/SystemAction.tsx",
    "components/conversion/FilesTable.tsx",
    "components/conversion/SystemDownload.tsx",
  ]) {
    assert.doesNotMatch(source(path), /\bfetch\(|node:https?|XMLHttpRequest|\bhttps?\.(?:get|request)|new URL\(|axios|undici/, `${path} reaches for the network`);
  }
});

const NAME_TABLES = ["FilesTable", "CommandTable", "FileName", "InlineCode", "SystemAction"].map((name) => `components/conversion/${name}.tsx`);

test("file names, archive names and addresses break only where Breakable marks, and never anywhere", () => {
  for (const path of NAME_TABLES) {
    const text = source(path);
    assert.ok(!/overflow-wrap:\s*anywhere|anywhere\]|break-all|whitespace-nowrap|soft\(/.test(text), `${path} does not break names anywhere or keep them from breaking`);
  }
  for (const path of ["FileName", "InlineCode"].map((name) => `components/conversion/${name}.tsx`)) {
    const text = source(path);
    assert.match(text, /<Breakable text=/, `${path} sets its names with Breakable`);
    assert.match(text, /overflow-wrap:break-word/, `${path} breaks inside a piece only when one piece is wider than the line`);
  }
});

test("both tables set a file the same way: the name its button saves, then the name a release holds it by", () => {
  for (const table of ["FilesTable", "CommandTable"]) {
    assert.match(source(`components/conversion/${table}.tsx`), /<FileName file=\{/, `${table} sets each file with FileName`);
  }
  assert.match(source("components/conversion/FilesTable.tsx"), /record=\{releases\}/, "the plate's versioned names still lead to the Releases page");
  assert.ok(!/record=/.test(source("components/conversion/CommandTable.tsx")), "an archive's versioned name is the record, not a link");
});

/** The built /download, once `npm run build` has made it. */
const BUILT = new URL("../out/download.html", import.meta.url);
const built = existsSync(BUILT) ? readFileSync(BUILT, "utf8") : "";
const skip = built ? false : NOT_BUILT;

/** Text of a built page: its tags gone, its entities read, its soft breaks closed up. */
function words(text: string): string {
  return text
    .replace(/<wbr\/?>/g, "")
    .replace(/<[^>]+>/g, " ")
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">")
    .replace(/&amp;/g, "&")
    .replace(/\s+/g, " ");
}

/** Each anchor in a built page as its attributes and the words it shows. */
function anchors(text: string): { attributes: string; href: string; words: string }[] {
  return [...text.matchAll(/<a ([^>]*)>([\s\S]*?)<\/a>/g)].map((found) => ({
    attributes: found[1],
    href: found[1].match(/href="([^"]*)"/)?.[1] ?? "",
    words: words(found[2]).trim(),
  }));
}

test("the built page shows every one of INSTALL.md's eight version-less names, each as a link to its file", { skip }, () => {
  const text = words(built);
  const links = anchors(built);
  for (const name of listedNames()) {
    assert.ok(text.includes(name), `${name} is on the page`);
    assert.ok(links.some((link) => link.href === `${LATEST}${name}`), `${name} is a link to its file`);
  }
});

test("the built page's download buttons are straight links to files, and every one reports its system and file", { skip }, () => {
  const downloads = anchors(built).filter((link) => link.attributes.includes('data-event="download_click"'));
  assert.ok(downloads.length >= 7, "the three sections' buttons (four files) and the four archives");
  for (const link of downloads) {
    assert.ok(link.href.startsWith(LATEST), `${link.href} is a file of the newest full release`);
    assert.ok(link.href.endsWith(link.attributes.match(/data-event-file="([^"]*)"/)?.[1] ?? "none"), "the click names the file it fetches");
    assert.match(link.attributes, /data-event-system="(?:macos|windows|linux)"/);
  }
  assert.ok(!anchors(built).some((link) => link.href === `${RELEASES}/latest`), "no button goes to the latest release's page");
});

test("the built page keeps the versioned file table, each name linking to the Releases page", { skip }, () => {
  const links = anchors(built).filter((link) => link.href === RELEASES);
  for (const versioned of installGuide().files.map((row) => row.file)) {
    assert.ok(
      links.some((link) => link.words === versioned),
      `${versioned} links to the Releases page`,
    );
  }
});

test("the built page's first action is for the visitor's system once known, and a way to the systems before", { skip }, () => {
  const text = words(built);
  const primary = anchors(built).filter((link) => link.attributes.includes("data-primary-action"));
  assert.equal(primary.length, 1, "one primary action");
  assert.equal(primary[0].href, "#systems");
  assert.equal(primary[0].words, "Choose your system");
  assert.match(built, /id="systems"/);
  for (const id of ["macos", "windows", "linux"]) {
    assert.match(built, new RegExp(`id="${id}"`));
  }
  assert.ok(anchors(built).some((link) => link.href === "/install" && link.words === "Read the install guide"));
  assert.ok(text.includes(installGuide().install.split("\n")[0]), "the install line is on the page");
});

test("the built page's files plate names what each button saves, with its versioned name under it", { skip }, () => {
  const links = anchors(built);
  const files = installGuide().files;
  for (const row of files) {
    const saved = links.find((link) => link.href === `${LATEST}${row.latest.name}` && link.attributes.includes('data-event-from="files"'));
    assert.ok(saved, `${row.latest.name} is a link to its file in the plate`);
    assert.equal(saved.words, row.latest.name);
    const at = built.indexOf(saved.attributes);
    const record = built.indexOf(`>${row.file.replace(/</g, "&lt;")}`.replace(/[_-]/g, (mark) => `${mark}<wbr/>`).slice(0, 12), at);
    assert.ok(record > at, `${row.file} follows ${row.latest.name}`);
  }
  for (const link of links.filter((found) => found.attributes.includes('data-event-from="files"'))) {
    assert.ok(link.href.startsWith(LATEST));
  }
});

test("the built page's install line breaks after a slash, an underscore or a hyphen, never inside a word", { skip }, () => {
  const line = installGuide().install.split("\n")[0];
  const shown = built
    .replace(/<wbr\/?>/g, "\u0001")
    .replace(/<[^>]+>/g, "")
    .replace(/&amp;/g, "&")
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">");
  assert.ok(shown.includes(breakPieces(line).join("\u0001")), "the line carries its marks at breakPieces' places and nowhere else");
  assert.ok(!shown.includes("lat\u0001est"), "no mark inside latest");
  assert.ok(shown.includes("latest/\u0001download"), "the mark follows the slash after latest");
});
