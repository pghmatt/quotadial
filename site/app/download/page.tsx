import type { Metadata } from "next";

import { CodeBlock } from "../../components/CodeBlock.tsx";
import { CommandTable } from "../../components/conversion/CommandTable.tsx";
import { appFiles, commandFiles } from "../../components/conversion/download-links.ts";
import { FilesTable } from "../../components/conversion/FilesTable.tsx";
import { InlineCode } from "../../components/conversion/InlineCode.tsx";
import { SystemAction } from "../../components/conversion/SystemAction.tsx";
import { SystemDownload } from "../../components/conversion/SystemDownload.tsx";
import { Notice } from "../../components/Notice.tsx";
import { Prose } from "../../components/Prose.tsx";
import { Plate } from "../../components/Plate.tsx";
import { Section, SectionHeading } from "../../components/Section.tsx";
import { SITE } from "../../lib/data/site.ts";
import { installGuide } from "../../lib/markdown.ts";
import { jsonLd, pageMetadata, softwareJsonLd } from "../../lib/metadata.ts";

export const metadata: Metadata = pageMetadata({
  title: "Download for macOS, Windows and Linux",
  description:
    "Download QuotaDial for macOS, Windows or Linux from its GitHub releases, check the file against its SHA-256, and open a build that is not signed yet.",
  path: "/download",
});

const H3 = "m-0 text-[1.375rem] leading-[1.15] font-[700]";
const LINK = "underline underline-offset-[0.2em]";

/** What the page shows of docs/INSTALL.md, read when the site is built and never copied. */
const GUIDE = installGuide();
const FILES = appFiles(GUIDE.files);
const COMMANDS = commandFiles(GUIDE.commands);

