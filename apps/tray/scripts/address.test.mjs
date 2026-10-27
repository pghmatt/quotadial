// Where an address may break (src/address.ts, read straight from the window's source, as
// code.test.mjs reads code.ts): before its @ and before each dot, never at its start, so that
// "a.very.long.account.name.that.keeps.going@engineering.example.com" can go over two lines as its
// name and its domain, and the window writes a <wbr> at each of those places and never a character,
// so the parts joined are the address, byte for byte (the manager's decision on #73, 2026-10-27,
// 13:26 UTC, D-141). And where an address stands in a line the app writes, such as the summary's,
// which names the account by its address when it has no name given. Both are held for the cases
// below and for random strings (CLAUDE.md: every parser of text from outside has a property test,
// and an account's address comes from a provider's answer or a file a CLI keeps).
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import { addressParts, addressRuns } from "../src/address.ts";
import { seeded, seedOfThisRun } from "./random-text.mjs";

const LONG = "a.very.long.account.name.that.keeps.going@engineering.example.com";

test("an address breaks before its @ and before each dot, and nowhere else", () => {
  assert.deepEqual(addressParts(LONG), [
    "a",
    ".very",
    ".long",
    ".account",
    ".name",
    ".that",
    ".keeps",
    ".going",
    "@engineering",
    ".example",
    ".com",
  ]);
  assert.deepEqual(addressParts("dev@example.com"), ["dev", "@example", ".com"]);
  assert.deepEqual(addressParts("first.last+tag@mail.example.co.uk"), ["first", ".last+tag", "@mail", ".example", ".co", ".uk"]);
});

test("never at its start: an address that begins with a dot or an @ keeps it on its first part", () => {
  assert.deepEqual(addressParts(".credentials.json"), [".credentials", ".json"]);
  assert.deepEqual(addressParts("@example.com"), ["@example", ".com"]);
  assert.deepEqual(addressParts("@"), ["@"]);
  assert.deepEqual(addressParts("."), ["."]);
});

test("an address with no dot breaks only before its @, and one with no @ only before its dots", () => {
  assert.deepEqual(addressParts("root@localhost"), ["root", "@localhost"]);
  assert.deepEqual(addressParts("first.last"), ["first", ".last"]);
  // A Claude sign-in whose address is not known is named for its file, its Keychain item or its
  // variable, and a Gemini CLI sign-in for its kind: the same rule, which leaves a name with
  // neither as it is.
  assert.deepEqual(addressParts("/Users/someone/.claude-work/.credentials.json"), [
    "/Users/someone/",
    ".claude-work/",
    ".credentials",
    ".json",
  ]);
  for (const name of ["CLAUDE_CODE_OAUTH_TOKEN", "Claude Code-credentials-1a2b3c4d", "Google account", "x"]) {
    assert.deepEqual(addressParts(name), [name]);
  }
});

test("a long part with no dot is one part, which only the stylesheet's last resort breaks, where it is wider than the line", () => {
  // The longest a name before the @ may be, 64 characters, and a domain's label, 63.
  const name = "continuousintegrationsandboxfortheengineeringplatformteams2026ci";
  assert.equal(name.length, 64);
  assert.deepEqual(addressParts(`${name}@example.com`), [name, "@example", ".com"]);
  const label = "y".repeat(63);
  assert.deepEqual(addressParts(`someone@${label}`), ["someone", `@${label}`]);
});

test("two dots in a row, a dot before the @ and an @ at the end each start a part of their own", () => {
  assert.deepEqual(addressParts("a..b"), ["a", ".", ".b"]);
  assert.deepEqual(addressParts("a.@b"), ["a", ".", "@b"]);
  assert.deepEqual(addressParts("a@"), ["a", "@"]);
  assert.deepEqual(addressParts("a@@b"), ["a", "@", "@b"]);
});

test("an empty address has no parts", () => {
  assert.deepEqual(addressParts(""), []);
});

/**
 * What an address is made of, and what can come with one from outside: letters, figures, the
 * characters a name before the @ may hold, dots and @ above all, a space, a slash as a folder has
 * one, an accented letter, a character that takes two UTF-16 units, and the invisible characters a
 * break must never be made of.
 */
