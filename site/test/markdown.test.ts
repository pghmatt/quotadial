import assert from "node:assert/strict";
import { test } from "node:test";

import { DOWNLOADS } from "../lib/data/release.ts";
import { changelog, html, installGuide, repositoryFile, section, tables, withoutTitle } from "../lib/markdown.ts";
import { changelogCases, type ChangelogCase } from "./changelogs.ts";

test("the privacy page carries docs/PRIVACY.md and every address in it", () => {
  const { title, body } = withoutTitle(repositoryFile("docs/PRIVACY.md"));
  assert.equal(title, "What QuotaDial reads and sends");
  const rendered = html(body);
  for (const address of ["chatgpt.com", "status.claude.com", "status.openai.com", "github.com"]) {
    assert.ok(rendered.includes(address), `the page names ${address}`);
  }
  // Claude's limits come from Claude Code's status line, never with its sign-in (D-145).
  assert.ok(!rendered.includes("api.anthropic.com"), "the page names no address of Anthropic's");
  assert.ok(!rendered.includes("<h1"), "the file's own heading is the page's H1, set once");
});

/** Above zero when the release numbered `a` is newer than the one numbered `b`. */
function newer(a: string, b: string): number {
  const [x, y] = [a, b].map((version) => version.split(".").map(Number));
  const at = x.findIndex((part, index) => part !== y[index]);
  return at === -1 ? 0 : x[at] - y[at];
}

/**
 * The changelog's shape, which no release changes: an Unreleased entry, when there is one, comes
 * first and has no date; every other version is a release with its number and its date; and the
 * releases come newest first, by number and by date.
 */
function assertNewestFirst({ name, markdown }: ChangelogCase): void {
  const { versions } = changelog(markdown);
  versions.forEach((version, index) => {
    if (version.version === "Unreleased") {
      assert.equal(index, 0, `${name}: Unreleased comes before every release`);
      assert.equal(version.date, null, `${name}: Unreleased has no date`);
    }
  });
  const releases = versions.filter((version) => version.version !== "Unreleased");
  assert.ok(releases.length > 0, `${name}: lists its releases`);
  for (const release of releases) {
    assert.match(release.version, /^\d+\.\d+\.\d+$/, `${name}: ${release.version} is a version number`);
    assert.match(release.date ?? "", /^\d{4}-\d{2}-\d{2}$/, `${name}: ${release.version} has its date`);
  }
  releases.slice(1).forEach((older, index) => {
    const later = releases[index];
    assert.ok(newer(later.version, older.version) > 0, `${name}: ${later.version} comes before ${older.version}, newest first`);
    assert.ok((later.date ?? "") >= (older.date ?? ""), `${name}: ${later.version} is dated no earlier than ${older.version}`);
  });
}

test("the changelog's versions come newest first, today, on the version bump and after it", () => {
  for (const shape of changelogCases()) {
    assertNewestFirst(shape);
  }
});

test("a changelog is read as its versions in order, each with its date and its sections as HTML", () => {
  const { intro, versions } = changelog(
    [
      "# Changelog",
      "",
      "Opening words.",
      "",
      "## [Unreleased]",
      "",
      "## [0.3.0] - 2026-10-26",
      "",
      "### Added",
      "",
      "- One.",
      "- Two.",
      "",
      "### Fixed",
      "",
      "- Three.",
      "",
      "## [0.2.0] - 2026-10-24",
      "",
      "### Added",
      "",
      "- Four.",
      "",
      "[Unreleased]: https://github.com/pghmatt/quotadial/compare/v0.3.0...HEAD",
      "",
    ].join("\n"),
  );
  assert.equal(intro, "Opening words.");
  assert.deepEqual(
    versions.map((version) => [version.version, version.date, version.sections.map((part) => part.heading)]),
    [
      ["Unreleased", null, []],
      ["0.3.0", "2026-10-26", ["Added", "Fixed"]],
      ["0.2.0", "2026-10-24", ["Added"]],
    ],
  );
  assert.match(versions[1].sections[0].html, /<li>One\.<\/li>\s*<li>Two\.<\/li>/);
  assert.doesNotMatch(versions[2].sections[0].html, /compare/, "the links at the end are not a change");
});

test("the changelog tests run over the file, its version bump and an empty Unreleased above that", () => {
  const read = (markdown: string) =>
    changelogCases(markdown).map((shape) =>
      changelog(shape.markdown).versions.map((version) => `${version.version} ${version.date} ${version.sections.length}`),
    );
  const release = "## [0.2.0] - 2026-10-24\n\n### Added\n\n- Two.\n";
  assert.deepEqual(read(`# Changelog\n\n## [Unreleased]\n\n### Added\n\n- One.\n\n${release}`), [
    ["Unreleased null 1", "0.2.0 2026-10-24 1"],
    ["0.3.0 2026-10-25 1", "0.2.0 2026-10-24 1"],
    ["Unreleased null 0", "0.3.0 2026-10-25 1", "0.2.0 2026-10-24 1"],
  ]);
  assert.deepEqual(
    read(`# Changelog\n\n${release}`),
    [["0.2.0 2026-10-24 1"], ["Unreleased null 0", "0.2.0 2026-10-24 1"]],
    "a changelog with no Unreleased heading is already as the bump leaves it",
  );
  assert.throws(() => changelogCases("# Changelog\n\n## [Unreleased]\n\n- One.\n"), /names no release/);
  assert.equal(changelogCases()[0].markdown, repositoryFile("CHANGELOG.md"));
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
