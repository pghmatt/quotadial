import { ButtonLink } from "../Button.tsx";
import { Question } from "../Question.tsx";
import { SectionHeading } from "../Section.tsx";
import { Part } from "./Part.tsx";
import { Code } from "./Text.tsx";

/** The questions a developer asks before installing, each answered where it is asked. */
export function Questions() {
  const items = [
    {
      q: "Why use a tracker when Claude Code has /usage and Codex has /status?",
      a: (
        <p>
          Those commands answer when you type them, inside the one tool you are in. QuotaDial keeps both providers at
          a glance in the menu bar or tray, adds the pace and a history, and warns you before a limit instead of
          after it.
        </p>
      ),
    },
    {
      q: "Does it read my conversations?",
      a: (
        <p>
          No. From the CLIs' session logs it reads only numbers: how much was used and which model, for the week's
          cost, and the Codex CLI's own record of your limits. What you and the model wrote never leaves your
          machine.
        </p>
      ),
    },
    {
      q: "Does it need an API key or a new sign-in?",
      a: (
        <p>
          Neither. It uses the sign-in Claude Code and the Codex CLI already keep, and only reads it. If a sign-in stops
          working, QuotaDial says so and names the command to run.
        </p>
      ),
    },
    {
      q: "Will it warn me before I hit a limit?",
      a: (
        <p>
          Yes. “Warn me at 80%” sends a notification when any window reaches 80%. On Pro, “tell me 30 minutes before the
          weekly window runs out” uses the pace to do it earlier. You write the rule in plain words.
        </p>
      ),
    },
    {
      q: "Does it work on Windows and Linux?",
      a: (
        <p>
          Yes. QuotaDial runs on macOS, Windows and Linux from one codebase. It sits in the notification area on
          Windows and the panel's tray on Linux, and the <Code>quotadial</Code> command has its own download for each.
        </p>
      ),
    },
    {
      q: "Does it work with Codex?",
      a: (
        <p>
          Yes. It reads the sign-in the Codex CLI keeps and shows its 5-hour and weekly windows beside Claude's. When
          ChatGPT cannot be asked, it falls back to the Codex CLI's own record of your limits and says so.
        </p>
      ),
    },
    {
      q: "Can it show several Claude subscriptions?",
      a: (
        <p>
          Yes, on Pro. Each subscription kept under its own <Code>CLAUDE_CONFIG_DIR</Code> shows as its own account
          with its own windows. QuotaDial Free shows one account per provider and says how many more it found.
        </p>
      ),
    },
    {
      q: "Can my agent check its limits?",
      a: (
        <p>
          Yes. <Code>quotadial status --json</Code> gives an agent every window with its reset and its pace, and so do
          the local API and the MCP server's <Code>get_usage</Code> tool. The SKILL.md tells it when to look.
        </p>
      ),
    },
    {
      q: "Is it open source?",
      a: (
        <p>
          Yes, under the MIT licence. The source is on GitHub, and you can read it, build it and change it. Pro is a key
          in the same build, checked on your machine.
        </p>
      ),
    },
  ];
  return (
    <Part id="questions" headingId="questions-title">
      <SectionHeading id="questions-title">Questions before you install</SectionHeading>
      <div className="mt-10 gap-x-16 md:columns-2">
        {items.map((item) => (
          <div key={item.q} className="mb-9 break-inside-avoid">
            <Question question={item.q}>{item.a}</Question>
          </div>
        ))}
      </div>
      <p className="m-0 mt-2">
        <ButtonLink variant="text" href="/support">
          Ask support something else
        </ButtonLink>
      </p>
    </Part>
  );
}
