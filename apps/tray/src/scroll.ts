import type { RefObject } from "preact";
import { useEffect, useState } from "preact/hooks";

/**
 * How many CSS pixels of hidden content are not content beneath an edge. A window a pixel short
 * of what it holds, as rounding to the screen's pixels can leave it, hides a pixel, and a line
 * drawn for that would stay up on a window that shows everything.
 */
const SLACK = 1.5;

/** What a Tab can reach: the controls the window has, outside a disclosure that is closed. */
const STOPS =
  'a[href], button:not(:disabled), input:not(:disabled), select:not(:disabled), textarea:not(:disabled), summary, [tabindex]:not([tabindex="-1"])';

/**
 * How much the scrolling part holds beyond its bottom edge, in CSS pixels, or none while it does
 * not scroll, as at 200% zoom, where the whole page scrolls in its place.
 */
function beyond(region: HTMLElement): number {
  return getComputedStyle(region).overflowY === "visible"
    ? 0
    : region.scrollHeight - region.clientHeight;
}

/** What the scrolling part tells a person of itself. */
export interface Scrolling {
  /** There is more below the bottom edge, which the toolbar is under: it has more to scroll to. */
  below: boolean;
  /** It scrolls and holds nothing Tab reaches, so it is a Tab stop itself, or a keyboard could
   * not scroll it. Chrome does the same for a scroller with no control in it; WebKit does not. */
  stop: boolean;
}

/**
 * What the scrolling part holds beyond its bottom edge, as it is scrolled, as the window grows or
 * shrinks and as what it holds changes; `view` names what it holds, so a new view is read at once.
 * Nothing is below while the part does not scroll. Whether a Tab reaches anything in it changes when
 * a control comes or goes or is disabled, which need not change its size, so what it holds is
 * watched as well as its size (a window that was told only when its size changed was left with a
 * part a keyboard could not scroll, in a window that stayed as tall).
 */
export function useScrolling(region: RefObject<HTMLElement | null>, view: string): Scrolling {
  const [scrolling, setScrolling] = useState<Scrolling>({ below: false, stop: false });
  useEffect(() => {
    const element = region.current;
    if (!element) {
      return;
    }
    const read = () => {
      const hidden = beyond(element);
      const below = hidden - element.scrollTop > SLACK;
      const stop = hidden > SLACK && element.querySelector(STOPS) === null;
      setScrolling((now) => (now.below === below && now.stop === stop ? now : { below, stop }));
    };
    read();
    element.addEventListener("scroll", read, { passive: true });
    const observer = new ResizeObserver(read);
    observer.observe(element);
    for (const child of element.children) {
      observer.observe(child);
    }
    const changes = new MutationObserver(read);
    changes.observe(element, {
      childList: true,
      subtree: true,
      attributes: true,
      attributeFilter: ["disabled", "tabindex", "href"],
    });
    return () => {
      element.removeEventListener("scroll", read);
      observer.disconnect();
      changes.disconnect();
    };
  }, [region, view]);
  return scrolling;
}

/**
 * The height the popover needs, in CSS pixels, for the app to size the window to: its own height,
 * which the window's height caps once there is more than fits, and what its scrolling part holds
 * beyond its bottom edge, so the window can grow to the most it is allowed to be. What the part
 * holds is measured from its content, to the fraction of a pixel, and not from `scrollHeight`, which
 * is rounded and could leave the window a pixel short.
 */
export function neededHeight(frame: HTMLElement): number {
  const region = frame.querySelector<HTMLElement>(".scroll");
  const content = region?.firstElementChild;
  if (!region || !content) {
    return Math.ceil(frame.getBoundingClientRect().height);
  }
  const style = getComputedStyle(region);
  const holds =
    content.getBoundingClientRect().height +
    Number.parseFloat(style.paddingTop) +
    Number.parseFloat(style.paddingBottom);
  return Math.ceil(frame.getBoundingClientRect().height - region.getBoundingClientRect().height + holds);
}
