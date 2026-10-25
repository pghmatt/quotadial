# Releasing QuotaDial

How a release is made, what signs it, how the app's updater finds it (D-103 to D-105), and the
names that always fetch the newest full release (D-116). Every kind of signing is a step in the
Release workflow (`.github/workflows/release.yml`) that runs only when its secrets are set, through
`scripts/release-signing.sh`, so a release builds with all of them, some or none. Without them the
builds are unsigned, as `docs/INSTALL.md` describes.

## Making a release

1. **The version bump**, in one small pull request: the version in `Cargo.toml`,
   `apps/tray/src-tauri/Cargo.toml`, `apps/tray/src-tauri/tauri.conf.json` and
   `apps/tray/package.json`; `CHANGELOG.md`'s Unreleased section renamed to the version and its
   date; `scripts/update-prices.sh` run and its diff reviewed; and the manual pages written again,
   since each names its version (`QUOTADIAL_WRITE_PACKAGING=1 cargo test -p quotadial packaging`;
   CI fails until they are). Merge it on green.
2. **The tag**, on `main` at that commit: `git tag v0.3.0 && git push origin v0.3.0`.
3. **The Release workflow** builds the CLI for four platforms and the app as a universal dmg, an
   msi, a deb and an AppImage, signing what its secrets allow, and drafts a pre-release with:
   - every file, and `SHA256SUMS.txt`, their checksums;
   - each of the eight downloads (the dmg, the msi, the deb, the AppImage and the command's four
     archives) a second time, under a name without the version, such as `QuotaDial_universal.dmg`
     ([below](#names-without-the-version));
   - `quotadial-v<version>-cli.cdx.json` and `QuotaDial_<version>_app.cdx.json`, the software bills
     of materials: every crate the command and the app are built from, for every system, as
     CycloneDX JSON (`cargo cyclonedx`);
   - with the updater key set, `latest.json`, the macOS update
     (`QuotaDial_<version>_universal.app.tar.gz`) and a `.sig` beside each file the updater
     installs;
   - notes from the version's section of `CHANGELOG.md`, under a line saying what is signed.

   Each CLI it ships, the archives' and the one inside the app, is first run where no sign-in can
   be found (`scripts/no-example-provider.sh`), and the workflow fails if its `doctor` or
   `status --json` names a provider `fake`: the example build's, whose made-up accounts are only
   for pictures of the app (D-117).
4. **Publishing** is Matt's. Published as a pre-release, it can be downloaded. Published as a full
   release, which GitHub then marks Latest, it is also what every installed copy's updater offers,
   within a day, and what the names without the version fetch.
5. **The package managers** follow a full release, never a pre-release: the Packages workflow
   (`.github/workflows/packages.yml`) runs when one is published. `scripts/packages.mjs` writes the
   Homebrew formula (`quotadial`, the command for macOS and Linux) and cask (`quotadial-app`, the
   app with the command it carries), the Scoop manifest (the command for Windows) and winget's
   manifests (the app, as `QuotaDial.QuotaDial`), from the release's `SHA256SUMS.txt` and the msi's
   product code. With `HOMEBREW_TAP_TOKEN` set it commits the first three to
   `pghmatt/homebrew-quotadial`, which is both the tap and the Scoop bucket; with `WINGET_TOKEN`
   set it opens the pull request to `microsoft/winget-pkgs`, which winget's maintainers merge
   after their checks. Without them, the files are the run's `packages` artifact. Running the
   workflow by hand with a version does the same for a release already out.
6. **The website** follows a full release too: publishing one runs the Site deploy workflow
   (`.github/workflows/site-deploy.yml`), which publishes quotadial.com once
   `CLOUDFLARE_API_TOKEN` is set (#51) and only when the newest full release carries
   `SHA256SUMS.txt` and all eight copies without the version, so no download button answers 404.

## The secrets and variables

Set each under the repository's **Settings > Secrets and variables > Actions**, or with `gh` as
shown. A secret is never printed: GitHub hides it in logs, and the workflow passes each one only to
the step that builds. Never paste one into an issue or a commit.

### macOS: a Developer ID and notarization

With all six, the app and the dmg are signed with the Developer ID, with the hardened runtime, and
the app is notarized and stapled, so it opens with no warning.

| Name | Kind | What |
| --- | --- | --- |
| `APPLE_CERTIFICATE` | Secret | The Developer ID Application certificate with its private key, exported from Keychain Access as a `.p12`, then base64: `base64 -i certificate.p12 \| gh secret set APPLE_CERTIFICATE --repo pghmatt/quotadial` |
| `APPLE_CERTIFICATE_PASSWORD` | Secret | The password the `.p12` was exported with |
| `APPLE_SIGNING_IDENTITY` | Secret | The certificate's name, as `security find-identity -v -p codesigning` lists it, such as `Developer ID Application: Your Name (TEAMID)` |
| `APPLE_ID` | Secret | The Apple Account that notarizes |
| `APPLE_PASSWORD` | Secret | An app-specific password for that account, made at account.apple.com under Sign-In and Security |
| `APPLE_TEAM_ID` | Secret | The team's id, from developer.apple.com under Membership |

With the first three and not the last three, the app is signed but not notarized, and macOS still
warns. A Developer ID also keeps the Keychain's "allow" for Claude Code's sign-in across updates:
macOS ties it to the signature, and an ad hoc signature changes with every build.

### Windows: Azure Artifact Signing

With all six, the app and the msi are signed by Azure Artifact Signing (formerly Trusted Signing)
through [`artifact-signing-cli`](https://crates.io/crates/artifact-signing-cli), which the workflow
installs. Certificate authorities have issued code-signing keys only on hardware since 2023, so a
certificate file cannot sign in CI; Artifact Signing keeps the key in Azure.

| Name | Kind | What |
| --- | --- | --- |
| `AZURE_TENANT_ID` | Secret | The Microsoft Entra tenant of the app registration below |
| `AZURE_CLIENT_ID` | Secret | An app registration given the Artifact Signing Certificate Profile Signer role on the certificate profile |
| `AZURE_CLIENT_SECRET` | Secret | A client secret of that app registration |
| `AZURE_SIGNING_ENDPOINT` | Variable | The account's region endpoint, such as `https://eus.codesigning.azure.net` |
| `AZURE_SIGNING_ACCOUNT` | Variable | The Artifact Signing account's name |
| `AZURE_SIGNING_PROFILE` | Variable | The certificate profile's name |

The last three are variables, not secrets, since they are not secret: `gh variable set
AZURE_SIGNING_ENDPOINT --repo pghmatt/quotadial --body https://eus.codesigning.azure.net`.

### Updates: the updater key

With both, each file the updater installs gets a `.sig`, and the release gets `latest.json`.

| Name | Kind | What |
| --- | --- | --- |
| `TAURI_SIGNING_PRIVATE_KEY` | Secret | The updater key: the contents of `updater.key` |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Secret | Its password |

The one command that makes an updater keypair, run in `apps/tray`:

```sh
npx tauri signer generate -w "$HOME/Library/Application Support/quotadial-issue/updater.key"
```

It asks for a password and writes `updater.key` and `updater.key.pub`. The public half goes in
`apps/tray/src-tauri/tauri.conf.json` as `plugins.updater.pubkey`, which every build carries.

QuotaDial's updater key was made this way on Matt's Mac for the pull request that added updates
(#59), with a random password kept in his login Keychain as "QuotaDial updater signing key"; the
public half is the one committed. These two commands put both into the repository's secrets without
showing either:

```sh
gh secret set TAURI_SIGNING_PRIVATE_KEY --repo pghmatt/quotadial \
  < "$HOME/Library/Application Support/quotadial-issue/updater.key"
printf '%s' "$(security find-generic-password -s 'QuotaDial updater signing key' -w)" \
  | gh secret set TAURI_SIGNING_PRIVATE_KEY_PASSWORD --repo pghmatt/quotadial
```

Keep a copy of `updater.key` and its password in a password manager. Without them, no update can
reach the copies already installed.

### Package managers: the tap and winget

- `HOMEBREW_TAP_TOKEN`: a fine-grained personal access token with **Contents: Read and write** on
  `pghmatt/homebrew-quotadial` only. The repository is public and starts empty; the workflow
  writes `Formula/`, `Casks/` and `bucket/` into it.
- `WINGET_TOKEN`: a classic personal access token with the `public_repo` scope, which winget's tool
  needs to fork `microsoft/winget-pkgs` and open the pull request. Its first pull request adds
  QuotaDial to winget; later ones update it.

```sh
gh secret set HOMEBREW_TAP_TOKEN --repo pghmatt/quotadial   # asks for the value; nothing is shown
gh secret set WINGET_TOKEN --repo pghmatt/quotadial
```

## How the app updates

- While Settings' **Check for updates once a day** is on, the app asks for
  `https://github.com/pghmatt/quotadial/releases/latest/download/latest.json` two minutes after it
  starts, then a day after the last answer, or an hour after an ask that went unanswered. Help's
  **Check for updates** asks at any time. `docs/PRIVACY.md` says what the request carries: nothing
  about the person or their usage.
- GitHub answers that address from the newest full release. A draft or a pre-release never reaches
  the updater, so publishing a release as a full release is what rolls it out. Before the first
  full release with a `latest.json` the address answers 404, which the app takes as nothing newer.
- A newer version is offered in the popover and in Help, and the daily check tells it once as a
  notification. Nothing is downloaded until the person chooses **Update**.
- The updater checks the download against the public key in `tauri.conf.json`, and checks that the
  version its signature names is the version `latest.json` announces (`requireSignedVersion`).
  `latest.json` itself is not signed, so this is what stops an older release, still validly signed,
  from being passed off as newer. A download that fails either check is not installed.
- The app installs it, stops its daemon, and starts again as the new version. On macOS it swaps the
  app in place, asking for the administrator password only when the folder needs it; on Windows the
  msi installs with a progress bar and opens the new version; on Linux the AppImage is replaced, or
  the deb installed through the system's password prompt.
- `scripts/updater-manifest.mjs` writes `latest.json` from the release's `.sig` files, giving each
  kind of install its own file: the app on macOS, the msi on Windows, and the AppImage or the deb on
  Linux. It refuses a signature for another version and a system with no signed file, so a release
  is never half updatable.

## Names without the version

From the release after 0.2.0, each of the eight downloads, the app's dmg, msi, deb and AppImage
and the command's four archives, is on the release twice: under the name its build gives it, such
as `QuotaDial_0.3.0_universal.dmg`, and as a copy named without the version, such as
`QuotaDial_universal.dmg` (D-116). GitHub answers
`https://github.com/pghmatt/quotadial/releases/latest/download/<name>` from the newest full release,
so a link to a copy always fetches that release's file. `docs/INSTALL.md` and the website's download
buttons link to the copies: one address for each system, which never goes stale.

- The copy is a convenience name for the newest full release, and the versioned file is the record.
  `latest.json` and `scripts/packages.mjs` name only the versioned files: the updater is unchanged,
  and a package manager pins a version.
- The publish job makes the copies with `scripts/latest-names.mjs`, after `latest.json` is written
  and before the checksums. It copies, never renames or links, so both names hold the same signed
  and notarized bytes and `SHA256SUMS.txt` lists both with one SHA-256. It fails, naming the file,
  if any of the eight is missing, so a release never goes out with a link that answers 404, and it
  copies nothing else: not the `.sig` files, the macOS update, `latest.json` or the bills of
  materials.
- While every release is a pre-release, `releases/latest/download/` answers 404. The website waits
  for a full release with all eight copies (step 6), and `docs/INSTALL.md` names each versioned
  file beside its link and sends people to the Releases page until then.

## Checking a download

`SHA256SUMS.txt` lists every file's SHA-256, each download under both its names, so the check passes
whichever was downloaded. In a folder with the downloads:

```sh
shasum -a 256 -c SHA256SUMS.txt --ignore-missing    # macOS
sha256sum -c SHA256SUMS.txt --ignore-missing        # Linux
```

On Windows, `Get-FileHash QuotaDial_<version>_x64_en-US.msi` prints the hash to compare.

## If the updater key is lost or leaks

A copy trusts only the public key it was built with, so a new key reaches installed copies through
one release signed with the old key:

1. Make a new keypair with the command above, into a new file.
2. Put its public half in `tauri.conf.json`, and release that version signed with the **old** key.
   Installed copies accept it, since the old key signed it, and from then on trust the new key.
3. Put the new key and its password in the two secrets, in place of the old ones. Every release
   after is signed with the new key.

If the old key is lost, step 2 cannot happen: installed copies will not update again, and each
person has to download the next release by hand once, after which updates work as before. If it
leaked, do all three at once, since whoever holds it could sign an update that a release published
on this repository would carry.
