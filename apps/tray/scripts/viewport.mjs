// The window's size at a zoom, for the window check (check-window.mjs). Zooming in narrows and
// shortens the page in CSS pixels: the tallest the popover is, 640 of them at 100%, is 320 at 200%
// and 213.33 at 300%. Chrome takes the size to emulate in whole pixels and refuses a fraction with
// "Invalid parameters". The check's pages rounded it, and so were judged at 300%, but its pictures
// (`--window-pictures`) did not, so `--window-zoom 3` could not make one. The rule is here, once,
// for both (the manager's decision on #73, 2026-10-27), so that a picture shows the very window the
// checks judged at that zoom.

/**
 * The window's size at `zoom` (2 for 200%) in whole CSS pixels, when it is `size.width` by
 * `size.height` at 100%: each the nearest whole number to its size over the zoom, as the check's
 * pages have always had it. Where the size divides by the zoom, as 360 by 640 does at 100% and 200%,
 * that is the size over the zoom itself.
 */
export function viewportAt(zoom, size) {
  return { width: Math.round(size.width / zoom), height: Math.round(size.height / zoom) };
}

/**
 * The zoom `--window-zoom` asks for (2 for 200%), from what follows the flag, or 100% with no flag:
 * a number above 0. What is not one is refused before the check makes a folder or asks Chrome for
 * anything (the manager's review of #100), in the form `--scroll-bars` refuses a value in: "abc"
 * reached Chrome as NaN and failed there with "Invalid parameters", and 0, or less, is no zoom.
 */
export function zoomOf(flag) {
  if (flag === undefined) {
    return 1;
  }
  const zoom = Number(flag);
  if (!Number.isFinite(zoom) || zoom <= 0) {
    throw new Error(`--window-zoom is a number above 0, such as 2 for 200%, not "${flag}"`);
  }
  return zoom;
}
