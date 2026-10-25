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
      title="Versions"
      caption="Every version in the changelog, with the number of changes listed under each heading."
    >
      <TideTable
        caption={{ text: "Versions of QuotaDial and the changes each lists" }}
        columns={["Version", "Released", ...headings]}
        align={["words", "figures", ...headings.map(() => "figures" as const)]}
        whole={[true, true]}
        /* The times never break, so on a phone the cells give up 6 px of their side padding each (text keeps its size) and the plate fits the column down to 360 px. */
        className="max-sm:[&_td]:px-1.5 max-sm:[&_th]:px-1.5"
        rows={versions.map((version) => ({
          key: version.version,
          cells: [
            { text: <InlineLink href={`#${versionId(version.version)}`}>{version.version}</InlineLink> },
            version.date ? sayDate(version.date, true) : "Not yet",
            ...headings.map((heading) => String(countUnder(version, heading))),
          ],
        }))}
      />
    </Plate>
  );
}
