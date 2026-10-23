import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "preact/hooks";

import type { Meter } from "./meter";

/** The window: the tightest window's share used, then every account and anything wrong. */
export function App() {
  const [meter, setMeter] = useState<Meter | null>(null);

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
    return () => {
      closed = true;
      stopListening?.();
    };
  }, []);

  if (!meter) {
    return <main class="meter" />;
  }
  return (
    <main class="meter">
      <header>
        <p class={`percent ${meter.level}`}>
          {meter.percent === null ? "–" : `${meter.percent}%`}
        </p>
        <p class="caption">
          {meter.percent === null
            ? "No numbers yet"
            : "used of the tightest limit across your accounts"}
        </p>
      </header>
      {meter.accounts.length > 0 && (
        <ul class="accounts">
          {meter.accounts.map((line) => (
            <li key={line}>{line}</li>
          ))}
        </ul>
      )}
      {meter.problems.length > 0 && (
        <ul class="problems">
          {meter.problems.map((line) => (
            <li key={line}>{line}</li>
          ))}
        </ul>
      )}
    </main>
  );
}
