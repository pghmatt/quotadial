/**
 * The site's events, as docs/seo/site.config.json names them. No analytics service is chosen (that
 * is Matt's decision, and the privacy page would say so): an event is kept in
 * `window.quotadialEvents` and dispatched on the page as `quotadial:event`, so it can be tested now
 * and sent somewhere later.
 */

export const EVENTS = [
  "download_click",
  "checkout_click",
  "key_shown",
  "key_copied",
  "support_mail_click",
  "issue_click",
] as const;

export type EventName = (typeof EVENTS)[number];

export interface SiteEvent {
  name: EventName;
  detail: Record<string, string>;
  at: number;
}

declare global {
  interface Window {
    quotadialEvents?: SiteEvent[];
  }
}

export function isEventName(value: string): value is EventName {
  return (EVENTS as readonly string[]).includes(value);
}

export function track(name: EventName, detail: Record<string, string> = {}): SiteEvent | null {
  if (typeof window === "undefined") {
    return null;
  }
  const event: SiteEvent = { name, detail, at: Date.now() };
  window.quotadialEvents ??= [];
  window.quotadialEvents.push(event);
  window.dispatchEvent(new CustomEvent("quotadial:event", { detail: event }));
  return event;
}
