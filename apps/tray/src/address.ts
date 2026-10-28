/**
 * Where an address may break over two lines (the manager's decision on #73, 2026-10-27, 13:26 UTC,
 * D-141): before its `@` and before each `.`, so that a long one goes over two lines as its name and
 * its domain, "a.very.long.account.name.that.keeps.going" over "@engineering.example.com", where a
 * line that may break anywhere cut it inside a word. The window writes a `<wbr>` element at each of
 * those places and never a character, so the address's text, and what a copy of it gives, is the
 * address byte for byte, and hides each from screen readers, so the address is heard whole (`Address`
 * in `account.tsx`). A part wider than the line still breaks where it must, by the stylesheet's
 * `overflow-wrap: anywhere` (D-139), its last resort.
 *
 * A host written in a sentence breaks the same way (D-141, extended to a host: the manager's
 * review of #107): it has no `@`, so its parts are before each `.`, and
 * "release-assets.githubusercontent.com" goes over two lines as "release-assets" over
 * ".githubusercontent.com", never as "...githubuserco" over "ntent.com". The window finds a host
 * only from a list the app gives it (`hostRuns`), never by guessing at a word with a dot in it, as
 * a file's name has too.
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

/** A run of a line: words, or one of the hosts it names, which the window writes with breaks. */
export interface HostRun {
  text: string;
  host: boolean;
}

/**
 * `text` as runs, in order: each of `names` that stands in it, as `stands(at, length)` says of one
 * found at `at`, as a run of its own, the longest where two begin at one place, and the words
 * between them as runs of words, `named` false. The runs joined are the text.
 */
function namedRuns(
  text: string,
  names: readonly string[],
  stands: (at: number, length: number) => boolean,
): { text: string; named: boolean }[] {
  const known = [...new Set(names.filter((name) => name.length > 0))].sort(
    (a, b) => b.length - a.length,
  );
  const runs: { text: string; named: boolean }[] = [];
  let words = "";
  let at = 0;
  while (at < text.length) {
    const found = known.find((name) => text.startsWith(name, at) && stands(at, name.length));
    if (found === undefined) {
      words += text[at];
      at += 1;
      continue;
    }
    if (words) {
      runs.push({ text: words, named: false });
      words = "";
    }
    runs.push({ text: found, named: true });
    at += found.length;
  }
  if (words) {
    runs.push({ text: words, named: false });
  }
  return runs;
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
  // Whether a space, or the line's start or end, is beside `at`: before it or at it.
  const spaceBefore = (at: number) => at === 0 || /\s/.test(text[at - 1]);
  const spaceAt = (at: number) => at === text.length || /\s/.test(text[at]);
  return namedRuns(text, addresses, (at, length) => spaceBefore(at) && spaceAt(at + length)).map(
    ({ text, named }) => ({ text, address: named }),
  );
}

/**
 * `text` as runs, in order: each of `hosts` that stands in it as a word of a sentence does, as a
 * run of its own, the longest where two begin at one place, and the words between them as runs of
 * words. The runs joined are the text. A host stands with the line's start, a space, a bracket or a
 * quotation mark before it, and the line's end, a space, a mark of punctuation or the stop that
 * ends the sentence after it, so "chatgpt.com, OpenAI's" and "Also sent to
 * hooks.slack.com." name a host, and "api.github.com", "github.com.au", "github.com/pghmatt" and
 * "someone@github.com" do not name "github.com".
 */
export function hostRuns(text: string, hosts: readonly string[]): HostRun[] {
  const startsAt = (at: number) => at === 0 || /[\s(["']/.test(text[at - 1]);
  const endsAt = (at: number) =>
    at === text.length ||
    /[\s,;:!?)\]"']/.test(text[at]) ||
    // A stop that ends the sentence, not a dot that goes on into a longer host.
    (text[at] === "." && (at + 1 === text.length || /\s/.test(text[at + 1])));
  return namedRuns(text, hosts, (at, length) => startsAt(at) && endsAt(at + length)).map(
    ({ text, named }) => ({ text, host: named }),
  );
}
