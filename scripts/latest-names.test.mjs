// Tests for latest-names.mjs, run by CI with `node --test scripts/*.test.mjs`. A release is laid out
// as the Release workflow's publish job downloads it, with the names of v0.2.0's real files.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  lstatSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  statSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join, relative } from "node:path";
import { after, test } from "node:test";
import { fileURLToPath } from "node:url";

import { NAMES, copyLatest, latestNames } from "./latest-names.mjs";

const SCRIPT = fileURLToPath(new URL("./latest-names.mjs", import.meta.url));
const INSTALL = fileURLToPath(new URL("../docs/INSTALL.md", import.meta.url));

/** v0.2.0's downloads, as `gh release view v0.2.0 --json assets` lists them. */
const V0_2_0 = [
  "quotadial-v0.2.0-aarch64-apple-darwin.tar.gz",
  "quotadial-v0.2.0-x86_64-apple-darwin.tar.gz",
  "quotadial-v0.2.0-x86_64-pc-windows-msvc.zip",
  "quotadial-v0.2.0-x86_64-unknown-linux-gnu.tar.gz",
  "QuotaDial_0.2.0_amd64.AppImage",
  "QuotaDial_0.2.0_amd64.deb",
  "QuotaDial_0.2.0_universal.dmg",
  "QuotaDial_0.2.0_x64_en-US.msi",
];

/** The rest of a release built with the updater key: the update and its signatures, latest.json
 * and the bills of materials. */
const OTHERS = [
  "macos/QuotaDial_0.2.0_universal.app.tar.gz",
  "macos/QuotaDial_0.2.0_universal.app.tar.gz.sig",
  "QuotaDial_0.2.0_x64_en-US.msi.sig",
  "deb/QuotaDial_0.2.0_amd64.deb.sig",
  "appimage/QuotaDial_0.2.0_amd64.AppImage.sig",
  "latest.json",
  "quotadial-v0.2.0-cli.cdx.json",
  "QuotaDial_0.2.0_app.cdx.json",
];

/** Where the publish job finds a file: the dmg, deb and AppImage keep their bundle's folder. */
const at = (name) => `${{ dmg: "dmg/", deb: "deb/", AppImage: "appimage/" }[name.split(".").pop()] ?? ""}${name}`;

const made = [];
after(() => made.forEach((folder) => rmSync(folder, { recursive: true, force: true })));

/** A release folder holding `paths`, each file's bytes its own path. */
function release(paths) {
  const folder = mkdtempSync(join(tmpdir(), "quotadial-latest-"));
  made.push(folder);
  for (const path of paths) {
    mkdirSync(join(folder, path, ".."), { recursive: true });
    writeFileSync(join(folder, path), `the bytes of ${path}`);
  }
  return folder;
}

/** Every file in `folder`, by its path inside it. */
const list = (folder) =>
  readdirSync(folder, { recursive: true, withFileTypes: true })
    .filter((entry) => entry.isFile())
    .map((entry) => relative(folder, join(entry.parentPath, entry.name)))
    .sort();

test("the eight are v0.2.0's downloads, each named without its version and nothing else changed", () => {
  const names = latestNames("0.2.0");
  assert.deepEqual([...names.keys()].sort(), [...V0_2_0].sort());
  assert.deepEqual([...names.values()], NAMES);
  for (const [file, name] of names) {
    assert.equal(name, file.replace("_0.2.0_", "_").replace("-v0.2.0-", "-"));
  }
});

test("each download is copied beside itself, the same bytes in a file of its own", () => {
  const folder = release(V0_2_0.map(at));
  const copies = copyLatest(folder, "0.2.0");
  assert.deepEqual(copies.map((path) => relative(folder, path)).sort(), NAMES.map(at).sort());
  for (const [file, name] of latestNames("0.2.0")) {
    const original = join(folder, at(file));
    const copy = join(folder, at(name));
    assert.deepEqual(readFileSync(copy), readFileSync(original));
    assert.equal(lstatSync(copy).isSymbolicLink(), false, `${name} is a link to ${file}`);
    assert.notEqual(statSync(copy).ino, statSync(original).ino, `${name} is ${file} under a second name`);
  }
});

test("nothing else is copied: no signature, update, latest.json or bill of materials", () => {
  const folder = release([...V0_2_0.map(at), ...OTHERS]);
  const before = list(folder);
  copyLatest(folder, "0.2.0");
  const added = list(folder).filter((path) => !before.includes(path));
  assert.deepEqual(added, NAMES.map(at).sort());
});

test("a missing download fails, naming each one, before anything is copied", () => {
  const folder = release(V0_2_0.filter((name) => !/\.deb$|\.zip$/.test(name)).map(at));
  const before = list(folder);
  const named = /has no QuotaDial_0\.2\.0_amd64\.deb, quotadial-v0\.2\.0-x86_64-pc-windows-msvc\.zip, so a link without the version would answer 404/;
  assert.throws(() => copyLatest(folder, "0.2.0"), named);
  assert.deepEqual(list(folder), before);

  // As the Release workflow runs it: the step fails, with the same words.
  const run = spawnSync(process.execPath, [SCRIPT, folder, "0.2.0"], { encoding: "utf8" });
  assert.equal(run.status, 1);
  assert.match(run.stderr, named);
  assert.deepEqual(list(folder), before);
});

test("a download that is there twice fails rather than pick one", () => {
  const folder = release([...V0_2_0.map(at), "msi/QuotaDial_0.2.0_x64_en-US.msi"]);
  assert.throws(() => copyLatest(folder, "0.2.0"), /QuotaDial_0\.2\.0_x64_en-US\.msi is in .* 2 times/);
  assert.equal(list(folder).some((path) => NAMES.includes(path.split("/").pop())), false);
});

test("--names prints the eight, one a line, for the site deploy gate", () => {
  const run = spawnSync(process.execPath, [SCRIPT, "--names"], { encoding: "utf8" });
  assert.equal(run.status, 0);
  assert.equal(run.stdout, `${NAMES.join("\n")}\n`);
  assert.equal(spawnSync(process.execPath, [SCRIPT], { encoding: "utf8" }).status, 2);
});

test("it runs when its path passes through a link, rather than doing nothing and succeeding", () => {
  const linked = mkdtempSync(join(tmpdir(), "quotadial-latest-"));
  made.push(linked);
  symlinkSync(SCRIPT, join(linked, "latest-names.mjs"));
  const run = spawnSync(process.execPath, [join(linked, "latest-names.mjs"), "--names"], { encoding: "utf8" });
  assert.equal(run.stdout, `${NAMES.join("\n")}\n`);
});

test("docs/INSTALL.md links each of the eight copies of v0.2.0's downloads, and nothing else", () => {
  const linked = [...readFileSync(INSTALL, "utf8").matchAll(/releases\/latest\/download\/([^\s)`'"<>]+)/g)].map(
    (found) => found[1],
  );
  const copies = V0_2_0.map((file) => latestNames("0.2.0").get(file));
  for (const name of linked) {
    assert.ok(copies.includes(name), `INSTALL.md links releases/latest/download/${name}, which no release carries`);
  }
  for (const name of copies) {
    assert.ok(linked.includes(name), `INSTALL.md has no link to releases/latest/download/${name}`);
  }
});
