// The window check's Chrome launcher, with a script standing in for Chrome: what it gives when
// Chrome answers, exits or keeps quiet, and how long it waits.

import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { chmodSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { after, test } from "node:test";

import { START_WAIT_MS, launch, wantsClassicScrollbars } from "./chrome.mjs";

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

/**
 * How long `program` takes, started now, to say anything at all: what a launcher's wait has to
 * outlast before what Chrome said can be in its message. The program is stopped once heard.
 */
async function firstWords(program) {
  const started = performance.now();
  const child = spawn(program, [], { stdio: ["ignore", "ignore", "pipe"] });
  try {
    await new Promise((heard, failed) => {
      child.stderr.once("data", heard);
      child.once("error", failed);
      child.once("exit", (code) => failed(new Error(`the stand-in exited (${code}) before it said anything`)));
    });
    return performance.now() - started;
  } finally {
    // A program that never started has nothing to stop, and no exit to wait for.
    if (child.pid !== undefined) {
      await stop(child);
    }
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

test("a Chrome that has not answered within the wait fails, saying what it said", { skip }, async (t) => {
  const { program, profile } = standIn(`
    process.stderr.write("starting slowly");
    setInterval(() => {}, 1000);
  `);
  // The wait has to outlast the stand-in's start, or the launcher gives up before it has said
  // anything and its message ends at the colon, as a fixed 200 ms did under load (the manager's
  // review of #109). So it scales with this stand-in's own start, measured first. A new program's
  // first start is its slowest, most likely the system checking it before it first runs: on a Mac
  // about 150 ms, and over 300 ms under load, where the same file started again took about 25 ms,
  // and the launch below, its second start, under 75 ms. Five times that first start, since one new
  // stand-in once took 4.35 times another started just before it, and never under 200 ms, which
  // keeps the test quick where starting is quick.
  const spoke = await firstWords(program);
  const waitMs = Math.max(200, Math.ceil(5 * spoke));
  t.diagnostic(`the stand-in first spoke after ${Math.round(spoke)} ms, so the wait is ${waitMs} ms`);
  const { browser, address } = launch(program, profile, { waitMs });
  try {
    await assert.rejects(address, {
      name: "Error",
      message: `Chrome did not start within ${waitMs / 1000} s: starting slowly`,
    });
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

test("Chrome's scroll bars take room unless the pictures ask for the system's own, and scrolling is at once", { skip }, async () => {
  for (const [options, classic] of [
    [{}, true],
    [{ classicScrollbars: false }, false],
    [{ platform: "linux" }, true],
    [{ platform: "linux", classicScrollbars: false }, false],
  ]) {
    const { program, profile, folder } = standIn(`
      require("node:fs").writeFileSync(require("node:path").join(__dirname, "args.json"), JSON.stringify(process.argv.slice(2)));
      process.stderr.write("DevTools listening on ws://127.0.0.1:9/devtools/browser/abc\\n");
      setInterval(() => {}, 1000);
    `);
    const { browser, address } = launch(program, profile, options);
    try {
      await address;
      const args = JSON.parse(readFileSync(join(folder, "args.json"), "utf8"));
      assert.equal(args.some((arg) => arg.startsWith("--disable-features=") && arg.includes("OverlayScrollbar")), classic, JSON.stringify(options));
      assert.ok(args.includes("--disable-smooth-scrolling"), JSON.stringify(options));
      assert.equal(args.at(-1), "about:blank", "the page Chrome opens is still the last argument");
    } finally {
      await stop(browser);
    }
  }
});

test("scroll bars take room for the checks and are the system's own for the pictures, unless --scroll-bars says", () => {
  assert.equal(wantsClassicScrollbars(undefined, false), true, "the checks");
  assert.equal(wantsClassicScrollbars(undefined, true), false, "the pictures");
  assert.equal(wantsClassicScrollbars("classic", true), true, "pictures of the Windows and Linux look");
  assert.equal(wantsClassicScrollbars("system", false), false, "the checks in a Mac's own scroll bars");
  assert.throws(() => wantsClassicScrollbars("thin", false), /^Error: --scroll-bars is "classic" or "system", not "thin"$/);
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
