import { ButtonLink } from "../Button.tsx";
import { SectionHeading } from "../Section.tsx";
import { BandedTable } from "./BandedTable.tsx";
import { Part } from "./Part.tsx";
import { Body, Code, Lead } from "./Text.tsx";

/** The addresses QuotaDial connects to, as docs/PRIVACY.md lists them. */
const ADDRESSES = [
  {
    key: "anthropic",
    address: ["api.anthropic.com"],
    why: "Each Claude account's limits, with your Claude Code sign-in.",
    often: "At most every 5 minutes per account",
  },
  {
    key: "chatgpt",
    address: ["chatgpt.com"],
    why: "Your ChatGPT/Codex limits, with your Codex CLI sign-in.",
    often: "At most every 2 minutes",
  },
  {
    key: "status",
    address: ["status.claude.com", "status.openai.com"],
    why: "The providers' public status pages. No sign-in.",
    often: "Every 5 minutes, and you can turn it off",
  },
  {
    key: "github",
    address: ["github.com", "release-assets.githubusercontent.com"],
    why: "Checking for a newer QuotaDial, and fetching it when you choose Update. No sign-in, nothing about you or your usage.",
    often: "Once a day, or when you check yourself",
  },
  {
    key: "yours",
    address: [],
    why: "The webhook and Slack addresses you add, if you add any. Each gets the alert and its numbers.",
    often: "When an alert fires",
  },
];

/** Privacy: every address it connects to, and what it never reads. */
export function Addresses() {
  return (
    <Part id="addresses" headingId="addresses-title">
      <div className="grid grid-cols-[minmax(0,1fr)] gap-10 lg:grid-cols-[minmax(0,26rem)_minmax(0,1fr)] lg:items-start lg:gap-16">
        <div>
          <SectionHeading id="addresses-title">Everything it connects to</SectionHeading>
          <Lead>
            Anthropic and ChatGPT for your limits, their public status pages, GitHub for updates, and webhooks you add:
            nothing else, no account, no analytics.
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
            cells: [
              row.address.length ? (
                <span className="grid justify-items-start gap-1 font-code text-[0.875rem] leading-[1.35] font-[500] [overflow-wrap:anywhere]">
                  {row.address.map((address) => (
                    <Code key={address}>{address}</Code>
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
