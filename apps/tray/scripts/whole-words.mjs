// Holds a caption's words whole (the manager's decision on #73, 2026-10-27, 08:30 UTC, D-139). A word
// of four letters or more, such as "Weekly", is never drawn on two lines, as "Wee" over "kly", in
// a caption: at 200% zoom History's caption broke a short label so, because the body had
// `overflow-wrap: anywhere`, which counts a break between any two letters when the least room a flex
// item can be given is worked out, so a row of a label and the shares beside it squeezed the label
// below its own word. Only the lines that can carry text from outside with no spaces in it break
// anywhere now. The window check (check-window.mjs) finds, in the page, each caption the window
// draws and each word in it with the boxes the browser draws it in; what it makes of them, and which
// captions the window should be drawing, is here, so that tests which need no browser hold it. It
// reads the page as the check of shares does (whole-shares.mjs), and uses its pieces.

import { PAGE_TEXT, linesOf, matchesIn } from "./whole-shares.mjs";

/**
 * The captions the window draws, in the order it draws them: the line under an account's windows
 * saying where and when they were read, each History chart's caption (the window's label and its
 * latest and highest share) and the caption of each account's heatmap.
 */
export const CAPTIONS = ".source, .chart figcaption, .heatmap figcaption";

/**
 * Where each word is in the text of one block (`matchesIn`'s answer): a word, here, is four letters
 * or more in a row, in any script. "5-hour" holds "hour", and "GPT" and "and" are shorter than a
 * word. The page runs this too, beside `matchesIn`, which it names.
 */
export const wordsIn = (texts) => matchesIn(texts, /\p{L}{4,}/gu);

/** The words of one caption's text, as `wordsIn` finds them. */
export function wordsOf(text) {
  return wordsIn([text]).map((word) => word.text);
}

/**
 * The code to run in the page: each caption under `captions`, a CSS selector, in the order the page
 * has them, as `{ what, text, room, words }`: the caption as an element, its text (each block of it
 * read as one, whatever nodes it is in, and the blocks, such as a label and the shares beside it,
 * apart by a space), how wide it is in CSS pixels, and each of its words as `{ text, boxes }`, the
 * boxes being those the browser draws it in, one for each line it runs over (a range's client
 * rects). What is not shown, or only for a screen reader, is left out.
 */
export function CAPTION_WORDS(captions) {
  return `(() => {
    const matchesIn = ${matchesIn};
    const wordsIn = ${wordsIn};
    ${PAGE_TEXT}
    return [...document.querySelectorAll(${JSON.stringify(captions)})].map((caption) => {
      const blocks = new Map();
      addBlocks(caption, blocks, new Set());
      const lines = [];
      const words = [];
      for (const nodes of blocks.values()) {
        lines.push(nodes.map((node) => node.textContent).join(''));
        for (const word of wordsIn(nodes.map((node) => node.textContent))) {
          const range = document.createRange();
          range.setStart(nodes[word.from.node], word.from.offset);
          range.setEnd(nodes[word.to.node], word.to.offset);
          const boxes = [...range.getClientRects()].map(({ top, bottom, left, right }) => ({ top, bottom, left, right }));
          words.push({ text: word.text, boxes });
        }
      }
      return { what: named(caption), text: lines.join(' ').replace(/\\s+/g, ' ').trim(), room: caption.getBoundingClientRect().width, words };
    });
  })()`;
}

/**
 * The captions a copy of the window should draw in `panel`, in the order it draws them, as the text
 * of each: in the accounts, the line saying where and when an account's windows were read, for each
 * account the app sends one for; in History, for each account, a caption for each of its charts
 * (the label, then "latest" and "highest" with their shares, written as the window writes them,
 * rounded down) and then its heatmap's, when it was used at all. History shows the last 7 days at
 * first. Settings and Help have none.
 */
export function expectedCaptions(copy, panel) {
  const share = (used) => `${Math.floor(used)}%`;
  if (panel === "accounts") {
    return copy.meter.accounts.flatMap((account) => (account.fetched ? [account.fetched] : []));
  }
  if (panel === "history") {
    return copy.history[7].accounts.flatMap((account) => [
      ...account.windows.map((chart) => `${chart.label} latest ${share(chart.latest)}, highest ${share(chart.highest)}`),
      ...(account.heatmap.most > 0 ? [`When it was used, by hour (${account.heatmap.window.toLowerCase()})`] : []),
    ]);
  }
  return [];
}

/**
 * What is wrong with the captions `CAPTION_WORDS` found against the ones the copy should have, or false:
 * the window draws exactly `wanted`, in order, and the words found in each caption are the words
 * its text has, so that a check that finds no caption, or a caption's words, cannot pass.
 */
export function captionVerdict(captions, wanted) {
  const written = captions.map((caption) => caption.text);
  for (let at = 0; at < Math.max(written.length, wanted.length); at += 1) {
    if (written[at] !== wanted[at]) {
      const drawn = written[at] === undefined ? "no caption" : JSON.stringify(written[at]);
      const asked = wanted[at] === undefined ? "none" : JSON.stringify(wanted[at]);
      return `the window's caption ${at + 1} is ${drawn}, where its copy has ${asked} (${written.length} drawn, ${wanted.length} wanted)`;
    }
  }
  for (const { text, words } of captions) {
    const found = words.map((word) => word.text).join(" ");
    if (found !== wordsOf(text).join(" ")) {
      return `the words found in the caption ${JSON.stringify(text)} are ${JSON.stringify(found)}, not ${JSON.stringify(wordsOf(text).join(" "))}`;
    }
  }
  return false;
}

/**
 * A verdict on each word `CAPTION_WORDS` found: false where it is drawn on one line, or what is wrong,
 * naming the word and its caption. A word wider than its whole caption has to be broken somewhere
 * or it would be cut off, which the check of cut-off text holds, so it is not a verdict here; any
 * other, which would fit the caption on a line of its own, is whole or it is a verdict.
 */
export function wordVerdicts(captions) {
  return captions.flatMap(({ what, text, room, words }) =>
    words.map(({ text: word, boxes }) => {
      const lines = linesOf(boxes);
      if (lines === 0) {
        return `"${word}" has no box to measure (${what} ${JSON.stringify(text)})`;
      }
      // A word broken over lines is in a box for each, which together are as wide as the word.
      const width = boxes.reduce((sum, box) => sum + (box.right - box.left), 0);
      return lines > 1 && width <= room + 0.5 && `"${word}" is broken over ${lines} lines, though it fits its caption (${what} ${JSON.stringify(text)})`;
    }),
  );
}
