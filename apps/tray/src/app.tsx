import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "preact/hooks";

import type { Account, Meter, Window } from "./meter";
import { SettingsPanel } from "./settings";

/** The popover: the tightest window's share used, every account's windows, and what is wrong. */
export function App() {
  const [meter, setMeter] = useState<Meter | null>(null);
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

  // The window takes the height of what it shows.
  useEffect(() => {
    const element = main.current;
    if (!element) {
      return;
    }
    let last = 0;
    const observer = new ResizeObserver(() => {
      const height = Math.ceil(element.getBoundingClientRect().height);
      if (height !== last) {
        last = height;
        void invoke("fit", { height });
      }
    });
    observer.observe(element);
    return () => observer.disconnect();
  }, []);

  return (
    <main class="popover" ref={main} aria-label="QuotaDial">
      {meter && <Contents meter={meter} />}
    </main>
  );
}

function Contents({ meter }: { meter: Meter }) {
  const [settingsOpen, setSettingsOpen] = useState(false);
  return (
    <>
      <header class="summary">
        <p class={`percent ${meter.level}`}>
          {meter.percent === null ? "–" : `${meter.percent}%`}
        </p>
        <p class="headline">{meter.headline ?? "Looking for the accounts on this machine…"}</p>
      </header>
      {settingsOpen && <SettingsPanel onDone={() => setSettingsOpen(false)} />}
      {!settingsOpen &&
        meter.accounts.map((account, index) => (
          <AccountSection
            key={account.key}
            account={account}
            index={index}
          />
        ))}
      {!settingsOpen && meter.problems.length > 0 && (
        <section class="problems" aria-label="Problems">
          <ul>
            {meter.problems.map((problem) => (
              <li key={problem}>{problem}</li>
            ))}
          </ul>
        </section>
      )}
      <footer>
        <p class="origin">{meter.origin ?? ""}</p>
        {!settingsOpen && (
          <button type="button" onClick={() => setSettingsOpen(true)}>
            Settings
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
        <span class="name">{account.name}</span>
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
      <details class="sign-in">
        <summary>How QuotaDial reads this sign-in</summary>
        <p>{account.sign_in}</p>
      </details>
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
