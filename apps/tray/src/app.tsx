import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "preact/hooks";

import { HelpPanel } from "./help";
import { HistoryPanel } from "./history";
import type { Account, Cost, Meter, UpdateView, Window } from "./meter";
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
        <section class="problems" aria-label="Problems">
          <ul>
            {meter.problems.map((problem) => (
              <li key={problem}>{problem}</li>
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
        <section class="update-offer" aria-label="Update">
          <UpdateStatus update={update} offer />
        </section>
      )}
      <footer ref={footer}>
        <p class="origin">{meter.origin ?? ""}</p>
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
        <button type="button" onClick={() => void invoke("quit")}>
          Quit QuotaDial
        </button>
      </footer>
    </>
  );
}

function AccountSection({ account, index }: { account: Account; index: number }) {
  const heading = `account-${index}`;
  return (
    <section class="account" aria-labelledby={heading}>
      <h2 id={heading}>
        <span class="provider">{account.provider}</span>
        {account.alias && <span class="alias">{account.alias}</span>}
        {account.name && <span class="name">{account.name}</span>}
        {account.plan && <span class="tag">{account.plan}</span>}
        {!account.ok && <span class="tag attention">{account.status}</span>}
      </h2>
      {!account.ok && account.note && <p class="note">{account.note}</p>}
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
          <p>{account.sign_in}</p>
        </details>
      )}
    </section>
  );
}

/** What each provider's week would have cost at API prices, from the CLIs' logs. */
function Costs({ costs }: { costs: Cost[] }) {
  return (
    <section class="costs" aria-label="The week at API prices">
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
          <span
            class="tag"
            title="A sudden 0% that QuotaDial holds until the next fetch agrees"
          >
            unconfirmed
          </span>
        )}
      </p>
      {window.pace && (
        <p class={window.runs_out ? "pace runs-out" : "pace"}>{window.pace}</p>
      )}
    </li>
  );
}
