"use client";

import { Check, Copy } from "@phosphor-icons/react";
import { useState } from "react";

import { type EventName, track } from "../lib/events.ts";

/** Copies `text`, says so for a moment, and records `event` if given. */
export function CopyButton({
  text,
  label,
  event,
  primary = false,
}: {
  text: string;
  /** Names what is copied, such as "Copy the key". */
  label: string;
  event?: EventName;
  primary?: boolean;
}) {
  const [copied, setCopied] = useState(false);
  const [failed, setFailed] = useState(false);
  const copy = async () => {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
      setFailed(false);
      if (event) {
        track(event);
      }
      window.setTimeout(() => setCopied(false), 2000);
    } catch {
      setFailed(true);
    }
  };
  return (
    <span className="inline-flex items-center gap-3">
      <button
        type="button"
        onClick={copy}
        data-primary-action={primary ? "" : undefined}
        className={`inline-flex items-center gap-2 rounded-chart px-4 py-2 font-display text-[1.0625rem] font-[650] ${primary ? "bg-magenta-fill text-on-magenta" : "border border-current"}`}
      >
        {copied ? <Check size={18} aria-hidden /> : <Copy size={18} aria-hidden />}
        {copied ? "Copied" : label}
      </button>
      <span role="status" className="text-[0.9375rem] text-ink-2">
        {failed ? "Your browser would not copy it; select the text and copy it instead." : ""}
      </span>
    </span>
  );
}
