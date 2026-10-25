import assert from "node:assert/strict";
import { test } from "node:test";

import { DOWNLOADS } from "../lib/data/release.ts";
import { changelog, html, installGuide, repositoryFile, section, tables, withoutTitle } from "../lib/markdown.ts";

test("the privacy page carries docs/PRIVACY.md and every address in it", () => {
  const { title, body } = withoutTitle(repositoryFile("docs/PRIVACY.md"));
  assert.equal(title, "What QuotaDial reads and sends");
  const rendered = html(body);
  for (const address of ["api.anthropic.com", "chatgpt.com", "status.claude.com", "status.openai.com", "github.com"]) {
    assert.ok(rendered.includes(address), `the page names ${address}`);
  }
  assert.ok(!rendered.includes("<h1"), "the file's own heading is the page's H1, set once");
});

test("the changelog's versions come newest first, each with its sections", () => {
  const { versions } = changelog(repositoryFile("CHANGELOG.md"));
  assert.equal(versions[0].version, "Unreleased");
  assert.equal(versions[0].date, null);
  assert.ok(versions.some((version) => version.version === "0.2.0" && version.date === "2026-10-24"));
  assert.ok(versions[0].sections.some((section) => section.heading === "Added" && section.html.includes("<li>")));
});

test("a section runs to the next heading of its level, and a fenced # is not a heading", () => {
  const text = "# Title\n\n## One\n\nfirst\n\n```sh\n# a comment\n```\n\n### Inner\n\ninner\n\n## Two\n\nsecond\n";
  assert.equal(section(text, "One"), "first\n\n```sh\n# a comment\n```\n\n### Inner\n\ninner");
  assert.equal(section(text, "Two"), "second");
  assert.throws(() => section(text, "Three"), /no section "Three"/);
});

test("tables come out as rows of cells, without code marks", () => {
  const text = "| A | B |\n| --- | --- |\n| x | `y z` |\n\ntext\n\n| C |\n| --- |\n| w |\n";
  assert.deepEqual(tables(text), [[["x", "y z"]], [["w"]]]);
});

test("/download shows INSTALL.md's own files, check commands and unsigned-build steps", () => {
  const guide = installGuide();
  assert.deepEqual(
    guide.files.map((row) => row.file),
    [
      "QuotaDial_<version>_universal.dmg",
      "QuotaDial_<version>_x64_en-US.msi",
      "QuotaDial_<version>_amd64.deb",
      "QuotaDial_<version>_amd64.AppImage",
    ],
  );
  assert.deepEqual(
    guide.checks.map((row) => row.system),
    ["macOS", "Linux", "Windows"],
  );
  assert.ok(guide.checks.every((row) => repositoryFile("docs/INSTALL.md").includes(`\`${row.command}\``)));
  for (const system of ["macOS", "Windows", "Linux"]) {
    assert.ok(guide.unsigned.includes(`<h3>${system}</h3>`), `the steps for ${system}`);
  }
  assert.ok(guide.unsigned.includes("<ol>"), "numbered steps");
});

test("the site's files for each system are INSTALL.md's", () => {
  const listed = installGuide().files.map((row) => row.file);
  assert.deepEqual(
    DOWNLOADS.flatMap((download) => download.files.map((file) => file.name)).sort(),
    [...listed].sort(),
  );
});
