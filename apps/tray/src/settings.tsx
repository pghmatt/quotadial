import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "preact/hooks";

import type { Settings } from "./meter";

/**
 * The settings panel: opening at login, the alert rules, which providers to show, the names given
 * to accounts, and the licence key.
 */
export function SettingsPanel({ onDone }: { onDone: () => void }) {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [problem, setProblem] = useState<string | null>(null);
  const [draft, setDraft] = useState("");
  const [key, setKey] = useState("");

  useEffect(() => {
    void invoke<Settings>("settings").then(setSettings);
  }, []);

  // A name is saved when its field loses focus. Leaving the popover does not take the focus from
  // the field by itself, so it is taken here, and a name typed before clicking away is kept.
  useEffect(() => {
    const leave = () => {
      if (document.activeElement instanceof HTMLElement) {
        document.activeElement.blur();
      }
    };
    window.addEventListener("blur", leave);
    return () => window.removeEventListener("blur", leave);
  }, []);

  if (!settings) {
    return <section class="settings" aria-label="Settings" />;
  }

  // Each change applies at once; the app answers with the settings as they now are.
  const apply = (saving: Promise<Settings>, done?: () => void) => {
    saving
      .then((saved) => {
        setSettings(saved);
        setProblem(null);
        done?.();
      })
      .catch((error: unknown) => {
        setProblem(String(error));
        void invoke<Settings>("settings").then(setSettings);
      });
  };
  const change = (next: Settings) => {
    setSettings(next);
    apply(invoke<Settings>("set_settings", { view: next }));
  };
  const addAlert = () => {
    if (draft.trim()) {
      apply(invoke<Settings>("add_alert", { rule: draft }), () => setDraft(""));
    }
  };
  const addLicence = () => {
    if (key.trim()) {
      apply(invoke<Settings>("add_licence", { key }), () => setKey(""));
    }
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
        <legend>Alerts</legend>
        {settings.alerts.map((alert) => (
          <div class="alert-choice" key={alert.id}>
            <label class="choice">
              <input
                type="checkbox"
                checked={alert.enabled}
                onChange={(event) =>
                  apply(
                    invoke<Settings>("set_alert", {
                      id: alert.id,
                      enabled: event.currentTarget.checked,
                    }),
                  )
                }
              />
              {alert.written}
            </label>
            <button
              type="button"
              class="remove"
              aria-label={`Remove the alert "${alert.written}"`}
              onClick={() => apply(invoke<Settings>("remove_alert", { id: alert.id }))}
            >
              Remove
            </button>
            <p class="read-as">Read as: {alert.read_as}</p>
            {alert.waits && <p class="read-as">Waits for QuotaDial Pro.</p>}
          </div>
        ))}
        {settings.alert_targets.length > 0 && (
          <p class="detail">Also sent to {settings.alert_targets.join(", ")}.</p>
        )}
        {settings.alert_problems.map((problem) => (
          <p class="note" key={problem}>
            {problem}
          </p>
        ))}
        <form
          class="add-alert"
          onSubmit={(event) => {
            event.preventDefault();
            addAlert();
          }}
        >
          <input
            type="text"
            value={draft}
            placeholder="warn me at 80%"
            aria-label="A new alert, in plain words"
            onInput={(event) => setDraft(event.currentTarget.value)}
          />
          <button type="submit" disabled={!draft.trim()}>
            Add
          </button>
        </form>
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
      {settings.accounts.length > 0 && (
        <fieldset>
          <legend>Names</legend>
          {settings.accounts.map((account) => (
            <label class="name-choice" key={account.key}>
              <span>{account.label}</span>
              <input
                type="text"
                value={account.alias}
                placeholder="A name for this account"
                onChange={(event) =>
                  change({
                    ...settings,
                    accounts: settings.accounts.map((choice) =>
                      choice.key === account.key
                        ? { ...choice, alias: event.currentTarget.value }
                        : choice,
                    ),
                  })
                }
              />
            </label>
          ))}
        </fieldset>
      )}
      <fieldset>
        <legend>Licence</legend>
        <p class="licence-text">{settings.licence.text}</p>
        {settings.licence.adds.length > 0 && (
          <div class="licence-adds">
            <p>QuotaDial Pro, at quotadial.com/pro, adds:</p>
            <ul>
              {settings.licence.adds.map((feature) => (
                <li key={feature}>{feature}</li>
              ))}
            </ul>
          </div>
        )}
        <form
          class="add-licence"
          onSubmit={(event) => {
            event.preventDefault();
            addLicence();
          }}
        >
          <input
            type="text"
            value={key}
            placeholder="Paste a licence key"
            aria-label="A licence key for QuotaDial Pro"
            autocomplete="off"
            spellcheck={false}
            onInput={(event) => setKey(event.currentTarget.value)}
          />
          <button type="submit" disabled={!key.trim()}>
            Add
          </button>
        </form>
        {settings.licence.kept && (
          <button
            type="button"
            class="remove-licence"
            onClick={() => apply(invoke<Settings>("remove_licence"))}
          >
            Remove the licence key
          </button>
        )}
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
