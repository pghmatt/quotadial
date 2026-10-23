import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "preact/hooks";

import type { Settings } from "./meter";

/** The settings panel: opening at login, the two notifications, and which providers to show. */
export function SettingsPanel({ onDone }: { onDone: () => void }) {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [problem, setProblem] = useState<string | null>(null);

  useEffect(() => {
    void invoke<Settings>("settings").then(setSettings);
  }, []);

  if (!settings) {
    return <section class="settings" aria-label="Settings" />;
  }

  // Each change applies at once; the app answers with the settings as they now are.
  const change = (next: Settings) => {
    setSettings(next);
    invoke<Settings>("set_settings", { view: next })
      .then((saved) => {
        setSettings(saved);
        setProblem(null);
      })
      .catch((error: unknown) => {
        setProblem(String(error));
        void invoke<Settings>("settings").then(setSettings);
      });
  };

  return (
    <section class="settings" aria-label="Settings">
      <h2>Settings</h2>
      <label class="choice">
        <input
          type="checkbox"
          checked={settings.launch_at_login}
          onChange={(event) =>
            change({ ...settings, launch_at_login: event.currentTarget.checked })
          }
        />
        Open QuotaDial when I log in
      </label>
      <fieldset>
        <legend>Notify me when</legend>
        <label class="choice">
          <input
            type="checkbox"
            checked={settings.notify_sign_in}
            onChange={(event) =>
              change({ ...settings, notify_sign_in: event.currentTarget.checked })
            }
          />
          an account needs signing in again
        </label>
        <label class="choice">
          <input
            type="checkbox"
            checked={settings.notify_high}
            onChange={(event) =>
              change({ ...settings, notify_high: event.currentTarget.checked })
            }
          />
          a limit passes 90%
        </label>
      </fieldset>
      <fieldset>
        <legend>Show</legend>
        {settings.providers.map((provider) => (
          <label class="choice" key={provider.id}>
            <input
              type="checkbox"
              checked={provider.shown}
              onChange={(event) =>
                change({
                  ...settings,
                  providers: settings.providers.map((choice) =>
                    choice.id === provider.id
                      ? { ...choice, shown: event.currentTarget.checked }
                      : choice,
                  ),
                })
              }
            />
            {provider.name}
          </label>
        ))}
      </fieldset>
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
