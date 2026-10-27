// How the window check holds a window's share whole (whole-shares.mjs): that each share is found
// wherever it is written, across the text nodes Preact splits it into; that its line boxes, and not
// a height, say whether it is drawn on one line or two; and which shares a copy of the window should
// write. The page's own code runs here against a small stand-in for the page, and the browser's
// part, which boxes a share is drawn in, is held by the window check itself.
import assert from "node:assert/strict";
import { test } from "node:test";
import { Script } from "node:vm";

import { inPage, tag, text } from "./page-stand-in.mjs";
import { between, randomTexts, seeded, seedOfThisRun } from "./random-text.mjs";
import { rules } from "./stylesheet.mjs";
import { PAGE_TEXT, PLACES, SHARES, expectedShares, isShare, lineVerdicts, linesOf, matchesIn, sharesIn } from "./whole-shares.mjs";
import { crowded, free, many, pro } from "./window-fixtures.mjs";

test("a share is found whole across the nodes it is written in, with where it starts and ends", () => {
  // Preact writes {percent}% as two text nodes.
  assert.deepEqual(sharesIn(["64", "%"]), [{ text: "64%", from: { node: 0, offset: 0 }, to: { node: 1, offset: 1 } }]);
  // History's caption, "latest 8%, highest 11%", is five.
  assert.deepEqual(sharesIn(["latest ", "8", "%, highest ", "11", "%"]), [
    { text: "8%", from: { node: 1, offset: 0 }, to: { node: 2, offset: 1 } },
    { text: "11%", from: { node: 3, offset: 0 }, to: { node: 4, offset: 1 } },
  ]);
  // One node, with other figures in the line.
  const line = "$12.50 of $50.00 used (25%)";
  const at = line.indexOf("25%");
  assert.deepEqual(sharesIn([line]), [
    { text: "25%", from: { node: 0, offset: at }, to: { node: 0, offset: at + 3 } },
  ]);
  assert.deepEqual(sharesIn(["6.22", "%", " and ", "100%"]).map((share) => share.text), ["6.22%", "100%"]);
  // Empty nodes between the figure and its sign change nothing.
  assert.deepEqual(sharesIn(["6", "", "4", "", "%"])[0], { text: "64%", from: { node: 0, offset: 0 }, to: { node: 4, offset: 1 } });
});

test("what is not a figure and its sign is not a share", () => {
  for (const texts of [[], [""], ["%"], ["64"], ["64 %"], ["64", " ", "%"], ["64", "\n", "%"], ["a%"], ["weekly"], ["at 8,%"]]) {
    assert.deepEqual(sharesIn(texts), [], JSON.stringify(texts));
  }
  // A rate has a figure and its sign too, and is held whole as well.
  assert.deepEqual(sharesIn(["at 8.5%/h"]).map((share) => share.text), ["8.5%"]);
});

test("however the text is cut into nodes, each share found is the text between its start and end, and none is missed", () => {
  // A new seed each run; the seed is in every failure, and QUOTADIAL_TEST_SEED=<seed> runs the same strings again.
  const seed = seedOfThisRun();
  const next = seeded(seed);
  // Figures and signs, which make shares, and what breaks one: a space, a line break, a letter, a
  // bracket, a stop, a comma, and a character that takes two UTF-16 units.
  const pieces = [..."0123456789012345678901234567890123456789", "%", "%", ".", ",", " ", "\n", "a", "(", ")", "/", "\u{1f600}"];
  const cases = 5000;
  let withShares = 0;
  for (let index = 0; index < cases; index += 1) {
    // Cut at random places, in the middle of a share as often as not, leaving some nodes empty.
    const texts = randomTexts(next, pieces);
    const where = `seed ${seed}, case ${index}, ${JSON.stringify(texts)} (QUOTADIAL_TEST_SEED=${seed} runs it again)`;
    const wanted = [...texts.join("").matchAll(/\d+(?:[.,]\d+)?%/g)].map((found) => found[0]);
    const found = sharesIn(texts);
    assert.deepEqual(
      found.map((share) => share.text),
      wanted,
      `the shares found are not those in the text: ${where}`,
    );
    for (const share of found) {
      assert.equal(between(texts, share.from, share.to), share.text, `a share's start and end give other text: ${where}`);
      assert.ok(share.from.offset < texts[share.from.node].length, `a share starts past the end of its node: ${where}`);
      assert.ok(share.to.offset >= 1 && share.to.offset <= texts[share.to.node].length, `a share ends outside its node: ${where}`);
    }
    withShares += found.length > 0 ? 1 : 0;
  }
  assert.ok(withShares >= cases / 10, `only ${withShares} of ${cases} strings held a share, seed ${seed}`);
});

