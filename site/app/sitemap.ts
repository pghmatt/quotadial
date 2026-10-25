import type { MetadataRoute } from "next";

import { SITE } from "../lib/data/site.ts";

export const dynamic = "force-static";

/** Every page meant for search; the key page is not one. */
const PAGES = ["/", "/download", "/pro", "/privacy", "/support", "/changelog", "/install", "/cli"];

export default function sitemap(): MetadataRoute.Sitemap {
  return PAGES.map((path) => ({ url: new URL(path, SITE.origin).href }));
}
