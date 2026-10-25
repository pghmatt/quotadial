// Checks the built window in a real browser engine, as a keyboard and a screen reader use it (#52):
//
// - every control is reached by Tab in the order it is shown, draws its focus ring, and does what
//   it says when pressed with Enter (Space for a checkbox, typing for a text field);
// - every control has a name, a role and its state for a screen reader, and every chart and meter
//   has words;
// - a panel takes the focus when it opens, and its Done gives it back to the button that opened it;
// - no text is cut off, at 100% and at 200% zoom, with a very long account name;
// - nothing moves when the system asks for less motion;
// - nothing goes wrong on the page: no error, and nothing the app's content security policy refuses;
// - on QuotaDial Pro, each account in History has a Share image button named by its label, which
//   works from the keyboard and sends the app a whole 2400 by 1350 PNG for that account, drawn only
//   with the strings `shareStrings` lists (the label, never an address), in the stylesheet's text
//   colours and the window's font, with tabular figures; on QuotaDial Free there is no button and
//   the feature's line shows once (D-115).
//
// Each check runs over the accounts, Settings, History and Help, in the light and dark themes, for a
// QuotaDial Pro and a QuotaDial Free copy, with the made-up accounts in window-fixtures.mjs answering
// in place of the app. It drives Chrome (Chromium is WebView2's engine on Windows) through the
// DevTools protocol, with nothing to install: Chrome is found where it usually is, or named by
// CHROME. `npm run build` first; then `npm run check:window`; `-- --screenshots <folder>` saves a
// picture of each view, `-- --share-images <folder>` saves the picture to share of each account in
// each copy and theme, `-- --only "pro, dark"` checks only the views named so, and
// `-- --fixtures <module>` answers from another set of copies (the website's pictures use
// `site-fixtures.mjs`). Fails, naming the view and the control, when a check does not pass.
import { spawn } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { createServer } from "node:http";
import { tmpdir } from "node:os";
import { delimiter, extname, join, normalize, resolve } from "node:path";
import { fileURLToPath } from "node:url";

// The picture's own list of what it draws, read straight from the window's source (Node strips its
// types), so the check holds the drawing to it.
import { ACCOUNT_HEADER, HEIGHT as PICTURE_HEIGHT, WIDTH as PICTURE_WIDTH, shareStrings } from "../src/share.ts";

/** The picture to share, in pixels: fixed, as the app checks it, whatever the window's source says. */
const SHARED = { width: 2400, height: 1350 };

const DIST = fileURLToPath(new URL("../dist/", import.meta.url));
const CONFIG = JSON.parse(
  readFileSync(new URL("../src-tauri/tauri.conf.json", import.meta.url), "utf8"),
);
/** The popover's width (POPOVER_WIDTH in src-tauri/src/lib.rs) and its tallest height. */
const WIDTH = CONFIG.app.windows[0].width;
const HEIGHT = 640;
/** The copies of the app to check: each named export of the fixtures module that has a meter. */
const FIXTURES = process.argv.includes("--fixtures")
  ? new URL(process.argv[process.argv.indexOf("--fixtures") + 1], `file://${process.cwd()}/`)
  : new URL("./window-fixtures.mjs", import.meta.url);
const COPIES = Object.fromEntries(
  Object.entries(await import(FIXTURES.href)).filter(([, copy]) => copy && typeof copy === "object" && "meter" in copy),
);
const THEMES = ["light", "dark"];
const PANELS = ["accounts", "history", "settings", "help"];
/** How each panel is opened from the footer: History with Space, the others with Enter. */
const OPEN_WITH = { history: "Space", settings: "Enter", help: "Enter" };
const WAIT = 5000;

const KEYS = {
  Tab: { key: "Tab", code: "Tab", windowsVirtualKeyCode: 9 },
  Enter: { key: "Enter", code: "Enter", windowsVirtualKeyCode: 13, text: "\r" },
  Space: { key: " ", code: "Space", windowsVirtualKeyCode: 32, text: " " },
};
const SHIFT = 8;

const failures = [];
let checks = 0;

/** Records a check: `problem` is falsy when it passed, or says what is wrong. */
function check(view, problem) {
  checks += 1;
  if (problem) {
    failures.push(`${view}: ${problem}`);
  }
}

/** Chrome: CHROME, or where each system usually has Chrome, Chromium or Edge. */
function findChrome() {
  if (process.env.CHROME) {
    return process.env.CHROME;
  }
  const candidates = [];
  if (process.platform === "darwin") {
    candidates.push(
      "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
      "/Applications/Chromium.app/Contents/MacOS/Chromium",
      "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
    );
  } else if (process.platform === "win32") {
    for (const root of [process.env.PROGRAMFILES, process.env["PROGRAMFILES(X86)"], process.env.LOCALAPPDATA]) {
      if (root) {
        candidates.push(
          join(root, "Google", "Chrome", "Application", "chrome.exe"),
          join(root, "Microsoft", "Edge", "Application", "msedge.exe"),
        );
      }
    }
  } else {
    for (const directory of (process.env.PATH ?? "").split(delimiter)) {
      for (const name of ["google-chrome", "google-chrome-stable", "chromium", "chromium-browser"]) {
        candidates.push(join(directory, name));
      }
    }
  }
  return candidates.find((candidate) => existsSync(candidate));
}

