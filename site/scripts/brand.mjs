// Renders QuotaDial's brand files for the site from docs/brand/render/ in headless Chrome, with the
// site's own fonts (codus-brand-furniture's recipe):
//
//   app/favicon.ico           16, 32 and 48 px, each an RGBA PNG
//   app/icon.png              512 px
//   app/apple-icon.png        180 px, the tile filling the square (iOS rounds it)
//   app/opengraph-image.png   1200x630, the share card
//   docs/brand/mark-sheet.png the mark at 16 to 256 px on both grounds, for review by eye
//
// The render pages are private: they live in docs/, and nothing here is served by the site.
// `node scripts/brand.mjs` after the mark or the card changes; CHROME names Chrome if it is not
// where it usually is.
import { spawn } from "node:child_process";
import { existsSync, mkdtempSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { createServer } from "node:http";
import { tmpdir } from "node:os";
import { extname, join, normalize } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = fileURLToPath(new URL("../", import.meta.url));
const TYPES = {
  ".html": "text/html",
  ".css": "text/css",
  ".svg": "image/svg+xml",
  ".woff2": "font/woff2",
  ".png": "image/png",
};

function chrome() {
  const candidates = [
    process.env.CHROME,
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    "/Applications/Chromium.app/Contents/MacOS/Chromium",
    "/usr/bin/google-chrome",
    "/usr/bin/chromium",
  ].filter(Boolean);
  const found = candidates.find((candidate) => existsSync(candidate));
  if (!found) {
    throw new Error("Chrome was not found; set CHROME to its path.");
  }
  return found;
}

/** Serves the site's folder on 127.0.0.1, so the render pages can load the fonts. */
function serve() {
  const server = createServer((request, response) => {
    const path = decodeURIComponent(new URL(request.url ?? "/", "http://127.0.0.1").pathname);
    const file = normalize(join(ROOT, path));
    if (!file.startsWith(ROOT) || !existsSync(file) || !statSync(file).isFile()) {
      response.writeHead(404).end();
      return;
    }
    response.writeHead(200, { "content-type": TYPES[extname(file)] ?? "application/octet-stream" }).end(readFileSync(file));
  });
  return new Promise((done) => server.listen(0, "127.0.0.1", () => done(server)));
}

class DevTools {
  constructor(socket) {
    this.socket = socket;
    this.next = 1;
    this.pending = new Map();
    socket.addEventListener("message", (message) => {
      const data = JSON.parse(String(message.data));
      const pending = this.pending.get(data.id);
      if (pending) {
        this.pending.delete(data.id);
        data.error ? pending.fail(new Error(data.error.message)) : pending.done(data.result);
      }
    });
  }

  send(method, params = {}, sessionId) {
    const id = this.next++;
    this.socket.send(JSON.stringify({ id, method, params, sessionId }));
    return new Promise((done, fail) => this.pending.set(id, { done, fail }));
  }
}

async function launch() {
  const profile = mkdtempSync(join(tmpdir(), "quotadial-brand-"));
  const args = ["--headless=new", "--remote-debugging-port=0", `--user-data-dir=${profile}`, "--no-first-run", "--hide-scrollbars", "--force-color-profile=srgb", "about:blank"];
  if (process.platform === "linux") {
    args.push("--no-sandbox");
  }
  const browser = spawn(chrome(), args, { stdio: ["ignore", "ignore", "pipe"] });
  const address = await new Promise((done, fail) => {
    let said = "";
    browser.stderr.on("data", (chunk) => {
      said += chunk;
      const found = said.match(/DevTools listening on (ws:\/\/\S+)/);
      if (found) {
        done(found[1]);
      }
    });
    browser.on("exit", (code) => fail(new Error(`Chrome exited (${code})`)));
  });
  const socket = new WebSocket(address);
  await new Promise((done) => socket.addEventListener("open", done, { once: true }));
  const tools = new DevTools(socket);
  const close = async () => {
    await tools.send("Browser.close").catch(() => {});
    await new Promise((done) => (browser.exitCode === null ? browser.once("exit", done) : done()));
    rmSync(profile, { recursive: true, force: true, maxRetries: 5 });
  };
  return { tools, close };
}

/** A PNG of `url` at `width` x `height`, with transparency where the page has none of its own. */
async function render(tools, url, width, height) {
  const { targetId } = await tools.send("Target.createTarget", { url: "about:blank" });
  const { sessionId } = await tools.send("Target.attachToTarget", { targetId, flatten: true });
  const send = (method, params) => tools.send(method, params, sessionId);
  await send("Page.enable");
  await send("Emulation.setDeviceMetricsOverride", { width, height, deviceScaleFactor: 1, mobile: false });
  await send("Emulation.setDefaultBackgroundColorOverride", { color: { r: 0, g: 0, b: 0, a: 0 } });
  await send("Page.navigate", { url });
  for (let tries = 0; tries < 100; tries += 1) {
    const { result } = await send("Runtime.evaluate", { expression: "document.body?.dataset.ready === '1'", returnByValue: true });
    if (result.value) {
      break;
    }
    await new Promise((done) => setTimeout(done, 50));
  }
  const { data } = await send("Page.captureScreenshot", {
    format: "png",
    clip: { x: 0, y: 0, width, height, scale: 1 },
    captureBeyondViewport: true,
  });
  // Chrome may write an opaque PNG; drawing it through a canvas keeps every channel, alpha included.
  const { result } = await send("Runtime.evaluate", {
    expression: `(async () => {
      const bytes = Uint8Array.from(atob(${JSON.stringify(data)}), (c) => c.charCodeAt(0));
      const bitmap = await createImageBitmap(new Blob([bytes], { type: "image/png" }));
      const canvas = new OffscreenCanvas(bitmap.width, bitmap.height);
      canvas.getContext("2d", { alpha: true }).drawImage(bitmap, 0, 0);
      const out = new Uint8Array(await (await canvas.convertToBlob({ type: "image/png" })).arrayBuffer());
      let binary = "";
      for (let i = 0; i < out.length; i += 8192) binary += String.fromCharCode(...out.subarray(i, i + 8192));
      return btoa(binary);
    })()`,
    awaitPromise: true,
    returnByValue: true,
  });
  await tools.send("Target.closeTarget", { targetId });
  const png = Buffer.from(result.value, "base64");
  if (png[25] !== 6 || png.readUInt32BE(16) !== width || png.readUInt32BE(20) !== height) {
    throw new Error(`${url}: expected a ${width}x${height} RGBA PNG`);
  }
  return png;
}

/** An ICO holding `pngs`, each square and RGBA. */
function ico(pngs) {
  const header = Buffer.alloc(6 + pngs.length * 16);
  header.writeUInt16LE(1, 2);
  header.writeUInt16LE(pngs.length, 4);
  let offset = header.length;
  pngs.forEach((png, index) => {
    const size = png.readUInt32BE(16);
    const at = 6 + index * 16;
    header[at] = header[at + 1] = size;
    header.writeUInt16LE(1, at + 4);
    header.writeUInt16LE(32, at + 6);
    header.writeUInt32LE(png.length, at + 8);
    header.writeUInt32LE(offset, at + 12);
    offset += png.length;
  });
  return Buffer.concat([header, ...pngs]);
}

const server = await serve();
const base = `http://127.0.0.1:${server.address().port}/docs/brand/render`;
const { tools, close } = await launch();
try {
  const marks = [];
  for (const size of [16, 32, 48]) {
    marks.push(await render(tools, `${base}/mark.html?size=${size}`, size, size));
  }
  writeFileSync(join(ROOT, "app/favicon.ico"), ico(marks));
  writeFileSync(join(ROOT, "app/icon.png"), await render(tools, `${base}/mark.html?size=512`, 512, 512));
  writeFileSync(join(ROOT, "app/apple-icon.png"), await render(tools, `${base}/mark.html?size=180&bleed=1`, 180, 180));
  writeFileSync(join(ROOT, "app/opengraph-image.png"), await render(tools, `${base}/share.html`, 1200, 630));
  writeFileSync(join(ROOT, "docs/brand/mark-sheet.png"), await render(tools, `${base}/sheet.html`, 960, 720));
  console.log("Wrote app/favicon.ico (16, 32, 48), app/icon.png, app/apple-icon.png, app/opengraph-image.png and docs/brand/mark-sheet.png.");
} finally {
  await close();
  server.close();
}
