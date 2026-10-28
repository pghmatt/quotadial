import { Breakable } from "../Breakable.tsx";
import { ButtonLink } from "../Button.tsx";
import { SectionHeading } from "../Section.tsx";
import { BandedTable } from "./BandedTable.tsx";
import { Part } from "./Part.tsx";
import { Body, Lead } from "./Text.tsx";

/** The addresses QuotaDial connects to, as docs/PRIVACY.md lists them. */
const ADDRESSES = [
  {
    key: "chatgpt",
    group: "limits",
    address: ["chatgpt.com"],
    why: "Your ChatGPT/Codex limits, with your Codex CLI sign-in.",
    often: "At most every 2 minutes",
  },
  {
    key: "status",
    group: "status",
    address: ["status.claude.com", "status.openai.com"],
    why: "The providers' public status pages. No sign-in.",
    often: "Every 5 minutes, and you can turn it off",
  },
  {
    key: "github",
    group: "updates",
    address: ["github.com", "release-assets.githubusercontent.com"],
    why: "Checking for a newer QuotaDial, and fetching it when you choose Update. No sign-in, nothing about you or your usage.",
    often: "Once a day, or when you check yourself",
  },
  {
    key: "yours",
    group: "yours",
    address: [],
    why: "The webhook and Slack addresses you add, if you add any. Each gets the alert and its numbers.",
    often: "When an alert fires",
  },
];

/**
 * Privacy: every address it connects to, and what it never reads. An address breaks only at a
 * space or after a hyphen (`Breakable`), and from 1280 px, where the table is 752 px wide, the
 * longest one is held to a line (no `<wbr>`: it breaks the line even under nowrap), as the Unbroken
 * Time Rule holds a file name in a table.
 */
export function Addresses() {
  return (
    <Part id="addresses" headingId="addresses-title">
      <div className="grid grid-cols-[minmax(0,1fr)] gap-10 lg:grid-cols-[minmax(0,26rem)_minmax(0,1fr)] lg:items-start lg:gap-16">
        <div>
          <SectionHeading id="addresses-title">Everything it connects to</SectionHeading>
          <Lead>
            ChatGPT for your Codex limits, the providers' public status pages, GitHub for updates, and webhooks you
            add: nothing else, no account, no analytics. Claude's limits come from Claude Code's status line.
          </Lead>
          <Body>
            What you and the model wrote never leaves your machine, and QuotaDial never reads it. From the CLIs' session
            logs it reads only numbers.
          </Body>
          <p className="m-0 mt-6">
            <ButtonLink variant="text" href="/privacy">
              Read what QuotaDial reads and sends
            </ButtonLink>
          </p>
        </div>
        <BandedTable
          caption="Every address QuotaDial connects to, what for, and how often"
          columns={["Address", "What for", "How often"]}
          rows={ADDRESSES.map((row) => ({
            key: row.key,
            group: row.group,
            cells: [
              row.address.length ? (
                <span className="grid justify-items-start gap-1 font-code text-[0.875rem] leading-[1.35] font-[500] [overflow-wrap:break-word]">
                  {row.address.map((address) => (
                    <span key={address} className="xl:whitespace-nowrap xl:[&_wbr]:hidden">
                      <Breakable text={address} />
                    </span>
                  ))}
                </span>
              ) : (
                "Yours"
              ),
              row.why,
              row.often,
            ],
          }))}
        />
      </div>
    </Part>
  );
}