/** Serves dist/ on 127.0.0.1 under the app's own content security policy. */
function serve() {
  const types = {
    ".html": "text/html",
    ".js": "text/javascript",
    ".css": "text/css",
    ".svg": "image/svg+xml",
    ".png": "image/png",
    ".woff2": "font/woff2",
  };
  const server = createServer((request, response) => {
    const path = new URL(request.url ?? "/", "http://127.0.0.1").pathname;
    // A browser asks for an icon the app's own window never does.
    if (path === "/favicon.ico") {
      response.writeHead(204).end();
      return;
    }
    const file = normalize(join(DIST, path === "/" ? "index.html" : decodeURIComponent(path)));
    if (!file.startsWith(DIST) || !existsSync(file) || !statSync(file).isFile()) {
      response.writeHead(404).end();
      return;
    }
    response
      .writeHead(200, {
        "content-type": types[extname(file)] ?? "application/octet-stream",
        "content-security-policy": CONFIG.app.security.csp,
      })
      .end(readFileSync(file));
  });
  return new Promise((done) => server.listen(0, "127.0.0.1", () => done(server)));
}

/** Starts Chrome headless with a profile of its own, and answers its DevTools address. */
function launch(chrome, profile) {
  const args = [
    "--headless=new",
    "--remote-debugging-port=0",
    `--user-data-dir=${profile}`,
    "--no-first-run",
    "--no-default-browser-check",
    "--disable-extensions",
    "--disable-background-networking",
    "--disable-component-update",
    "--disable-sync",
    "--force-color-profile=srgb",
    "about:blank",
  ];
  // Linux runners refuse Chrome's sandbox; the page is only this build, served on this machine.
  if (process.platform === "linux") {
    args.push("--no-sandbox");
  }
  const browser = spawn(chrome, args, { stdio: ["ignore", "ignore", "pipe"] });
  const address = new Promise((done, fail) => {
    let said = "";
    const timer = setTimeout(() => fail(new Error(`Chrome did not start: ${said}`)), 20_000);
    browser.stderr.on("data", (chunk) => {
      said += chunk;
      const found = said.match(/DevTools listening on (ws:\/\/\S+)/);
      if (found) {
        clearTimeout(timer);
        done(found[1]);
      }
    });
    browser.on("exit", (code) => fail(new Error(`Chrome exited (${code}): ${said}`)));
  });
  return { browser, address };
}

/** The DevTools protocol over one WebSocket, a session per page. */
class DevTools {
  constructor(socket) {
    this.socket = socket;
    this.next = 1;
    this.pending = new Map();
    this.waiters = new Set();
    socket.addEventListener("message", (message) => this.receive(JSON.parse(String(message.data))));
  }

  static async connect(address) {
    const socket = new WebSocket(address);
    await new Promise((done, fail) => {
      socket.addEventListener("open", done, { once: true });
      socket.addEventListener("error", () => fail(new Error(`cannot reach ${address}`)), {
        once: true,
      });
    });
    return new DevTools(socket);
  }

  receive(message) {
    if (message.id !== undefined) {
      const pending = this.pending.get(message.id);
      this.pending.delete(message.id);
      if (message.error) {
        pending?.fail(new Error(`${pending.method}: ${message.error.message}`));
      } else {
        pending?.done(message.result);
      }
      return;
    }
    for (const waiter of this.waiters) {
      waiter(message);
    }
  }

  send(method, params = {}, sessionId = undefined) {
    const id = this.next++;
    this.socket.send(JSON.stringify({ id, method, params, sessionId }));
    return new Promise((done, fail) => this.pending.set(id, { done, fail, method }));
  }

  /** Calls `each` with every event until it is stopped. */
  on(each) {
    this.waiters.add(each);
    return () => this.waiters.delete(each);
  }
}

/**
 * What answers the window's commands in place of the app: `copy`'s answers, and a record of each
 * command the window sends, in `window.__check`. A command the app does not have fails the check.
 */
