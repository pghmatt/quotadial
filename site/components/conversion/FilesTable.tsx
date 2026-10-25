"use client";

import { useEffect, useState } from "react";

import { COMMAND_ONLY, DOWNLOADS, type System } from "../../lib/data/release.ts";
import { detectSystem } from "../DownloadButton.tsx";
import { TideTable } from "../TideTable.tsx";
import { soft } from "./soft.tsx";

const NOTE = "block text-[0.9375rem] font-[400] text-ink-2";

/** The files in each release, one row each; the row for the visitor's own system says so. */
export function FilesTable() {
  const [system, setSystem] = useState<System | null>(null);
  useEffect(() => setSystem(detectSystem()), []);
  const rows = [
    ...DOWNLOADS.flatMap((download) =>
      download.files.map((file) => ({
        key: file.name,
        cells: [
          {
            text: (
              <>
                {download.label}
                {system === download.system && <span className={NOTE}>Your system</span>}
              </>
            ),
          },
          {
            text: (
              <>
                {soft(file.name)}
                <span className={NOTE}>{file.what}</span>
              </>
            ),
          },
        ],
      })),
    ),
    {
      key: COMMAND_ONLY.file,
      cells: [
        { text: "The command alone" },
        {
          text: (
            <>
              {soft(COMMAND_ONLY.file)}
              <span className={NOTE}>{COMMAND_ONLY.windows.slice(COMMAND_ONLY.windows.lastIndexOf("."))} on Windows</span>
            </>
          ),
        },
      ],
    },
  ];
  return <TideTable caption={{ text: "Files in each release", visible: true }} columns={["System", "File"]} rows={rows} />;
}
