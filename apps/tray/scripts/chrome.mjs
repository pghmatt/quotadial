// Headless Chrome for the window check: started with a profile of its own, answering the address
// its DevTools listen on once it says so.

import { spawn } from "node:child_process";

/**
 * How long Chrome may take to say where its DevTools listen. Chrome usually says so within a few
 * seconds, but a cold CI runner once took over 20 s, the wait this had, and failed main's CI on a
 * change that never touched the window (2026-10-25, CI run 40236269486). A minute leaves room for a
 * slow start and still stops a Chrome that will never answer.
 */
export const START_WAIT_MS = 60_000;

/**
 * Whether the check's Chrome draws scroll bars that take room, as Windows' and Linux's do, or ones
 * that lie over what they scroll, as a Mac's do: what `--scroll-bars` says, `classic` or `system`,
 * and without it the first for the checks and the system's own for the pictures, which are of what
 * a Mac shows.
 */
export function wantsClassicScrollbars(flag, pictures) {
  if (flag === undefined) {
    return !pictures;
  }
  if (flag !== "classic" && flag !== "system") {
    throw new Error(`--scroll-bars is "classic" or "system", not "${flag}"`);
  }
  return flag === "classic";
}

/**
 * Starts Chrome headless with `profile` as its own profile. Gives the running process, which the
 * caller stops, and a promise of its DevTools address, which fails with what Chrome said when it
 * exits first or has not answered within `waitMs`.
 *
 * Scroll bars take room, as they do on Windows and Linux and as they did on CI's Macs, unless
 * `classicScrollbars` is false. A Mac's own Chrome lets them lie over what they scroll, which hid a
 * window whose width changed when one appeared until CI showed it (D-136); the pictures of the
 * window are the one thing that wants a system's own, so they ask for it.
 */
export function launch(
  chrome,
  profile,
  { waitMs = START_WAIT_MS, platform = process.platform, classicScrollbars = true } = {},
) {
  const args = [
    "--headless=new",
    "--remote-debugging-port=0",
    `--user-data-dir=${profile}`,
    "--no-first-run",
    "--no-default-browser-check",
    "--disable-extensions",
    "--disable-background-networking",
    "--disable-component-update",
    "--disable-sync",
    "--force-color-profile=srgb",
    // A key or a wheel scrolls at once, so that where it has got to does not depend on how often a
    // slow machine draws a frame; the window does not glide, whatever the browser does.
    "--disable-smooth-scrolling",
    "about:blank",
  ];
  if (classicScrollbars) {
    args.splice(-1, 0, "--disable-features=OverlayScrollbar,FluentOverlayScrollbar");
  }
  // Linux runners refuse Chrome's sandbox; the page is only this build, served on this machine. Like
  // every flag, it goes before the page Chrome opens, which stays the last argument.
  if (platform === "linux") {
    args.splice(-1, 0, "--no-sandbox");
  }
  const browser = spawn(chrome, args, { stdio: ["ignore", "ignore", "pipe"] });
  const address = new Promise((done, fail) => {
    let said = "";
    const timer = setTimeout(
      () => fail(new Error(`Chrome did not start within ${waitMs / 1000} s: ${said}`)),
      waitMs,
    );
    browser.stderr.on("data", (chunk) => {
      said += chunk;
      const found = said.match(/DevTools listening on (ws:\/\/\S+)/);
      if (found) {
        clearTimeout(timer);
        done(found[1]);
      }
    });
    browser.on("exit", (code) => {
      clearTimeout(timer);
      fail(new Error(`Chrome exited (${code}): ${said}`));
    });
  });
  return { browser, address };
}
