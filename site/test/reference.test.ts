import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import { GUIDES, finish, guide, guideAnchors, guideLink, readGuide, slug, type Guide } from "../components/reference/guide.ts";
import {
  countUnder,
  doctorSample,
  headingsUsed,
  sayDate,
  setAddresses,
  versionId,
  versionsShown,
  wordsOf,
} from "../components/reference/text.ts";
import { changelog, html, repositoryFile, withoutTitle } from "../lib/markdown.ts";
import { NOT_BUILT, built, builtFiles } from "./built.ts";
import { changelogCases, type ChangelogCase } from "./changelogs.ts";

const privacy = () => html(withoutTitle(repositoryFile("docs/PRIVACY.md")).body);

test("every address in the privacy text is set as an address, and no word of the text changes", () => {
  const original = privacy();
  const set = setAddresses(original);
  assert.equal(wordsOf(set), wordsOf(original), "setting addresses changes tags only");
  for (const address of [
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

/** The kinds of change Keep a Changelog names; the page's table counts each one used in a column. */
const KINDS = ["Added", "Changed", "Deprecated", "Removed", "Fixed", "Security"];

/**
 * What the changelog page counts, dates and links to, in a shape no release changes: 0.2.0, dated
 * 24 October 2026, lists 6 additions and no fixes; every release has a date the page can say;
 * every heading is one of Keep a Changelog's kinds; every version has an anchor of its own; and the
 * page shows every release, in order, and an Unreleased entry only with a change in it.
 */
function assertPageFacts({ name, markdown }: ChangelogCase): void {
  const { versions } = changelog(markdown);
  const second = versions.find((version) => version.version === "0.2.0");
  assert.ok(second, `${name}: lists 0.2.0`);
  assert.equal(second.date, "2026-10-24", `${name}: 0.2.0's date`);
  assert.equal(countUnder(second, "Added"), 6, `${name}: 0.2.0's additions`);
  assert.equal(countUnder(second, "Fixed"), 0, `${name}: a version without the heading lists none`);
  const releases = versions.filter((version) => version.date !== null);
  for (const release of releases) {
    assert.doesNotThrow(() => sayDate(release.date ?? ""), `${name}: ${release.version}'s date is one the page can say`);
  }
  for (const heading of headingsUsed(versions)) {
    assert.ok(KINDS.includes(heading), `${name}: "${heading}" is one of Keep a Changelog's kinds of change`);
  }
  const anchors = versions.map((version) => versionId(version.version));
  assert.equal(new Set(anchors).size, anchors.length, `${name}: every version has an anchor of its own`);
  const shown = versionsShown(versions);
  assert.deepEqual(
    shown.filter((version) => version.date !== null),
    releases,
    `${name}: the page shows every release, in order`,
  );
  for (const version of shown.filter((one) => one.date === null)) {
    assert.ok(
      version.sections.some((part) => countUnder(version, part.heading) > 0),
      `${name}: the page shows ${version.version} only with a change in it`,
    );
  }
}

test("the changelog page's counts, dates and anchors come from CHANGELOG.md, today, on the version bump and after it", () => {
  for (const shape of changelogCases()) {
    assertPageFacts(shape);
  }
  assert.deepEqual(
    headingsUsed(changelog("## [0.3.0] - 2026-10-26\n\n### Fixed\n\n- A.\n\n## [0.2.0] - 2026-10-24\n\n### Added\n\n- B.\n\n### Fixed\n\n- C.\n").versions),
    ["Fixed", "Added"],
    "the table's columns are the headings in the order they first appear",
  );
  assert.equal(sayDate("2026-10-24"), "24 October 2026");
  assert.equal(sayDate("2026-01-05"), "5 January 2026");
  assert.equal(sayDate("2026-10-24", true), "24 Oct 2026");
  assert.throws(() => sayDate("24/10/2026"));
  assert.throws(() => sayDate("2026-13-01"));
  assert.equal(versionId("Unreleased"), "v-unreleased");
  assert.equal(versionId("0.2.0"), "v-0-2-0");
});

test("the changelog page has an Unreleased plate only while Unreleased lists a change", () => {
  const shown = (markdown: string) => versionsShown(changelog(markdown).versions).map((version) => version.version);
  const release = "## [0.2.0] - 2026-10-24\n\n### Added\n\n- A change.\n";
  assert.deepEqual(shown(`# Changelog\n\n## [Unreleased]\n\n### Fixed\n\n- A fix.\n\n${release}`), ["Unreleased", "0.2.0"]);
  assert.deepEqual(shown(`# Changelog\n\n${release}`), ["0.2.0"], "no Unreleased entry, as the version bump leaves it: no plate");
  assert.deepEqual(shown(`# Changelog\n\n## [Unreleased]\n\n${release}`), ["0.2.0"], "an empty Unreleased heading: no plate");
  assert.deepEqual(shown(`# Changelog\n\n## [Unreleased]\n\n### Added\n\n${release}`), ["0.2.0"], "a kind of change with none under it: no plate");
  const page = readFileSync(new URL("../app/changelog/page.tsx", import.meta.url), "utf8");
  assert.match(
    page,
    /const versions = versionsShown\(changelog\(repositoryFile\("CHANGELOG\.md"\)\)\.versions\);/,
    "the page's table and its versions are both what versionsShown keeps",
  );
  assert.equal([...page.matchAll(/versions=\{versions\}/g)].length, 2, "the table and the versions are both given them");
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
    "Antigravity",
    "Build it yourself",
  ],
  cli: [
    "Commands",
    "Completions and manual pages",
    "How often providers are asked",
    "Accounts",
    "Antigravity's status line",
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
  assert.ok(table.kind === "table" && table.columns.join() === "Name,Meaning" && unmarked(table.rows[0][0]) === "<code>x</code>");
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

/** A guide's HTML without the marks that say where a word may break: they add no word and no space. */
const unmarked = (html: string) => html.replace(/<wbr>/g, "").replace(/<span class="whitespace-nowrap">([^<]*)<\/span>/g, "$1");

const DOWNLOAD = /<a href="([^"]*\/releases\/latest\/download\/([^"]+))"([^>]*)>/g;

test("each of the install guide's eight direct file links reports a download with its file, its system and where it was", () => {
  const cells = guide("install")
    .sections.flatMap((section) => section.blocks)
    .flatMap((block) => (block.kind === "table" ? block.rows.flat() : []))
    .join("\n");
  const links = [...cells.matchAll(DOWNLOAD)];
  assert.equal(links.length, 8, "four app files and four archives");
  const systems: Record<string, string> = { dmg: "macos", msi: "windows", deb: "linux", AppImage: "linux" };
  for (const [, , name, attributes] of links) {
    const system = systems[name.slice(name.lastIndexOf(".") + 1)] ?? (name.includes("darwin") ? "macos" : name.includes("windows") ? "windows" : "linux");
    assert.match(attributes, /data-event="download_click"/, `${name} reports a download`);
    assert.match(attributes, new RegExp(`data-event-file="${name.replace(/\./g, "\\.")}"`), `${name} names its file`);
    assert.match(attributes, new RegExp(`data-event-system="${system}"`), `${name} names its system`);
    assert.match(attributes, /data-event-from="install-guide"/, `${name} says it came from the install guide`);
  }
  assert.equal(unmarked(finish('<a href="https://example.com/x.dmg">x</a>', "guide")), '<a href="https://example.com/x.dmg">x</a>', "a link elsewhere is left alone");
});

test("the built /install page carries those events on its links, and a click on one reports the file", { skip: built ? false : NOT_BUILT }, () => {
  const page = builtFiles("install.html")[0].text;
  const links = [...page.matchAll(/<a [^>]*data-event="download_click"[^>]*data-event-from="install-guide"[^>]*>/g)];
  assert.equal(links.length, 8);
  for (const [link] of links) {
    const name = link.match(/data-event-file="([^"]+)"/)?.[1] ?? "none";
    assert.ok(link.includes(`/releases/latest/download/${name}"`), `${name}: the link fetches the file it reports`);
  }
});

test("a guide's code and links break only at a space or a mark, and its stylesheet never lets a word break anywhere", () => {
  /** A piece the browser may not break, as Breakable sets it. */
  const whole = (...pieces: string[]) => pieces.map((piece) => `<span class="whitespace-nowrap">${piece}</span>`).join("<wbr>");
  assert.equal(finish("<code>releases/latest/download/quotadial-aarch64.tar.gz</code>", "g"), `<code>${whole("releases/", "latest/", "download/", "quotadial-", "aarch64.tar.gz")}</code>`);
  assert.equal(
    finish("<code>shasum -c --ignore-missing</code>", "g"),
    `<code>${whole("shasum")} ${whole("-c")} ${whole("--ignore-missing")}</code>`,
    "a flag is one unbreakable piece, and the browser may break only at the spaces round it",
  );
  assert.equal(finish("<p>built-in and/or plain</p>", "g"), "<p>built-in and/or plain</p>", "words outside code and links are not marked");
  assert.equal(finish('<a href="/x">QuotaDial_universal.dmg</a>', "g"), `<a href="/x">${whole("QuotaDial_", "universal.dmg")}</a>`);
  for (const file of ["Guide.tsx", "GuidePage.tsx"]) {
    const source = readFileSync(new URL(`../components/reference/${file}`, import.meta.url), "utf8");
    assert.ok(!source.includes("anywhere]") && !source.includes("overflow-wrap:anywhere"), `${file} never breaks a word anywhere`);
  }
  const archive = guide("install")
    .sections.flatMap((section) => section.blocks)
    .find((block) => block.kind === "table" && block.caption.startsWith("The command alone"));
  assert.ok(archive?.kind === "table" && archive.rows[2][1].includes(whole("quotadial-", "v&lt;version&gt;-x86_", "64-", "unknown-", "linux-", "gnu.tar.gz")));
});

test("example output that scrolls shows the side that has more text, and is still a labelled region the keyboard reaches", { skip: built ? false : NOT_BUILT }, () => {
  const source = readFileSync(new URL("../components/CodeBlock.tsx", import.meta.url), "utf8");
  assert.match(source, /backgroundAttachment: "local, local, scroll, scroll"/, "covers travel with the text, shades stay at the edges");
  assert.match(source, /style=\{wrap \? undefined : SCROLL_SIGN\}/, "only a block that scrolls has the sign");
  for (const route of ["cli.html", "install.html"]) {
    const page = builtFiles(route)[0].text;
    const regions = [...page.matchAll(/<pre [^>]*role="region"[^>]*>/g)].map((found) => found[0]);
    for (const region of regions) {
      assert.match(region, /background-attachment:local,\s*local,\s*scroll,\s*scroll/);
      assert.match(region, /tabindex="0"/);
      assert.match(region, /aria-label="[^"]+"/);
      assert.ok(!/aria-hidden/.test(region));
    }
    const wrapped = [...page.matchAll(/<pre (?![^>]*role="region")[^>]*>/g)].map((found) => found[0]);
    assert.ok(wrapped.every((pre) => !pre.includes("background-attachment")), `${route}: a command that wraps has no sign`);
    if (route === "cli.html") {
      assert.ok(regions.length > 0, "the command guide has output that scrolls");
    }
  }
});
