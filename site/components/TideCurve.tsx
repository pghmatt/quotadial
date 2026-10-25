/**
 * A window's share used over its period, drawn like a tide curve: time along, share up, the
 * high-water line at 100% dashed, the reset marked, and, when the window would run out first at
 * its pace, the prediction dashed in magenta to the point it meets 100%. Static: every time is
 * given, so the drawing is the same on every build.
 */

const WIDTH = 640;
const HEIGHT = 300;
const LEFT = 44;
const RIGHT = 612;
const TOP = 36;
const BOTTOM = 252;

export interface CurvePoint {
  /** Hours since the window began. */
  hour: number;
  used: number;
}

export function TideCurve({
  title,
  hours,
  days,
  points,
  reset,
  runsOut,
  animate = false,
}: {
  /** What the curve shows, for screen readers and the figure's title. */
  title: string;
  /** The window's length in hours, such as 168 for a week. */
  hours: number;
  /** Labels along the time axis, each with its hour, such as { hour: 24, label: "Fri" }. */
  days: { hour: number; label: string }[];
  points: CurvePoint[];
  /** When the window resets, in hours since it began, and how to say it. */
  reset: { hour: number; label: string };
  /** When the window runs out at its pace, if before the reset. */
  runsOut?: { hour: number; label: string };
  animate?: boolean;
}) {
  const x = (hour: number) => LEFT + ((RIGHT - LEFT) * hour) / hours;
  const y = (used: number) => BOTTOM - ((BOTTOM - TOP) * used) / 100;
  const line = points.map((point) => `${x(point.hour).toFixed(1)},${y(point.used).toFixed(1)}`).join(" ");
  const last = points[points.length - 1];
  const area = `${x(points[0].hour).toFixed(1)},${BOTTOM} ${line} ${x(last.hour).toFixed(1)},${BOTTOM}`;
  return (
    <svg
      viewBox={`0 0 ${WIDTH} ${HEIGHT}`}
      role="img"
      aria-label={title}
      className="block h-auto w-full font-display"
    >
      {/* The grid: a line a day, and a quarter of the way up at a time. */}
      {days.map((day) => (
        <line key={day.hour} x1={x(day.hour)} x2={x(day.hour)} y1={TOP} y2={BOTTOM} stroke="var(--rule)" strokeWidth="1" />
      ))}
      {[25, 50, 75].map((used) => (
        <line key={used} x1={LEFT} x2={RIGHT} y1={y(used)} y2={y(used)} stroke="var(--rule)" strokeWidth="1" />
      ))}
      <line x1={LEFT} x2={RIGHT} y1={BOTTOM} y2={BOTTOM} stroke="var(--ink-2)" strokeWidth="1" />

      {/* High water: the limit. */}
      <line x1={LEFT} x2={RIGHT} y1={y(100)} y2={y(100)} stroke="var(--ink)" strokeWidth="1.5" strokeDasharray="6 5" />
      <text x={LEFT} y={y(100) - 10} fill="var(--ink)" fontSize="15" fontStyle="italic" fontWeight="600">
        High water: 100%
      </text>

      {/* The reset. */}
      <line x1={x(reset.hour)} x2={x(reset.hour)} y1={TOP - 8} y2={BOTTOM} stroke="var(--ink-2)" strokeWidth="1.5" />
      <text x={x(reset.hour) - 6} y={BOTTOM + 30} fill="var(--ink-2)" fontSize="15" fontStyle="italic" textAnchor="end">
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

      {/* At this pace: the prediction to high water. */}
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
          <text x={x(runsOut.hour) + 8} y={y(100) + 22} fill="var(--magenta)" fontSize="15" fontStyle="italic" fontWeight="650">
            {runsOut.label}
          </text>
        </g>
      )}

      {days.map((day) => (
        <text key={day.label} x={x(day.hour) + 6} y={BOTTOM + 18} fill="var(--ink-2)" fontSize="13" className="font-table">
          {day.label}
        </text>
      ))}
      {[0, 50, 100].map((used) => (
        <text key={used} x={LEFT - 8} y={y(used) + 4} fill="var(--ink-2)" fontSize="13" textAnchor="end" className="font-table">
          {used}%
        </text>
      ))}
    </svg>
  );
}
