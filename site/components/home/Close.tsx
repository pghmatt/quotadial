import { DownloadButton } from "../DownloadButton.tsx";
import { Section } from "../Section.tsx";

/** The close, on the deep: the download action again, the same event, without the primary mark. */
export function Close() {
  return (
    <Section tone="deep" labelledBy="close-title">
      <div className="flex flex-col gap-8 md:flex-row md:items-center md:justify-between">
        <p
          id="close-title"
          className="m-0 max-w-[22ch] font-display text-[2.25rem] leading-[1.02] font-[750] tracking-[-0.01em] md:text-[3rem]"
        >
          Put every limit and its reset in your menu bar or tray.
        </p>
        <div>
          <DownloadButton primary={false} />
          <p className="m-0 mt-4 text-on-deep-2">Free software under the MIT licence, for macOS, Windows and Linux.</p>
        </div>
      </div>
    </Section>
  );
}
