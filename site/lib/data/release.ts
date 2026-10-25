/**
 * What QuotaDial's releases on GitHub hold, by system, as docs/INSTALL.md lists them. While every
 * release is a pre-release, downloads go to the releases page; once a full release exists,
 * `DOWNLOAD_PAGE` becomes `${SITE.releases}/latest`, a one-line change.
 */

import { SITE } from "./site.ts";

export type System = "macos" | "windows" | "linux";

export const DOWNLOAD_PAGE = SITE.releases;

export interface Download {
  system: System;
  /** As people name it. */
  label: string;
  /** The files to pick on the releases page, with what each is for. */
  files: { name: string; what: string }[];
}

export const DOWNLOADS: Download[] = [
  {
    system: "macos",
    label: "macOS",
    files: [{ name: "QuotaDial_<version>_universal.dmg", what: "Apple silicon and Intel" }],
  },
  {
    system: "windows",
    label: "Windows",
    files: [{ name: "QuotaDial_<version>_x64_en-US.msi", what: "the installer" }],
  },
  {
    system: "linux",
    label: "Linux",
    files: [
      { name: "QuotaDial_<version>_amd64.deb", what: "Debian or Ubuntu" },
      { name: "QuotaDial_<version>_amd64.AppImage", what: "other distributions" },
    ],
  },
];

/** The command on its own, without the tray app. */
export const COMMAND_ONLY = {
  file: "quotadial-<version>-<platform>.tar.gz",
  windows: "quotadial-<version>-<platform>.zip",
  holds: ["quotadial", "qd"],
} as const;
