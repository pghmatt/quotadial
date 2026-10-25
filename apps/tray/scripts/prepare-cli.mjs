// Builds the `quotadial` CLI and puts it where Tauri bundles it beside the app's own binary
// (D-069): src-tauri/binaries/quotadial-<target triple>, with `.exe` on Windows. Tauri strips the
// triple when it bundles, so the app carries it as plain `quotadial`.
//
// A universal Mac app (`tauri build --target universal-apple-darwin`) needs three files: Tauri
// compiles the app once per architecture and checks for that architecture's CLI each time, then
// bundles one CLI holding both, made here with `lipo`.
//
//   node scripts/prepare-cli.mjs            a debug build, for `tauri dev` and the checks
//   node scripts/prepare-cli.mjs --release  a release build, for `tauri build`
//   node scripts/prepare-cli.mjs --fake     the example build's CLI (D-117), with the fake provider,
//                                           which only tauri.example.conf.json asks for
import { execFileSync } from "node:child_process";
import { copyFileSync, mkdirSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const UNIVERSAL = "universal-apple-darwin";
const release = process.argv.includes("--release");
const fake = process.argv.includes("--fake");
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

/** Builds the CLI for `target` and copies it into `binaries`, returning where it went. */
function build(target) {
  const crossing = target !== host;
  const cargo = ["build", "--locked", "--package", "quotadial", "--bin", "quotadial"];
  if (release) cargo.push("--release");
  if (fake) cargo.push("--features", "fake");
  if (crossing) cargo.push("--target", target);
  execFileSync("cargo", cargo, { cwd: root, stdio: "inherit" });
  const extension = target.includes("windows") ? ".exe" : "";
  const built = join(
    root,
    "target",
    ...(crossing ? [target] : []),
    release ? "release" : "debug",
    `quotadial${extension}`,
  );
  const bundled = join(binaries, `quotadial-${target}${extension}`);
  copyFileSync(built, bundled);
  console.log(`Bundled the ${fake ? "example build's " : ""}CLI as binaries/quotadial-${target}${extension}.`);
  return bundled;
}

mkdirSync(binaries, { recursive: true });
if (triple === UNIVERSAL) {
  const parts = ["aarch64-apple-darwin", "x86_64-apple-darwin"].map(build);
  execFileSync("lipo", ["-create", "-output", join(binaries, `quotadial-${UNIVERSAL}`), ...parts], {
    stdio: "inherit",
  });
  console.log(`Bundled the CLI for both Mac architectures as binaries/quotadial-${UNIVERSAL}.`);
} else {
  build(triple);
}
