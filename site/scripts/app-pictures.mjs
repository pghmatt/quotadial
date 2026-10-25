// Captures the real QuotaDial window for the site's pictures (public/images/app/): the app's own
// window check (apps/tray/scripts/check-window.mjs) answering from the website's example accounts
// (apps/tray/scripts/site-fixtures.mjs), in light and dark. Build the window first:
// `cd ../apps/tray && npm ci && npm run build`.
import { execFileSync } from "node:child_process";
import { copyFileSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const TRAY = fileURLToPath(new URL("../../apps/tray/", import.meta.url));
const OUT = fileURLToPath(new URL("../public/images/app/", import.meta.url));
const shots = mkdtempSync(join(tmpdir(), "quotadial-app-pictures-"));
try {
  execFileSync(
    "node",
    ["scripts/check-window.mjs", "--fixtures", "./scripts/site-fixtures.mjs", "--screenshots", shots],
    { cwd: TRAY, stdio: "inherit" },
  );
  for (const view of ["accounts", "settings", "history"]) {
    for (const theme of ["light", "dark"]) {
      copyFileSync(join(shots, `site-${theme}-${view}.png`), join(OUT, `${view}-${theme}.png`));
    }
  }
  console.log("Wrote the accounts, settings and history pictures, light and dark, to public/images/app/.");
} finally {
  rmSync(shots, { recursive: true, force: true });
}
