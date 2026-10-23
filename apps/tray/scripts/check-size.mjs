// The window's JavaScript stays under 150 KB (the manager's budget for M3, D-065), so the app
// opens fast and carries no UI library.
import { readdirSync, statSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const LIMIT_KB = 150;
const assets = fileURLToPath(new URL("../dist/assets", import.meta.url));
const bytes = readdirSync(assets)
  .filter((name) => name.endsWith(".js"))
  .reduce((total, name) => total + statSync(join(assets, name)).size, 0);
const kb = (bytes / 1024).toFixed(1);
if (bytes > LIMIT_KB * 1024) {
  console.error(`The window's JavaScript is ${kb} KB, over the ${LIMIT_KB} KB budget.`);
  process.exit(1);
}
console.log(`The window's JavaScript is ${kb} KB, within the ${LIMIT_KB} KB budget.`);
