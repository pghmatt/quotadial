// How the window check holds an address whole (whole-addresses.mjs, D-141): that each address is
// found wherever it stands, across the text nodes the window cuts it into at its <wbr>, and not when
// a character has been added to it; which addresses a copy of the window should write, in order;
// that the addresses the window writes are held to those before any line is judged; that a word of
// an address, the part between two places it may break, is broken only where it is wider than its
// line; and that the text of an address in a heading, and what a copy of it gives, is the address
// byte for byte. The page's own code runs here against a small stand-in for the page, and the
// browser's part, which boxes a word is drawn in and what a selection gives, is held by the window
// check itself.
import assert from "node:assert/strict";
import { test } from "node:test";
import { Script } from "node:vm";

import { inPage, tag, text } from "./page-stand-in.mjs";
import { between, randomTexts, seeded, seedOfThisRun } from "./random-text.mjs";
import {
  ADDRESSES,
  COPIED,
  addressVerdict,
  addressWordsIn,
  addressesIn,
  breakVerdicts,
  expectedAddresses,
  heardAt,
  knownAddresses,
  textVerdicts,
  visibly,
} from "./whole-addresses.mjs";
import { PAGE_TEXT, matchesIn } from "./whole-shares.mjs";
import { crowded, free, many, pro } from "./window-fixtures.mjs";

const LONG = "a.very.long.account.name.that.keeps.going@engineering.example.com";
const LONG_WORDS = ["a", ".very", ".long", ".account", ".name", ".that", ".keeps", ".going", "@engineering", ".example", ".com"];

/** The words of an address, as the check reads them. */
const wordsOf = (address) => addressWordsIn([address]).map((word) => word.text);

test("an address's words are what lies between its start, each @, each dot and each space", () => {
  assert.deepEqual(wordsOf(LONG), LONG_WORDS);
  assert.deepEqual(wordsOf("dev@example.com"), ["dev", "@example", ".com"]);
  // What names an account with no address is held by the same words: at a space, as a line breaks.
  assert.deepEqual(wordsOf("Google account"), ["Google", "account"]);
  assert.deepEqual(wordsOf(".credentials.json"), [".credentials", ".json"]);
  assert.deepEqual(wordsOf("a..b@"), ["a", ".", ".b", "@"]);
  assert.deepEqual(wordsOf(""), []);
});

test("an address cut into nodes, as the window writes one with a <wbr> between its parts, is one address with its words", () => {
  const texts = ["Weekly · Claude ", "max", "@example", ".com", " · resets in 2 d 6 h"];
  const [found] = addressesIn(texts, ["max@example.com"]);
  assert.equal(found.text, "max@example.com");
  assert.deepEqual(found.from, { node: 1, offset: 0 });
  assert.deepEqual(found.to, { node: 3, offset: 4 });
  assert.deepEqual(found.words, [
    { text: "max", from: { node: 1, offset: 0 }, to: { node: 1, offset: 3 } },
    { text: "@example", from: { node: 2, offset: 0 }, to: { node: 2, offset: 8 } },
    { text: ".com", from: { node: 3, offset: 0 }, to: { node: 3, offset: 4 } },
  ]);
  // As main's window writes it, in one node, it is the same address with the same words.
  const [whole] = addressesIn([LONG], [LONG]);
  assert.deepEqual(whole.words.map((word) => word.text), LONG_WORDS);
  assert.deepEqual(whole.words.at(-1), { text: ".com", from: { node: 0, offset: 61 }, to: { node: 0, offset: 65 } });
});

test("an address with a character added to make a break, such as a zero-width space, is not found", () => {
  for (const added of ["\u200b", "\u00ad", "\u2060", "\ufeff", " "]) {
    const written = `a.very${added}.long@example.com`;
    assert.deepEqual(addressesIn([written], ["a.very.long@example.com"]), [], visibly(written));
  }
  // Nor one with a letter more or less.
  assert.deepEqual(addressesIn(["dev@example.co"], ["dev@example.com"]), []);
});