function stand_in(copy) {
  return `(() => {
    const copy = ${JSON.stringify(copy)};
    const calls = [];
    // Each picture sent to share, and everything a canvas writes and fills, as it is drawn.
    const pictures = [];
    const drawn = [];
    const fillText = CanvasRenderingContext2D.prototype.fillText;
    CanvasRenderingContext2D.prototype.fillText = function (text, ...rest) {
      drawn.push({ text: String(text), fill: String(this.fillStyle), font: this.font });
      return fillText.call(this, text, ...rest);
    };
    const fillRect = CanvasRenderingContext2D.prototype.fillRect;
    CanvasRenderingContext2D.prototype.fillRect = function (...box) {
      drawn.push({ rect: box, fill: String(this.fillStyle) });
      return fillRect.apply(this, box);
    };
    const callbacks = new Map();
    const listeners = new Map();
    let next = 1;
    let settings = structuredClone(copy.settings);
    const keep = (changed) => (settings = { ...settings, ...changed });
    const answers = {
      meter: () => copy.meter,
      updates: () => copy.update,
      help: () => copy.help,
      history: ({ days }) => copy.history[days],
      export_history: () => "Saved 1,204 readings to ~/Downloads/quotadial-history.csv.",
      // As the app answers: it names the file from what it knows of the account the header names.
      save_share_image: (body, options) => {
        const key = decodeURIComponent(options?.headers?.["${ACCOUNT_HEADER}"] ?? "");
        const account = Object.values(copy.history).flatMap((view) => view.accounts).find((one) => one.key === key);
        if (!account) {
          throw "QuotaDial no longer knows the account this picture is of, so nothing was saved; open History again and press Share image.";
        }
        pictures.push({ key, bytes: new Uint8Array(body) });
        return "Saved to ~/Downloads/QuotaDial " + account.label + " 2026-10-25.png.";
      },
      settings: () => settings,
      set_settings: ({ view }) => (settings = view),
      add_alert: ({ rule }) =>
        keep({ alerts: [...settings.alerts, { id: 100 + settings.alerts.length, written: rule, read_as: rule, enabled: true, waits: false }] }),
      set_alert: ({ id, enabled }) =>
        keep({ alerts: settings.alerts.map((alert) => (alert.id === id ? { ...alert, enabled } : alert)) }),
      remove_alert: ({ id }) => keep({ alerts: settings.alerts.filter((alert) => alert.id !== id) }),
      add_licence: () => keep({ licence: { text: "QuotaDial Pro, licence 1234.", kept: true, adds: [] } }),
      remove_licence: () => keep({ licence: copy.settings.licence.kept ? { ...copy.settings.licence, kept: false } : copy.settings.licence }),
      fit: () => null,
      hide: () => null,
      quit: () => null,
      open_link: () => null,
      send_feedback: () => null,
      check_for_updates: () => null,
      install_update: () => null,
      "plugin:event|listen": ({ event, handler }) => {
        listeners.set(event, [...(listeners.get(event) ?? []), handler]);
        return next++;
      },
      "plugin:event|unlisten": () => null,
    };
    window.__TAURI_INTERNALS__ = {
      invoke: async (cmd, args = {}, options = {}) => {
        // A raw body, as the picture is sent, is recorded by its size.
        const raw = args instanceof ArrayBuffer || ArrayBuffer.isView(args);
        calls.push({ cmd, args: raw ? { bytes: args.byteLength } : JSON.parse(JSON.stringify(args)) });
        if (!answers[cmd]) {
          throw new Error("The app has no command " + cmd);
        }
        return structuredClone(answers[cmd](args, options));
      },
      transformCallback: (callback, once) => {
        const id = next++;
        callbacks.set(id, (payload) => {
          if (once) callbacks.delete(id);
          callback(payload);
        });
        return id;
      },
      unregisterCallback: (id) => callbacks.delete(id),
      convertFileSrc: (path) => path,
    };
    window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener: () => {} };
    window.__check = {
      calls,
      pictures,
      drawn,
      emit: (event, payload) => {
        for (const id of listeners.get(event) ?? []) callbacks.get(id)?.({ event, id: 0, payload });
      },
      /** The picture sent at \`index\`, as base64, to read out of the page. */
      picture: (index) => {
        const { key, bytes } = pictures[index];
        let binary = "";
        for (let at = 0; at < bytes.length; at += 0x8000) {
          binary += String.fromCharCode(...bytes.subarray(at, at + 0x8000));
        }
        return { key, base64: btoa(binary) };
      },
      /** The stylesheet's colour \`property\` as a canvas writes it back. */
      colour: (property) => {
        const context = document.createElement("canvas").getContext("2d");
        context.fillStyle = getComputedStyle(document.documentElement).getPropertyValue(property).trim();
        return String(context.fillStyle);
      },
    };
  })();`;
}

/** In the page: the controls Tab reaches, in the order they are shown. */
const FOCUSABLE = `(() => {
  const selector = 'a[href], button, input, select, textarea, summary, [tabindex]';
  window.__focusables = [...document.querySelectorAll(selector)].filter((element) =>
    element.tabIndex >= 0 &&
    !element.disabled &&
    !element.closest('[inert], details:not([open]) > :not(summary)') &&
    element.getClientRects().length > 0 &&
    getComputedStyle(element).visibility !== 'hidden');
  return window.__focusables.map((element) =>
    (element.getAttribute('aria-label') || element.textContent || element.placeholder || element.tagName).trim().replace(/\\s+/g, ' ').slice(0, 60));
})()`;

/** In the page: what the focus is on, as its place among the controls, and how its ring is drawn. */
const FOCUSED = `(() => {
  const element = document.activeElement;
  const style = getComputedStyle(element);
  return {
    index: (window.__focusables ?? []).indexOf(element),
    what: element === document.body ? 'the page' : element.tagName.toLowerCase() + ' "' + (element.getAttribute('aria-label') || element.textContent || '').trim().slice(0, 40) + '"',
    panel: element.matches('section[tabindex="-1"]') ? element.getAttribute('aria-label') : null,
    opener: element.dataset.panel ?? null,
    ring: style.outlineStyle !== 'none' && parseFloat(style.outlineWidth) >= 2,
  };
})()`;

/** In the page: what pressing a key could change, to tell whether it did anything. */
const STATE = `JSON.stringify({
  calls: window.__check.calls.filter((call) => call.cmd !== 'fit' && !call.cmd.startsWith('plugin:')).length,
  drawn: window.__check.drawn.length,
  open: [...document.querySelectorAll('details')].map((details) => details.open),
  checked: [...document.querySelectorAll('input[type=checkbox]')].map((box) => box.checked),
  pressed: [...document.querySelectorAll('[aria-pressed]')].map((button) => button.getAttribute('aria-pressed')),
  values: [...document.querySelectorAll('input[type=text]')].map((field) => field.value),
  panels: [...document.querySelectorAll('main > section')].map((section) => section.className),
})`;

/**
 * In the page: text that is cut off, by the window's edge or by a box that hides what overflows.
 * What a closed disclosure holds is checked open, as it shows when someone opens it.
 */
