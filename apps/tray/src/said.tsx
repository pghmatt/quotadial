import { codeSpans } from "./code";

/**
 * A message QuotaDial shows, with each command in it as code that one click selects whole, so it
 * can be copied exactly as it must be pasted (the manager's note on #85).
 */
export function Said({ text }: { text: string }) {
  return (
    <>
      {codeSpans(text).map((span, index) =>
        span.code ? <code key={index}>{span.text}</code> : span.text,
      )}
    </>
  );
}
