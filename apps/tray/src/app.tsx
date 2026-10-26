import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "preact/hooks";

import { HelpPanel } from "./help";
import { HistoryPanel } from "./history";
import type { Account, Cost, Meter, UpdateView, Window } from "./meter";
import { Said } from "./said";
import { SettingsPanel } from "./settings";
import { UpdateStatus, useUpdate } from "./update";

/** The popover: the tightest window's share used, every account's windows, and what is wrong. */
export function App() {
  const [meter, setMeter] = useState<Meter | null>(null);
  const update = useUpdate();
  const main = useRef<HTMLElement>(null);

  useEffect(() => {
    let stopListening: (() => void) | undefined;
    let closed = false;
    void listen<Meter>("meter", (event) => setMeter(event.payload)).then((stop) => {
      if (closed) {
        stop();
      } else {
        stopListening = stop;
      }
    });
    void invoke<Meter>("meter").then(setMeter);
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        void invoke("hide");
      }
    };
    window.addEventListener("keydown", onKey);
    return () => {
      closed = true;
      stopListening?.();
      window.removeEventListener("keydown", onKey);
    };
  }, []);

  // The window takes the height of what it shows, at the zoom it is shown at: zooming in (Cmd or
  // Ctrl and +) narrows the page in CSS pixels, which tells the app how far it is zoomed.
  useEffect(() => {
    const element = main.current;
    if (!element) {
      return;
    }
    let last = "";
    const fit = () => {
      const height = Math.ceil(element.getBoundingClientRect().height);
      const width = window.innerWidth;
      if (`${height} ${width}` !== last) {
        last = `${height} ${width}`;
        void invoke("fit", { height, width });
      }
    };
    const observer = new ResizeObserver(fit);
    observer.observe(element);
    window.addEventListener("resize", fit);
    return () => {
      observer.disconnect();
      window.removeEventListener("resize", fit);
    };
  }, []);

  return (
    <main class="popover" ref={main} aria-label="QuotaDial">
      {meter && <Contents meter={meter} update={update} />}
    </main>
  );
}

/** What the popover shows below its number: the accounts, the settings, the history, or Help. */
type Panel = "accounts" | "settings" | "history" | "help";

function Contents({ meter, update }: { meter: Meter; update: UpdateView | null }) {
  const [panel, setPanel] = useState<Panel>("accounts");
  const accounts = panel === "accounts";
  const footer = useRef<HTMLElement>(null);
  const shown = useRef<Panel>("accounts");

  // A panel's Done gives the focus back to the button that opened it, so the keyboard carries on
  // from where it was (#52).
  useEffect(() => {
    const closed = shown.current;
    shown.current = panel;
    if (panel === "accounts" && closed !== "accounts") {
      footer.current?.querySelector<HTMLElement>(`[data-panel="${closed}"]`)?.focus();
    }
  }, [panel]);

  // The tray menu's Help opens the popover at Help.
  useEffect(() => {
    let stopListening: (() => void) | undefined;
    let closed = false;
    void listen<Panel>("panel", (event) => setPanel(event.payload)).then((stop) => {
      if (closed) {
        stop();
      } else {
        stopListening = stop;
      }
    });
    return () => {
      closed = true;
      stopListening?.();
    };
  }, []);
  return (
    <>
      <header class="summary">
        {/* With no number yet, the dash is only for the eye; the headline says why. */}
        <p class={`percent ${meter.level}`} aria-hidden={meter.percent === null ? "true" : undefined}>
          {meter.percent === null ? "–" : `${meter.percent}%`}
        </p>
        <p class="headline">{meter.headline ?? "Looking for the accounts on this machine…"}</p>
        {meter.incidents.map((incident) => (
          <p class="incident" key={incident} role="status">
            {"\u26a0"} {incident}
          </p>
        ))}
      </header>
      {panel === "settings" && <SettingsPanel onDone={() => setPanel("accounts")} />}
      {panel === "history" && <HistoryPanel onDone={() => setPanel("accounts")} />}
      {panel === "help" && <HelpPanel update={update} onDone={() => setPanel("accounts")} />}
      {accounts &&
        meter.accounts.map((account, index) => (
          <AccountSection
            key={account.key}
            account={account}
            index={index}
          />
        ))}
      {accounts && meter.problems.length > 0 && (
        <section class="problems card" aria-label="Problems">
          <ul>
            {meter.problems.map((problem) => (
              <li key={problem}>
                <Said text={problem} />
              </li>
            ))}
          </ul>
        </section>
      )}
      {accounts && meter.left_out.length > 0 && (
        <section class="left-out" aria-label="Part of QuotaDial Pro">
          {meter.left_out.map((line) => (
            <p key={line}>{line}</p>
          ))}
        </section>
      )}
      {accounts && meter.costs.length > 0 && <Costs costs={meter.costs} />}
      {accounts && update?.available && (
        <section class="update-offer card" aria-label="Update">
          <UpdateStatus update={update} offer />
        </section>
      )}
      {/* A calm toolbar in line with the cards' text, Quit set apart at its far end, and under it
          where the numbers come from. */}
      <footer ref={footer}>
        <div class="toolbar">
          {accounts && (
            <button type="button" data-panel="history" onClick={() => setPanel("history")}>
              History
            </button>
          )}
          {accounts && (
            <button type="button" data-panel="settings" onClick={() => setPanel("settings")}>
              Settings
            </button>
          )}
          {accounts && (
            <button type="button" data-panel="help" onClick={() => setPanel("help")}>
              Help
            </button>
          )}
          <button type="button" class="quit" onClick={() => void invoke("quit")}>
            Quit QuotaDial
          </button>
        </div>
        {meter.origin && <p class="origin">{meter.origin}</p>}
      </footer>
    </>
  );
}

