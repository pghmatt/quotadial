// How the window check knows a scroll is over: that it waits out a scroll that starts late and
// stalls between frames, which reading the position twice 50 ms apart, or waiting for a quiet
// moment, does not.

import assert from "node:assert/strict";
import { test } from "node:test";

import { waitForScrollEnd } from "./quiet.mjs";

/**
 * A page whose clock moves only as the wait sleeps, which scrolls at each time in `scrolls` and
 * says a scroll has ended at each time in `ends`, all in ms from the start.
 */
function page(scrolls, ends) {
  let now = 1000;
  const last = (times) => Math.max(0, ...times.filter((at) => at <= now));
  return {
    now: () => now,
    look: async () => ({ now, lastScroll: last(scrolls), lastEnd: last(ends) }),
    sleep: async (ms) => {
      now += ms;
    },
  };
}

/** What the check once did: read the position every 50 ms until two reads agree, and say when. */
function whenTwoReadsAgree(scrolls, from) {
  const position = (at) => scrolls.filter((scrolled) => scrolled <= at).length;
  for (let at = from + 50; ; at += 50) {
    if (position(at) === position(at - 50)) {
      return at;
    }
  }
}

// The key goes down at 1000. The glide starts 120 ms late, draws a few frames, stalls for 300 ms
// (a slow machine drawing nothing), draws the rest and ends at 1700.
const GLIDE = [1120, 1160, 1200, 1240, 1540, 1580, 1620, 1660];

test("a scroll that starts late and stalls between frames is waited out to its end", async () => {
  // Reading the position until two reads agree stops before the glide has begun.
  const early = whenTwoReadsAgree(GLIDE, 1000);
  assert.ok(early < 1120, `the old reading stopped at ${early}, before the glide had begun at 1120`);

  const { look, sleep, now } = page(GLIDE, [1700]);
  await waitForScrollEnd(look, { sleep });
  assert.ok(now() >= 1700, `it returned at ${now()}, before the scroll ended at 1700`);
  assert.ok(now() < 1900, `and it did not wait on after it: ${now()}`);
});

test("a stall longer than any quiet moment does not end the wait", async () => {
  // 600 ms between frames, which a wait for a quiet 250 or 400 ms would take for the end.
  const stalled = [1100, 1140, 1180, 1780, 1820, 1860];
  const { look, sleep, now } = page(stalled, [1900]);
  await waitForScrollEnd(look, { sleep });
  assert.ok(now() >= 1900, `it returned at ${now()}, in the stall, before the scroll ended at 1900`);
});

test("it waits as long as a scroll takes to begin when nothing scrolls, and no longer", async () => {
  const { look, sleep, now } = page([], []);
  const waited = await waitForScrollEnd(look, { sleep });
  assert.ok(waited >= 400, `it waited ${waited} ms, under the 400 ms a slow start can take`);
  assert.ok(now() - 1000 < 500, "and it did not wait much longer");
});

test("a scroll that was over before the wait began does not count as one that is under way", async () => {
  const { look, sleep } = page([200, 240, 280], [320]);
  const waited = await waitForScrollEnd(look, { sleep });
  assert.ok(waited >= 400 && waited < 500, `it waited ${waited} ms, not for an end it had already seen`);
});

test("a second scroll started as the first ends is waited for as well", async () => {
  // A key pressed again: the first scroll ends at 1300 and the next begins at 1340.
  const { look, sleep, now } = page([1100, 1150, 1200, 1340, 1380], [1300, 1480]);
  await waitForScrollEnd(look, { sleep });
  assert.ok(now() >= 1480, `it returned at ${now()}, between the two scrolls`);
});

test("it gives up on a scroll that never ends", async () => {
  const scrolls = Array.from({ length: 400 }, (_, index) => 1000 + index * 40);
  const { look, sleep, now } = page(scrolls, []);
  const waited = await waitForScrollEnd(look, { sleep, limit: 1500 });
  assert.ok(waited >= 1500 && now() - 1000 < 1700, `it waited ${waited} ms for a limit of 1500`);
});
