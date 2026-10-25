import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import { GUIDES, guide, guideAnchors, guideLink, readGuide, slug, type Guide } from "../components/reference/guide.ts";
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

/** What a guide's file says, as the page's pieces say it: title, opening, headings, text, tables and code. */
function guideWords(page: Guide): string {
  const pieces = [`<h1>${page.title}</h1>`, `<p>${page.intro}</p>`];
  for (const section of page.sections) {
    pieces.push(`<h2>${section.heading.html}</h2>`);
    for (const block of section.blocks) {
      if (block.kind === "html") {
        pieces.push(block.html);
      } else if (block.kind === "code") {
        pieces.push(`<pre>${block.code.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")}</pre>`);
      } else {
        pieces.push(`<p>${block.columns.join(" ")}</p>`, ...block.rows.map((row) => `<p>${row.join(" ")}</p>`));
      }
    }
  }
  return wordsOf(pieces.join("\n"));
}

/** The headings of a Markdown file from its second level down, found line by line outside code. */
function fileHeadings(markdown: string): string[] {
  let code = false;
  const found: string[] = [];
  for (const line of markdown.split("\n")) {
    if (/^\s*(```|~~~)/.test(line)) {
      code = !code;
    } else if (!code && /^#{2,6} /.test(line)) {
      found.push(line.replace(/^#+ /, "").replace(/`/g, "").trim());
    }
  }
  return found;
}

const GUIDE_SECTIONS = {
  install: [
    "Download",
    "Package managers",
    "Open a build that is not signed yet",
    "Settings",
    "Updates",
    "The quotadial command",
    "Build it yourself",
  ],
  cli: [
    "Commands",
    "Completions and manual pages",
    "How often providers are asked",
    "Accounts",
    "Settings",
    "The daemon",
    "Alerts",
    "Status pages",
    "The week's cost",
    "History",
    "QuotaDial Free and Pro",
    "Licence",
    "Doctor",
    "The local API",
    "The MCP server",
    "Text output",
    "JSON output",
    "Exit codes",
  ],
} as const;

test("each guide page shows every heading of its file, in order, and the sections are the ones the page was built for", () => {
  for (const name of ["install", "cli"] as const) {
    const markdown = repositoryFile(GUIDES[name].file);
    const page = guide(name);
    assert.deepEqual(
      page.headings.map((heading) => heading.text),
      fileHeadings(markdown),
      `${name}: the page's headings are the file's`,
    );
    assert.deepEqual(
      page.sections.map((section) => section.heading.text),
      GUIDE_SECTIONS[name],
      `${name}: a section renamed, added or dropped in ${GUIDES[name].file} needs this page's expected list (and its links) looked at`,
    );
    const ids = page.headings.map((heading) => heading.id);
    assert.equal(new Set(ids).size, ids.length, `${name}: every anchor is unique`);
  }
  assert.equal(guide("install").title, "Installing QuotaDial");
  assert.equal(guide("cli").title, "The QuotaDial CLI");
});

test("a section renamed in a guide's file no longer matches the page's expected sections", () => {
  const renamed = repositoryFile("docs/INSTALL.md").replace("## Updates\n", "## Updating\n");
  assert.notDeepEqual(
    readGuide(renamed).sections.map((section) => section.heading.text),
    GUIDE_SECTIONS.install,
  );
});

test("no word of either guide is lost, added or changed by the page", () => {
  for (const name of ["install", "cli"] as const) {
    const markdown = repositoryFile(GUIDES[name].file);
    assert.equal(guideWords(guide(name)), wordsOf(html(markdown)), `${name}: the page says what the file says`);
  }
});

test("every link in a guide to a section of either guide reaches a heading that is there", () => {
  const pages = { "/install": guide("install"), "/cli": guide("cli") };
  let checked = 0;
  for (const name of ["install", "cli"] as const) {
    for (const { page, id } of guideAnchors(repositoryFile(GUIDES[name].file), GUIDES[name].path)) {
      const target = pages[page as keyof typeof pages];
      assert.ok(target, `${name} links to a guide page ${page}`);
      assert.ok(
        target.headings.some((heading) => heading.id === id),
        `${name} links to ${page}#${id}, which has no heading`,
      );
      checked += 1;
    }
  }
  assert.ok(checked >= 7, "the guides link to each other's sections");
});