const CUT_OFF = `(() => {
  const closed = [...document.querySelectorAll('details:not([open])')];
  for (const details of closed) details.open = true;
  const width = document.documentElement.clientWidth;
  const found = [];
  const shown = (element) => element.getClientRects().length > 0 && !element.closest('.visually-hidden');
  const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
  for (let node = walker.nextNode(); node; node = walker.nextNode()) {
    const text = node.textContent.trim();
    const element = node.parentElement;
    if (!text || !element || !shown(element)) continue;
    const range = document.createRange();
    range.selectNodeContents(node);
    for (const rect of range.getClientRects()) {
      if (rect.width === 0) continue;
      if (rect.left < -0.5 || rect.right > width + 0.5) {
        found.push('"' + text.slice(0, 50) + '" runs past the edge of the window');
        break;
      }
      let clipped = null;
      for (let box = element; box && box !== document.body; box = box.parentElement) {
        const style = getComputedStyle(box);
        if (style.overflowX === 'visible' && style.overflowY === 'visible') continue;
        const edge = box.getBoundingClientRect();
        if (rect.left < edge.left - 0.5 || rect.right > edge.right + 0.5 || rect.top < edge.top - 0.5 || rect.bottom > edge.bottom + 0.5) {
          clipped = box;
          break;
        }
      }
      if (clipped) {
        found.push('"' + text.slice(0, 50) + '" is cut off by its ' + clipped.tagName.toLowerCase() + (clipped.className ? '.' + clipped.className : ''));
        break;
      }
    }
  }
  // A field's hint is drawn inside the field, which cuts off what does not fit.
  const measure = document.createElement('canvas').getContext('2d');
  for (const field of document.querySelectorAll('input[placeholder]')) {
    if (!shown(field) || field.value) continue;
    const style = getComputedStyle(field);
    measure.font = style.font;
    const room = field.clientWidth - parseFloat(style.paddingLeft) - parseFloat(style.paddingRight);
    if (measure.measureText(field.placeholder).width > room + 0.5) {
      found.push('the hint "' + field.placeholder + '" is cut off by its field');
    }
  }
  for (const details of closed) details.open = false;
  return found;
})()`;

/** In the page: anything that moves, which nothing may do when less motion is asked for. */
const MOVING = `(() => {
  const found = document.getAnimations().map((animation) => 'an animation on ' + (animation.effect?.target?.tagName ?? 'the page'));
  for (const element of document.querySelectorAll('*')) {
    const style = getComputedStyle(element);
    const transition = style.transitionDuration.split(',').some((duration) => parseFloat(duration) > 0);
    if (transition || style.animationName !== 'none') {
      found.push(element.tagName.toLowerCase() + (element.className ? '.' + element.className : '') + ' still moves');
    }
  }
  return [...new Set(found)].slice(0, 10);
})()`;

/** A scroll bar's width where it takes room, in CSS pixels, as Chrome draws one. */
const CLASSIC_BAR = 15;

/** In the page: how much room a scroll bar takes, which is none where it lies over the page. */
const SCROLL_BAR = `(() => {
  const probe = document.createElement('div');
  probe.style.cssText = 'position:absolute;top:-500px;width:100px;height:100px;overflow:scroll';
  document.documentElement.append(probe);
  const room = probe.offsetWidth - probe.clientWidth;
  probe.remove();
  return room;
})()`;

/** In the page: whether charts, meters and the heatmap carry their facts in words. */
const WORDS = `(() => {
  const found = [];
  for (const meter of document.querySelectorAll('meter')) {
    if (!meter.getAttribute('aria-valuetext')) found.push('a meter has no words for its value');
  }
  for (const heatmap of document.querySelectorAll('.heatmap')) {
    const rows = heatmap.querySelectorAll('.heat-row:not(.heat-hours)');
    const said = heatmap.querySelectorAll('ul.visually-hidden li');
    if (said.length !== rows.length) found.push('a heatmap says ' + said.length + ' days in words for ' + rows.length + ' rows');
    if ([...heatmap.querySelectorAll('.heat-row')].some((row) => row.getAttribute('aria-hidden') !== 'true')) {
      found.push('a heatmap row is not hidden from screen readers, which hear its words instead');
    }
  }
  return found;
})()`;

/** A page of the window, opened on one copy of the app in one theme. */
class View {
  constructor(tools, sessionId, label, url) {
    this.tools = tools;
    this.sessionId = sessionId;
    this.label = label;
    this.url = url;
    this.errors = [];
    this.stop = tools.on((message) => {
      if (message.sessionId !== sessionId) {
        return;
      }
      if (message.method === "Runtime.exceptionThrown") {
        const details = message.params.exceptionDetails;
        this.errors.push((details.exception?.description ?? details.text).split("\n")[0]);
      } else if (message.method === "Runtime.consoleAPICalled" && message.params.type === "error") {
        this.errors.push(message.params.args.map((arg) => arg.value ?? arg.description).join(" "));
      } else if (message.method === "Log.entryAdded" && message.params.entry.level === "error") {
        this.errors.push(message.params.entry.text);
      }
    });
  }

  static async open(tools, label, url, copy, theme) {
    const { targetId } = await tools.send("Target.createTarget", { url: "about:blank" });
    const { sessionId } = await tools.send("Target.attachToTarget", { targetId, flatten: true });
    const view = new View(tools, sessionId, label, url);
    view.targetId = targetId;
    for (const domain of ["Page", "Runtime", "Log", "Accessibility"]) {
      await view.send(`${domain}.enable`);
    }
    await view.send("Emulation.setFocusEmulationEnabled", { enabled: true });
    // Not a Mac's user agent, so the window paints its own background, as on Windows and Linux.
    await view.send("Emulation.setUserAgentOverride", { userAgent: "QuotaDial window check" });
    await view.send("Emulation.setEmulatedMedia", {
      features: [
        { name: "prefers-color-scheme", value: theme },
        { name: "prefers-reduced-motion", value: "reduce" },
      ],
    });
    await view.zoom(1);
    await view.send("Page.addScriptToEvaluateOnNewDocument", { source: stand_in(copy) });
    return view;
  }

