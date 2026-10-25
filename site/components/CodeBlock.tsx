import type { CSSProperties } from "react";

import { Breakable } from "./Breakable.tsx";
import { CopyButton } from "./CopyButton.tsx";

/** How far the sign of more text reaches in from the block's edge. */
const EDGE = "2rem";
const SHADE = "color-mix(in srgb, var(--ink) 24%, transparent)";

/**
 * The sign that a block of output scrolls: a shade over the last characters on the side that has
 * more text, and no shade on a side that has none, so a block that fits shows nothing. Two covers in
 * the block's own colour travel with the text (`local`) and hide the two shades (`scroll`, fixed to
 * the edges) when the text is at that end. It is drawn by CSS alone, follows the colour tokens in
 * both themes, and adds nothing to the page for a screen reader or the keyboard.
 */
const SCROLL_SIGN: CSSProperties = {
  backgroundRepeat: "no-repeat",
  backgroundSize: `${EDGE} 100%`,
  backgroundPosition: "left, right, left, right",
  backgroundAttachment: "local, local, scroll, scroll",
  backgroundImage: [
    "linear-gradient(to right, var(--mid), var(--mid))",
    "linear-gradient(to left, var(--mid), var(--mid))",
    `linear-gradient(to right, ${SHADE}, transparent)`,
    `linear-gradient(to left, ${SHADE}, transparent)`,
  ].join(", "),
};

/**
 * Commands or output, in JetBrains Mono, with a button that copies them. Long lines wrap, so a
 * phone shows the whole command, breaking after a slash, an underscore or a word's hyphen and never
 * inside a word or a flag (`Breakable`); `wrap={false}` keeps columns of output aligned and scrolls
 * them sideways instead, as a region the keyboard can reach, with a shade over the last characters
 * while there is more text to the side (`SCROLL_SIGN`).
 */
export function CodeBlock({
  code,
  label = "Copy",
  copy = true,
  wrap = true,
  name,
}: {
  code: string;
  label?: string;
  copy?: boolean;
  wrap?: boolean;
  /** What a screen reader calls the block when it scrolls, such as "quotadial status output". */
  name?: string;
}) {
  return (
    <div className="rounded-chart bg-mid p-4 text-ink">
      <pre
        className={`font-code text-[0.9375rem] leading-[1.6] ${wrap ? "m-0 whitespace-pre-wrap [overflow-wrap:break-word]" : "-m-4 overflow-x-auto p-4"}`}
        style={wrap ? undefined : SCROLL_SIGN}
        tabIndex={wrap ? undefined : 0}
        role={wrap ? undefined : "region"}
        aria-label={wrap ? undefined : name}
      >
        <code>{wrap ? <Breakable text={code} /> : code}</code>
      </pre>
      {copy && (
        <div className="mt-3">
          <CopyButton text={code} label={label} />
        </div>
      )}
    </div>
  );
}
