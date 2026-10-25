import { Fragment } from "react";

import { breakPieces } from "../lib/breaks.ts";

/**
 * `text` with a `<wbr>` at each place it may break (`breakPieces`). Set it with
 * `overflow-wrap: break-word`, never `anywhere`: the browser then breaks at these places first and
 * inside a piece only when one piece is wider than the whole line, and a table keeps a file name
 * whole when it has the room.
 */
export function Breakable({ text }: { text: string }) {
  const pieces = breakPieces(text);
  return pieces.map((piece, i) => (
    <Fragment key={i}>
      {piece}
      {i < pieces.length - 1 && <wbr />}
    </Fragment>
  ));
}
