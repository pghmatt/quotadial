import type { NextConfig } from "next";

// The site is static files for Cloudflare Pages; the one step that is not, turning a receipt's key
// into a QuotaDial key, is the Pages Function in functions/api/key.ts. `npm run check` type-checks
// everything with TypeScript 7, so the build does not check types a second time.
const config: NextConfig = {
  output: "export",
  trailingSlash: false,
  images: { unoptimized: true },
  typescript: { ignoreBuildErrors: true },
  poweredByHeader: false,
};

export default config;
