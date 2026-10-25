import { InlineLink } from "./InlineLink.tsx";
import { Plate } from "../Plate.tsx";
import { TideTable } from "../TideTable.tsx";
import type { ChangelogVersion } from "../../lib/markdown.ts";
import { countUnder, headingsUsed, sayDate, versionId } from "./text.ts";

/** Every version in the changelog as a small table: when it came out and how many changes it lists. */
export function ReleaseTable({ versions }: { versions: ChangelogVersion[] }) {
  const headings = headingsUsed(versions);
  return (
    <Plate
      tone="shallow"
      caption="Every version in the changelog, with the number of changes listed under each heading."
    >
      <TideTable
        caption={{ text: "Versions of QuotaDial and the changes each lists" }}
        columns={["Version", "Released", ...headings]}
        rows={versions.map((version) => ({
          key: version.version,
          cells: [
            { text: <InlineLink href={`#${versionId(version.version)}`}>{version.version}</InlineLink> },
            version.date ? sayDate(version.date) : "Not yet",
            ...headings.map((heading) => String(countUnder(version, heading))),
          ],
        }))}
      />
    </Plate>
  );
}