const PIECES = [
  ...["@", "@", ".", ".", ".", "a", "b", "z", "0", "9", "-", "_", "+", "'", " ", "/"],
  ...["é", "日", "\u{1f600}", "\u200b", "\u00ad", "\u2060"],
];

/** A string of up to `longest` of `PIECES`, chosen by `next`. */
function randomAddress(next, longest) {
  let text = "";
  for (let at = Math.floor(next() * longest); at > 0; at -= 1) {
    text += PIECES[Math.floor(next() * PIECES.length)];
  }
  return text;
}

test("whatever it is given, the parts joined are the address, byte for byte, each after the first starting at its @ or dot", () => {
  // A new seed each run; the seed is in every failure, and QUOTADIAL_TEST_SEED=<seed> runs the same strings again.
  const seed = seedOfThisRun();
  const next = seeded(seed);
  const cases = 5000;
  let broken = 0;
  for (let index = 0; index < cases; index += 1) {
    const address = randomAddress(next, next() < 0.5 ? 12 : 70);
    const where = `seed ${seed}, case ${index}, ${JSON.stringify(address)} (QUOTADIAL_TEST_SEED=${seed} runs it again)`;
    let parts;
    try {
      parts = addressParts(address);
    } catch (error) {
      assert.fail(`addressParts threw ${error} for ${where}`);
    }
    assert.equal(parts.join(""), address, `the parts joined are not the address: ${where}`);
    assert.ok(parts.every((part) => part.length > 0), `a part is empty: ${where}`);
    // A part ends just before each @ and dot after the first character, and nowhere else.
    const places = [...address.slice(1).matchAll(/[@.]/g)].length;
    assert.equal(parts.length, address ? places + 1 : 0, `the parts are not cut at each @ and dot: ${where}`);
    for (const [at, part] of parts.entries()) {
      assert.ok(at === 0 || /^[@.]/.test(part), `part ${at + 1} does not start at an @ or a dot: ${where}`);
      assert.ok(!/[@.]/.test(part.slice(1)), `part ${at + 1} holds an @ or a dot after its first character: ${where}`);
    }
    broken += parts.length > 1 ? 1 : 0;
  }
  assert.ok(broken >= cases / 4, `only ${broken} of ${cases} strings had a place to break, seed ${seed}`);
});

