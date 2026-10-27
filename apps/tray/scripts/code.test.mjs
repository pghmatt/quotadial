// The commands in what QuotaDial says (src/code.ts, read straight from the window's source, as
// check-window.mjs reads share.ts): each one found whole, with nothing said lost around it, for the
// cases below and for random strings of backticks, spaces, brackets and punctuation (CLAUDE.md:
// every parser of text from outside has a property test, and a provider's error can carry words
// from outside to the window).
import assert from "node:assert/strict";
import { test } from "node:test";

import { codeSpans } from "../src/code.ts";

/** The message as the window writes it: each command between backticks again. */
function written(spans) {
  return spans.map((span) => (span.code ? `\`${span.text}\`` : span.text)).join("");
}

/** The commands the window shows as code. */
function commands(text) {
  return codeSpans(text)
    .filter((span) => span.code)
    .map((span) => span.text);
}

test("Claude's sign-in command shows whole, in a POSIX shell and in PowerShell (D-131)", () => {
  const posix = "Run `CLAUDE_CONFIG_DIR=~/'My Claude' claude` to sign in again.";
  assert.deepEqual(codeSpans(posix), [
    { text: "Run ", code: false },
    { text: "CLAUDE_CONFIG_DIR=~/'My Claude' claude", code: true },
    { text: " to sign in again.", code: false },
  ]);
  // PowerShell escapes a backtick, a $ and a double quote with a backtick, so a folder called
  // it's "$HOME" `Claude` puts backticks inside the command.
  const powershell =
    'This Claude Code sign-in expired on 26 Oct 2026 at 09:12 UTC. Run `$env:CLAUDE_CONFIG_DIR = "$HOME\\it\'s `"`$HOME`" ``Claude``"; claude` once to refresh it.';
  assert.deepEqual(commands(powershell), [
    '$env:CLAUDE_CONFIG_DIR = "$HOME\\it\'s `"`$HOME`" ``Claude``"; claude',
  ]);
  assert.equal(written(codeSpans(powershell)), powershell);
  // In a POSIX shell a backtick in a folder's name sits inside single quotes.
  assert.deepEqual(commands("Run `CLAUDE_CONFIG_DIR=~/'it'\\''s `x`' claude` to sign in again."), [
    "CLAUDE_CONFIG_DIR=~/'it'\\''s `x`' claude",
  ]);
});

test("every provider's commands show as code: a slash command, a settings block and a chmod", () => {
  assert.deepEqual(commands("In Gemini CLI, `/model` shows each model's share used and when it resets."), [
    "/model",
  ]);
  assert.deepEqual(
    commands(
      'once `"statusLine": { "type": "command", "command": "quotadial antigravity status-line" }` is in ~/.gemini/antigravity-cli/settings.json.',
    ),
    ['"statusLine": { "type": "command", "command": "quotadial antigravity status-line" }'],
  );
  assert.deepEqual(
    commands("Other users can read it; run `chmod 600 ~/'Library/Application Support/quotadial/config.json'`."),
    ["chmod 600 ~/'Library/Application Support/quotadial/config.json'"],
  );
  assert.deepEqual(commands("`claude` with the CLAUDE_CONFIG_DIR that made the Keychain item (run `claude`)"), [
    "claude",
    "claude",
  ]);
});

test("a backtick that opens no command stays in the words, and nothing said is lost", () => {
  for (const text of [
    "No command here.",
    "A lone ` backtick.",
    "An unclosed `command at the end",
    "A word`with a backtick inside`.",
    "Empty `` backticks.",
    "` starts with a space`.",
    "`",
    "",
  ]) {
    assert.equal(written(codeSpans(text)), text, text);
  }
  assert.deepEqual(commands("A lone ` backtick and an unclosed `one"), []);
  assert.deepEqual(codeSpans(""), []);
  assert.deepEqual(codeSpans("`claude`"), [{ text: "claude", code: true }]);
});

/**
 * A seeded generator of numbers from 0 up to 1 (mulberry32), enough for choosing characters: the
 * same seed makes the same strings, so a failing input is found again from its seed.
 */
function seeded(seed) {
  let state = seed >>> 0;
  return () => {
    state = (state + 0x6d2b79f5) >>> 0;
    let mixed = Math.imul(state ^ (state >>> 15), state | 1);
    mixed ^= mixed + Math.imul(mixed ^ (mixed >>> 7), mixed | 61);
    return ((mixed ^ (mixed >>> 14)) >>> 0) / 2 ** 32;
  };
}

/**
 * What decides where a command opens and closes, and what a message holds around one: backticks
 * above all, then spaces, the brackets that may open a command and the punctuation that may end a
 * phrase after it, quotes, a dollar sign and a backslash as a shell command has them, a few
 * letters, a line break and a tab, a no-break space, an accented letter and a character that takes
 * two UTF-16 units.
 */
const PIECES = [
  ...["`", "`", "`", "`", " ", " ", " ", "(", ")", "[", "]", "{", "}", ".", ",", ";", ":", "!", "?"],
  ...["'", '"', "$", "\\", "a", "b", "z", "\n", "\t", " ", "é", "\u{1f600}"],
];

test("whatever it is given, the spans joined give back the text, so nothing said is ever lost", () => {
  // A new seed each run, so over time the strings are many; the seed is in every failure, and
  // QUOTADIAL_TEST_SEED=<seed> runs the same strings again.
  const given = process.env.QUOTADIAL_TEST_SEED;
  const seed = given === undefined ? Math.floor(Math.random() * 2 ** 32) : Number(given);
  assert.ok(
    Number.isInteger(seed) && seed >= 0 && seed < 2 ** 32,
    `QUOTADIAL_TEST_SEED is "${given}", not a whole number from 0 to 2 ** 32 - 1`,
  );
  const next = seeded(seed);
  const cases = 5000;
  let withCommands = 0;
  for (let index = 0; index < cases; index += 1) {
    // Half of the strings are short, so a command only a few characters long is common.
    const length = Math.floor(next() * (next() < 0.5 ? 12 : 60));
    let text = "";
    for (let at = 0; at < length; at += 1) {
      text += PIECES[Math.floor(next() * PIECES.length)];
    }
    const where = `seed ${seed}, case ${index}, ${JSON.stringify(text)} (QUOTADIAL_TEST_SEED=${seed} runs it again)`;
    let spans;
    try {
      spans = codeSpans(text);
    } catch (error) {
      assert.fail(`codeSpans threw ${error} for ${where}`);
    }
    assert.equal(written(spans), text, `the spans joined are not the text: ${where}`);
    withCommands += spans.some((span) => span.code) ? 1 : 0;
  }
  // The strings reach the code that finds a command, not only the words around it: about 18% hold one.
  assert.ok(withCommands >= cases / 20, `only ${withCommands} of ${cases} strings held a command, seed ${seed}`);
});
