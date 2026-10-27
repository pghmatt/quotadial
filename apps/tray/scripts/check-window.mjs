// Checks the built window in a real browser engine, as a keyboard and a screen reader use it (#52):
//
// - every control is reached by Tab in the order it is shown, draws its focus ring, and does what
//   it says when pressed with Enter (Space for a checkbox, typing for a text field);
// - every control has a name, a role and its state for a screen reader, and every chart and meter
//   has words;
// - a panel takes the focus when it opens, and its Done gives it back to the button that opened it;
// - no text is cut off, at 100% and at 200% zoom, with text from outside as long as it gets in every
//   place the window shows some, none of it with a space to break at: an address, a name given to an
//   account, a plan, a window's label, a folder and a command in a problem, an incident, a webhook's
//   host and an alert (the copy of ten accounts has each, D-139);
// - each account is a card of its own, apart from the next and overlapping none, whose heading
//   names it by the name given it or its provider's, with its address, visible inside the card,
//   selectable and read as part of the heading, at 100% and at 200%;
// - every command a message names between backticks, such as Claude's sign-in command, shows as
//   code with no backtick left, and one click selects the whole command, to copy it, at 100% and at
//   200%, where a long one is drawn over several lines;
// - the line under the summary's number shows what the app sent, and names the window and the
//   account that set the number, by the name given it or its address;
// - where and when an account's windows were read is said once, in a quiet caption under its
//   windows and above its sign-in, when the app sends one for the card, and under each window when
//   it does not, never both and never neither; the caption stays above the sign-in when the sign-in
//   opens, and a screen reader is told it, in order, after the windows (D-135);
// - History's cards and Settings' names head each account as the popover's cards do: the name
//   given it or its provider's, then its address, inside the card;
// - one toolbar at the foot of every view: History, Settings and Help under the accounts, Done
//   under a panel, which has no Done of its own, and Quit set apart at its far end;
// - the toolbar stays at the window's foot while the accounts scroll above it, at every place along
//   the scroll, with nothing drawn under it and a hairline over it only while content is hidden below
//   it, scrolled by the keys, the wheel and Tab, in the order Tab and a screen reader have always
//   had, the app told the height of the whole of what it holds, and, in a window under 300 px, as at
//   220% zoom and more, the whole page scrolling as one (D-136);
// - what the window holds is as wide when its scrolling part scrolls as when it does not, in a Chrome
//   whose scroll bars take room as Windows' and Linux's do, with the cards the same distance from
//   each side and in line with the toolbar's labels (D-136);
// - a window's share is one token (the manager on #73, 2026-10-27): no share, such as 64%, is ever
//   drawn on two lines, whichever label is beside it, which wraps instead: the number under the
//   summary, each window's share in an account and the latest and highest share in each History
//   chart's caption, at 100% and 200% zoom and on at 300% and 500%, and every figure and its sign
//   anywhere in the window's text at 100% and 200%, judged by each share's own line boxes
//   (whole-shares.mjs);
// - a caption's words are whole (the manager on #73, 2026-10-27, 08:30 UTC): no word of four letters or
//   more, such as "Weekly", in the line saying where an account's windows were read, a History chart's
//   caption or a heatmap's is ever drawn on two lines, as "Wee" over "kly", at 100% and 200% zoom,
//   unless it, with what is written against it, such as a model's name in its brackets, is wider
//   than the caption, and the captions are held to the ones each copy has, in order, before any is
//   judged (whole-words.mjs);
// - nothing moves when the system asks for less motion;
// - nothing goes wrong on the page: no error, and nothing the app's content security policy refuses;
// - on QuotaDial Pro, each account in History has a Share image button named by its label, which
//   works from the keyboard and sends the app a whole 2400 by 1350 PNG for that account, drawn only
//   with the strings `shareStrings` lists (the label, never an address), every share in it written
//   whole and none cut short, in the stylesheet's text colours and the window's font, with tabular
//   figures; on QuotaDial Free there is no button and the feature's line shows once (D-115);
// - that picture is the account's card on the theme's material, #ececec in light and #2c2c2e in dark
//   (D-137): its corners are that colour, so is the margin round the card, the card's padding is the
//   card's fill laid over it, its edge is the card's hairline, and nothing is drawn as a plate behind
//   a chart, each read from the PNG's own pixels (share-picture.mjs);
// - and each chart's 90% line in it is dashed 6/6 in --chart-limit along the whole of it, over the
//   card and over the soft fill under a chart's line, read pixel by pixel but where the chart's own
//   line covers it (the manager on #73, 2026-10-27, 11:26 UTC; share-picture.mjs), and the run says
//   how much of the lines it read and how much it left out.
//
// Each check runs over the accounts, Settings, History and Help, in the light and dark themes, for a
// QuotaDial Pro and a QuotaDial Free copy, and a Pro copy with ten accounts, and over History alone for
// a Pro copy with one account of seven windows, with the made-up accounts in window-fixtures.mjs
// answering in place of the app. It drives Chrome (Chromium is WebView2's engine on Windows) through the
// DevTools protocol, with nothing to install: Chrome is found where it usually is, or named by
// CHROME. `npm run build` first; then `npm run check:window`; `-- --screenshots <folder>` saves a
// picture of the whole of each view, as tall as it needs, `-- --window-pictures <folder>` saves
// pictures of each view as the window shows it, no taller than the app lets it be, at the top and
// half way down what scrolls (`-- --window-zoom 2` at 200% zoom, `3` at 300%: a number above 0,
// and anything else is refused before Chrome is asked),
// `-- --share-images <folder>` saves the picture to share of each
// account in each copy and theme, `-- --scroll-bars classic` or `system` says whether Chrome's scroll
// bars take room, as Windows' and Linux's do (the checks' own choice), or lie over what they scroll,
// as a Mac's do (the pictures' own choice), `-- --only "pro, dark"` checks only the views named so, and
// `-- --fixtures <module>` answers from another set of copies (the website's pictures use
// `site-fixtures.mjs`). Fails, naming the view and the control, when a check does not pass.
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { createServer } from "node:http";
import { tmpdir } from "node:os";
import { delimiter, extname, join, normalize, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { launch, wantsClassicScrollbars } from "./chrome.mjs";
import { waitForScrollEnd } from "./quiet.mjs";
import { viewportAt, zoomOf } from "./viewport.mjs";

// The picture's own list of what it draws, read straight from the window's source (Node strips its
// types), so the check holds the drawing to it.
import { ACCOUNT_HEADER, CARD, HEIGHT as PICTURE_HEIGHT, WIDTH as PICTURE_WIDTH, chartBoxes, shareStrings } from "../src/share.ts";
import { MATERIAL, dashColours, dashPlaces, decodePng, fromHex, pictureVerdicts } from "./share-picture.mjs";
import { PLACES, SHARES, expectedShares, isShare, lineVerdicts } from "./whole-shares.mjs";
import { CAPTIONS, CAPTION_WORDS, captionVerdict, expectedCaptions, wordVerdicts } from "./whole-words.mjs";

/** The picture to share, in pixels: fixed, as the app checks it, whatever the window's source says. */
const SHARED = { width: 2400, height: 1350 };

const DIST = fileURLToPath(new URL("../dist/", import.meta.url));
const CONFIG = JSON.parse(
  readFileSync(new URL("../src-tauri/tauri.conf.json", import.meta.url), "utf8"),
);
/** The popover's width (POPOVER_WIDTH in src-tauri/src/lib.rs) and its tallest height. */
const WIDTH = CONFIG.app.windows[0].width;
const HEIGHT = 640;
/** The popover at 100%: its size at any other zoom is only ever `viewportAt(zoom, POPOVER)`. */
const POPOVER = { width: WIDTH, height: HEIGHT };
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
  PageUp: { key: "PageUp", code: "PageUp", windowsVirtualKeyCode: 33 },
  PageDown: { key: "PageDown", code: "PageDown", windowsVirtualKeyCode: 34 },
  End: { key: "End", code: "End", windowsVirtualKeyCode: 35 },
  Home: { key: "Home", code: "Home", windowsVirtualKeyCode: 36 },
  ArrowUp: { key: "ArrowUp", code: "ArrowUp", windowsVirtualKeyCode: 38 },
  ArrowDown: { key: "ArrowDown", code: "ArrowDown", windowsVirtualKeyCode: 40 },
};
const SHIFT = 8;

const failures = [];
let checks = 0;
/**
 * How much of the pictures' dashed 90% lines their verdicts read: of the pixels the lines cover
 * whole (`all`), those judged and how many of them are over the soft fill, and those left out where
 * a chart's own line covers its 90% line (`dashPlaces`); the rest are within a pixel of a dash's end.
 */