  send(method, params) {
    return this.tools.send(method, params, this.sessionId);
  }

  async evaluate(expression) {
    const { result, exceptionDetails } = await this.send("Runtime.evaluate", {
      expression,
      returnByValue: true,
      awaitPromise: true,
    });
    if (exceptionDetails) {
      throw new Error(`${this.label}: ${exceptionDetails.exception?.description ?? exceptionDetails.text}`);
    }
    return result.value;
  }

  /** Waits until `expression` is true in the page. */
  async until(expression, what) {
    const end = Date.now() + WAIT;
    while (Date.now() < end) {
      if (await this.evaluate(expression)) {
        return;
      }
      await new Promise((done) => setTimeout(done, 25));
    }
    throw new Error(`${this.label}: ${what} did not happen within ${WAIT / 1000} s`);
  }

  /** Lets the page draw what the last key changed. */
  settle() {
    return this.evaluate(
      "new Promise((done) => requestAnimationFrame(() => requestAnimationFrame(() => setTimeout(done, 20))))",
    );
  }

  /**
   * Shows the page as the window does at `zoom`: zooming in narrows it in CSS pixels. Where scroll
   * bars take no room, as on a Mac by default, the page is narrowed by a scroll bar as well, so
   * every system checks the room Windows and Linux leave.
   */
  async zoom(zoom) {
    this.bar ??= (await this.evaluate(SCROLL_BAR)) > 0 ? 0 : CLASSIC_BAR;
    await this.send("Emulation.setDeviceMetricsOverride", {
      width: Math.round(WIDTH / zoom) - this.bar,
      height: Math.round(HEIGHT / zoom),
      deviceScaleFactor: zoom,
      mobile: false,
    });
  }

  /** Opens the window afresh, then `panel` from the footer, by keyboard. */
  async load(panel) {
    const loaded = new Promise((done) => {
      const stop = this.tools.on((message) => {
        if (message.sessionId === this.sessionId && message.method === "Page.loadEventFired") {
          stop();
          done();
        }
      });
    });
    await this.send("Page.navigate", { url: this.url });
    await loaded;
    await this.until("!!document.querySelector('footer')", "the window showing the accounts");
    if (panel === "accounts") {
      return;
    }
    for (let presses = 0; presses < 40; presses += 1) {
      await this.press("Tab");
      if ((await this.evaluate(FOCUSED)).opener === panel) {
        await this.press(OPEN_WITH[panel]);
        await this.until(
          `!!document.querySelector('main > section.${panel} h2')`,
          `the ${panel} panel opening`,
        );
        return;
      }
    }
    throw new Error(`${this.label}: Tab never reached the footer's ${panel} button`);
  }

  async press(name, modifiers = 0) {
    const key = KEYS[name];
    await this.send("Input.dispatchKeyEvent", { type: key.text ? "keyDown" : "rawKeyDown", modifiers, ...key });
    await this.send("Input.dispatchKeyEvent", { type: "keyUp", modifiers, ...key, text: undefined });
    await this.settle();
  }

  /** What a screen reader is told about the control at `index`. */
  async told(index) {
    const { result } = await this.send("Runtime.evaluate", { expression: `window.__focusables[${index}]` });
    const { nodes } = await this.send("Accessibility.getPartialAXTree", {
      objectId: result.objectId,
      fetchRelatives: false,
    });
    const node = nodes[0];
    const properties = Object.fromEntries(
      (node.properties ?? []).map((property) => [property.name, property.value.value]),
    );
    // Where the name came from: the first source that gave one and was not overridden.
    const from = node.name?.sources?.find((source) => source.value && !source.superseded);
    return {
      role: node.role?.value ?? "",
      name: String(node.name?.value ?? "").trim(),
      from: from?.type === "placeholder" || from?.attribute === "placeholder" ? "hint" : from?.type,
      properties,
    };
  }

  async close() {
    this.stop();
    await this.tools.send("Target.closeTarget", { targetId: this.targetId });
  }
}

/** Roles a control may have, and what a screen reader must be told of its state. */
function toldProblem(tag, type, told) {
  if (!told.name) {
    return "has no name";
  }
  // A hint disappears as soon as something is typed, so it cannot be the field's only name.
  if (told.from === "hint") {
    return "is named only by its hint";
  }
  const expected = {
    button: ["button", "togglebutton"],
    summary: ["disclosuretriangle"],
    "input:checkbox": ["checkbox"],
    "input:text": ["textbox"],
  }[type ? `${tag}:${type}` : tag];
  if (expected && !expected.includes(told.role.toLowerCase())) {
    return `is told as a ${told.role}, not a ${expected[0]}`;
  }
  if (type === "checkbox" && told.properties.checked === undefined) {
    return "does not say whether it is ticked";
  }
  if (tag === "summary" && told.properties.expanded === undefined) {
    return "does not say whether it is open";
  }
  return null;
}

