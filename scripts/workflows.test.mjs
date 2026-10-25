// Runs workflow steps' own shell on this machine, as GitHub runs it, with CI's other script tests:
// the Release workflow's step that writes latest.json, the copies without the version and the
// checksums, on a release laid out as its publish job downloads it. Nothing here uses the network.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  copyFileSync,
  cpSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  realpathSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { basename, dirname, join } from "node:path";
import { after, test } from "node:test";
import { fileURLToPath } from "node:url";

import { NAMES, latestNames } from "./latest-names.mjs";

const ROOT = fileURLToPath(new URL("..", import.meta.url));

const made = [];
after(() => made.forEach((path) => rmSync(path, { recursive: true, force: true })));

/** An empty folder, by its real path, removed when the tests end. */
function folder() {
  const path = realpathSync(mkdtempSync(join(tmpdir(), "quotadial-workflow-")));
  made.push(path);
  return path;
}

/**
 * The shell of the step named `step`, or with that id, in `workflow`: its `run: |` block without the
 * indent. GitHub fills `${{ }}` expressions before running it, so a step tested here must have none.
 */
function shell(workflow, step) {
  const lines = readFileSync(join(ROOT, ".github/workflows", workflow), "utf8").split("\n");
  const indent = (line) => line.search(/\S/);
  const start = lines.findIndex((line) => [`- name: ${step}`, `- id: ${step}`].includes(line.trim()));
  assert.notEqual(start, -1, `${workflow} has no step ${step}`);
  let end = start + 1;
  while (end < lines.length && (lines[end].trim() === "" || indent(lines[end]) > indent(lines[start]))) {
    end += 1;
  }
  const run = lines.slice(start, end).findIndex((line) => /^\s+run: \|$/.test(line));
  assert.notEqual(run, -1, `${step} in ${workflow} runs no shell`);
  const block = lines.slice(start + run + 1, end);
  const margin = Math.min(...block.filter((line) => line.trim() !== "").map(indent));
  const script = `${block.map((line) => line.slice(margin)).join("\n").trimEnd()}\n`;
  assert.doesNotMatch(script, /\$\{\{/, `${step} has an expression only GitHub can fill`);
  return script;
}

/** Runs `script` with `bash -e`, as GitHub runs a step that names no shell, in `cwd`. */
function bash(script, cwd, env) {
  const file = join(folder(), "step.sh");
  writeFileSync(file, script);
  return spawnSync("bash", ["-e", file], { cwd, env: { ...process.env, ...env }, encoding: "utf8" });
}

const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");

/** A signature as `tauri signer` writes it, naming its file and `version`. */
function signature(file, version) {
  const comment = `timestamp:1\tfile:${file}\tversion:${version}`;
  const text = `untrusted comment: signature from tauri secret key\nRUQ=\ntrusted comment: ${comment}\nZmFrZQ==\n`;
  return Buffer.from(text).toString("base64");
}

/** How a person checks downloads (docs/INSTALL.md): `sha256sum -c`, or `shasum -a 256 -c` on macOS. */
function checks(cwd, ...args) {
  const [command, ...flags] = process.platform === "darwin" ? ["shasum", "-a", "256", "-c"] : ["sha256sum", "-c"];
  return spawnSync(command, [...flags, "SHA256SUMS.txt", ...args], { cwd, encoding: "utf8" });
}

const ok = (name) => new RegExp(`^${name.replaceAll(".", "\\.")}: OK$`, "m");

test("the release's SHA256SUMS.txt lists both names of each download, and latest.json the versioned one", () => {
  // The step runs in a checkout, with the release in dist/ as the download step leaves it: the
  // command's archives and the msi at the top, and the dmg, the deb, the AppImage and the update for
  // macOS in their bundle's folder, the update still named as `tauri build` names it. The scripts
  // are copied, not linked, since updater-manifest.mjs run through a link does nothing.
  const work = folder();
  cpSync(join(ROOT, "scripts"), join(work, "scripts"), { recursive: true });
  copyFileSync(join(ROOT, "CHANGELOG.md"), join(work, "CHANGELOG.md"));
  const signed = [
    "macos/QuotaDial.app.tar.gz",
    "QuotaDial_0.2.0_x64_en-US.msi",
    "deb/QuotaDial_0.2.0_amd64.deb",
    "appimage/QuotaDial_0.2.0_amd64.AppImage",
  ];
  const unsigned = [
    "dmg/QuotaDial_0.2.0_universal.dmg",
    "quotadial-v0.2.0-aarch64-apple-darwin.tar.gz",
    "quotadial-v0.2.0-x86_64-apple-darwin.tar.gz",
    "quotadial-v0.2.0-x86_64-unknown-linux-gnu.tar.gz",
    "quotadial-v0.2.0-x86_64-pc-windows-msvc.zip",
    "quotadial-v0.2.0-cli.cdx.json",
    "QuotaDial_0.2.0_app.cdx.json",
  ];
  for (const path of [...signed, ...unsigned]) {
    mkdirSync(join(work, "dist", dirname(path)), { recursive: true });
    writeFileSync(join(work, "dist", path), `the bytes of ${path}`);
  }
  for (const path of signed) {
    writeFileSync(join(work, "dist", `${path}.sig`), signature(basename(path), "0.2.0"));
  }

  const step = "The updater's latest.json, the copies without the version and every file's checksum";
  const ran = bash(shell("release.yml", step), work, { GITHUB_REF_NAME: "v0.2.0", GITHUB_REPOSITORY: "pghmatt/quotadial" });
  assert.equal(ran.status, 0, ran.stderr);
  assert.match(ran.stdout, /Wrote dist\/latest\.json for 0\.2\.0/);

  const released = new Map(
    readdirSync(join(work, "dist"), { recursive: true, withFileTypes: true })
      .filter((entry) => entry.isFile())
      .map((entry) => [entry.name, join(entry.parentPath, entry.name)]),
  );
  const sums = new Map(
    readFileSync(released.get("SHA256SUMS.txt"), "utf8")
      .trimEnd()
      .split("\n")
      .map((line) => line.split("  ").reverse()),
  );
  assert.deepEqual([...sums.keys()].sort(), [...released.keys()].filter((name) => name !== "SHA256SUMS.txt").sort());
  for (const [file, name] of latestNames("0.2.0")) {
    assert.ok(released.has(name), `the release has no ${name}`);
    assert.equal(dirname(released.get(name)), dirname(released.get(file)), `${name} is not beside ${file}`);
    const hash = sha256(readFileSync(released.get(file)));
    assert.equal(sums.get(file), hash, file);
    assert.equal(sums.get(name), hash, `${name} is not listed with ${file}'s SHA-256`);
  }

  const { platforms } = JSON.parse(readFileSync(released.get("latest.json"), "utf8"));
  const urls = Object.values(platforms).map(({ url }) => url);
  assert.deepEqual([...new Set(urls.map((url) => url.split("/").pop()))].sort(), [
    "QuotaDial_0.2.0_amd64.AppImage",
    "QuotaDial_0.2.0_amd64.deb",
    "QuotaDial_0.2.0_universal.app.tar.gz",
    "QuotaDial_0.2.0_x64_en-US.msi",
  ]);
  for (const name of NAMES) {
    assert.ok(!urls.some((url) => url.endsWith(`/${name}`)), `latest.json names ${name}`);
  }

  // Every file the release carries in one folder, as `gh release download` leaves them: the check
  // passes for both names of each download.
  const downloads = folder();
  for (const [name, path] of released) {
    copyFileSync(path, join(downloads, name));
  }
  const all = checks(downloads);
  assert.equal(all.status, 0, all.stdout + all.stderr);
  for (const [file, name] of latestNames("0.2.0")) {
    assert.match(all.stdout, ok(file));
    assert.match(all.stdout, ok(name));
  }

  // Only the copy downloaded, and checked as docs/INSTALL.md says.
  const one = folder();
  copyFileSync(released.get("SHA256SUMS.txt"), join(one, "SHA256SUMS.txt"));
  copyFileSync(released.get("QuotaDial_universal.dmg"), join(one, "QuotaDial_universal.dmg"));
  const copy = checks(one, "--ignore-missing");
  assert.equal(copy.status, 0, copy.stdout + copy.stderr);
  assert.equal(copy.stdout.trim(), "QuotaDial_universal.dmg: OK");
});