/** A box, in CSS pixels. */
const box = (top, bottom, left = 0, right = 20) => ({ top, bottom, left, right });

test("a share on one line is one line, whether its figure and its sign are one box or two", () => {
  assert.equal(linesOf([box(10, 26, 0, 27)]), 1);
  assert.equal(linesOf([box(10, 26, 0, 16), box(10, 26, 16, 27)]), 1);
});

test('a share on two lines is two, "64" over "%", and on three is three', () => {
  assert.equal(linesOf([box(10, 26, 0, 16), box(26, 42, 0, 11)]), 2);
  assert.equal(linesOf([box(10, 26, 0, 8), box(26, 42, 0, 8), box(42, 58, 0, 11)]), 3);
});

test("the line boxes decide, not a height: a taller box on the line is no second line, and boxes that overlap a little are two", () => {
  // A badge's box or a larger sign, beside a figure, reaches above and below it and is on its line.
  assert.equal(linesOf([box(10, 26, 0, 16), box(8, 29, 16, 40)]), 1);
  // 34 px type with a line height of 1.1: each line's box is 41 px tall and the next begins 37.4 px lower.
  assert.equal(linesOf([box(0, 41, 0, 40), box(37.4, 78.4, 0, 20)]), 2);
  // A single box as tall as three lines of small text is one box, and so is not told apart from one line:
  // the browser gives a box for each line, so a share on three lines comes as three.
  assert.equal(linesOf([box(0, 48, 0, 30)]), 1);
});

test("a box with no width or no height is no line, and the order the boxes come in does not matter", () => {
  assert.equal(linesOf([]), 0);
  assert.equal(linesOf([box(5, 5, 0, 20), box(5, 20, 3, 3)]), 0);
  const boxes = [box(10, 26, 0, 16), box(26, 42, 0, 11), box(42, 58, 0, 9)];
  for (const order of [[0, 1, 2], [2, 1, 0], [1, 0, 2], [1, 2, 0], [2, 0, 1], [0, 2, 1]]) {
    assert.equal(linesOf(order.map((at) => boxes[at])), 3, order.join());
  }
  // Two boxes of one line with another line's between them in the list are still one line.
  const apart = [box(10, 26, 0, 16), box(26, 42, 0, 11), box(10, 26, 16, 27)];
  for (const order of [[0, 1, 2], [0, 2, 1], [1, 0, 2], [1, 2, 0], [2, 0, 1], [2, 1, 0]]) {
    assert.equal(linesOf(order.map((at) => apart[at])), 2, order.join());
  }
  // And two boxes on the second line, after one on the first, are two lines.
  assert.equal(linesOf([box(10, 26, 0, 16), box(26, 42, 0, 11), box(26, 42, 11, 30)]), 2);
});

test("a verdict for each share: none where it is on one line, and what is wrong, with where, where it is not", () => {
  const found = [
    { text: "91%", where: 'p.percent in "91%"', boxes: [box(0, 38, 0, 70)] },
    { text: "64%", where: 'span.used.normal in "Weekly (Claude and GPT)64%resets in 3 d 2 h"', boxes: [box(0, 16, 0, 16), box(16, 32, 0, 11)] },
    { text: "8%", where: 'span.nobreak in "x"', boxes: [] },
  ];
  assert.deepEqual(lineVerdicts(found), [
    false,
    '"64%" is drawn on 2 lines (span.used.normal in "Weekly (Claude and GPT)64%resets in 3 d 2 h")',
    '"8%" has no box to measure (span.nobreak in "x")',
  ]);
  assert.deepEqual(lineVerdicts([]), []);
});

