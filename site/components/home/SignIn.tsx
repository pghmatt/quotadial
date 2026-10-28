import { SectionHeading } from "../Section.tsx";
import { BandedTable } from "./BandedTable.tsx";
import { Part } from "./Part.tsx";
import { Body, Code, Lead } from "./Text.tsx";

/** No sign-in: where it looks, per tool, and what happens when a sign-in stops working. */
export function SignIn() {
  return (
    <Part id="sign-in" headingId="sign-in-title">
      <SectionHeading id="sign-in-title">No new sign-in or API key</SectionHeading>
      <Lead>
        QuotaDial reads the sign-in the Codex CLI already keeps, and only reads it: it never writes, refreshes, moves or
        logs it. Claude's numbers come from Claude Code's status line once you add one line to its settings, and Claude
        Code's sign-in is never read. Every Claude subscription you keep under its own <Code>CLAUDE_CONFIG_DIR</Code>{" "}
        is found too: QuotaDial Pro shows each one, and Free shows one and says how many more it found.
      </Lead>
      <div className="mt-8">
        <BandedTable
          wholeFirst
          caption="Where QuotaDial gets each tool's numbers, and what to do when they stop"
          columns={["Tool", "Where QuotaDial gets the numbers", "If they stop"]}
          rows={[
            {
              key: "claude",
              cells: [
                "Claude Code",
                <>
                  What Claude Code's status line hands <Code>quotadial claude status-line</Code>, once you add it to the
                  settings in <Code>~/.claude</Code> or in the folder <Code>CLAUDE_CONFIG_DIR</Code> names. Its sign-in
                  is never read.
                </>,
                <>Start Claude Code again, and send it a message.</>,
              ],
            },
            {
              key: "codex",
              cells: [
                "Codex CLI",
                <>
                  The sign-in file in <Code>~/.codex</Code>, or in the folder <Code>CODEX_HOME</Code> names.
                </>,
                <>
                  Run <Code>codex</Code> once.
                </>,
              ],
            },
          ]}
        />
      </div>
      <Body>
        So there is no new sign-in to make, no API key to paste and no account to open. When numbers stop coming,
        QuotaDial says so and names what to do.
      </Body>
    </Part>
  );
}
