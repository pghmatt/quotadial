import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import { countUnder, doctorSample, headingsUsed, sayDate, setAddresses, versionId, wordsOf } from "../components/reference/text.ts";
import { changelog, html, repositoryFile, withoutTitle } from "../lib/markdown.ts";

const privacy = () => html(withoutTitle(repositoryFile("docs/PRIVACY.md")).body);

test("every address in the privacy text is set as an address, and no word of the text changes", () => {
  const original = privacy();
  const set = setAddresses(original);
  assert.equal(wordsOf(set), wordsOf(original), "setting addresses changes tags only");
  for (const address of [
    "api.anthropic.com",
    "chatgpt.com",
    "status.claude.com",
    "status.openai.com",
    "github.com",
    "release-assets.githubusercontent.com",
    "127.0.0.1",
    "quotadial.com/key",
  ]) {
    assert.ok(set.includes(`<code class="address">${address}</code>`), `${address} is set as an address`);
  }
  assert.ok(!/<strong>[a-z0-9.-]+\.[a-z]{2,}<\/strong>/.test(set), "no host name is left only in bold");
});

test("setAddresses leaves code that is already code, and other bold text, alone", () => {
  const input = "<p><strong>Note</strong> <code>127.0.0.1</code> and 127.0.0.1:8080 and <strong>github.com</strong></p>";
  const set = setAddresses(input);
  assert.ok(set.includes("<strong>Note</strong>"));
  assert.ok(set.includes("<code>127.0.0.1</code>"), "already code: not wrapped again");
  assert.ok(set.includes("<code class=\"address\">127.0.0.1</code>:8080"), "a plain address is set, its port stays text");
  assert.ok(set.endsWith('<code class="address">github.com</code></p>'));
});

test("the changelog page's counts, dates and anchors come from CHANGELOG.md", () => {
  const { versions } = changelog(repositoryFile("CHANGELOG.md"));
  const names = versions.map((version) => version.version);
  assert.deepEqual(names.slice(0, 3), ["Unreleased", "0.2.0", "0.1.0"]);
  const [unreleased, second] = versions;
  assert.ok(countUnder(unreleased, "Added") > 10, "Unreleased lists many additions");
  assert.equal(countUnder(second, "Added"), 6);
  assert.equal(countUnder(second, "Fixed"), 0, "a version without the heading lists none");
  assert.deepEqual(headingsUsed(versions).slice(0, 2), ["Added", "Fixed"]);
  assert.equal(sayDate("2026-10-24"), "24 October 2026");
  assert.equal(sayDate("2026-01-05"), "5 January 2026");
  assert.equal(sayDate("2026-10-24", true), "24 Oct 2026");
  assert.throws(() => sayDate("24/10/2026"));
  assert.throws(() => sayDate("2026-13-01"));
  assert.equal(versionId("Unreleased"), "v-unreleased");
  assert.equal(versionId("0.2.0"), "v-0-2-0");
});

test("the doctor example is the command guide's, and a guide without one is an error", () => {
  const sample = doctorSample();
  assert.match(sample, /^QuotaDial \S+ on /);
  assert.match(sample, /Accounts \(\d+\):/);
  assert.match(sample, /\w\*\*\*@\w\*\*\*\.\w+/, "addresses in it are cut short");
  assert.throws(() => doctorSample("# Guide\n\n## Doctor\n\nNo example here.\n"), /no example/);
});

test("the changelog's versions end at the reading measure, and the Unreleased plate closes on its text", () => {
  const source = readFileSync(new URL("../components/reference/Versions.tsx", import.meta.url), "utf8");
  assert.match(source, /lg:grid-cols-\[14rem_minmax\(0,var\(--container-measure\)\)\]/, "the changes column is one measure wide, from the token");
  assert.doesNotMatch(source, /minmax\(0,1fr\)/, "no column stretches past the measure into an empty strip");
  assert.doesNotMatch(source, /\b52ch\b/, "the measure is the token, never a literal width");
  assert.match(source, /<Plate [^>]*tone="shallow"[^>]*className="w-fit"/, "the Unreleased plate is as wide as its content");
  assert.match(source, /<div className="max-w-measure [^"]*">\s*<h2 /, "under 1024 px the heading block is held to the measure too");
});
