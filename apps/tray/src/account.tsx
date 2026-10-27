import { Fragment, type ComponentChildren } from "preact";

import { addressParts, addressRuns, hostRuns } from "./address";

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
      <AddressLine who={who} />
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
function AddressLine({ who }: { who: Who }) {
  const named = who.alias !== null;
  if (!named && !who.name) {
    return null;
  }
  return (
    <>
      {" "}
      <span class="address">
        {who.name && (
          <span class="name">
            <Address text={who.name} />
          </span>
        )}
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

/**
 * An address, written the one way the window writes every address (D-141): whole, with a `<wbr>`
 * before its `@` and before each `.` (`addressParts`), so that a line too short for it breaks it
 * there and not inside a word. A `<wbr>` is an element and adds no character, so the text of what
 * holds it, and what a copy of it gives, is the address byte for byte. Each is hidden from screen
 * readers: Chrome, WebView2's engine, names a heading, a field or a button with a space where a
 * `<wbr>` it can see is, "dev @example .com", and with none where it cannot, so the address is
 * heard whole. An account's name with neither `@` nor `.`, such as "Google account", is written as
 * it is.
 */
export function Address({ text }: { text: string }) {
  return (
    <>
      {addressParts(text).map((part, index) => (
        <Fragment key={index}>
          {index > 0 && <wbr aria-hidden="true" />}
          {part}
        </Fragment>
      ))}
    </>
  );
}

/**
 * A line the app writes, with each of `addresses` that stands in it written as `Address` writes
 * one (`addressRuns`): the summary's line, which names the account whose window sets the number by
 * its address when it has no name given (D-132).
 */
export function WithAddresses({ text, addresses }: { text: string; addresses: readonly string[] }) {
  return (
    <>
      {addressRuns(text, addresses).map((run, index) =>
        run.address ? <Address key={index} text={run.text} /> : run.text,
      )}
    </>
  );
}

/**
 * A host, such as "release-assets.githubusercontent.com", written as `Address` writes an address
 * (D-141, extended to a host in a sentence): a host has no `@`, so a line too short for it breaks
 * it before a `.` and not inside a word, the text of its line holds it byte for byte, and a screen
 * reader reads it whole.
 */
export function Host({ text }: { text: string }) {
  return <Address text={text} />;
}

/**
 * A sentence the app writes, with each of `hosts` that stands in it written as `Host` writes one
 * (`hostRuns`): Help's text of what QuotaDial reads and sends, which names every place QuotaDial
 * connects to.
 */
export function WithHosts({ text, hosts }: { text: string; hosts: readonly string[] }) {
  return (
    <>
      {hostRuns(text, hosts).map((run, index) =>
        run.host ? <Host key={index} text={run.text} /> : run.text,
      )}
    </>
  );
}
