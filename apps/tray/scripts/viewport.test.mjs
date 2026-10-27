// The window's size at a zoom (viewport.mjs), which the window check asks Chrome for twice: to judge
// the window at a zoom, and to take its pictures at one. Chrome takes the size in whole pixels, and
// `--window-pictures` at `--window-zoom 3` failed with "Invalid parameters" for want of it (the
// manager's review of #98). So the size is a whole number of pixels at every zoom, it is the same
// size at 100% and 200% as both asked for before, so that the pictures made at those stay as they
// were, and the check asks for no size of its own.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { viewportAt, zoomOf } from "./viewport.mjs";

// The popover as the check shows it at 100%: its width from the config the app is built with, as
// check-window.mjs reads it, and the tallest it is (its HEIGHT, which src-tauri/src/lib.rs clamps to).
const CONFIG = JSON.parse(readFileSync(new URL("../src-tauri/tauri.conf.json", import.meta.url), "utf8"));
const POPOVER = { width: CONFIG.app.windows[0].width, height: 640 };

/** The zoom as the check names it in its messages. */
function percent(zoom) {
  return `${Math.round(zoom * 100)}%`;
}

/** What the check's pages (`View.zoom`) asked Chrome for before: the popover over the zoom, to the nearest pixel. */
function pagesAskedFor(zoom) {
  return { width: Math.round(POPOVER.width / zoom), height: Math.round(POPOVER.height / zoom) };
}

/** What its pictures (`windowPictures`) asked for before: the popover over the zoom, as it came. */
function picturesAskedFor(zoom) {
  return { width: POPOVER.width / zoom, height: POPOVER.height / zoom };
}

test("the window is a whole number of pixels at every zoom, 300% and 150% among them", () => {
  // 1, 2, 3 and 1.5 are the ask; the rest are the zooms the check judges the window at, the last of
  // them the one at which the popover is 300 pixels tall.
  const problems = [];
  for (const zoom of [1, 2, 3, 1.5, 1.25, 2.1, POPOVER.height / 300, 2.2, 2.5, 5]) {
    const { width, height } = viewportAt(zoom, POPOVER);
    if (!Number.isInteger(width) || !Number.isInteger(height)) {
      problems.push(`at ${percent(zoom)} the window is ${width} by ${height} pixels, not a whole number of them`);
    }
    // The nearest whole pixel, and not the one below or above whichever way a fraction falls.
    if (Math.abs(width - POPOVER.width / zoom) > 0.5 || Math.abs(height - POPOVER.height / zoom) > 0.5) {
      problems.push(`at ${percent(zoom)} ${width} by ${height} pixels is not the nearest to ${POPOVER.width / zoom} by ${POPOVER.height / zoom}`);
    }
  }
  assert.deepEqual(problems, []);
});

test("at 100% and 200% it is the size both the pages and the pictures asked for before", () => {
  const problems = [];
  for (const zoom of [1, 2]) {
    for (const [who, before] of [["pages", pagesAskedFor(zoom)], ["pictures", picturesAskedFor(zoom)]]) {
      const now = viewportAt(zoom, POPOVER);
      if (now.width !== before.width || now.height !== before.height) {
        problems.push(`at ${percent(zoom)} the ${who} asked for ${before.width} by ${before.height} and now get ${now.width} by ${now.height}`);
      }
    }
  }
  assert.deepEqual(problems, []);
  // In so many words: the popover itself, then half of it.
  assert.deepEqual(viewportAt(1, POPOVER), POPOVER);
  assert.deepEqual(viewportAt(2, POPOVER), { width: POPOVER.width / 2, height: POPOVER.height / 2 });
});

