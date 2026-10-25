/**
 * What /download does with the files docs/INSTALL.md lists: which system each belongs to, what a
 * button for it says, and the detail its click reports. The names and links themselves come from
 * `installGuide` at build time; nothing here lists a file.
 */

import type { System } from "../../lib/data/release.ts";

/** A file the visitor can fetch: INSTALL.md's row for it, with the version-less copy it links. */
export interface AppFile {
  /** INSTALL.md's own words for who it is for, such as "Debian or Ubuntu". */
  who: string;
  /** The versioned name, as a release holds it, such as `QuotaDial_<version>_universal.dmg`. */
  versioned: string;
  /** The version-less name the link fetches, such as `QuotaDial_universal.dmg`. */
  name: string;
  /** Where that name is: `releases/latest/download/` plus the name. */
  href: string;
}

/** An archive of the command on its own: INSTALL.md's row, with the version-less copy it links. */
export interface CommandFile {
  /** INSTALL.md's own words for who it is for, such as "macOS, Apple silicon". */
  who: string;
  /** The versioned name, such as `quotadial-v<version>-aarch64-apple-darwin.tar.gz`. */
  versioned: string;
  /** The version-less name the link fetches. */
  name: string;
  href: string;
}

export const SYSTEM_LABEL: Record<System, string> = { macos: "macOS", windows: "Windows", linux: "Linux" };

const BY_EXTENSION: Record<string, System> = { dmg: "macos", msi: "windows", deb: "linux", AppImage: "linux" };

/** A file's extension without its dot, such as "AppImage". */
export function extensionOf(name: string): string {
  return name.slice(name.lastIndexOf(".") + 1);
}

/** The system a file is for. A file of a kind the site does not know fails the build. */
export function systemOf(name: string): System {
  const system = BY_EXTENSION[extensionOf(name)];
  if (!system) {
    throw new Error(`docs/INSTALL.md lists ${name}, which the download page cannot place on a system.`);
  }
  return system;
}

/** INSTALL.md's rows as the page's files. */
export function appFiles(rows: { system: string; file: string; latest: { name: string; href: string } }[]): AppFile[] {
  return rows.map((row) => ({ who: row.system, versioned: row.file, name: row.latest.name, href: row.latest.href }));
}

/** INSTALL.md's archive rows as the page's command files. */
export function commandFiles(rows: { system: string; archive: string; latest: { name: string; href: string } }[]): CommandFile[] {
  return rows.map((row) => ({ who: row.system, versioned: row.archive, name: row.latest.name, href: row.latest.href }));
}

/** The files for one system, in INSTALL.md's order. */
export function filesFor(files: AppFile[], system: System): AppFile[] {
  return files.filter((file) => systemOf(file.name) === system);
}

/**
 * What a button says. A system with one file is named for the system; Linux has two, so each is
 * named for its kind, as the visitor meets it on the file ("Download the .deb").
 */
export function buttonLabel(file: AppFile, among: AppFile[]): string {
  return among.length > 1 ? `Download the .${extensionOf(file.name)}` : `Download for ${SYSTEM_LABEL[systemOf(file.name)]}`;
}

/** The system a command archive is for, from the target in its name. */
export function commandSystemOf(name: string): System {
  if (name.includes("apple-darwin")) {
    return "macos";
  }
  if (name.includes("windows")) {
    return "windows";
  }
  if (name.includes("linux")) {
    return "linux";
  }
  throw new Error(`docs/INSTALL.md lists ${name}, which the download page cannot place on a system.`);
}

/** The `data-event-*` attributes of a download click: the system and the file, and where it was. */
export function clickDetail(file: { name: string }, from: string, system: System = systemOf(file.name)) {
  return {
    "data-event": "download_click",
    "data-event-system": system,
    "data-event-file": file.name,
    "data-event-from": from,
  } as const;
}
