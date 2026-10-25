# Blueprint: /support (utility)

Job: send a person with a problem to the right channel, with what support needs from them. Sources:
`SECURITY.md`, `docs/CLI.md` (Doctor), Help in the app.

## First phone viewport

H1: "Get help with QuotaDial" · Answer: the three channels in one line · Primary action: "Write to
support@quotadial.com" (`data-primary-action`, `support_mail_click`, a `mailto:` link).

## Sections

| H2 | Answer |
|---|---|
| Write to support | The address, and `quotadial doctor`, which prints what support asks for with nothing secret in it; Help in the app has Send feedback, which fills in the same |
| Report a problem on GitHub | Open an issue (`issue_click`), with the doctor output; the issue templates |
| Report a security problem | Privately, through GitHub's private vulnerability reporting or the support address with "Security" in the subject; an answer within 3 working days (`SECURITY.md`) |

Links: /privacy, /changelog. No promise beyond `SECURITY.md`'s.
Title: "Support, feedback and security reports | QuotaDial" · Meta: "Write to support@quotadial.com,
open an issue on GitHub with the details support needs, or report a security problem privately, as
SECURITY.md describes." (147)
