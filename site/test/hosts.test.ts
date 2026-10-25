import assert from "node:assert/strict";
import { test } from "node:test";

import { SITE } from "../lib/data/site.ts";
import { NOT_BUILT, built, builtFiles, type BuiltFile } from "./built.ts";

/** A host a built page may name, or one repository on it, and why. */
interface Allowed {
  host: string;
  /** Only addresses under this path, such as one repository on github.com. */
  path?: string;
  why: string;
}

/** The site's own host: the only one a page or stylesheet fetches from. */
const SITE_HOST = new URL(SITE.origin).host;

/**
 * Every host the built pages name, each with why, as the build of 25 October 2026 found them. The
 * app's promise on its own behalf (D-113) is no third-party scripts, fonts or analytics: apart from
 * the site, a page names these only as links a visitor may follow, examples in the guides' code,
 * or names that are never fetched, and the second test holds that nothing is fetched from them.
 * An entry no page uses any more fails the first test, so the list stays as short as the pages.
 */
const ALLOWED: Allowed[] = [
  { host: SITE_HOST, why: "the site itself: each page's canonical address and share card, and the structured data's addresses" },
  {
    host: "github.com",
    path: "/pghmatt/quotadial",
    why: "QuotaDial's repository: the downloads, release notes, issues, security reports, the licence, and the files the guides link to",
  },
  {
    host: "github.com",
    path: "/pghmatt/homebrew-quotadial",
    why: "QuotaDial's Homebrew tap and Scoop bucket, in the Scoop command on /install (docs/INSTALL.md, D-112)",
  },
  { host: "www.w3.org", why: "the SVG namespace of the icons drawn in each page (xmlns): a name, never fetched" },
  { host: "schema.org", why: "the vocabulary of the structured data on / and /download (@context): a name, never fetched" },
  { host: "status.claude.com", why: "Claude's status page, which /cli links as the page QuotaDial reads (docs/CLI.md)" },
  { host: "status.openai.com", why: "OpenAI's status page, which /cli links likewise and quotes in an example line" },
  { host: "rustup.rs", why: "where /install sends someone building QuotaDial for Rust (docs/INSTALL.md)" },
  { host: "127.0.0.1", why: "the local API on the reader's own machine, in the curl example on /cli (docs/CLI.md)" },
  { host: "example.com", why: "the webhook in the example config.json on /cli: a domain kept for examples" },
  { host: "hooks.slack.com", why: "the start of a Slack webhook's address in the example config.json on /cli" },
];

/** An address written in a built file. */
interface Address {
  url: string;
  host: string;
  path: string;
}

const ENTITIES: Record<string, string> = { "&quot;": '"', "&#x27;": "'", "&#39;": "'", "&lt;": "<", "&gt;": ">", "&amp;": "&" };

/**
 * Every address in `text` with its host: written with http or https anywhere, in the page or in
 * the data its scripts carry, or without a scheme ("//host/...") at the start of a value. HTML's
 * entities and the scripts' escapes (`&`, `\/`) are read first, so an address reads whole, as
 * it would on the page.
 */
