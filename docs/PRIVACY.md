# What QuotaDial reads and sends

QuotaDial runs on your machine. It reads the sign-ins that Claude Code and the Codex CLI already
keep, and never writes, refreshes, moves or logs them. From the CLIs' session logs it reads only
numbers: token counts and model names for the week's cost, and the Codex CLI's own record of your
limits. What you and the model wrote never leaves the machine, and QuotaDial never reads it.

For Gemini CLI, QuotaDial reads no sign-in. It only checks that Gemini CLI's sign-in file,
`oauth_creds.json` in `~/.gemini`, is there, without opening it, and reads one field of Gemini CLI's
`settings.json` beside it: the one that says whether it signs in with Google, a Gemini API key or
Vertex AI. It asks no one about Gemini CLI and sends nothing about it.

For Antigravity, QuotaDial reads no sign-in and asks no one, and only checks whether Antigravity
CLI's folder, `~/.gemini/antigravity-cli`, is there, without opening anything in it; once you add
`quotadial antigravity status-line` to Antigravity CLI's status line, it keeps what the status line
hands that command, each limit's share or amount left and its reset, the plan and the account's
address, with the time, in `antigravity.json` in QuotaDial's data folder, readable only by you.

For Copilot, which is built but in no release yet, as it waits on a check with a real GitHub
account, QuotaDial reads the sign-in GitHub CLI keeps in `hosts.yml` in its folder, such as
`~/.config/gh`, when GitHub CLI keeps it there rather than in the system's keychain, or, on macOS,
the one the Copilot CLI keeps in the Keychain item `copilot-cli`, which macOS may ask you to allow.
It never writes, refreshes, moves or logs either, and never signs in to GitHub itself.

QuotaDial connects to these, and nothing else:

- **api.anthropic.com**, Anthropic's, with your Claude Code sign-in, for each Claude account's
  limits, at most once every 5 minutes per account.
- **chatgpt.com**, OpenAI's, with your Codex CLI sign-in, for your ChatGPT/Codex limits, at most
  once every 2 minutes.
- **api.github.com**, GitHub's, with your GitHub CLI or Copilot CLI sign-in, for each Copilot
  account's limits, at most once every 5 minutes per account, and only in a build with Copilot,
  which no release is yet. It is Copilot's usage endpoint, apart from **github.com** below, which
  is only for QuotaDial's own updates.
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
stay on your machine, readable only by you. A picture made with History's Share image is drawn on
your machine and saved in your Downloads folder, and QuotaDial sends it nowhere. There are no
analytics, no crash reports and no account to make.

QuotaDial Pro is sold through Lemon Squeezy, which keeps the order. The link in your receipt opens
quotadial.com/key, which asks Lemon Squeezy whether the receipt's key is valid and gives you a
QuotaDial key, keeping and logging nothing. QuotaDial checks that key on your machine, with no
network call.
