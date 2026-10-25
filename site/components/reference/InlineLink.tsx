import type { AnchorHTMLAttributes } from "react";

/** A link inside a sentence: underlined, in the text's own colour, as the site's reading text sets links. */
export function InlineLink({ className = "", ...anchor }: AnchorHTMLAttributes<HTMLAnchorElement>) {
  return (
    <a
      {...anchor}
      className={`underline decoration-1 underline-offset-[0.2em] hover:decoration-2 ${className}`}
    />
  );
}
