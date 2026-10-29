// The window check's Chrome launcher, with a script standing in for Chrome: what it gives when
// Chrome answers, exits, keeps quiet or will not start, how long it waits, and that a run ends once
// Chrome has gone, though a crash handler Chrome started still holds its stderr. And the window check
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

/**
 * How long a run may go on after its last line. One with nothing left to do ends at once, 1 to 4 ms
 * after it in ten runs on a Mac with a load average of 13 to 16; 10 s leaves a loaded runner room,
 * and fails long before the crash handler below would have gone and let a run that waits for it end.
 */
const ENDS_WITHIN_MS = 10_000;

/**
 * How long the stand-in's crash handler lives unless the test stops it first: well past a run's
 * time limit, and short enough that a test stopped before its own clean-up leaves nothing running
 * for long.
 */
const HANDLER_LIVES_MS = 120_000;

/**
 * The end of a window check's run, as a program of its own: it starts the program it is given for
 * Chrome with this launcher, stops it once it has its address, as the check does when it is done,
 * waits until it has gone and says so in its last line, after which it has nothing left to do.
 */
const RUN = `
  import { launch } from ${JSON.stringify(new URL("./chrome.mjs", import.meta.url).href)};
  const [program, profile] = process.argv.slice(1);
  const { browser, address, gone } = launch(program, profile);
  await address;
  browser.kill();
  await gone;
  console.log("Chrome has gone");
`;

/** Whether `promise` is kept within `ms`, waiting no longer than it takes. */
async function within(promise, ms) {
  let timer;
  const late = new Promise((done) => {
    timer = setTimeout(() => done(false), ms);
  });
  try {
    return await Promise.race([promise.then(() => true), late]);
  } finally {
    clearTimeout(timer);
  }
}

/** Whether the process `pid` is still running. */
function running(pid) {
  try {
    process.kill(pid, 0);
    return true;
  } catch (error) {
    if (error.code === "ESRCH") {
      return false;
    }
    throw error;
  }
}

test("a run ends once Chrome has gone, though a crash handler Chrome started still holds its stderr", { skip }, async (t) => {
  // Chrome's crash handler, and those of Google's updater, inherit Chrome's stderr and outlive it, so
  // the window check sat after its last line until they had gone, for 3 to 17 minutes (#115, and
  // twice on #116). The stand-in starts such a handler, which keeps the pipe open after the stand-in
  // has exited, and writes down its PID, since nothing else will stop it.
  const { program, profile, folder } = standIn(`
    const handler = require("node:child_process").spawn(process.execPath, ["-e", "setTimeout(() => {}, ${HANDLER_LIVES_MS})"], {
      stdio: ["ignore", "ignore", "inherit"],
    });
    require("node:fs").writeFileSync(require("node:path").join(__dirname, "handler.pid"), String(handler.pid));
    process.stderr.write("DevTools listening on ws://127.0.0.1:9/devtools/browser/abc\\n");
    setInterval(() => {}, 1000);
  `);
  const run = spawn(process.execPath, ["--input-type=module", "-e", RUN, program, profile], {
    stdio: ["ignore", "pipe", "pipe"],
  });
  // Kept once the run has exited and everything it said has been read.
  const ended = new Promise((done) => run.once("close", done));
  let said = "";
  const lastLine = new Promise((done) => {
    for (const stream of [run.stdout, run.stderr]) {
      stream.on("data", (chunk) => {
        said += chunk;
        if (said.includes("Chrome has gone\n")) {
          done();
        }
      });
    }
  });
  const pidFile = join(folder, "handler.pid");
  try {
    await Promise.race([lastLine, ended]);
    assert.ok(said.includes("Chrome has gone\n"), `the run ended before Chrome had gone: ${said}`);
    const heard = performance.now();
    assert.ok(
      await within(ended, ENDS_WITHIN_MS),
      `the run had not ended ${ENDS_WITHIN_MS / 1000} s after its last line, waiting on the stderr the stand-in's crash handler still holds`,
    );
    t.diagnostic(`the run ended ${Math.round(performance.now() - heard)} ms after its last line`);
    assert.equal(run.exitCode, 0, said);
    // Held only while the handler is there to hold the pipe: one that had gone would let any run end.
    assert.ok(running(Number(readFileSync(pidFile, "utf8"))), "the stand-in's crash handler had gone before the run ended");
  } finally {
    await stop(run);
    // The handler outlives the stand-in, as Chrome's do, so the test stops it, by its own PID and
    // only by one: a PID of 0 would stop every process in the test's own group.
    const pid = existsSync(pidFile) ? Number(readFileSync(pidFile, "utf8")) : NaN;
    if (Number.isInteger(pid) && pid > 0 && running(pid)) {
      process.kill(pid);
    }
  }
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

