import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import { breakableHtml, markBreaks } from "../lib/break-html.ts";
import { NOT_BUILT, built, builtFiles } from "./built.ts";

/** A piece the browser may not break, as Breakable sets it. */
const whole = (...pieces: string[]) => pieces.map((piece) => `<span class="whitespace-nowrap">${piece}</span>`).join("<wbr>");

test("a word breaks only between the pieces it is given, and spaces are left as they are", () => {
  assert.equal(breakableHtml("release-assets.githubusercontent.com"), whole("release-", "assets.githubusercontent.com"));
  assert.equal(breakableHtml("status --json"), `${whole("status")} ${whole("--json")}`);
  assert.equal(breakableHtml("a\n  b"), `${whole("a")}\n  ${whole("b")}`, "line ends and runs of spaces in a block of code survive");
});

test("markBreaks marks the words of code and links only, and marking twice changes nothing", () => {
  const html = '<p>built-in text and <code>releases/latest</code> or <a href="/x">sign-in page</a>.</p><pre><code>a b\nc-d</code></pre>';
  const once = markBreaks(html);
  assert.equal(
    once,
    `<p>built-in text and <code>${whole("releases/", "latest")}</code> or <a href="/x">${whole("sign-", "in")} ${whole("page")}</a>.</p><pre><code>${whole("a")} ${whole("b")}\n${whole("c-", "d")}</code></pre>`,
  );
  assert.equal(markBreaks(once), once);
});

test("no component lets a word break anywhere", () => {
  for (const file of ["Prose.tsx", "home/Addresses.tsx", "CodeBlock.tsx", "reference/Guide.tsx", "reference/GuidePage.tsx"]) {
    const source = readFileSync(new URL(`../components/${file}`, import.meta.url), "utf8");
    assert.ok(!/overflow-wrap:\s*anywhere|\[overflow-wrap:anywhere\]/.test(source.replace(/`anywhere`/g, "")), `${file} never breaks a word anywhere`);
  }
});

test("the built pages carry code as pieces, and no stylesheet rule breaks a word anywhere", { skip: built ? false : NOT_BUILT }, () => {
  for (const sheet of builtFiles(".css")) {
    assert.ok(!/overflow-wrap:\s*anywhere/.test(sheet.text), `${sheet.path} has no overflow-wrap:anywhere`);
  }
  const privacy = builtFiles("privacy.html")[0].text;
  assert.ok(privacy.includes('<span class="whitespace-nowrap">'), "/privacy's code is set in pieces");
  const home = builtFiles("index.html")[0].text.replace(/<wbr\/>/g, "<wbr>");
  assert.ok(home.includes(whole("release-", "assets.githubusercontent.com")), "the home page's address is set in pieces and breaks only after its hyphen");
});

test("the home page's addresses are set with Breakable, and from 1280 px the longest is held to a line without its break mark", () => {
  const source = readFileSync(new URL("../components/home/Addresses.tsx", import.meta.url), "utf8");
  assert.match(source, /<Breakable text=\{address\} \/>/);
  assert.match(source, /xl:whitespace-nowrap xl:\[&_wbr\]:hidden/);
});
