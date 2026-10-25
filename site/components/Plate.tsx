import type { ReactNode } from "react";

/**
 * An almanac plate: a figure, what goes with it, and a caption saying what it shows. The site's
 * container in place of cards; never nested.
 *
 * A plate is framed as a chart plate is: a hairline neatline inset from its edge, with a graduated
 * scale in the margin between, drawn as SVG line work. A title, when it has one, is set like the
 * chart's own labels.
 */
export function Plate({
  tone = "shallow",
  title,
  caption,
  frame = true,
  className = "",
  children,
}: {
  tone?: "shallow" | "mid" | "chart";
  /** What the plate is, set at its head like a chart's title. */
  title?: ReactNode;
  caption?: ReactNode;
  /** The neatline and its scale; off only where the plate holds something framed already. */
  frame?: boolean;
  className?: string;
  children: ReactNode;
}) {
  const field = { shallow: "bg-shallow", mid: "bg-mid", chart: "bg-chart" }[tone];
  return (
    <figure className={`relative m-0 rounded-chart ${field} p-5 md:p-7 ${className}`}>
      {frame && <Neatline />}
      <div className="relative">
        {title && <p className="m-0 mb-4 font-display text-[1rem] leading-[1.3] font-[650] text-ink-2 italic">{title}</p>}
        {children}
        {caption && (
          <figcaption className="mt-4 font-table text-[0.9375rem] text-ink-2 italic">{caption}</figcaption>
        )}
      </div>
    </figure>
  );
}

/** The neatline, 8 px in from the plate's edge, and a scale on each side: a tick every 12 px, a longer one every 60. */
function Neatline() {
  const scale = (side: "top" | "bottom" | "left" | "right") => {
    const across = side === "top" || side === "bottom";
    const place = {
      top: "top-0 left-2 h-2 w-[calc(100%-1rem)]",
      bottom: "bottom-0 left-2 h-2 w-[calc(100%-1rem)] rotate-180",
      left: "left-0 top-2 w-2 h-[calc(100%-1rem)]",
      right: "right-0 top-2 w-2 h-[calc(100%-1rem)] rotate-180",
    }[side];
    // Each scale is one line drawn thick and dashed: the dashes are the ticks, hanging from the
    // neatline towards the plate's edge.
    return (
      <svg key={side} aria-hidden className={`pointer-events-none absolute ${place}`} shapeRendering="crispEdges">
        {across ? (
          <>
            <line x1="0" y1="7" x2="100%" y2="7" stroke="var(--line)" strokeOpacity="0.45" strokeWidth="3" strokeDasharray="1 11" />
            <line x1="0" y1="5" x2="100%" y2="5" stroke="var(--line)" strokeOpacity="0.6" strokeWidth="6" strokeDasharray="1 59" />
          </>
        ) : (
          <>
            <line x1="7" y1="0" x2="7" y2="100%" stroke="var(--line)" strokeOpacity="0.45" strokeWidth="3" strokeDasharray="1 11" />
            <line x1="5" y1="0" x2="5" y2="100%" stroke="var(--line)" strokeOpacity="0.6" strokeWidth="6" strokeDasharray="1 59" />
          </>
        )}
      </svg>
    );
  };
  return (
    <>
      <span aria-hidden className="pointer-events-none absolute inset-2 border border-[color-mix(in_srgb,var(--line)_55%,transparent)]" />
      {(["top", "bottom", "left", "right"] as const).map(scale)}
    </>
  );
}
