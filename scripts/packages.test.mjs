// node --test scripts/packages.test.mjs
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdtempSync, readFileSync, readdirSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

import { NAMES, latestNames } from "./latest-names.mjs";
import { cask, files, formula, readSums, scoop, winget, write } from "./packages.mjs";

const sha = (text) => createHash("sha256").update(text).digest("hex");

/** A release's SHA256SUMS.txt, as the release workflow writes it, for `names`. */
function sumsFor(version, names = Object.values(files(version))) {
  return names.map((name) => `${sha(name)}  ${name}`).join("\n") + "\n";
}

test("reads SHA256SUMS.txt as sha256sum writes it", () => {
  const sums = readSums(`${sha("a")}  quotadial-v0.3.0-x86_64-pc-windows-msvc.zip\n${sha("b")} *latest.json\nnot a line\n`);
  assert.equal(sums.get("quotadial-v0.3.0-x86_64-pc-windows-msvc.zip"), sha("a"));
  assert.equal(sums.get("latest.json"), sha("b"));
  assert.equal(sums.size, 2);
});

test("the formula installs both commands, their pages and completions, per system", () => {
  const text = formula("0.3.0", readSums(sumsFor("0.3.0")));
  for (const target of ["aarch64-apple-darwin", "x86_64-apple-darwin", "x86_64-unknown-linux-gnu"]) {
    const name = `quotadial-v0.3.0-${target}.tar.gz`;
    assert.match(text, new RegExp(`url "https://github.com/pghmatt/quotadial/releases/download/v0\\.3\\.0/${name}"\\n\\s+sha256 "${sha(name)}"`));
  }
  assert.match(text, /bin\.install "quotadial", "qd"/);
  assert.match(text, /man1\.install Dir\["man\/\*\.1"\]/);
  assert.match(text, /zsh_completion\.install "completions\/_quotadial", "completions\/_qd"/);
  assert.match(text, /assert_match version\.to_s/);
});

test("the cask links the app's command, every manual page and the completions", () => {
  const text = cask("0.3.0", readSums(sumsFor("0.3.0")), ["quotadial-status.1", "quotadial.1"]);
  assert.match(text, /^cask "quotadial-app" do$/m);
  assert.match(text, new RegExp(`sha256 "${sha("QuotaDial_0.3.0_universal.dmg")}"`));
  assert.match(text, /url "https:\/\/github\.com\/pghmatt\/quotadial\/releases\/download\/v#\{version\}\/QuotaDial_#\{version\}_universal\.dmg"/);
  assert.match(text, /binary "#\{appdir\}\/QuotaDial\.app\/Contents\/MacOS\/quotadial"/);
  assert.match(text, /manpage "#\{appdir\}\/QuotaDial\.app\/Contents\/Resources\/man\/quotadial-status\.1"/);
  assert.match(text, /manpage "#\{appdir\}\/QuotaDial\.app\/Contents\/Resources\/man\/quotadial\.1"/);
  assert.doesNotMatch(text, /auto_updates/, "no latest.json, so the app cannot update itself");
  assert.doesNotMatch(text, /\n\n\n/);
});

test("the cask says the app updates itself only when the release has latest.json", () => {
  const names = [...Object.values(files("0.3.0")), "latest.json"];
  assert.match(cask("0.3.0", readSums(sumsFor("0.3.0", names)), ["quotadial.1"]), /\n  auto_updates true\n/);
});

test("the Scoop manifest installs both commands from the Windows archive and updates itself", () => {
  const manifest = JSON.parse(scoop("0.3.0", readSums(sumsFor("0.3.0"))));
  const zip = "quotadial-v0.3.0-x86_64-pc-windows-msvc.zip";
  assert.equal(manifest.version, "0.3.0");
  assert.equal(manifest.architecture["64bit"].url, `https://github.com/pghmatt/quotadial/releases/download/v0.3.0/${zip}`);
  assert.equal(manifest.architecture["64bit"].hash, sha(zip));
  assert.equal(manifest.architecture["64bit"].extract_dir, "quotadial-v0.3.0-x86_64-pc-windows-msvc");
  assert.deepEqual(manifest.bin, ["quotadial.exe", "qd.exe"]);
  assert.equal(manifest.autoupdate.architecture["64bit"].extract_dir, "quotadial-v$version-x86_64-pc-windows-msvc");
  assert.equal(manifest.autoupdate.hash.url, "$baseurl/SHA256SUMS.txt");
});