test("an address stands with a space or the text's start or end on either side, the longest where two begin at one place", () => {
  const texts = (...parts) => addressesIn(parts, ["ops@example.com", "dev@example.com", "dev@example.com.au"]).map((found) => found.text);
  assert.deepEqual(texts("devops@example.com"), []);
  assert.deepEqual(texts("dev@example.com.au"), ["dev@example.com.au"]);
  assert.deepEqual(texts("dev@example.com and ops@example.com"), ["dev@example.com", "ops@example.com"]);
  assert.deepEqual(texts("(dev@example.com)"), []);
  assert.deepEqual(texts("Write to dev@example.com"), ["dev@example.com"]);
  // A character a regular expression reads is only itself in an address.
  assert.deepEqual(addressesIn(["a+b@x.com and a+b@xycom"], ["a+b@x.com"]).map((found) => found.text), ["a+b@x.com"]);
  assert.deepEqual(addressesIn(["axb@x.com"], ["a.b@x.com"]), []);
  assert.deepEqual(addressesIn(["dev@example.com"], []), []);
  assert.deepEqual(addressesIn(["dev@example.com"], [""]), []);
});

test("however the text is cut into nodes, each address found is the text between its start and end, and its words are its own", () => {
  const seed = seedOfThisRun();
  const next = seeded(seed);
  const addresses = ["dev@example.com", "a.b@c.d", "x@y", "Google account", LONG];
  // Addresses, the words of a line, spaces between them or not, and what is added to make a break.
  const pieces = [...addresses, ...addresses, " ", " ", " · ", "Claude", "@", ".", "\u200b", "(", ")"];
  const cases = 3000;
  let withAddresses = 0;
  for (let index = 0; index < cases; index += 1) {
    const texts = randomTexts(next, pieces, 8);
    const where = `seed ${seed}, case ${index}, ${JSON.stringify(texts)} (QUOTADIAL_TEST_SEED=${seed} runs it again)`;
    const all = texts.join("");
    const found = addressesIn(texts, addresses);
    for (const address of found) {
      assert.ok(addresses.includes(address.text), `found "${address.text}", which is no address: ${where}`);
      assert.equal(between(texts, address.from, address.to), address.text, `an address's start and end give other text: ${where}`);
      assert.equal(address.words.map((word) => word.text).join(""), address.text.replace(/\s/g, ""), `an address's words are not its own: ${where}`);
      for (const word of address.words) {
        assert.equal(between(texts, word.from, word.to), word.text, `a word's start and end give other text: ${where}`);
      }
    }
    // Every address standing in the text is found, and no other.
    const wanted = [...all.matchAll(new RegExp(`(?<!\\S)(?:${[...addresses].sort((a, b) => b.length - a.length).map((one) => one.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")).join("|")})(?!\\S)`, "gu"))].map((match) => match[0]);
    assert.deepEqual(found.map((address) => address.text), wanted, `the addresses found are not those standing in the text: ${where}`);
    withAddresses += found.length > 0 ? 1 : 0;
  }
  assert.ok(withAddresses >= cases / 10, `only ${withAddresses} of ${cases} texts held an address, seed ${seed}`);
});

/**
 * A copy of the window in the shape the address functions read: two accounts, one with a name given,
 * one with none, and an example account whose name says nothing beyond its provider.
 */
const small = {
  meter: {
    headline: "Weekly · Codex x.y@example.com · resets in 1 h",
    accounts: [
      { name: "dev@example.com", alias: "Work" },
      { name: "x.y@example.com", alias: null },
      { name: "", alias: null },
    ],
  },
  settings: { accounts: [{ name: "dev@example.com" }, { name: "x.y@example.com" }, { name: "" }] },
  history: { 7: { accounts: [{ name: "x.y@example.com" }] }, 30: { accounts: [{ name: "old@example.com" }] } },
  help: { support: "support@example.com" },
};

test("the addresses a copy should write: the summary's line's in every view, then each card's, each account's in History and Settings, and Help's", () => {
  assert.deepEqual(expectedAddresses(small, "accounts"), ["x.y@example.com", "dev@example.com", "x.y@example.com"]);
  assert.deepEqual(expectedAddresses(small, "history"), ["x.y@example.com", "x.y@example.com"]);
  assert.deepEqual(expectedAddresses(small, "settings"), ["x.y@example.com", "dev@example.com", "x.y@example.com"]);
  assert.deepEqual(expectedAddresses(small, "help"), ["x.y@example.com", "support@example.com"]);
  // A line under the number that names the account by the name given it, or no line yet, names no address.
  for (const headline of ["Weekly · Claude Work · resets in 1 h", null]) {
    const named = { ...small, meter: { ...small.meter, headline } };
    assert.deepEqual(expectedAddresses(named, "help"), ["support@example.com"]);
    assert.deepEqual(expectedAddresses(named, "accounts"), ["dev@example.com", "x.y@example.com"]);
  }
  // Every address the copy can write is looked for, once, whichever period History shows.
  assert.deepEqual(knownAddresses(small), ["dev@example.com", "x.y@example.com", "old@example.com", "support@example.com"]);
});

