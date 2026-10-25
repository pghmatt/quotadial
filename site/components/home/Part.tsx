import type { ReactNode } from "react";

/**
 * One section of the home page inside a depth band: a named region, set apart from the one before
 * it by space alone, because the band's tone does not change between them.
 */
export function Part({ id, headingId, children }: { id: string; headingId: string; children: ReactNode }) {
  return (
    <section
      id={id}
      aria-labelledby={headingId}
      className="[&:not(:first-child)]:mt-24 md:[&:not(:first-child)]:mt-32"
    >
      {children}
    </section>
  );
}
