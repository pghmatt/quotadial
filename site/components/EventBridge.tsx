"use client";

import { useEffect } from "react";

import { isEventName, track } from "../lib/events.ts";

/**
 * Turns a click on anything marked `data-event="<name>"` into that event, with its
 * `data-event-<key>` attributes as the detail, so a page built on the server can declare its events
 * without becoming a client component.
 */
export function EventBridge() {
  useEffect(() => {
    const onClick = (event: MouseEvent) => {
      const marked = (event.target as Element | null)?.closest?.("[data-event]");
      const name = marked?.getAttribute("data-event") ?? "";
      if (!marked || !isEventName(name)) {
        return;
      }
      const detail: Record<string, string> = {};
      for (const { name: attribute, value } of Array.from(marked.attributes)) {
        if (attribute.startsWith("data-event-")) {
          detail[attribute.slice("data-event-".length)] = value;
        }
      }
      track(name, detail);
    };
    document.addEventListener("click", onClick, { capture: true });
    return () => document.removeEventListener("click", onClick, { capture: true });
  }, []);
  return null;
}
