import { DOWNLOAD_PAGE, DOWNLOADS, type System } from "../../lib/data/release.ts";
import { ButtonLink } from "../Button.tsx";

/**
 * A system's own way to the files, for the visitor who arrives at that system's part of the page
 * (such as /download#macos) and would otherwise find instructions with no download in view. A
 * secondary action: only the hero's is the primary one.
 */
export function SystemAction({ system }: { system: System }) {
  const label = DOWNLOADS.find((download) => download.system === system)?.label ?? system;
  return (
    <div className="mt-5">
      <ButtonLink variant="secondary" href={DOWNLOAD_PAGE} data-event="download_click" data-event-system={system} data-event-from="system-section">
        Download for {label}
      </ButtonLink>
    </div>
  );
}
