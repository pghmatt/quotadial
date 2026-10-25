"use client";

import { useEffect, useState } from "react";

import { COMMAND_ONLY, DOWNLOADS, type System } from "../../lib/data/release.ts";
import { detectSystem } from "../DownloadButton.tsx";
import { TideTable } from "../TideTable.tsx";

const NOTE = "block text-[0.9375rem] font-[400] text-ink-2";

interface Entry {
  /** What the files belong to: one band for each system, both Linux files in one. */
  group: string;
  label: string;
  system: System | null;
  file: string;
  note: string;
}

const ENTRIES: Entry[] = [
  ...DOWNLOADS.flatMap((download) =>
    download.files.map((file) => ({ group: download.system, label: download.label, system: download.system, file: file.name, note: file.what })),
  ),
  { group: "command", label: "The command alone", system: null, file: COMMAND_ONLY.file, note: `${COMMAND_ONLY.windows.slice(COMMAND_ONLY.windows.lastIndexOf("."))} on Windows` },
];

/**
 * The files in each release, a band for each system. From 768 px it is a table, system then file;
 * on a phone each file stands under its system, so no name is squeezed or broken. The rows for the
 * visitor's own system say so.
 */
export function FilesTable() {
  const [system, setSystem] = useState<System | null>(null);
  useEffect(() => setSystem(detectSystem()), []);
  const yours = (entry: Entry) => entry.system !== null && entry.system === system;
  const groups = ENTRIES.filter((entry, index) => ENTRIES.findIndex((other) => other.group === entry.group) === index);
  return (
    <>
      <TideTable
        className="max-md:hidden"
        caption={{ text: "Files in each release" }}
        columns={["System", "File"]}
        whole={[false, true]}
        rows={ENTRIES.map((entry) => ({
          key: entry.file,
          group: entry.group,
          cells: [
            {
              text: (
                <>
                  {entry.label}
                  {yours(entry) && <span className={NOTE}>Your system</span>}
                </>
              ),
            },
            {
              text: (
                <>
                  {entry.file}
                  <span className={NOTE}>{entry.note}</span>
                </>
              ),
            },
          ],
        }))}
      />
      <div className="md:hidden">
        <p className="sr-only">Files in each release</p>
        {groups.map((group, number) => (
          <div
            key={group.group}
            className={`px-3 py-3 ${number % 2 === 1 ? "bg-[color-mix(in_srgb,var(--ink)_5%,transparent)]" : ""}`}
          >
            <p className="m-0 font-table font-[600]">
              {group.label}
              {yours(group) && <span className={NOTE}>Your system</span>}
            </p>
            {ENTRIES.filter((entry) => entry.group === group.group).map((entry) => (
              <p key={entry.file} className="m-0 mt-2 font-table">
                <span className="whitespace-nowrap">{entry.file}</span>
                <span className={NOTE}>{entry.note}</span>
              </p>
            ))}
          </div>
        ))}
      </div>
    </>
  );
}
