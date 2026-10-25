/**
 * What the reference pages (/privacy, /support, /changelog) do to the repository's own texts, and
 * nothing more: set an address as an address, count what a changelog lists, say a date in words,
 * and take the example of `quotadial doctor` from the command guide. Each function leaves the
 * text's words exactly as they are.
 */

import { repositoryFile } from "../../lib/markdown.ts";
import type { ChangelogVersion } from "../../lib/markdown.ts";

const HOST = "[a-z0-9]+(?:[.-][a-z0-9]+)*\\.[a-z]{2,}";

/**
 * Sets every address in rendered HTML in the code face with the class `address`: the bold host
 * names PRIVACY.md writes, and the two addresses it gives in plain words (127.0.0.1 and
 * quotadial.com/key). Only tags change; the text between them is untouched.
 */
export function setAddresses(rendered: string): string {
  const bold = rendered.replace(new RegExp(`<strong>(${HOST})</strong>`, "gi"), '<code class="address">$1</code>');
  let inCode = false;
  return bold
    .split(/(<[^>]+>)/)
    .map((part) => {
      if (part.startsWith("<")) {
        if (/^<code[\s>]/i.test(part)) {
          inCode = true;
        } else if (/^<\/code>/i.test(part)) {
          inCode = false;
        }
        return part;
      }
      return inCode ? part : part.replace(/\b(127\.0\.0\.1|quotadial\.com\/key)(?![\w/])/g, '<code class="address">$1</code>');
    })
    .join("");
}

/** The words of rendered HTML, with its tags and the spacing between them taken out. */
export function wordsOf(rendered: string): string {
  return rendered
    .replace(/<[^>]+>/g, "")
    .replace(/&amp;/g, "&")
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">")
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'")
    .replace(/\s+/g, " ")
    .trim();
}

const MONTHS = [
  "January",
  "February",
  "March",
  "April",
  "May",
  "June",
  "July",
  "August",
  "September",
  "October",
  "November",
  "December",
];

/** "2026-10-24" as "24 October 2026". */
export function sayDate(iso: string): string {
  const found = iso.match(/^(\d{4})-(\d{2})-(\d{2})$/);
  if (!found) {
    throw new Error(`Not a date: ${iso}`);
  }
  const month = MONTHS[Number(found[2]) - 1];
  if (!month) {
    throw new Error(`Not a month: ${iso}`);
  }
  return `${Number(found[3])} ${month} ${found[1]}`;
}

/** A version's anchor on the changelog page. */
export function versionId(version: string): string {
  return `v-${version.toLowerCase().replace(/[^a-z0-9]+/g, "-")}`;
}

/** How many changes a version lists under a heading such as "Added"; none if it has no such part. */
export function countUnder(version: ChangelogVersion, heading: string): number {
  const section = version.sections.find((candidate) => candidate.heading === heading);
  return section ? (section.html.match(/<li[\s>]/g) ?? []).length : 0;
}

/** The headings the changelog uses, in the order they first appear. */
export function headingsUsed(versions: ChangelogVersion[]): string[] {
  const seen: string[] = [];
  for (const version of versions) {
    for (const section of version.sections) {
      if (!seen.includes(section.heading)) {
        seen.push(section.heading);
      }
    }
  }
  return seen;
}

/** The example of what `quotadial doctor` prints, from the command guide's Doctor part. */
export function doctorSample(guide: string = repositoryFile("docs/CLI.md")): string {
  const part = guide.split(/^## Doctor$/m)[1];
  const block = part?.match(/```text\n([\s\S]*?)\n```/);
  if (!block) {
    throw new Error("docs/CLI.md has no example under its Doctor heading");
  }
  return block[1];
}
