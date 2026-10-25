# What QuotaDial reads and sends

QuotaDial runs on your machine. It reads the sign-ins that Claude Code and the Codex CLI already
keep, and never writes, refreshes, moves or logs them. From the CLIs' session logs it reads only
numbers: token counts and model names for the week's cost, and the Codex CLI's own record of your
limits. What you and the model wrote never leaves the machine, and QuotaDial never reads it.

QuotaDial connects to these, and nothing else:

- **api.anthropic.com**, with your Claude Code sign-in, for each Claude account's limits, at most
  once every 5 minutes per account.
- **chatgpt.com**, with your Codex CLI sign-in, for your ChatGPT/Codex limits, at most once every
  2 minutes.
- **status.claude.com** and **status.openai.com**, the providers' public status pages, every 5
  minutes while `status_pages` is on. These carry no sign-in.
- **github.com**, where QuotaDial's releases are, once a day while the app's "Check for updates"
  is on, or when you check yourself; and **release-assets.githubusercontent.com**, where GitHub
  keeps the files, when you choose Update. These carry no sign-in and nothing about you or your
  usage. GitHub sees your IP address, as it does for any download.
- The **webhook and Slack addresses you add** to `config.json`, if you add any. Each gets every
  alert: what fired and its numbers, never a token or where a sign-in is kept.

Help's links, to Send feedback, the changelog and the support address, open in your own browser or
mail app, and only when you click them. Send feedback fills in a GitHub issue that you read before
you send it.

The local API answers only on this machine, at 127.0.0.1, and asks for QuotaDial's own token. The
MCP server talks only to the agent that starts it. QuotaDial's history, settings and licence key
stay on your machine, readable only by you. There are no analytics, no crash reports and no
account to make.

QuotaDial Pro is sold through Lemon Squeezy, which keeps the order. The link in your receipt opens
quotadial.com/key, which asks Lemon Squeezy whether the receipt's key is valid and gives you a
QuotaDial key, keeping and logging nothing. QuotaDial checks that key on your machine, with no
network call.
