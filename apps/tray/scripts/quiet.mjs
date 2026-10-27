// How the window check knows a scroll is over. A key or a wheel can scroll in a glide, and on a
// slow machine the glide can start late and stall between frames: on the CI Macs it left the
// check reading where a window had got to, twice in a row 50 ms apart, while it was still on its
// way, so PageUp "did not scroll" and the Home after it stopped at the PageUp's end (D-136). So the
// check does not ask whether the position has stopped changing, which a stall answers falsely. It
// asks the page, which says when a scroll starts (`scroll`) and when it ends (`scrollend`), and
// waits for the end of the scroll it saw start; and a scroll that has not started within the time
// one takes to begin is one that is not going to.

/**
 * Waits until the scroll that follows an input is over, and says how long it waited. `look` answers,
 * from the page's clock, `{ now, lastScroll, lastEnd }`: the time, when anything last scrolled and
 * when a scroll last ended. If nothing has scrolled by `patience` ms, nothing will; once something
 * has, the wait is over when a scroll has ended since the last movement and `settle` ms have passed
 * with nothing moving, so that a second scroll started at once, such as a key pressed again, is
 * waited for too. It gives up after `limit` ms.
 */
export async function waitForScrollEnd(
  look,
  { patience = 400, settle = 100, limit = 6000, step = 50, sleep = (ms) => new Promise((done) => setTimeout(done, ms)) } = {},
) {
  const began = (await look()).now;
  for (;;) {
    const { now, lastScroll, lastEnd } = await look();
    const waited = now - began;
    const started = lastScroll >= began;
    if (waited >= limit) {
      return waited;
    }
    if (!started && waited >= patience) {
      return waited;
    }
    if (started && lastEnd >= lastScroll && now - lastEnd >= settle) {
      return waited;
    }
    await sleep(step);
  }
}
