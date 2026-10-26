import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useLayoutEffect, useRef, useState } from "preact/hooks";

import { AccountHeading } from "./account";
import { HelpPanel } from "./help";
import { HistoryPanel } from "./history";
import type { Account, Cost, Meter, UpdateView, Window } from "./meter";
import { Said } from "./said";
import { neededHeight, useScrolling } from "./scroll";
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
  // Ctrl and +) narrows the page in CSS pixels, which tells the app how far it is zoomed. Once the
  // window is as tall as it may be, the popover's own height stops growing with what it holds, so
  // the parts it is made of are watched too, and `neededHeight` adds up what they hold.
  const loaded = meter !== null;
  useEffect(() => {
    const element = main.current;
    if (!element) {
      return;
    }
    let last = "";
    const fit = () => {
      const height = neededHeight(element);
      const width = window.innerWidth;
      if (`${height} ${width}` !== last) {
        last = `${height} ${width}`;
        void invoke("fit", { height, width });
      }
    };
    const observer = new ResizeObserver(fit);
    observer.observe(element);
    for (const part of element.querySelectorAll(".content, footer")) {
      observer.observe(part);
    }
    window.addEventListener("resize", fit);
    return () => {
      observer.disconnect();
      window.removeEventListener("resize", fit);
    };
  }, [loaded]);

  return (
    <main class="popover" ref={main} aria-label="QuotaDial">
      {meter && <Contents meter={meter} update={update} />}
    </main>
  );
}

/** What the popover shows below its number: the accounts, the settings, the history, or Help. */
type Panel = "accounts" | "settings" | "history" | "help";

/** What each is called, for the scrolling part to be named by when a keyboard has to reach it. */
const VIEWS: Record<Panel, string> = {
  accounts: "Accounts",
  settings: "Settings",
  history: "History",
  help: "Help",
};

function Contents({ meter, update }: { meter: Meter; update: UpdateView | null }) {
  const [panel, setPanel] = useState<Panel>("accounts");
  const accounts = panel === "accounts";
  const footer = useRef<HTMLElement>(null);
  const region = useRef<HTMLDivElement>(null);
  const shown = useRef<Panel>("accounts");
  const scrolling = useScrolling(region, panel);

  // Each view starts at its top, as the popover opens at its top, and not where the last was left.
  useLayoutEffect(() => {
    if (region.current) {
      region.current.scrollTop = 0;
    }
  }, [panel]);

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
      <div
        class="scroll"
        ref={region}
        tabIndex={scrolling.stop ? 0 : undefined}
        role={scrolling.stop ? "region" : undefined}
        aria-label={scrolling.stop ? VIEWS[panel] : undefined}
      >
        <div class="content">
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
          {panel === "settings" && <SettingsPanel />}
          {panel === "history" && <HistoryPanel />}
          {panel === "help" && <HelpPanel update={update} />}
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
        </div>
      </div>
      {/* One calm toolbar at the foot of every view, in line with the cards' text: History,
          Settings and Help under the accounts, Done under a panel, and Quit set apart at its far
          end; under it, where the numbers come from. It stays where it is while what is above it
          scrolls. */}
      <footer ref={footer} class={scrolling.below ? "edge" : undefined}>
        <div class="toolbar">
          {accounts ? (
            <>
              <button type="button" data-panel="history" onClick={() => setPanel("history")}>
                History
              </button>
              <button type="button" data-panel="settings" onClick={() => setPanel("settings")}>
                Settings
              </button>
              <button type="button" data-panel="help" onClick={() => setPanel("help")}>
                Help
              </button>
            </>
          ) : (
            <button type="button" class="done" onClick={() => setPanel("accounts")}>
              Done
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
      <AccountHeading who={account} id={heading} level={2}>
        {(account.plan || !account.ok) && (
          <span class="badges">
            {account.plan && <span class="badge">{account.plan}</span>}
            {account.plan && !account.ok && " "}
            {!account.ok && <span class="badge attention">{account.status}</span>}
          </span>
        )}
      </AccountHeading>
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
      {/* Where and when every window was read, said once for all of them: the card's caption, under
          the numbers it is about and above the sign-in, whose panel opens below it. */}
      {account.fetched && <p class="source">{account.fetched}</p>}
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
        {window.resets}
        {window.fetched && `, ${window.fetched}`}
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
