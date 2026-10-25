import { Fragment } from "react";

import { breakTokens } from "../lib/breaks.ts";

/**
 * `text`, breaking only at a space or at the places `breakTokens` marks: each piece between them is
 * set unbreakable and the pieces of a word are joined by `<wbr>`, so the browser never breaks after
 * a hyphen of its own choosing (a flag such as `--json` stays whole). Set it with
 * `overflow-wrap: break-word`, never `anywhere`.
 */
export function Breakable({ text }: { text: string }) {
  return breakTokens(text).map((token, i) =>
    "space" in token ? (
      <Fragment key={i}>{token.space}</Fragment>
    ) : (
      <Fragment key={i}>
        {token.pieces.map((piece, j) => (
          <Fragment key={j}>
            {j > 0 && <wbr />}
            <span className="whitespace-nowrap">{piece}</span>
          </Fragment>
        ))}
      </Fragment>
    ),
  );
}
