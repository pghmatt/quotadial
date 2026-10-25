import type { ReactNode } from "react";

import { CodeBlock } from "../CodeBlock.tsx";
import { BandedTable } from "../home/BandedTable.tsx";
import { Prose } from "../Prose.tsx";
import type { Guide, GuideBlock, GuideSection } from "./guide.ts";

/** Code, links and bold inside a table's cells, set as the reading text sets them; a long word breaks only after the marks `finish` puts in it, and only when the cell is narrower than the word. */
const CELL =
  "[&_a]:underline [&_a]:underline-offset-[0.2em] [&_code]:rounded-chart [&_code]:bg-shallow [&_code]:px-1 [&_code]:font-code [&_code]:text-[0.875em] [&_code]:[overflow-wrap:break-word]";

/**
 * From 768 px the cells give up 4 px of side padding each, so /install's archive table holds its two
 * longest names (`quotadial-v<version>-x86_64-unknown-linux-gnu.tar.gz` and the copy without the
 * version) whole in one row from a 1280 px window up, where they would otherwise break in two at
 * 1440 and 1920 for want of 11 px.
 */
const TABLE = "md:[&_td]:px-2 md:[&_th]:px-2";

/** The words of a table cell's HTML, to judge whether a first column is short enough to keep on one line. */
const cellWords = (cell: string) => cell.replace(/<[^>]+>/g, "").replace(/&[a-z#0-9]+;/g, "x");
/** A first column of names (a command, a request, a setting) stays on one line; one of sentences wraps. */
const SHORT = 36;

/** One block of a guide's section, in the site's own components. */
function Block({ block, id }: { block: GuideBlock; id: string }) {
  if (block.kind === "html") {
    return <Prose html={block.html} className="[&>:first-child]:mt-0! [&>:last-child]:mb-0! [&_code]:[overflow-wrap:break-word]" />;
  }
  if (block.kind === "table") {
    return (
      <BandedTable
        caption={block.caption}
        className={TABLE}
        columns={block.columns}
        wholeFirst={block.rows.every((cells) => cellWords(cells[0]).length <= SHORT)}
        rows={block.rows.map((cells, row) => ({
          key: `${id}-${row}`,
          cells: cells.map((cell) => (cell ? <span className={CELL} dangerouslySetInnerHTML={{ __html: cell }} /> : "")),
        }))}
      />
    );
  }
  return (
    <div className={block.nested ? "md:pl-6" : undefined}>
      {block.command ? (
        <CodeBlock code={block.code} label="Copy the command" />
      ) : (
        <CodeBlock code={block.code} copy={false} wrap={false} name={block.name} />
      )}
    </div>
  );
}

/** A guide's section: its heading in a column of its own from 1024 px, with what it says beside it. */
function Section({ section }: { section: GuideSection }) {
  const { heading } = section;
  return (
    <section
      aria-labelledby={heading.id}
      className="grid gap-6 border-t border-rule pt-10 md:pt-14 lg:grid-cols-[14rem_minmax(0,1fr)] lg:gap-10"
    >
      <h2
        id={heading.id}
        className="m-0 scroll-mt-6 text-[2rem] leading-[1.05] font-[750] lg:sticky lg:top-6 lg:self-start"
        dangerouslySetInnerHTML={{ __html: heading.html }}
      />
      <div className="grid min-w-0 grid-cols-1 gap-5">
        {section.blocks.map((block, index) => (
          <Block key={index} block={block} id={`${heading.id}-${index}`} />
        ))}
      </div>
    </section>
  );
}

/** Every section of a guide, in the file's order. */
export function GuideSections({ guide }: { guide: Guide }) {
  return (
    <div className="grid gap-14 md:gap-16">
      {guide.sections.map((section) => (
        <Section key={section.heading.id} section={section} />
      ))}
    </div>
  );
}

/** The guide's sections as links, at its head, in the reading face. */
export function Contents({ guide, caption }: { guide: Guide; caption: ReactNode }) {
  return (
    <nav aria-label="In this guide">
      <p aria-hidden className="m-0 mb-3 font-display text-[1rem] leading-[1.3] font-[650] text-ink-2 italic">
        In this guide
      </p>
      <ul className="m-0 list-none columns-1 gap-x-10 p-0 sm:columns-2">
        {guide.sections.map((section) => (
          <li key={section.heading.id} className="break-inside-avoid py-1 text-[1.0625rem] leading-[1.35]">
            <a
              href={`#${section.heading.id}`}
              className="underline decoration-1 underline-offset-[0.2em] hover:decoration-2 [&_code]:rounded-chart [&_code]:bg-shallow [&_code]:px-1 [&_code]:font-code [&_code]:text-[0.875em]"
              dangerouslySetInnerHTML={{ __html: section.heading.html }}
            />
          </li>
        ))}
      </ul>
      <p className="mt-4 mb-0 max-w-measure font-table text-[0.9375rem] text-ink-2 italic">{caption}</p>
    </nav>
  );
}
