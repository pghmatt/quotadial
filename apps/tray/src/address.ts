/**
 * Where an address may break over two lines (the manager's decision on #73, 2026-10-27, 13:26 UTC,
 * D-141): before its `@` and before each `.`, so that a long one goes over two lines as its name and
 * its domain, "a.very.long.account.name.that.keeps.going" over "@engineering.example.com", where a
 * line that may break anywhere cut it inside a word. The window writes a `<wbr>` element at each of
 * those places and never a character, so the address's text, and what a copy of it gives, is the
 * address byte for byte, and hides each from screen readers, so the address is heard whole (`Address`
 * in `account.tsx`). A part wider than the line still breaks where it must, by the stylesheet's
 * `overflow-wrap: anywhere` (D-139), its last resort.
 */

/**
 * `address` as the parts a line may break between, in order: a new part at each `@` and `.` after
 * its first character, so none at its start, and the parts joined are the address. A name with
 * neither, such as "CLAUDE_CODE_OAUTH_TOKEN", is one part; an empty one has none.
 */
export function addressParts(address: string): string[] {
  const parts: string[] = [];
  let from = 0;
  for (let at = 1; at < address.length; at += 1) {
    if (address[at] === "@" || address[at] === ".") {
      parts.push(address.slice(from, at));
      from = at;
    }
  }
  if (address) {
    parts.push(address.slice(from));
  }
  return parts;
}

/** A run of a line: words, or one of the addresses it names, which the window writes with its breaks. */
export interface Run {
  text: string;
  address: boolean;
}

/**
 * `text` as runs, in order: each of `addresses` that stands in it, with a space or the line's start
 * or end on either side, as a run of its own, the longest where two begin at one place, and the
 * words between them as runs of words. The runs joined are the text. It finds the address an app's
 * line names an account by, such as the summary's "Weekly · Claude max@example.com · resets in
 * 2 d 6 h", and never one inside a longer word, as "ops@example.com" is inside
 * "devops@example.com".
 */
export function addressRuns(text: string, addresses: readonly string[]): Run[] {
  const known = [...new Set(addresses.filter((address) => address.length > 0))].sort(
    (a, b) => b.length - a.length,
  );
  // Whether a space, or the line's start or end, is beside `at`: before it or at it.
  const spaceBefore = (at: number) => at === 0 || /\s/.test(text[at - 1]);
  const spaceAt = (at: number) => at === text.length || /\s/.test(text[at]);
  const runs: Run[] = [];
  let words = "";
  let at = 0;
  while (at < text.length) {
    const found = spaceBefore(at)
      ? known.find((address) => text.startsWith(address, at) && spaceAt(at + address.length))
      : undefined;
    if (found === undefined) {
      words += text[at];
      at += 1;
      continue;
    }
    if (words) {
      runs.push({ text: words, address: false });
      words = "";
    }
    runs.push({ text: found, address: true });
    at += found.length;
  }
  if (words) {
    runs.push({ text: words, address: false });
  }
  return runs;
}
