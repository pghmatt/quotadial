/**
 * Where a line of code, a file name or an address may break, as the pieces between those places:
 * after a slash or an underscore, and after a hyphen inside a word, so
 * `releases/latest/download/quotadial-aarch64-apple-darwin.tar.gz` breaks between its parts and
 * never inside "latest". A word that starts with a hyphen is a command's flag and stays whole
 * (`--ignore-missing`), and so does a version (`0.2.0`). Spaces and line ends are kept as they are.
 */
export function breakPieces(text: string): string[] {
  const pieces: string[] = [];
  let piece = "";
  for (const word of text.split(/(\s+)/)) {
    if (word === "" || /^\s+$/.test(word) || word.startsWith("-")) {
      piece += word;
      continue;
    }
    for (let i = 0; i < word.length; i++) {
      piece += word[i];
      // After a slash (but not between the two of `https://`), an underscore, or a hyphen that
      // follows a letter or figure.
      const breaksAfter =
        (word[i] === "/" && word[i + 1] !== "/") ||
        word[i] === "_" ||
        (word[i] === "-" && i > 0 && /[\p{L}\p{N}]/u.test(word[i - 1]));
      if (breaksAfter && i < word.length - 1) {
        pieces.push(piece);
        piece = "";
      }
    }
  }
  if (piece !== "") pieces.push(piece);
  return pieces;
}

/** A run of spaces, where a line may always break, or a word, as the pieces between its marks. */
export type BreakToken = { space: string } | { pieces: string[] };

/**
 * `text` as spaces and words, each word in the pieces between the places it may break
 * (`breakPieces`). Set each piece unbreakable (`white-space: nowrap`) and join a word's pieces with
 * `<wbr>`: the browser then breaks only at a space or a mark, never after a hyphen of its own
 * choosing, so `--ignore-missing` and `status --json` keep their flags whole.
 */
export function breakTokens(text: string): BreakToken[] {
  return text
    .split(/(\s+)/)
    .filter((part) => part !== "")
    .map((part) => (/^\s+$/.test(part) ? { space: part } : { pieces: breakPieces(part) }));
}
