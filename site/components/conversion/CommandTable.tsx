import { TideTable } from "../TideTable.tsx";
import { type CommandFile, commandSystemOf } from "./download-links.ts";
import { FileName } from "./FileName.tsx";

/**
 * The four archives of the command on its own. Each name fetches the newest full release's copy;
 * the versioned name under it is the file a release holds as the record. Names break only where
 * `Breakable` marks, so none runs off a phone's edge and none breaks where there is room.
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
            text: <FileName file={command} from="command" system={commandSystemOf(command.name)} />,
          },
        ],
      }))}
    />
  );
}
