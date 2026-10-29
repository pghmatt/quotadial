import type { Metadata } from "next";

import { GuidePage } from "../../components/reference/GuidePage.tsx";
import { GUIDES, guide } from "../../components/reference/guide.ts";
import { pageMetadata } from "../../lib/metadata.ts";

export const metadata: Metadata = pageMetadata({
  title: "The quotadial command: output, API and exit codes",
  description:
    "Every quotadial command, what it prints as text and as JSON, the local API and MCP server for agents, and the exit codes scripts can rely on.",
  path: "/cli",
});

/** docs/CLI.md, whole, as the page shows it: its own heading is the H1 and every section follows in its order. */
export default function Cli() {
  return (
    <GuidePage
      guide={guide("cli")}
      file={GUIDES.cli.file}
      answer="Every quotadial command, what it prints, the local API and MCP server for agents, and the exit codes scripts can rely on."
      other={{ href: "/install", label: "Read the install guide" }}
      closing={{
        heading: "Download QuotaDial",
        text: "The command comes inside the app. QuotaDial Free reads the sign-in the Codex CLI already keeps, and for Claude needs one line in Claude Code's settings, as the install guide says.",
      }}
    />
  );
}
