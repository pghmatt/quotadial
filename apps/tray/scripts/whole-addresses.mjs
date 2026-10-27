// Holds an address whole (the manager's decision on #73, 2026-10-27, 13:26 UTC, D-141). An address
// breaks before its @ and before each dot, where the window writes a <wbr>, and inside a word only
// where that word is wider than its line, by the stylesheet's last resort (D-139): before, a line that
// may break anywhere drew "a.very.long.account.name.that.keeps.going@engineeri" over
// "ng.example.com". And it is written byte for byte: no character is added to make a break, so its
// text, and what a copy of it gives, is the address. The window check (check-window.mjs) finds, in
// the page, each address the window writes and the boxes the browser draws each of its words in;
// what it makes of them, and which addresses the window should be writing, is here, so that tests
// which need no browser hold it. It reads the page as the checks of shares and caption words do
// (whole-shares.mjs), and uses their pieces.

import { PAGE_TEXT, linesOf, matchesIn } from "./whole-shares.mjs";

/**
 * Where each word of an address is in the text of one block (`matchesIn`'s answer): what lies
 * between the places a line may break in it, which are its start, each @ and each dot, where the
 * window writes a <wbr>, and each space, so "dev@example.com" is "dev", "@example" and ".com", and an
 * account's name with no address in it, such as "Google account", is its words. The page runs this
 * too, beside `matchesIn`, which it names.
 */
export const addressWordsIn = (texts) => matchesIn(texts, /[@.][^\s@.]*|[^\s@.]+/gu);

/**
 * Each of `addresses` written in the text of one block, which `texts` gives as the text of its nodes,
 * in order, as `matchesIn` gives a match, with its words (`addressWordsIn`): where it stands with a
 * space or the text's start or end on either side, the longest where two begin at one place, and so
 * never inside a longer word. An address written with any character added, such as a zero-width
 * space, is not found. The page runs this too, beside `matchesIn` and `addressWordsIn`, which it names.
 */
