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
