"use client";

import { useEffect, useState } from "react";

import { DOWNLOAD_PAGE, DOWNLOADS, type System } from "../../lib/data/release.ts";
import { SITE } from "../../lib/data/site.ts";
import { ButtonLink } from "../Button.tsx";
import { DownloadButton, detectSystem } from "../DownloadButton.tsx";
import { InlineCode } from "./InlineCode.tsx";

function fileFor(system: System, index = 0): string {
  return DOWNLOADS.find((download) => download.system === system)?.files[index]?.name ?? "";
}

/** What the visitor's system needs, said with the file's name. */
function Need({ system }: { system: System }) {
  const mark = (name: string) => <InlineCode>{name}</InlineCode>;
  switch (system) {
    case "macos":
      return <>You are on macOS, so you need one file: {mark(fileFor("macos"))}, which runs on Apple silicon and Intel.</>;
    case "windows":
      return <>You are on Windows, so you need one file: {mark(fileFor("windows"))}, the installer.</>;
    case "linux":
      return (
        <>
          You are on Linux, so you need {mark(fileFor("linux", 0))} for Debian or Ubuntu, or {mark(fileFor("linux", 1))} for
          any other distribution.
        </>
      );
  }
}

/**
 * The answer to "which file?" for the system the visitor is on, once the page knows it, and the
 * primary action, to the releases page where the files are. Before then, and on a phone, it
 * offers the page's own list.
 */
export function SystemDownload() {
  const [system, setSystem] = useState<System | null>(null);
  useEffect(() => setSystem(detectSystem()), []);
  return (
    <>
      <p className="m-0 mt-5 max-w-[36ch] text-[1.25rem] leading-[1.4] text-ink-2">
        {system ? (
          <Need system={system} />
        ) : (
          <>QuotaDial comes from its GitHub releases, one file for each system. Pick yours from the table.</>
        )}
      </p>
      <div className="mt-7 flex flex-wrap items-center gap-x-6 gap-y-4">
        <DownloadButton href={DOWNLOAD_PAGE} />
        <ButtonLink variant="text" href={SITE.installGuide}>
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
        Each release after 0.2.0 lists every file's SHA-256, so you can check what you downloaded. QuotaDial Free needs no key.{" "}
        <a href="/pro" className="text-ink underline underline-offset-[0.2em]">
          See what Pro adds
        </a>
        .
      </p>
    </>
  );
}