const dashes = { all: 0, judged: 0, overFill: 0, covered: 0 };

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
    // What is behind each piece of text, in the canvas's own pixels, so its contrast can be
    // measured against what is really there: across its middle just before it is written, and
    // just above it once the picture is finished, in case something was drawn over it after.
    const texts = new WeakMap();
    CanvasRenderingContext2D.prototype.fillText = function (text, x, y, ...rest) {
      const width = this.measureText(String(text)).width;
      const size = Number(this.font.match(/([0-9.]+)px/)?.[1] ?? 10);
      const place = this.getTransform();
      const at = (along, up) => [
        Math.round(place.a * (x + width * along) + place.e),
        Math.round(place.d * (y - up) + place.f),
      ];
      const ground = [0.15, 0.5, 0.85].map((along) => [...this.getImageData(...at(along, size * 0.35), 1, 1).data]);
      // Just over its figures and capitals, below the line above.
      const above = [0.15, 0.5, 0.85].map((along) => at(along, size * 0.75 + 3));
      const item = { text: String(text), fill: String(this.fillStyle), font: this.font, x, y, width, ground };
      drawn.push(item);
      texts.set(this.canvas, [...(texts.get(this.canvas) ?? []), { item, above }]);
      return fillText.call(this, text, x, y, ...rest);
    };
    const toBlob = HTMLCanvasElement.prototype.toBlob;
    HTMLCanvasElement.prototype.toBlob = function (...args) {
      const context = this.getContext("2d");
      for (const { item, above } of texts.get(this) ?? []) {
        for (const point of above) {
          item.ground.push([...context.getImageData(...point, 1, 1).data]);
        }
      }
      return toBlob.apply(this, args);
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
    // When anything last scrolled and when a scroll last ended, on the page's own clock, which is how
    // the check knows a scroll is over (quiet.mjs).
    window.__check.lastScroll = 0;
    window.__check.lastEnd = 0;
    document.addEventListener("scroll", () => { window.__check.lastScroll = performance.now(); }, true);
    document.addEventListener("scrollend", () => { window.__check.lastEnd = performance.now(); }, true);
    // What the window's own code scrolls, and how it asks to: an assignment to scrollTop or a call to
    // scrollTo and the like, whose behaviour is the stylesheet's unless the call says (the keys and the
    // wheel scroll by the browser's own code, which this does not see).
    window.__check.scrolls = [];
    const scrollTop = Object.getOwnPropertyDescriptor(Element.prototype, "scrollTop");
    Object.defineProperty(Element.prototype, "scrollTop", {
      ...scrollTop,
      set(value) {
        window.__check.scrolls.push({ how: "scrollTop", behavior: "the stylesheet's" });
        scrollTop.set.call(this, value);
      },
    });
    for (const how of ["scrollTo", "scrollBy", "scroll", "scrollIntoView"]) {
      const original = Element.prototype[how];
      Element.prototype[how] = function (...given) {
        const behavior = given[0] && typeof given[0] === "object" && given[0].behavior ? given[0].behavior : "the stylesheet's";
        window.__check.scrolls.push({ how, behavior });
        return original.apply(this, given);
      };
    }
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
  panels: [...document.querySelectorAll('main .content > section')].map((section) => section.className),
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
        // What a box that scrolls holds beyond its top or its bottom is reached by scrolling, so it
        // is not cut off there; beyond its sides, which never scroll, it is.
        const scrolls = style.overflowY === 'auto' || style.overflowY === 'scroll';
        const beyondEnds = !scrolls && (rect.top < edge.top - 0.5 || rect.bottom > edge.bottom + 0.5);
        if (rect.left < edge.left - 0.5 || rect.right > edge.right + 0.5 || beyondEnds) {
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
    // A scroll that glides moves too. Whether Chrome glides is the system's choice, and this check's
    // Chrome is told not to, so what is read is what the stylesheet asks for.
    if (style.scrollBehavior !== 'auto') {
      found.push(element.tagName.toLowerCase() + (element.className ? '.' + element.className : '') + ' still scrolls in a glide');
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

/**
 * In the page: every card in the window, in order, with where it is, and for an account's card its
 * heading's parts: its title, its address line and the address itself, where each is, whether it
 * is shown, and whether the address can be selected.
 */
const CARDS = `(() => {
  const place = (element) => {
    const { top, bottom, left, right, width, height } = element.getBoundingClientRect();
    const style = getComputedStyle(element);
    return { top, bottom, left, right, width, height, shown: element.getClientRects().length > 0 && style.visibility !== 'hidden' && Number(style.opacity) > 0 && !element.closest('.visually-hidden') };
  };
  return [...document.querySelectorAll('main .content > .card')].map((card) => {
    const heading = card.querySelector('h2');
    const address = card.querySelector('.address');
    const name = card.querySelector('.address .name');
    return {
      account: card.matches('section.account'),
      what: card.getAttribute('aria-label') || heading?.textContent.trim() || card.className,
      card: place(card),
      labelled: card.getAttribute('aria-labelledby'),
      heading: heading ? { id: heading.id, holds: [address, name].every((part) => !part || heading.contains(part)) } : null,
      title: card.querySelector('.title')?.textContent ?? null,
      address: address ? { text: address.textContent.replace(/\\s+/g, ' ').trim(), ...place(address) } : null,
      name: name ? { text: name.textContent, select: getComputedStyle(name).userSelect, ...place(name) } : null,
    };
  });
})()`;

/** In the page: the messages that can name a command, and every command shown as code in them. */
const COMMANDS = `(() => {
  const messages = [...document.querySelectorAll('.note, .sign-in p, .problems li')];
  return {
    // What each message says outside its code, where no backtick may be left.
    words: messages.map((message) => {
      const copy = message.cloneNode(true);
      for (const code of copy.querySelectorAll('code')) code.remove();
      return copy.textContent;
    }),
    codes: [...document.querySelectorAll('main code')].map((code) => ({
      text: code.textContent,
      select: getComputedStyle(code).userSelect,
      inMessage: messages.some((message) => message.contains(code)),
    })),
  };
})()`;

/** The commands `text` names between backticks, as the copies' messages write them. */
function commandsIn(text) {
  return [...(text ?? "").matchAll(/(?:^|[\s(])`([^`\s][^`]*?)`(?=$|[\s.,;:!?)])/g)].map((found) => found[1]);
}

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
   * Where what scrolls has got to once the scroll an input began is over: a key's or a wheel's can
   * start late and glide, and on a slow machine stall between frames, so the wait is for the page to
   * say the scroll has ended, and not for the position to stop changing (quiet.mjs).
   */
  async scrolled() {
    await waitForScrollEnd(() =>
      this.evaluate("({ now: performance.now(), lastScroll: window.__check.lastScroll, lastEnd: window.__check.lastEnd })"),
    );
    return this.position();
  }

  /** Where what scrolls is now. */
  position() {
    return this.evaluate(`(${SCROLLER}).scrollTop`);
  }

  /** Whether `expression` becomes true in the page within `ms`; it is for the caller to fail the check if not. */
  async eventually(expression, ms = WAIT) {
    const end = Date.now() + ms;
    while (Date.now() < end) {
      if (await this.evaluate(expression)) {
        return true;
      }
      await new Promise((done) => setTimeout(done, 25));
    }
    return false;
  }

  /**
   * Shows the page as the window does at `zoom`: zooming in narrows it in CSS pixels. Where scroll
   * bars take no room, as on a Mac by default, the page is narrowed by a scroll bar as well, so
   * every system checks the room Windows and Linux leave. The window is its tallest unless
   * `height` says, in CSS pixels, how tall it is.
   */
  async zoom(zoom, height = viewportAt(zoom, POPOVER).height) {
    this.bar ??= (await this.evaluate(SCROLL_BAR)) > 0 ? 0 : CLASSIC_BAR;
    await this.send("Emulation.setDeviceMetricsOverride", {
      width: viewportAt(zoom, POPOVER).width - this.bar,
      height,
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
    if (panel !== "accounts") {
      await this.open(panel);
    }
  }

  /** Opens `panel` from the footer, by keyboard, from where the focus is. */
  async open(panel) {
    for (let presses = 0; presses < 40; presses += 1) {
      await this.press("Tab");
      if ((await this.evaluate(FOCUSED)).opener === panel) {
        await this.press(OPEN_WITH[panel]);
        await this.until(
          `!!document.querySelector('main .content > section.${panel} h2')`,
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

  /** Clicks the middle of the `index`th element `selector` finds, as a person would, and answers
   * what is selected after, or null when there is no such element. */
  async clickAndSelect(selector, index) {
    const at = await this.evaluate(`(() => {
      const element = document.querySelectorAll(${JSON.stringify(selector)})[${index}];
      if (!element) return null;
      element.closest('details')?.setAttribute('open', '');
      window.getSelection().removeAllRanges();
      element.scrollIntoView({ block: 'center' });
      const box = element.getClientRects()[0];
      return { x: box.left + Math.min(box.width / 2, 12), y: box.top + box.height / 2 };
    })()`);
    if (!at) {
      return null;
    }
    for (const type of ["mousePressed", "mouseReleased"]) {
      await this.send("Input.dispatchMouseEvent", { type, x: at.x, y: at.y, button: "left", clickCount: 1 });
    }
    await this.settle();
    return this.evaluate("window.getSelection().toString()");
  }

  /** What a screen reader calls the `index`th element `selector` finds, or null when there is none. */
  async called(selector, index) {
    const { result } = await this.send("Runtime.evaluate", {
      expression: `document.querySelectorAll(${JSON.stringify(selector)})[${index}]`,
    });
    if (!result.objectId) {
      return null;
    }
    const { nodes } = await this.send("Accessibility.getPartialAXTree", {
      objectId: result.objectId,
      fetchRelatives: false,
    });
    return String(nodes[0]?.name?.value ?? "").trim();
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
async function checkView(view, panel, operate, copy, light) {
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
    const seen = await view.evaluate(FOCUS_IN_VIEW);
    check(label, seen !== true && `"${controls[index]}" ${seen}, when Tab reaches it`);
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

  if (panel === "history" || panel === "settings") {
    await checkHeadings(view, copy, panel);
  }
  await checkToolbar(view, panel);
  // The order does not change with the theme.
  if (light) {
    await checkOrder(view, copy, panel);
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
    await checkFrame(view, zoom);
    await checkShares(view, copy, panel, zoom);
    await checkWords(view, copy, panel, zoom);
    if (panel === "accounts" || panel === "settings") {
      await checkCommands(view, copy, panel, zoom);
    }
    if (panel === "accounts") {
      await checkCards(view, copy, zoom);
      await checkSummary(view, copy, zoom);
      await checkSources(view, copy, zoom);
    }
  }
  // A share is held whole past 200% too, which the other checks stop at.
  for (const zoom of SHARE_ZOOMS_PAST_200) {
    await view.zoom(zoom);
    await view.settle();
    await checkShares(view, copy, panel, zoom);
  }
  await view.zoom(1);
  // What the app is told does not change with the theme, nor does how the window scrolls: once.
  if (light) {
    await checkFit(view);
    await checkWidth(view);
  }
  if (light && copy.forScrolling) {
    await checkKeys(view, panel);
    await checkWheel(view, panel);
    if (panel === "accounts") {
      await checkSizes(view);
      await checkStop(view);
      await checkTop(view);
    }
  }

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

/** The space between two cards, in CSS pixels, as style.css sets it. */
const CARD_GAP = 8;

/**
 * Each account as a card of its own, its address easy to see (Matt's ask of 2026-10-26, D-132): a
 * card per account, in the copy's order, each at least the cards' gap from the next and none
 * overlapping, all in one column. Its title is the name given it, or its provider's own; its
 * second line its address, after the provider's name when the title is a name given, or nothing
 * when its name says nothing beyond its provider (D-124). The address is visible, inside its card
 * and its heading, and can be selected, and a screen reader calls the card by its heading, address
 * and all.
 */
async function checkCards(view, copy, zoom) {
  const label = `${view.label}, cards at ${zoom * 100}%`;
  const cards = await view.evaluate(CARDS);
  for (let index = 1; index < cards.length; index += 1) {
    const [above, below] = [cards[index - 1], cards[index]];
    const gap = below.card.top - above.card.bottom;
    check(label, gap < CARD_GAP - 0.5 && `"${below.what}" is ${gap.toFixed(1)} px below "${above.what}", not ${CARD_GAP} or more`);
    const aside = Math.max(Math.abs(below.card.left - above.card.left), Math.abs(below.card.right - above.card.right));
    check(label, aside > 0.5 && `"${below.what}" is not in line with "${above.what}"`);
  }
  const accounts = cards.filter((card) => card.account);
  check(label, accounts.length !== copy.meter.accounts.length && `the window shows ${accounts.length} account cards for ${copy.meter.accounts.length} accounts`);
  for (const [index, account] of copy.meter.accounts.entries()) {
    const card = accounts[index];
    if (!card) {
      break;
    }
    const what = [account.provider, account.alias, account.name].filter(Boolean).join(" ");
    const title = account.alias ?? account.provider;
    check(label, card.title !== title && `${what}'s card is titled "${card.title}", not "${title}"`);
    const line = account.alias === null ? account.name : [account.name, account.provider].filter(Boolean).join(" · ");
    if (!line) {
      check(label, card.address !== null && `${what}'s card has the line "${card.address?.text}", though its name says nothing more`);
      continue;
    }
    check(label, card.address?.text !== line && `${what}'s card's second line is "${card.address?.text}", not "${line}"`);
    check(label, !card.heading?.holds && `${what}'s address is not in its card's heading`);
    check(label, (!card.heading || card.labelled !== card.heading.id) && `${what}'s card is not labelled by its heading`);
    if (!account.name) {
      continue;
    }
    const name = card.name;
    if (!name || name.text !== account.name) {
      check(label, `${what}'s address is not shown as a part of its own, which can be selected alone`);
      continue;
    }
    check(label, (!name.shown || name.width <= 0 || name.height <= 0) && `${what}'s address is not visible`);
    const outside =
      name.left < card.card.left - 0.5 ||
      name.right > card.card.right + 0.5 ||
      name.top < card.card.top - 0.5 ||
      name.bottom > card.card.bottom + 0.5;
    check(label, outside && `${what}'s address is not inside its card`);
    check(label, name.select !== "text" && `${what}'s address cannot be selected to copy it (user-select: ${name.select})`);
    if (zoom === 1) {
      const called = await view.called("main .content > section.account", index);
      const heard = called !== null && called.includes(title) && called.includes(account.name);
      check(label, !heard && `a screen reader calls ${what}'s card "${called}", not by its name and address`);
    }
  }
}

/** In the page: each account's heading in the panel, its title and second line, and whether the
 * line is inside its card. */
const HEADINGS = (selector) => `[...document.querySelectorAll(${JSON.stringify(selector)})].map((part) => {
  const card = part.closest('.card');
  const line = part.querySelector('.address');
  const inside = (element) => {
    if (!element || !card) return false;
    const [inner, outer] = [element.getBoundingClientRect(), card.getBoundingClientRect()];
    return inner.width > 0 && inner.left >= outer.left - 0.5 && inner.right <= outer.right + 0.5 && inner.top >= outer.top - 0.5 && inner.bottom <= outer.bottom + 0.5;
  };
  return {
    title: part.querySelector('.title')?.textContent ?? null,
    line: line ? line.textContent.replace(/\\s+/g, ' ').trim() : null,
    inside: !line || inside(line),
  };
})`;

/**
 * The same heading for an account everywhere (D-132): History's cards, in the order History lists
 * them, are titled by the name given each account, or its provider's, with its address, and then
 * the provider under a name given, on the line under it, as the popover's are. Settings' names,
 * where the name to give is typed, are headed by which account it is: its address, with its
 * provider under it, or the provider alone when it has no address. Each second line is inside its
 * card.
 */
async function checkHeadings(view, copy, panel) {
  const label = `${view.label}, headings`;
  const [selector, accounts] =
    panel === "history"
      ? ["section.history .account-history .card-head", copy.history[7].accounts]
      : [".name-choice", copy.settings.accounts];
  const shown = await view.evaluate(HEADINGS(selector));
  check(label, shown.length !== accounts.length && `${panel} heads ${shown.length} accounts, not ${accounts.length}`);
  for (const [index, account] of accounts.entries()) {
    const heading = shown[index];
    if (!heading) {
      break;
    }
    const what = [account.provider, account.alias, account.name].filter(Boolean).join(" ");
    let [title, line] = [account.name || account.provider, account.name ? account.provider : ""];
    if (panel === "history") {
      title = account.alias ?? account.provider;
      line = account.alias === null ? account.name : [account.name, account.provider].filter(Boolean).join(" · ");
    }
    check(label, heading.title !== title && `${what} is titled "${heading.title}", not "${title}"`);
    check(label, (heading.line ?? "") !== line && `${what}'s second line is "${heading.line}", not "${line}"`);
    check(label, !heading.inside && `${what}'s second line is not inside its card`);
  }
}

/** In the page: the buttons of the footer's toolbar, in order, where each is, and the panel's own
 * buttons called Done. */
const TOOLBAR = `({
  buttons: [...document.querySelectorAll('footer .toolbar > button')].map((button) => {
    const { left, right, top } = button.getBoundingClientRect();
    return { text: button.textContent.trim(), left, right, top };
  }),
  panelDone: [...document.querySelectorAll('main .content > section button')].filter((button) => button.textContent.trim() === 'Done').length,
})`;

/**
 * One calm toolbar at the foot of every view (D-132): under the accounts, History, Settings and
 * Help, then Quit QuotaDial; under a panel, Done, where those three were, then Quit, and no Done in
 * the panel itself. Quit is set apart at the far end, further from the button before it than the
 * others are from each other.
 */
async function checkToolbar(view, panel) {
  const label = `${view.label}, toolbar`;
  const { buttons, panelDone } = await view.evaluate(TOOLBAR);
  const expected = panel === "accounts" ? ["History", "Settings", "Help", "Quit QuotaDial"] : ["Done", "Quit QuotaDial"];
  const texts = buttons.map((button) => button.text);
  check(label, texts.join("|") !== expected.join("|") && `the footer's toolbar holds ${JSON.stringify(texts)}, not ${JSON.stringify(expected)}`);
  check(label, panelDone > 0 && `the ${panel} panel has a Done of its own as well as the toolbar's`);
  if (buttons.length < 2) {
    return;
  }
  const [before, quit] = buttons.slice(-2);
  const between = buttons.slice(1, -1).map((button, index) => button.left - buttons[index].right);
  const apart = quit.top === before.top ? quit.left - before.right : Number.POSITIVE_INFINITY;
  check(label, apart <= Math.max(0, ...between) + 8 && `Quit QuotaDial is not set apart from ${before.text}`);
}

/** In the page: the line under the summary's number, and whether it is shown. */
const SUMMARY = `(() => {
  const line = document.querySelector('header.summary .headline');
  if (!line) return null;
  const { width, height } = line.getBoundingClientRect();
  return { text: line.textContent, shown: line.getClientRects().length > 0 && width > 0 && height > 0 };
})()`;

/**
 * The summary names the account (D-132): its line shows the headline the app sent, and that names
 * the window that sets the number and its account, by the name given it, or its address, or its
 * provider alone when the address says nothing more, so two accounts of one provider are never
 * confused. The account is the one whose window holds the number, as the app chooses it: the
 * fullest window of an account that is not waiting for a sign-in.
 */
async function checkSummary(view, copy, zoom) {
  const label = `${view.label}, summary at ${zoom * 100}%`;
  const summary = await view.evaluate(SUMMARY);
  const { meter } = copy;
  check(label, (!summary || !summary.shown) && "the line under the number is not shown");
  if (!summary || meter.headline === null) {
    return;
  }
  check(label, summary.text !== meter.headline && `the line under the number reads "${summary.text}", not "${meter.headline}"`);
  const [account, window] = meter.accounts
    .filter((one) => !["sign-in required", "unsupported"].includes(one.status))
    .flatMap((one) => one.windows.map((each) => [one, each]))
    .reduce((fullest, pair) => (pair[1].used_pct > (fullest?.[1].used_pct ?? -1) ? pair : fullest), null) ?? [];
  if (!account) {
    check(label, "the copy has a headline but no window to set the number");
    return;
  }
  const named = account.alias ?? (account.name || account.provider);
  check(label, Math.floor(window.used_pct) !== meter.percent && `the copy's number ${meter.percent}% is not its fullest window's`);
  check(label, !summary.text.includes(window.label) && `the line under the number does not name the window, ${window.label}`);
  check(label, !summary.text.includes(named) && `the line under the number does not name the account, "${named}"`);
}

/**
 * The zooms past 200%, where the other checks stop, that a share is held whole at as well: 300%,
 * where the page scrolls as one, and 500%, which the app zooms to and on past, where a window is
 * too narrow for almost anything and a share is cut off at its edge, which is not broken.
 */
const SHARE_ZOOMS_PAST_200 = [3, 5];

/**
 * A window's share is one token (the manager's decision on #73, 2026-10-27): such as 64%, it is
 * never drawn on two lines, as "64" over "%", whatever label is beside it, which wraps instead.
 * In each view at each zoom, the places the window draws a share (the number under the summary,
 * each window's share in the accounts, the latest and highest in each History chart's caption)
 * hold exactly the shares the copy has, in order, so that a check that finds none cannot pass; and,
 * at 100% and 200%, every figure and its sign anywhere in the window's text is held to one line
 * too, so that a place the window comes to draw one in, which the list of places does not name, is
 * held as well. Each share is judged by its own line boxes (whole-shares.mjs).
 */
async function checkShares(view, copy, panel, zoom) {
  const label = `${view.label}, shares at ${Math.round(zoom * 100)}%`;
  const places = await view.evaluate(SHARES(PLACES));
  const written = places.map((share) => share.text);
  const wanted = expectedShares(copy, panel);
  check(label, written.join(" ") !== wanted.join(" ") && `the window writes the shares ${JSON.stringify(written)}, not the ${JSON.stringify(wanted)} its copy has`);
  const everywhere = zoom <= 2 ? await view.evaluate(SHARES("main")) : places;
  for (const verdict of lineVerdicts(everywhere)) {
    check(label, verdict);
  }
}

/**
 * A caption's words are whole (the manager's decision on #73, 2026-10-27, 08:30 UTC, D-139): in each
 * caption the window draws (the line saying where and when an account's windows were read, each
 * History chart's caption and each heatmap's), at 100% and 200%, no word of four letters or more,
 * such as "Weekly", is drawn on two lines, as "Wee" over "kly", unless its run, the word with what
 * is written against it, such as a model's name in its brackets, is wider than the whole caption:
 * that has to break somewhere or be cut off, which the check of cut-off text holds, and the browser
 * breaks it where the line ends, inside the word if that is where. The captions the window draws
 * must be exactly the ones the copy has, in order, with the words each has, before any line is
 * judged, so that a check that finds none cannot pass. Each word is judged by its own line boxes,
 * and its run by its own (whole-words.mjs).
 */
async function checkWords(view, copy, panel, zoom) {
  const label = `${view.label}, caption words at ${Math.round(zoom * 100)}%`;
  const captions = await view.evaluate(CAPTION_WORDS(CAPTIONS));
  check(label, captionVerdict(captions, expectedCaptions(copy, panel)));
  for (const verdict of wordVerdicts(captions)) {
    check(label, verdict);
  }
}

/** The space between two windows of a card, in CSS pixels, as style.css sets it. */
const WINDOW_GAP = 10;

/**
 * In the page: for each account's card, in order, the lines that say where and when its windows
 * were read (the card's caption, `.source`), each window's own line, and the sign-in panel's
 * disclosure, with where each is, how it is drawn and whether a screen reader reaches it.
 */
const SOURCES = `(() => {
  const place = (element) => {
    const { top, bottom, left, right, width, height } = element.getBoundingClientRect();
    const style = getComputedStyle(element);
    return { top, bottom, left, right, width, height, shown: element.getClientRects().length > 0 && style.visibility !== 'hidden' && Number(style.opacity) > 0 && !element.closest('.visually-hidden') };
  };
  const words = (element) => element.textContent.replace(/\\s+/g, ' ').trim();
  const follows = (first, second) => !!(first.compareDocumentPosition(second) & Node.DOCUMENT_POSITION_FOLLOWING);
  return [...document.querySelectorAll('main .content > section.account')].map((card) => {
    const heading = card.querySelector('h2');
    const list = card.querySelector('ul.windows');
    const panel = card.querySelector('details.sign-in');
    const detail = card.querySelector('.window .detail');
    const address = card.querySelector('.address');
    return {
      card: place(card),
      lines: [...card.querySelectorAll('.source')].map((line) => {
        const look = getComputedStyle(line);
        const above = line.previousElementSibling;
        return {
          text: words(line),
          ...place(line),
          inHeading: !!heading?.contains(line),
          inPanel: !!panel?.contains(line),
          hidden: !!line.closest('[aria-hidden="true"], [hidden]'),
          afterWindows: !!list && follows(list, line),
          beforePanel: !panel || follows(line, panel),
          size: parseFloat(look.fontSize),
          weight: Number(look.fontWeight),
          colour: look.color,
          room: above ? line.getBoundingClientRect().top - above.getBoundingClientRect().bottom : null,
        };
      }),
      windows: [...card.querySelectorAll('.window')].map((window) => {
        const copy = window.querySelector('.detail').cloneNode(true);
        for (const badge of copy.querySelectorAll('.badge')) badge.remove();
        const { top, bottom } = window.getBoundingClientRect();
        return { detail: words(copy), top, bottom };
      }),
      detail: detail ? { size: parseFloat(getComputedStyle(detail).fontSize), colour: getComputedStyle(detail).color } : null,
      addressSize: address ? parseFloat(getComputedStyle(address).fontSize) : null,
      panel: panel ? { summary: place(panel.querySelector('summary')), text: panel.querySelector('p') ? place(panel.querySelector('p')) : null, last: !panel.nextElementSibling } : null,
    };
  });
})()`;

/**
 * Where and when each account's windows were read (#52: nothing reaches a person without its age
 * and origin; D-135): said once for the card, in a caption of its own, when the app sends one, and
 * under each window, as before, when the app sends none, never both and never neither. The caption
 * is the card's alone: visible, inside the card, quiet (no larger and no bolder than a window's
 * lines, in their colour, and smaller than the address), under the windows with the space between
 * windows above it, and above the sign-in's disclosure, where it stays when the panel opens, so a
 * screen reader reads the windows, then where they were read, then the sign-in. It is outside the
 * card's heading, which names the card, and it is not hidden from a screen reader.
 */
async function checkSources(view, copy, zoom) {
  const label = `${view.label}, where the windows were read at ${zoom * 100}%`;
  const accounts = copy.meter.accounts;
  const closed = await view.evaluate(SOURCES);
  check(label, closed.length !== accounts.length && `the window shows ${closed.length} account cards for ${accounts.length} accounts`);
  // The sign-in panels open, to see that the caption stays where it is, and shut again.
  const was = await view.evaluate("[...document.querySelectorAll('details.sign-in')].map((panel) => panel.open)");
  await view.evaluate("document.querySelectorAll('details.sign-in').forEach((panel) => { panel.open = true; })");
  await view.settle();
  const opened = await view.evaluate(SOURCES);
  await view.evaluate(`${JSON.stringify(was)}.forEach((open, index) => { document.querySelectorAll('details.sign-in')[index].open = open; })`);
  await view.settle();
  for (const [index, account] of accounts.entries()) {
    const card = closed[index];
    if (!card) {
      break;
    }
    const what = [account.provider, account.alias, account.name].filter(Boolean).join(" ");
    const own = account.windows.filter((window) => window.fetched !== null);
    check(label, account.fetched !== null && own.length > 0 && `the copy gives ${what} a line for its card and one for ${own.length} of its windows, which the app never sends`);
    check(label, account.fetched !== null && account.windows.length === 0 && `the copy gives ${what} a line for where its windows were read and no windows`);
    check(label, card.windows.length !== account.windows.length && `${what}'s card shows ${card.windows.length} windows for ${account.windows.length}`);
    // A window says its own where and when only when the card does not.
    for (const [position, window] of account.windows.entries()) {
      const expected = window.fetched === null ? window.resets : `${window.resets}, ${window.fetched}`;
      const shown = card.windows[position]?.detail;
      check(label, shown !== expected && `${what}'s ${window.label} says "${shown}", not "${expected}"`);
    }
    if (account.fetched === null) {
      check(label, card.lines.length > 0 && `${what}'s card says "${card.lines[0].text}" for where its windows were read, though the app sent no line for the card`);
      continue;
    }
    check(label, card.lines.length !== 1 && `${what}'s card says where its windows were read ${card.lines.length} times, not once`);
    const [line] = card.lines;
    if (!line) {
      continue;
    }
    check(label, line.text !== account.fetched && `${what}'s card says "${line.text}", not "${account.fetched}"`);
    check(label, (!line.shown || line.width <= 0 || line.height <= 0) && `${what}'s line for where its windows were read is not visible`);
    const outside = line.left < card.card.left - 0.5 || line.right > card.card.right + 0.5 || line.top < card.card.top - 0.5 || line.bottom > card.card.bottom + 0.5;
    check(label, outside && `${what}'s line for where its windows were read is not inside its card`);
    check(label, (line.inHeading || line.inPanel) && `${what}'s line for where its windows were read is inside its card's ${line.inHeading ? "heading, which names the card" : "sign-in panel"}`);
    check(label, line.hidden && `${what}'s line for where its windows were read is hidden from screen readers`);
    // Quiet: no larger, bolder or louder than a window's lines, and smaller than the address.
    const quiet = card.detail && line.size <= card.detail.size + 0.1 && line.weight <= 400 && line.colour === card.detail.colour;
    check(label, !quiet && `${what}'s line for where its windows were read is not as quiet as a window's lines (${line.size} px, weight ${line.weight}, ${line.colour})`);
    check(label, card.addressSize !== null && line.size >= card.addressSize && `${what}'s line for where its windows were read is not smaller than its address (${line.size} px, ${card.addressSize} px)`);
    // Under its windows, with the space between windows above it.
    const last = card.windows.at(-1);
    check(label, (!line.afterWindows || (last && line.top < last.bottom - 0.5)) && `${what}'s line for where its windows were read is not under its windows`);
    const gap = card.windows.length > 1 ? card.windows[1].top - card.windows[0].bottom : WINDOW_GAP;
    check(label, line.room !== null && Math.abs(line.room - gap) > 0.5 && `${what}'s line for where its windows were read has ${line.room.toFixed(1)} px above it, not the ${gap.toFixed(1)} px between windows`);
    // Above the sign-in's disclosure, which is the card's last thing, and still there when the panel is open.
    if (card.panel) {
      check(label, !card.panel.last && `${what}'s sign-in disclosure is not the last thing in its card`);
      check(label, (!line.beforePanel || line.bottom > card.panel.summary.top + 0.5) && `${what}'s line for where its windows were read is not above the sign-in disclosure`);
      const [after] = opened[index]?.lines ?? [];
      const open = opened[index]?.panel;
      if (!after || !open) {
        check(label, `${what}'s line for where its windows were read is gone when the sign-in panel opens`);
        continue;
      }
      // Measured from its card's top, which scrolling, and the panels of the cards above opening, move as a whole.
      const [from, now] = [line.top - card.card.top, after.top - opened[index].card.top];
      check(label, Math.abs(now - from) > 0.5 && `${what}'s line for where its windows were read moves ${(now - from).toFixed(1)} px within its card when the sign-in panel opens`);
      check(label, after.bottom > open.summary.top + 0.5 && `${what}'s line for where its windows were read is not above the sign-in disclosure when its panel is open`);
      check(label, open.text && open.text.top < after.bottom - 0.5 && `${what}'s open sign-in panel is not below the line for where its windows were read`);
    }
  }
  // A screen reader is told each caption, in the order of the card: the windows, where they were
  // read, the sign-in. The order is the markup's, which the line's place after the windows and
  // before the panel has already shown; this checks that the accessibility tree has the words.
  if (zoom === 1) {
    const { nodes } = await view.send("Accessibility.getFullAXTree");
    // The tree has the words as drawn, the first letter capitalised by the stylesheet.
    const told = (text) =>
      nodes.filter((node) => !node.ignored && node.role?.value === "StaticText" && String(node.name?.value ?? "").trim().toLowerCase() === text.toLowerCase()).length;
    const wanted = new Map();
    for (const account of accounts) {
      if (account.fetched !== null && account.windows.length > 0) {
        wanted.set(account.fetched, (wanted.get(account.fetched) ?? 0) + 1);
      }
    }
    for (const [text, cards] of wanted) {
      check(label, told(text) < cards && `a screen reader is told "${text}" ${told(text)} times for ${cards} cards`);
    }
  }
}

/**
 * The commands in what QuotaDial says (the manager's note on #85): each one a message in `panel`
 * names between backticks, such as Claude's sign-in command in an account's note, Gemini CLI's
 * `/model`, Antigravity's block in its sign-in panel or config.json's chmod in Settings, shows as
 * code, whole, with no backtick left in the message's words; and a click on it selects the whole
 * command, so it can be copied as it must be pasted, at 100% and at 200% zoom, where a long command,
 * such as the `chmod` of a folder with a long name, is drawn over several lines (D-139).
 */
async function checkCommands(view, copy, panel, zoom) {
  const label = `${view.label}, commands at ${Math.round(zoom * 100)}%`;
  const said =
    panel === "accounts"
      ? [
          ...copy.meter.accounts.flatMap((account) => [account.ok ? null : account.note, account.sign_in]),
          ...copy.meter.problems,
        ]
      : copy.settings.alert_problems;
  const expected = said.flatMap(commandsIn);
  const { words, codes } = await view.evaluate(COMMANDS);
  for (const left of words.filter((text) => text.includes("`"))) {
    check(label, `a message still shows a backtick: "${left.trim().slice(0, 80)}"`);
  }
  const shown = codes.filter((code) => code.inMessage);
  check(label, shown.length !== expected.length && `the window shows ${shown.length} commands as code for ${expected.length}`);
  for (const command of expected) {
    const index = codes.findIndex((code) => code.text === command);
    if (index < 0) {
      check(label, `"${command}" is not shown whole as code`);
      continue;
    }
    check(label, codes[index].select !== "all" && `"${command}" is not selected whole by a click (user-select: ${codes[index].select})`);
    const selected = await view.clickAndSelect("main code", index);
    check(label, selected !== command && `a click on "${command}" selects "${selected}", not the whole command`);
  }
}

/**
 * The shortest window, in CSS pixels, in which the toolbar stays at its foot while the accounts
 * scroll above it (`min-height` in style.css, D-136). The 640 px window is 320 CSS pixels tall at
 * 200% zoom, where the toolbar must stay, as the manager held on #73; the app zooms on past that, to
 * 500% and more, and in a window shorter than this the whole page scrolls as one.
 */
const PINNED_FROM = 300;

/**
 * The least room, in CSS pixels, that what scrolls is left in a window that keeps its toolbar at its
 * foot, with the toolbar and the origin line at their tallest, at the check's narrowest width: about
 * a card's heading and a window. With a Mac's fonts it is 204 at 200% zoom and 169 in the shortest
 * window that keeps the toolbar, 300 px tall; the floor is lower so that a wider font, as Windows
 * and Linux have, wrapping the toolbar onto one more row, still meets it (D-136).
 */
const ROOM = 120;

/**
 * In the page: how the window is held together: the window, the page's own scroll, the part that
 * scrolls with how far it is scrolled, and the footer with where it is, the line drawn along its
 * head, where the origin line under its toolbar is and, for each button of its toolbar, where it
 * is and whether a click at its middle would reach it. A window with no part that scrolls of its own is read as one whose whole page is that
 * part, which is how it scrolls.
 */
const FRAME = `(() => {
  const rect = (element) => {
    const { top, bottom, left, right } = element.getBoundingClientRect();
    return { top, bottom, left, right };
  };
  const main = document.querySelector('main');
  const footer = main.querySelector('footer');
  if (!footer) {
    return null;
  }
  const region = main.querySelector(':scope > .scroll') ?? main;
  const page = document.scrollingElement;
  return {
    window: { width: window.innerWidth, height: window.innerHeight },
    page: { top: page.scrollTop, height: page.scrollHeight },
    region: { ...rect(region), overflow: getComputedStyle(region).overflowY, top: region.scrollTop, height: region.scrollHeight, shown: region.clientHeight },
    footer: { ...rect(footer), line: getComputedStyle(footer).boxShadow },
    origin: footer.querySelector('.origin') ? rect(footer.querySelector('.origin')) : null,
    buttons: [...footer.querySelectorAll('.toolbar > button')].map((button) => {
      const box = button.getBoundingClientRect();
      const at = document.elementFromPoint(box.left + box.width / 2, box.top + box.height / 2);
      return { text: button.textContent.trim(), ...rect(button), reachable: !!at && (at === button || button.contains(at)) };
    }),
  };
})()`;

/** In the page: scrolls what scrolls to `along` of its way, 0 at its top and 1 at its end. */
const SCROLL_TO = (along) => `(() => {
  const scroller = ${SCROLLER};
  scroller.scrollTo({ top: Math.round((scroller.scrollHeight - scroller.clientHeight) * ${along}), behavior: 'instant' });
})()`;

/**
 * Whether `shadow`, an element's computed `box-shadow`, is a hairline along its head: inset, with
 * no blur and no spread, a pixel thick at most, and in a colour that shows.
 */
function hairline(shadow) {
  const found = shadow.match(/^rgba?\(([^)]+)\)\s+(-?[\d.]+)px\s+(-?[\d.]+)px\s+([\d.]+)px\s+(-?[\d.]+)px\s+inset$/);
  if (!found) {
    return false;
  }
  const alpha = found[1].split(",").map(Number)[3] ?? 1;
  const [across, down, blur, spread] = found.slice(2).map(Number);
  return across === 0 && blur === 0 && spread === 0 && down >= 0.4 && down <= 1.01 && alpha >= 0.05;
}

/**
 * The toolbar stays at the window's foot while the accounts scroll above it (D-136, from the
 * owner's ask of 2026-10-26 to make the window more professional): in a window at least
 * `PINNED_FROM` tall, at each of five places along the scroll from the top to the end, every button
 * of the toolbar and the line under it are inside the window, each button is what a click at its
 * middle reaches, the toolbar has not moved, the part that scrolls ends where the toolbar begins, so nothing is drawn under it, the
 * page itself does not scroll but that part does, to its very end, with no animation, and a
 * hairline is drawn along the toolbar's head exactly while content is hidden below it; and that
 * part is left at least `ROOM` to scroll. In a shorter window everything scrolls as one, as it did
 * before: the toolbar is in the window at the end of the page, no line is drawn, and the part with
 * the accounts does not scroll of itself.
 */
async function checkFrame(view, zoom, height = viewportAt(zoom, POPOVER).height) {
  const label = `${view.label}, the toolbar at ${Math.round(zoom * 100)}% in a window ${height} px tall`;
  await view.zoom(zoom, height);
  await view.settle();
  const go = async (along) => {
    await view.evaluate(SCROLL_TO(along));
    await view.settle();
    return view.evaluate(FRAME);
  };
  const top = await go(0);
  if (!top) {
    check(label, "the window has no toolbar");
    return;
  }
  const pinned = height >= PINNED_FROM;
  const scrolls = top.region.overflow !== "visible";
  check(label, pinned && !scrolls && `the accounts do not scroll above the toolbar, so the toolbar goes with them (overflow-y is ${top.region.overflow})`);
  check(label, !pinned && scrolls && `the toolbar is kept in place in a window under ${PINNED_FROM} px, where it takes too great a part of it`);

  if (!pinned) {
    // Everything scrolls as one: the page, to its end, with the toolbar at its foot.
    const end = await go(1);
    check(label, end.region.height - end.region.shown > 1 && "the accounts scroll of themselves as well as the page");
    const range = end.page.height - end.window.height;
    check(label, range > 1 && Math.abs(end.page.top - range) > 1.5 && `the page scrolls to ${end.page.top}, not to its end at ${range}`);
    check(label, end.buttons.some((button) => button.top < -0.5 || button.bottom > end.window.height + 0.5) && "the toolbar is not in the window at the end of the page");
    check(label, end.buttons.some((button) => !button.reachable) && "a click at the middle of a toolbar button at the end of the page would not reach it");
    check(label, end.origin && (end.origin.top < -0.5 || end.origin.bottom > end.window.height + 0.5) && "the line saying where the numbers come from is not in the window at the end of the page");
    for (const along of [0, 1]) {
      check(label, hairline((await go(along)).footer.line) && "a line is drawn above the toolbar though the whole page scrolls as one");
    }
    return;
  }

  const hidden = top.region.height - top.region.shown;
  check(label, hidden > 1 && top.region.shown < ROOM && `only ${top.region.shown} px is left to scroll above the toolbar, under the ${ROOM} px a window this tall must leave`);
  for (const along of [0, 0.25, 0.5, 0.75, 1]) {
    const now = await go(along);
    const at = `${along * 100}% of the way down`;
    const names = now.buttons.map((button) => button.text).join(", ");
    check(label, now.buttons.some((button) => button.top < -0.5 || button.bottom > now.window.height + 0.5) && `the toolbar (${names}) is not inside the window at ${at}`);
    check(label, now.buttons.some((button) => !button.reachable) && `a click at the middle of a toolbar button (${names}) at ${at} would not reach it`);
    check(label, now.origin && (now.origin.top < -0.5 || now.origin.bottom > now.window.height + 0.5) && `the line saying where the numbers come from is not inside the window at ${at}`);
    check(label, Math.abs(now.footer.top - top.footer.top) > 0.5 && `the toolbar moves at ${at}, from ${top.footer.top} to ${now.footer.top}`);
    check(label, now.region.bottom > now.footer.top + 0.5 && `the part that scrolls reaches ${(now.region.bottom - now.footer.top).toFixed(1)} px under the toolbar at ${at}`);
    check(label, (now.page.top !== 0 || now.page.height > now.window.height + 1) && `the page itself scrolls at ${at}, not only the accounts`);
    // A line along the toolbar's head exactly while content is hidden below that edge.
    const below = hidden - now.region.top > 1.5;
    check(label, hairline(now.footer.line) !== below && `${below ? "no line is" : "a line is"} drawn above the toolbar at ${at}, where content is ${below ? "" : "not "}hidden below it`);
    if (hidden > 1) {
      check(label, Math.abs(now.region.top - Math.round(hidden * along)) > 1.5 && `the accounts are scrolled to ${now.region.top} at ${at}, not ${Math.round(hidden * along)}`);
    }
  }
  // A pixel short of the end hides a pixel, which rounding to the screen can leave and which is no
  // content beneath the edge: no line.
  if (hidden >= 3) {
    await view.evaluate(`(() => { const scroller = ${SCROLLER}; scroller.scrollTop = scroller.scrollHeight - scroller.clientHeight - 1; })()`);
    await view.settle();
    const almost = await view.evaluate(FRAME);
    check(label, hairline(almost.footer.line) && "a line is drawn above the toolbar with a single pixel hidden below it, which is only rounding");
  }
}

/**
 * The window can be as tall as the most it may be, and the app is told how tall everything is. Once
 * the window is capped, the popover's own height stops growing with what it holds, so unless the
 * app is told what the part that scrolls holds beyond its edge, the window could never grow back
 * and a window made short by a small view would stay short. The app is told what the window holds,
 * whole, as measured with room for all of it; and told again, to the pixel, when more is put into
 * the window, or taken out of it, while it is at its tallest.
 */
async function checkFit(view) {
  const label = `${view.label}, the height the app is told`;
  await view.zoom(1);
  await view.settle();
  const told = () => view.evaluate("window.__check.calls.filter((call) => call.cmd === 'fit').at(-1)?.args ?? null");
  const first = await told();
  if (!first) {
    check(label, "the window never told the app how tall it needs to be");
    return;
  }
  await view.zoom(1, 12000);
  await view.settle();
  const whole = await view.evaluate("Math.ceil(document.querySelector('main').getBoundingClientRect().height)");
  check(label, first.height !== whole && `the app is told ${first.height} px, though what the window holds is ${whole} px tall`);
  await view.zoom(1);
  await view.settle();
  const put = 123;
  const added = await view.evaluate(`(() => {
    const content = document.querySelector('main .content');
    if (!content) return false;
    const spacer = document.createElement('div');
    spacer.id = 'check-spacer';
    spacer.style.height = '${put}px';
    content.append(spacer);
    return true;
  })()`);
  if (!added) {
    check(label, "what the window holds is not in a part of its own, to put anything into");
    return;
  }
  await view.settle();
  const grown = await told();
  check(label, grown.height !== first.height + put && `the app is told ${grown.height} px when ${put} px is put into the window, not ${first.height + put}`);
  await view.evaluate("document.getElementById('check-spacer').remove()");
  await view.settle();
  const shrunk = await told();
  check(label, shrunk.height !== first.height && `the app is told ${shrunk.height} px when what was put in is taken out, not ${first.height}`);
}

/**
 * A person who scrolls by keyboard has what a page has: with the focus in the part that scrolls, the
 * arrows, Page Up and Page Down, Home and End scroll it, and Space too where the focus is on nothing
 * that Space presses, as it is on a panel when it opens; and with nothing focused, as the window is
 * just opened, the first of those keys scrolls it, as it scrolled the page before the toolbar stayed
 * put, and Tab then reaches the first control, as it does from a window nothing has touched.
 */
async function checkKeys(view, panel) {
  const label = `${view.label}, scrolling by keyboard`;
  await view.zoom(1);
  await view.load(panel);
  if (panel === "accounts" && (await view.evaluate("document.querySelector('main > .scroll') !== null"))) {
    for (const key of ["PageDown", "Space", "ArrowDown", "End"]) {
      await view.load(panel);
      const room = await view.evaluate("(() => { const region = document.querySelector('main > .scroll'); return region.scrollHeight - region.clientHeight; })()");
      if (room < 300) {
        break;
      }
      await view.press(key);
      check(label, (await view.scrolled()) <= 0.5 && `${key}, with nothing focused, does not scroll the accounts`);
      await view.evaluate(FOCUSABLE);
      await view.press("Tab");
      const next = await view.evaluate(FOCUSED);
      check(label, next.index !== 0 && `Tab after ${key}, with nothing focused before it, reached ${next.what}, not the first control`);
    }
    await view.load(panel);
  }
  const hidden = await view.evaluate("(() => { const region = document.querySelector('main > .scroll'); return region ? region.scrollHeight - region.clientHeight : 0; })()");
  if (hidden < 300) {
    return;
  }
  await view.evaluate(FOCUSABLE);
  if (panel === "accounts") {
    await view.press("Tab");
  }
  let at = await view.position();
  const down = panel === "accounts" ? ["ArrowDown", "PageDown", "End"] : ["ArrowDown", "PageDown", "Space", "End"];
  for (const key of down) {
    await view.press(key);
    const now = await view.scrolled();
    check(label, at < hidden - 1 && now <= at && `${key} does not scroll the part with the accounts down (${at}, then ${now})`);
    at = now;
  }
  check(label, Math.abs(at - hidden) > 1.5 && `End leaves it at ${at}, not at its end, ${hidden}`);
  for (const key of ["ArrowUp", "PageUp", "Home"]) {
    await view.press(key);
    const now = await view.scrolled();
    check(label, at > 0.5 && now >= at && `${key} does not scroll it up (${at}, then ${now})`);
    at = now;
  }
  check(label, at > 0.5 && `Home leaves it at ${at}, not at its top`);
}

/**
 * In the page: `true` when the control with the focus is in view with its focus ring, inside what
 * shows it, or what is wrong. The ring is drawn 2 px out from a control and is 2 px thick.
 */
const FOCUS_IN_VIEW = `(() => {
  const element = document.activeElement;
  const box = element.getBoundingClientRect();
  const ring = 4;
  // Whatever clips what is drawn outside it, as the part that scrolls does and the footer does where
  // it keeps the room for a scroll bar: the window, and each of those that holds the control and
  // is not the control itself, which draws its own ring inside its edge.
  const shows = { top: 0, bottom: window.innerHeight };
  for (const clip of document.querySelectorAll('main > .scroll, main > footer')) {
    const style = getComputedStyle(clip);
    if (clip === element || !clip.contains(element) || (style.overflowX === 'visible' && style.overflowY === 'visible')) continue;
    const edges = clip.getBoundingClientRect();
    shows.top = Math.max(shows.top, edges.top);
    shows.bottom = Math.min(shows.bottom, edges.bottom);
  }
  if (box.top - ring < shows.top - 0.5) return 'is above what shows it, or its ring is cut off by that edge';
  if (box.bottom + ring > shows.bottom + 0.5) return 'is below what shows it, or its ring is cut off by that edge, or it is under the toolbar';
  return true;
})()`;

/** In the page: where each control Tab reaches is, in the order Tab reaches them. */
const STOP_PLACES = `(() => {
  const region = document.querySelector('main > .scroll');
  const footer = document.querySelector('main > footer');
  return window.__focusables.map((element) => (region?.contains(element) ? 'content' : footer?.contains(element) ? 'toolbar' : 'elsewhere'));
})()`;

/**
 * The words a screen reader meets, in the order it reads them: the accessibility tree's text and
 * buttons, depth first.
 */
async function readOrder(view) {
  const { nodes } = await view.send("Accessibility.getFullAXTree");
  const byId = new Map(nodes.map((node) => [node.nodeId, node]));
  const heard = [];
  const walk = (node) => {
    if (!node) {
      return;
    }
    const name = String(node.name?.value ?? "").trim();
    if (!node.ignored && name && ["StaticText", "button"].includes(node.role?.value)) {
      heard.push(name);
    }
    for (const id of node.childIds ?? []) {
      walk(byId.get(id));
    }
  };
  walk(nodes[0]);
  return heard;
}

/**
 * Tab reaches what is in the scrolling part first and the toolbar after, as it always has, and a
 * screen reader reads in the same order: the number's line, then the accounts or the panel, then
 * the toolbar's buttons, then where the numbers come from.
 */
async function checkOrder(view, copy, panel) {
  const label = `${view.label}, order`;
  await view.evaluate(FOCUSABLE);
  const places = (await view.evaluate(STOP_PLACES)).join(" ");
  check(label, !/^(content )*(toolbar ?)+$/.test(places) && `Tab reaches ${places}, not the scrolling part's controls and then the toolbar's`);
  const heard = await readOrder(view);
  const { meter } = copy;
  const accounts = meter.accounts;
  const [first, last] = [accounts[0], accounts.at(-1)].map((account) => account.alias ?? account.provider);
  const toolbar = heard.indexOf(panel === "accounts" ? "History" : "Done");
  const quit = heard.indexOf("Quit QuotaDial");
  check(label, toolbar < 0 && "a screen reader does not read the toolbar's first button");
  if (meter.headline) {
    const line = heard.indexOf(meter.headline);
    const body = heard.indexOf(panel === "accounts" ? first : panel[0].toUpperCase() + panel.slice(1));
    check(label, !(line >= 0 && line < body) && `a screen reader does not read the line under the number before the ${panel === "accounts" ? "first account" : "panel"}`);
  }
  // Nothing of the accounts or the panel is read after the toolbar begins.
  const end = panel === "accounts" ? heard.lastIndexOf(last) : heard.lastIndexOf("Export CSV");
  check(label, panel === "accounts" && end > toolbar && `a screen reader reads "${last}", the last account, after the toolbar`);
  check(label, !(toolbar < quit) && "a screen reader does not read Quit QuotaDial after the toolbar's first button");
  if (meter.origin) {
    check(label, heard.indexOf(meter.origin) < quit && `a screen reader reads "${meter.origin}" before the toolbar`);
  }
}

/**
 * The part that scrolls is the one a wheel scrolls: a wheel over the accounts scrolls them and
 * leaves the page and the toolbar where they are, and one over the toolbar scrolls nothing.
 */
async function checkWheel(view, panel) {
  const label = `${view.label}, scrolling by wheel`;
  await view.zoom(1);
  await view.load(panel);
  const before = await view.evaluate(FRAME);
  if (!before || before.region.height - before.region.shown < 300) {
    return;
  }
  const wheel = async (x, y) => {
    await view.send("Input.dispatchMouseEvent", { type: "mouseWheel", x, y, deltaX: 0, deltaY: 300 });
    await view.scrolled();
    return view.evaluate(FRAME);
  };
  const over = await wheel(before.window.width / 2, (before.region.top + before.region.bottom) / 2);
  check(label, over.region.top <= 0.5 && "a wheel over the accounts does not scroll them");
  check(label, over.page.top !== 0 && "a wheel over the accounts scrolls the page");
  check(label, Math.abs(over.footer.top - before.footer.top) > 0.5 && "a wheel over the accounts moves the toolbar");
  const [button] = over.buttons;
  const at = over.region.top;
  const onToolbar = await wheel((button.left + button.right) / 2, (button.top + button.bottom) / 2);
  check(label, Math.abs(onToolbar.region.top - at) > 0.5 && "a wheel over the toolbar scrolls the accounts");
  check(label, onToolbar.page.top !== 0 && "a wheel over the toolbar scrolls the page");
}

/**
 * A keyboard can scroll the part with the accounts even when nothing in it is a control: with no
 * summary, button, link or field in it for Tab to reach, it is a Tab stop itself, named for a
 * screen reader, drawing the window's ring and scrolled by the keys that scroll a page. It becomes
 * one when the last control goes, though the window need not change size when that happens, so its
 * size alone does not tell the app (CI's Macs found it stale, D-136). The check lets the window draw
 * and set itself up first, as it has by the time anyone can change what it holds, then swaps each
 * disclosure for plain text of the same height (one with its summary gone draws a default summary
 * of the same height) and removes the other controls, so that only the change, and no resize, can
 * wake the part; and does the same in History by disabling every button, as an update's are while
 * it installs. Chrome makes such a part a Tab stop of its own accord, and WebKit, which draws the
 * app on macOS and Linux, does not.
 */
async function checkStop(view) {
  const label = `${view.label}, nothing else to reach`;
  await view.zoom(1);
  await view.load("accounts");
  await view.settle();
  const emptied = await view.evaluate(`(() => {
    const region = document.querySelector('main > .scroll');
    if (!region) return false;
    for (const details of region.querySelectorAll('details')) {
      const text = document.createElement('div');
      text.className = details.className;
      text.style.height = details.getBoundingClientRect().height + 'px';
      details.replaceWith(text);
    }
    region.querySelectorAll('summary, button, a, input').forEach((control) => control.remove());
    return true;
  })()`);
  if (!emptied) {
    return;
  }
  const stop = await view.eventually("document.querySelector('main > .scroll').getAttribute('tabindex') === '0'", 2000);
  const found = await view.evaluate(`(() => {
    const region = document.querySelector('main > .scroll');
    return { role: region.getAttribute('role'), name: region.getAttribute('aria-label'), hidden: region.scrollHeight - region.clientHeight };
  })()`);
  if (found.hidden <= 1) {
    return;
  }
  check(label, !stop && "the part with the accounts is not a Tab stop though nothing else in it is one, as the last control in it goes");
  check(label, (found.role !== "region" || !found.name) && `it is not a named region for a screen reader (${found.role}, "${found.name}")`);
  await view.evaluate(FOCUSABLE);
  await view.press("Tab");
  const now = await view.evaluate(FOCUSED);
  check(label, now.what !== 'div "Accounts"' && `Tab reached ${now.what}, not the part with the accounts`);
  check(label, !now.ring && "the part with the accounts does not draw the window's focus ring when Tab reaches it");
  for (const key of ["Space", "PageDown"]) {
    await view.press("Home");
    const at = await view.scrolled();
    await view.press(key);
    const next = await view.scrolled();
    check(label, next <= at && `${key} does not scroll it when it has the focus (${at}, then ${next})`);
  }
  // The same when every control in a panel is disabled, as an update's buttons are while it installs,
  // which changes no size either.
  await view.load("history");
  await view.settle();
  await view.evaluate("document.querySelectorAll('main > .scroll button').forEach((button) => { button.disabled = true; })");
  const disabled = await view.eventually("document.querySelector('main > .scroll').getAttribute('tabindex') === '0'", 2000);
  const room = await view.evaluate("(() => { const region = document.querySelector('main > .scroll'); return region.scrollHeight - region.clientHeight; })()");
  check(label, room > 1 && !disabled && "the part with History in it is not a Tab stop though every control in it is disabled");
}

/**
 * Each view opens at its top, however far the one before it was scrolled: History, opened with the
 * accounts scrolled to their middle, shows its own first line, and Done, pressed with History
 * scrolled, shows the accounts from their first.
 */
async function checkTop(view) {
  const label = `${view.label}, each view at its top`;
  await view.zoom(1);
  await view.load("accounts");
  const room = () => view.evaluate("(() => { const region = document.querySelector('main > .scroll'); return region ? region.scrollHeight - region.clientHeight : 0; })()");
  if ((await room()) < 300) {
    return;
  }
  // Where the app scrolls by its own code it asks for the scroll to be at once, whatever a stylesheet
  // says of gliding: a view that glided to its top would, for a moment, not be at it. Whether Chrome
  // glides depends on the system it runs on, and this check's Chrome does not, so the check reads what
  // the app asked for: each scroll its own code has made since the check last looked.
  const own = async (when) => {
    for (const { how, behavior } of await view.evaluate("window.__check.scrolls.splice(0)")) {
      check(label, behavior !== "instant" && `${when}, the app scrolls by ${how} with ${behavior} behaviour, which glides where a stylesheet says smooth, and not at once`);
    }
  };
  await own("as the accounts open");
  await view.evaluate(SCROLL_TO(0.5));
  await view.settle();
  await view.open("history");
  await view.settle();
  const opened = await view.scrolled();
  check(label, opened > 0.5 && `History opens ${opened} px down, not at its top`);
  await own("as History opens");
  if ((await room()) < 300) {
    return;
  }
  await view.evaluate(SCROLL_TO(0.5));
  await view.settle();
  for (let presses = 0; presses < 40; presses += 1) {
    await view.press("Tab");
    if ((await view.evaluate(FOCUSED)).what === 'button "Done"') {
      await view.press("Enter");
      break;
    }
  }
  await view.until("!!document.querySelector('main .content > section.account')", "the accounts coming back");
  await view.settle();
  const back = await view.scrolled();
  check(label, back > 0.5 && `Done shows the accounts ${back} px down, not from their first`);
  await own("as Done goes back to the accounts");
}

/**
 * In the page: how wide what the window holds is, and where the first card sits against the sides
 * of the window and its text against the toolbar's first label.
 */
const WIDTHS = `(() => {
  const region = document.querySelector('main > .scroll');
  const content = region && region.querySelector(':scope > .content');
  const card = document.querySelector('main .content .card');
  const button = document.querySelector('main > footer .toolbar > button');
  if (!region || !content || !card || !button) return null;
  const box = card.getBoundingClientRect();
  const label = button.getBoundingClientRect();
  return {
    content: content.getBoundingClientRect().width,
    left: box.left,
    right: window.innerWidth - box.right,
    text: box.left + parseFloat(getComputedStyle(card).paddingLeft),
    label: label.left + parseFloat(getComputedStyle(button).paddingLeft),
  };
})()`;

/**
 * What the window holds is as wide when its scrolling part scrolls as when it does not, and the
 * cards sit as evenly, the same distance from each side of the window, and in line with the
 * toolbar's labels, whichever it does (D-136). Where a scroll bar takes room, as it does on Windows
 * and Linux and does in this check's Chrome on every system, one that appears when the part starts to
 * scroll narrows what it holds, so the text wraps onto more lines and the height the app is told is
 * not the height of the content once the window is tall enough for the bar to go: the part keeps the
 * room for it on each side whether or not there is a bar, and the footer, which scrolls nothing,
 * keeps the same, so its labels stay in line with the cards' text.
 */
async function checkWidth(view) {
  const label = `${view.label}, as wide with and without a scroll bar`;
  const measure = async (zoom, height) => {
    await view.zoom(zoom, height);
    await view.settle();
    return view.evaluate(WIDTHS);
  };
  const scrolling = await measure(1);
  if (!scrolling) {
    check(label, "the window has no card, content or toolbar to measure");
    return;
  }
  const scrolls = await view.evaluate("(() => { const region = document.querySelector('main > .scroll'); return region.scrollHeight > region.clientHeight + 1; })()");
  const whole = await measure(1, 12000);
  // In a window under 300 px the page scrolls as one, and its scroll bar is the window's: the part
  // that scrolls keeps no room for one then, so the footer keeps none either.
  const one = await measure(3);
  await view.zoom(1);
  await view.settle();
  if (scrolls) {
    check(label, Math.abs(scrolling.content - whole.content) > 0.5 && `what the window holds is ${scrolling.content} px wide when it scrolls and ${whole.content} px when it does not`);
  }
  for (const [when, found, even] of [
    ["when the window scrolls", scrolling, true],
    ["when it does not", whole, true],
    ["when the page scrolls as one, at 300% zoom", one, false],
  ]) {
    check(label, even && Math.abs(found.left - found.right) > 1.5 && `the first card is ${found.left.toFixed(1)} px from the window's left side and ${found.right.toFixed(1)} px from its right ${when}`);
    check(label, Math.abs(found.text - found.label) > 1.5 && `the toolbar's first label is ${found.label.toFixed(1)} px from the window's left side and the card's text ${found.text.toFixed(1)} ${when}`);
  }
}

/**
 * The toolbar kept in frame at the other sizes the window is shown at: zoomed to 125% and 150%, to
 * 210%, the zoom at which the window is the shortest that keeps it, and to 220%, which a Mac's zoom
 * keys reach in a step after 200%, and 250%, which Windows' zoom reaches, where the page scrolls as
 * one; and, at full width, in windows a pixel either side of that height and in one tall enough to
 * hold everything.
 */
async function checkSizes(view) {
  for (const zoom of [1.25, 1.5, 2.1, HEIGHT / PINNED_FROM, 2.2, 2.5]) {
    await checkFrame(view, zoom);
  }
  for (const height of [PINNED_FROM - 1, PINNED_FROM, 4000]) {
    await checkFrame(view, 1, height);
  }
  await view.zoom(1);
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
 * stylesheet's text colours, the window's font and tabular figures, on a card on the theme's
 * material (D-137), with no plate behind a chart and each chart's 90% line dashed in its own colour:
 * each account over 7 days, and the first over 30.
 * On QuotaDial Free: no button, and the feature's line once. Saves each picture in `folder` when
 * one is named.
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
    // The picture writes in these and no others.
    allowed: new Set(await view.evaluate(`['--text', '--secondary', '--muted', '--warning', '--high'].map(window.__check.colour)`)),
    // Its ground is the material, the check's own text of the decision, and not the page's background.
    material: MATERIAL[theme],
    // The theme, whose dashed line and soft fill are read as the contrast check reads them.
    theme,
    // The card's fill and its hairline, as the canvas writes them, to be laid over the ground.
    card: await view.evaluate(`window.__check.colour('--card')`),
    edge: await view.evaluate(`window.__check.colour('--card-edge')`),
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
      for (const verdict of pixelVerdicts(bytes, account, look)) {
        check(label, verdict && `${what} ${verdict}`);
      }
    }
    if (folder) {
      const slug = account.label.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "").slice(0, 40);
      writeFileSync(join(folder, `${name}-${theme}-${days}-days-${number + 1}-${slug}.png`), bytes);
    }
    const drawn = await view.evaluate("window.__check.drawn");
    for (const verdict of drawingVerdicts(drawn, shareStrings(copy.history[days], account), account, copy, look)) {
      check(label, verdict && `${what} ${verdict}`);
    }
    const strings = writings(drawn);
    for (const verdict of [
      ...readingVerdicts(strings, account),
      ...listingVerdicts(strings, shareStrings(copy.history[days], account)),
      ...contrastVerdicts(drawn),
    ]) {
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
 * The strings a picture wrote, rebuilt from the canvas's record of each piece: a piece joins the
 * one before when it is in the same font, colour and line and starts where that one ended, give or
 * take the room a figure is centred in. Each string keeps its font size.
 */
function writings(drawn) {
  const strings = [];
  for (const item of drawn.filter((one) => "text" in one)) {
    const size = Number(item.font.match(/([0-9.]+)px/)?.[1] ?? 0);
    const last = strings.at(-1);
    const joins =
      last &&
      last.font === item.font &&
      last.fill === item.fill &&
      Math.abs(last.y - item.y) < 0.01 &&
      item.x >= last.end - 0.5 &&
      item.x <= last.end + size * 0.3;
    if (joins) {
      last.text += item.text;
      last.end = item.x + item.width;
    } else {
      strings.push({ text: item.text, font: item.font, fill: item.fill, y: item.y, size, end: item.x + item.width });
    }
  }
  return strings;
}

/**
 * Verdicts on what a picture says about each window, read away from the app with nothing around
 * it: "highest N%" only for a window whose highest differs from its latest share, where it adds
 * something, in the windows' order; and "90%" in small type beside every chart, so its dashed line
 * is not taken for the limit itself.
 */
function readingVerdicts(strings, account) {
  const differing = account.windows
    .filter((window) => Math.floor(window.highest) !== Math.floor(window.latest))
    .map((window) => `highest ${Math.floor(window.highest)}%`);
  const highest = strings.filter((one) => one.text.startsWith("highest ")).map((one) => one.text);
  const limits = strings.filter((one) => one.text === "90%");
  return [
    highest.join() !== differing.join() &&
      `writes ${JSON.stringify(highest)} where only ${JSON.stringify(differing)} add something`,
    limits.length !== account.windows.length &&
      `labels the 90% line ${limits.length} times for ${account.windows.length} charts`,
    limits.some((one) => one.size > 18) && "labels the 90% line in large type",
  ];
}

/**
 * Verdicts holding what a picture wrote, string by string, to what `shareStrings` lists: each
 * written string is a listed one, or the start of one cut short with "…", and each listed string
 * is written, so the list is the whole of the picture's words and nothing in it goes unseen. A
 * label, a title or a date may be cut short when it has too little room; a share may not (the
 * manager on #73, 2026-10-27): "highest 1…" is a share broken, and is two verdicts, that the
 * picture writes what is not listed and never writes what is.
 */
function listingVerdicts(strings, listed) {
  const verdicts = [];
  const isStart = (written, whole) =>
    written === whole || (!isShare(whole) && written.endsWith("…") && whole.startsWith(written.slice(0, -1)));
  for (const written of strings) {
    verdicts.push(
      !listed.some((whole) => isStart(written.text, whole)) && `writes "${written.text}", which shareStrings does not list`,
    );
  }
  for (const whole of listed) {
    verdicts.push(!strings.some((written) => isStart(written.text, whole)) && `lists "${whole}" but never writes it`);
  }
  return verdicts;
}

/** `colour` as the canvas wrote it, `#rrggbb` or `rgba(r, g, b, a)`, laid over the opaque `under`. */
function rgb(colour, under = [255, 255, 255]) {
  const hex = colour.match(/^#([0-9a-f]{6})$/i);
  if (hex) {
    const number = Number.parseInt(hex[1], 16);
    return [(number >> 16) & 255, (number >> 8) & 255, number & 255];
  }
  const parts = colour.match(/^rgba?\(([^)]+)\)$/)?.[1].split(",").map(Number) ?? [0, 0, 0, 1];
  const alpha = parts[3] ?? 1;
  return [0, 1, 2].map((index) => parts[index] * alpha + under[index] * (1 - alpha));
}

/** The contrast of two colours, as WCAG works it out (check-contrast.mjs does the same). */
function contrast(a, b) {
  const luminance = (colour) => {
    const [r, g, b] = colour.map((channel) => {
      const value = channel / 255;
      return value <= 0.039_28 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
    });
    return 0.2126 * r + 0.7152 * g + 0.0722 * b;
  };
  const [light, dark] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (light + 0.05) / (dark + 0.05);
}

/**
 * Verdicts on the contrast of every piece of text a picture wrote against what the canvas held
 * behind it just before: 4.5:1, as check-contrast.mjs holds the window's text, whatever the theme.
 */
function contrastVerdicts(drawn) {
  const said = new Set();
  return drawn
    .filter((item) => "text" in item && item.text.trim())
    .map((item) => {
      const lowest = Math.min(
        ...item.ground.map(([r, g, b, a]) => {
          const behind = rgb(`rgba(${r}, ${g}, ${b}, ${a / 255})`);
          return contrast(rgb(item.fill, behind), behind);
        }),
      );
      const problem = lowest < 4.5 && `writes "${item.text}" at ${lowest.toFixed(2)}:1 against what is behind it`;
      if (!problem || said.has(problem)) {
        return false;
      }
      said.add(problem);
      return problem;
    });
}

/**
 * Verdicts on the picture's own pixels, as the person who posts it gets them (D-137): the ground is
 * the theme's material at the corners and in the margin, the card sits on it with its fill and its
 * hairline edge, no plate is drawn behind a chart, and each chart's 90% line is dashed 6/6 in
 * `--chart-limit` over the card and the soft fill. The card's fill and edge are the stylesheet's,
 * laid over the material as the canvas lays them; the dashed line's colour and the soft fill are
 * the stylesheet's and SHADING, read as the contrast check reads them (`dashColours`); the card and
 * the charts are where `src/share.ts` puts them. A PNG that cannot be read is a verdict too. Adds
 * how much of each chart's dashed line was read, and left out, to `dashes`.
 */
function pixelVerdicts(bytes, account, look) {
  let picture;
  try {
    picture = decodePng(bytes);
  } catch (error) {
    return [`has pixels that cannot be read: ${error.message}`];
  }
  const ground = fromHex(look.material);
  const fill = rgb(look.card, ground);
  const { limit, shading } = dashColours(look.theme, fill);
  const want = {
    scale: SHARED.width / PICTURE_WIDTH,
    card: CARD,
    ground,
    fill,
    edge: rgb(look.edge, fill),
    limit,
    shading,
    charts: chartBoxes(account.windows.length).map((box, index) => ({
      label: account.windows[index].label,
      box,
      points: account.windows[index].points,
    })),
  };
  for (const { box, points } of want.charts) {
    const { places, all, covered } = dashPlaces(box, points, want.scale);
    dashes.all += all;
    dashes.judged += places.length;
    dashes.overFill += places.filter((place) => place.over === "fill").length;
    dashes.covered += covered.length;
  }
  return pictureVerdicts(picture, want);
}

/**
 * A verdict on each thing a picture drew, from the canvas's record of it: false where it passed,
 * or what is wrong (each kind of problem said once). Nothing may be written that `listed` does not
 * hold, or a cut-short start of it; nothing with an @; the name given the account must be in its
 * label; writing only in the stylesheet's text colours and the window's font, figures one by one
 * in equal room; and the ground the theme's material, the picture's own choice (D-137), in the
 * picture's whole size, painted first.
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
      !(ground && "rect" in ground && ground.fill === look.material && ground.rect.join() === `0,0,${PICTURE_WIDTH},${PICTURE_HEIGHT}`) &&
        `does not begin with the material, ${look.material}, over its whole size`,
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

/**
 * In the page: what scrolls when a person scrolls the window: the scrolling part between the
 * summary and the toolbar while it scrolls, and the whole page when it does not (at 200% zoom, in a
 * short window, or in a window made before the toolbar stayed put).
 */
const SCROLLER = `(() => {
  const region = document.querySelector('main > .scroll');
  const scrolls = !!region && getComputedStyle(region).overflowY !== 'visible' && region.scrollHeight > region.clientHeight;
  return scrolls ? region : document.scrollingElement;
})()`;

/**
 * In the page: the height of the whole of what the window holds, in CSS pixels, which the window's
 * own height caps once what it holds is taller than it may be: the popover's own height and what its
 * scrolling part holds beyond its bottom edge.
 */
const WHOLE = `(() => {
  const main = document.querySelector('main');
  const region = main.querySelector(':scope > .scroll');
  const beyond = region && getComputedStyle(region).overflowY !== 'visible' ? region.scrollHeight - region.clientHeight : 0;
  return Math.ceil(main.getBoundingClientRect().height + beyond);
})()`;

/**
 * A picture of the whole of the window's content, as the website and the README show it: the
 * window as tall as what it holds, whatever the most the app lets it be.
 */
async function screenshot(view, panel, folder) {
  await view.load(panel);
  await view.settle();
  // Room to spare, so that the window is never the shorter of the two.
  const needed = (await view.evaluate(WHOLE)) + 200;
  await view.send("Emulation.setDeviceMetricsOverride", {
    width: WIDTH,
    height: Math.max(HEIGHT, needed),
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

/**
 * Pictures of the window as a person sees it, zoomed in as far as `zoom` says: no taller than the
 * most the app lets it be, with whatever scrolls at its top and half way down its length, `-top`
 * and `-half` after the view's name. A window that does not scroll is the same in both.
 */
async function windowPictures(view, panel, folder, zoom) {
  await view.load(panel);
  // The window at this zoom in whole pixels, as the checks show it (viewport.mjs), with as many
  // pixels as the window has at 2x, whatever the zoom.
  const viewport = viewportAt(zoom, POPOVER);
  await view.send("Emulation.setDeviceMetricsOverride", {
    width: viewport.width,
    height: viewport.height,
    deviceScaleFactor: 2 * zoom,
    mobile: false,
  });
  await view.evaluate("document.activeElement.blur()");
  for (const [place, along] of [["top", 0], ["half", 0.5]]) {
    await view.evaluate(`(() => {
      const scroller = ${SCROLLER};
      scroller.scrollTop = Math.round((scroller.scrollHeight - scroller.clientHeight) * ${along});
    })()`);
    await view.settle();
    const height = await view.evaluate("Math.ceil(document.querySelector('main').getBoundingClientRect().height)");
    // The clip is in the page's own coordinates, so a page that is scrolled is clipped from where it is.
    const scrolled = await view.evaluate("document.scrollingElement.scrollTop");
    const { data } = await view.send("Page.captureScreenshot", {
      format: "png",
      clip: { x: 0, y: scrolled, width: viewport.width, height: Math.min(viewport.height, Math.max(120 / zoom, height)), scale: 1 },
    });
    const zoomed = zoom === 1 ? "" : `-zoom${Math.round(zoom * 100)}`;
    writeFileSync(join(folder, `${view.label.replaceAll(", ", "-")}${zoomed}-${place}.png`), Buffer.from(data, "base64"));
  }
  await view.zoom(1);
}

async function main() {
  const at = process.argv.indexOf("--screenshots");
  const pictures = at > 0 ? resolve(process.argv[at + 1] ?? "") : null;
  const sharing = process.argv.indexOf("--share-images");
  const shares = sharing > 0 ? resolve(process.argv[sharing + 1] ?? "") : null;
  const framing = process.argv.indexOf("--window-pictures");
  const windows = framing > 0 ? resolve(process.argv[framing + 1] ?? "") : null;
  const zooming = process.argv.indexOf("--window-zoom");
  // A zoom that is not one is refused here, before a folder is made or Chrome is asked (viewport.mjs).
  const windowZoom = zoomOf(zooming > 0 ? (process.argv[zooming + 1] ?? "") : undefined);
  const only = process.argv.includes("--only") ? process.argv[process.argv.indexOf("--only") + 1] : "";
  const barsAt = process.argv.indexOf("--scroll-bars");
  const classic = wantsClassicScrollbars(barsAt > 0 ? (process.argv[barsAt + 1] ?? "") : undefined, Boolean(pictures || windows));
  for (const folder of [pictures, shares, windows]) {
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
  const { browser, address } = launch(chrome, profile, { classicScrollbars: classic });
  let tools;
  try {
    tools = await DevTools.connect(await address);
    // Which browser, and which scroll bars it draws, so that a run on another machine can say what it was.
    const { product } = await tools.send("Browser.getVersion");
    console.log(`Window check in ${product}, ${classic ? "with scroll bars that take room" : "with the system's own scroll bars"}.`);
    let views = 0;
    for (const [name, copy] of Object.entries(COPIES)) {
      for (const theme of THEMES) {
        for (const panel of copy.panels ?? PANELS) {
          const label = `${name}, ${theme}, ${panel}`;
          if (!label.includes(only ?? "")) {
            continue;
          }
          const view = await View.open(tools, label, url, copy, theme);
          try {
            // Pressing each control is the same in either theme, so once is enough, and a copy
            // made to scroll has its controls pressed on the others.
            await checkView(view, panel, theme === "light" && !copy.forScrolling, copy, theme === "light");
            // The picture to share follows the theme, so it is checked in both.
            if (panel === "history" && !copy.forScrolling) {
              await checkSharing(view, name, copy, theme, shares);
            }
            if (pictures) {
              await screenshot(view, panel, pictures);
            }
            if (windows) {
              await windowPictures(view, panel, windows, windowZoom);
            }
          } finally {
            await view.close();
          }
          views += 1;
        }
      }
    }
    if (dashes.all > 0) {
      // So that a run shows the dashed lines were judged, and not passed by being left out.
      console.log(
        `The pictures' dashed 90% lines cover ${dashes.all} pixels whole: ${dashes.judged} were judged, ` +
          `${dashes.overFill} of them over the soft fill under a chart's line, ${dashes.covered} left out where a chart's own ` +
          `line covers its 90% line, and the rest within a pixel of a dash's end.`,
      );
    }
    if (failures.length > 0) {
      console.error(failures.join("\n"));
      console.error(`${failures.length} of ${checks} window checks failed.`);
      process.exitCode = 1;
    } else {
      console.log(
        `The window passed ${checks} checks over ${views} views: Tab order, focus, names and roles, ` +
          `keys, zoom to 200%, long names, shares and caption words kept whole, reduced motion, the toolbar kept at its foot, the same width with a scroll bar or ` +
          `without and the picture to share.` +
          (pictures ? ` Pictures are in ${pictures}.` : "") +
          (windows ? ` Pictures of the window as it is shown are in ${windows}.` : "") +
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