test("at 300% and 150% it is the size the pages were always shown at, where the pictures asked for part of a pixel", () => {
  const problems = [];
  for (const zoom of [3, 1.5]) {
    // These are zooms the pictures failed at, so that this cannot pass for want of one that does.
    const asked = picturesAskedFor(zoom);
    if (Number.isInteger(asked.width) && Number.isInteger(asked.height)) {
      problems.push(`at ${percent(zoom)} the pictures asked for ${asked.width} by ${asked.height}, which Chrome takes`);
    }
    const now = viewportAt(zoom, POPOVER);
    const pages = pagesAskedFor(zoom);
    if (now.width !== pages.width || now.height !== pages.height) {
      problems.push(`at ${percent(zoom)} the window is ${now.width} by ${now.height} pixels, and the pages are shown at ${pages.width} by ${pages.height}`);
    }
  }
  assert.deepEqual(problems, []);
  // 640 / 3 is 213.33 and 640 / 1.5 is 426.67: down at one, up at the other.
  assert.equal(viewportAt(3, POPOVER).height, 213);
  assert.equal(viewportAt(1.5, POPOVER).height, 427);
});

test("the check asks for no size of its own: the popover over a zoom is only ever worked out in viewport.mjs", () => {
  // The pictures are not made in CI, so nothing else would notice a second rule for the size.
  const source = readFileSync(new URL("./check-window.mjs", import.meta.url), "utf8");
  const second = source
    .split("\n")
    .map((line, index) => ({ line: line.trim(), number: index + 1 }))
    .filter(({ line }) => /\b(?:WIDTH|HEIGHT)\s*\/\s*zoom\b/.test(line));
  assert.deepEqual(second, [], "ask viewportAt for the size at a zoom, so that every place gets the same whole pixels");
});

/** What the window check says of a `--window-zoom` that is not a zoom, in the form `--scroll-bars` refuses a value in. */
const notAZoom = (flag) => `--window-zoom is a number above 0, such as 2 for 200%, not "${flag}"`;

test("--window-zoom refuses what is not a zoom: \"abc\", which reached Chrome as NaN, and 0, which is no zoom", () => {
  // The manager's review of #100: `--window-zoom abc` failed in Chrome with "Invalid parameters".
  for (const flag of ["abc", "0", "", " ", "-2", "NaN", "Infinity", "2x", "1/2"]) {
    assert.throws(() => zoomOf(flag), { message: notAZoom(flag) }, JSON.stringify(flag));
  }
  // A zoom is the number it says, and with no flag the window is at 100%.
  for (const [flag, zoom] of [["2", 2], ["3", 3], ["1.5", 1.5], ["0.5", 0.5], ["5", 5]]) {
    assert.equal(zoomOf(flag), zoom, flag);
  }
  assert.equal(zoomOf(undefined), 1);
});

test("the window check refuses a zoom that is not one, abc, 0 or a negative, before it makes a folder or asks for Chrome", () => {
  const check = fileURLToPath(new URL("./check-window.mjs", import.meta.url));
  // A folder for the pictures that must never be made, and a Chrome that is not there, which the
  // check would say it could not find, or fail to start, had it got that far.
  const folder = join(tmpdir(), `quotadial-window-zoom-refused-${process.pid}`);
  assert.ok(!existsSync(folder));
  for (const [args, flag] of [
    [["--window-zoom", "abc"], "abc"],
    [["--window-zoom", "0"], "0"],
    [["--window-zoom", "-2"], "-2"],
    // The flag with nothing after it is a zoom of nothing, as `--scroll-bars` with none is.
    [["--window-zoom"], ""],
  ]) {
    const run = spawnSync(process.execPath, [check, "--window-pictures", folder, ...args], {
      encoding: "utf8",
      env: { ...process.env, CHROME: join(tmpdir(), "quotadial-no-chrome-here") },
      timeout: 60_000,
    });
    assert.equal(run.status, 1, `${args.join(" ")}: ${run.stderr}`);
    assert.equal(run.stderr.trim(), notAZoom(flag), args.join(" "));
    assert.ok(!existsSync(folder), `${args.join(" ")} made the pictures' folder`);
  }
});
