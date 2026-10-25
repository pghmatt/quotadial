// Copies each of a release's eight downloads, the app's dmg, msi, deb and AppImage and the command's
// four archives, to a name without the version, beside it (D-116). GitHub answers
// https://github.com/pghmatt/quotadial/releases/latest/download/<name> from the newest full release,
// so a link to a copy never goes stale; docs/INSTALL.md and the website link to them. The versioned
// file stays the record: latest.json and the package managers name it, never a copy.
//
//   node scripts/latest-names.mjs <folder> <version>   copy each download in <folder>
//   node scripts/latest-names.mjs --names              the eight names, one a line
//
// The Release workflow runs it after signing and latest.json and before the checksums, so both names
// hold the same signed bytes and SHA256SUMS.txt lists both. A download that is not there fails the
// run, naming it, so a release never goes out with a link that answers 404. Nothing else is copied:
// not the .sig files, the updater's .app.tar.gz, latest.json or the bills of materials.
import { copyFileSync, readdirSync } from "node:fs";
import { basename, dirname, join } from "node:path";
import { pathToFileURL } from "node:url";

/** The eight names without the version: the app's installers, then the command's archives. */
export const NAMES = [
  "QuotaDial_universal.dmg",
  "QuotaDial_x64_en-US.msi",
  "QuotaDial_amd64.deb",
  "QuotaDial_amd64.AppImage",
  "quotadial-aarch64-apple-darwin.tar.gz",
  "quotadial-x86_64-apple-darwin.tar.gz",
  "quotadial-x86_64-unknown-linux-gnu.tar.gz",
  "quotadial-x86_64-pc-windows-msvc.zip",
];

/**
 * Each of the eight downloads of `version` by the name the release gives it, with its name without
 * the version: the app's as Tauri names them (`QuotaDial_<version>_...`), the command's as the
 * Release workflow packs them (`quotadial-v<version>-...`).
 */
export function latestNames(version) {
  return new Map(
    NAMES.map((name) => [
      name.startsWith("QuotaDial_")
        ? name.replace("QuotaDial_", `QuotaDial_${version}_`)
        : name.replace("quotadial-", `quotadial-v${version}-`),
      name,
    ]),
  );
}

/** Every file under `folder`, with its path. */
function files(folder) {
  return readdirSync(folder, { recursive: true, withFileTypes: true })
    .filter((entry) => entry.isFile())
    .map((entry) => join(entry.parentPath, entry.name));
}

/**
 * Copies each download of `version` in `folder`, in whichever subfolder it is, to its name without
 * the version in the same folder, and returns the copies' paths. Throws before copying anything,
 * naming each, when a download is missing or is there twice.
 */
export function copyLatest(folder, version) {
  const all = files(folder);
  const copies = [];
  const missing = [];
  for (const [file, name] of latestNames(version)) {
    const found = all.filter((path) => basename(path) === file);
    if (found.length === 0) {
      missing.push(file);
    } else if (found.length > 1) {
      throw new Error(`${file} is in ${folder} ${found.length} times: a release holds one file of each name.`);
    } else {
      copies.push([found[0], join(dirname(found[0]), name)]);
    }
  }
  if (missing.length > 0) {
    throw new Error(
      `${folder} has no ${missing.join(", ")}, so a link without the version would answer 404: ` +
        `is ${version} the release's version?`,
    );
  }
  for (const [from, to] of copies) {
    copyFileSync(from, to);
  }
  return copies.map(([, to]) => to);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const [first, version, extra] = process.argv.slice(2);
  if (first === "--names" && version === undefined) {
    console.log(NAMES.join("\n"));
  } else if (first && first !== "--names" && version && extra === undefined) {
    try {
      const names = copyLatest(first, version).map((path) => basename(path));
      console.log(`Copied ${names.length} downloads to their names without the version: ${names.join(", ")}.`);
    } catch (error) {
      console.error(error instanceof Error ? error.message : String(error));
      process.exit(1);
    }
  } else {
    console.error("usage: node scripts/latest-names.mjs <folder> <version>\n       node scripts/latest-names.mjs --names");
    process.exit(2);
  }
}
