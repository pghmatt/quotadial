// The window check's Chrome launcher, with a script standing in for Chrome: what it gives when
// Chrome answers, exits or keeps quiet, and how long it waits.

import assert from "node:assert/strict";
import { chmodSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { after, test } from "node:test";

import { START_WAIT_MS, launch } from "./chrome.mjs";

// A script runs as a program only where a shebang line is honoured.
const skip = process.platform === "win32" ? "a script stands in for Chrome only where scripts run as programs" : false;

const folders = [];
after(() => {
  for (const folder of folders) {
    rmSync(folder, { recursive: true, force: true });
  }
});

/** A stand-in for Chrome running `body` with Node, and a profile folder for it. */
function standIn(body) {
  const folder = mkdtempSync(join(tmpdir(), "quotadial-chrome-test-"));
  folders.push(folder);
  const program = join(folder, "chrome");
  writeFileSync(program, `#!/usr/bin/env node\n${body}\n`);
  chmodSync(program, 0o755);
  return { program, profile: join(folder, "profile"), folder };
}

/** Stops `browser` and waits until it has gone. */
async function stop(browser) {
  if (browser.exitCode === null && browser.signalCode === null) {
    const gone = new Promise((done) => browser.once("exit", done));
    browser.kill();
    await gone;
  }
}

test("Chrome's DevTools address is what it says when it starts", { skip }, async () => {
  const { program, profile } = standIn(`
    setTimeout(() => process.stderr.write("DevTools listening on ws://127.0.0.1:9/devtools/browser/abc\\n"), 50);
    setInterval(() => {}, 1000);
  `);
  const { browser, address } = launch(program, profile);
  try {
    assert.equal(await address, "ws://127.0.0.1:9/devtools/browser/abc");
  } finally {
    await stop(browser);
  }
});

test("a Chrome that has not answered within the wait fails, saying what it said", { skip }, async () => {
  const { program, profile } = standIn(`
    process.stderr.write("starting slowly");
    setInterval(() => {}, 1000);
  `);
  const { browser, address } = launch(program, profile, { waitMs: 200 });
  try {
    await assert.rejects(address, /^Error: Chrome did not start within 0\.2 s: starting slowly$/);
  } finally {
    await stop(browser);
  }
});

test("a Chrome that exits first fails with its exit status", { skip }, async () => {
  const { program, profile } = standIn(`
    process.stderr.write("no display");
    process.exit(3);
  `);
  const { browser, address } = launch(program, profile);
  try {
    await assert.rejects(address, /^Error: Chrome exited \(3\): no display$/);
  } finally {
    await stop(browser);
  }
});

test("the wait leaves a slow runner a minute, three times the 20 s one runner overran", () => {
  assert.ok(START_WAIT_MS >= 60_000, `Chrome is given ${START_WAIT_MS} ms to start`);
});

test("Chrome runs without its sandbox on Linux and with it everywhere else", { skip }, async () => {
  for (const [platform, unsandboxed] of [
    ["linux", true],
    ["darwin", false],
    ["win32", false],
  ]) {
    const { program, profile, folder } = standIn(`
      require("node:fs").writeFileSync(require("node:path").join(__dirname, "args.json"), JSON.stringify(process.argv.slice(2)));
      process.stderr.write("DevTools listening on ws://127.0.0.1:9/devtools/browser/abc\\n");
      setInterval(() => {}, 1000);
    `);
    const { browser, address } = launch(program, profile, { platform });
    try {
      await address;
      const args = JSON.parse(readFileSync(join(folder, "args.json"), "utf8"));
      assert.equal(args.includes("--no-sandbox"), unsandboxed, platform);
      assert.ok(args.includes("--headless=new") && args.includes(`--user-data-dir=${profile}`), platform);
    } finally {
      await stop(browser);
    }
  }
});
