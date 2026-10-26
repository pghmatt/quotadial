// The commands in what QuotaDial says (src/code.ts, read straight from the window's source, as
// check-window.mjs reads share.ts): each one found whole, with nothing said lost around it.
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
    'This Claude Code sign-in expired at 2026-10-26T09:12:00Z. Run `$env:CLAUDE_CONFIG_DIR = "$HOME\\it\'s `"`$HOME`" ``Claude``"; claude` once to refresh it.';
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
