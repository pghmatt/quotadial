# Installing QuotaDial

QuotaDial's tray app runs on macOS, Windows and Linux. It puts the share used of your tightest
Claude or ChatGPT/Codex limit in the menu bar or system tray. It also runs the QuotaDial daemon,
so the `quotadial` command, the local API and the MCP server all read its numbers. The app carries
the `quotadial` command inside it.

## Download

Each release on the repository's [Releases page](https://github.com/pghmatt/quotadial/releases)
has these files:

| System | File |
| --- | --- |
| macOS (Apple silicon and Intel) | `QuotaDial_<version>_universal.dmg` |
| Windows | `QuotaDial_<version>_x64_en-US.msi` |
| Debian or Ubuntu | `QuotaDial_<version>_amd64.deb` |
| Other Linux | `QuotaDial_<version>_amd64.AppImage` |

The command alone, without the tray app, comes as `quotadial-<version>-<platform>.tar.gz` (or
`.zip` on Windows). Each archive holds `quotadial` and its short alias `qd`.

## Open a build that is not signed yet

Until the M5 release, QuotaDial's builds are not signed or notarized. Each operating system warns
the first time you open one. This is how to open it anyway. Only do this for a build you got from
QuotaDial's GitHub repository.

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
- Notify you when an account needs signing in again, or when a limit passes 90%. Each
  notification comes once: again only after the account has worked in between, or after the limit
  resets.
- Show or hide each provider. A hidden provider still has its usage read for the `quotadial`
  command.
- Give each account a name, such as "Work", shown in the popover beside its email address and in
  the tray's menu in place of it. Accounts are listed by the name shown. Clear the name to go back
  to the email address.

The popover lists every Claude subscription stored on the machine, not only the one Claude Code is
signed in to now. Where QuotaDial looks for them, and the settings in `config.json` that add
folders or limit how many accounts it shows, are in [`CLI.md`](CLI.md#accounts).

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

The links keep working when you replace the app with a newer build.

On Windows, the installer puts `quotadial.exe` beside the app, in `C:\Program Files\QuotaDial`; add
that folder to your `PATH` to type `quotadial`. On Linux, the deb installs `quotadial` in
`/usr/bin`, so it is ready to use. See [CLI.md](CLI.md) for what the command does.

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
