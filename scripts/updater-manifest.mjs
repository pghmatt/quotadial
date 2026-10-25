// Writes latest.json, which the app's updater reads from QuotaDial's newest full release (D-103),
// from a release's files and the .sig signatures `tauri build` makes when the updater key is set.
// Each system gets the file its kind of install takes: the app bundle on macOS, the msi on
// Windows, and the AppImage or the deb on Linux. Every signature must name the release's version,
// as the app requires (`requireSignedVersion`), and every system must have one. A release built
// without the key has no signatures, and no latest.json: it offers no update.
//
//   node scripts/updater-manifest.mjs <folder> <version> <notes file> <output>
//
// The download addresses are the release's own files on GitHub, in the repository GITHUB_REPOSITORY
// names (pghmatt/quotadial by default).
import { readdirSync, readFileSync, writeFileSync } from "node:fs";
import { basename, join } from "node:path";
import { pathToFileURL } from "node:url";

/** The kinds of file the updater installs, and the targets each serves, as the updater looks
 * them up: `{os}-{arch}-{installer}` first, then `{os}-{arch}`. */
const KINDS = [
  {
    suffix: ".app.tar.gz",
    system: "macOS",
    targets: ["darwin-aarch64-app", "darwin-x86_64-app", "darwin-aarch64", "darwin-x86_64"],
  },
  { suffix: ".msi", system: "Windows", targets: ["windows-x86_64-msi", "windows-x86_64"] },
  {
    suffix: ".AppImage",
    system: "Linux (AppImage)",
    targets: ["linux-x86_64-appimage", "linux-x86_64"],
  },
  { suffix: ".deb", system: "Linux (deb)", targets: ["linux-x86_64-deb"] },
];

/** Every file under `folder`, with its path, in order. */
function files(folder) {
  return readdirSync(folder, { recursive: true, withFileTypes: true })
    .filter((entry) => entry.isFile())
    .map((entry) => join(entry.parentPath, entry.name))
    .sort();
}

/** The version a signature names in its trusted comment, which the signature covers. */
export function signedVersion(signature) {
  const text = Buffer.from(signature, "base64").toString("utf8");
  const comment = text.split("\n").find((line) => line.startsWith("trusted comment: "));
  return comment
    ?.slice("trusted comment: ".length)
    .split("\t")
    .find((field) => field.startsWith("version:"))
    ?.slice("version:".length);
}

/**
 * The update manifest for `version`, from the signed files in `folder`, or null when none is
 * signed. Throws, naming what is wrong, when a signature names another version, a system has no
 * signed file, or a signed file is of a kind the updater does not install.
 */
export function manifest({ folder, version, notes, repository, now }) {
  const all = files(folder);
  const signatures = all.filter((path) => path.endsWith(".sig"));
  if (signatures.length === 0) {
    return null;
  }
  const platforms = {};
  const found = new Set();
  for (const signaturePath of signatures) {
    const file = basename(signaturePath, ".sig");
    const kind = KINDS.find((candidate) => file.endsWith(candidate.suffix));
    if (!kind) {
      throw new Error(`${file} is signed, but the updater installs no file of its kind.`);
    }
    if (found.has(kind)) {
      throw new Error(`There is more than one signed file for ${kind.system}.`);
    }
    if (!all.includes(signaturePath.slice(0, -".sig".length))) {
      throw new Error(`${file}.sig has no ${file} beside it.`);
    }
    const signature = readFileSync(signaturePath, "utf8").trim();
    const signed = signedVersion(signature);
    if (signed !== version) {
      throw new Error(
        `${file} is signed for version ${signed ?? "none"}, not ${version}: build it again from the tag.`,
      );
    }
    found.add(kind);
    const url = `https://github.com/${repository}/releases/download/v${version}/${encodeURIComponent(file)}`;
    for (const target of kind.targets) {
      platforms[target] = { signature, url };
    }
  }
  const missing = KINDS.filter((kind) => !found.has(kind)).map((kind) => kind.system);
  if (missing.length > 0) {
    throw new Error(`No signed file for ${missing.join(", ")}, so it could not update.`);
  }
  return {
    version,
    notes,
    pub_date: now.toISOString().replace(/\.\d{3}Z$/, "Z"),
    platforms,
  };
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) {
  const [folder, version, notesFile, output] = process.argv.slice(2);
  if (!folder || !version || !notesFile || !output) {
    console.error("usage: node scripts/updater-manifest.mjs <folder> <version> <notes file> <output>");
    process.exit(2);
  }
  try {
    const written = manifest({
      folder,
      version,
      notes: readFileSync(notesFile, "utf8").trim(),
      repository: process.env.GITHUB_REPOSITORY || "pghmatt/quotadial",
      now: new Date(),
    });
    if (written) {
      writeFileSync(output, `${JSON.stringify(written, null, 2)}\n`);
      console.log(`Wrote ${output} for ${version}, for ${Object.keys(written.platforms).length} targets.`);
    } else {
      console.log(`No file is signed for the updater, so ${version} offers no update.`);
    }
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}
