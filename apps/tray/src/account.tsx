import type { ComponentChildren } from "preact";

/** Who an account is: its provider's own name (P-020), the name the user gave it, if any, and its
 * own name, such as its email address, empty when that says nothing beyond its provider. */
export interface Who {
  provider: string;
  alias: string | null;
  name: string;
}

/**
 * An account's heading, the same in the popover and History: the name given it, or its provider's,
 * with whatever `children` puts beside it (the popover's plan and status), and under it the
 * account's address. The heading holds both lines, so a screen reader hears the address with the
 * name; the spaces between the parts are for it, as a flex row draws none.
 */
export function AccountHeading({
  who,
  id,
  level,
  children,
}: {
  who: Who;
  id: string;
  level: 2 | 3;
  children?: ComponentChildren;
}) {
  const parts = (
    <>
      <span class="title-row">
        <span class="title">{who.alias ?? who.provider}</span> {children}
      </span>
      <Address who={who} />
    </>
  );
  return level === 2 ? (
    <h2 id={id} class="card-head">
      {parts}
    </h2>
  ) : (
    <h3 id={id} class="card-head">
      {parts}
    </h3>
  );
}

/**
 * An account's second line: its address, then its provider's name when the title is the name the
 * user gave it, as "dev@example.com · Claude", so the addresses start at one edge down the cards
 * and read as a column. Nothing when the provider's name says it all, as for the example build's
 * accounts, which have no address (D-124). The address alone can be selected, to copy it.
 */
function Address({ who }: { who: Who }) {
  const named = who.alias !== null;
  if (!named && !who.name) {
    return null;
  }
  return (
    <>
      {" "}
      <span class="address">
        {who.name && <span class="name">{who.name}</span>}
        {named && who.name && (
          <>
            {" "}
            <span aria-hidden="true">·</span>{" "}
          </>
        )}
        {named && <span class="provider">{who.provider}</span>}
      </span>
    </>
  );
}
