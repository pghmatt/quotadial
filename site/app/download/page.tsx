import type { Metadata } from "next";

import { CodeBlock } from "../../components/CodeBlock.tsx";
import { InlineCode } from "../../components/conversion/InlineCode.tsx";
import { FilesTable } from "../../components/conversion/FilesTable.tsx";
import { Steps } from "../../components/conversion/Steps.tsx";
import { SystemDownload } from "../../components/conversion/SystemDownload.tsx";
import { Notice } from "../../components/Notice.tsx";
import { Plate } from "../../components/Plate.tsx";
import { Section, SectionHeading } from "../../components/Section.tsx";
import { COMMAND_ONLY } from "../../lib/data/release.ts";
import { SITE } from "../../lib/data/site.ts";
import { jsonLd, pageMetadata, softwareJsonLd } from "../../lib/metadata.ts";

export const metadata: Metadata = pageMetadata({
  title: "Download for macOS, Windows and Linux",
  description:
    "Download QuotaDial for macOS, Windows or Linux from its GitHub releases, check the file against its SHA-256, and open a build that is not signed yet.",
  path: "/download",
});

const H3 = "m-0 text-[1.375rem] leading-[1.15] font-[700]";
const LINK = "underline underline-offset-[0.2em]";

export default function Download() {
  return (
    <>
      <script type="application/ld+json" dangerouslySetInnerHTML={{ __html: jsonLd(softwareJsonLd()) }} />

      <Section>
        <div className="grid items-start gap-x-6 gap-y-10 grid-cols-[minmax(0,1fr)] md:grid-cols-12">
          <div className="md:col-span-6 lg:col-span-5">
            <h1 className="m-0 text-[2.75rem] leading-[1.02] font-[800] md:text-[4rem]">Download QuotaDial</h1>
            <SystemDownload />
          </div>
          <div className="md:col-span-6 lg:col-span-7">
            <Plate caption="The files a release holds. Replace <version> with the number in the release's name.">
              <FilesTable />
            </Plate>
          </div>
        </div>
      </Section>

      <Section tone="shallow">
        <div className="grid grid-cols-[minmax(0,1fr)] gap-x-6 gap-y-14 md:grid-cols-12">
          <div id="macos" className="md:col-span-4">
            <SectionHeading id="macos-heading">macOS</SectionHeading>
            <p className="mt-4 mb-0 max-w-[38ch]">
              Open the disk image and drag QuotaDial to Applications. A dial appears in the menu bar with the share
              used beside it, and a click on it shows every account and limit.
            </p>
            <p className="mt-4 mb-0 max-w-[38ch]">
              QuotaDial has no Dock icon. Quit it from its menu, or from the bottom of the window.
            </p>
            <p className="mt-4 mb-0 max-w-[38ch]">
              The first time you open it, macOS asks you to confirm it.{" "}
              <a href="#unsigned" className={LINK}>
                The steps are below
              </a>
              .
            </p>
          </div>

          <div id="windows" className="md:col-span-3">
            <SectionHeading id="windows-heading">Windows</SectionHeading>
            <p className="mt-4 mb-0 max-w-[34ch]">
              Run the installer. QuotaDial's number appears in the notification area. If Windows tucks it under the ^
              arrow, drag it onto the taskbar to keep it in view.
            </p>
            <p className="mt-4 mb-0 max-w-[34ch]">
              Edge or Windows may warn you about a build that is not signed.{" "}
              <a href="#unsigned" className={LINK}>
                The steps are below
              </a>
              .
            </p>
          </div>

          <div id="linux" className="md:col-span-5">
            <SectionHeading id="linux-heading">Linux</SectionHeading>
            <div className="mt-4 grid grid-cols-[minmax(0,1fr)] gap-6">
              <div className="grid grid-cols-[minmax(0,1fr)] gap-2">
                <h3 className={H3}>Debian or Ubuntu</h3>
                <p className="m-0 max-w-[44ch]">
                  The <InlineCode on="shallow">.deb</InlineCode> also puts the <InlineCode on="shallow">quotadial</InlineCode> command in{" "}
                  <InlineCode on="shallow">/usr/bin</InlineCode>.
                </p>
                <CodeBlock code="sudo apt install ./QuotaDial_*.deb" label="Copy the command" />
              </div>
              <div className="grid grid-cols-[minmax(0,1fr)] gap-2">
                <h3 className={H3}>Other distributions</h3>
                <p className="m-0 max-w-[44ch]">Make the AppImage executable, then run it.</p>
                <CodeBlock
                  code={"chmod +x QuotaDial_*.AppImage\n./QuotaDial_*.AppImage"}
                  label="Copy the commands"
                />
              </div>
              <p className="m-0 max-w-[44ch]">
                The dial and number appear in your panel's tray area. GNOME needs the AppIndicator extension for that;
                Ubuntu has it on already.
              </p>
            </div>
          </div>
        </div>

        <div className="mt-16 grid grid-cols-[minmax(0,1fr)] gap-x-6 gap-y-14 border-t border-rule pt-16 md:grid-cols-12">
          <div className="md:col-span-5">
            <SectionHeading id="command-heading">The command on its own</SectionHeading>
            <p className="mt-4 mb-0 max-w-[44ch]">
              If a terminal is all you want, take <InlineCode on="shallow">{COMMAND_ONLY.file}</InlineCode> (a{" "}
              <InlineCode on="shallow">.zip</InlineCode> on Windows). Each archive holds{" "}
              <InlineCode on="shallow">quotadial</InlineCode> and its short alias <InlineCode on="shallow">qd</InlineCode>.
            </p>
            <p className="mt-4 mb-0 max-w-[44ch]">
              The app carries the same command inside it, and{" "}
              <a href={SITE.installGuide} className={LINK}>
                the install guide
              </a>{" "}
              shows how to put it on your PATH.
            </p>
            <div className="mt-6">
              <CodeBlock code="quotadial status" label="Copy the command" />
            </div>
          </div>

          <div className="md:col-span-7">
            <SectionHeading id="check-heading">Check a download</SectionHeading>
            <p className="mt-4 mb-0 max-w-[56ch]">
              <InlineCode on="shallow">SHA256SUMS.txt</InlineCode> lists every file's SHA-256. In the folder you
              downloaded to, paste the line for your system. It prints OK beside each file that matches, and FAILED
              beside any that does not.
            </p>
            <div className="mt-6 grid grid-cols-[minmax(0,1fr)] gap-5">
              <div className="grid grid-cols-[minmax(0,1fr)] gap-2">
                <h3 className={H3}>On macOS</h3>
                <CodeBlock code="shasum -a 256 -c SHA256SUMS.txt --ignore-missing" label="Copy the command" />
              </div>
              <div className="grid grid-cols-[minmax(0,1fr)] gap-2">
                <h3 className={H3}>On Linux</h3>
                <CodeBlock code="sha256sum -c SHA256SUMS.txt --ignore-missing" label="Copy the command" />
              </div>
              <div className="grid grid-cols-[minmax(0,1fr)] gap-2">
                <h3 className={H3}>On Windows</h3>
                <p className="m-0 max-w-[56ch]">
                  PowerShell prints the hash. Compare it with the line for that file in{" "}
                  <InlineCode on="shallow">SHA256SUMS.txt</InlineCode>.
                </p>
                <CodeBlock code="Get-FileHash .\QuotaDial_*.msi" label="Copy the command" />
              </div>
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
                  Until QuotaDial's signing certificates are in place, each system warns you the first time you open
                  one. Only do this for a build you got from QuotaDial's GitHub repository.
                </p>
                <p className="mt-3 mb-0">
                  A release's notes say whether it is signed for each system. A signed build opens with no warning at
                  all.
                </p>
              </Notice>
            </div>
            <p className="mt-6 mb-0 max-w-[44ch]">
              A warning that is not described here?{" "}
              <a href="/support" className={LINK}>
                Write to support
              </a>{" "}
              with what it says.
            </p>
          </div>

          <div className="grid grid-cols-[minmax(0,1fr)] gap-10 md:col-span-7">
            <div className="grid grid-cols-[minmax(0,1fr)] gap-3">
              <h3 className={H3}>On macOS</h3>
              <Steps>
                <li>Open the disk image and drag <strong>QuotaDial</strong> to <strong>Applications</strong>.</li>
                <li>
                  Open QuotaDial from Applications. macOS says it cannot verify the app. Choose <strong>Done</strong>,
                  not <strong>Move to Trash</strong>.
                </li>
                <li>
                  Open <strong>System Settings &gt; Privacy &amp; Security</strong>. Under <strong>Security</strong>,
                  choose <strong>Open Anyway</strong> next to the message about QuotaDial, and confirm with your
                  password or Touch ID.
                </li>
                <li>
                  Open QuotaDial again and choose <strong>Open</strong>.
                </li>
              </Steps>
              <p className="m-0 max-w-[60ch]">
                On macOS 14 and earlier you can instead Control-click QuotaDial in Applications, choose{" "}
                <strong>Open</strong>, then <strong>Open</strong> again.
              </p>
            </div>

            <div className="grid grid-cols-[minmax(0,1fr)] gap-3">
              <h3 className={H3}>On Windows</h3>
              <Steps>
                <li>
                  Run the installer. Edge may say it “isn't commonly downloaded”: choose the three dots, then{" "}
                  <strong>Keep</strong>, <strong>Show more</strong> and <strong>Keep anyway</strong>.
                </li>
                <li>
                  If Windows says it <strong>protected your PC</strong>, choose <strong>More info</strong>, then{" "}
                  <strong>Run anyway</strong>.
                </li>
              </Steps>
            </div>

            <div className="grid grid-cols-[minmax(0,1fr)] gap-3">
              <h3 className={H3}>On Linux</h3>
              <p className="m-0 max-w-[60ch]">
                The steps are the commands under Linux above: <InlineCode on="mid">apt install</InlineCode> for the{" "}
                <InlineCode on="mid">.deb</InlineCode>, or <InlineCode on="mid">chmod +x</InlineCode> for the AppImage.
              </p>
            </div>
          </div>
        </div>
      </Section>
    </>
  );
}
