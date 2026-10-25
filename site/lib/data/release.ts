/**
 * What QuotaDial's releases on GitHub hold, by system, as docs/INSTALL.md lists them (a test holds
 * the two together; /download reads INSTALL.md itself through `installGuide`).
 *
 * The links come from docs/INSTALL.md through `installGuide`; this list only maps a system to the
 * files home names.
 */

export type System = "macos" | "windows" | "linux";

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
