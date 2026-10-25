"use client";

import { type ReactNode, useEffect, useRef, useState } from "react";

/**
 * Keeps a chart at a readable width on a phone: under 768 px it scrolls sideways inside its own
 * box, starting at the right where the week is heading, and from 768 px it is the figure's width.
 * The box takes a keyboard stop only while there is something to scroll.
 */
export function ChartScroll({ label, children }: { label: string; children: ReactNode }) {
  const box = useRef<HTMLDivElement>(null);
  const [scrolls, setScrolls] = useState(false);
  useEffect(() => {
    const element = box.current;
    if (!element) {
      return;
    }
    element.scrollLeft = element.scrollWidth;
    const measure = () => setScrolls(element.scrollWidth > element.clientWidth + 1);
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(element);
    return () => observer.disconnect();
  }, []);
  return (
    <div
      ref={box}
      role={scrolls ? "region" : undefined}
      tabIndex={scrolls ? 0 : undefined}
      aria-label={scrolls ? label : undefined}
      className="overflow-x-auto md:overflow-visible"
    >
      <div className="min-w-[32rem] md:min-w-0">{children}</div>
    </div>
  );
}
