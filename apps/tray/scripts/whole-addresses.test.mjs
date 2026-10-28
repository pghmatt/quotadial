// How the window check holds an address whole (whole-addresses.mjs, D-141): that each address is
// found wherever it stands, across the text nodes the window cuts it into at its <wbr>, and not when
// a character has been added to it; which addresses a copy of the window should write, in order;
// that the addresses the window writes are held to those before any line is judged; that a word of
// an address, the part between two places it may break, is broken only where it is wider than its
// line; and that the text of an address in a heading, and what a copy of it gives, is the address
// byte for byte. A host in a sentence is held the same way (D-141, extended to a host by the
// manager's review of #107): found only from the lists the app sends, where it stands as a word of a
// sentence does, exactly where the window writes one as a host; which hosts a copy should write, in
// Settings and Help; and the lines that name them, their text and what a copy gives, byte for byte.
// The page's own code runs here against a small stand-in for the page, and the browser's part, which
// boxes a word is drawn in and what a selection gives, is held by the window check itself.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { Script } from "node:vm";

import { addressParts, hostRuns } from "../src/address.ts";
import { inPage, tag, text } from "./page-stand-in.mjs";
import { between, randomTexts, seeded, seedOfThisRun } from "./random-text.mjs";
import {
  ADDRESSES,
  COPIED,
  HOSTS,
  addressVerdict,
  addressWordsIn,
  addressesIn,
  breakVerdicts,
  expectedAddresses,
  expectedHosts,
  heardAt,
  hostsIn,
  knownAddresses,
  knownHosts,
  linesWithHosts,
  textVerdicts,
  visibly,
  writtenVerdicts,
} from "./whole-addresses.mjs";
import { PAGE_TEXT, matchesIn } from "./whole-shares.mjs";
import { site } from "./site-fixtures.mjs";
import { DESTINATIONS, PRIVACY, crowded, free, many, pro } from "./window-fixtures.mjs";

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

// A host in a sentence (D-141, extended to a host: the manager's review of #107).

const RELEASES = "release-assets.githubusercontent.com";
const LONG_HOST = "alerts.engineering.platform.internal.corp.example.co.uk";

/** The hosts found in `texts`, as text. */
const hostsOf = (texts, hosts = DESTINATIONS) => hostsIn(texts, hosts).map((host) => host.text);

test("a host cut into nodes, as the window writes one with a <wbr> before each dot, is one host with its words, and the mark after it is in none", () => {
  const texts = ["", "release-assets", ".githubusercontent", ".com", ", GitHub's, where an update's files are fetched from."];
  const [found] = hostsIn(texts, DESTINATIONS);
  assert.equal(found.text, RELEASES);
  assert.deepEqual(found.from, { node: 1, offset: 0 });
  assert.deepEqual(found.to, { node: 3, offset: 4 });
  assert.deepEqual(found.words, [
    { text: "release-assets", from: { node: 1, offset: 0 }, to: { node: 1, offset: 14 } },
    { text: ".githubusercontent", from: { node: 2, offset: 0 }, to: { node: 2, offset: 18 } },
    { text: ".com", from: { node: 3, offset: 0 }, to: { node: 3, offset: 4 } },
  ]);
  // As main's window writes Settings' line, in one node, with the stop that ends the sentence.
  const said = `Also sent to hooks.slack.com, ${LONG_HOST}.`;
  const hosts = hostsIn([said], ["hooks.slack.com", LONG_HOST]);
  assert.deepEqual(hosts.map((host) => host.text), ["hooks.slack.com", LONG_HOST]);
  assert.deepEqual(hosts[0].words.map((word) => word.text), ["hooks", ".slack", ".com"]);
  assert.deepEqual(hosts[1].words.map((word) => word.text), ["alerts", ".engineering", ".platform", ".internal", ".corp", ".example", ".co", ".uk"]);
  assert.deepEqual(hosts[1].words.at(-1), { text: ".uk", from: { node: 0, offset: said.length - 4 }, to: { node: 0, offset: said.length - 1 } });
  // A bracket before it is in no word either.
  assert.deepEqual(hostsIn(["(status.openai.com)"], DESTINATIONS)[0].words.map((word) => word.text), ["status", ".openai", ".com"]);
});