test("winget's manifests name the msi, its hash and the product code", () => {
  const manifests = winget("0.3.0", readSums(sumsFor("0.3.0")), "{01234567-89AB-CDEF-0123-456789ABCDEF}");
  assert.deepEqual(Object.keys(manifests).sort(), [
    "QuotaDial.QuotaDial.installer.yaml",
    "QuotaDial.QuotaDial.locale.en-US.yaml",
    "QuotaDial.QuotaDial.yaml",
  ]);
  const installer = manifests["QuotaDial.QuotaDial.installer.yaml"];
  assert.match(installer, /InstallerUrl: https:\/\/github\.com\/pghmatt\/quotadial\/releases\/download\/v0\.3\.0\/QuotaDial_0\.3\.0_x64_en-US\.msi/);
  assert.match(installer, new RegExp(`InstallerSha256: ${sha("QuotaDial_0.3.0_x64_en-US.msi").toUpperCase()}`));
  assert.match(installer, /ProductCode: '\{01234567-89AB-CDEF-0123-456789ABCDEF\}'/);
  for (const text of Object.values(manifests)) {
    assert.match(text, /^PackageVersion: 0\.3\.0$/m);
    assert.match(text, /^ManifestVersion: 1\.10\.0$/m);
  }
});

test("a release that lacks a file names every one it lacks", () => {
  const names = Object.values(files("0.3.0")).filter((name) => !name.endsWith(".msi") && !name.endsWith(".dmg"));
  assert.throws(
    () => formula("0.3.0", readSums(sumsFor("0.3.0", names))),
    /SHA256SUMS\.txt has no QuotaDial_0\.3\.0_universal\.dmg, QuotaDial_0\.3\.0_x64_en-US\.msi/,
  );
});

test("every package names the versioned files, never a copy without the version (D-116)", () => {
  // A package manager pins a version, so even with both names in SHA256SUMS.txt, as each release
  // after 0.2.0 has them, no package may name a copy or releases/latest.
  const names = [...latestNames("0.3.0")].flat();
  const out = mkdtempSync(join(tmpdir(), "quotadial-packages-"));
  try {
    write("0.3.0", readSums(sumsFor("0.3.0", [...names, "latest.json"])), out, "{01234567-89AB-CDEF-0123-456789ABCDEF}");
    const written = readdirSync(out, { recursive: true, withFileTypes: true }).filter((entry) => entry.isFile());
    assert.equal(written.length, 6);
    for (const entry of written) {
      const text = readFileSync(join(entry.parentPath, entry.name), "utf8");
      assert.doesNotMatch(text, /releases\/latest/, entry.name);
      for (const name of NAMES) {
        assert.ok(!text.includes(name), `${entry.name} names ${name}`);
      }
    }
  } finally {
    rmSync(out, { recursive: true, force: true });
  }
});

test("writes the tap's layout and winget's", () => {
  const out = mkdtempSync(join(tmpdir(), "quotadial-packages-"));
  try {
    write("0.3.0", readSums(sumsFor("0.3.0")), out);
    assert.deepEqual(readdirSync(out).sort(), ["Casks", "Formula", "bucket", "winget"]);
    assert.match(readFileSync(join(out, "Casks/quotadial-app.rb"), "utf8"), /manpage .*\/quotadial\.1"/);
    assert.equal(readdirSync(join(out, "winget/0.3.0")).length, 3);
  } finally {
    rmSync(out, { recursive: true, force: true });
  }
});
