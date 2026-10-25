/**
 * QuotaDial's mark: the app icon (apps/tray/app-icon.svg), its dial drawn at `used` percent. The
 * 240-degree arc runs from lower left over the top to lower right; the needle points at the share
 * used. With `animate`, the needle sweeps up from 0 once; reduced motion shows it at rest.
 */

const CENTRE = { x: 512, y: 560 };
const RADIUS = 300;
const START = 210;
const SWEEP = 240;

function point(degrees: number) {
  const radians = (degrees * Math.PI) / 180;
  return { x: CENTRE.x + RADIUS * Math.cos(radians), y: CENTRE.y - RADIUS * Math.sin(radians) };
}

export function Dial({
  used = 65,
  size = 32,
  animate = false,
  title,
}: {
  used?: number;
  size?: number;
  animate?: boolean;
  /** Read by screen readers; leave out when the dial sits beside the name. */
  title?: string;
}) {
  const share = Math.max(0, Math.min(100, used)) / 100;
  const end = START - SWEEP * share;
  const from = point(START);
  const to = point(end);
  const large = SWEEP * share > 180 ? 1 : 0;
  const track = point(START - SWEEP);
  // The needle is drawn pointing straight up and turned to the share used.
  const turn = 90 - end;
  return (
    <svg
      viewBox="0 0 1024 1024"
      width={size}
      height={size}
      role={title ? "img" : undefined}
      aria-label={title}
      aria-hidden={title ? undefined : true}
      className="shrink-0"
    >
      <rect x="64" y="64" width="896" height="896" rx="200" fill="#1c1c1e" />
      <path
        d={`M ${from.x.toFixed(2)} ${from.y.toFixed(2)} A ${RADIUS} ${RADIUS} 0 1 1 ${track.x.toFixed(2)} ${track.y.toFixed(2)}`}
        fill="none"
        stroke="#3a3a3c"
        strokeWidth="84"
        strokeLinecap="round"
      />
      {share > 0 && (
        <path
          d={`M ${from.x.toFixed(2)} ${from.y.toFixed(2)} A ${RADIUS} ${RADIUS} 0 ${large} 1 ${to.x.toFixed(2)} ${to.y.toFixed(2)}`}
          fill="none"
          stroke="#0a84ff"
          strokeWidth="84"
          strokeLinecap="round"
          pathLength={1}
          strokeDasharray="1"
          className={animate ? "motion-safe:animate-draw" : undefined}
        />
      )}
      <g
        style={{ transformOrigin: `${CENTRE.x}px ${CENTRE.y}px`, transform: `rotate(${turn}deg)`, ["--needle" as string]: `${turn}deg` }}
        className={animate ? "motion-safe:animate-sweep" : undefined}
      >
        <line x1={CENTRE.x} y1={CENTRE.y} x2={CENTRE.x} y2={CENTRE.y - 230} stroke="#ffffff" strokeWidth="44" strokeLinecap="round" />
      </g>
      <circle cx={CENTRE.x} cy={CENTRE.y} r="60" fill="#ffffff" />
    </svg>
  );
}