test("the window writes each break as a <wbr> hidden from screen readers, and never a character", () => {
  // The window check holds what this comes to in Chrome: the address's text and what a copy gives
  // are the address, and every name a screen reader is told holds it whole, where a <wbr> it can see
  // makes Chrome say "dev @example .com" (D-141). This holds the source to it without a browser.
  const source = readFileSync(new URL("../src/account.tsx", import.meta.url), "utf8");
  assert.match(source, /\{index > 0 && <wbr aria-hidden="true" \/>\}\s*\{part\}/);
  // One element, written once, in Address: a <wbr> in a comment is not one.
  assert.equal([...source.matchAll(/<wbr\b[^>]*\/>/g)].length, 1, "a <wbr> written anywhere but in Address");
  for (const character of ["\u200b", "\u00ad", "\u2060", "\ufeff"]) {
    assert.ok(!source.includes(character), `account.tsx holds ${JSON.stringify(character)}`);
  }
  assert.doesNotMatch(source, /\\u(?:200b|00ad|2060|feff)|\\u\{(?:200b|ad|2060|feff)\}|&(?:shy|#8203|#x200b);/i);
});

const HEADLINE = "Weekly · Claude max@example.com · resets in 2 d 6 h";

test("the summary's line names an account by its address when it has no name given, and the address is a run of its own", () => {
  assert.deepEqual(addressRuns(HEADLINE, ["max@example.com"]), [
    { text: "Weekly · Claude ", address: false },
    { text: "max@example.com", address: true },
    { text: " · resets in 2 d 6 h", address: false },
  ]);
  // The window looks for every account's address; only the one the line names is in it.
  assert.deepEqual(
    addressRuns(`Weekly · Codex ${LONG} · resets in 3 h 41 min`, ["dev@example.com", LONG, "", "Google account"]),
    [
      { text: "Weekly · Codex ", address: false },
      { text: LONG, address: true },
      { text: " · resets in 3 h 41 min", address: false },
    ],
  );
});

test("a line that names no address, as one naming an account by the name given it does, is one run of words", () => {
  const named = "Weekly · Claude Work · resets in 2 d 4 h";
  assert.deepEqual(addressRuns(named, ["dev@example.com", "max@example.com"]), [{ text: named, address: false }]);
  assert.deepEqual(addressRuns(named, []), [{ text: named, address: false }]);
  // An empty address is no address, whatever the line holds.
  assert.deepEqual(addressRuns(named, [""]), [{ text: named, address: false }]);
  assert.deepEqual(addressRuns("", ["dev@example.com"]), []);
});

test("an address stands with a space or the line's start or end on either side, so one inside a longer word is not it", () => {
  // "ops@example.com" is the end of "devops@example.com", and "dev@example.co" the start of "dev@example.com".
  assert.deepEqual(addressRuns("Weekly · Codex devops@example.com · resets", ["ops@example.com", "dev@example.co"]), [
    { text: "Weekly · Codex devops@example.com · resets", address: false },
  ]);
  assert.deepEqual(addressRuns("max@example.com", ["max@example.com"]), [{ text: "max@example.com", address: true }]);
  assert.deepEqual(addressRuns("a@b.c a@b.c", ["a@b.c"]), [
    { text: "a@b.c", address: true },
    { text: " ", address: false },
    { text: "a@b.c", address: true },
  ]);
});

test("where two addresses begin at one place, the longer is the one that stands there", () => {
  assert.deepEqual(addressRuns("to dev@example.com.au now", ["dev@example.com", "dev@example.com.au"]), [
    { text: "to ", address: false },
    { text: "dev@example.com.au", address: true },
    { text: " now", address: false },
  ]);
});

test("whatever the line and the addresses, the runs joined are the line, and each address run is an address standing in it", () => {
  const seed = seedOfThisRun();
  const next = seeded(seed);
  const cases = 3000;
  let withAddresses = 0;
  for (let index = 0; index < cases; index += 1) {
    const addresses = Array.from({ length: Math.floor(next() * 4) }, () => randomAddress(next, 8));
    // A line of addresses and words, mostly with a space between, so that an address often stands
    // in it, and sometimes run together, so that one is often inside a longer word.
    const pieces = Array.from({ length: Math.floor(next() * 6) }, () =>
      next() < 0.5 && addresses.length > 0 ? addresses[Math.floor(next() * addresses.length)] : randomAddress(next, 6),
    );
    const line = pieces.join(next() < 0.7 ? " " : "");
    const where = `seed ${seed}, case ${index}, ${JSON.stringify(line)} with ${JSON.stringify(addresses)} (QUOTADIAL_TEST_SEED=${seed} runs it again)`;
    let runs;
    try {
      runs = addressRuns(line, addresses);
    } catch (error) {
      assert.fail(`addressRuns threw ${error} for ${where}`);
    }
    assert.equal(runs.map((run) => run.text).join(""), line, `the runs joined are not the line: ${where}`);
    assert.ok(runs.every((run) => run.text.length > 0), `a run is empty: ${where}`);
    let at = 0;
    for (const [number, run] of runs.entries()) {
      if (run.address) {
        assert.ok(addresses.includes(run.text), `run ${number + 1} is not one of the addresses: ${where}`);
        assert.ok(at === 0 || /\s/.test(line[at - 1]), `run ${number + 1} has no space or start before it: ${where}`);
        const end = at + run.text.length;
        assert.ok(end === line.length || /\s/.test(line[end]), `run ${number + 1} has no space or end after it: ${where}`);
      } else {
        assert.ok(number === 0 || runs[number - 1].address, `two runs of words are side by side: ${where}`);
      }
      at += run.text.length;
    }
    withAddresses += runs.some((run) => run.address) ? 1 : 0;
  }
  assert.ok(withAddresses >= cases / 10, `only ${withAddresses} of ${cases} lines held an address, seed ${seed}`);
});