export default function Download() {
  return (
    <>
      <script type="application/ld+json" dangerouslySetInnerHTML={{ __html: jsonLd(softwareJsonLd()) }} />

      <Section>
        <div className="grid items-start gap-x-6 gap-y-10 grid-cols-[minmax(0,1fr)] md:grid-cols-12">
          <div className="md:col-span-6 lg:col-span-5">
            <h1 className="m-0 text-[2.75rem] leading-[1.02] font-[800] md:text-[4rem]">Download QuotaDial</h1>
            <SystemDownload files={FILES} />
          </div>
          <div className="md:col-span-6 lg:col-span-7">
            <Plate
              title="Files in each release"
              caption="The files a release holds, with the version in their names. Replace <version> with the number in the release's name."
            >
              <FilesTable files={FILES} releases={SITE.releases} />
            </Plate>
          </div>
        </div>
      </Section>

      <Section tone="shallow">
        <div id="systems" className="grid scroll-mt-8 grid-cols-[minmax(0,1fr)] gap-x-6 gap-y-14 md:grid-cols-12">
          <div id="macos" className="md:col-span-4">
            <SectionHeading id="macos-heading">macOS</SectionHeading>
            <p className="mt-4 mb-0 max-w-[38ch]">
              One disk image covers Apple silicon and Intel. Drag QuotaDial to Applications, then open it. macOS will
              not open it until you choose Open Anyway in System Settings.{" "}
              <a href="#unsigned" className={LINK}>
                The steps are below
              </a>
              .
            </p>
            <SystemAction system="macos" files={FILES} />
          </div>

          <div id="windows" className="md:col-span-3">
            <SectionHeading id="windows-heading">Windows</SectionHeading>
            <p className="mt-4 mb-0 max-w-[34ch]">
              Run the installer. Edge or Windows may warn you about a build that is not signed.{" "}
              <a href="#unsigned" className={LINK}>
                The steps are below
              </a>
              .
            </p>
            <SystemAction system="windows" files={FILES} />
          </div>

          <div id="linux" className="md:col-span-5">
            <SectionHeading id="linux-heading">Linux</SectionHeading>
            <p className="mt-4 mb-0 max-w-measure">
              Take the <InlineCode on="shallow">.deb</InlineCode> for Debian or Ubuntu, and the{" "}
              <InlineCode on="shallow">.AppImage</InlineCode> for any other distribution. The commands that install and
              run them are in{" "}
              <a href="#unsigned" className={LINK}>
                the steps below
              </a>
              .
            </p>
            <SystemAction system="linux" files={FILES} />
          </div>
        </div>

        <div className="mt-16 grid grid-cols-[minmax(0,1fr)] gap-x-6 gap-y-10 border-t border-rule pt-16 md:grid-cols-12">
          <div className="md:col-span-5">
            <SectionHeading id="command-heading">The command on its own</SectionHeading>
            <p className="mt-4 mb-0 max-w-measure">
              If a terminal is all you want, take the archive for your system. Each holds{" "}
              <InlineCode on="shallow">quotadial</InlineCode> and its short alias <InlineCode on="shallow">qd</InlineCode>,
              with shell completions and manual pages.
            </p>
            <p className="mt-4 mb-0 max-w-measure">
              On macOS or Linux, these lines fetch the newest full release's command and put it on your PATH. The first
              names Apple silicon, so use your system's name from the table for any other.
            </p>
            <div className="mt-6">
              <CodeBlock code={GUIDE.install} label="Copy the commands" />
            </div>
            <p className="mt-6 mb-0 max-w-measure">
              The app carries the same command inside it.{" "}
              <a href="/install" className={LINK}>
                The install guide
              </a>{" "}
              covers package managers, QuotaDial's settings and its updates.
            </p>
          </div>

          <div className="md:col-span-7">
            <CommandTable commands={COMMANDS} />
          </div>
        </div>

        <div className="mt-16 grid grid-cols-[minmax(0,1fr)] gap-x-6 gap-y-10 border-t border-rule pt-16 md:grid-cols-12">
          <div className="md:col-span-5">
            <SectionHeading id="check-heading">Check a download</SectionHeading>
            <p className="mt-4 mb-0 max-w-measure">
              <InlineCode on="shallow">SHA256SUMS.txt</InlineCode> lists every file's SHA-256, in each release after
              0.2.0, under both its names, so the check passes whichever you took. In the folder you downloaded to, paste
              the line for your system. It prints OK beside each file that matches, and FAILED beside any that does not.
            </p>
          </div>

          <div className="md:col-span-7">
            <div className="grid grid-cols-[minmax(0,1fr)] gap-5">
              {GUIDE.checks.map((check) => (
                <div key={check.system} className="grid grid-cols-[minmax(0,1fr)] gap-2">
                  <h3 className={H3}>On {check.system}</h3>
                  {check.system === "Windows" && (
                    <p className="m-0 max-w-measure">
                      PowerShell prints the hash. Compare it with the line for that file in{" "}
                      <InlineCode on="shallow">SHA256SUMS.txt</InlineCode>.
                    </p>
                  )}
                  <CodeBlock code={check.command} label="Copy the command" />
                </div>
              ))}
            </div>
          </div>
        </div>
      </Section>

      <Section tone="mid" id="unsigned" labelledBy="unsigned-heading">
        <div className="grid items-start gap-x-6 gap-y-10 grid-cols-[minmax(0,1fr)] md:grid-cols-12">
          <div className="md:sticky md:top-8 md:col-span-5">
            <SectionHeading id="unsigned-heading">Opening a build that is not signed yet</SectionHeading>
            <div className="mt-6">
              <Notice title="Today's builds are not signed">
                <p className="m-0 text-magenta">
                  Follow the steps for your system, and only for a build from QuotaDial's GitHub.
                </p>
              </Notice>
            </div>
            <p className="mt-6 mb-0 max-w-measure">
              A warning that is not described here?{" "}
              <a href="/support" className={LINK}>
                Write to support
              </a>{" "}
              with what it says.
            </p>
          </div>

          <Prose html={GUIDE.unsigned} className="md:col-span-7" />
        </div>
      </Section>
    </>
  );
}
