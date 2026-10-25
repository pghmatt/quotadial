// Tests for updater-manifest.mjs, run by CI with `node --test scripts/*.test.mjs`. The signatures
// are shaped as `tauri signer` writes them; what they sign is not checked here, since the app's
// updater checks that against its public key.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { NAMES, copyLatest } from "./latest-names.mjs";
import { manifest, signedVersion } from "./updater-manifest.mjs";

const NOW = new Date("2026-10-25T12:34:56.789Z");

/** A signature as `tauri signer` writes it, base64 encoded, naming `version`. */
function signature(file, version) {
  const comment = version === undefined ? `timestamp:1\tfile:${file}` : `timestamp:1\tfile:${file}\tversion:${version}`;
  const text = `untrusted comment: signature from tauri secret key\nRUQ=\ntrusted comment: ${comment}\nZmFrZQ==\n`;
  return Buffer.from(text).toString("base64");
}

/** A release folder laid out as the workflow downloads it, with `files` signed for `version`. */
function release(files, version = "0.3.0") {
  const folder = mkdtempSync(join(tmpdir(), "quotadial-manifest-"));
  for (const path of files) {
    const full = join(folder, path);
    mkdirSync(join(full, ".."), { recursive: true });
    writeFileSync(full, "bundle");
    const name = path.split("/").pop();
    writeFileSync(`${full}.sig`, signature(name, version));
  }
  return folder;
}

const EVERY = [
  "QuotaDial_0.3.0_universal.app.tar.gz",
  "QuotaDial_0.3.0_x64_en-US.msi",
  "appimage/QuotaDial_0.3.0_amd64.AppImage",
  "deb/QuotaDial_0.3.0_amd64.deb",
];

const options = (folder, version = "0.3.0") => ({
  folder,
  version,
  notes: "### Added\n\n- Updates.",
  repository: "pghmatt/quotadial",
  now: NOW,
});

test("each kind of install gets its own file, and older updaters the generic one", () => {
  const folder = release(EVERY);
  writeFileSync(join(folder, "QuotaDial_0.3.0_universal.dmg"), "not an update");
  writeFileSync(join(folder, "SHA256SUMS.txt"), "sums");

  const written = manifest(options(folder));

  assert.equal(written.version, "0.3.0");
  assert.equal(written.notes, "### Added\n\n- Updates.");
  assert.equal(written.pub_date, "2026-10-25T12:34:56Z");
  const base = "https://github.com/pghmatt/quotadial/releases/download/v0.3.0/";
  const url = (target) => written.platforms[target].url;
  for (const target of ["darwin-aarch64-app", "darwin-x86_64-app", "darwin-aarch64", "darwin-x86_64"]) {
    assert.equal(url(target), `${base}QuotaDial_0.3.0_universal.app.tar.gz`);
  }
  assert.equal(url("windows-x86_64-msi"), `${base}QuotaDial_0.3.0_x64_en-US.msi`);
  assert.equal(url("windows-x86_64"), `${base}QuotaDial_0.3.0_x64_en-US.msi`);
  assert.equal(url("linux-x86_64-appimage"), `${base}QuotaDial_0.3.0_amd64.AppImage`);
  assert.equal(url("linux-x86_64"), `${base}QuotaDial_0.3.0_amd64.AppImage`);
  assert.equal(url("linux-x86_64-deb"), `${base}QuotaDial_0.3.0_amd64.deb`);
  assert.equal(Object.keys(written.platforms).length, 9);
  assert.equal(signedVersion(written.platforms["linux-x86_64-deb"].signature), "0.3.0");
});

test("the copies without the version change nothing: latest.json names the versioned files (D-116)", () => {
  const folder = release(EVERY);
  for (const name of [
    "QuotaDial_0.3.0_universal.dmg",
    "quotadial-v0.3.0-aarch64-apple-darwin.tar.gz",
    "quotadial-v0.3.0-x86_64-apple-darwin.tar.gz",
    "quotadial-v0.3.0-x86_64-unknown-linux-gnu.tar.gz",
    "quotadial-v0.3.0-x86_64-pc-windows-msvc.zip",
  ]) {
    writeFileSync(join(folder, name), "not an update");
  }
  copyLatest(folder, "0.3.0");

  const files = Object.values(manifest(options(folder)).platforms).map(({ url }) => url.split("/").pop());
  assert.deepEqual([...new Set(files)].sort(), [
    "QuotaDial_0.3.0_amd64.AppImage",
    "QuotaDial_0.3.0_amd64.deb",
    "QuotaDial_0.3.0_universal.app.tar.gz",
    "QuotaDial_0.3.0_x64_en-US.msi",
  ]);
  assert.equal(files.some((file) => NAMES.includes(file)), false);
});

test("a release built without the updater key offers no update", () => {
  const folder = mkdtempSync(join(tmpdir(), "quotadial-manifest-"));
  writeFileSync(join(folder, "QuotaDial_0.3.0_universal.dmg"), "unsigned");

  assert.equal(manifest(options(folder)), null);
});

test("a file signed for another version is refused, as the app would refuse it", () => {
  assert.throws(
    () => manifest(options(release(EVERY, "0.2.0"))),
    /QuotaDial_0\.3\.0_.* is signed for version 0\.2\.0, not 0\.3\.0/,
  );
  const unversioned = release(EVERY);
  writeFileSync(join(unversioned, "QuotaDial_0.3.0_x64_en-US.msi.sig"), signature("x.msi", undefined));
  assert.throws(() => manifest(options(unversioned)), /signed for version none, not 0\.3\.0/);
});

test("every system needs its signed file", () => {
  assert.throws(
    () => manifest(options(release(EVERY.slice(0, 3)))),
    /No signed file for Linux \(deb\), so it could not update\./,
  );
});

test("a signed file the updater cannot install is a mistake, not something to skip", () => {
  assert.throws(
    () => manifest(options(release([...EVERY, "QuotaDial_0.3.0_x64-setup.exe"]))),
    /QuotaDial_0\.3\.0_x64-setup\.exe is signed, but the updater installs no file of its kind\./,
  );
});

test("each system has one signed file, and each signature its file", () => {
  const twice = release([...EVERY, "QuotaDial_0.3.0_x64_en-GB.msi"]);
  assert.throws(() => manifest(options(twice)), /more than one signed file for Windows/);

  const lonely = release(EVERY.slice(1));
  writeFileSync(join(lonely, "QuotaDial.app.tar.gz.sig"), signature("QuotaDial.app.tar.gz", "0.3.0"));
  assert.throws(() => manifest(options(lonely)), /QuotaDial\.app\.tar\.gz\.sig has no QuotaDial\.app\.tar\.gz beside it\./);
});

test("run through a link, it still writes latest.json rather than doing nothing and succeeding", () => {
  const folder = release(EVERY);
  const linked = mkdtempSync(join(tmpdir(), "quotadial-manifest-"));
  symlinkSync(fileURLToPath(new URL("./updater-manifest.mjs", import.meta.url)), join(linked, "updater-manifest.mjs"));
  writeFileSync(join(linked, "notes.md"), "### Added\n\n- Updates.\n");
  const output = join(linked, "latest.json");
  const run = spawnSync(
    process.execPath,
    [join(linked, "updater-manifest.mjs"), folder, "0.3.0", join(linked, "notes.md"), output],
    { encoding: "utf8" },
  );
  assert.equal(run.stdout, `Wrote ${output} for 0.3.0, for 9 targets.\n`);
  assert.equal(Object.keys(JSON.parse(readFileSync(output, "utf8")).platforms).length, 9);
});