test("a host stands after the line's start, a space, a bracket or a quotation mark, and before its end, a space, a mark or the stop that ends the sentence", () => {
  assert.deepEqual(hostsOf(["chatgpt.com, OpenAI's, with your Codex CLI sign-in."]), ["chatgpt.com"]);
  assert.deepEqual(hostsOf(["status.claude.com and status.openai.com, the providers' public status pages."]), ["status.claude.com", "status.openai.com"]);
  assert.deepEqual(hostsOf(["Updates come from github.com."]), ["github.com"]);
  for (const [open, close] of [["(", ")"], ["[", "]"], ['"', '"'], ["'", "'"]]) {
    assert.deepEqual(hostsOf([`See ${open}chatgpt.com${close} now`]), ["chatgpt.com"], `${open}${close}`);
  }
  for (const mark of [",", ";", ":", "!", "?", ".", ""]) {
    assert.deepEqual(hostsOf([`Is it github.com${mark}`]), ["github.com"], JSON.stringify(mark));
    assert.deepEqual(hostsOf([`github.com${mark} then`]), ["github.com"], JSON.stringify(mark));
  }
});

test("a host is never found inside a longer name, an address or a link, nor with a character added", () => {
  for (const written of [
    "api.github.com",
    "github.com.au",
    "github.com.example",
    "github.comx",
    "github.com-mirror",
    "github.com/pghmatt/quotadial",
    "https://github.com/pghmatt",
    "someone@github.com",
    "mygithub.com",
    "github.com.",
    "status\u200b.openai.com",
    "status.openai.com\u200b",
    "status.openai\u00ad.com",
  ]) {
    const found = hostsOf([`Not ${written} here`]);
    // "github.com." ends with the sentence's stop only when nothing follows on; here a space does.
    assert.deepEqual(found, written === "github.com." ? ["github.com"] : [], visibly(written));
  }
});

test("only a host the app lists is found, never a word with a dot in it, as a file's name has", () => {
  const line = "Open config.json, QuotaDial_universal.dmg and docs/ACCESSIBILITY.md, or models.dev.";
  assert.deepEqual(hostsOf([line]), []);
  assert.deepEqual(hostsOf([line], ["config.json"]), ["config.json"]);
  assert.deepEqual(hostsOf(["github.com"], []), []);
  assert.deepEqual(hostsOf(["github.com"], [""]), []);
  // The longer wins where two begin at one place, and a shorter one inside a longer is not found.
  assert.deepEqual(hostsOf([`${RELEASES}, GitHub's`], ["githubusercontent.com", RELEASES]), [RELEASES]);
  assert.deepEqual(hostsOf(["status.claude.com"], ["claude.com", "status.claude.com"]), ["status.claude.com"]);
});

test("however a line is cut into nodes, the hosts found in it are those the window writes as hosts, with its parts as their words", () => {
  const seed = seedOfThisRun();
  const next = seeded(seed);
  const hosts = ["github.com", RELEASES, "status.claude.com", "claude.com", "a.b", "localhost"];
  // Hosts, what may stand beside one or not, and what is added to make a break.
  const pieces = [...hosts, ...hosts, " ", " ", ", ", ". ", ".", "(", ")", "[", "]", '"', "'", ";", "?", "@", "/", "-", "s", "api.", "config.json", "\u200b"];
  const cases = 3000;
  let withHosts = 0;
  for (let index = 0; index < cases; index += 1) {
    const texts = randomTexts(next, pieces, 8);
    const where = `seed ${seed}, case ${index}, ${JSON.stringify(texts)} (QUOTADIAL_TEST_SEED=${seed} runs it again)`;
    const found = hostsIn(texts, hosts);
    const written = hostRuns(texts.join(""), hosts).filter((run) => run.host).map((run) => run.text);
    assert.deepEqual(found.map((host) => host.text), written, `the hosts found are not those the window writes as hosts: ${where}`);
    for (const host of found) {
      assert.equal(between(texts, host.from, host.to), host.text, `a host's start and end give other text: ${where}`);
      assert.deepEqual(host.words.map((word) => word.text), addressParts(host.text), `a host's words are not the parts the window writes: ${where}`);
      for (const word of host.words) {
        assert.equal(between(texts, word.from, word.to), word.text, `a word's start and end give other text: ${where}`);
      }
    }
    withHosts += found.length > 0 ? 1 : 0;
  }
  assert.ok(withHosts >= cases / 10, `only ${withHosts} of ${cases} lines held a host, seed ${seed}`);
});

