import { Fragment, type ReactNode } from "react";

/** `text` with a place to break the line after each underscore and hyphen, for file names on a phone. */
export function soft(text: string): ReactNode {
  return text.split(/(?<=[_-])/).map((part, index) => (
    <Fragment key={index}>
      {index > 0 && <wbr />}
      {part}
    </Fragment>
  ));
}
