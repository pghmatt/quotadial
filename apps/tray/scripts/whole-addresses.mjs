// Holds an address whole (the manager's decision on #73, 2026-10-27, 13:26 UTC, D-141). An address
// breaks before its @ and before each dot, where the window writes a <wbr>, and inside a word only
// where that word is wider than its line, by the stylesheet's last resort (D-139): before, a line that
// may break anywhere drew "a.very.long.account.name.that.keeps.going@engineeri" over
// "ng.example.com". And it is written byte for byte: no character is added to make a break, so its
// text, and what a copy of it gives, is the address. A host in a sentence is held the same way
// (D-141, extended to a host by the manager's review of #107): it has no @, so it breaks before each
// dot, and "release-assets.githubusercontent.com" never goes over two lines as "...githubuserco"
// over "ntent.com". The window check (check-window.mjs) finds, in the page, each address and host the
// window writes and the boxes the browser draws each of its words in; what it makes of them, and
// which addresses and hosts the window should be writing, is here, so that tests which need no
// browser hold it. It reads the page as the checks of shares and caption words do (whole-shares.mjs),
// and uses their pieces.

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
 * Each of `names` written in the text of one block, which `texts` gives as the text of its nodes, in
 * order, as `matchesIn` gives a match, with its words: where what `before` matches is just before it
 * and what `after` matches just after it (a regular expression's lookbehind and lookahead), the
 * longest where two begin at one place. Its words are those `addressWordsIn` finds in its own text, so
 * a mark beside it, such as the comma after a host, is in none. A name written with any character
 * added, such as a zero-width space, is not found. The page runs this too, beside `matchesIn` and
 * `addressWordsIn`, which it names.
 */
const namesIn = (texts, names, before, after) => {
  const known = [...new Set(names.filter(Boolean))].sort((a, b) => b.length - a.length);
  if (known.length === 0) {
    return [];
  }
  const escaped = known.map((name) => name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"));
  const pattern = new RegExp(`(?<=${before})(?:${escaped.join("|")})(?=${after})`, "gu");
  return matchesIn(texts, pattern).map((found) => {
    // The text of each node as far as it is the name's, so that the words found in it are its own.
    const own = texts.map((text, node) =>
      node < found.from.node || node > found.to.node
        ? ""
        : text.slice(node === found.from.node ? found.from.offset : 0, node === found.to.node ? found.to.offset : text.length),
    );
    const back = ({ node, offset }) => ({ node, offset: node === found.from.node ? offset + found.from.offset : offset });
    return { ...found, words: addressWordsIn(own).map((word) => ({ ...word, from: back(word.from), to: back(word.to) })) };
  });
};

/**
 * Each of `addresses` written in the text of one block (`namesIn`), with its words: where it stands
 * with a space or the text's start or end on either side, and so never inside a longer word. The
 * page runs this too.
 */
export const addressesIn = (texts, addresses) => namesIn(texts, addresses, String.raw`^|\s`, String.raw`$|\s`);

/**
 * Each of `hosts` written in the text of one block (`namesIn`), with its words: where it stands as a
 * word of a sentence does, as the window finds a host to write it as one (`hostRuns` in
 * src/address.ts): with the text's start, a space, a bracket or a quotation mark before it, and the
 * text's end, a space, a mark of punctuation or the stop that ends a sentence after it, and so never
 * inside a longer host, an address or a link. Only a host the app lists is looked for, never a word
 * with a dot in it, as a file's name has. The page runs this too.
 */
export const hostsIn = (texts, hosts) =>
  namesIn(texts, hosts, String.raw`^|[\s(\["']`, String.raw`$|[\s,;:!?)\]"']|\.(?:$|\s)`);

/**
 * The code to run in the page: each of `names` that `find` (`addressesIn` or `hostsIn`) finds the
 * window writing, in the order the page has them, as `{ text, where, room, words }`: the address or
 * host, the block it is written in as an element with the start of its text, how wide a line of that
 * block is in CSS pixels (its content box), and each of its words as `{ text, boxes }`, the boxes
 * being those the browser draws it in, one for each line it runs over (a range's client rects). The
 * text of a block is read as one, whatever nodes it is in, so an address or a host cut into a node a
 * part, with a <wbr> between, is one. What is not shown, or only for a screen reader, is left out.
 */
function written(find, names) {
  return `(() => {
    const matchesIn = ${matchesIn};
    const addressWordsIn = ${addressWordsIn};
    const namesIn = ${namesIn};
    const find = ${find};
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
      for (const address of find(texts, ${JSON.stringify(names)})) {
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

/** The code to run in the page: each of `addresses` the window writes (`written`). */
export const ADDRESSES = (addresses) => written(addressesIn, addresses);

/** The code to run in the page: each of `hosts` the window writes (`written`). */
export const HOSTS = (hosts) => written(hostsIn, hosts);

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
 * Every host a copy of the window can write, for `HOSTS` to look for in every view: the hosts of the
 * webhooks alerts are also sent to, and every place QuotaDial connects to, which Help's text of what
 * QuotaDial reads and sends names. Both are lists the app sends; no host is guessed from a word with
 * a dot in it.
 */
export function knownHosts(copy) {
  return [...new Set([...copy.settings.alert_targets, ...copy.help.hosts].filter(Boolean))];
}

/** Each line of Help's text of what QuotaDial reads and sends, in order: a paragraph, or an item of a list. */
const privacyLines = (copy) => copy.help.privacy.flatMap((block) => (block.kind === "paragraph" ? [block.text] : block.items));

/**
 * The hosts a copy of the window should write in `panel`, in the order it writes them: in Settings,
 * each webhook's host, in the line saying where alerts are also sent; in Help, each place QuotaDial
 * connects to where it stands in the text of what QuotaDial reads and sends (`hostsIn`), a line at a
 * time; and no other view writes one.
 */
export function expectedHosts(copy, panel) {
  if (panel === "settings") {
    return copy.settings.alert_targets.filter(Boolean);
  }
  if (panel === "help") {
    return privacyLines(copy).flatMap((line) => hostsIn([line], copy.help.hosts).map((host) => host.text));
  }
  return [];
}

/**
 * The lines a copy of the window writes hosts in, in `panel`, in order, as `{ what, line }`: in
 * Settings, the line saying where alerts are also sent, naming each webhook's host, when there is
 * one; in Help, each line of its text of what QuotaDial reads and sends, as the app sends it. No
 * other view writes a host.
 */
export function linesWithHosts(copy, panel) {
  if (panel === "settings") {
    const targets = copy.settings.alert_targets;
    return targets.length === 0
      ? []
      : [{ what: "Settings' line saying where alerts are also sent", line: `Also sent to ${targets.join(", ")}.` }];
  }
  if (panel === "help") {
    return privacyLines(copy).map((line, index) => ({ what: `line ${index + 1} of Help's text of what QuotaDial reads and sends`, line }));
  }
  return [];
}

/**
 * What is wrong with the addresses `ADDRESSES` found, or the hosts `HOSTS` found, against the ones
 * the copy should have, or false, `kind` naming which: the window writes exactly `wanted`, whole, in
 * order, so that a check that finds none cannot pass, and one written with a character added, which
 * is not found, fails here.
 */
export function addressVerdict(found, wanted, kind = "address") {
  const written = found.map((one) => one.text);
  for (let at = 0; at < Math.max(written.length, wanted.length); at += 1) {
    if (written[at] !== wanted[at]) {
      const drawn = written[at] === undefined ? `no ${kind}` : `${JSON.stringify(written[at])} (${found[at].where})`;
      const asked = wanted[at] === undefined ? "none" : JSON.stringify(wanted[at]);
      return `the window's ${kind} ${at + 1} written whole is ${drawn}, where its copy has ${asked} (${written.length} written whole, ${wanted.length} wanted)`;
    }
  }
  return false;
}

/**
 * A verdict on each word of each address `ADDRESSES` found, or each host `HOSTS` found, `kind` naming
 * which: false where it is drawn on one line, or what is wrong, naming the word, its address or host
 * and where it is. A word on two lines or more is broken inside it, which is a verdict where the word
 * fits its block's line, since a break before its @ or dot, or at a space, would have let it go to the
 * next line whole; a word wider than the line has to break somewhere or be cut off, which the check
 * of cut-off text holds, and the stylesheet's last resort breaks it where the line ends.
 */
export function breakVerdicts(found, kind = "address") {
  return found.flatMap(({ text, where, room, words }) =>
    words.map(({ text: word, boxes }) => {
      const lines = linesOf(boxes);
      if (lines === 0) {
        return `"${word}" of the ${kind} "${text}" has no box to measure (${where})`;
      }
      // A word broken over lines is in a box for each, which together are as wide as the word.
      const width = boxes.reduce((sum, box) => sum + (box.right - box.left), 0);
      return (
        lines > 1 &&
        width <= room + 0.5 &&
        `the ${kind} "${text}" is broken inside "${word}", over ${lines} lines, though "${word}" fits its line (${where})`
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
 * Verdicts on what the window writes, against `wanted`, byte for byte: the text of its element
 * (`text`) and what a copy of it gives (`copied`, from `COPIED`), either left out where it is not
 * read. `subject` names what is written, and `thing` what it should be, such as "the line". A <wbr>
 * adds nothing to either; a character added to make a break, such as a zero-width space, is a verdict.
 */
export function writtenVerdicts(wanted, { text, copied }, subject, thing) {
  return [
    text !== undefined &&
      text !== wanted &&
      `${subject} is written ${visibly(text)}, not ${thing} byte for byte, ${visibly(wanted)}`,
    copied !== undefined &&
      copied !== wanted &&
      `a copy of ${subject} gives ${visibly(copied)}, not ${thing} byte for byte, ${visibly(wanted)}`,
  ];
}

/**
 * Verdicts on an address as the window writes it in a heading, against `address`, byte for byte
 * (`writtenVerdicts`). `what` names the account.
 */
export function textVerdicts(address, seen, what) {
  return writtenVerdicts(address, seen, `${what}'s address`, "the address");
}