/** A copy in the shape the host functions read: two webhooks, and Help's privacy text naming hosts in a list and a paragraph. */
const hosted = {
  settings: { alert_targets: ["hooks.slack.com", LONG_HOST] },
  help: {
    hosts: ["api.anthropic.com", "github.com", RELEASES],
    privacy: [
      { kind: "paragraph", text: "QuotaDial connects to these, and nothing else:" },
      { kind: "list", items: ["api.anthropic.com, Anthropic's.", `${RELEASES} and github.com, GitHub's.`] },
      { kind: "paragraph", text: "Updates come from github.com." },
    ],
  },
};

test("the hosts a copy should write: each webhook's in Settings, each place QuotaDial connects to as Help's privacy text names it, in order, and none elsewhere", () => {
  assert.deepEqual(expectedHosts(hosted, "settings"), ["hooks.slack.com", LONG_HOST]);
  assert.deepEqual(expectedHosts(hosted, "help"), ["api.anthropic.com", RELEASES, "github.com", "github.com"]);
  assert.deepEqual(expectedHosts(hosted, "accounts"), []);
  assert.deepEqual(expectedHosts(hosted, "history"), []);
  // With no webhook, Settings has no line naming one.
  assert.deepEqual(expectedHosts({ ...hosted, settings: { alert_targets: [] } }, "settings"), []);
  // Every host the copy can write is looked for, once, in every view.
  assert.deepEqual(knownHosts(hosted), ["hooks.slack.com", LONG_HOST, "api.anthropic.com", "github.com", RELEASES]);
});

test("the lines a copy writes hosts in: Settings' line naming the webhooks', and each line of Help's privacy text, in order", () => {
  assert.deepEqual(linesWithHosts(hosted, "settings"), [
    { what: "Settings' line saying where alerts are also sent", line: `Also sent to hooks.slack.com, ${LONG_HOST}.` },
  ]);
  assert.deepEqual(linesWithHosts(hosted, "help"), [
    { what: "line 1 of Help's text of what QuotaDial reads and sends", line: "QuotaDial connects to these, and nothing else:" },
    { what: "line 2 of Help's text of what QuotaDial reads and sends", line: "api.anthropic.com, Anthropic's." },
    { what: "line 3 of Help's text of what QuotaDial reads and sends", line: `${RELEASES} and github.com, GitHub's.` },
    { what: "line 4 of Help's text of what QuotaDial reads and sends", line: "Updates come from github.com." },
  ]);
  assert.deepEqual(linesWithHosts({ ...hosted, settings: { alert_targets: [] } }, "settings"), []);
  assert.deepEqual(linesWithHosts(hosted, "accounts"), []);
  assert.deepEqual(linesWithHosts(hosted, "history"), []);
});

