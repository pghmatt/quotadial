# Lanes: who builds what

The Brain built the foundation on `feat/site`; three lanes build the pages on it, each in its own
worktree, branch and port, and the Brain merges them with the merge train
(`codus-site-seo/scripts/merge-train.mjs`) before the captures and the two reviews.

| Lane | Branch | Worktree | Port | Routes | Owns (and nothing else) |
| --- | --- | --- | --- | --- | --- |
| home | `site/home` | `~/ClaudeCode/QuotaDial-site-home` | 3082 | `/` | `site/app/page.tsx`, `site/components/home/`, `site/test/home.test.ts` |
| conversion | `site/conversion` | `~/ClaudeCode/QuotaDial-site-conversion` | 3083 | `/download`, `/pro`, `/key` | `site/app/download/`, `site/app/pro/`, `site/app/key/`, `site/components/conversion/`, `site/test/conversion.test.ts` |
| reference | `site/reference` | `~/ClaudeCode/QuotaDial-site-reference` | 3084 | `/privacy`, `/support`, `/changelog`, the 404 page | `site/app/privacy/`, `site/app/support/`, `site/app/changelog/`, `site/app/not-found.tsx`, `site/components/reference/`, `site/test/reference.test.ts` |

Everything else is shared and the Brain's: `app/layout.tsx`, `app/globals.css`, `components/*.tsx`,
`lib/`, `scripts/`, `public/`, the brand files in `app/`, `docs/`, `package.json` and the configs.
A lane that needs a change there asks the Brain (message_brain) and carries on; it never edits them.
Each lane's tests go in its own `site/test/<lane>.test.ts`, which `npm test` runs.

## Each lane's done report

`codus-site-seo/templates/lane-report.json`, filled from measurements on the lane's final commit,
saved outside the repository at
`~/.codus/brains/brain-mvjjpuep149/quotadial-site-private/lanes/<lane>.done.json`: the skills read in
the lane's own context, each route's H1, title, meta length, canonical, schema and links measured
from the built page, the sections present, the primary action's position at 390x844, each event
seen firing, captures at 1440x900, 390x844 and 1920x1080 with no overflow and the contrast measured,
the copy checks and the detector's output, and the commit. `sharedChanges` empty.

## Before the final walk

Merge `feat/site` into the lane's branch once, so the walk sees the foundation as it will ship.
Never push, never merge into `feat/site` or `main`: the Brain merges.
