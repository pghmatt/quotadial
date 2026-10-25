# Blueprint: /download (utility)

Job: let a visitor pick the right file for their system, check it, and open a build that is not
signed yet. Source of truth: `docs/INSTALL.md` (Download, Open a build that is not signed yet).

## First phone viewport (390x844)

1. H1: "Download QuotaDial"
2. Answer: the visitor's own system first, with its file named and one action.
3. Primary action: "Download for <system>" (`data-primary-action`, `download_click` with the
   system and the file), straight to the file's version-less copy,
   `releases/latest/download/<name>`, as `docs/INSTALL.md`'s Download table links it (item 12, the
   manager's decision of 2026-10-25 05:30 UTC on #49). Linux offers both the `.deb` and the
   `.AppImage`. A visitor whose system is not known chooses it from the systems below. The build
   never fetches these addresses; the deploy waits for a full release that carries the copies.
4. Proof: the file names and where they come from (QuotaDial's GitHub releases), and that each
   release lists every file's SHA-256.

## Sections

| H2 | Answer | Notes |
|---|---|---|
| macOS | The universal `.dmg`, for Apple silicon and Intel; drag to Applications | anchor `#macos` |
| Windows | The `.msi` installer | anchor `#windows` |
| Linux | The `.deb` for Debian and Ubuntu, the `.AppImage` for the rest; GNOME needs the AppIndicator extension | anchor `#linux` |
| The command on its own | The four archives from `docs/INSTALL.md`'s "The command alone", each with its version-less link, and the one install line; each holds `quotadial` and `qd` | code, copyable |
| Check a download | `shasum -a 256 -c SHA256SUMS.txt --ignore-missing`, `sha256sum`, `Get-FileHash` | code, copyable |
| Opening a build that is not signed yet | The steps per system from `docs/INSTALL.md`, until signing is in place; only for a build from QuotaDial's GitHub | honest: today's builds are not signed |

Under the actions, the versioned file table stays, naming each file and linking to the releases page,
so a visitor is never at a dead end. Links: /install ("Read the install guide": package managers,
settings and updates), /pro ("See what Pro adds"), /support. Schema: `SoftwareApplication` (as on
home).
Title: "Download for macOS, Windows and Linux | QuotaDial" · Meta: "Download QuotaDial for macOS,
Windows or Linux from its GitHub releases, check the file against its SHA-256, and open a build
that is not signed yet." (149)