test("every copy writes hosts in Help, and in Settings where it has a webhook, and the long ones in the ten accounts' copy", () => {
  for (const [name, copy] of Object.entries({ pro, free, many, crowded })) {
    const panels = copy.panels ?? ["accounts", "history", "settings", "help"];
    if (panels.includes("help")) {
      assert.ok(expectedHosts(copy, "help").length > 0, `${name} writes no host in Help, so nothing there is held`);
    }
    assert.equal(expectedHosts(copy, "settings").length, copy.settings.alert_targets.length, `${name}'s Settings does not name each webhook's host`);
    for (const panel of panels) {
      for (const host of expectedHosts(copy, panel)) {
        assert.ok(knownHosts(copy).includes(host), `${name}'s ${JSON.stringify(host)} is not looked for`);
      }
    }
  }
  assert.deepEqual(expectedHosts(many, "settings"), ["hooks.slack.com", LONG_HOST]);
  assert.ok(expectedHosts(many, "help").includes(RELEASES), "the long host is not held in Help");
  assert.deepEqual(expectedHosts(free, "settings"), []);
});

test("the copies' places QuotaDial connects to are the app's own, as crates/core/src/privacy.rs lists them", () => {
  const source = readFileSync(new URL("../../../crates/core/src/privacy.rs", import.meta.url), "utf8");
  const list = source.match(/pub const HOSTS: \[&str; \d+\] = \[([^\]]*)\];/);
  assert.ok(list, "privacy.rs lists no HOSTS");
  assert.deepEqual(DESTINATIONS, [...list[1].matchAll(/"([^"]+)"/g)].map((host) => host[1]));
  for (const copy of [pro, free, many, crowded, site]) {
    assert.deepEqual(copy.help.hosts, DESTINATIONS);
  }
});

/** Whether `line` is `own`, a line of the app's text, or its first sentences. */
const quotes = (line, own) => line === own || (/[.:!?]$/.test(line) && own.startsWith(`${line} `));

/**
 * The blocks of `privacy`, a copy's Help text of what QuotaDial reads and sends, that are not the
 * app's own. A paragraph must be one of `PRIVACY`'s, the app's text as `help.rs` holds it, or its
 * first sentences; a list's items must be its list's, each whole or its first sentences, in their
 * order; and the blocks must come in the app's order, none of the app's shown twice.
 */
function notTheAppsOwn(privacy) {
  const wrong = [];
  let after = -1;
  for (const block of privacy) {
    const lines = block.kind === "paragraph" ? [block.text] : block.items;
    const at = PRIVACY.findIndex((own, index) => {
      if (index <= after || own.kind !== block.kind) return false;
      const owned = own.kind === "paragraph" ? [own.text] : own.items;
      let from = 0;
      return lines.every((line) => {
        const found = owned.findIndex((mine, place) => place >= from && quotes(line, mine));
        from = found + 1;
        return found >= 0;
      });
    });
    if (at < 0) wrong.push(block);
    else after = at;
  }
  return wrong;
}

test("the app's text of what QuotaDial reads and sends is its own whole, or each paragraph's and item's first sentences, in its order", () => {
  const [opening] = PRIVACY;
  const list = PRIVACY.find((block) => block.kind === "list");
  assert.ok(opening.kind === "paragraph" && list?.kind === "list" && list.items.length > 1, "the app's text has no opening or no list");
  const firstSentence = opening.text.slice(0, opening.text.indexOf(". ") + 1);
  assert.deepEqual(notTheAppsOwn(PRIVACY), []);
  assert.deepEqual(notTheAppsOwn([{ kind: "paragraph", text: firstSentence }, { kind: "list", items: [list.items[1]] }]), []);
  const cut = { kind: "paragraph", text: firstSentence.slice(0, -5) };
  const ownWords = { kind: "paragraph", text: "QuotaDial reads every sign-in it finds." };
  const itemAsParagraph = { kind: "paragraph", text: list.items[0] };
  const secondList = { kind: "list", items: [list.items[1]] };
  assert.deepEqual(notTheAppsOwn([cut, ownWords, itemAsParagraph]), [cut, ownWords, itemAsParagraph]);
  assert.deepEqual(notTheAppsOwn([{ kind: "list", items: [list.items[0]] }, secondList]), [secondList], "two lists from its one");
  assert.deepEqual(notTheAppsOwn([list, opening]), [opening], "out of its order");
  assert.deepEqual(notTheAppsOwn([{ kind: "list", items: [list.items[1], list.items[0]] }]), [{ kind: "list", items: [list.items[1], list.items[0]] }]);
});