test("the shares a copy should write are the summary's, then each window's, and each chart's latest and highest", () => {
  // Claude Work (91, 74, 38), the Codex account (12, 0), Antigravity (64, 25, 6), the second Codex (31, 18) and the one with one window (63).
  assert.deepEqual(expectedShares(pro, "accounts"), ["91%", "91%", "74%", "38%", "12%", "0%", "64%", "25%", "6%", "31%", "18%", "63%"]);
  // The Free copy lists the example account as well, after the others.
  assert.equal(expectedShares(free, "accounts").length, expectedShares(pro, "accounts").length + 1);
  assert.equal(expectedShares(free, "accounts").at(-1), "91%");
  // History: two for each chart after the summary's.
  const charts = (copy) => copy.history[7].accounts.reduce((sum, account) => sum + account.windows.length, 0);
  for (const copy of [pro, free, many, crowded]) {
    const history = expectedShares(copy, "history");
    assert.equal(history.length, 1 + 2 * charts(copy));
    assert.ok(history.every((share) => /^\d+%$/.test(share)));
  }
  assert.deepEqual(expectedShares(pro, "history").slice(0, 3), ["91%", "91%", "91%"]);
  // A chart's latest comes before its highest: Claude Work's session, where the two differ.
  const session = pro.history[7].accounts[0].windows.find((chart) => chart.label === "Session");
  assert.notEqual(Math.floor(session.latest), Math.floor(session.highest));
  const history = expectedShares(pro, "history");
  const at = history.findIndex((share, index) => share === `${Math.floor(session.latest)}%` && history[index + 1] === `${Math.floor(session.highest)}%`);
  assert.equal(at, 1 + 2 * 2, "after the summary's number, Weekly's two and Weekly Opus's two");
  // History opens on 7 days; the other period is not asked for.
  assert.deepEqual(expectedShares({ ...pro, history: { 7: pro.history[7] } }, "history"), history);
  // The account of seven windows: the summary's 100%, then each chart's latest and its highest, 100% in every one.
  assert.deepEqual(expectedShares(crowded, "history"), ["100%", "64%", "100%", "100%", "100%", "7%", "100%", "99%", "100%", "12%", "100%", "38%", "100%", "91%", "100%"]);
  // Settings and Help show the summary's number and no more; with no number there is nothing.
  assert.deepEqual(expectedShares(pro, "settings"), ["91%"]);
  assert.deepEqual(expectedShares(pro, "help"), ["91%"]);
  const none = { ...pro, meter: { ...pro.meter, percent: null } };
  assert.deepEqual(expectedShares(none, "settings"), []);
  assert.deepEqual(expectedShares(none, "accounts"), expectedShares(pro, "accounts").slice(1));
});

test("a share is written rounded down, as the window writes it", () => {
  const window = (used_pct) => ({ used_pct });
  const chart = (latest, highest) => ({ latest, highest });
  const copy = {
    meter: { percent: 63, accounts: [{ windows: [window(63.9), window(99.99), window(0.4)] }] },
    history: { 7: { accounts: [{ windows: [chart(12.5, 99.99), chart(100, 100)] }] } },
  };
  assert.deepEqual(expectedShares(copy, "accounts"), ["63%", "63%", "99%", "0%"]);
  assert.deepEqual(expectedShares(copy, "history"), ["63%", "12%", "99%", "100%", "100%"]);
});

test("a share of the picture is a figure and its sign, or 'highest' and one, and a label or a date is not", () => {
  for (const text of ["64%", "0%", "100%", "6.22%", "highest 11%", "highest 100%", "90%"]) {
    assert.equal(isShare(text), true, text);
  }
  for (const text of ["Weekly (Claude and GPT)", "The last 7 days, 21 to 27 October 2026", "highest", "highest %", "64% used", "Mon 20", "6…", "highest 1…", "", "QuotaDial"]) {
    assert.equal(isShare(text), false, text);
  }
});

// The page's own code, run against a stand-in for the page (page-stand-in.mjs).

/**
 * What `SHARES(roots)` finds in a page whose roots are `roots`, with `drawnIn(start, end)` giving the
 * boxes a range from `start` to `end`, each `[node, offset]`, is drawn in.
 */
const found = (roots, drawnIn) => inPage(SHARES(".percent"), roots, drawnIn);

test("the code for the page is valid, names the places it is given and carries the function the tests hold", () => {
  const code = SHARES(PLACES);
  assert.doesNotThrow(() => new Script(code));
  assert.ok(code.includes(JSON.stringify(PLACES)));
  assert.ok(code.includes(String(sharesIn)));
  // What `sharesIn` calls, and what reads the page's text, are in the page's code too.
  assert.ok(code.includes(String(matchesIn)));
  assert.ok(code.includes(PAGE_TEXT));
});

