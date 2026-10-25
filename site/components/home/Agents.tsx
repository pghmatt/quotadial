import { CodeBlock } from "../CodeBlock.tsx";
import { SectionHeading } from "../Section.tsx";
import { Part } from "./Part.tsx";
import { STATUS_SAMPLE } from "./examples.ts";
import { Body, Code, Lead } from "./Text.tsx";

const MCP = `claude mcp add quotadial -- quotadial mcp
codex mcp add quotadial -- quotadial mcp`;

const caption = "mt-3 mb-0 font-table text-[0.9375rem] text-ink-2 italic";

/** Agents: the same numbers as a command, a local API and an MCP server, with the command's output. */
export function Agents() {
  return (
    <Part id="agents" headingId="agents-title">
      <div className="grid grid-cols-[minmax(0,1fr)] gap-10 xl:grid-cols-[minmax(0,1fr)_minmax(0,41rem)] xl:items-start xl:gap-20">
        <div>
          <SectionHeading id="agents-title">Your agents can check before they start</SectionHeading>
          <Lead>
            The same numbers come through <Code>quotadial status</Code>, a local API on <Code>127.0.0.1</Code> and an
            MCP server with a SKILL.md, so an agent can check its headroom before it begins a long job.
          </Lead>
          <ul className="m-0 mt-6 grid max-w-[62ch] list-none gap-3 p-0 text-ink-2">
            <li>
              <Code>quotadial status</Code> in a terminal, or with <Code>--json</Code> for a script. <Code>qd</Code> is
              the short name.
            </li>
            <li>
              The local API answers only on your machine, at <Code>127.0.0.1</Code>.
            </li>
            <li>The MCP server is named <Code>quotadial</Code>, and the SKILL.md tells an agent when to look.</li>
          </ul>
          <Body>
            Reading every window and its reset works on any copy. Ranking your accounts by how much room each has left is
            part of QuotaDial Pro.
          </Body>
        </div>
        <div className="grid grid-cols-[minmax(0,1fr)] gap-8">
          <figure className="m-0">
            <CodeBlock code={STATUS_SAMPLE} copy={false} wrap={false} name="quotadial status output, example accounts" />
            <figcaption className={caption}>
              Example accounts. What <Code>quotadial status</Code> prints, for an agent or for you.
            </figcaption>
          </figure>
          <figure className="m-0">
            <CodeBlock code={MCP} label="Copy the commands" />
            <figcaption className={caption}>Add the server to Claude Code or to Codex.</figcaption>
          </figure>
        </div>
      </div>
    </Part>
  );
}
