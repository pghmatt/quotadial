import { breakTokens } from "./breaks.ts";

const TAG = /(<[^>]+>)/;
/** The piece a word's text is set in, which the browser may not break inside. */
const PIECE = '<span class="whitespace-nowrap">';

/**
 * `text` (HTML-escaped) set as components/Breakable.tsx sets it, as HTML: spaces as they are, each
 * word's pieces unbreakable and joined by `<wbr>`, so the browser breaks only at a space or a mark
 * (`breakTokens`) and never after a hyphen of its own choosing.
 */
export function breakableHtml(text: string): string {
  return breakTokens(text)
    .map((token) => ("space" in token ? token.space : token.pieces.map((piece) => `${PIECE}${piece}</span>`).join("<wbr>")))
    .join("");
}

/**
 * HTML rendered from Markdown, with the words of its code and links set by `breakableHtml`, so a
 * file name, an address or a flag in running text breaks only where Breakable would break it. The
 * pieces it has made are left as they are, so marking twice changes nothing; a component that
 * renders HTML cannot use Breakable, which is why this is the same rule for a string.
 */
export function markBreaks(html: string): string {
  let inside = 0;
  let made = false;
  return html
    .split(TAG)
    .map((part) => {
      if (part.startsWith("<")) {
        if (part === PIECE) {
          made = true;
        } else if (made && /^<\/span>/i.test(part)) {
          made = false;
        } else if (/^<(code|a)[\s>]/i.test(part)) {
          inside += 1;
        } else if (/^<\/(code|a)>/i.test(part)) {
          inside = Math.max(0, inside - 1);
        }
        return part;
      }
      return inside && !made ? breakableHtml(part) : part;
    })
    .join("");
}