test("in the page, a share written as two text nodes is one share, and its boxes are those of its range", () => {
  const nodes = [text("64"), text("%")];
  const used = tag("span", { classes: ["used", "normal"], display: "block" }, ...nodes);
  tag("li", { classes: ["window"], display: "list-item" }, tag("span", { display: "block" }, text("Weekly (Claude and GPT)")), used);
  const asked = [];
  const shares = found([used], (start, end) => {
    asked.push([start[0].textContent, start[1], end[0].textContent, end[1]]);
    return [{ top: 0, bottom: 16, left: 0, right: 16 }, { top: 16, bottom: 32, left: 0, right: 11 }];
  });
  assert.deepEqual(asked, [["64", 0, "%", 1]]);
  assert.equal(shares.length, 1);
  assert.equal(shares[0].text, "64%");
  assert.equal(shares[0].where, 'span.used.normal in "Weekly (Claude and GPT)64%"');
  assert.equal(shares[0].boxes.length, 2);
  assert.deepEqual(lineVerdicts(shares), ['"64%" is drawn on 2 lines (span.used.normal in "Weekly (Claude and GPT)64%")']);
});

test("in the page, a share inside an inline span is read with the words around it, and each is found", () => {
  const caption = tag(
    "span",
    { classes: ["detail"], display: "block" },
    text("latest "),
    tag("span", { classes: ["nobreak"] }, text("8"), text("%")),
    text(", highest "),
    tag("span", { classes: ["nobreak"] }, text("11%")),
  );
  const asked = [];
  const shares = found([caption], (start, end) => {
    asked.push([start[0].textContent, start[1], end[0].textContent, end[1]]);
    return [{ top: 0, bottom: 14, left: 0, right: 20 }];
  });
  assert.deepEqual(shares.map((share) => share.text), ["8%", "11%"]);
  assert.deepEqual(asked, [["8", 0, "%", 1], ["11%", 0, "11%", 3]]);
  assert.deepEqual(lineVerdicts(shares), [false, false]);
});

test("in the page, a figure and its sign in different inline elements are one share, and a share in the middle of a text is where it is", () => {
  const block = tag("p", { display: "block" }, tag("em", {}, text("64")), text("%"));
  const asked = [];
  const shares = found([block], (start, end) => {
    asked.push([start[0].textContent, start[1], end[0].textContent, end[1]]);
    return [{ top: 0, bottom: 16, left: 0, right: 30 }];
  });
  assert.deepEqual(shares.map((share) => share.text), ["64%"]);
  assert.deepEqual(asked, [["64", 0, "%", 1]]);
  // 80% starts after "reaches " and ends 3 characters on.
  const where = [];
  found([tag("p", { display: "block" }, text("reaches 80% soon"))], (start, end) => {
    where.push([start[1], end[1]]);
    return [];
  });
  assert.deepEqual(where, [[8, 11]]);
});

test("in the page, a figure at the end of one block and a sign at the start of the next are not a share", () => {
  const first = tag("p", { display: "block" }, text("up to 64"));
  const second = tag("p", { display: "block" }, text("% of it"));
  assert.deepEqual(found([first, second], () => []), []);
});

test("in the page, what is not shown, or is for a screen reader only, is left out, and a text under two roots is read once", () => {
  const closed = tag("details", { display: "block", shown: false }, text("17%"));
  const aside = tag("ul", { classes: ["visually-hidden"], display: "block" }, tag("li", { display: "list-item" }, text("Mon 20: used 9:00 to 18:00 (12%)")));
  const open = tag("p", { display: "block" }, text("91%"));
  const everything = tag("div", { display: "block" }, closed, aside, open);
  const drawn = () => [{ top: 0, bottom: 16, left: 0, right: 20 }];
  assert.deepEqual(found([everything], drawn).map((share) => share.text), ["91%"]);
  // The page's own roots can hold one another, and a share in both is found once.
  assert.deepEqual(found([everything, open], drawn).map((share) => share.text), ["91%"]);
});

test("style.css keeps each share on one line, and a window's share its width, so the label beside it is what wraps", () => {
  const declaring = (selector) => rules().filter((rule) => rule.selectors.includes(selector));
  // The summary's number, a window's share and a share in History's caption.
  for (const selector of [".percent", ".row .used", ".nobreak"]) {
    const spaces = declaring(selector).flatMap((rule) => rule.declarations.filter((declaration) => declaration.startsWith("white-space")));
    assert.deepEqual(spaces, ["white-space: nowrap"], `${selector} does not keep its share on one line`);
  }
  assert.ok(
    declaring(".row .used").some((rule) => rule.declarations.includes("flex: none")),
    "a window's share can be squeezed by the label beside it",
  );
});
