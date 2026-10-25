"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import type { ReactNode } from "react";

import { track } from "../../lib/events.ts";
import type { Licence } from "../../lib/licence.ts";
import { SITE } from "../../lib/data/site.ts";
import { ButtonLink } from "../Button.tsx";
import { CodeBlock } from "../CodeBlock.tsx";
import { CopyButton } from "../CopyButton.tsx";
import { Notice } from "../Notice.tsx";
import { Plate } from "../Plate.tsx";
import { longDay, receiptKey, requestKey } from "./key-state.ts";
import { Steps } from "./Steps.tsx";

type State =
  | { step: "none" }
  | { step: "checking"; receipt: string }
  | { step: "ready"; key: string; licence: Licence }
  | { step: "refused"; receipt: string; message: string; retry: boolean };

const SUPPORT_MAIL = `mailto:${SITE.support}?subject=${encodeURIComponent("My QuotaDial key")}`;
const LINK = "underline underline-offset-[0.2em]";
const LABEL = "m-0 mb-2 font-display text-[1.125rem] font-[700]";
const H2 = "m-0 text-[1.75rem] leading-[1.1] font-[750]";

/** What the page says beside its heading, for each step. */
const LEAD: Record<State["step"], string> = {
  none: "This page turns the link in your Lemon Squeezy receipt into your QuotaDial key.",
  checking: "Checking the key from your receipt with Lemon Squeezy. This takes a moment.",
  ready: "Your key is ready. Copy it, then paste it into QuotaDial.",
  refused: "Here is what went wrong, and what to do next.",
};

/** The most of a key from the address that is shown; a real key is 36 characters. */
const SHOWN = 120;
const shown = (text: string) => (text.length > SHOWN ? `${text.slice(0, SHOWN)}…` : text);

/** A key or an address in the code face, broken wherever the line ends so none is cut off. */
function Mono({ children, label }: { children: ReactNode; label: string }) {
  return (
    <p
      aria-label={label}
      className="m-0 rounded-chart bg-mid p-4 font-code text-[0.9375rem] leading-[1.6] break-all text-ink select-all"
    >
      {children}
    </p>
  );
}

/** The key page. The heading is the page's own, passed in so it is in the HTML before this runs. */
export function KeyFlow({ heading }: { heading: ReactNode }) {
  const [state, setState] = useState<State>({ step: "none" });
  const asking = useRef<AbortController | null>(null);

  const ask = useCallback((receipt: string) => {
    asking.current?.abort();
    const controller = new AbortController();
    asking.current = controller;
    setState({ step: "checking", receipt });
    void requestKey(receipt, (url, init) => fetch(url, init), controller.signal).then((answer) => {
      if (controller.signal.aborted) {
        return;
      }
      setState(
        answer.ok
          ? { step: "ready", key: answer.key, licence: answer.licence }
          : { step: "refused", receipt, message: answer.message, retry: answer.retry },
      );
    });
  }, []);

  // Read the receipt's key once, take it out of the address so the browser's history does not keep
  // it, and ask for the QuotaDial key.
  useEffect(() => {
    const receipt = receiptKey(window.location.hash);
    if (receipt !== null) {
      window.history.replaceState(null, "", window.location.pathname + window.location.search);
      ask(receipt);
    }
    return () => asking.current?.abort();
  }, [ask]);

  useEffect(() => {
    if (state.step === "ready") {
      track("key_shown");
    }
  }, [state.step]);

  return (
    <div className="grid items-start gap-x-6 gap-y-8 grid-cols-[minmax(0,1fr)] md:grid-cols-12">
      <div className="md:col-span-5">
        {heading}
        <p role="status" className="m-0 mt-5 max-w-[34ch] text-[1.25rem] leading-[1.4] text-ink-2">
          {LEAD[state.step]}
        </p>
      </div>
      <div className="md:col-span-7">
        {state.step === "none" && <NoLink />}
        {state.step === "checking" && <Checking receipt={state.receipt} />}
        {state.step === "ready" && <Ready keyText={state.key} licence={state.licence} />}
        {state.step === "refused" && (
          <Refused message={state.message} receipt={state.receipt} retry={state.retry} onRetry={() => ask(state.receipt)} />
        )}
      </div>
    </div>
  );
}

/** Where the key page's visitors can go next, whatever happened. */
function Onward({ pro = true }: { pro?: boolean }) {
  return (
    <p className="m-0 mt-6 max-w-[52ch] text-ink-2">
      <a href="/download" className={`${LINK} text-ink`}>
        Download QuotaDial
      </a>
      {pro ? ", " : " or "}
      {pro && (
        <>
          <a href="/pro" className={`${LINK} text-ink`}>
            see what Pro adds
          </a>{" "}
          or{" "}
        </>
      )}
      <a href="/support" className={`${LINK} text-ink`}>
        get help with QuotaDial
      </a>
      .
    </p>
  );
}

function SupportButton({ primary = true }: { primary?: boolean }) {
  return (
    <ButtonLink
      href={SUPPORT_MAIL}
      variant={primary ? "primary" : "secondary"}
      data-primary-action={primary ? "" : undefined}
      data-event="support_mail_click"
      data-event-from="key"
    >
      Write to support
    </ButtonLink>
  );
}

