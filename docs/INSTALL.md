# Installing QuotaDial

QuotaDial's tray app runs on macOS, Windows and Linux. It puts the share used of your tightest
Claude or ChatGPT/Codex limit in the menu bar or system tray. It also runs the QuotaDial daemon,
so the `quotadial` command, the local API and the MCP server all read its numbers. The app carries
the `quotadial` command inside it.

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

QuotaDial shows a dial in the menu bar, with the share used beside it. Click the dial to open its
window. QuotaDial has no Dock icon; quit it from the menu bar.

### Windows

1. Run the installer. Edge may say it "isn't commonly downloaded": choose **...** > **Keep** >
   **Show more** > **Keep anyway**.
2. If Windows says it **protected your PC**, choose **More info**, then **Run anyway**.

QuotaDial's number appears in the notification area. If Windows tucks it under the **^** arrow,
drag it onto the taskbar to keep it in view.

### Linux

- **AppImage:** make it executable, then run it:

  ```sh
  chmod +x QuotaDial_*.AppImage
  ./QuotaDial_*.AppImage
  ```

- **Debian or Ubuntu:** `sudo apt install ./quotadial_*.deb`

The dial and number appear in the panel's tray area. GNOME needs the AppIndicator extension for
that; Ubuntu has it on already.

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

The links keep working when you replace the app with a newer build. See [CLI.md](CLI.md) for what
the command does.

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