function addresses(text: string): Address[] {
  const plain = text
    .replace(/&(?:quot|#x27|#39|lt|gt|amp);/g, (entity) => ENTITIES[entity])
    .replace(/\\u([0-9a-f]{4})/gi, (_, code: string) => String.fromCharCode(Number.parseInt(code, 16)))
    .replaceAll("\\/", "/");
  // A full stop or comma after an address in a sentence is not part of it.
  const trimmed = (part: string) => part.replace(/[.,;:!?]+$/, "");
  return [...plain.matchAll(/(?:\bhttps?:|(?<=["'(=]))\/\/([a-z0-9-]+(?:\.[a-z0-9-]+)*)(?::\d+)?([^\s"'<>()\\`]*)/gi)].map((found) => ({
    url: trimmed(found[0]),
    host: found[1].toLowerCase(),
    path: trimmed(found[2]),
  }));
}

/** Whether `entry` allows `address`: the same host, and under the entry's path when it has one. */
function allows(entry: Allowed, address: Address): boolean {
  if (entry.host !== address.host) {
    return false;
  }
  if (!entry.path) {
    return true;
  }
  const rest = address.path.slice(entry.path.length);
  return address.path.startsWith(entry.path) && (rest === "" || /^[/?#]/.test(rest));
}

/**
 * The addresses in `files` that no entry allows. The Buy button's link to Lemon Squeezy's checkout
 * is the one address that depends on the build: /pro carries it only when the build has
 * QUOTADIAL_CHECKOUT_URL, as the deploy's does, so it is allowed as that one link, read from the
 * same variable (`checkout`), and never as a host.
 */
function unlisted(files: BuiltFile[], checkout: string | null = SITE.checkout): string[] {
  return files.flatMap((file) =>
    addresses(file.text)
      .filter((address) => address.url !== checkout && !ALLOWED.some((entry) => allows(entry, address)))
      .map((address) => `${file.path} names ${address.url}`),
  );
}

/** The entries of the allowlist that no address in `files` uses. */
function unused(files: BuiltFile[]): Allowed[] {
  const found = files.flatMap((file) => addresses(file.text));
  return ALLOWED.filter((entry) => !found.some((address) => allows(entry, address)));
}

/** `text` without its data: addresses, whose contents (an SVG's namespace) are never fetched. */
function withoutData(text: string): string {
  return text.replace(/data:[^)"'\s]*/g, "");
}

/**
 * What `file` makes the browser fetch: everything a stylesheet names; and in a page, every address
 * in an element's attributes, but for a link's href (followed only when the visitor chooses) and a
 * namespace (xmlns), and in its style sheets. A script's own words are code and data, which the
 * first test reads.
 */
function fetched(file: BuiltFile): Address[] {
  if (file.path.endsWith(".css")) {
    return addresses(withoutData(file.text));
  }
  const found = [...file.text.matchAll(/<style\b[^>]*>([\s\S]*?)<\/style>/gi)].flatMap((sheet) => addresses(withoutData(sheet[1])));
  const tags = file.text.replace(/(<(script|style)\b[^>]*>)[\s\S]*?(<\/\2>)/gi, "$1$3");
  for (const [, element, attributes] of tags.matchAll(/<([a-z][a-z0-9-]*)\b([^>]*)>/gi)) {
    for (const [, name, value] of attributes.matchAll(/([^\s=/]+)\s*=\s*("[^"]*"|'[^']*'|[^\s>]+)/g)) {
      const link = element.toLowerCase() === "a" && name.toLowerCase() === "href";
      if (!link && !/^xmlns(?::|$)/i.test(name)) {
        found.push(...addresses(`=${withoutData(value)}`));
      }
    }
  }
  return found;
}

/** What `files` fetch from anywhere but the site. */
function fetchedElsewhere(files: BuiltFile[]): string[] {
  return files.flatMap((file) =>
    fetched(file)
      .filter((address) => address.host !== SITE_HOST)
      .map((address) => `${file.path} fetches ${address.url}`),
  );
}

const skip = built ? false : NOT_BUILT;

test("every address the built pages and stylesheet name is the site's own or on the allowlist, and every entry is used", { skip }, () => {
  const files = builtFiles(".html", ".css");
  assert.ok(files.some((file) => file.path === "index.html"), "the built pages were read");
  assert.ok(files.some((file) => file.path.endsWith(".css")), "the built stylesheet was read");
  assert.deepEqual(
    unlisted(files),
    [],
    "a built page names a host the allowlist does not; if it belongs there, add it with why (the Buy button's link is allowed only when the tests run with the QUOTADIAL_CHECKOUT_URL the build had)",
  );
  assert.deepEqual(
    unused(files).map((entry) => `${entry.host}${entry.path ?? ""}`),
    [],
    "no built page names these any more: take them off the allowlist",
  );
});

test("nothing the built pages and stylesheet fetch comes from anywhere but the site", { skip }, () => {
  const files = builtFiles(".html", ".css");
  const pages = files.filter((file) => file.path.endsWith(".html"));
  assert.ok(
    pages.length > 0 && pages.every((page) => fetched(page).some((address) => address.host === SITE_HOST)),
    "each page's own addresses are read, such as its share card's",
  );
  assert.deepEqual(fetchedElsewhere(files), []);
});

test("a third party planted in a built page is found, named or fetched, and the allowlist goes no wider than its entries", { skip }, () => {
  const [home] = builtFiles(".html").filter((file) => file.path === "index.html");
  const planted = (html: string): BuiltFile[] => [{ path: "index.html", text: home.text.replace("</body>", `${html}</body>`) }];
  assert.deepEqual(unlisted([home]), [], "the page as built");
  assert.deepEqual(fetchedElsewhere([home]), [], "the page as built");

  const analytics = planted('<script async src="https://www.googletagmanager.com/gtag/js?id=G-1"></script>');
  assert.deepEqual(unlisted(analytics), ["index.html names https://www.googletagmanager.com/gtag/js?id=G-1"]);
  assert.deepEqual(fetchedElsewhere(analytics), ["index.html fetches https://www.googletagmanager.com/gtag/js?id=G-1"]);

  // A font from Google, written without its scheme, in a link and in a style sheet.
  const font = planted('<link rel="stylesheet" href="//fonts.googleapis.com/css2?family=Inter"><style>@font-face{src:url(//fonts.gstatic.com/x.woff2)}</style>');
  assert.equal(unlisted(font).length, 2);
  assert.equal(fetchedElsewhere(font).length, 2);

  // An allowed host is still never fetched from: a picture from the repository.
  const picture = planted('<img src="https://github.com/pghmatt/quotadial/raw/main/docs/x.png" alt="">');
  assert.deepEqual(unlisted(picture), []);
  assert.deepEqual(fetchedElsewhere(picture), ["index.html fetches https://github.com/pghmatt/quotadial/raw/main/docs/x.png"]);

  // A link goes to a host on the list, and on GitHub only to QuotaDial's own repositories.
  for (const href of ["https://github.com/pghmatt", "https://github.com/pghmatt/quotadial-fork", "https://gist.github.com/x", "https://example.net/"]) {
    const link = planted(`<a href="${href}">x</a>`);
    assert.deepEqual(unlisted(link), [`index.html names ${href}`], href);
    assert.deepEqual(fetchedElsewhere(link), [], `${href} is a link, not fetched`);
  }
  assert.deepEqual(unlisted(planted('<a href="https://github.com/pghmatt/quotadial/issues">x</a>')), []);

  // The checkout link is allowed only as the one the build was given, read whole both where the page
  // writes it (&amp;) and where its script data does (&).
  const checkout = "https://quotadial.lemonsqueezy.com/buy/abc?enabled=1&media=0";
  const buy = planted(
    '<a href="https://quotadial.lemonsqueezy.com/buy/abc?enabled=1&amp;media=0">Buy</a>' +
      '<script>self.__next_f.push([1,"{\\"href\\":\\"https://quotadial.lemonsqueezy.com/buy/abc?enabled=1\\u0026media=0\\"}"])</script>',
  );
  assert.deepEqual(unlisted(buy, checkout), []);
  assert.deepEqual(unlisted(buy, null), [`index.html names ${checkout}`, `index.html names ${checkout}`]);
  assert.equal(unlisted(planted('<a href="https://quotadial.lemonsqueezy.com/other">x</a>'), checkout).length, 1);

  // An entry no page uses is reported: the home page alone names neither status page.
  assert.ok(unused([home]).some((entry) => entry.host === "status.claude.com"));
});
