import type { System } from "../../lib/data/release.ts";
import { ButtonLink } from "../Button.tsx";
import { type AppFile, buttonLabel, clickDetail, filesFor } from "./download-links.ts";
import { InlineCode } from "./InlineCode.tsx";

/**
 * A system's own download, for the visitor who arrives at that system's part of the page (such as
 * /download#macos) and would otherwise find instructions with no download in view. Each button
 * fetches its file's version-less copy. Secondary actions: only the hero's is the primary one. They
 * are in the page's HTML, so they work with no script.
 */
export function SystemAction({ system, files }: { system: System; files: AppFile[] }) {
  const own = filesFor(files, system);
  return (
    <>
      <div className="mt-5 flex flex-wrap items-center gap-x-4 gap-y-3">
        {own.map((file) => (
          <ButtonLink key={file.name} variant="secondary" href={file.href} {...clickDetail(file, "system-section")}>
            {buttonLabel(file, own)}
          </ButtonLink>
        ))}
      </div>
      <p className="m-0 mt-3 max-w-measure text-ink-2">
        Saves as{" "}
        {own.map((file, index) => (
          <span key={file.name}>
            {index > 0 && " or "}
            <InlineCode on="shallow">{file.name}</InlineCode>
          </span>
        ))}
        .
      </p>
    </>
  );
}
