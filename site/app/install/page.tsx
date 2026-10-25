import type { Metadata } from "next";

import { GuidePage } from "../../components/reference/GuidePage.tsx";
import { GUIDES, guide } from "../../components/reference/guide.ts";
import { pageMetadata } from "../../lib/metadata.ts";

export const metadata: Metadata = pageMetadata({
  title: "Install guide: package managers, settings, updates",
  description:
    "How to install QuotaDial on macOS, Windows and Linux with a download or a package manager, open a build not signed yet, change its settings and update it.",
  path: "/install",
});

/** docs/INSTALL.md, whole, as the page shows it: its own heading is the H1 and every section follows in its order. */
export default function Install() {
  return (
    <GuidePage
      guide={guide("install")}
      file={GUIDES.install.file}
      answer="Every way to install QuotaDial on macOS, Windows and Linux, what its settings do and how it updates."
      other={{ href: "/cli", label: "Read the command guide" }}
      closing={{
        heading: "Download QuotaDial",
        text: "The download page names the file for your system, how to check it and, for a build that is not signed yet, how to open it.",
      }}
    />
  );
}