/** The checks over one panel of one copy in one theme. */
async function checkView(view, panel, operate) {
  const label = view.label;
  await view.load(panel);
  const focused = await view.evaluate(FOCUSED);
  if (panel !== "accounts") {
    check(label, focused.panel === null && `opening ${panel} left the focus on ${focused.what}, not in the panel`);
  }

  // Tab reaches each control in the order it is shown, with its ring drawn.
  const controls = await view.evaluate(FOCUSABLE);
  const kinds = await view.evaluate(
    "window.__focusables.map((element) => [element.tagName.toLowerCase(), element.type ?? null])",
  );
  for (let index = 0; index < controls.length; index += 1) {
    await view.press("Tab");
    const now = await view.evaluate(FOCUSED);
    check(label, now.index !== index && `Tab ${index + 1} reached ${now.what}, not "${controls[index]}"`);
    if (now.index !== index) {
      break;
    }
    check(label, !now.ring && `"${controls[index]}" does not draw the window's focus ring`);
    const [tag, type] = kinds[index];
    const told = await view.told(index);
    const problem = toldProblem(tag, tag === "input" ? type : null, told);
    check(label, problem && `"${controls[index]}" ${problem} for a screen reader`);
  }
  if (controls.length > 1) {
    await view.press("Tab", SHIFT);
    const back = await view.evaluate(FOCUSED);
    check(label, back.index !== controls.length - 2 && `Shift-Tab went to ${back.what}, not back one`);
  }

  // Every chart, meter and image says its facts in words.
  const { nodes } = await view.send("Accessibility.getFullAXTree");
  for (const node of nodes) {
    const role = String(node.role?.value ?? "").toLowerCase();
    if (!node.ignored && ["img", "image", "meter", "progressbar"].includes(role)) {
      const what = role === "img" || role === "image" ? "a chart or picture" : `a ${role}`;
      check(label, !String(node.name?.value ?? "").trim() && `${what} has no name for a screen reader`);
    }
  }
  for (const problem of await view.evaluate(WORDS)) {
    check(label, problem);
  }

  // Nothing moves, and no text is cut off at 100% or 200%.
  for (const problem of await view.evaluate(MOVING)) {
    check(label, `${problem} with less motion asked for`);
  }
  for (const zoom of [1, 2]) {
    await view.zoom(zoom);
    await view.settle();
    const cut = await view.evaluate(CUT_OFF);
    check(label, cut.length > 0 && `at ${zoom * 100}%, ${cut.slice(0, 6).join("; ")}`);
  }
  await view.zoom(1);

  // Each control does what it says from the keyboard, each on a window opened afresh.
  if (operate) {
    for (let index = 0; index < controls.length; index += 1) {
      await view.load(panel);
      await view.evaluate(FOCUSABLE);
      for (let presses = 0; presses <= index; presses += 1) {
        await view.press("Tab");
      }
      if ((await view.evaluate(FOCUSED)).index !== index) {
        continue; // Tab order is already reported above.
      }
      // The period already chosen stays chosen when it is pressed again, which changes nothing.
      const chosen = await view.evaluate(
        `window.__focusables[${index}].getAttribute('aria-pressed') === 'true'`,
      );
      const before = await view.evaluate(STATE);
      const [tag, type] = kinds[index];
      let how = "Enter";
      if (tag === "input" && type === "checkbox") {
        how = "Space";
        await view.press("Space");
      } else if (tag === "input") {
        how = "typing and Enter";
        await view.send("Input.insertText", { text: "warn me at 95%" });
        await view.press("Enter");
      } else {
        await view.press("Enter");
      }
      const after = await view.evaluate(STATE);
      if (chosen) {
        check(label, before !== after && `${how} on the chosen "${controls[index]}" changed it`);
      } else {
        check(label, before === after && `${how} on "${controls[index]}" did nothing`);
      }
      if (panel !== "accounts" && /^Done$/.test(controls[index])) {
        const back = await view.evaluate(FOCUSED);
        check(label, back.opener !== panel && `Done gave the focus to ${back.what}, not the ${panel} button`);
      }
    }
  }
  check(label, view.errors.length > 0 && `the page reported ${[...new Set(view.errors)].slice(0, 3).join("; ")}`);
  view.errors = [];
  return controls.length;
}

/** The CRC-32 a PNG chunk ends with. */
function crc32(bytes) {
  let crc = 0xffffffff;
  for (const byte of bytes) {
    crc ^= byte;
    for (let bit = 0; bit < 8; bit += 1) {
      crc = crc & 1 ? (crc >>> 1) ^ 0xedb88320 : crc >>> 1;
    }
  }
  return (crc ^ 0xffffffff) >>> 0;
}

/** A PNG's width and height, read from its header as the app reads them, or why it is not a whole
 * PNG. */
function pngSize(bytes) {
  const signature = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
  const end = [0, 0, 0, 0, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82];
  if (bytes.length < 45 || signature.some((byte, at) => bytes[at] !== byte)) {
    return "it does not begin as a PNG";
  }
  if (end.some((byte, at) => bytes[bytes.length - end.length + at] !== byte)) {
    return "it does not end as a PNG";
  }
  if (bytes.readUInt32BE(8) !== 13 || bytes.toString("latin1", 12, 16) !== "IHDR") {
    return "its first chunk is not its header";
  }
  if (bytes.readUInt32BE(29) !== crc32(bytes.subarray(12, 29))) {
    return "its header's checksum is wrong";
  }
  return { width: bytes.readUInt32BE(16), height: bytes.readUInt32BE(20) };
}

/** In the page: each account's Share image button in History, as its place among the controls Tab
 * reaches, or null for an account without one. */
const SHARE_BUTTONS = `[...document.querySelectorAll('section.history .account-history')].map((section) => {
  const button = section.querySelector('button.share');
  return button ? window.__focusables.indexOf(button) : null;
})`;

/** In the page: the lines History shows, outside the accounts. */
const HISTORY_LINES = `[...document.querySelectorAll('section.history > p')].map((line) => line.textContent.trim())`;

