import type { ReactNode } from "react";

import { Breakable } from "../Breakable.tsx";
import { type AppFile, type CommandFile, clickDetail } from "./download-links.ts";

const LINK = "underline underline-offset-[0.2em] [overflow-wrap:break-word]";
const NOTE = "block text-[0.9375rem] font-[400] text-ink-2 [overflow-wrap:break-word]";

/**
 * A file in a table, set one way for the app's files and the command's archives: the name its
 * button saves, which fetches it, and under it in second ink the name a release holds it by. The
 * versioned name links to `record` (the Releases page) when given. Both break only where
 * `Breakable` marks, so a name stays whole when its column has the room.
 */
export function FileName({
  file,
  from,
  system,
  record,
}: {
  file: AppFile | CommandFile;
  from: string;
  system?: Parameters<typeof clickDetail>[2];
  record?: string;
}): ReactNode {
  return (
    <>
      <a href={file.href} className={LINK} {...clickDetail(file, from, system)}>
        <Breakable text={file.name} />
      </a>
      <span className={NOTE}>
        {record ? (
          <a href={record} className={LINK}>
            <Breakable text={file.versioned} />
          </a>
        ) : (
          <Breakable text={file.versioned} />
        )}
      </span>
    </>
  );
}
