import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "preact/hooks";

import { Address } from "./account";
import { usePanelFocus } from "./focus";
import type { Settings } from "./meter";
import { Said } from "./said";

/**
 * The settings panel: opening at login, the daily check for updates, the alert rules, which
 * providers to show, the names given to accounts, and the licence key.
 */
export function SettingsPanel() {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [problem, setProblem] = useState<string | null>(null);
  const [draft, setDraft] = useState("");
  const [key, setKey] = useState("");
  const panel = usePanelFocus();

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
    return <section class="settings" aria-label="Settings" tabIndex={-1} ref={panel} />;
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
    <section class="settings" aria-label="Settings" tabIndex={-1} ref={panel}>
      <h2>Settings</h2>
      <div class="card">
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
        <label class="choice">
          <input
            type="checkbox"
            checked={settings.check_updates}
            aria-describedby="check-updates-detail"
            onChange={(event) =>
              change({ ...settings, check_updates: event.currentTarget.checked })
            }
          />
          Check for updates once a day
        </label>
        <p class="detail choice-detail" id="check-updates-detail">
          Asks GitHub whether there is a newer QuotaDial, sending nothing about you or your usage.
          An update installs only when you choose it, from Help or the popover.
        </p>
      </div>
      <fieldset>
        <legend>Alerts</legend>
        <div class="card">
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
              <Said text={problem} />
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
        </div>
      </fieldset>
      <fieldset>
        <legend>Show</legend>
        <div class="card">
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
        </div>
      </fieldset>
      {settings.accounts.length > 0 && (
        <fieldset>
          <legend>Names</legend>
          <div class="card">
            {settings.accounts.map((account) => (
              <label class="name-choice" key={account.key}>
                {/* Who the account is, as the popover heads its card, but by its provider's name,
                    since the field below is where the name given goes. */}
                <span class="card-head">
                  <span class="title-row">
                    <span class="title">{account.provider}</span>
                  </span>
                  <Address
                    who={{ provider: account.provider, alias: null, name: account.name }}
                  />
                </span>
                <input
                  type="text"
                  value={account.alias}
                  placeholder="such as Work"
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
          </div>
        </fieldset>
      )}
      <fieldset>
        <legend>Licence</legend>
        <div class="card">
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
              placeholder="Paste your key"
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
        </div>
      </fieldset>
      {problem && (
        <p class="note" role="alert">
          {problem}
        </p>
      )}
    </section>
  );
}
