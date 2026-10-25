/**
 * The repository's own texts, rendered at build time: docs/PRIVACY.md for /privacy (D-106: one
 * privacy text everywhere) and CHANGELOG.md for /changelog (the same text the app's Help shows).
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
