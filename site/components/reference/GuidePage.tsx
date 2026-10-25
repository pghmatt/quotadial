import type { ReactNode } from "react";

import { ButtonLink } from "../Button.tsx";
import { DownloadButton } from "../DownloadButton.tsx";
import { Section } from "../Section.tsx";
import { Contents, GuideSections } from "./Guide.tsx";
import { InlineLink } from "./InlineLink.tsx";
import { PageIntro } from "./PageIntro.tsx";
import type { Guide } from "./guide.ts";
import { SITE } from "../../lib/data/site.ts";

/** The other guide, which each guide links to at its head and at its close. */
export interface OtherGuide {
  href: string;
  /** What the link says, such as "Read the install guide". */
  label: string;
}

/**
 * A guide from the repository as a page: its own heading as the H1, its own opening paragraph as
 * the answer, the primary Download action and a link to the other guide, the sections as links,
 * then every section, and a closing band with the action again. Nothing the guide says is changed.
 */
export function GuidePage({
  guide,
  file,
  answer,
  other,
  closing,
}: {
  guide: Guide;
  /** What the page is for, in the site's own words (the guide's own opening follows, word for word, under the contents). */
  answer: string;
  /** The file in the repository, such as "docs/INSTALL.md". */
  file: string;
  other: OtherGuide;
  closing: { heading: string; text: ReactNode };
}) {
  const id = `${file.replace(/\W+/g, "-").toLowerCase()}-next`;
  return (
    <>
      <Section>
        <div className="grid gap-12 lg:grid-cols-12 lg:gap-x-12">
          <div className="lg:col-span-5">
            <PageIntro
              title={guide.title}
              answer={answer}
            >
              <DownloadButton />
              <ButtonLink href={other.href} variant="text">
                {other.label}
              </ButtonLink>
            </PageIntro>
          </div>
          <div className="lg:col-span-7 lg:mt-3">
            <Contents
              guide={guide}
              caption={
                <>
                  This page is <InlineLink href={`${SITE.repository}/blob/main/${file}`}>{file}</InlineLink> from the
                  repository, word for word.
                </>
              }
            />
          </div>
        </div>
        <div className="mt-16 grid gap-14 md:mt-20 md:gap-16">
          {/* The guide's own opening, in the column its sections' text uses. */}
          <div className="lg:grid lg:grid-cols-[14rem_minmax(0,1fr)] lg:gap-10">
            <p
              className="m-0 max-w-measure text-[1.25rem] leading-[1.5] lg:col-start-2 [&_code]:rounded-chart [&_code]:bg-shallow [&_code]:px-1 [&_code]:font-code [&_code]:text-[0.875em]"
              dangerouslySetInnerHTML={{ __html: guide.intro }}
            />
          </div>
          <GuideSections guide={guide} />
        </div>
      </Section>
      <Section tone="shallow" labelledBy={id}>
        <h2 id={id} className="m-0 text-[2rem] leading-[1.05] font-[750] md:text-[2.75rem]">
          {closing.heading}
        </h2>
        <p className="mt-4 mb-0 max-w-[48ch] text-[1.1875rem]">{closing.text}</p>
        <div className="mt-8 flex flex-wrap items-center gap-x-6 gap-y-4">
          <DownloadButton primary={false} />
          <ButtonLink href={other.href} variant="text">
            {other.label}
          </ButtonLink>
        </div>
      </Section>
    </>
  );
}
