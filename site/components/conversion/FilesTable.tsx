"use client";

import { useEffect, useState } from "react";

import type { System } from "../../lib/data/release.ts";
import { detectSystem } from "../DownloadButton.tsx";
import { TideTable } from "../TideTable.tsx";
import { type AppFile, systemOf } from "./download-links.ts";
import { FileName } from "./FileName.tsx";

const NOTE = "block text-[0.9375rem] font-[400] text-ink-2";

/**
 * The files in each release, a band for each system, set as the command's archives are: the name
 * a button saves, which fetches it, and under it the name a release holds it by, which leads to the
 * Releases page, where every version's files are, so a visitor whose button found no full release
 * yet is never at a dead end. From 768 px it is a table, system then file; on a phone each file
 * stands under its system. The rows for the visitor's own system say so.
 */
export function FilesTable({ files, releases }: { files: AppFile[]; releases: string }) {
  const [system, setSystem] = useState<System | null>(null);
  useEffect(() => setSystem(detectSystem()), []);
  const rows = files.map((file) => ({ ...file, group: systemOf(file.name) }));
  const yours = (group: System) => group === system;
  const groups = rows.filter((row, index) => rows.findIndex((other) => other.group === row.group) === index);
  return (
    <>
      <TideTable
        className="max-md:hidden"
        caption={{ text: "Files in each release" }}
        columns={["System", "File"]}
        rows={rows.map((row) => ({
          key: row.versioned,
          group: row.group,
          cells: [
            {
              text: (
                <>
                  {row.who}
                  {yours(row.group) && <span className={NOTE}>Your system</span>}
                </>
              ),
            },
            { text: <FileName file={row} from="files" record={releases} /> },
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
            {rows
              .filter((row) => row.group === group.group)
              .map((row, index) => (
                <div key={row.versioned} className={index > 0 ? "mt-3" : undefined}>
                  <p className="m-0 font-table font-[600]">
                    {row.who}
                    {yours(row.group) && <span className={NOTE}>Your system</span>}
                  </p>
                  <p className="m-0 mt-1 font-table">
                    <FileName file={row} from="files" record={releases} />
                  </p>
                </div>
              ))}
          </div>
        ))}
      </div>
    </>
  );
}
