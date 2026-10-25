/**
 * A window's share used over its period, drawn like a tide curve: time along, share up, the
 * high-water line at 100% dashed, the reset marked, and, when the window would run out first at
 * its pace, the prediction dashed in magenta to the point it meets 100%. Static: every time is
 * given, so the drawing is the same on every build.
 *
 * It is drawn twice, for wide screens and for phones, and CSS shows one: on a phone the wide
 * drawing would shrink its labels to about 7 px, so the narrow one keeps them at their size.
 */

/** Where things sit in one drawing, in its own units. */
interface Geometry {
  width: number;
  height: number;
  left: number;
  right: number;
  top: number;
  bottom: number;
  /** The size of the annotations (high water, the prediction, the reset). */
  note: number;
  /** The size of the axis labels. */
  axis: number;
}

const WIDE: Geometry = { width: 640, height: 314, left: 44, right: 612, top: 44, bottom: 252, note: 15, axis: 13 };
const NARROW: Geometry = { width: 360, height: 266, left: 44, right: 344, top: 42, bottom: 210, note: 14, axis: 12.5 };

export interface CurvePoint {
  /** Hours since the window began. */
  hour: number;
  used: number;
}

interface Props {
  /** What the curve shows, for screen readers and the figure's title. */
  title: string;
  /** The window's length in hours, such as 168 for a week. */
  hours: number;
  /** Labels along the time axis, each with its hour, such as { hour: 24, label: "Fri" }. */
  days: readonly { hour: number; label: string }[];
  points: readonly CurvePoint[];
  /** When the window resets, in hours since it began, and how to say it. */
  reset: { hour: number; label: string };
  /** When the window runs out at its pace, if before the reset. */
  runsOut?: { hour: number; label: string };
  animate?: boolean;
}

export function TideCurve(props: Props) {
  return (
    <>
      <Drawing {...props} geometry={WIDE} className="hidden sm:block" />
      <Drawing {...props} geometry={NARROW} className="block sm:hidden" />
    </>
  );
}

function Drawing({
  title,
  hours,
  days,
  points,
  reset,
  runsOut,
  animate = false,
  geometry: g,
  className,
}: Props & { geometry: Geometry; className: string }) {
  const x = (hour: number) => g.left + ((g.right - g.left) * hour) / hours;
  const y = (used: number) => g.bottom - ((g.bottom - g.top) * used) / 100;
  const line = points.map((point) => `${x(point.hour).toFixed(1)},${y(point.used).toFixed(1)}`).join(" ");
  const last = points[points.length - 1];
  const area = `${x(points[0].hour).toFixed(1)},${g.bottom} ${line} ${x(last.hour).toFixed(1)},${g.bottom}`;
  // Two rows of words: the notes above high water, and the axis with the reset below it.
  const above = y(100) - 14;
  const daysRow = g.bottom + g.axis + 6;
  const resetRow = daysRow + g.note + 8;
  return (
    <svg
      viewBox={`0 0 ${g.width} ${g.height}`}
      role="img"
      aria-label={title}
      className={`${className} h-auto w-full font-display`}
    >
      {/* The grid: a line a day, and a quarter of the way up at a time. */}
      {days.map((day) => (
        <line key={day.hour} x1={x(day.hour)} x2={x(day.hour)} y1={g.top} y2={g.bottom} stroke="var(--rule)" strokeWidth="1" />
      ))}
      {[25, 50, 75].map((used) => (
        <line key={used} x1={g.left} x2={g.right} y1={y(used)} y2={y(used)} stroke="var(--rule)" strokeWidth="1" />
      ))}
      <line x1={g.left} x2={g.right} y1={g.bottom} y2={g.bottom} stroke="var(--ink-2)" strokeWidth="1" />

      {/* High water: the limit. */}
      <line x1={g.left} x2={g.right} y1={y(100)} y2={y(100)} stroke="var(--ink)" strokeWidth="1.5" strokeDasharray="6 5" />
      <text x={g.left} y={above} fill="var(--ink)" fontSize={g.note} fontStyle="italic" fontWeight="600">
        High water: 100%
      </text>

      {/* The reset: its line rises from the axis, and its words sit under the axis's days. */}
      <line x1={x(reset.hour)} x2={x(reset.hour)} y1={y(100) - 6} y2={g.bottom + 4} stroke="var(--ink-2)" strokeWidth="1.5" />
      <text x={x(reset.hour)} y={resetRow} fill="var(--ink-2)" fontSize={g.note} fontStyle="italic" textAnchor="end">
        {reset.label}
      </text>

      {/* The share used so far. */}
      <polygon points={area} fill="var(--mid)" opacity="0.7" />
      <polyline
        points={line}
        fill="none"
        stroke="var(--line)"
        strokeWidth="3"
        strokeLinejoin="round"
        strokeLinecap="round"
        pathLength={1}
        strokeDasharray="1"
        className={animate ? "motion-safe:animate-draw" : undefined}
      />

      {/* At this pace: the prediction to high water, its words above the line at the right. */}
      {runsOut && (
        <g>
          <line
            x1={x(last.hour)}
            y1={y(last.used)}
            x2={x(runsOut.hour)}
            y2={y(100)}
            stroke="var(--magenta)"
            strokeWidth="2.5"
            strokeDasharray="5 5"
            strokeLinecap="round"
          />
          <circle cx={x(runsOut.hour)} cy={y(100)} r="5" fill="var(--magenta)" />
          <text x={g.right} y={above} fill="var(--magenta)" fontSize={g.note} fontStyle="italic" fontWeight="650" textAnchor="end">
            {runsOut.label}
          </text>
        </g>
      )}

      {days.map((day) => (
        <text key={day.label} x={x(day.hour) + 5} y={daysRow} fill="var(--ink-2)" fontSize={g.axis} className="font-table">
          {day.label}
        </text>
      ))}
      {[0, 50, 100].map((used) => (
        <text key={used} x={g.left - 7} y={y(used) + 4} fill="var(--ink-2)" fontSize={g.axis} textAnchor="end" className="font-table">
          {used}%
        </text>
      ))}
    </svg>
  );
}
