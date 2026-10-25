import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "preact/hooks";

import type { HelpView, UpdateView } from "./meter";
import { UpdateStatus } from "./update";

/**
 * Help: what is new in this version, Send feedback, the support address, About with updates, and
 * what QuotaDial reads and sends. Links open in the person's own browser or mail app, and only
 * the ones named here.
 */
export function HelpPanel({ update, onDone }: { update: UpdateView | null; onDone: () => void }) {
  const [help, setHelp] = useState<HelpView | null>(null);
  const [problem, setProblem] = useState<string | null>(null);

  useEffect(() => {
    void invoke<HelpView>("help").then(setHelp);
  }, []);

  if (!help) {
    return <section class="help" aria-label="Help" />;
  }

  const run = (command: string, args?: Record<string, string>) => {
    invoke(command, args)
      .then(() => setProblem(null))
      .catch((error: unknown) => setProblem(String(error)));
  };
  const heading =
    help.whats_new.version === "Unreleased"
      ? "What's new since the last release"
      : `What's new in ${help.whats_new.version}`;

  return (
    <section class="help" aria-label="Help">
      <h2>Help</h2>
      <section class="whats-new" aria-label={heading}>
        <h3>{heading}</h3>
        {help.whats_new.sections.map((section) => (
          <div key={section.heading}>
            <p class="detail">{section.heading}</p>
            <ul>
              {section.items.map((item) => (
                <li key={item}>{item}</li>
              ))}
            </ul>
          </div>
        ))}
        <button type="button" class="link" onClick={() => run("open_link", { link: "changelog" })}>
          Every change
        </button>
      </section>
      <section aria-label="Feedback and support">
        <h3>Feedback and support</h3>
        <p class="detail">
          Send feedback opens a new GitHub issue in your browser, with what support asks for
          filled in: the version, the system, each account and where its sign-in is, and the
          settings. It holds no token, no whole email address and no licence id, and nothing is
          sent until you send it.
        </p>
        <div class="help-actions">
          <button type="button" onClick={() => run("send_feedback")}>
            Send feedback
          </button>
          <button type="button" onClick={() => run("open_link", { link: "support" })}>
            Write to {help.support}
          </button>
        </div>
      </section>
      <section aria-label="About QuotaDial">
        <h3>About</h3>
        <p>
          QuotaDial {help.version}. {help.licence}
        </p>
        {update && <UpdateStatus update={update} />}
        {help.credits.map((credit) => (
          <p class="detail" key={credit}>
            {credit}
          </p>
        ))}
        <button type="button" class="link" onClick={() => run("open_link", { link: "site" })}>
          quotadial.com
        </button>
      </section>
      <section class="privacy" aria-label="What QuotaDial reads and sends">
        <h3>What QuotaDial reads and sends</h3>
        {help.privacy.map((block, index) =>
          block.kind === "paragraph" ? (
            <p key={index}>{block.text}</p>
          ) : (
            <ul key={index}>
              {block.items.map((item) => (
                <li key={item}>{item}</li>
              ))}
            </ul>
          ),
        )}
      </section>
      {problem && (
        <p class="note" role="alert">
          {problem}
        </p>
      )}
      <button type="button" class="done" onClick={onDone}>
        Done
      </button>
    </section>
  );
}
