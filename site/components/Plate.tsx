import type { ReactNode } from "react";

/**
 * An almanac plate: a figure, what goes with it, and a caption saying what it shows. The site's
 * container in place of cards; never nested.
 */
export function Plate({
  tone = "shallow",
  caption,
  className = "",
  children,
}: {
  tone?: "shallow" | "mid" | "chart";
  caption?: ReactNode;
  className?: string;
  children: ReactNode;
}) {
  const field = { shallow: "bg-shallow", mid: "bg-mid", chart: "bg-chart" }[tone];
  return (
    <figure className={`m-0 rounded-chart ${field} p-4 md:p-6 ${className}`}>
      {children}
      {caption && (
        <figcaption className="mt-4 font-table text-[0.9375rem] text-ink-2 italic">{caption}</figcaption>
      )}
    </figure>
  );
}
