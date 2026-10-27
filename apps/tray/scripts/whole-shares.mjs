// Holds a window's share whole (the manager's decision on #73, 2026-10-27). A share such as "64%" is
// one token: drawn on two lines, as "64" over "%", it reads as two things, and at 200% zoom a long
// label beside it left it so. The window check (check-window.mjs) finds, in the page, each share the
// window writes and the boxes the browser draws it in; what it makes of them, and which shares the
// window should be writing, is here, so that tests which need no browser hold it.

/**
 * Where each share is in the text of one block, which `texts` gives as the text of its nodes, in
 * order: the share's own text and the node and offset it starts at and ends at, `to` being the
 * end, exclusive. A share runs on from one node to the next, since Preact writes "64" and "%" as
 * two text nodes. A share is a figure and its sign, such as "64%" or "6.22%".
 *
 * The page runs this very function (`SHARES` puts its source in the page's code), so what the tests
 * hold is what the check runs; it names nothing outside itself.
 */
export function sharesIn(texts) {
  const ends = [];
  let total = 0;
  for (const text of texts) {
    total += text.length;
    ends.push(total);
  }
  // The node that holds the character at `position`, and where that node begins.
  const nodeAt = (position) => ends.findIndex((end) => position < end);
  const beginsAt = (node) => ends[node] - texts[node].length;
  const shares = [];
  for (const found of texts.join("").matchAll(/\d+(?:[.,]\d+)?%/g)) {
    const first = nodeAt(found.index);
    const last = nodeAt(found.index + found[0].length - 1);
    shares.push({
      text: found[0],
      from: { node: first, offset: found.index - beginsAt(first) },
      to: { node: last, offset: found.index + found[0].length - beginsAt(last) },
    });
  }
  return shares;
}

/**
 * The places a window's share is drawn as text, in the order the window writes them: the number
 * under the summary, a window's share in the accounts, and the latest and highest share in each
 * chart's caption in History.
 */
export const PLACES = ".percent, .row .used, .chart figcaption .detail";

/**
 * The code to run in the page: every share written in the text under `roots`, a CSS selector, as
 * `{ text, where, boxes }`, the boxes being those the browser draws it in, one for each line it
 * runs over (a range's client rects). The text of a block is read as one, whatever nodes it is in:
 * a block is the nearest ancestor that is not drawn inline. What is not shown, or only for a screen
 * reader, is left out.
 */
export function SHARES(roots) {
  return `(() => {
    const sharesIn = ${sharesIn};
    const named = (element) => element.tagName.toLowerCase() + [...element.classList].map((name) => '.' + name).join('');
    const shown = (element) => element.getClientRects().length > 0 && !element.closest('.visually-hidden') && getComputedStyle(element).visibility !== 'hidden';
    const blockOf = (node) => {
      let element = node.parentElement;
      while (element.parentElement && ['inline', 'contents'].includes(getComputedStyle(element).display)) element = element.parentElement;
      return element;
    };
    const blocks = new Map();
    const seen = new Set();
    for (const root of document.querySelectorAll(${JSON.stringify(roots)})) {
      const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
      for (let node = walker.nextNode(); node; node = walker.nextNode()) {
        if (seen.has(node) || !shown(node.parentElement)) continue;
        seen.add(node);
        const block = blockOf(node);
        blocks.set(block, [...(blocks.get(block) ?? []), node]);
      }
    }
    const found = [];
    for (const [block, nodes] of blocks) {
      for (const share of sharesIn(nodes.map((node) => node.textContent))) {
        const range = document.createRange();
        range.setStart(nodes[share.from.node], share.from.offset);
        range.setEnd(nodes[share.to.node], share.to.offset);
        const boxes = [...range.getClientRects()].map(({ top, bottom, left, right }) => ({ top, bottom, left, right }));
        const row = block.closest('li, figure, header, p') ?? block;
        found.push({ text: share.text, where: named(block) + ' in "' + row.textContent.replace(/\\s+/g, ' ').trim().slice(0, 60) + '"', boxes });
      }
    }
    return found;
  })()`;
}

/**
 * How many lines a piece of text is drawn on, from the boxes the browser gives it, one for each
 * line of it, taken from the top down: a box is on the line before it when it shares more than
 * half the height of the shorter of the two with the box that began that line, and begins another
 * when it does not. It compares the line boxes themselves and not a height, which a taller font, a
 * badge or a line height can change without a second line. A box with no width or no height is no
 * line.
 */
export function linesOf(boxes) {
  const firsts = [];
  const drawn = boxes.filter((box) => box.right > box.left && box.bottom > box.top);
  for (const box of drawn.toSorted((a, b) => a.top - b.top || a.left - b.left)) {
    const first = firsts.at(-1);
    const shared = first ? Math.min(first.bottom, box.bottom) - Math.max(first.top, box.top) : 0;
    if (!first || shared <= Math.min(first.bottom - first.top, box.bottom - box.top) / 2) {
      firsts.push(box);
    }
  }
  return firsts.length;
}

/**
 * A verdict on each share `SHARES` found: false where it is drawn on one line, or what is wrong,
 * naming the share and where it is.
 */
export function lineVerdicts(found) {
  return found.map(({ text, where, boxes }) => {
    const lines = linesOf(boxes);
    if (lines === 0) {
      return `"${text}" has no box to measure (${where})`;
    }
    return lines > 1 && `"${text}" is drawn on ${lines} lines (${where})`;
  });
}

/**
 * The shares a copy of the window should write in `panel`, in the order it writes them: the number
 * under the summary in every view, and then in the accounts one for each window, in the order of
 * the accounts, and in History two for each chart, its latest and its highest. History shows the
 * last 7 days at first.
 */
export function expectedShares(copy, panel) {
  const share = (used) => `${Math.floor(used)}%`;
  const summary = copy.meter.percent === null ? [] : [share(copy.meter.percent)];
  if (panel === "accounts") {
    return [
      ...summary,
      ...copy.meter.accounts.flatMap((account) => account.windows.map((window) => share(window.used_pct))),
    ];
  }
  if (panel === "history") {
    return [
      ...summary,
      ...copy.history[7].accounts.flatMap((account) =>
        account.windows.flatMap((chart) => [share(chart.latest), share(chart.highest)]),
      ),
    ];
  }
  return summary;
}

/** Whether `text`, a string of the share picture, is a share: "64%" or "highest 64%". */
export function isShare(text) {
  return /^(?:highest )?\d+(?:[.,]\d+)?%$/.test(text);
}
