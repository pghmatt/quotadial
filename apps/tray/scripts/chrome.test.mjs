// The window check's Chrome launcher, with a script standing in for Chrome: what it gives when
// Chrome answers, exits, keeps quiet or will not start, and how long it waits. And the window check
// run as a program with a CHROME that is not there, or one that will not start: it says so in one
// line and leaves no folder behind (the manager's review of #103).

import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import { chmodSync, existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { after, test } from "node:test";
import { fileURLToPath } from "node:url";

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

test("a Chrome that is there but will not start, such as a folder, fails with what stopped it, and has gone at once", { skip }, async () => {
  const folder = mkdtempSync(join(tmpdir(), "quotadial-chrome-test-"));
  folders.push(folder);
  // A folder is there, but it is no program: the system will not start it, and Node says so with an
  // `error` and no `exit`, which nothing heard before, so the check died of an unhandled error.
  const { address, gone } = launch(folder, join(folder, "profile"));
  await assert.rejects(address, (error) => error.message.startsWith(`Chrome at ${folder} would not start: `));
  await gone;
});

const CHECK = fileURLToPath(new URL("./check-window.mjs", import.meta.url));

/**
 * The window check run as a program with `args`, CHROME set to what `chrome(folder)` gives and a
 * temporary folder of its own, `folder`: its exit status, what it wrote to stderr, and what it left
 * in that folder that is its own, a `quotadial-window-check-` profile.
 */
function runCheck(args, chrome) {
  const folder = mkdtempSync(join(tmpdir(), "quotadial-check-temporary-"));
  folders.push(folder);
  const CHROME = chrome(folder);
  const run = spawnSync(process.execPath, [CHECK, ...args], {
    encoding: "utf8",
    // Each system's own name for its temporary folder, so that the check makes its profile here.
    env: { ...process.env, CHROME, TMPDIR: folder, TMP: folder, TEMP: folder },
    timeout: 120_000,
  });
  const left = readdirSync(folder).filter((name) => name.startsWith("quotadial-window-check-"));
  return { CHROME, status: run.status, stderr: run.stderr, left };
}

test("with a zoom that is one, the window check says a CHROME that is not there was not found, in one line, and leaves no folder", () => {
  const { CHROME, status, stderr, left } = runCheck(["--window-zoom", "2"], (folder) => join(folder, "no-chrome-here"));
  assert.equal(status, 1, stderr);
  assert.equal(
    stderr.trim(),
    `Chrome was not found at ${CHROME}, where CHROME says it is; set CHROME to Chrome's path, or leave it unset to look where Chrome usually is.`,
  );
  assert.deepEqual(left, [], "the check left its profile folder behind");
});

// The check gets as far as starting Chrome only with a built window to serve.
const built = existsSync(fileURLToPath(new URL("../dist/index.html", import.meta.url)));

test(
  "the window check says a Chrome that will not start, in one line, and removes the profile it made for it",
  { skip: skip || (!built && "the window is not built, so the check stops before it starts Chrome (npm run build)") },
  () => {
    const { CHROME, status, stderr, left } = runCheck([], (folder) => {
      const chrome = join(folder, "a-folder-not-a-program");
      mkdirSync(chrome);
      return chrome;
    });
    assert.equal(status, 1, stderr);
    const said = stderr.trim();
    assert.ok(said.startsWith(`Chrome at ${CHROME} would not start: `), said);
    assert.ok(!said.includes("\n"), `the check said more than one line: ${said}`);
    assert.deepEqual(left, [], "the check left its profile folder behind");
  },
);