/** No link in the address: how to get here from the receipt. */
function NoLink() {
  return (
    <div data-none-state>
      <Plate>
        <h2 className={H2}>Where to find the link</h2>
        <div className="mt-4">
          <Steps>
            <li>Open the receipt Lemon Squeezy emailed you when you bought QuotaDial Pro.</li>
            <li>
              Choose its button, <strong>Get your QuotaDial key</strong>.
            </li>
            <li>This page opens again with your key, ready to copy.</li>
          </Steps>
        </div>
      </Plate>
      <noscript>
        <p className="m-0 mt-6 max-w-[52ch] font-[650]">
          This page needs JavaScript to turn the receipt's key into your QuotaDial key. Write to support with your
          order number instead.
        </p>
      </noscript>
      <p className="m-0 mt-6 max-w-[52ch]">
        Can't find the receipt? Write to support with your order number, and we'll send your key.
      </p>
      <div className="mt-5">
        <SupportButton />
      </div>
      <Onward />
    </div>
  );
}

/** The receipt's key is being checked: the shape of the answer is already on the page. */
function Checking({ receipt }: { receipt: string }) {
  return (
    <div>
      <Plate>
        <p className={LABEL}>The key from your receipt</p>
        <Mono label="The key from your receipt">{shown(receipt)}</Mono>
        <p className="mt-6 mb-2 font-display text-[1.125rem] font-[700]">Your QuotaDial key</p>
        <div aria-hidden className="grid grid-cols-[minmax(0,1fr)] gap-2 rounded-chart bg-mid p-4">
          {["w-full", "w-11/12", "w-2/5"].map((width, index) => (
            <span key={index} className={`block h-3 rounded-chart bg-shallow ${width}`} />
          ))}
        </div>
      </Plate>
      <div className="mt-6 flex flex-wrap items-center gap-x-4 gap-y-2">
        <button
          type="button"
          disabled
          className="inline-flex items-center rounded-chart border border-current px-5 py-3 font-display text-[1.125rem] leading-none font-[650] opacity-60"
        >
          Copy the key
        </button>
        <span className="text-ink-2">Ready when the check is done.</span>
      </div>
    </div>
  );
}

/** The QuotaDial key, with the one action that matters and the two ways to add it. */
function Ready({ keyText, licence }: { keyText: string; licence: Licence }) {
  return (
    <div>
      <Plate
        caption={`Licence ${licence.id}, issued ${longDay(licence.issued)}${licence.until ? `, through ${longDay(licence.until)}` : ""}.`}
      >
        <p className={LABEL}>Your QuotaDial key</p>
        <Mono label="Your QuotaDial key">{keyText}</Mono>
        <div className="mt-4">
          <CopyButton text={keyText} label="Copy the key" event="key_copied" primary />
        </div>
      </Plate>

      <h2 className={`${H2} mt-10`}>Add it to QuotaDial</h2>
      <div className="mt-4 grid grid-cols-[minmax(0,1fr)] gap-5">
        <p className="m-0 max-w-[56ch]">
          In the app, open <strong>Settings</strong>, find <strong>Licence</strong>, paste the key into the field and
          choose <strong>Add</strong>.
        </p>
        <div className="grid grid-cols-[minmax(0,1fr)] gap-2">
          <p className="m-0 max-w-[56ch]">In a terminal, run this and paste the key when it asks:</p>
          <CodeBlock code="quotadial licence add" label="Copy the command" />
        </div>
        <p className="m-0 max-w-[56ch]">
          Then forget about it. QuotaDial checks the key on your machine with no network call
          {licence.until ? "." : ", and a licence with no last day works offline for ever."}
        </p>
      </div>
      <p className="m-0 mt-6 max-w-[52ch] text-ink-2">
        To see this key again, open the receipt's link again: the same receipt always gives the same key.
      </p>
      <Onward pro={false} />
    </div>
  );
}

/** The function's own sentence, and the next step it calls for. */
function Refused({
  message,
  receipt,
  retry,
  onRetry,
}: {
  message: string;
  receipt: string;
  retry: boolean;
  onRetry: () => void;
}) {
  return (
    <div>
      <Notice title={retry ? "The key could not be checked just now" : "The key was not accepted"}>
        <p role="alert" className="m-0 max-w-[56ch] text-[1.125rem] leading-[1.45]">
          {message}
        </p>
      </Notice>
      <div className="mt-6 flex flex-wrap items-center gap-x-4 gap-y-4">
        {retry ? (
          <>
            <button
              type="button"
              onClick={onRetry}
              data-primary-action=""
              className="inline-flex items-center rounded-chart bg-magenta-fill px-5 py-3 font-display text-[1.125rem] leading-none font-[650] whitespace-nowrap text-on-magenta transition-colors hover:bg-[color-mix(in_srgb,var(--magenta-fill)_86%,black)] active:translate-y-px"
            >
              Try again
            </button>
            <SupportButton primary={false} />
          </>
        ) : (
          <SupportButton />
        )}
      </div>
      <p className={`${LABEL} mt-8`}>The key this page received from the link</p>
      <Mono label="The key this page received from the link">{shown(receipt) || "(nothing after licence=)"}</Mono>
      <Onward />
    </div>
  );
}
