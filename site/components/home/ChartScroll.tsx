"use client";

import { type ReactNode, useEffect, useRef } from "react";

/**
 * Keeps a chart at a readable width on a phone: under 768 px it scrolls sideways inside its own
 * box, starting at the right where the week is heading, and from 768 px it is the figure's width.
 */
export function ChartScroll({ label, children }: { label: string; children: ReactNode }) {
  const box = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const element = box.current;
    if (element) {
      element.scrollLeft = element.scrollWidth;
    }
  }, []);
  return (
    <div
      ref={box}
      role="region"
      tabIndex={0}
      aria-label={label}
      className="overflow-x-auto md:overflow-visible"
    >
      <div className="min-w-[32rem] md:min-w-0">{children}</div>
    </div>
  );
}
