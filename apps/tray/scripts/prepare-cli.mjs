// Builds the `quotadial` CLI and puts it where Tauri bundles it beside the app's own binary
// (D-067): src-tauri/binaries/quotadial-<target triple>, with `.exe` on Windows. Tauri strips the
// triple when it bundles, so the app carries it as plain `quotadial`.
//
//   node scripts/prepare-cli.mjs            a debug build, for `tauri dev` and the checks
//   node scripts/prepare-cli.mjs --release  a release build, for `tauri build`
import { execFileSync } from "node:child_process";
import { copyFileSync, mkdirSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const release = process.argv.includes("--release");
const root = fileURLToPath(new URL("../../..", import.meta.url));
const binaries = fileURLToPath(new URL("../src-tauri/binaries", import.meta.url));

const host = execFileSync("rustc", ["-vV"], { encoding: "utf8" })
  .split("\n")
  .find((line) => line.startsWith("host: "))
  ?.slice("host: ".length)
  .trim();
// `tauri build --target <triple>` says which triple it is building for.
const triple = process.env.TAURI_ENV_TARGET_TRIPLE || host;
if (!triple) {
  console.error("Could not tell which target to build the CLI for: `rustc -vV` names no host.");
  process.exit(1);
}
const crossing = triple !== host;

const cargo = ["build", "--locked", "--package", "quotadial", "--bin", "quotadial"];
if (release) cargo.push("--release");
if (crossing) cargo.push("--target", triple);
execFileSync("cargo", cargo, { cwd: root, stdio: "inherit" });

const extension = triple.includes("windows") ? ".exe" : "";
const built = join(
  root,
  "target",
  ...(crossing ? [triple] : []),
  release ? "release" : "debug",
  `quotadial${extension}`,
);
mkdirSync(binaries, { recursive: true });
copyFileSync(built, join(binaries, `quotadial-${triple}${extension}`));
console.log(`Bundled the CLI as binaries/quotadial-${triple}${extension}.`);