/**
 * The History panel's Share image (D-115). On QuotaDial Pro: a button for each account, after the
 * periods and before Export CSV, in the accounts' order, named by the account's label; pressed
 * from the keyboard, it sends the app a whole PNG of 2400 by 1350 for that account, says where it
 * went, and draws only what `shareStrings` lists, with the label and never an @, in the
 * stylesheet's text colours on its background, the window's font and tabular figures: each
 * account over 7 days, and the first over 30. On QuotaDial Free: no button, and the feature's line
 * once. Saves each picture in `folder` when one is named.
 */
async function checkSharing(view, name, copy, theme, folder) {
  const label = `${view.label}, Share image`;
  const history = copy.history[7];
  await view.load("history");
  const controls = await view.evaluate(FOCUSABLE);
  const places = await view.evaluate(SHARE_BUTTONS);
  const lines = await view.evaluate(HISTORY_LINES);
  if (history.share_left_out !== null) {
    check(label, places.some((place) => place !== null) && "a QuotaDial Free copy has a Share image button");
    const shown = lines.filter((line) => line === history.share_left_out).length;
    check(label, shown !== 1 && `a QuotaDial Free copy says "${history.share_left_out}" ${shown} times, not once`);
    return;
  }
  check(label, lines.some((line) => line.includes("part of QuotaDial Pro")) && "a QuotaDial Pro copy says Share image is part of Pro");
  check(label, places.length !== history.accounts.length && `History shows ${places.length} accounts, not ${history.accounts.length}`);
  const month = controls.indexOf("30 days");
  const exporting = controls.indexOf("Export CSV");
  let before = month;
  for (const [number, account] of history.accounts.entries()) {
    const place = places[number];
    if (place === null || place === undefined || place < 0) {
      check(label, `${account.label} has no Share image button that Tab reaches`);
      continue;
    }
    check(label, !(place > before && place < exporting) && `${account.label}'s Share image is Tab stop ${place + 1}, out of order`);
    before = place;
    const told = await view.told(place);
    check(label, told.name !== `Share image of ${account.label}` && `${account.label}'s Share image is named "${told.name}" for a screen reader`);
  }

  const look = {
    allowed: new Set(await view.evaluate(`['--text', '--muted', '--warning', '--high'].map(window.__check.colour)`)),
    background: await view.evaluate(`window.__check.colour('--background')`),
    family: await view.evaluate("getComputedStyle(document.documentElement).fontFamily"),
  };
  // Each account over 7 days, and the first over 30 where the copy has a month of its own.
  const month30 = copy.history[30];
  const shares = [
    ...history.accounts.map((account, number) => ({ days: 7, number, account })),
    ...(month30?.days === 30 && month30.accounts.length > 0 ? [{ days: 30, number: 0, account: month30.accounts[0] }] : []),
  ];
  for (const { days, number, account } of shares) {
    const place = places[number];
    if (place === null || place === undefined || place < 0) {
      continue;
    }
    // From a window opened afresh, by keyboard: the period, then Tab to the button and Enter.
    await view.load("history");
    await view.evaluate(FOCUSABLE);
    if (days === 30) {
      await tabTo(view, month);
      await view.press("Enter");
      await view.until(
        `document.querySelector('section.history > p.detail')?.textContent === ${JSON.stringify(copy.history[30].span)}`,
        "History showing 30 days",
      );
    }
    const sent = await view.evaluate("window.__check.pictures.length");
    await view.evaluate("window.__check.drawn.length = 0");
    if (!(await tabTo(view, place))) {
      check(label, `Tab did not reach ${account.label}'s Share image`);
      continue;
    }
    await view.press("Enter");
    await view.until(`window.__check.pictures.length > ${sent}`, `${account.label}'s picture reaching the app`);
    await view.until(
      `document.querySelector('section.history [role=status]').textContent.startsWith('Saved to ')`,
      `History saying where ${account.label}'s picture went`,
    );
    const picture = await view.evaluate(`window.__check.picture(${sent})`);
    const what = `${account.label}'s picture of ${days} days`;
    check(label, picture.key !== account.key && `${what} was sent for the account "${picture.key}"`);
    const bytes = Buffer.from(picture.base64, "base64");
    const size = pngSize(bytes);
    if (typeof size === "string") {
      check(label, `${what} is not a whole PNG: ${size}`);
    } else {
      check(
        label,
        (size.width !== SHARED.width || size.height !== SHARED.height) &&
          `${what} is ${size.width} by ${size.height}, not ${SHARED.width} by ${SHARED.height}`,
      );
    }
    if (folder) {
      const slug = account.label.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "").slice(0, 40);
      writeFileSync(join(folder, `${name}-${theme}-${days}-days-${number + 1}-${slug}.png`), bytes);
    }
    const drawn = await view.evaluate("window.__check.drawn");
    for (const verdict of drawingVerdicts(drawn, shareStrings(copy.history[days], account), account, copy, look)) {
      check(label, verdict && `${what} ${verdict}`);
    }
  }
}

/** Presses Tab until the control at `index` has the focus, and answers whether it got there. */
async function tabTo(view, index) {
  for (let presses = 0; presses < 60; presses += 1) {
    if ((await view.evaluate(FOCUSED)).index === index) {
      return true;
    }
    await view.press("Tab");
  }
  return false;
}

/**
 * A verdict on each thing a picture drew, from the canvas's record of it: false where it passed,
 * or what is wrong (each kind of problem said once). Nothing may be written that `listed` does not
 * hold, or a cut-short start of it; nothing with an @; the name given the account must be in its
 * label; writing only in the stylesheet's text colours and the window's font, figures one by one
 * in equal room; and the ground the window's background.
 */
