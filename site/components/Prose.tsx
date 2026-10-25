import { markBreaks } from "../lib/break-html.ts";

/** The reading style: the reading measure, with room between paragraphs and list items. */
const TEXT =
  "max-w-measure text-[1.0625rem] leading-[1.6] [&_p]:my-4 [&_li]:mt-2 [&_strong]:font-[700] [&_a]:underline [&_a]:underline-offset-[0.2em]";
/** Lists: bullets, and numbers where the order is the information (steps). */
const LISTS = "[&_ul]:my-4 [&_ul]:list-disc [&_ul]:pl-6 [&_ol]:my-4 [&_ol]:list-decimal [&_ol]:pl-6";
/** A section's own headings, in the display face the base styles give h2 and h3. */
const HEADINGS =
  "[&_h2]:mt-10 [&_h2]:mb-3 [&_h2]:text-[1.75rem] [&_h2]:leading-[1.1] [&_h2]:font-[750] [&_h3]:mt-8 [&_h3]:mb-2 [&_h3]:text-[1.375rem] [&_h3]:leading-[1.15] [&_h3]:font-[700]";
/** Code and addresses in the code face; a block of commands wraps, at a space or a mark, rather than scrolling. */
const CODE =
  "[&_code]:rounded-chart [&_code]:bg-shallow [&_code]:px-1 [&_code]:font-code [&_code]:text-[0.875em] [&_pre]:my-4 [&_pre]:rounded-chart [&_pre]:bg-shallow [&_pre]:p-4 [&_pre]:whitespace-pre-wrap [&_pre]:[overflow-wrap:break-word] [&_pre_code]:bg-transparent [&_pre_code]:p-0";

/**
 * HTML rendered from the repository's own Markdown (lib/markdown.ts), in the site's reading style.
 * The words of its code and links break only at a space or a mark (`markBreaks`).
 */
export function Prose({ html, className = "" }: { html: string; className?: string }) {
  return (
    <div className={`${TEXT} ${LISTS} ${HEADINGS} ${CODE} ${className}`} dangerouslySetInnerHTML={{ __html: markBreaks(html) }} />
  );
}
