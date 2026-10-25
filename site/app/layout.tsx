import "@fontsource-variable/sofia-sans";
import "@fontsource-variable/sofia-sans/wght-italic.css";
import "@fontsource-variable/sofia-sans-condensed";
import "@fontsource-variable/sofia-sans-condensed/wght-italic.css";
import "@fontsource-variable/sofia-sans-semi-condensed";
import "@fontsource-variable/jetbrains-mono";
import "./globals.css";

import type { Metadata, Viewport } from "next";
import type { ReactNode } from "react";

import { EventBridge } from "../components/EventBridge.tsx";
import { Footer } from "../components/Footer.tsx";
import { Header } from "../components/Header.tsx";
import { SITE } from "../lib/data/site.ts";

export const metadata: Metadata = {
  metadataBase: new URL(SITE.origin),
  applicationName: SITE.name,
};

export const viewport: Viewport = {
  width: "device-width",
  initialScale: 1,
  themeColor: [
    { media: "(prefers-color-scheme: light)", color: "#f6f9fb" },
    { media: "(prefers-color-scheme: dark)", color: "#08131d" },
  ],
};

export default function RootLayout({ children }: { children: ReactNode }) {
  return (
    <html lang="en-GB">
      <body>
        <a
          href="#content"
          className="sr-only rounded-chart bg-ink px-4 py-2 text-chart focus:not-sr-only focus:absolute focus:top-3 focus:left-3 focus:z-30"
        >
          Skip to the content
        </a>
        <Header />
        <main id="content">{children}</main>
        <Footer />
        <EventBridge />
      </body>
    </html>
  );
}
