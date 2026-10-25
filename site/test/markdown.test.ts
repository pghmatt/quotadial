import assert from "node:assert/strict";
import { test } from "node:test";

import { changelog, html, repositoryFile, withoutTitle } from "../lib/markdown.ts";

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
