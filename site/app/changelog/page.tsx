import type { Metadata } from "next";

import { ButtonLink } from "../../components/Button.tsx";
import { DownloadButton } from "../../components/DownloadButton.tsx";
import { Section } from "../../components/Section.tsx";
import { PageIntro } from "../../components/reference/PageIntro.tsx";
import { ReleaseTable } from "../../components/reference/ReleaseTable.tsx";
import { Versions } from "../../components/reference/Versions.tsx";
import { SITE } from "../../lib/data/site.ts";
import { changelog, repositoryFile } from "../../lib/markdown.ts";
import { pageMetadata } from "../../lib/metadata.ts";

export const metadata: Metadata = pageMetadata({
  title: "Changelog: what changed in each version",
  description:
    "Every change people will notice in QuotaDial, newest first: what each version added, changed and fixed, from the changelog the app itself shows.",
  path: "/changelog",
});

export default function Changelog() {
  const { versions } = changelog(repositoryFile("CHANGELOG.md"));
  return (
    <>
      <Section>
        <div className="grid gap-14 lg:grid-cols-12 lg:items-start lg:gap-x-12">
          <div className="lg:col-span-6">
            <PageIntro
              title="What changed in QuotaDial"
              answer="Every change you will notice, newest first. Help in the app shows the same list under What's new."
            >
              <DownloadButton />
              <ButtonLink href={SITE.releases} variant="text">
                Read the release notes on GitHub
              </ButtonLink>
            </PageIntro>
          </div>
          <div className="lg:col-span-6 lg:col-start-7">
            <ReleaseTable versions={versions} />
          </div>
        </div>
        <div className="mt-20 md:mt-28">
          <Versions versions={versions} />
        </div>
      </Section>
      <Section tone="shallow" labelledBy="changelog-next">
        <h2 id="changelog-next" className="m-0 text-[2rem] leading-[1.05] font-[750] md:text-[2.75rem]">
          If you already have QuotaDial
        </h2>
        <p className="mt-4 mb-0 max-w-measure text-[1.1875rem]">
          Every version after 0.2.0 asks GitHub once a day for a newer full release, offers it in the popover and in
          Help, and installs it only when you choose Update. Versions 0.1.0 and 0.2.0 do not check, so download the
          newer version here.
        </p>
        <div className="mt-8">
          <DownloadButton primary={false} />
        </div>
      </Section>
    </>
  );
}
