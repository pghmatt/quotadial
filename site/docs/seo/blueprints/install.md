# Blueprint: /install (utility)

Job: show `docs/INSTALL.md` whole, rendered at build time as /privacy renders `docs/PRIVACY.md`: every
way to install QuotaDial (the files, package managers, opening a build that is not signed yet),
its settings, its updates, the `quotadial` command and building it yourself. The site never
paraphrases the guide, and /download keeps its own job: the file for your system, first.

H1: "Installing QuotaDial" (the file's own heading). Every H2 of the file renders, in its order, with
an anchor; a test fails the build when the page and the file's headings differ. The guide is long,
so its H2s are listed as links at its head (a contents list in the reading face, never a card).
Tables stay tables; commands stay code blocks with copy buttons, as on /download. Reading lines
hold the site's measure.
Closing: "Download QuotaDial" (`data-primary-action`, `download_click`, to /download) and a link to
/cli ("Read the command guide").
Title: "Install guide: package managers, settings, updates | QuotaDial" · Meta: "How to install
QuotaDial on macOS, Windows and Linux with a download or a package manager, open a build not signed
yet, change its settings and update it." (154)
