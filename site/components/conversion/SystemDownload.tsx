"use client";

import { ArrowDown, DownloadSimple } from "@phosphor-icons/react";
import { useEffect, useState } from "react";

import type { System } from "../../lib/data/release.ts";
import { ButtonLink } from "../Button.tsx";
import { detectSystem } from "../DownloadButton.tsx";
import { type AppFile, buttonLabel, clickDetail, filesFor } from "./download-links.ts";
import { InlineCode } from "./InlineCode.tsx";

/** What the visitor's system needs, said with the files' names. */
function Need({ system, own }: { system: System; own: AppFile[] }) {
  const mark = (file: AppFile) => <InlineCode>{file.name}</InlineCode>;
  switch (system) {
    case "macos":
      return <>You are on macOS, so you need one file: {mark(own[0])}, which runs on Apple silicon and Intel.</>;
    case "windows":
      return <>You are on Windows, so you need one file: {mark(own[0])}, the installer.</>;
    case "linux":
      return (
        <>
          You are on Linux, so you need {mark(own[0])} for Debian or Ubuntu, or {mark(own[1])} for any other
          distribution.
        </>
      );
  }
}

/**
 * The answer to "which file?" for the system the visitor is on, once the page knows it, and the
 * primary action, which fetches that file. Before then, and on a phone, where no desktop app can
 * be installed, it offers the page's own list instead of a file that may not suit.
 */
export function SystemDownload({ files }: { files: AppFile[] }) {
  const [system, setSystem] = useState<System | null>(null);
  useEffect(() => setSystem(detectSystem()), []);
  const own = system ? filesFor(files, system) : [];
  return (
    <>
      <p className="m-0 mt-5 max-w-[36ch] text-[1.25rem] leading-[1.4] text-ink-2">
        {system ? (
          <Need system={system} own={own} />
        ) : (
          <>QuotaDial comes from its GitHub releases, one file for each system. Choose yours.</>
        )}
      </p>
      <div className="mt-7 flex flex-wrap items-center gap-x-4 gap-y-4">
        {system ? (
          own.map((file, index) => (
            <ButtonLink
              key={file.name}
              variant={index === 0 ? "primary" : "secondary"}
              href={file.href}
              data-primary-action={index === 0 ? "" : undefined}
              {...clickDetail(file, "hero")}
            >
              {index === 0 && <DownloadSimple size={20} aria-hidden />}
              {buttonLabel(file, own)}
            </ButtonLink>
          ))
        ) : (
          <ButtonLink variant="primary" href="#systems" data-primary-action="">
            <ArrowDown size={20} aria-hidden />
            Choose your system
          </ButtonLink>
        )}
        <ButtonLink variant="text" href="/install">
          Read the install guide
        </ButtonLink>
      </div>
      <p className="m-0 mt-5 max-w-measure">
        Builds are not signed yet, so your system warns you the first time.{" "}
        <a href="#unsigned" className="underline underline-offset-[0.2em]">
          The steps are below
        </a>
        .
      </p>
      <p className="m-0 mt-3 max-w-measure text-ink-2">
        Each release after 0.2.0 lists every file's SHA-256, so you can check what you downloaded. QuotaDial Free needs no
        key.{" "}
        <a href="/pro" className="text-ink underline underline-offset-[0.2em]">
          See what Pro adds
        </a>
        .
      </p>
    </>
  );
}
