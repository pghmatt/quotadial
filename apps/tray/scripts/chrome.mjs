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
 * Starts Chrome headless with `profile` as its own profile. Gives the running process, which the
 * caller stops, and a promise of its DevTools address, which fails with what Chrome said when it
 * exits first or has not answered within `waitMs`.
 */
export function launch(chrome, profile, { waitMs = START_WAIT_MS, platform = process.platform } = {}) {
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
    "about:blank",
  ];
  // Linux runners refuse Chrome's sandbox; the page is only this build, served on this machine.
  if (platform === "linux") {
    args.push("--no-sandbox");
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
