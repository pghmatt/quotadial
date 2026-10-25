/**
 * Every page's metadata, from one helper, which alone adds " | QuotaDial" to a title
 * (docs/seo/site.config.json holds each page's exact title).
 */

import type { Metadata } from "next";

import { SITE } from "./data/site.ts";

const SHARE = { path: "/opengraph-image.png", width: 1200, height: 630 } as const;
export const SHARE_ALT =
  "QuotaDial: Claude and Codex plan limits with when each resets, beside the app's dial";

export function pageMetadata(page: {
  /** The page's own title, without the brand. */
  title: string;
  description: string;
  path: string;
  /** Left out of search results, such as the key page. */
  noindex?: boolean;
}): Metadata {
  const title = `${page.title} | ${SITE.name}`;
  const url = new URL(page.path, SITE.origin).href;
  const image = new URL(SHARE.path, SITE.origin).href;
  return {
    title: { absolute: title },
    description: page.description,
    alternates: { canonical: url },
    robots: page.noindex ? { index: false, follow: true } : undefined,
    openGraph: {
      title,
      description: page.description,
      url,
      siteName: SITE.name,
      type: "website",
      images: [{ url: image, width: SHARE.width, height: SHARE.height, alt: SHARE_ALT }],
    },
    twitter: {
      card: "summary_large_image",
      title,
      description: page.description,
      images: [{ url: image, alt: SHARE_ALT }],
    },
  };
}

/** QuotaDial as schema.org describes software, with no price and no rating (none exist). */
export function softwareJsonLd() {
  return {
    "@context": "https://schema.org",
    "@type": "SoftwareApplication",
    name: SITE.name,
    url: SITE.origin,
    applicationCategory: "DeveloperApplication",
    operatingSystem: "macOS, Windows, Linux",
    downloadUrl: new URL("/download", SITE.origin).href,
    license: SITE.licence,
    isAccessibleForFree: true,
    sameAs: [SITE.repository],
  };
}

/** A JSON-LD script's text, with nothing in it able to close the script early. */
export const jsonLd = (value: object) => JSON.stringify(value).replace(/</g, "\\u003c");
