// Writes what the package managers install QuotaDial from, for one release, from the files and
// SHA-256s in its SHA256SUMS.txt (#52, D-112):
//
//   Formula/quotadial.rb       Homebrew: the command, from the macOS and Linux archives
//   Casks/quotadial-app.rb     Homebrew: the app from the dmg, with the command it carries
//   bucket/quotadial.json      Scoop: the command, from the Windows archive
//   winget/<version>/...       winget: the app from the msi, as three manifests
//
// The first three are what pghmatt/homebrew-quotadial holds, which is a Homebrew tap and a Scoop
// bucket at once; the winget manifests go to microsoft/winget-pkgs. The packages workflow runs this
// when a release is published as a full release, as the updater only offers full releases.
//
//   node scripts/packages.mjs <version> <SHA256SUMS.txt> <output folder> [--product-code <guid>]
//
// QUOTADIAL_DOWNLOADS replaces the releases' address, so a formula can be tried against archives on
// this machine (a file:// address); the manual pages the cask links are the ones in packaging/man.
import { mkdirSync, readFileSync, readdirSync, realpathSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const REPOSITORY = "https://github.com/pghmatt/quotadial";
const HOMEPAGE = REPOSITORY;
const SUMMARY = "Track Claude and ChatGPT/Codex usage limits";
const IDENTIFIER = "com.quotadial.desktop";
const WINGET_ID = "QuotaDial.QuotaDial";
const MAN = fileURLToPath(new URL("../packaging/man/", import.meta.url));

/** The release's files by name, each with its SHA-256, from SHA256SUMS.txt's `<hash>  <name>`. */
export function readSums(text) {
  const sums = new Map();
  for (const line of text.split("\n")) {
    const found = line.trim().match(/^([0-9a-f]{64})\s+\*?(\S+)$/);
    if (found) {
      sums.set(found[2], found[1]);
    }
  }
  return sums;
}

/** The files each package needs, named as the release workflow names them. */
export function files(version) {
  const cli = (target, ext = "tar.gz") => `quotadial-v${version}-${target}.${ext}`;
  return {
    macArm: cli("aarch64-apple-darwin"),
    macIntel: cli("x86_64-apple-darwin"),
    linux: cli("x86_64-unknown-linux-gnu"),
    windows: cli("x86_64-pc-windows-msvc", "zip"),
    dmg: `QuotaDial_${version}_universal.dmg`,
    msi: `QuotaDial_${version}_x64_en-US.msi`,
  };
}

/** Each needed file's SHA-256, or an error naming every file the release lacks. */
function hashes(version, sums) {
  const wanted = files(version);
  const missing = Object.values(wanted).filter((name) => !sums.has(name));
  if (missing.length > 0) {
    throw new Error(`SHA256SUMS.txt has no ${missing.join(", ")}: is ${version} the release's version?`);
  }
  return Object.fromEntries(Object.entries(wanted).map(([key, name]) => [key, { name, sha256: sums.get(name) }]));
}

const downloads = (version) =>
  process.env.QUOTADIAL_DOWNLOADS?.replace(/\/$/, "") ?? `${REPOSITORY}/releases/download/v${version}`;

export function formula(version, sums) {
  const file = hashes(version, sums);
  const base = downloads(version);
  return `# QuotaDial's command, written by scripts/packages.mjs in ${REPOSITORY} for each release.
class Quotadial < Formula
  desc "${SUMMARY}"
  homepage "${HOMEPAGE}"
  license "MIT"

  on_macos do
    on_arm do
      url "${base}/${file.macArm.name}"
      sha256 "${file.macArm.sha256}"
    end
    on_intel do
      url "${base}/${file.macIntel.name}"
      sha256 "${file.macIntel.sha256}"
    end
  end

  on_linux do
    on_intel do
      url "${base}/${file.linux.name}"
      sha256 "${file.linux.sha256}"
    end
  end

  def install
    bin.install "quotadial", "qd"
    man1.install Dir["man/*.1"]
    man1.install_symlink "quotadial.1" => "qd.1"
    bash_completion.install "completions/quotadial.bash" => "quotadial"
    bash_completion.install "completions/qd.bash" => "qd"
    zsh_completion.install "completions/_quotadial", "completions/_qd"
    fish_completion.install "completions/quotadial.fish", "completions/qd.fish"
  end

  def caveats
    <<~EOS
      QuotaDial.app, the menu bar app, carries this command too:
        brew install --cask pghmatt/quotadial/quotadial-app
    EOS
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/quotadial --version")
    assert_match "quotadial alerts add", shell_output("#{bin}/qd --help")
  end
end
`;
}

export function cask(version, sums, manPages = readdirSync(MAN).filter((name) => name.endsWith(".1")).sort()) {
  const file = hashes(version, sums);
  const base = downloads(version).replace(`/v${version}`, "/v#{version}");
  const app = "#{appdir}/QuotaDial.app/Contents";
  // The app updates itself only from a release that carries latest.json (D-103).
  const updates = sums.has("latest.json") ? "\n  auto_updates true" : "";
  return `# QuotaDial's app, written by scripts/packages.mjs in ${REPOSITORY} for each release.
cask "quotadial-app" do
  version "${version}"
  sha256 "${file.dmg.sha256}"

  url "${base}/QuotaDial_#{version}_universal.dmg"
  name "QuotaDial"
  desc "Claude and ChatGPT/Codex usage limits in the menu bar"
  homepage "${HOMEPAGE}"
${updates}
  depends_on :macos

  app "QuotaDial.app"
  binary "${app}/MacOS/quotadial"
${manPages.map((page) => `  manpage "${app}/Resources/man/${page}"`).join("\n")}
  bash_completion "${app}/Resources/completions/quotadial.bash"
  zsh_completion "${app}/Resources/completions/_quotadial"
  fish_completion "${app}/Resources/completions/quotadial.fish"

  uninstall quit: "${IDENTIFIER}"

  zap trash: [
    "~/Library/Application Support/${IDENTIFIER}",
    "~/Library/Application Support/quotadial",
    "~/Library/Caches/${IDENTIFIER}",
    "~/Library/Caches/quotadial",
    "~/Library/Caches/quotadial-tray",
    "~/Library/LaunchAgents/QuotaDial.plist",
    "~/Library/WebKit/${IDENTIFIER}",
    "~/Library/WebKit/quotadial-tray",
  ]
end
`.replace("\n\n\n", "\n\n");
}

export function scoop(version, sums) {
  const file = hashes(version, sums);
  const folder = (v) => `quotadial-v${v}-x86_64-pc-windows-msvc`;
  return `${JSON.stringify(
    {
      version,
      description: SUMMARY,
      homepage: HOMEPAGE,
      license: "MIT",
      architecture: {
        "64bit": {
          url: `${downloads(version)}/${file.windows.name}`,
          hash: file.windows.sha256,
          extract_dir: folder(version),
        },
      },
      bin: ["quotadial.exe", "qd.exe"],
      notes: "Tab completion for PowerShell: add `. \"$dir\\completions\\quotadial.ps1\"` to your $PROFILE.",
      checkver: "github",
      autoupdate: {
        architecture: {
          "64bit": {
            url: `${REPOSITORY}/releases/download/v$version/${folder("$version")}.zip`,
            extract_dir: folder("$version"),
          },
        },
        hash: { url: "$baseurl/SHA256SUMS.txt" },
      },
    },
    null,
    2,
  )}\n`;
}

/** winget's three manifests for this version, by file name. */
export function winget(version, sums, productCode) {
  const file = hashes(version, sums);
  const head = (type) =>
    `# Written by scripts/packages.mjs in ${REPOSITORY}.\n` +
    `# yaml-language-server: $schema=https://aka.ms/winget-manifest.${type}.1.10.0.schema.json\n\n` +
    `PackageIdentifier: ${WINGET_ID}\nPackageVersion: ${version}\n`;
  const tail = (type) => `ManifestType: ${type}\nManifestVersion: 1.10.0\n`;
  return {
    [`${WINGET_ID}.yaml`]: `${head("version")}DefaultLocale: en-US\n${tail("version")}`,
    [`${WINGET_ID}.installer.yaml`]:
      `${head("installer")}InstallerType: wix\nScope: machine\nUpgradeBehavior: install\n` +
      (productCode ? `ProductCode: '${productCode}'\n` : "") +
      `ReleaseDate: ${new Date().toISOString().slice(0, 10)}\n` +
      `Installers:\n- Architecture: x64\n  InstallerUrl: ${downloads(version)}/${file.msi.name}\n` +
      `  InstallerSha256: ${file.msi.sha256.toUpperCase()}\n${tail("installer")}`,
    [`${WINGET_ID}.locale.en-US.yaml`]:
      `${head("defaultLocale")}PackageLocale: en-US\nPublisher: QuotaDial\nPublisherUrl: ${HOMEPAGE}\n` +
      `PublisherSupportUrl: ${REPOSITORY}/issues\nPackageName: QuotaDial\nPackageUrl: ${HOMEPAGE}\n` +
      `License: MIT\nLicenseUrl: ${REPOSITORY}/blob/main/LICENSE\n` +
      `ShortDescription: Claude and ChatGPT/Codex usage limits in the notification area.\n` +
      `Description: QuotaDial shows how much of each Claude and ChatGPT/Codex plan limit you have used, ` +
      `when each resets and how old every number is, from the sign-ins Claude Code and the Codex CLI ` +
      `already keep. It carries the quotadial command, a local API and an MCP server for agents.\n` +
      `Tags:\n- claude\n- codex\n- chatgpt\n- usage\n- limits\n` +
      `ReleaseNotesUrl: ${REPOSITORY}/releases/tag/v${version}\n${tail("defaultLocale")}`,
  };
}

