import type { Metadata } from "next";

import { ButtonLink } from "../../components/Button.tsx";
import { CodeBlock } from "../../components/CodeBlock.tsx";
import { Plate } from "../../components/Plate.tsx";
import { Section } from "../../components/Section.tsx";
import { DoctorSample } from "../../components/reference/DoctorSample.tsx";
import { InlineLink } from "../../components/reference/InlineLink.tsx";
import { PageIntro } from "../../components/reference/PageIntro.tsx";
import { SITE } from "../../lib/data/site.ts";
import { pageMetadata } from "../../lib/metadata.ts";

export const metadata: Metadata = pageMetadata({
  title: "Support, feedback and security reports",
  description:
    "Write to support@quotadial.com, open an issue on GitHub with the details support needs, or report a security problem privately, as SECURITY.md describes.",
  path: "/support",
});

const MAIL = `mailto:${SITE.support}`;
const MAIL_EVENT = { "data-event": "support_mail_click" } as const;
const ISSUE_EVENT = { "data-event": "issue_click" } as const;
/** The issue forms in .github/ISSUE_TEMPLATE, by name, with the line each gives for itself. */
const TEMPLATES = [
  { name: "Bug report", what: "Something in QuotaDial is broken or shows the wrong numbers." },
  { name: "Feature request", what: "Suggest something QuotaDial should do." },
  { name: "Provider request", what: "Ask for QuotaDial to track another AI subscription or coding tool." },
] as const;
const H2 = "m-0 text-[2rem] leading-[1.05] font-[750] md:text-[2.75rem]";

export default function Support() {
  return (
    <>
      <Section>
        <div className="grid gap-14 lg:grid-cols-12 lg:gap-x-12">
          <div className="lg:col-span-5">
            <PageIntro
              title="Get help with QuotaDial"
              answer="Write to support for questions and feedback, open an issue on GitHub for a bug others can see, or report a security problem in private."
            >
              <ButtonLink href={MAIL} data-primary-action="" data-event-channel="support" {...MAIL_EVENT}>
                Write to {SITE.support}
              </ButtonLink>
              <ButtonLink href={SITE.newIssue} variant="secondary" {...ISSUE_EVENT}>
                Open an issue on GitHub
              </ButtonLink>
            </PageIntro>
          </div>
          <section aria-labelledby="write" className="lg:col-span-7">
            <h2 id="write" className={H2}>
              Write to support
            </h2>
            <p className="mt-4 mb-0 max-w-[54ch]">
              Say what you were doing and what you saw. Run this first and paste what it prints: it names your
              version, your system and each account (with its address cut short), and nothing secret.
            </p>
            <div className="mt-5 max-w-[34rem]">
              <CodeBlock code="quotadial doctor" label="Copy the command" />
            </div>
            <p className="mt-5 mb-0 max-w-[54ch]">
              In the app, Help has Send feedback, which fills in the same details as a GitHub issue for you to read
              before you send it. Before you write, <InlineLink href="/changelog">the changelog</InlineLink> shows whether a newer
              version already fixes it, and <InlineLink href="/privacy">what QuotaDial reads and sends</InlineLink> says what leaves
              your machine.
            </p>
          </section>
        </div>
      </Section>

      <Section tone="shallow" labelledBy="github">
        <div className="grid gap-10 lg:grid-cols-12 lg:gap-x-12">
          <div className="lg:col-span-5">
            <h2 id="github" className={H2}>
              Report a problem on GitHub
            </h2>
            <p className="mt-4 mb-0 max-w-[48ch]">
              A bug, or something you want changed, goes best in an issue, where others with the same trouble can
              find it. <InlineLink href={SITE.newIssue} {...ISSUE_EVENT}>Open one on GitHub</InlineLink> and paste the
              output of <code className="rounded-chart bg-mid px-1 font-code text-[0.875em]">quotadial doctor</code>{" "}
              into it. The form has a template for each kind:
            </p>
            <dl className="mt-6 mb-0 grid gap-4">
              {TEMPLATES.map((template) => (
                <div key={template.name}>
                  <dt className="font-display text-[1.25rem] leading-[1.2] font-[700]">{template.name}</dt>
                  <dd className="m-0 mt-1 max-w-[44ch] text-ink-2">{template.what}</dd>
                </div>
              ))}
            </dl>
          </div>
          <div className="min-w-0 lg:col-span-7">
            <DoctorSample />
          </div>
        </div>
      </Section>

      <Section tone="mid" labelledBy="security">
        <div className="grid gap-10 lg:grid-cols-12 lg:gap-x-12">
          <div className="lg:col-span-6">
            <h2 id="security" className={H2}>
              Report a security problem
            </h2>
            <p className="mt-4 mb-0 max-w-[54ch]">
              Report it privately, and please do not open a public issue or pull request for it. Say what you found,
              how to make it happen and what it lets someone do. You will hear back within 3 working days;{" "}
              <InlineLink href={`${SITE.repository}/blob/main/SECURITY.md`}>SECURITY.md</InlineLink> has the policy in full.
            </p>
          </div>
          <div className="lg:col-span-5 lg:col-start-8">
            <Plate tone="chart">
              <p className="m-0 font-display text-[1.25rem] font-[700]">Two private ways</p>
              <ul className="mt-3 mb-0 grid list-none gap-3 p-0">
                <li>
                  <InlineLink href={SITE.securityReport}>Use GitHub&rsquo;s private vulnerability reporting</InlineLink>
                </li>
                <li>
                  <InlineLink href={`${MAIL}?subject=Security`} data-event-channel="security" {...MAIL_EVENT}>
                    Write to {SITE.support}
                  </InlineLink>{" "}
                  with &ldquo;Security&rdquo; in the subject
                </li>
              </ul>
            </Plate>
          </div>
        </div>
      </Section>
    </>
  );
}
