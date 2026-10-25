# Blueprint: /cli (utility)

Job: show `docs/CLI.md` whole, rendered at build time as /privacy renders `docs/PRIVACY.md`: every
command, what it prints as text and as JSON, the daemon, alerts, history, the licence, `quotadial
doctor`, the local API, the MCP server and the exit codes. The site never paraphrases the guide.

H1: "The QuotaDial CLI" (the file's own heading). Every H2 of the file renders, in its order, with an
anchor; a test fails the build when the page and the file's headings differ. The guide is long, so
its H2s are listed as links at its head (a contents list in the reading face, never a card). Tables
stay tables; examples stay code blocks with copy buttons where they are commands to run. Each
reading line holds the site's measure; wide example output may scroll inside its own block.
Closing: "Download QuotaDial" (`data-primary-action`, `download_click`, to /download) and a link to
/install ("Read the install guide").
Title: "The quotadial command: output, API and exit codes | QuotaDial" · Meta: "Every quotadial
command, what it prints as text and as JSON, the local API and MCP server for agents, and the exit
codes scripts can rely on." (141)
