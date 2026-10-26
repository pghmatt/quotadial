/**
 * Commands in what QuotaDial says. A message names a command between backticks, as the CLI prints
 * it, such as "Run `CLAUDE_CONFIG_DIR=~/'My Claude' claude` to sign in again." The window shows each
 * one as code that a click selects whole, so it can be copied as it must be pasted.
 */

/** A run of a message: words, or a command to show as code. */
export interface Span {
  text: string;
  code: boolean;
}

/** Before a command's opening backtick: the start of the message, a space or an opening bracket. */
const OPENS_AFTER = /[\s([{]/;
/**
 * After a command's closing backtick: the end of the message, a space or the punctuation that ends
 * a phrase. A backtick inside a command never has one of these after it: in PowerShell a backtick
 * escapes the `, $ or quote that follows it (D-131), and in a POSIX command one can only be inside
 * single quotes, which close after it.
 */
const CLOSES_BEFORE = /[\s.,;:!?)\]}]/;

/**
 * `text` as words and commands, in order. A command opens at a backtick at the start or after a
 * space or bracket, with no space after it, and closes at the first backtick with no space before
 * it and the end, a space or a phrase's punctuation after it. A backtick that opens nothing it can
 * close stays in the words, so nothing said is ever lost.
 */
export function codeSpans(text: string): Span[] {
  const spans: Span[] = [];
  let words = "";
  let at = 0;
  while (at < text.length) {
    const close = text[at] === "`" ? closing(text, at) : -1;
    if (close < 0) {
      words += text[at];
      at += 1;
      continue;
    }
    if (words) {
      spans.push({ text: words, code: false });
      words = "";
    }
    spans.push({ text: text.slice(at + 1, close), code: true });
    at = close + 1;
  }
  if (words) {
    spans.push({ text: words, code: false });
  }
  return spans;
}

/** Where the command opened by the backtick at `open` closes, or -1 when it opens none. */
function closing(text: string, open: number): number {
  const before = open === 0 ? " " : text[open - 1];
  const first = text[open + 1];
  if (!OPENS_AFTER.test(before) || first === undefined || /\s|`/.test(first)) {
    return -1;
  }
  for (let at = open + 2; at < text.length; at += 1) {
    if (text[at] !== "`" || /\s/.test(text[at - 1])) {
      continue;
    }
    const after = text[at + 1];
    if (after === undefined || CLOSES_BEFORE.test(after)) {
      return at;
    }
  }
  return -1;
}
