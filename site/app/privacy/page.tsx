import type { Metadata } from "next";

import { ButtonLink } from "../../components/Button.tsx";
import { DownloadButton } from "../../components/DownloadButton.tsx";
import { Prose } from "../../components/Prose.tsx";
import { Section } from "../../components/Section.tsx";
import { InlineLink } from "../../components/reference/InlineLink.tsx";
import { PageIntro } from "../../components/reference/PageIntro.tsx";
import { WebsiteNote } from "../../components/reference/WebsiteNote.tsx";
import { setAddresses } from "../../components/reference/text.ts";
import { SITE } from "../../lib/data/site.ts";
import { html, repositoryFile, withoutTitle } from "../../lib/markdown.ts";
import { pageMetadata } from "../../lib/metadata.ts";

export const metadata: Metadata = pageMetadata({
  title: "What QuotaDial reads and sends",
  description:
    "Every address QuotaDial connects to, and nothing else: Anthropic and ChatGPT for your limits, their status pages, GitHub for updates, and webhooks you add.",
  path: "/privacy",
});

/** docs/PRIVACY.md, the one privacy text (D-106), as the page shows it: its own heading is the H1. */
function privacyText() {
  const { title, body } = withoutTitle(repositoryFile("docs/PRIVACY.md"));
  return { title, html: setAddresses(html(body)) };
}

export default function Privacy() {
  const text = privacyText();
  return (
    <>
      <Section>
        <div className="grid gap-10 lg:grid-cols-12 lg:gap-x-12">
          <div className="lg:col-span-7">
            <PageIntro
              title={text.title}
              answer={
                <>
                  This is the text that Help in the app and the README show, word for word. See for yourself in{" "}
                  <InlineLink href={`${SITE.repository}/blob/main/docs/PRIVACY.md`}>docs/PRIVACY.md</InlineLink> on
                  GitHub.
                </>
              }
            >
              <DownloadButton />
              <ButtonLink href="/support" variant="text">
                Write to support
              </ButtonLink>
            </PageIntro>
          </div>
          <div className="lg:col-span-5 lg:col-start-8">
            <WebsiteNote />
          </div>
        </div>
        <Prose
          html={text.html}
          className="mt-16 max-w-none! md:mt-20 lg:columns-2 lg:gap-x-12 [&>p:first-child]:mt-0 [&>p:first-child]:max-w-[62ch] [&>p:first-child]:text-[1.25rem] [&>p:first-child]:[column-span:all] [&_code.address]:font-[600] [&_li]:break-inside-avoid [&_p]:break-inside-avoid [&_ul]:break-inside-avoid [&_p:has(+ul)]:break-after-avoid"
        />
      </Section>
      <Section tone="shallow" labelledBy="privacy-next">
        <h2 id="privacy-next" className="m-0 text-[2rem] leading-[1.05] font-[750] md:text-[2.75rem]">
          When you have read it
        </h2>
        <p className="mt-4 mb-0 max-w-[48ch] text-[1.1875rem]">
          QuotaDial Free needs no account and no key, no strings attached. If a line above is unclear, ask support
          before you install.
        </p>
        <div className="mt-8 flex flex-wrap items-center gap-x-6 gap-y-4">
          <DownloadButton primary={false} />
          <ButtonLink href="/support" variant="text">
            Write to support
          </ButtonLink>
        </div>
      </Section>
    </>
  );
}
