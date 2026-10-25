import { AppleLogo, LinuxLogo, WindowsLogo } from "@phosphor-icons/react/dist/ssr";
import type { ComponentType } from "react";

import { DOWNLOADS, type Download, type System } from "../../lib/data/release.ts";
import { ButtonLink } from "../Button.tsx";
import { SectionHeading } from "../Section.tsx";
import { Part } from "./Part.tsx";
import { Body, Code, Lead } from "./Text.tsx";

const LOGOS: Record<System, ComponentType<{ size?: number; "aria-hidden"?: boolean }>> = {
  macos: AppleLogo,
  windows: WindowsLogo,
  linux: LinuxLogo,
};

/** Where on the screen QuotaDial sits, by system. */
const PLACE: Record<System, string> = {
  macos: "In the menu bar.",
  windows: "In the notification area.",
  linux: "In the panel's tray.",
};

const extension = (name: string) => name.slice(name.lastIndexOf("."));

function SystemColumn({ download }: { download: Download }) {
  const Logo = LOGOS[download.system];
  return (
    <div className="border-t border-rule pt-5">
      <div className="flex items-center gap-3">
        <Logo size={30} aria-hidden />
        <h3 className="m-0 text-[2rem] leading-none font-[750]">{download.label}</h3>
      </div>
      <p className="m-0 mt-3 text-ink-2">{PLACE[download.system]}</p>
      <ul className="m-0 mt-5 grid list-none gap-4 p-0">
        {download.files.map((file) => (
          <li key={file.name}>
            <span className="font-display text-[1.75rem] leading-none font-[700]">{extension(file.name)}</span>
            <span className="ml-3 text-ink-2">{file.what}</span>
          </li>
        ))}
      </ul>
      <p className="m-0 mt-6">
        <ButtonLink
          variant="text"
          href={`/download#${download.system}`}
          data-event="download_click"
          data-event-system={download.system}
        >
          Download for {download.label}
        </ButtonLink>
      </p>
    </div>
  );
}

/** Systems: one app for all three, in columns of unequal width. */
export function Systems() {
  return (
    <Part id="systems" headingId="systems-title">
      <SectionHeading id="systems-title">On macOS, Windows and Linux</SectionHeading>
      <Lead>
        One app for all three: it sits in the menu bar on macOS, in the notification area on Windows and in the
        panel's tray on Linux. The <Code>quotadial</Code> command has its own download for each of them.
      </Lead>
      <div className="mt-10 grid gap-10 lg:grid-cols-[5fr_4fr_6fr] lg:gap-12">
        {DOWNLOADS.map((download) => (
          <SystemColumn key={download.system} download={download} />
        ))}
      </div>
      <Body>
        Builds are not signed yet, so each system warns the first time you open one.{" "}
        <a href="/download">The download page</a> says how to open it anyway.
      </Body>
    </Part>
  );
}
