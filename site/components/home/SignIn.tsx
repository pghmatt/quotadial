import { SectionHeading } from "../Section.tsx";
import { BandedTable } from "./BandedTable.tsx";
import { Part } from "./Part.tsx";
import { Body, Code, Lead } from "./Text.tsx";

/** No sign-in: where it looks, per tool, and what happens when a sign-in stops working. */
export function SignIn() {
  return (
    <Part id="sign-in" headingId="sign-in-title">
      <SectionHeading id="sign-in-title">Uses the sign-in you already have</SectionHeading>
      <Lead>
        QuotaDial reads the sign-in that Claude Code and the Codex CLI already keep, and only reads it: it never
        writes, refreshes, moves or logs it. Every Claude subscription you keep under its own{" "}
        <Code>CLAUDE_CONFIG_DIR</Code> is found too: QuotaDial Pro shows each one, and Free shows one and says how many
        more it found.
      </Lead>
      <div className="mt-8">
        <BandedTable
          wholeFirst
          caption="Where QuotaDial looks for each tool's sign-in, and what to do when it stops working"
          columns={["Tool", "Where QuotaDial looks, in this order", "If it stops working"]}
          rows={[
            {
              key: "claude",
              cells: [
                "Claude Code",
                <>
                  A sign-in set in the environment, then the one in the macOS Keychain, then the sign-in file in{" "}
                  <Code>~/.claude</Code> or in the folder <Code>CLAUDE_CONFIG_DIR</Code> names.
                </>,
                <>
                  Run <Code>claude</Code> once.
                </>,
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
        So there is no new sign-in to make, no API key to paste and no account to open. If a sign-in stops working,
        QuotaDial says so and names the command to run.
      </Body>
    </Part>
  );
}
