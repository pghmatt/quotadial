import type { Metadata } from "next";

import { AppShot } from "../../components/AppShot.tsx";
import { CodeBlock } from "../../components/CodeBlock.tsx";
import { ProActions } from "../../components/conversion/ProActions.tsx";
import { ProLedger } from "../../components/conversion/ProLedger.tsx";
import { Steps } from "../../components/conversion/Steps.tsx";
import { Plate } from "../../components/Plate.tsx";
import { Section, SectionHeading } from "../../components/Section.tsx";
import { FREE_HAS } from "../../lib/data/pro.ts";
import { SITE } from "../../lib/data/site.ts";
import { pageMetadata } from "../../lib/metadata.ts";

export const metadata: Metadata = pageMetadata({
  title: "What Pro adds: every account and a year of history",
  description:
    "QuotaDial Pro adds every account on the machine, a year of history, early warnings, webhooks and Slack, and the week's cost. QuotaDial Free stays free.",
  path: "/pro",
});

const LINK = "underline underline-offset-[0.2em]";
const upper = (text: string) => text.charAt(0).toUpperCase() + text.slice(1);

export default function Pro() {
  return (
    <>
      <Section>
        <div className="grid items-start gap-x-6 gap-y-12 grid-cols-[minmax(0,1fr)] md:grid-cols-12">
          <div className="md:col-span-5">
            <h1 className="m-0 text-[2.75rem] leading-[1.02] font-[800] md:text-[4rem]">QuotaDial Pro</h1>
            <p className="m-0 mt-5 max-w-[34ch] text-[1.25rem] leading-[1.4] text-ink-2">
              If you keep two Claude subscriptions, or want a warning before a weekly window runs out, QuotaDial Pro
              covers both. QuotaDial Free stays free.
            </p>
            <ProActions checkout={SITE.checkout} />
          </div>

          <div className="md:col-span-7 md:col-start-6 md:row-span-2 md:row-start-1">
            <Plate caption="The eight additions, in the words Settings uses. A Free copy says in one line where each would be.">
              <h2
                id="adds-heading"
                className="m-0 mb-4 px-3 text-[1.875rem] leading-[1.1] font-[750] tracking-[-0.01em]"
              >
                What Pro adds
              </h2>
              <ProLedger />
            </Plate>
          </div>

          <div className="md:col-span-5">
            <SectionHeading id="free-heading">What stays free</SectionHeading>
            <p className="mt-4 mb-0 max-w-[40ch]">
              If you keep one Claude and one Codex sign-in, QuotaDial Free already covers both, because it reads one
              account for each provider. It needs no key.
            </p>
            <ul className="m-0 mt-5 grid list-disc gap-2 pl-6 text-[1.0625rem] leading-[1.45]">
              {FREE_HAS.map((item) => (
                <li key={item}>{upper(item)}</li>
              ))}
            </ul>
            <p className="mt-5 mb-0 max-w-[40ch]">
              QuotaDial is open source under the MIT licence, and Pro is a key, checked on your machine, for features in
              the same build.
            </p>
          </div>
        </div>
      </Section>

      <Section tone="shallow">
        <div className="grid items-start gap-x-6 gap-y-12 grid-cols-[minmax(0,1fr)] md:grid-cols-12">
          <div className="grid grid-cols-[minmax(0,1fr)] gap-14 md:col-span-7">
            <div id="how-buying-works">
              <SectionHeading id="buying-heading">How buying works</SectionHeading>
              <div className="mt-5">
                <Steps>
                  <li>
                    Lemon Squeezy, the merchant of record, takes the payment and shows the price and the tax at its
                    checkout.
                  </li>
                  <li>
                    Its receipt has a button, <strong>Get your QuotaDial key</strong>. It opens the{" "}
                    <a href="/key" className={LINK}>
                      key page
                    </a>{" "}
                    on this site.
                  </li>
                  <li>
                    The key page asks Lemon Squeezy whether the receipt's key is valid, then shows your QuotaDial key.
                    It keeps and logs nothing.
                  </li>
                </Steps>
              </div>
              <p className="mt-4 mb-0 max-w-measure">
                Lost the page? Open the receipt's button again: the same receipt always gives the same key.
              </p>
            </div>

            <div id="adding-your-key">
              <SectionHeading id="key-heading">Adding your key</SectionHeading>
              <p className="mt-5 mb-0 max-w-measure">
                In QuotaDial, open <strong>Settings</strong>, find <strong>Licence</strong>, paste the key into the
                field and choose <strong>Add</strong>. It counts at once.
              </p>
              <p className="mt-4 mb-3 max-w-measure">If you would rather use a terminal, run this and paste the key when it asks:</p>
              <CodeBlock code="quotadial licence add" label="Copy the command" />
              <p className="mt-4 mb-0 max-w-measure">
                QuotaDial checks the key on your machine with no network call, so it never phones home. Once the key is
                in, the lines saying where Pro would be go away.{" "}
                <a href="/support" className={LINK}>
                  Write to support
                </a>{" "}
                if a key will not go in.
              </p>
            </div>
          </div>

          <div className="md:col-span-5">
            <Plate
              tone="mid"
              caption="QuotaDial's Settings: three alert rules, and the Licence field where the key goes. Example accounts."
            >
              <div className="mx-auto max-w-[22.5rem]">
                <AppShot
                  name="settings"
                  alt="QuotaDial's Settings with three alert rules and the Licence field"
                  width={720}
                  height={1826}
                />
              </div>
            </Plate>
          </div>
        </div>
      </Section>
    </>
  );
}
