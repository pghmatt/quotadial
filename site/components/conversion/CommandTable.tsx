import { TideTable } from "../TideTable.tsx";
import { type CommandFile, clickDetail, commandSystemOf } from "./download-links.ts";
import { soft } from "./soft.tsx";

const NOTE = "block text-[0.9375rem] font-[400] text-ink-2";

/**
 * The four archives of the command on its own. Each name fetches the newest full release's copy;
 * the versioned name under it is the file a release holds as the record. Names break after each
 * hyphen, so none runs off a phone's edge.
 */
export function CommandTable({ commands }: { commands: CommandFile[] }) {
  return (
    <TideTable
      caption={{ text: "The command on its own, one archive for each system" }}
      columns={["System", "Archive"]}
      rows={commands.map((command) => ({
        key: command.name,
        group: commandSystemOf(command.name),
        cells: [
          command.who,
          {
            text: (
              <>
                <a
                  href={command.href}
                  className="underline underline-offset-[0.2em]"
                  {...clickDetail(command, "command", commandSystemOf(command.name))}
                >
                  {soft(command.name)}
                </a>
                <span className={NOTE}>{soft(command.versioned)}</span>
              </>
            ),
          },
        ],
      }))}
    />
  );
}
