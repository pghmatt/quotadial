# Installing QuotaDial

QuotaDial's tray app runs on macOS, Windows and Linux. It puts the share used of your tightest
Claude or ChatGPT/Codex limit in the menu bar or system tray. It also runs the QuotaDial daemon,
so the `quotadial` command, the local API and the MCP server all read its numbers. The app carries
the `quotadial` command inside it.

## Download

Each release on the repository's [Releases page](https://github.com/pghmatt/quotadial/releases)
has these files. From the release after 0.2.0, each also comes as a copy named without the version,
so the links in the last column always fetch the newest full release. They work once the first
full release after 0.2.0 is out; until then, take the file from the Releases page.

| System | File | The newest full release |
| --- | --- | --- |
| macOS (Apple silicon and Intel) | `QuotaDial_<version>_universal.dmg` | [QuotaDial_universal.dmg](https://github.com/pghmatt/quotadial/releases/latest/download/QuotaDial_universal.dmg) |
| Windows | `QuotaDial_<version>_x64_en-US.msi` | [QuotaDial_x64_en-US.msi](https://github.com/pghmatt/quotadial/releases/latest/download/QuotaDial_x64_en-US.msi) |
| Debian or Ubuntu | `QuotaDial_<version>_amd64.deb` | [QuotaDial_amd64.deb](https://github.com/pghmatt/quotadial/releases/latest/download/QuotaDial_amd64.deb) |
| Other Linux | `QuotaDial_<version>_amd64.AppImage` | [QuotaDial_amd64.AppImage](https://github.com/pghmatt/quotadial/releases/latest/download/QuotaDial_amd64.AppImage) |

`SHA256SUMS.txt` lists every file's SHA-256, to check a download against. In the folder you
downloaded to, run the line for your system. On macOS and Linux it prints OK beside each file that
matches; on Windows, PowerShell prints the file's hash, to compare with its line in
`SHA256SUMS.txt`.

| System | Command |
| --- | --- |
| macOS | `shasum -a 256 -c SHA256SUMS.txt --ignore-missing` |
| Linux | `sha256sum -c SHA256SUMS.txt --ignore-missing` |
| Windows | `Get-FileHash .\QuotaDial_*.msi` |

From the release after 0.2.0, `SHA256SUMS.txt` lists both names of each file, so the check passes
whichever you downloaded.

### The command alone

The `quotadial` command without the tray app comes as an archive for each system. Each holds
`quotadial` and its short alias `qd`, with their shell completions in `completions/` and manual
pages in `man/` ([CLI.md](CLI.md) says how to turn them on).

| System | Archive | The newest full release |
| --- | --- | --- |
| macOS, Apple silicon | `quotadial-v<version>-aarch64-apple-darwin.tar.gz` | [quotadial-aarch64-apple-darwin.tar.gz](https://github.com/pghmatt/quotadial/releases/latest/download/quotadial-aarch64-apple-darwin.tar.gz) |
| macOS, Intel | `quotadial-v<version>-x86_64-apple-darwin.tar.gz` | [quotadial-x86_64-apple-darwin.tar.gz](https://github.com/pghmatt/quotadial/releases/latest/download/quotadial-x86_64-apple-darwin.tar.gz) |
| Linux | `quotadial-v<version>-x86_64-unknown-linux-gnu.tar.gz` | [quotadial-x86_64-unknown-linux-gnu.tar.gz](https://github.com/pghmatt/quotadial/releases/latest/download/quotadial-x86_64-unknown-linux-gnu.tar.gz) |
| Windows | `quotadial-v<version>-x86_64-pc-windows-msvc.zip` | [quotadial-x86_64-pc-windows-msvc.zip](https://github.com/pghmatt/quotadial/releases/latest/download/quotadial-x86_64-pc-windows-msvc.zip) |

On macOS or Linux, these lines fetch the newest full release's command and `SHA256SUMS.txt` into a
new folder, and check the archive against that list as above. Only an archive that passes is
unpacked and put on your `PATH`. They name Apple silicon: for another system, use its archive's name
from the table, and on Linux, `sha256sum` in place of `shasum -a 256`.

```sh
cd "$(mktemp -d)" &&
curl -fsSLO https://github.com/pghmatt/quotadial/releases/latest/download/quotadial-aarch64-apple-darwin.tar.gz &&
curl -fsSLO https://github.com/pghmatt/quotadial/releases/latest/download/SHA256SUMS.txt &&
shasum -a 256 -c SHA256SUMS.txt --ignore-missing &&
tar -xzf quotadial-*.tar.gz &&
sudo mkdir -p /usr/local/bin &&
sudo install quotadial-v*/quotadial quotadial-v*/qd /usr/local/bin/
```

## Package managers

The package managers install QuotaDial from its first full release after its Homebrew tap, Scoop
bucket and winget listing are published; a pre-release never reaches them, as it never reaches the
app's updater. If a command below finds nothing yet, take the file from Download above.

| System | The app, with the command | The command alone |
| --- | --- | --- |
| macOS | `brew install --cask pghmatt/quotadial/quotadial-app` | `brew install pghmatt/quotadial/quotadial` |
| Linux | The deb or AppImage above | `brew install pghmatt/quotadial/quotadial` |
| Windows | `winget install QuotaDial.QuotaDial` | `scoop bucket add quotadial https://github.com/pghmatt/homebrew-quotadial`, then `scoop install quotadial` |

Homebrew also installs the manual pages and the completions for bash, zsh and fish. The cask
links the command the app carries, so install the cask or the formula, not both. Until the builds
are signed, macOS will not open the app until you choose Open Anyway in System Settings, as the next
section describes.

## Open a build that is not signed yet

A release's notes say whether it is signed for each system. A signed build opens with no warning.
Until QuotaDial's signing certificates are in place, each operating system warns the first time
you open a build that is not signed. This is how to open it anyway. Only do this for a build you
got from QuotaDial's GitHub repository.

### macOS

1. Open the disk image and drag **QuotaDial** to **Applications**.
2. Open QuotaDial from Applications. macOS says it cannot verify the app. Choose **Done**, not
   **Move to Trash**.
3. Open **System Settings > Privacy & Security**. Under **Security**, choose **Open Anyway** next
   to the message about QuotaDial, and confirm with your password or Touch ID.
4. Open QuotaDial again and choose **Open**.

On macOS 14 and earlier you can instead Control-click QuotaDial in Applications, choose **Open**,
then **Open** again.

QuotaDial shows a dial in the menu bar, with the share used beside it. Click the dial to see every
account and limit; right-click it, or click with two fingers, for the menu. QuotaDial has no Dock
icon: quit it from the menu or from the bottom of the popover.

### Windows

1. Run the installer. Edge may say it "isn't commonly downloaded": choose **...** > **Keep** >
   **Show more** > **Keep anyway**.
2. If Windows says it **protected your PC**, choose **More info**, then **Run anyway**.

QuotaDial's number appears in the notification area. If Windows tucks it under the **^** arrow,
drag it onto the taskbar to keep it in view. Click it to see every account and limit; right-click
it for the menu.

### Linux

- **AppImage:** make it executable, then run it:

  ```sh
  chmod +x QuotaDial_*.AppImage
  ./QuotaDial_*.AppImage
  ```

- **Debian or Ubuntu:** `sudo apt install ./QuotaDial_*.deb`. This also puts the `quotadial`
  command in `/usr/bin`.

The dial and number appear in the panel's tray area. GNOME needs the AppIndicator extension for
that; Ubuntu has it on already.

## Settings

**Settings**, at the bottom of the popover, has these choices:
- Open QuotaDial when you log in.
- Check for updates once a day, on unless you turn it off ([Updates](#updates)).
- Alerts: each alert rule, with a box to turn it on or off and a button to remove it, and a field
  to add one in plain words, such as "warn me at 80%". QuotaDial starts with two: an account needs
  signing in again, and a limit passes 90%. Each alert comes once as a notification, as
  [`CLI.md`](CLI.md#alerts) describes with the words it reads. Alerts can also go to a webhook or
  to Slack, set in `config.json` ([`CLI.md`](CLI.md#settings)); Settings names where they go.
- Show or hide each provider. A hidden provider still has its usage read for the `quotadial`
  command.
- Give each account a name, such as "Work", shown in the popover beside its email address and in
  the tray's menu in place of it. Accounts are listed by the name shown. Clear the name to go back
  to the email address.
- Licence: whether this copy is QuotaDial Free or QuotaDial Pro and, on Free, what Pro adds; a
  field to paste a licence key into, and a button to remove the key. A key is checked on this
  machine, with no network call, and counts at once ([`CLI.md`](CLI.md#licence)).

On QuotaDial Free the popover shows one account per provider, the one the current shell would use,
and a line under the accounts says how many more there are and that the week's cost is part of
QuotaDial Pro. An alert rule only Pro checks says it waits for Pro, and Settings says that alerts go
only to the system's notifications where it would name webhooks. What each tier has is in
[`CLI.md`](CLI.md#quotadial-free-and-pro).

While a provider's status page reports an incident, ⚠ appears beside the tray's number and the
popover names the incident under it.

Under the accounts, the popover shows what the last 7 days of Claude Code and Codex use would have
cost at each provider's API prices, read from their session logs, with each model's part. How it is
worked out, and the setting that turns it off, are in [`CLI.md`](CLI.md#the-weeks-cost).

**Help**, beside Settings, shows what is new in this version, from the changelog built into the
app; About, with the version, the licence, the credits and **Check for updates**; and what QuotaDial
reads and sends, the same text as [`PRIVACY.md`](PRIVACY.md). Send feedback opens a new GitHub issue
in your browser with what support asks for filled in, the same as `quotadial doctor` prints, so you
see all of it before you send it; it holds no token, no whole email address and no licence id. The
tray's menu has Help too, with Send Feedback, Write to Support (support@quotadial.com) and Check for
Updates.

**History**, beside Settings, charts each account's windows over the last 7 or 30 days, shows when
each account was used by hour of the day, and exports the readings as CSV to the Downloads folder.

The popover lists every Claude subscription stored on the machine, not only the one Claude Code is
signed in to now. Where QuotaDial looks for them, and the settings in `config.json` that add
folders or limit how many accounts it shows, are in [`CLI.md`](CLI.md#accounts).

## Updates

Once a day, while **Check for updates once a day** is on in Settings, the app asks GitHub whether
there is a newer QuotaDial; Help's **Check for updates** asks at any time. A newer version is
offered in the popover and in Help, with what changed in it, and the app tells you once as a
notification. Nothing is downloaded until you choose **Update**. QuotaDial then checks the download
against its signature, installs it and opens again as the new version, keeping your settings,
history and licence key. On macOS it may ask for your password if QuotaDial's folder needs it; on
Linux the deb asks through the system's password prompt.

Only releases published as QuotaDial's latest full release are offered; a pre-release is downloaded
by hand. The request carries nothing about you or your usage ([`PRIVACY.md`](PRIVACY.md)). The
`quotadial` command on its own does not update itself: install a newer build the way you installed
it. How releases are signed for the updater is in [`RELEASE.md`](RELEASE.md).

## The `quotadial` command

The app carries the same `quotadial` command as a standalone build of the CLI. While the app runs,
the command asks it for numbers instead of asking Claude or ChatGPT itself.

On macOS, the command is inside the app at
`/Applications/QuotaDial.app/Contents/MacOS/quotadial`. To type it as `quotadial` or `qd`, link it
into a folder on your `PATH`:

```sh
sudo mkdir -p /usr/local/bin
sudo ln -sf /Applications/QuotaDial.app/Contents/MacOS/quotadial /usr/local/bin/quotadial
sudo ln -sf /Applications/QuotaDial.app/Contents/MacOS/quotadial /usr/local/bin/qd
```

The links keep working when the app updates itself or you replace it with a newer build. Homebrew's
cask makes them for you. The app also carries the command's manual pages and completions, in
`Contents/Resources/man` and `Contents/Resources/completions`.

On Windows, the installer puts `quotadial.exe` beside the app, in `C:\Program Files\QuotaDial`; add
that folder to your `PATH` to type `quotadial`. On Linux, the deb installs `quotadial` in
`/usr/bin`, so it is ready to use, with its manual pages and its completions for bash, zsh and
fish. See [CLI.md](CLI.md) for what the command does.

## Build it yourself

You need Rust ([rustup](https://rustup.rs)) and Node.js 22 or later. On Linux you also need the
webview and tray libraries:

```sh
sudo apt install libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev libxdo-dev
```

Then:

```sh
cd apps/tray
npm ci
npx tauri build
```

The app and its installer are written to `apps/tray/src-tauri/target/release/bundle/`. A build you
make yourself opens without the warnings above, because your machine did not download it.