function drawingVerdicts(drawn, listed, account, copy, look) {
  const said = new Set();
  const verdict = (problem) => {
    if (!problem || said.has(problem)) {
      return false;
    }
    said.add(problem);
    return problem;
  };
  const verdicts = [];
  const alias = copy.meter.accounts.find((one) => one.key === account.key)?.alias;
  verdicts.push(verdict(alias && !listed.some((text) => text.includes(alias)) && `does not name the account by "${alias}"`));
  verdicts.push(verdict(listed.some((text) => text.includes("@")) && "lists a string with an @"));
  const [ground] = drawn;
  verdicts.push(
    verdict(
      !(ground && "rect" in ground && ground.fill === look.background && ground.rect.join() === `0,0,${PICTURE_WIDTH},${PICTURE_HEIGHT}`) &&
        "does not begin with the window's background",
    ),
  );
  const written = drawn.filter((item) => "text" in item);
  verdicts.push(verdict(written.length === 0 && "writes nothing"));
  const title = written.find((item) => item.text !== "QuotaDial" && item.text !== "quotadial.com");
  const opening = title?.text.replace(/…$/, "") ?? "";
  verdicts.push(verdict(!(opening.length > 0 && account.label.startsWith(opening)) && `opens with "${title?.text}", not the account's label`));
  for (const item of written) {
    const whole = item.text.replace(/…$/, "");
    verdicts.push(
      verdict(item.text.includes("@") && `writes "${item.text}", with an @`),
      verdict(!listed.some((text) => text.includes(whole)) && `writes "${item.text}", which shareStrings does not list`),
      verdict(!look.allowed.has(item.fill) && `writes "${item.text}" in ${item.fill}, not one of the stylesheet's text colours`),
      verdict(!item.font.endsWith(look.family) && `writes "${item.text}" in ${item.font}, not the window's font`),
      verdict(/[0-9]/.test(item.text) && item.text.length > 1 && `writes "${item.text}" without tabular figures`),
    );
  }
  return verdicts;
}

async function screenshot(view, panel, folder) {
  await view.load(panel);
  await view.send("Emulation.setDeviceMetricsOverride", {
    width: WIDTH,
    height: HEIGHT,
    deviceScaleFactor: 2,
    mobile: false,
  });
  await view.evaluate("document.activeElement.blur()");
  await view.settle();
  const height = await view.evaluate("Math.ceil(document.querySelector('main').getBoundingClientRect().height)");
  const { data } = await view.send("Page.captureScreenshot", {
    format: "png",
    captureBeyondViewport: true,
    clip: { x: 0, y: 0, width: WIDTH, height: Math.max(120, height), scale: 1 },
  });
  const file = join(folder, `${view.label.replaceAll(", ", "-")}.png`);
  writeFileSync(file, Buffer.from(data, "base64"));
  await view.zoom(1);
}

async function main() {
  const at = process.argv.indexOf("--screenshots");
  const pictures = at > 0 ? resolve(process.argv[at + 1] ?? "") : null;
  const sharing = process.argv.indexOf("--share-images");
  const shares = sharing > 0 ? resolve(process.argv[sharing + 1] ?? "") : null;
  const only = process.argv.includes("--only") ? process.argv[process.argv.indexOf("--only") + 1] : "";
  for (const folder of [pictures, shares]) {
    if (folder) {
      mkdirSync(folder, { recursive: true });
    }
  }
  if (!existsSync(join(DIST, "index.html"))) {
    console.error("There is no built window in dist/; run npm run build first.");
    process.exit(1);
  }
  const chrome = findChrome();
  if (!chrome) {
    console.error("Chrome was not found; install Chrome or Chromium, or set CHROME to its path.");
    process.exit(1);
  }
  const server = await serve();
  const url = `http://127.0.0.1:${server.address().port}/`;
  const profile = mkdtempSync(join(tmpdir(), "quotadial-window-check-"));
  const { browser, address } = launch(chrome, profile);
  let tools;
  try {
    tools = await DevTools.connect(await address);
    let views = 0;
    for (const [name, copy] of Object.entries(COPIES)) {
      for (const theme of THEMES) {
        for (const panel of PANELS) {
          const label = `${name}, ${theme}, ${panel}`;
          if (!label.includes(only ?? "")) {
            continue;
          }
          const view = await View.open(tools, label, url, copy, theme);
          try {
            // Pressing each control is the same in either theme, so once is enough.
            await checkView(view, panel, theme === "light");
            // The picture to share follows the theme, so it is checked in both.
            if (panel === "history") {
              await checkSharing(view, name, copy, theme, shares);
            }
            if (pictures) {
              await screenshot(view, panel, pictures);
            }
          } finally {
            await view.close();
          }
          views += 1;
        }
      }
    }
    if (failures.length > 0) {
      console.error(failures.join("\n"));
      console.error(`${failures.length} of ${checks} window checks failed.`);
      process.exitCode = 1;
    } else {
      console.log(
        `The window passed ${checks} checks over ${views} views: Tab order, focus, names and roles, ` +
          `keys, zoom to 200%, long names, reduced motion and the picture to share.` +
          (pictures ? ` Pictures are in ${pictures}.` : "") +
          (shares ? ` Pictures to share are in ${shares}.` : ""),
      );
    }
  } finally {
    // Chrome writes to its profile until it has exited, so the profile goes after it.
    const exited = new Promise((done) => {
      if (browser.exitCode !== null) {
        done();
      }
      browser.once("exit", done);
      setTimeout(done, 5000);
    });
    await tools?.send("Browser.close").catch(() => {});
    await exited;
    browser.kill();
    server.close();
    rmSync(profile, { recursive: true, force: true, maxRetries: 10, retryDelay: 200 });
  }
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