test("anchors are GitHub's, and a link to another file leads to its page here or on GitHub", () => {
  assert.equal(slug("The quotadial command"), "the-quotadial-command");
  assert.equal(slug("The week's cost"), "the-weeks-cost");
  assert.equal(slug("QuotaDial Free and Pro"), "quotadial-free-and-pro");
  assert.equal(guideLink("CLI.md#alerts"), "/cli#alerts");
  assert.equal(guideLink("INSTALL.md"), "/install");
  assert.equal(guideLink("PRIVACY.md"), "/privacy");
  assert.equal(guideLink("#updates"), "#updates");
  assert.equal(guideLink("https://rustup.rs"), "https://rustup.rs");
  assert.equal(guideLink("openapi.json"), "https://github.com/pghmatt/quotadial/blob/main/docs/openapi.json");
  assert.equal(guideLink("../skills/quotadial/SKILL.md"), "https://github.com/pghmatt/quotadial/blob/main/skills/quotadial/SKILL.md");
  assert.equal(guideLink("RELEASE.md"), "https://github.com/pghmatt/quotadial/blob/main/docs/RELEASE.md");
});

test("commands carry a copy button, example output scrolls, and a block inside a list stays with its item", () => {
  const sample = [
    "# Guide",
    "",
    "Opening words.",
    "",
    "## Steps",
    "",
    "- First item.",
    "- **Second:** run it:",
    "",
    "  ```sh",
    "  make all",
    "  ```",
    "",
    "- Third item.",
    "",
    "## Output",
    "",
    "```text",
    "a   b",
    "```",
    "",
    "| Name | Meaning |",
    "| --- | --- |",
    "| `x` | one |",
  ].join("\n");
  const page = readGuide(sample);
  assert.equal(page.intro, "Opening words.");
  const [steps, output] = page.sections;
  assert.deepEqual(
    steps.blocks.map((block) => block.kind),
    ["html", "code", "html"],
  );
  const code = steps.blocks[1];
  assert.ok(code.kind === "code" && code.command && code.nested && code.code === "make all");
  const first = steps.blocks[0];
  assert.ok(first.kind === "html" && first.html.includes("First item") && first.html.includes("Second"));
  assert.ok(!(first.kind === "html" && first.html.includes("<p>")), "a list with a block in it is set as tight as the others");
  const result = output.blocks[0];
  assert.ok(result.kind === "code" && !result.command && result.name === "Output, example output");
  const table = output.blocks[1];
  assert.ok(table.kind === "table" && table.columns.join() === "Name,Meaning" && table.rows[0][0] === "<code>x</code>");
  assert.throws(() => readGuide("# Guide\n\n## One\n\n## Two\n").sections[5].heading);
  assert.throws(() => readGuide("# Guide\n\nOpening.\n\nA second paragraph before any section.\n\n## One\n"), /before its first section/);
});

test("the support page sends the doctor to the command guide's own Doctor section", () => {
  const page = readFileSync(new URL("../app/support/page.tsx", import.meta.url), "utf8");
  assert.match(page, /href="\/cli#doctor"/);
  assert.ok(guide("cli").headings.some((heading) => heading.id === "doctor"), "CLI.md still has a Doctor heading");
});

test("the sitemap lists both guide pages and the footer leads to them rather than to GitHub", () => {
  const sitemap = readFileSync(new URL("../app/sitemap.ts", import.meta.url), "utf8");
  const footer = readFileSync(new URL("../components/Footer.tsx", import.meta.url), "utf8");
  for (const path of ["/install", "/cli"]) {
    assert.ok(sitemap.includes(`"${path}"`), `${path} is in the sitemap`);
    assert.ok(footer.includes(`href: "${path}"`), `the footer links to ${path}`);
  }
  assert.ok(!footer.includes("SITE.installGuide") && !footer.includes("SITE.cliGuide"));
});