/** Writes every package's files for `version` under `out`. */
export function write(version, sums, out, productCode) {
  const put = (path, text) => {
    mkdirSync(join(out, path, ".."), { recursive: true });
    writeFileSync(join(out, path), text);
  };
  put("Formula/quotadial.rb", formula(version, sums));
  put("Casks/quotadial-app.rb", cask(version, sums));
  put("bucket/quotadial.json", scoop(version, sums));
  for (const [name, text] of Object.entries(winget(version, sums, productCode))) {
    put(`winget/${version}/${name}`, text);
  }
}

// import.meta.url is the module's real path, and the path it was run by may pass through a link (as
// /var does on macOS): compared as given, the run would do nothing and succeed.
if (process.argv[1] && import.meta.url === pathToFileURL(realpathSync(process.argv[1])).href) {
  const [version, sumsFile, out, flag, productCode] = process.argv.slice(2);
  if (!version || !sumsFile || !out || (flag && (flag !== "--product-code" || !productCode))) {
    console.error("Usage: node scripts/packages.mjs <version> <SHA256SUMS.txt> <output folder> [--product-code <guid>]");
    process.exit(2);
  }
  if (!/^\d+\.\d+\.\d+$/.test(version)) {
    console.error(`${version} is not a version such as 0.3.0.`);
    process.exit(2);
  }
  write(version, readSums(readFileSync(sumsFile, "utf8")), out, productCode);
  console.log(`Wrote the Homebrew formula and cask, the Scoop manifest and the winget manifests for ${version} to ${out}.`);
}