test("every copy writes addresses in each view it checks, and the long one in the popover's card, History and Settings' names", () => {
  for (const [name, copy] of Object.entries({ pro, free, many, crowded })) {
    for (const panel of copy.panels ?? ["accounts", "history", "settings", "help"]) {
      const wanted = expectedAddresses(copy, panel);
      assert.ok(wanted.length > 0, `${name} writes no address in ${panel}, so nothing there is held`);
      for (const address of wanted) {
        assert.ok(knownAddresses(copy).includes(address), `${name}'s ${JSON.stringify(address)} is not looked for`);
      }
    }
  }
  for (const copy of [pro, free, many]) {
    for (const panel of ["accounts", "history", "settings"]) {
      assert.ok(expectedAddresses(copy, panel).includes(LONG), `the long address is not held in ${panel}`);
    }
  }
  // The crowded copy's line under the number names its account by its address, in History, its one view.
  assert.deepEqual(expectedAddresses(crowded, "history"), ["max@example.com", "max@example.com"]);
  // The ten accounts' copy has an address with a part wider than the window, which only the last resort breaks.
  assert.ok(knownAddresses(many).some((address) => address.split("@")[0].length === 64));
});

test("the addresses written are the copy's, in order, or the verdict says which is not", () => {
  const address = (text) => ({ text, where: "div.address", room: 280, words: [] });
  const wanted = ["dev@example.com", LONG];
  assert.equal(addressVerdict(wanted.map(address), wanted), false);
  assert.equal(addressVerdict([], []), false);
  assert.match(addressVerdict([address("dev@example.com")], wanted), new RegExp(`^the window's address 2 written whole is no address, where its copy has "${LONG}" \\(1 written whole, 2 wanted\\)$`));
  assert.match(addressVerdict([...wanted, "x@y"].map(address), wanted), /^the window's address 3 written whole is "x@y" \(div\.address\), where its copy has none \(3 written whole, 2 wanted\)$/);
  assert.match(addressVerdict(wanted.toReversed().map(address), wanted), /address 1 written whole is "a\.very/);
  // An address with a zero-width space in it is not found whole, so the count falls short.
  assert.match(addressVerdict([address("dev@example.com")], wanted), /1 written whole, 2 wanted/);
});

/** A box, in CSS pixels. */
const box = (top, bottom, left = 0, right = 20) => ({ top, bottom, left, right });

test("a word of an address on one line is whole, whether its range is one box or two", () => {
  const found = [{ text: "dev@example.com", where: "div.address", room: 280, words: [
    { text: "dev", boxes: [box(0, 15, 0, 22)] },
    { text: "@example", boxes: [box(0, 15, 22, 60), box(0, 15, 60, 70)] },
    { text: ".com", boxes: [box(0, 15, 70, 94)] },
  ] }];
  assert.deepEqual(breakVerdicts(found), [false, false, false]);
  assert.deepEqual(breakVerdicts([]), []);
});

test('a word broken over two lines that fits its line is a verdict, as main\'s window drew "...going@engineeri" over "ng.example.com"', () => {
  // The Codex card's address at 100%, the browser breaking where the line ends: "@engineering" is
  // 79 px of a line 281 px wide, so a break before its @ would have taken it to the next line whole.
  const found = [{ text: LONG, where: 'div.address "a.very.long.account.name.that.keeps.going@engineering.example.com"', room: 281, words: [
    { text: ".going", boxes: [box(0, 15, 222, 258)] },
    { text: "@engineering", boxes: [box(0, 15, 258, 281), box(15, 30, 0, 56)] },
    { text: ".example", boxes: [box(15, 30, 56, 108)] },
  ] }];
  assert.deepEqual(breakVerdicts(found), [
    false,
    `the address "${LONG}" is broken inside "@engineering", over 2 lines, though "@engineering" fits its line (div.address "${LONG}")`,
    false,
  ]);
});

test("a word wider than its line has to break, and the stylesheet's last resort breaks it where the line ends: no verdict", () => {
  const name = "continuousintegrationsandboxfortheengineeringplatformteams2026ci";
  const found = (room, ...boxes) => [{ text: `${name}@example.com`, where: "div.address", room, words: [{ text: name, boxes }] }];
  // 392 px of a word on a line of 281: it cannot fit, so it breaks.
  assert.deepEqual(breakVerdicts(found(281, box(0, 15, 0, 281), box(15, 30, 0, 111))), [false]);
  // As wide as its line, to half a pixel, it fits, and breaking it is a verdict.
  assert.match(breakVerdicts(found(281, box(0, 15, 0, 200), box(15, 30, 0, 81.4)))[0], /is broken inside "continuous/);
  assert.deepEqual(breakVerdicts(found(281, box(0, 15, 0, 200), box(15, 30, 0, 82))), [false]);
});

test("a word with no box to measure is a verdict, and the verdicts come in the order the words are", () => {
  const found = [
    { text: "dev@example.com", where: "div.address", room: 100, words: [
      { text: "dev", boxes: [box(0, 15)] },
      { text: "@example", boxes: [] },
    ] },
    { text: "x@y", where: "button", room: 100, words: [{ text: "@y", boxes: [box(0, 15, 0, 9), box(15, 30, 0, 9)] }] },
  ];
  assert.deepEqual(breakVerdicts(found), [
    false,
    '"@example" of the address "dev@example.com" has no box to measure (div.address)',
    'the address "x@y" is broken inside "@y", over 2 lines, though "@y" fits its line (button)',
  ]);
});

test("a verdict writes what draws nothing, such as a zero-width space, as its code point", () => {
  assert.equal(visibly("a\u200b.b"), '"a\\u{200b}.b"');
  assert.equal(visibly("a\u00ad\u2060\ufeff\u00a0b c"), '"a\\u{ad}\\u{2060}\\u{feff}\\u{a0}b c"');
  assert.equal(visibly(LONG), JSON.stringify(LONG));
  assert.equal(visibly("x\ny"), '"x\\ny"');
});

test("an address in a heading is its text and what a copy gives, byte for byte, or the verdict says what was added", () => {
  const what = "Codex a.very.long";
  assert.deepEqual(textVerdicts(LONG, { text: LONG, copied: LONG }, what), [false, false]);
  const slipped = LONG.replace("@", "\u200b@");
  assert.deepEqual(textVerdicts(LONG, { text: slipped, copied: slipped }, what), [
    `${what}'s address is written ${visibly(slipped)}, not the address byte for byte, ${visibly(LONG)}`,
    `a copy of ${what}'s address gives ${visibly(slipped)}, not the address byte for byte, ${visibly(LONG)}`,
  ]);
  // Each is its own verdict: a copy that adds a line break, or a text that loses a letter.
  assert.deepEqual(textVerdicts(LONG, { text: LONG, copied: `${LONG}\n` }, what).map(Boolean), [false, true]);
  assert.deepEqual(textVerdicts(LONG, { text: LONG.slice(1), copied: LONG }, what).map(Boolean), [true, false]);
  // What is not read is not judged.
  assert.deepEqual(textVerdicts(LONG, { text: LONG }, what), [false, false]);
});

test("a line a screen reader meets as several entries, as one naming an address with a <wbr> in it is, is found where it begins", () => {
  const line = "Weekly · Claude max@example.com · resets in 2 d 6 h";
  // Each node of the line is an entry, read trimmed, with what comes before and after it.
  const heard = ["91%", "Weekly · Claude", "max", "@example", ".com", "· resets in 2 d 6 h", "History"];
  assert.equal(heardAt(heard, line), 1);
  // In one entry, as a line with no address in it is read.
  assert.equal(heardAt(["91%", line, "History"], line), 1);
  // Out of order, with something between, or cut short, it is not read as the line.
  assert.equal(heardAt(["Weekly · Claude", "@example", "max", ".com", "· resets in 2 d 6 h"], line), -1);
  assert.equal(heardAt(["Weekly · Claude", "max", "Codex", "@example", ".com", "· resets in 2 d 6 h"], line), -1);
  assert.equal(heardAt(["Weekly · Claude", "max", "@example"], line), -1);
  assert.equal(heardAt([], line), -1);
});

// The page's own code, run against a stand-in for the page (page-stand-in.mjs).

test("the code for the page is valid, looks for the addresses it is given and carries what the tests hold", () => {
  const code = ADDRESSES([LONG, "dev@example.com"]);
  assert.doesNotThrow(() => new Script(code));
  assert.ok(code.includes(JSON.stringify([LONG, "dev@example.com"])));
  for (const piece of [matchesIn, addressWordsIn, addressesIn]) {
    assert.ok(code.includes(String(piece)));
  }
  assert.ok(code.includes(PAGE_TEXT));
  assert.doesNotThrow(() => new Script(`(() => {${COPIED} return copied; })()`));
});

/** An account's heading's second line as the window writes it: the address a node a part, with a <wbr> between. */
function addressLine(address, width = 281, provider = null) {
  const parts = address.split(/(?=[@.])/);
  const name = tag("span", { classes: ["name"] }, ...parts.flatMap((part, index) => (index === 0 ? [text(part)] : [tag("wbr", {}), text(part)])));
  return tag("div", { classes: ["address"], display: "block", width }, name, ...(provider ? [text(" "), tag("span", {}, text("·")), text(" "), text(provider)] : []));
}

test("in the page, an address cut into nodes is found whole in its block, with each word's boxes and the line's room", () => {
  const asked = [];
  const found = inPage(ADDRESSES([LONG, "dev@example.com"]), [addressLine(LONG), addressLine("dev@example.com", 200, "Claude")], (start, end) => {
    asked.push([start[0].textContent, start[1], end[0].textContent, end[1]]);
    return [box(0, 15, 0, 30)];
  });
  assert.deepEqual(found.map((address) => [address.text, address.where, address.room]), [
    [LONG, `div.address "${LONG.slice(0, 59)}…"`, 281],
    ["dev@example.com", 'div.address "dev@example.com · Claude"', 200],
  ]);
  assert.deepEqual(found[0].words.map((word) => word.text), LONG_WORDS);
  assert.deepEqual(found[1].words.map((word) => [word.text, word.boxes]), [["dev", [box(0, 15, 0, 30)]], ["@example", [box(0, 15, 0, 30)]], [".com", [box(0, 15, 0, 30)]]]);
  // Each word's range is its own node, from its start to its end.
  assert.deepEqual(asked.slice(0, 3), [["a", 0, "a", 1], [".very", 0, ".very", 5], [".long", 0, ".long", 5]]);
  assert.equal(addressVerdict(found, [LONG, "dev@example.com"]), false);
});

test("in the page, main's address in one node, broken by the browser inside a word that fits the line, is a verdict", () => {
  const line = tag("div", { classes: ["address"], display: "block", width: 281 }, tag("span", { classes: ["name"] }, text(LONG)));
  const found = inPage(ADDRESSES([LONG]), [line], (start, end) =>
    start[1] === 41 && end[1] === 53 ? [box(0, 15, 258, 281), box(15, 30, 0, 56)] : [box(0, 15, 0, 20)],
  );
  assert.equal(addressVerdict(found, [LONG]), false);
  assert.deepEqual(breakVerdicts(found).filter(Boolean), [
    `the address "${LONG}" is broken inside "@engineering", over 2 lines, though "@engineering" fits its line (div.address "${LONG.slice(0, 59)}…")`,
  ]);
});

test("in the page, the summary's line and Help's button are found as any block is, and what is not shown is left out", () => {
  const headline = tag("p", { classes: ["headline"], display: "block", width: 300 }, text("Weekly · Claude "), text("max"), tag("wbr", {}), text("@example"), tag("wbr", {}), text(".com"), text(" · resets in 2 d 6 h"));
  const button = tag("button", { display: "inline-block", width: 120 }, text("Write to "), text("support"), tag("wbr", {}), text("@quotadial"), tag("wbr", {}), text(".com"));
  const closed = tag("div", { classes: ["address"], display: "block", shown: false }, text("dev@example.com"));
  const aside = tag("p", { classes: ["visually-hidden"], display: "block" }, text("dev@example.com"));
  const found = inPage(ADDRESSES(["max@example.com", "support@quotadial.com", "dev@example.com"]), [headline, button, closed, aside], () => [box(0, 15)]);
  assert.deepEqual(found.map((address) => [address.text, address.where]), [
    ["max@example.com", 'p.headline "Weekly · Claude max@example.com · resets in 2 d 6 h"'],
    ["support@quotadial.com", 'button "Write to support@quotadial.com"'],
  ]);
  assert.deepEqual(found[1].words.map((word) => word.text), ["support", "@quotadial", ".com"]);
});

test("in the page, an address with a zero-width space slipped in is not found, so the copy's addresses are not all written whole", () => {
  const slipped = tag("div", { classes: ["address"], display: "block", width: 281 }, tag("span", { classes: ["name"] }, text("dev\u200b@example.com")));
  const found = inPage(ADDRESSES(["dev@example.com"]), [slipped], () => [box(0, 15)]);
  assert.deepEqual(found, []);
  assert.match(addressVerdict(found, ["dev@example.com"]), /^the window's address 1 written whole is no address, where its copy has "dev@example\.com"/);
});
