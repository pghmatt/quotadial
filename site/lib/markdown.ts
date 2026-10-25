/**
 * The repository's own texts, rendered at build time and never copied: docs/PRIVACY.md for /privacy
 * (D-106: one privacy text everywhere), CHANGELOG.md for /changelog (the same text the app's Help
 * shows), and docs/INSTALL.md's files, check commands and unsigned-build steps for /download.
 */

import { readFileSync } from "node:fs";
import { join } from "node:path";

import { marked } from "marked";

/** A file in the repository, from the site's folder (where `next build` and the tests run). */
export function repositoryFile(path: string): string {
  return readFileSync(join(process.cwd(), "..", path), "utf8");
}

/** Markdown as HTML. The text is the repository's own, so its HTML is trusted. */
export function html(markdown: string): string {
  return marked.parse(markdown, { async: false, gfm: true });
}

/** A Markdown file without its first heading, which the page sets as its H1 instead. */
export function withoutTitle(markdown: string): { title: string; body: string } {
  const match = markdown.match(/^# (.+)\n/m);
  if (!match || match.index === undefined) {
    return { title: "", body: markdown };
  }
  return { title: match[1].trim(), body: markdown.slice(match.index + match[0].length) };
}

export interface ChangelogVersion {
  /** Such as "0.2.0", or "Unreleased". */
  version: string;
  /** Such as "2026-10-24"; none for Unreleased. */
  date: string | null;
  sections: { heading: string; html: string }[];
}

/** CHANGELOG.md's versions, newest first, each with its Added, Changed and Fixed. */
export function changelog(markdown: string): { intro: string; versions: ChangelogVersion[] } {
  const [head, ...parts] = markdown.split(/^## /m);
  const intro = withoutTitle(head).body.trim();
  const versions = parts.map((part) => {
    const [first, ...rest] = part.split("\n");
    const found = first.match(/^\[([^\]]+)\](?:\s*-\s*(\d{4}-\d{2}-\d{2}))?/);
    const sections = rest
      .join("\n")
      .split(/^### /m)
      .slice(1)
      .map((section) => {
        const [heading, ...lines] = section.split("\n");
        return { heading: heading.trim(), html: html(lines.join("\n").trim()) };
      });
    return { version: found?.[1] ?? first.trim(), date: found?.[2] ?? null, sections };
  });
  return { intro, versions };
}

/** Each line of `markdown`, with whether it sits inside a fenced code block. */
function lines(markdown: string): { text: string; code: boolean }[] {
  let code = false;
  return markdown.split("\n").map((text) => {
    const fence = /^\s*(```|~~~)/.test(text);
    const line = { text, code: code || fence };
    if (fence) {
      code = !code;
    }
    return line;
  });
}

/** A heading line's level and words, or null for any other line. */
function heading(line: { text: string; code: boolean }): { level: number; words: string } | null {
  const found = line.code ? null : line.text.match(/^(#{1,6}) (.+?)\s*$/);
  return found ? { level: found[1].length, words: found[2] } : null;
}

/**
 * The part of `markdown` under the heading `words`, without the heading, up to the next heading of
 * the same or a higher level. A section that is not there fails the build, so renaming one in the
 * repository cannot leave a page showing nothing.
 */
export function section(markdown: string, words: string): string {
  const all = lines(markdown);
  const start = all.findIndex((line) => heading(line)?.words === words);
  if (start === -1) {
    throw new Error(`The text has no section "${words}".`);
  }
  const level = heading(all[start])?.level ?? 1;
  const after = all.slice(start + 1);
  const end = after.findIndex((line) => {
    const found = heading(line);
    return found !== null && found.level <= level;
  });
  return (end === -1 ? after : after.slice(0, end))
    .map((line) => line.text)
    .join("\n")
    .trim();
}

/** Every table in `markdown`, each as its body rows, each cell without its code marks. */
export function tables(markdown: string): string[][][] {
  const found: string[][][] = [];
  let rows: string[] = [];
  const close = () => {
    if (rows.length > 2) {
      found.push(
        rows.slice(2).map((row) =>
          row
            .trim()
            .replace(/^\||\|$/g, "")
            .split("|")
            .map((cell) => cell.trim().replace(/^`(.*)`$/, "$1")),
        ),
      );
    }
    rows = [];
  };
  for (const line of lines(markdown)) {
    if (!line.code && line.text.trim().startsWith("|")) {
      rows.push(line.text);
    } else {
      close();
    }
  }
  close();
  return found;
}

/** A file's version-less copy: its name, and the address `releases/latest/download/` gives it. */
export interface InstallLink {
  name: string;
  href: string;
}

export interface InstallGuide {
  /** Each system's files, as INSTALL.md's Download table names them, with the copy that links. */
  files: { system: string; file: string; latest: InstallLink }[];
  /** The command that checks a download on each system. */
  checks: { system: string; command: string }[];
  /** The archives of the command on its own, each with its version-less copy. */
  commands: { system: string; archive: string; latest: InstallLink }[];
  /** The shell lines under "The command alone", that fetch an archive and put it on the PATH. */
  install: string;
  /** How to open a build that is not signed yet, each system's steps under its H3, as HTML. */
  unsigned: string;
}

/** Where INSTALL.md's version-less copies are: GitHub's address for a release's file, by name. */
const LATEST = "https://github.com/pghmatt/quotadial/releases/latest/download/";

/**
 * A table cell that links a version-less copy, `[name](address)`. The address must be
 * `releases/latest/download/` and the same name, so a typo in INSTALL.md fails the build instead of
 * reaching a visitor. Nothing here fetches the address.
 */
function latest(cell: string): InstallLink {
  const found = cell.match(/^\[([^\]]+)\]\(([^)]+)\)$/);
  if (!found || found[2] !== `${LATEST}${found[1]}`) {
    throw new Error(`docs/INSTALL.md's "${cell}" is not a link to a file under ${LATEST}.`);
  }
  return { name: found[1], href: found[2] };
}

/** The first fenced code block in `markdown`, without its fences. */
export function fenced(markdown: string): string {
  const found = markdown.match(/^\s*(```|~~~)[^\n]*\n([\s\S]*?)\n\s*\1\s*$/m);
  if (!found) {
    throw new Error("The text has no fenced code block.");
  }
  return found[2];
}

/** What /download shows from docs/INSTALL.md, read when the site is built. */
export function installGuide(markdown: string = repositoryFile("docs/INSTALL.md")): InstallGuide {
  const download = section(markdown, "Download");
  const [files, checks, commands] = tables(download);
  if (!files || !checks || !commands) {
    throw new Error("docs/INSTALL.md's Download section has lost its files, its check commands or its command archives.");
  }
  return {
    files: files.map(([system, file, copy]) => ({ system, file, latest: latest(copy ?? "") })),
    checks: checks.map(([system, command]) => ({ system, command })),
    commands: commands.map(([system, archive, copy]) => ({ system, archive, latest: latest(copy ?? "") })),
    install: fenced(section(download, "The command alone")),
    unsigned: html(section(markdown, "Open a build that is not signed yet")),
  };
}