test("every copy's Help, the website's too, says what QuotaDial reads and sends only in the app's own words", () => {
  const copies = Object.entries({ pro, free, many, crowded, site });
  for (const [name, copy] of copies) {
    assert.ok(copy.help.privacy.length > 0, `${name}'s Help says nothing of what QuotaDial reads and sends`);
  }
  const ofItsOwn = copies
    .map(([name, copy]) => [name, notTheAppsOwn(copy.help.privacy)])
    .filter(([, blocks]) => blocks.length > 0);
  assert.deepEqual(Object.fromEntries(ofItsOwn), {}, "these copies' Help says what QuotaDial reads and sends in words of its own");
});

test("the hosts written are the copy's, in order, or the verdict says which is not, as for an address", () => {
  const host = (text) => ({ text, where: "li", room: 150, words: [] });
  assert.equal(addressVerdict([host("hooks.slack.com")], ["hooks.slack.com"], "host"), false);
  assert.equal(addressVerdict([], [], "host"), false);
  assert.equal(
    addressVerdict([], [RELEASES], "host"),
    `the window's host 1 written whole is no host, where its copy has "${RELEASES}" (0 written whole, 1 wanted)`,
  );
  assert.equal(
    addressVerdict([host("chatgpt.com")], [], "host"),
    `the window's host 1 written whole is "chatgpt.com" (li), where its copy has none (1 written whole, 0 wanted)`,
  );
  // An address's verdict reads as it did.
  assert.match(addressVerdict([], ["dev@example.com"]), /^the window's address 1 written whole is no address, where/);
});

test('a word of a host broken over two lines where it fits is a verdict, as main\'s window drew ".githubuserco" over "ntent.com"', () => {
  const where = `li "${RELEASES}, GitHub's, where an update's files are fe…"`;
  const found = [{ text: RELEASES, where, room: 150, words: [
    { text: "release-assets", boxes: [box(0, 15, 0, 95)] },
    { text: ".githubusercontent", boxes: [box(0, 15, 95, 150), box(15, 30, 0, 66)] },
    { text: ".com", boxes: [] },
  ] }];
  assert.deepEqual(breakVerdicts(found, "host"), [
    false,
    `the host "${RELEASES}" is broken inside ".githubusercontent", over 2 lines, though ".githubusercontent" fits its line (${where})`,
    `".com" of the host "${RELEASES}" has no box to measure (${where})`,
  ]);
  // A word wider than its line has to break: no verdict.
  assert.deepEqual(breakVerdicts([{ ...found[0], room: 100, words: found[0].words.slice(1, 2) }], "host"), [false]);
  // An address's verdicts read as they did.
  assert.match(breakVerdicts([{ ...found[0], text: "x@y" }])[1], /^the address "x@y" is broken inside/);
});

test("a line naming a host is its text and what a copy gives, byte for byte, or the verdict says what was added", () => {
  const line = `Also sent to hooks.slack.com, ${LONG_HOST}.`;
  const what = "Settings' line saying where alerts are also sent";
  assert.deepEqual(writtenVerdicts(line, { text: line, copied: line }, what, "the line"), [false, false]);
  const slipped = line.replace(".slack", "\u200b.slack");
  assert.deepEqual(writtenVerdicts(line, { text: slipped, copied: slipped }, what, "the line"), [
    `${what} is written ${visibly(slipped)}, not the line byte for byte, ${visibly(line)}`,
    `a copy of ${what} gives ${visibly(slipped)}, not the line byte for byte, ${visibly(line)}`,
  ]);
  assert.deepEqual(writtenVerdicts(line, { text: line, copied: `${line}\n` }, what, "the line").map(Boolean), [false, true]);
  assert.deepEqual(writtenVerdicts(line, {}, what, "the line"), [false, false]);
  // An address's verdicts read as they did.
  assert.deepEqual(textVerdicts(LONG, { text: "x", copied: LONG }, "Codex"), [
    `Codex's address is written "x", not the address byte for byte, ${visibly(LONG)}`,
    false,
  ]);
});

test("the code for the page that finds hosts is valid, looks for the hosts it is given and carries what the tests hold", () => {
  const code = HOSTS(DESTINATIONS);
  assert.doesNotThrow(() => new Script(code));
  assert.ok(code.includes(JSON.stringify(DESTINATIONS)));
  for (const piece of [matchesIn, addressWordsIn, hostsIn]) {
    assert.ok(code.includes(String(piece)));
  }
  assert.ok(code.includes(PAGE_TEXT));
});

/** A line of Help's privacy text as the window writes it: each host a node a part, with a <wbr> between. */
function hostItem(item, hosts, width = 150) {
  const runs = hostRuns(item, hosts).flatMap((run) =>
    run.host ? addressParts(run.text).flatMap((part, index) => (index === 0 ? [text(part)] : [tag("wbr", {}), text(part)])) : [text(run.text)],
  );
  return tag("li", { display: "list-item", width }, ...runs);
}

test("in the page, a host cut into nodes is found whole in its line, with each word's boxes and the line's room", () => {
  const item = `${RELEASES}, GitHub's, where an update's files are fetched from, only when you choose Update.`;
  const asked = [];
  const found = inPage(HOSTS(DESTINATIONS), [hostItem(item, DESTINATIONS)], (start, end) => {
    asked.push([start[0].textContent, start[1], end[0].textContent, end[1]]);
    return [box(0, 15, 0, 30)];
  });
  assert.deepEqual(found.map((host) => [host.text, host.where, host.room]), [[RELEASES, `li "${item.slice(0, 59)}…"`, 150]]);
  assert.deepEqual(found[0].words.map((word) => [word.text, word.boxes]), [
    ["release-assets", [box(0, 15, 0, 30)]],
    [".githubusercontent", [box(0, 15, 0, 30)]],
    [".com", [box(0, 15, 0, 30)]],
  ]);
  // Each word's range is its own node, from its start to its end, and the comma after it is in none.
  assert.deepEqual(asked, [["release-assets", 0, "release-assets", 14], [".githubusercontent", 0, ".githubusercontent", 18], [".com", 0, ".com", 4]]);
  assert.equal(addressVerdict(found, [RELEASES], "host"), false);
});

test("in the page, main's host in one node, broken by the browser inside a word that fits the line, is a verdict", () => {
  const item = `${RELEASES}, GitHub's, where an update's files are fetched from.`;
  const line = tag("li", { display: "list-item", width: 150 }, text(item));
  const found = inPage(HOSTS(DESTINATIONS), [line], (start, end) =>
    start[1] === 14 && end[1] === 32 ? [box(0, 15, 95, 150), box(15, 30, 0, 66)] : [box(0, 15, 0, 20)],
  );
  assert.equal(addressVerdict(found, [RELEASES], "host"), false);
  assert.deepEqual(breakVerdicts(found, "host").filter(Boolean), [
    `the host "${RELEASES}" is broken inside ".githubusercontent", over 2 lines, though ".githubusercontent" fits its line (li "${item.slice(0, 59)}…")`,
  ]);
});

test("in the page, a host with a zero-width space slipped in, or one only for a screen reader, is not found", () => {
  const slipped = tag("p", { classes: ["detail"], display: "block", width: 280 }, text("Also sent to hooks\u200b.slack.com."));
  const aside = tag("p", { classes: ["visually-hidden"], display: "block" }, text("Also sent to hooks.slack.com."));
  const found = inPage(HOSTS(["hooks.slack.com"]), [slipped, aside], () => [box(0, 15)]);
  assert.deepEqual(found, []);
  assert.match(addressVerdict(found, ["hooks.slack.com"], "host"), /^the window's host 1 written whole is no host, where its copy has "hooks\.slack\.com"/);
});
