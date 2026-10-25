"use client";

import { DownloadSimple } from "@phosphor-icons/react";
import { useEffect, useState } from "react";

import type { System } from "../lib/data/release.ts";

const NAMES: Record<System, string> = { macos: "macOS", windows: "Windows", linux: "Linux" };

/** The visitor's system, as well as the browser says it; null on a phone or when it cannot tell. */
export function detectSystem(): System | null {
  const agent = navigator.userAgent;
  if (/Mac OS X|Macintosh/.test(agent) && !/iPhone|iPad/.test(agent)) {
    return "macos";
  }
  if (/Windows/.test(agent)) {
    return "windows";
  }
  if (/Linux|X11/.test(agent) && !/Android/.test(agent)) {
    return "linux";
  }
  return null;
}

/**
 * The download action, named for the visitor's system once the page knows it ("Download QuotaDial"
 * before then, and on a phone), leading to that system's part of /download, or to `href` when given
 * (on /download itself, the releases page).
 */
export function DownloadButton({ primary = true, href }: { primary?: boolean; href?: string }) {
  const [system, setSystem] = useState<System | null>(null);
  useEffect(() => setSystem(detectSystem()), []);
  return (
    <a
      href={href ?? (system ? `/download#${system}` : "/download")}
      data-primary-action={primary ? "" : undefined}
      data-event="download_click"
      data-event-system={system ?? "unknown"}
      className="inline-flex items-center gap-2 rounded-chart bg-magenta-fill px-5 py-3 font-display text-[1.125rem] leading-none font-[650] whitespace-nowrap text-on-magenta no-underline transition-colors hover:bg-[color-mix(in_srgb,var(--magenta-fill)_86%,black)] active:translate-y-px"
    >
      <DownloadSimple size={20} aria-hidden />
      {system ? `Download for ${NAMES[system]}` : "Download QuotaDial"}
    </a>
  );
}
