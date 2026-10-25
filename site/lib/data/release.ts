/**
 * What QuotaDial's releases on GitHub hold, by system, as docs/INSTALL.md lists them (a test holds
 * the two together; /download reads INSTALL.md itself through `installGuide`).
 *
 * Downloads go to the latest release's page. GitHub sends that address to the newest full release,
 * and to the list of every release while all of them are pre-releases, as now, so publishing a
 * release never needs the site built again (#49).
 */

import { SITE } from "./site.ts";

export type System = "macos" | "windows" | "linux";

export const DOWNLOAD_PAGE = `${SITE.releases}/latest`;

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
