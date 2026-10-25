# Skill preflight: the QuotaDial website

The Brain's reads before any product, search or design decision for the site (M5 item 5, #49),
on 2026-10-25 from 02:55 UTC. Each was read in full in the Brain's own context, through a Skill
invocation or by reading the file; the digest is the first 12 hex of the file's SHA-256 when read.
This records reads, not a verdict on the design.

| Skill | Read from | SHA-256 | References read | Governs |
| --- | --- | --- | --- | --- |
| brain-site-build | Skill invocation (`.claude/skills/brain-site-build/SKILL.md` in the Brain workspace) | 2ca19e910733 | | The whole pipeline and its gates |
| codus-ux-build | Skill invocation | cacf06605580 | | The quality bar, the stack toolbox, the render walk |
| impeccable | Skill text in context; references read as files | 574331ee26a4 | `reference/craft-floor.md`, `reference/new-work.md`, `reference/init.md` | PRODUCT.md, the seven worlds and the roll, the craft floor, the finish review |
| codus-ui-proof | Skill invocation | d4e3802e3780 | `reference/design-contract.schema.json`; `source-status` run | The Design Contract and the evidence |
| codus-brand-identity | Skill invocation | de370eeac413 | | The mark, type and colour reasoning |
| codus-brand-furniture | Skill invocation | 989f42e3c145 | | Favicon set, icons, share card, header and footer identity |
| codus-site-seo | Skill invocation | 708f4b355bed | `references/research.md`, `references/implementation.md` | The route map, the search study, blueprints, lanes, captures, the ship check |
| codus-copy | Skill invocation (full text) | 84a9324d022e | | Every sentence on the site |
| design-taste-frontend | Skill invocation (full text) | aa194351b246 | | Persuade-surface layout and copy tells |
| frontend-design | Skill text in context | 1608ea77fbb6 | | Brief-specific originality and the signature |
| brain-worktree-setup | Skill invocation | dc0e1c05d156 | | Lane worktrees, ports |
| brain-quadrant-delegation | Skill invocation | 27a64c29d093 | | Lane dispatch and callbacks |
| codus-openai | Skill invocation | 41c1b9c76a0b | | Generated imagery, if the direction calls for it |
| brain-launch-runbook | Skill invocation | 917f6bd79577 | | The deploy workflow; nothing is deployed until Matt says |

`ui-proof.cjs source-status` reports impeccable, frontend-design and design-taste-frontend as
user-updated copies; the Design Contract records them by their installed digests, as that skill
directs.

## Known viewports

No user viewport was supplied (no URL with `biw`/`bih`). Captures use 1440x900, 390x844 and
1920x1080.

## Where the private material lives

The search study's competitor captures and inventories, and the copy harvest's working file, stay
outside this public repository (in the Brain's workspace), as `docs/PUBLIC_SCOPE.md` keeps
competitor research private. What is committed here is QuotaDial's own plan: the route map,
blueprints, the direction and the contract.