/**
 * One account, as a card of its own: who it is in its first two lines, then each window. The
 * heading holds both lines, so a screen reader hears the account's address with its name.
 */
function AccountSection({ account, index }: { account: Account; index: number }) {
  const heading = `account-${index}`;
  return (
    <section class="account card" aria-labelledby={heading}>
      <h2 id={heading} class="card-head">
        {/* The spaces between the parts are for a screen reader; a flex row draws none. */}
        <span class="title-row">
          {/* The name given the account, or the provider's own (P-020). */}
          <span class="title">{account.alias ?? account.provider}</span>{" "}
          {(account.plan || !account.ok) && (
            <span class="badges">
              {account.plan && <span class="badge">{account.plan}</span>}
              {account.plan && !account.ok && " "}
              {!account.ok && <span class="badge attention">{account.status}</span>}
            </span>
          )}
        </span>
        <Address account={account} />
      </h2>
      {!account.ok && account.note && (
        <p class="note">
          <Said text={account.note} />
        </p>
      )}
      {account.windows.length > 0 && (
        <ul class="windows">
          {account.windows.map((window, windowIndex) => (
            <WindowRow
              key={window.label}
              window={window}
              id={`${heading}-window-${windowIndex}`}
            />
          ))}
        </ul>
      )}
      {account.extra_usage && <p class="extra">Extra usage: {account.extra_usage}</p>}
      {account.sign_in && (
        <details class="sign-in">
          <summary>What QuotaDial does with this sign-in</summary>
          <p>
            <Said text={account.sign_in} />
          </p>
        </details>
      )}
    </section>
  );
}

/**
 * An account's second line: its address, after its provider's name when the title is the name the
 * user gave it, as "Claude · dev@example.com". Nothing when the provider's name says it all, as
 * for the example build's accounts, which have no address (D-124). The address alone can be
 * selected, to copy it.
 */
function Address({ account }: { account: Account }) {
  const named = account.alias !== null;
  if (!named && !account.name) {
    return null;
  }
  return (
    <>
      {" "}
      <span class="address">
        {named && <span class="provider">{account.provider}</span>}
        {named && account.name && (
          <>
            {" "}
            <span aria-hidden="true">·</span>{" "}
          </>
        )}
        {account.name && <span class="name">{account.name}</span>}
      </span>
    </>
  );
}

/** What each provider's week would have cost at API prices, from the CLIs' logs. */
function Costs({ costs }: { costs: Cost[] }) {
  return (
    <section class="costs card" aria-label="The week at API prices">
      {costs.map((cost) => (
        <div class="cost" key={cost.provider}>
          <p>
            <span class="provider">{cost.provider}</span> {cost.summary}
          </p>
          <p class="detail">{cost.detail}</p>
          {cost.models.length > 0 && (
            <details class="models">
              <summary>By model</summary>
              <ul>
                {cost.models.map((model) => (
                  <li key={model}>{model}</li>
                ))}
              </ul>
            </details>
          )}
          {cost.notes.map((note) => (
            <p class="detail" key={note}>
              {note}
            </p>
          ))}
        </div>
      ))}
    </section>
  );
}

function WindowRow({ window, id }: { window: Window; id: string }) {
  const percent = Math.floor(window.used_pct);
  return (
    <li class="window">
      <div class="row">
        <span class="label" id={id}>
          {window.label}
        </span>
        <span class={`used ${window.level}`}>{percent}%</span>
      </div>
      <meter
        min={0}
        max={100}
        low={70}
        high={90}
        optimum={0}
        value={window.used_pct}
        aria-labelledby={id}
        aria-valuetext={`${percent}% used, ${window.resets}`}
      />
      <p class="detail">
        {window.resets}, {window.fetched}
        {window.unconfirmed && (
          <>
            {" "}
            <span
              class="badge"
              title="A sudden 0% that QuotaDial holds until the next fetch agrees"
            >
              unconfirmed
            </span>
          </>
        )}
      </p>
      {window.pace && (
        <p class={window.runs_out ? "pace runs-out" : "pace"}>{window.pace}</p>
      )}
    </li>
  );
}
