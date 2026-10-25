import type { AnchorHTMLAttributes, ReactNode } from "react";

type Variant = "primary" | "secondary" | "text";

const VARIANTS: Record<Variant, string> = {
  primary:
    "no-underline bg-magenta-fill text-on-magenta px-5 py-3 hover:bg-[color-mix(in_srgb,var(--magenta-fill)_86%,black)] active:translate-y-px",
  secondary:
    "no-underline border border-current px-5 py-[calc(0.75rem-1px)] hover:bg-[color-mix(in_srgb,currentColor_8%,transparent)] active:translate-y-px",
  text: "underline decoration-1 underline-offset-[0.2em] hover:decoration-2",
};

/**
 * A link styled as the site's buttons. Primary is magenta, one per view; secondary is a 1 px
 * outline; text is an underlined link. Every label names its action.
 */
export function ButtonLink({
  variant = "primary",
  className = "",
  children,
  ...anchor
}: AnchorHTMLAttributes<HTMLAnchorElement> & { variant?: Variant; children: ReactNode }) {
  return (
    <a
      {...anchor}
      className={`inline-flex items-center gap-2 rounded-chart font-display text-[1.125rem] leading-none font-[650] whitespace-nowrap transition-colors ${VARIANTS[variant]} ${className}`}
    >
      {children}
    </a>
  );
}