export const addressesIn = (texts, addresses) => {
  const known = [...new Set(addresses.filter(Boolean))].sort((a, b) => b.length - a.length);
  if (known.length === 0) {
    return [];
  }
  const escaped = known.map((address) => address.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"));
  const at = ({ node, offset }) => texts.slice(0, node).join("").length + offset;
  const words = addressWordsIn(texts);
  return matchesIn(texts, new RegExp(`(?<!\\S)(?:${escaped.join("|")})(?!\\S)`, "gu")).map((address) => ({
    ...address,
    words: words.filter((word) => at(address.from) <= at(word.from) && at(word.to) <= at(address.to)),
  }));
};

/**
 * The code to run in the page: each of `addresses` the window writes, in the order the page has them,
 * as `{ text, where, room, words }`: the address, the block it is written in as an element with the
 * start of its text, how wide a line of that block is in CSS pixels (its content box), and each of its
 * words as `{ text, boxes }`, the boxes being those the browser draws it in, one for each line it runs
 * over (a range's client rects). The text of a block is read as one, whatever nodes it is in, so an
 * address cut into a node a part, with a <wbr> between, is one address. What is not shown, or only
 * for a screen reader, is left out.
 */
export function ADDRESSES(addresses) {
  return `(() => {
    const matchesIn = ${matchesIn};
    const addressWordsIn = ${addressWordsIn};
    const addressesIn = ${addressesIn};
    ${PAGE_TEXT}
    // The boxes the browser draws the text of nodes between from and to in, one for each line.
    const boxesOf = (nodes, { from, to }) => {
      const range = document.createRange();
      range.setStart(nodes[from.node], from.offset);
      range.setEnd(nodes[to.node], to.offset);
      return [...range.getClientRects()].map(({ top, bottom, left, right }) => ({ top, bottom, left, right }));
    };
    // How wide a line of a block is: its width, less its padding and its border.
    const roomOf = (block) => {
      const style = getComputedStyle(block);
      const edges = ['paddingLeft', 'paddingRight', 'borderLeftWidth', 'borderRightWidth'].reduce((sum, edge) => sum + (parseFloat(style[edge]) || 0), 0);
      return block.getBoundingClientRect().width - edges;
    };
    const blocks = new Map();
    const seen = new Set();
    for (const root of document.querySelectorAll('main')) {
      addBlocks(root, blocks, seen);
    }
    const found = [];
    for (const [block, nodes] of blocks) {
      const texts = nodes.map((node) => node.textContent);
      // Where it is: the block, and its text as far as 60 characters.
      const said = texts.join('').replace(/\\s+/g, ' ').trim();
      for (const address of addressesIn(texts, ${JSON.stringify(addresses)})) {
        found.push({
          text: address.text,
          where: named(block) + ' "' + (said.length > 60 ? said.slice(0, 59) + '…' : said) + '"',
          room: roomOf(block),
          words: address.words.map((word) => ({ text: word.text, boxes: boxesOf(nodes, word) })),
        });
      }
    }
    return found;
  })()`;
}

/**
 * Every address a copy of the window can write, for `ADDRESSES` to look for: each account's own name
 * (its email address, or what names an account with none, such as "Google account") in the
 * popover, in History over either period and in Settings, and Help's support address.
 */
export function knownAddresses(copy) {
  const names = [
    ...copy.meter.accounts,
    ...Object.values(copy.history).flatMap((view) => view.accounts),
    ...copy.settings.accounts,
  ].map((account) => account.name);
  return [...new Set([...names, copy.help.support].filter(Boolean))];
}

/**
 * The addresses a copy of the window should write in `panel`, in the order it writes them: in every
 * view, those of its accounts that the line under the number names (it names the account whose
 * window sets the number by its address when it has no name given, D-132); then in the accounts each
 * card's, in History each account's, and in Settings each account's under Names, for each account
 * with a name of its own; and in Help the support address. History shows the last 7 days at first.
 */
export function expectedAddresses(copy, panel) {
  const names = (accounts) => accounts.map((account) => account.name).filter(Boolean);
  const headline = copy.meter.headline === null ? [] : addressesIn([copy.meter.headline], names(copy.meter.accounts));
  const inPanel = {
    accounts: names(copy.meter.accounts),
    history: names(copy.history[7].accounts),
    settings: names(copy.settings.accounts),
    help: [copy.help.support],
  }[panel];
  return [...headline.map((address) => address.text), ...(inPanel ?? [])];
}

/**
 * What is wrong with the addresses `ADDRESSES` found against the ones the copy should have, or false:
 * the window writes exactly `wanted`, whole, in order, so that a check that finds none cannot pass,
 * and an address written with a character added, which is not found, fails here.
 */
export function addressVerdict(found, wanted) {
  const written = found.map((address) => address.text);
  for (let at = 0; at < Math.max(written.length, wanted.length); at += 1) {
    if (written[at] !== wanted[at]) {
      const drawn = written[at] === undefined ? "no address" : `${JSON.stringify(written[at])} (${found[at].where})`;
      const asked = wanted[at] === undefined ? "none" : JSON.stringify(wanted[at]);
      return `the window's address ${at + 1} written whole is ${drawn}, where its copy has ${asked} (${written.length} written whole, ${wanted.length} wanted)`;
    }
  }
  return false;
}

/**
 * A verdict on each word of each address `ADDRESSES` found: false where it is drawn on one line, or
 * what is wrong, naming the word, its address and where it is. A word on two lines or more is broken
 * inside it, which is a verdict where the word fits its block's line, since a break before its @ or
 * dot, or at a space, would have let it go to the next line whole; a word wider than the line has to
 * break somewhere or be cut off, which the check of cut-off text holds, and the stylesheet's last
 * resort breaks it where the line ends.
 */
export function breakVerdicts(found) {
  return found.flatMap(({ text, where, room, words }) =>
    words.map(({ text: word, boxes }) => {
      const lines = linesOf(boxes);
      if (lines === 0) {
        return `"${word}" of the address "${text}" has no box to measure (${where})`;
      }
      // A word broken over lines is in a box for each, which together are as wide as the word.
      const width = boxes.reduce((sum, box) => sum + (box.right - box.left), 0);
      return (
        lines > 1 &&
        width <= room + 0.5 &&
        `the address "${text}" is broken inside "${word}", over ${lines} lines, though "${word}" fits its line (${where})`
      );
    }),
  );
}

/**
 * `text` in double quotes, with every character that draws nothing or only a space written as its
 * code point, such as "a\u{200b}.b" for a zero-width space, so that a verdict shows what was added.
 * A plain space is left as it is.
 */
export function visibly(text) {
  return JSON.stringify(text).replace(/[\p{Cc}\p{Cf}\p{Z}]/gu, (character) =>
    character === " " ? character : `\\u{${character.codePointAt(0).toString(16)}}`,
  );
}

/**
 * The code, for the page, of `copied(element)`: the text a person gets from a copy of the whole of
 * `element` selected, as a drag from its first letter to its last selects it: the selection's own
 * text, which a copy writes as text. The selection is taken away again.
 */
export const COPIED = `
    const copied = (element) => {
      const selection = window.getSelection();
      selection.removeAllRanges();
      selection.selectAllChildren(element);
      const text = selection.toString();
      selection.removeAllRanges();
      return text;
    };`;

/**
 * Where `line` begins among `heard`, the words a screen reader meets in order, each trimmed, as the
 * window check reads them from the accessibility tree: the first entry from which entries one after
 * another are the line, spaces aside, or -1. A line written in several nodes, as one naming an
 * address with a <wbr> in it is, is several entries read one after another, with nothing between.
 */
export function heardAt(heard, line) {
  const wanted = line.replace(/\s+/g, "");
  for (let first = 0; first < heard.length; first += 1) {
    let joined = "";
    for (let last = first; last < heard.length && joined.length < wanted.length; last += 1) {
      joined += heard[last].replace(/\s+/g, "");
      if (joined === wanted) {
        return first;
      }
    }
  }
  return -1;
}

/**
 * Verdicts on an address as the window writes it in a heading, against `address`, byte for byte: the
 * text of its element (`text`) and what a copy of it gives (`copied`, from `COPIED`), either left out
 * where it is not read. `what` names the account. A <wbr> adds nothing to either; a character added
 * to make a break, such as a zero-width space, is a verdict.
 */
export function textVerdicts(address, { text, copied }, what) {
  return [
    text !== undefined &&
      text !== address &&
      `${what}'s address is written ${visibly(text)}, not the address byte for byte, ${visibly(address)}`,
    copied !== undefined &&
      copied !== address &&
      `a copy of ${what}'s address gives ${visibly(copied)}, not the address byte for byte, ${visibly(address)}`,
  ];
}
