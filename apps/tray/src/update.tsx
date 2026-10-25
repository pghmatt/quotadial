import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "preact/hooks";

import type { UpdateView } from "./meter";

/** Where updates stand: asked for once, then kept up to date by the `update` event. */
export function useUpdate(): UpdateView | null {
  const [update, setUpdate] = useState<UpdateView | null>(null);

  useEffect(() => {
    let stopListening: (() => void) | undefined;
    let closed = false;
    void listen<UpdateView>("update", (event) => setUpdate(event.payload)).then((stop) => {
      if (closed) {
        stop();
      } else {
        stopListening = stop;
      }
    });
    void invoke<UpdateView>("updates").then(setUpdate);
    return () => {
      closed = true;
      stopListening?.();
    };
  }, []);
  return update;
}

/**
 * Where updates stand, in one line, with what the person can do next: Update when a newer version
 * is on offer, and in Help, Check for updates, which also asks again after an install that failed.
 * A screen reader hears each step, not each percent.
 */
export function UpdateStatus({ update, offer }: { update: UpdateView; offer?: boolean }) {
  const step = update.text.replace(/ \d+%$/, "");
  return (
    <div class="update">
      <p aria-hidden="true">{update.text}</p>
      <p class="visually-hidden" role="status">
        {step}
      </p>
      {update.progress !== null && (
        <progress max={100} value={update.progress} aria-label={step} />
      )}
      {update.available && update.notes.length > 0 && (
        <details class="update-notes">
          <summary>What's new in {update.available}</summary>
          <ul>
            {update.notes.map((note) => (
              <li key={note}>{note}</li>
            ))}
          </ul>
        </details>
      )}
      {!update.busy && (update.available || !offer) && (
        <div class="help-actions">
          {update.available && (
            <button type="button" onClick={() => void invoke("install_update")}>
              Update to {update.available}
            </button>
          )}
          {!offer && (
            <button type="button" onClick={() => void invoke("check_for_updates")}>
              Check for updates
            </button>
          )}
        </div>
      )}
    </div>
  );
}
