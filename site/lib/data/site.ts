/**
 * Facts about QuotaDial the site shows. One place, so a fact changes here and nowhere else
 * (docs/CONTENT-BIBLE.md). Owner inputs that do not exist yet are null, and the pages say so.
 */

/** `value` if it is an https link to Lemon Squeezy, else null. */
export function checkoutLink(value: string | undefined): string | null {
  if (!value) {
    return null;
  }
  try {
    const url = new URL(value);
    return url.protocol === "https:" && /(^|\.)lemonsqueezy\.com$/.test(url.hostname) ? url.href : null;
  } catch {
    return null;
  }
}

export const SITE = {
  name: "QuotaDial",
  origin: "https://quotadial.com",
  repository: "https://github.com/pghmatt/quotadial",
  releases: "https://github.com/pghmatt/quotadial/releases",
  newIssue: "https://github.com/pghmatt/quotadial/issues/new/choose",
  securityReport: "https://github.com/pghmatt/quotadial/security/advisories/new",
  installGuide: "https://github.com/pghmatt/quotadial/blob/main/docs/INSTALL.md",
  cliGuide: "https://github.com/pghmatt/quotadial/blob/main/docs/CLI.md",
  licence: "https://github.com/pghmatt/quotadial/blob/main/LICENSE",
  support: "support@quotadial.com",
  /**
   * Lemon Squeezy's checkout link for QuotaDial Pro, from the build's QUOTADIAL_CHECKOUT_URL (a
   * repository variable the deploy workflow passes). Matt's to add (#51); until then /pro says
   * buying opens soon and offers QuotaDial Free.
   */
  checkout: checkoutLink(process.env.QUOTADIAL_CHECKOUT_URL),
} as const;

/** The header's links, in order. */
export const NAV = [
  { href: "/download", label: "Download" },
  { href: "/pro", label: "Pro" },
  { href: "/privacy", label: "Privacy" },
  { href: "/changelog", label: "Changelog" },
] as const;
