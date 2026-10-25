import type { ReactNode } from "react";

/** The depth tones, which descend the page from chart white to the deep. */
export type Tone = "chart" | "shallow" | "mid" | "deep";

const TONES: Record<Tone, string> = {
  chart: "bg-chart text-ink",
  shallow: "bg-shallow text-ink",
  mid: "bg-mid text-ink",
  deep: "bg-deep text-on-deep",
};

/** A full-width band in one depth tone, its content on the site's grid. */
export function Section({
  tone = "chart",
  id,
  labelledBy,
  className = "",
  children,
}: {
  tone?: Tone;
  id?: string;
  labelledBy?: string;
  className?: string;
  children: ReactNode;
}) {
  return (
    <section id={id} aria-labelledby={labelledBy} className={`${TONES[tone]} ${className}`}>
      <div className="mx-auto w-full max-w-[80rem] px-4 py-16 md:px-6 md:py-24">{children}</div>
    </section>
  );
}

/** A section heading: more space above than below, in the condensed display face. */
export function SectionHeading({ id, children }: { id: string; children: ReactNode }) {
  return (
    <h2 id={id} className="max-w-[24ch] text-[2rem] leading-[1.05] font-[750] tracking-[-0.01em] md:text-[2.75rem]">
      {children}
    </h2>
  );
}
