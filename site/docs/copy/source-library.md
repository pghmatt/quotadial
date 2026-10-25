# Copy source library: quotadial.com

Scope: one library for the whole site. Every page and section of quotadial.com (home, /download, /pro, /key, /privacy, /support, /changelog) takes its sentence moulds and its markers from this file, so the site reads as one voice from the first heading to the last question. Harvested on 2026-10-25 (UTC) by the structural-mould method in the codus-copy skill, before any page copy was written.

## What this file holds, and what it never holds

- It holds sentence skeletons (the grammar of a passage, with every content slot marked), the marker phrases that pass the idiom test, shape notes for everything else, attestation counts, and the archived URL of every source.
- It never holds a sentence, a clause or a distinctive phrase from any source. The verbatim quotations, and the analysis worked from them, stay in a private working file in the Brain's workspace (`quotadial-site-private/copy/harvest-working.md`), outside this repository, and are never committed. A page's age gives no licence: a 2021 snapshot is as protected as a page published this morning. The date bound buys human provenance and nothing else.
- It holds no QuotaDial page copy. The notes under each template say which kind of section the mould suits and stop there.

## How it was made

1. Twenty-eight unrelated home or product pages of small developer tools, command-line tools and menu-bar or tray utilities, none of them a tracker for Claude or Codex usage. Each page was read in its Internet Archive snapshot, never live: the CDX API listed each address's captures between 2019-01-01 and 2021-12-31 (status 200, HTML), the latest was taken, and the `id_` form of the snapshot gave the original HTML. Every snapshot predates 2022-11-01.
2. Visible text only: scripts, styles and SVG removed, hard line breaks inside paragraphs joined, image alt text kept because it is part of the page.
3. Forty templates chosen by job, from 19 of the 28 sources, no source supplying more than four, so no single voice dominates.
4. Every marker measured, never estimated: a case-insensitive phrase search (sentence-initial forms match case) over all 28 texts, a source counted once however often it repeats a phrase, and every hit read by eye to reject wrong senses. Hits inside a testimonial or press quote on a page belong to that page; no count rests on one person quoted on two sites (two reviewers do appear on two sites each, in different words).
5. The dictionary route was checked against Wiktionary's English entries on 2026-10-25, sense by sense where a single word carries the idiom. Merriam-Webster refused scripted requests and Cambridge's addresses for multi-word idioms proved unreliable, so neither is cited. The senses themselves are third-party text and live only in the private file; this file cites the entry.

## The reuse rules

These are codus-copy's rules, applied to this library. They bind every page on the site.

- No harvested sentence ships, whole or trimmed. Only its grammar does: clause order, connectors, punctuation, tense, where the subject lands.
- Every slot takes QuotaDial's own facts. No source fact survives: no product name, number, place or feature from the source. If a reader could recognise the source from the result, go again.
- A marker phrase ships verbatim only if it passes the idiom test: it appears in at least three unrelated sources here, or a dictionary carries it (one attesting source plus a dictionary entry is enough). The verdict column records which route it passed by.
- Two sources without a dictionary entry is not 'several': shape only. One source without an entry is that author's expression: shape only.
- A distinctive image never ships verbatim, whatever its count. Take its shape and build a new image in it.
- Grammar is nobody's expression. Rows marked 'word' (function words, stock connectives, single plain dictionary words) record frequency, not permission.
- Aim for at least a quarter of a finished page's markers to come from this file, weighted to verbatim idioms and device shapes.
- The banned constructions in codus-copy still apply on top of this file. Templates that sit near one (a question, a fragment, a run of three, a comparison) say so under 'Watch'.
- Visitor-facing words follow `docs/copy/jargon-terms.json`: a window, a reset, a sign-in.

## Reading a skeleton

- S1, S2 and so on are the sentences in order; 'Heading' is a heading the source pairs with them.
- `[CAPS: hint]` is a slot for QuotaDial's own content. Lower-case words outside brackets are the kept grammar.
- 'see M07' means the slot is a marker from this file, used under that marker's verdict.
- Where a source used a dash, the skeleton says so in words. This site uses commas, colons, brackets or full stops instead.

## Sources

| # | Tool | Kind | Snapshot (UTC) | Archived page |
|---|---|---|---|---|
| 1 | iStat Menus | menu-bar system monitor (Bjango) | 2021-12-02 | <https://web.archive.org/web/20211202143419id_/https://bjango.com/mac/istatmenus/> |
| 2 | Bartender | menu-bar organiser (Surtees Studios) | 2021-12-26 | <https://web.archive.org/web/20211226095114id_/https://www.macbartender.com/> |
| 3 | TripMode | menu-bar data-usage monitor | 2021-12-05 | <https://web.archive.org/web/20211205174946id_/https://tripmode.ch/> |
| 4 | Little Snitch | network monitor and firewall (Objective Development) | 2021-12-02 | <https://web.archive.org/web/20211202062659id_/https://www.obdev.at/products/littlesnitch/index.html> |
| 5 | Radio Silence | network monitor and firewall | 2021-12-24 | <https://web.archive.org/web/20211224152416id_/https://radiosilenceapp.com/> |
| 6 | Lunar | menu-bar display utility with a command line | 2021-12-21 | <https://web.archive.org/web/20211221053240id_/https://lunar.fyi/> |
| 7 | Moom | window manager in the menu bar (Many Tricks) | 2021-12-02 | <https://web.archive.org/web/20211202032956id_/https://manytricks.com/moom/> |
| 8 | Pastebot | clipboard manager (Tapbots) | 2021-12-06 | <https://web.archive.org/web/20211206120031id_/https://tapbots.com/pastebot/> |
| 9 | Maccy | open-source (MIT) clipboard manager | 2021-12-28 | <https://web.archive.org/web/20211228004002id_/https://maccy.app/> |
| 10 | Caffeine | menu-bar utility (Lighthead) | 2020-06-10 | <https://web.archive.org/web/20200610141948id_/http://lightheadsw.com/caffeine/> |
| 11 | Itsycal | menu-bar calendar (Mowglii) | 2021-11-01 | <https://web.archive.org/web/20211101022247id_/https://www.mowglii.com/itsycal/> |
| 12 | BitBar | open-source menu-bar script runner | 2020-07-25 | <https://web.archive.org/web/20200725075701id_/https://getbitbar.com/> |
| 13 | f.lux | display utility for macOS, Windows and Linux | 2021-12-02 | <https://web.archive.org/web/20211202161608id_/https://justgetflux.com/> |
| 14 | Rainmeter | open-source Windows desktop monitor | 2021-12-01 | <https://web.archive.org/web/20211201110216id_/https://www.rainmeter.net/> |
| 15 | Homebrew | package manager | 2021-12-01 | <https://web.archive.org/web/20211201022551id_/https://brew.sh/> |
| 16 | Oh My Zsh | open-source (MIT) shell framework | 2021-12-07 | <https://web.archive.org/web/20211207192612id_/https://ohmyz.sh/> |
| 17 | HTTPie | terminal HTTP client | 2020-09-16 | <https://web.archive.org/web/20200916093746id_/https://httpie.org/> |
| 18 | Mosh | command-line mobile shell | 2021-12-21 | <https://web.archive.org/web/20211221060334id_/https://mosh.org/> |
| 19 | ngrok | command-line tunnel tool | 2021-12-01 | <https://web.archive.org/web/20211201051314id_/https://ngrok.com/> |
| 20 | Volta | command-line JavaScript tool manager | 2021-12-29 | <https://web.archive.org/web/20211229025354id_/https://volta.sh/> |
| 21 | Hammerspoon | macOS automation tool | 2021-12-09 | <https://web.archive.org/web/20211209152826id_/http://www.hammerspoon.org/> |
| 22 | iTerm2 | terminal emulator | 2021-12-01 | <https://web.archive.org/web/20211201005206id_/https://iterm2.com/> |
| 23 | Sublime Merge | Git client | 2021-12-05 | <https://web.archive.org/web/20211205135206id_/https://www.sublimemerge.com/> |
| 24 | Fork | Git client | 2021-12-07 | <https://web.archive.org/web/20211207005819id_/https://git-fork.com/> |
| 25 | Tower | Git client | 2021-12-03 | <https://web.archive.org/web/20211203103227id_/https://www.git-tower.com/mac/> |
| 26 | Transmit | file transfer app (Panic) | 2021-12-04 | <https://web.archive.org/web/20211204044117id_/https://panic.com/transmit/> |
| 27 | Dash | documentation browser (Kapeli) | 2021-12-03 | <https://web.archive.org/web/20211203124459id_/https://kapeli.com/dash> |
| 28 | Alfred | launcher | 2021-12-01 | <https://web.archive.org/web/20211201060031id_/https://www.alfredapp.com/> |

Considered and left out: xbarapp.com (same author as BitBar, so one voice would count twice); rectangleapp.com (names, links and supporter lists only; no prose); getsharex.com (feature and destination lists only; no prose); coconut-flavour.com/coconutbattery/ (the archive refuses its snapshots with HTTP 403).

## Templates (40)

Spread: 19 sources, the most from any one source 4 (10%). By job: conversational_lead 9, short_punchy 8, editorial_reasoning 9, data_presentation 6, transition 6, quote_integration 2.

To use one: pick the template whose job matches the sentence you are about to write, take only its skeleton, pour QuotaDial's facts into every slot, then read the result aloud. If it sounds like the source's subject, content was copied; if it sounds like QuotaDial in someone else's rhythm, it worked.

### conversational_lead (9)

An opening that starts on the reader's situation.

#### T01 · Transmit

- Source: <https://web.archive.org/web/20211204044117id_/https://panic.com/transmit/>
- Skeleton: S1: So, you have [READER'S MATERIAL] you need to [VERB] [WHERE]. S2: Maybe you're [ROLE A]. S3: Maybe you're [ROLE B]. S4: Maybe you're [ROLE C] or [ROLE D].
- Clause map: S1 discourse opener + main clause (you have + object) + reduced relative clause (you need to + verb + place). S2 to S4 one short copular frame repeated three times (anaphora), each naming a kind of reader; the last joins two roles with 'or'.
- Connectors: So (opener); or
- Punctuation: Comma after the opener; full stops only.
- Tense: Present simple, second person.
- Fits: Opening a section on who the reader is before the product appears. Pairs with T10, which states the need and closes on the product.
- Watch: The anaphora is the page's one run of three; keep the roles concrete nouns so it never reads as a value tricolon.

#### T02 · TripMode

- Source: <https://web.archive.org/web/20211205174946id_/https://tripmode.ch/>
- Skeleton: S1: Your [MACHINE OR TOOL] doesn't [ATTITUDE VERB] whether you are [STATE] [SCENE A], [SCENE B], or [SCENE C]. S2: Plenty of [ACTORS] will [ADVERB] try to [VERB] [OBJECT] [INTENSITY IDIOM, see M11]. S3: [PRODUCT] [VERB]s that.
- Clause map: S1 personified subject + negated attitude verb + whether-clause carrying three concrete scenes. S2 quantifier subject + future modal + try to + verb + object + idiom of excess. S3 a three-word verdict whose demonstrative points back at S2.
- Connectors: whether; or; no connector between sentences (the demonstrative in S3 does the joining)
- Punctuation: Serial comma in the scene list; full stops.
- Tense: Present, then future (will), then present.
- Fits: Problem framing before the product, where the short S3 lands the product as the answer.
- Watch: S3 is the very short sentence for its five-sentence span; one per section.

#### T03 · Little Snitch

- Source: <https://web.archive.org/web/20211202062659id_/https://www.obdev.at/products/littlesnitch/index.html>
- Skeleton: S1: As soon as you're [STATE], [ACTORS] can [HEDGE ADVERB] [VERB] [DOUBLED WH-PHRASE: whatever X to wherever Y]. S2: [FREQUENCY HEDGE, see H03] they [PRO-VERB PHRASE] [FOR THE READER'S GOOD]. S3: But [FREQUENCY ADVERB], [EXAMPLE-INTRODUCING PHRASE] [EXAMPLE A], [EXAMPLE B] or [EXAMPLE C], they don't. S4: But you don't [PERCEPTION VERB] [OBJECT], because [WHOLE-OF-IT SUBJECT] happens [ADVERB] [IDIOM, see M01].
- Clause map: S1 temporal clause (as soon as) + main clause with a hedged modal + a doubled free relative. S2 frequency hedge + pro-verb (do this) + benefit phrase. S3 But + frequency adverb + inserted example list + elliptical negative. S4 But + negated perception verb + because-clause ending on an idiom.
- Connectors: As soon as; Most often; But sometimes; But; because
- Punctuation: Commas fence the inserted example list; full stops.
- Tense: Present simple.
- Fits: Explaining an invisible process honestly: concede the normal case, then the exception, then why the reader never sees it. Leads into what the product shows.
- Watch: Two sentence-initial 'But's work here because the second turns from the actors to the reader; never a third.

#### T04 · f.lux

- Source: <https://web.archive.org/web/20211202161608id_/https://justgetflux.com/>
- Skeleton: S1: During [TIME OR CONDITION], [THING] [VERB] [ADJECTIVE] (a dash in the source; cast it as a colon or a full stop) they're designed to [PURPOSE]. S2: But, at [TIME 1], [TIME 2], or [TIME 3], you probably shouldn't be [ACTION THAT ECHOES THE END OF S1].
- Clause map: S1 fronted time adverbial + main clause + appended explanation. S2 But + fronted list of three concrete times + hedged negative modal + progressive verb whose object repeats S1's last noun.
- Connectors: During; But
- Punctuation: Dash in S1 (recast); serial comma in the list of times.
- Tense: Present simple; present progressive after the modal.
- Fits: A situation lead that hinges on the hour, or the day of the week (a run started late in a five-hour window).
- Watch: The echo of S1's last noun is the joke; keep the echo, change every word around it.

#### T05 · Lunar

- Source: <https://web.archive.org/web/20211221053240id_/https://lunar.fyi/>
- Skeleton: S1: [FREQUENCY ADVERB] you only need to [MODEST GOAL], but you don't [WISH-TYPE VERB] to [COSTLY STEP A] or [COSTLY STEP B]. S2: In the case of [HARDEST CASE], [GERUND ACTION] isn't even [POSSIBLE-TYPE ADJECTIVE] [WITHOUT-EFFORT PHRASE]. S3: [FEATURE] [LETS-TYPE VERB] you [ADVERB] [VERB] [OBJECT] using a single [INPUT].
- Clause map: S1 frequency adverb + limiting 'only' + need clause, then but + negated want with two alternatives. S2 fronted case phrase + gerund subject + negative copula + 'even'. S3 feature as subject + verb + means phrase.
- Connectors: Sometimes; but; or; even
- Punctuation: One comma before 'but'; full stops.
- Tense: Present simple.
- Fits: Leading into a single feature from one narrow moment of need.
- Watch: The source drops 'of' in S2 (a slip; the skeleton restores it). Prefer 'lets you' to the source's verb, which leans towards brochure language.

#### T06 · Little Snitch

- Source: <https://web.archive.org/web/20211202062659id_/https://www.obdev.at/products/littlesnitch/index.html>
- Skeleton: S1: If you are [UNFAMILIAR-TYPE ADJECTIVE] to [PRODUCT], you may get [OVERLOAD PARTICIPLE] by [QUANTITY PHRASE] of [THINGS] from [SOURCES] [PARTICIPLE CLAUSE]. S2: [FEATURE] was [REBUILT-TYPE VERB] [IDIOM, see M16] to make [GERUND TASK] [IDIOM, see M07]. S3: It lets you [VERB A] [OBJECT] for a while, and [VERB B] [OBJECT] later, [ADVERB] with just a few [INPUTS], [IDIOM, see M17].
- Clause map: S1 conditional on the reader being unfamiliar + hedged main clause (may get + participle) + long noun phrase. S2 passive + purpose infinitive (to make X easy). S3 lets you + two verb phrases contrasting now and later + a manner tail in short comma beats.
- Connectors: If; and
- Punctuation: Commas build S3's tail in short beats.
- Tense: Present; passive past in S2.
- Fits: Speaking to a first-time user's overload, then the relief (alerts, several accounts).
- Watch: The source's launch adjective in S2 is puffery; leave it out.

#### T07 · Pastebot

- Source: <https://web.archive.org/web/20211206120031id_/https://tapbots.com/pastebot/>
- Skeleton: S1: There are [THINGS] you [VERB] [PREPOSITION] that you never want [PARTICIPLE] in [PRODUCT]. S2: Just [VERB] that [THING] to [THE SETTING] and [PRODUCT] will [VERB] it.
- Clause map: S1 existential (there are) + two stacked relative clauses. S2 softened imperative (Just) + coordinated future promise (and ... will).
- Connectors: Just; and
- Punctuation: Plain; full stops.
- Tense: Present; future in the promise.
- Fits: An edge case the reader already has, settled in one move (settings, privacy detail, a second subscription).
- Watch: None for the shape; the source's list word is one to avoid in any case.

#### T08 · iTerm2

- Source: <https://web.archive.org/web/20211201005206id_/https://iterm2.com/>
- Skeleton: S1: If you spend [AMOUNT OF TIME] in [PLACE], then you'll [APPRECIATE-TYPE VERB] all the [SMALL THINGS] that [ACCUMULATE VERB PHRASE, see M31]. S2: It is [LICENCE FACT] and you can find [WHAT] on [WHERE].
- Clause map: S1 conditional with an explicit 'then' + future main clause + relative clause. S2 copular fact + and + you can find.
- Connectors: If ... then; and
- Punctuation: One comma after the condition.
- Tense: Present; future in the main clause.
- Fits: Addressing the heavy user directly (people who live in a terminal beside Claude Code); S2 drops in the open-source fact as an aside.
- Watch: None.

#### T09 · Hammerspoon

- Source: <https://web.archive.org/web/20211209152826id_/http://www.hammerspoon.org/>
- Skeleton: S1: You might want to [ACTION A] [OBJECT], or [ALTERNATIVE]. S2: You might want to [ACTION B] when [TRIGGER B]. S3: You might want to [ACTION C] when [TRIGGER C]. S4: You might want to [ACTION VERB] [INDEFINITE PRONOUN] [PLAYFUL ADJECTIVE] like [ABSURD BUT POSSIBLE ACTION] when [UNLIKELY TRIGGER].
- Clause map: Four sentences on one hedged frame (anaphora); three carry a when-clause trigger; the fourth escalates to a deliberately silly example.
- Connectors: or; when; like
- Punctuation: Full stops; one comma before 'or' in S1.
- Tense: Modal present.
- Fits: Showing what the reader could set up (alert rules, an agent checking before it starts) without a feature list; the fourth sentence buys warmth.
- Watch: The anaphora is the device; once per page. The silly example must still be something the app can really do.

### short_punchy (8)

A fragment or a sentence under about eight words, used for emphasis, with the sentence that sets it up.

#### T10 · Transmit

- Source: <https://web.archive.org/web/20211204044117id_/https://panic.com/transmit/>
- Skeleton: S1: But you need to [CORE VERB] to a [DELIBERATELY VAGUE NOUN], and [VERB A], [VERB B], or [VERB C] the [OBJECT] on that [SAME VAGUE NOUN]. S2: That's what [PRODUCT] is for.
- Clause map: S1 But + need clause + three verbs sharing one object; the vague noun repeats. S2 a five-word closer: demonstrative + what-clause naming the product's purpose.
- Connectors: But; and; or
- Punctuation: Serial comma in the verb list; full stops.
- Tense: Present.
- Fits: Closing a reader-situation lead (after T01); the product is named only once the need is clear.
- Watch: The closer is a stock English frame; once per page.

#### T11 · Transmit

- Source: <https://web.archive.org/web/20211204044117id_/https://panic.com/transmit/>
- Skeleton: S1: [VERB A], [VERB B], and [VERB C] [OBJECT] on [QUANTITY IDIOM, see M22] [PLACES] with [an ADJECTIVE, ADJECTIVE, and ADJECTIVE] [NOUN]. S2: It's [MODEST ADVERB] [PLAIN ADJECTIVE].
- Clause map: S1 imperative with three verbs and one object, a colloquial quantity phrase and a three-adjective description. S2 a three-word understatement.
- Connectors: and
- Punctuation: Serial commas; full stops.
- Tense: Imperative; present.
- Fits: A feature line followed by a deliberately modest verdict; the understatement does the selling.
- Watch: S1 holds two lists of three. For QuotaDial keep one list and make the other a single item.

#### T12 · Hammerspoon

- Source: <https://web.archive.org/web/20211209152826id_/http://www.hammerspoon.org/>
- Skeleton: S1: [IDIOM, see M02], [PRODUCT] does [NOTHING OR VERY LITTLE: honest scope]. S2: You will need to [SETUP ACTION] in [EXACT PATH OR PLACE] using [TOOLS].
- Clause map: S1 fronted idiom + main clause with a blunt negative object. S2 future obligation (you will need to) + exact location + means phrase.
- Connectors: None.
- Punctuation: Comma after the fronted idiom; full stops.
- Tense: Present; future.
- Fits: An honest limitation without apology, then the one step that fixes it (what must be signed in first; a build that is not signed yet).
- Watch: None; this is the honest-limitation device (D19) at its smallest.

#### T13 · Caffeine

- Source: <https://web.archive.org/web/20200610141948id_/http://lightheadsw.com/caffeine/>
- Skeleton: S1: [PRODUCT] is a [SIZE ADJECTIVE] [CATEGORY NOUN] that puts [AN ELEMENT] in [EXACT PLACE] of your [BAR OR TRAY]. S2: [INPUT VERB] it to [VERB] [EFFECT A], [EFFECT B] or [EFFECT C]. S3: [INPUT VERB] it again to [REVERSE].
- Clause map: S1 copular definition + relative clause locating the thing on screen. S2 imperative + purpose infinitive with three effects. S3 the same imperative repeated with 'again' (very short).
- Connectors: or; again
- Punctuation: No serial comma in the source list; full stops.
- Tense: Present; imperative.
- Fits: Describing what the menu-bar or tray item is and what one click does.
- Watch: S3 works because it mirrors S2's verb; keep the mirror.

#### T14 · BitBar

- Source: <https://web.archive.org/web/20200725075701id_/https://getbitbar.com/>
- Skeleton: S1: The [PRODUCT] [CATEGORY NOUN] lets you [PLACE-TYPE VERB] [WHAT] from any [SOURCE A] or [SOURCE B] right in your [PLACE]. S2: And it's [INTENSIFIER] [PRICE OR LICENCE FACT].
- Clause map: S1 lets you + verb + object + the intensifier 'right' + place. S2 sentence-initial And + short copular add-on.
- Connectors: And (sentence start)
- Punctuation: Full stops.
- Tense: Present.
- Fits: A one-line definition followed by the free and open-source fact as the afterthought that matters.
- Watch: One sentence-initial 'And' per section.

#### T15 · f.lux

- Source: <https://web.archive.org/web/20211202161608id_/https://justgetflux.com/>
- Skeleton: S1: Tell [PRODUCT] [WH-CLAUSE A], and [WH-CLAUSE B]. S2: Then [IDIOM, see V08]. S3: [PRODUCT] will [HAND-OFF VERB PHRASE], [ADVERB].
- Clause map: S1 imperative + two coordinated wh-clauses (the reader's one input). S2 a four-word imperative opened by Then. S3 future promise + trailing adverb after a comma.
- Connectors: and; Then
- Punctuation: Comma before the trailing adverb.
- Tense: Imperative; future.
- Fits: Set-up sections: the one thing the reader does, then nothing more to do.
- Watch: Only where there truly is nothing more to do.

#### T16 · Oh My Zsh

- Source: <https://web.archive.org/web/20211207192612id_/https://ohmyz.sh/>
- Skeleton: S1: [PRODUCT] is [A DELIBERATELY DRY CATEGORY DEFINITION: adjectives + noun + for + gerund]. S2: [TWO-WORD SELF-VERDICT, subject dropped]. S3: [SHORT RESTART IMPERATIVE].
- Clause map: S1 a dry definition. S2 a subjectless verdict on S1. S3 an imperative restart that licenses a plainer second attempt.
- Connectors: None.
- Punctuation: Full stops.
- Tense: Present.
- Fits: A once-per-site wink where a dry definition cannot be avoided (what the MCP server is, on /support or in a question).
- Watch: Two fragments in a row: this spends the page's fragment budget.

#### T17 · Sublime Merge

- Source: <https://web.archive.org/web/20211205135206id_/https://www.sublimemerge.com/>
- Skeleton: S1: When you're [USING PRODUCT], you're [USING THE UNDERLYING THING]. S2: [VIEW-TYPE VERB] the exact [UNDERLYING ACTIONS] you're [VERB]ing, and [MOVE VERB] between [PLACE A] and [PRODUCT].
- Clause map: S1 when-clause and main clause on the same verb (an identity line under ten words). S2 imperative + relative clause + coordinated imperative of movement between two places.
- Connectors: When; and
- Punctuation: Comma after the when-clause.
- Tense: Present progressive.
- Fits: Saying the numbers are the same wherever the reader looks (menu bar, command, local API, MCP) in one identity line.
- Watch: The source's adverb in S2 is on the banned list; use no adverb.

### editorial_reasoning (9)

A judgement or an aside after a fact.

#### T18 · TripMode

- Source: <https://web.archive.org/web/20211205174946id_/https://tripmode.ch/>
- Skeleton: S1: [PRODUCT] [KEEPS-TRACK VERB] the [THINGS] you [PAST ACTION], so you don't [NEED-TYPE VERB] to [SAME VERB, echoed] [IT YOURSELF]. S2: It also [DETECTS-TYPE VERB] [CASE] and [ACTS] [ADVERB] when [TRIGGER]. S3: Frankly, you can [CONFIGURE-TYPE VERB] [IT] once and [IDIOM, see M12]. S4: That's how [ADJECTIVE] we [INTENT VERB] it to be.
- Clause map: S1 fact + so-clause that echoes the main verb as the chore the reader no longer has (D21). S2 also-addition with a when-clause. S3 candour adverb + can + once + idiom. S4 a closer stating the maker's intent.
- Connectors: so; also; and; when; Frankly; That's how
- Punctuation: Comma before 'so'; comma after 'Frankly'.
- Tense: Present.
- Fits: Reasoning after a set-up fact: why there is nothing to maintain (no new sign-in, no settings).
- Watch: S4 states intent; never let it stand in for a promise the app does not keep. The source has a doubled word in S2 (a typo).

#### T19 · Moom

- Source: <https://web.archive.org/web/20211202032956id_/https://manytricks.com/moom/>
- Skeleton: S1: With [PRODUCT], you can [ADVERB] [VERB GROUP A] [OBJECT] to [OPTION 1], [OPTION 2], or [OPTION 3]; [VERB GROUP B] [OBJECT], and [VERB GROUP C] [OBJECT] for [RESULT]. S2: Once you've [TRIED-TYPE VERB] [PRODUCT], you'll [JUDGEMENT VERB] how you [VERB] [THING] without it.
- Clause map: S1 fronted with-phrase + can + three verb groups, split by a semicolon and commas (a capability sweep). S2 Once + present perfect + future main clause + embedded how-question.
- Connectors: With; Once; without
- Punctuation: A semicolon separates the first verb group's own list from the rest.
- Tense: Present; present perfect then future in S2.
- Fits: Closing a capability paragraph with the reader's own later judgement.
- Watch: S2's frame is a stock testimonial cliché (the 'without it' shape, D27, has three sources); recast it around one concrete thing the reader stops doing.

#### T20 · Homebrew

- Source: <https://web.archive.org/web/20211201022551id_/https://brew.sh/>
- Skeleton: S1: [PASTE-TYPE VERB] that in a [PLACE A] or [PLACE B]. S2: The [THING] [TELLS-TYPE VERB] [WHAT-CLAUSE, future] and then [HOLDS-TYPE VERB] [BEFORE-CLAUSE, present].
- Clause map: S1 imperative with a demonstrative object pointing at a command shown above. S2 subject + verb + what-clause (future) + and then + second verb + before-clause; plain pronouns carry the clauses.
- Connectors: or; and then; before
- Punctuation: Full stops.
- Tense: Imperative; present with future in the what-clause.
- Fits: Reassurance straight after an instruction (download, checksum, first run, adding the key): what happens before it happens.
- Watch: Only for behaviour the app actually has.

#### T21 · Homebrew

- Source: <https://web.archive.org/web/20211201022551id_/https://brew.sh/>
- Skeleton: S1: It's all [FAMILIAR THING A] and [FAMILIAR THING B] [UNDERNEATH, see M23], so [IDIOM, see V22] with the [CERTAINTY NOUN] that you can [ADVERB] [UNDO VERB] [YOUR CHANGES] and [VERB] [UPDATES].
- Clause map: One sentence: copular fact + so + imperative + a with-the-certainty-that phrase + reassurance clause.
- Connectors: so; with the [noun] that; and
- Punctuation: One comma before 'so'.
- Tense: Present; imperative.
- Fits: Plain-files honesty: what it is built from, so the reader can experiment safely (config.json, the MIT source).
- Watch: One sentence by design (the source sets it between code samples).

#### T22 · Radio Silence

- Source: <https://web.archive.org/web/20211224152416id_/https://radiosilenceapp.com/>
- Skeleton: S1: [PRODUCT] is built on [EVIDENCE THE PRODUCT REALLY HAS]. S2: Instead of [CAUSATIVE GERUND] you [IDIOM, see V23] with [CHORE], it simply [IDIOM, see M19].
- Clause map: S1 passive basis claim. S2 fronted instead-of gerund clause naming the chore + main clause with 'simply'.
- Connectors: Instead of
- Punctuation: Comma after the fronted clause.
- Tense: Present.
- Fits: Why there is nothing to configure (the request floor is enforced in code, never a setting).
- Watch: QuotaDial has no customer feedback to cite (PRODUCT.md); S1 must name something it has (tests, the providers' own limits).

#### T23 · Maccy

- Source: <https://web.archive.org/web/20211228004002id_/https://maccy.app/>
- Skeleton: S1: [PRODUCT] [RESPECTS-TYPE VERB] your [VALUE]. S2: If your [OTHER TOOL] [DOES A CAREFUL THING], so will [PRODUCT]. S3: Everything is [VERB]ed on your [MACHINE].
- Clause map: S1 a short abstract claim. S2 conditional naming one concrete behaviour + 'so will' inversion (the product follows the other tool's lead). S3 a short passive fact that grounds S1.
- Connectors: If; so will
- Punctuation: Comma after the condition; full stops.
- Tense: Present; future in the inversion.
- Fits: Privacy: an abstract value proved at once by a behaviour and a location fact.
- Watch: S1 alone is a slogan (PRODUCT.md: privacy is a fact to show); never ship it without S2 and S3.

#### T24 · Maccy

- Source: <https://web.archive.org/web/20211228004002id_/https://maccy.app/>
- Skeleton: S1: [PRODUCT] is an [LICENCE KIND] [CATEGORY] using the [LICENCE NAME]. S2: It is and will [PERMANENCE ADVERB] be [STATUS]. S3: Its [WHAT] is available at [WHERE]. S4: You can [VERB A] or [VERB B] it [OPEN PERMISSION PHRASE].
- Clause map: S1 copular definition + participle phrase. S2 a doubled tense (present, then will + a permanence adverb). S3 an availability fact. S4 a permission clause ending on an open-ended phrase of permission.
- Connectors: and; or
- Punctuation: Full stops; four short sentences.
- Tense: Present; future in S2.
- Fits: The open-source answer (MIT) on home or /support.
- Watch: S2 is a promise about the future; phrase only what the project commits to (Free needs no key; Pro is a key in the same build).

#### T25 · Oh My Zsh

- Source: <https://web.archive.org/web/20211207192612id_/https://ohmyz.sh/>
- Skeleton: S1: [SHORT ADMISSION, first person plural]. S2: Early in [THE PROJECT'S HISTORY], we may have [BECOME-TYPE VERB] a bit too [PLAYFUL COMPOUND ADJECTIVE]. S3: We currently [SHIP-TYPE VERB] [NUMBER] [THINGS] [PARTICIPLE].
- Clause map: S1 a three-word admission. S2 fronted time phrase + hedged past (may have) + 'a bit too' + playful compound. S3 a present fact with a number that proves the admission.
- Connectors: None.
- Punctuation: Full stops.
- Tense: Future (the admission), past modal, present.
- Fits: Owning a quirk with a number attached (a number can be up to five minutes old because of the request floor).
- Watch: Self-deprecation once per page; the number must be current.

#### T26 · Mosh

- Source: <https://web.archive.org/web/20211221060334id_/https://mosh.org/>
- Skeleton: S1: [PRODUCT] is different: it [VERB]s [BENEFIT] to [ACTION A], [ACTION B], and [ACTION C]. S2: It does this [ADVERB] and works even in [HARDEST CASE] like [EXAMPLE A] and [EXAMPLE B]. S3: On a [BAD CONDITION], [UNCONFIRMED ITEMS] are [MARKED-TYPE PARTICIPLE] so you won't be [NOT-DECEIVED PARTICIPLE].
- Clause map: S1 short verdict + colon + explanation. S2 pro-verb (does this) + manner adverb + 'works even in' scope. S3 fronted condition + passive marking + so-clause about not misleading the reader.
- Connectors: colon; and; even; like; so
- Punctuation: Colon after the verdict; serial comma.
- Tense: Present; future in the so-clause.
- Fits: The honesty beat: how the app marks a number it is not sure of (its age, its source, the held 0%).
- Watch: S3 only for marking the app really does.

### data_presentation (6)

Introducing a number and saying what it means.

#### T27 · Mosh

- Source: <https://web.archive.org/web/20211221060334id_/https://mosh.org/>
- Skeleton: S1: [PRODUCT VERSION] was [VERB]ed in [MONTH YEAR]. S2: As of [LATER EVENT] in [MONTH YEAR], as far as [WHO] [KNOW-TYPE PHRASE]: in the last [SPAN], no [BAD THINGS] of any kind ([CATEGORY A] or [CATEGORY B]) have been [VERB]ed in [PRODUCT].
- Clause map: S1 a dated past fact. S2 dated as-of frame + knowledge hedge + colon + present perfect negative claim over a span, with a bracketed scope.
- Connectors: As of; as far as; colon
- Punctuation: Colon before the claim; brackets for the scope.
- Tense: Past; present perfect.
- Fits: Dated, hedged claims about what QuotaDial has never done, as of a version and a date.
- Watch: Every date and count must be real and refreshed at each release.

#### T28 · iStat Menus

- Source: <https://web.archive.org/web/20211202143419id_/https://bjango.com/mac/istatmenus/>
- Skeleton: S1: [PRODUCT] can [NOTIFY-TYPE VERB] you of [RANGE PHRASE] of [ALERT TRIGGERS], based on [LIST]. S2: This lets you be [ALERTED-TYPE PARTICIPLE] when [EVENT A], if [EVENT B], if [MEASURE] is above [NUMBER]% for more than [NUMBER] [UNIT], or [OPEN TAIL]. S3: It can even [VERB] you [SMALL DELIGHT], or [VERB] you of [SMALL DELIGHT].
- Clause map: S1 capability + list. S2 'This lets you' + a chain of conditions (when, if, if, or) with one concrete threshold buried mid-chain. S3 an even-escalation to a small human example (D22).
- Connectors: based on; when; if; or; even
- Punctuation: Commas between conditions.
- Tense: Present; present perfect inside one condition.
- Fits: Alert rules with a threshold and a duration buried mid-sentence (the bury-a-number move).
- Watch: Drop the source's two intensifiers.

#### T29 · iStat Menus

- Source: <https://web.archive.org/web/20211202143419id_/https://bjango.com/mac/istatmenus/>
- Skeleton: Heading: [EXACT NUMBER]-[UNIT] [OFFER]. S1: [GET-TYPE VERB] [PRODUCT] [IMMEDIACY ADVERB] and [USE-TYPE VERB] it for [ROUNDED AMOUNT, see H19] before [DECISION IDIOM]. S2: We think you're [FUTURE FRAME] [ADVERB] [VERB] it.
- Clause map: The exact figure sits in the heading, the rounded figure in the prose. S1 two coordinated imperatives + before-clause. S2 a hedged prediction.
- Connectors: and; before
- Punctuation: Full stops.
- Tense: Imperative; present with going-to future.
- Fits: Any number shown exactly in a heading or list and again in prose (30 days of history; a year of it).
- Watch: None.

#### T30 · Volta

- Source: <https://web.archive.org/web/20211229025354id_/https://volta.sh/>
- Skeleton: S1: No matter the [VARIABLE A], [VARIABLE B], or [VARIABLE C], one [UNIT] is all you need: [EXACT COMMAND IN A CODE SPAN].
- Clause map: Fronted concessive (no matter the + three variables) + main clause with a number word + colon + the literal command.
- Connectors: No matter; colon
- Punctuation: Serial comma; colon before the command.
- Tense: Present.
- Fits: One app or one command across macOS, Windows and Linux, with the literal command after the colon.
- Watch: The three items are what the reader varies, never a value tricolon.

#### T31 · Little Snitch

- Source: <https://web.archive.org/web/20211202062659id_/https://www.obdev.at/products/littlesnitch/index.html>
- Skeleton: Heading: [ADJECTIVE] [FEATURE NOUN]. S1: [DATA NOUN] for up to [SPAN] is now [KEPT-TYPE PARTICIPLE] [ADVERB] [IDIOM, see M06], allowing [WHAT IT LETS THE READER DO].
- Clause map: Subject with a span (for up to + time) + present passive + participle clause of consequence.
- Connectors: allowing
- Punctuation: Comma before the participle clause.
- Tense: Present passive.
- Fits: How long history is kept (30 days Free, a year Pro) and what that length lets the reader see.
- Watch: Replace the source's generic consequence with the concrete use.

#### T32 · Moom

- Source: <https://web.archive.org/web/20211202032956id_/https://manytricks.com/moom/>
- Skeleton: S1: [PRODUCT VERSION] requires [SYSTEM AND VERSION] or newer. S2: You can [TRY-TYPE VERB] it for free. S3: The [DOCUMENT] are an [MILD ADJECTIVE] read.
- Clause map: S1 a requirement with exact version numbers. S2 a short permission. S3 a wry understatement recommending a document.
- Connectors: None.
- Punctuation: Full stops.
- Tense: Present.
- Fits: The download page's requirement line, then a nudge towards the changelog.
- Watch: None.

### transition (6)

Bridging two ideas without However or Additionally.

#### T33 · Moom

- Source: <https://web.archive.org/web/20211202032956id_/https://manytricks.com/moom/>
- Skeleton: Heading: [PLAYFUL TURN ANNOUNCING A SECOND WAVE, shape only]. S1: Use [PRODUCT] as a [MODE A], as [MODE B], or as a [ADVERB] [MODE C].
- Clause map: A heading that turns to a second wave of features, then one imperative with three parallel 'as' phrases.
- Connectors: or
- Punctuation: Exclamation in the heading; serial comma.
- Tense: Imperative.
- Fits: Moving from the core (the menu-bar number) to the secondary surfaces (the command, the API, MCP).
- Watch: The source heading is a stock catchphrase without a dictionary entry: reuse its shape (a playful turn), never its words.

#### T34 · Transmit

- Source: <https://web.archive.org/web/20211204044117id_/https://panic.com/transmit/>
- Skeleton: S1: Now, [DISTANT-PAST PHRASE] we'd [NAME-TYPE VERB] [PRODUCT] a "[OLD CATEGORY]", but today, with [CURRENT VERSION], we [VERB] [WIDER SCOPE]. S2: For example, [OUTSIDE CHANGE] [CHANGED-TYPE VERB] the way [THINGS] are [VERB]ed, and [PRODUCT] is a [ADJECTIVE] way to [VERB] your [THINGS].
- Clause map: S1 a then-and-now contrast inside one sentence (a distant-past phrase ... but today, with ...). S2 For example + a change in the world + and + the product's fit.
- Connectors: Now; [distant past] ... but today; with; For example; and
- Punctuation: Commas around 'with [CURRENT VERSION]'.
- Tense: Past habitual (would); present.
- Fits: Explaining a scope that grew (from a menu-bar meter to a command, a local API and MCP).
- Watch: Only for real history.

#### T35 · Lunar

- Source: <https://web.archive.org/web/20211221053240id_/https://lunar.fyi/>
- Skeleton: S1: If you're [SITUATION A] or [SITUATION B], [OPTION 1] can be a [ADJECTIVE] alternative because [OPTION 2] is not available without [REQUIREMENT]. S2: If you still need [GOAL], [OPTION 3] would be a [COMPARATIVE] choice.
- Clause map: Two conditionals in a row. S1 names an alternative with a because-reason. S2 narrows with 'still' and recommends a third option.
- Connectors: If; because; If ... still
- Punctuation: Comma after each condition.
- Tense: Present; would in S2.
- Fits: Routing the reader between Free and Pro, or between systems, with the reason stated.
- Watch: None.

#### T36 · Oh My Zsh

- Source: <https://web.archive.org/web/20211207192612id_/https://ohmyz.sh/>
- Skeleton: S1: Not ready to [IDIOM, see V03]? S2: We're not [MILD FEELING ADJECTIVE]; it's never a [ADJECTIVE] idea to [CAREFUL ALTERNATIVE] first.
- Clause map: S1 a subjectless question to the hesitant reader. S2 reassurance + semicolon + a general maxim recommending the slower path.
- Connectors: semicolon; first
- Punctuation: Question mark; semicolon.
- Tense: Present.
- Fits: Under the main download action, routing hesitant readers to docs/PRIVACY.md or the install guide.
- Watch: A question, so mid-page and after the main action only; never a page or section opener.

#### T37 · iStat Menus

- Source: <https://web.archive.org/web/20211202143419id_/https://bjango.com/mac/istatmenus/>
- Skeleton: S1: [ADVERB] glance at [SUMMARY] in your [BAR OR TRAY], or [IDIOM, see V25] [DETAILED VIEW], covering [LONG CONCRETE LIST] and more. S2: [PRODUCT] can show [THING] based on [DEFAULT], or for [HEDGE, see H04] any [ALTERNATIVE].
- Clause map: S1 two imperatives joined by 'or', moving from shallow (glance) to deep (dive into); the second carries a long concrete list. S2 capability + based on + or + a hedged range.
- Connectors: or; covering; based on
- Punctuation: Commas through the long list.
- Tense: Imperative; present.
- Fits: The two depths of QuotaDial: the number shown in its menu-bar or tray item, and the full window with every reset and pace.
- Watch: Drop the source's intensifier.

#### T38 · Mosh

- Source: <https://web.archive.org/web/20211221060334id_/https://mosh.org/>
- Skeleton: S1: [PRODUCT] doesn't [VERB A] or [VERB B]. S2: The [COMPONENT] [VERB]s via [EXISTING TOOL], and [READERS] [USE-TYPE VERB] the same [THING] ([EXAMPLES]) as before. S3: Then [PRODUCT] [VERB]s [WHAT] and [VERB]s [HOW].
- Clause map: S1 restraint: two things it does not do. S2 hand-off to a tool the reader already has + 'the same ... as before' with bracketed examples. S3 Then + the product's own steps.
- Connectors: or; via; and; as before; Then
- Punctuation: Brackets for the examples.
- Tense: Present.
- Fits: The no-new-sign-in section: what QuotaDial does not do, what it relies on that the reader already has, then what it does.
- Watch: In visitor copy the reused thing is a 'sign-in'; the source's own noun is one of the words the project bans.

### quote_integration (2)

A quotation woven into a sentence.

#### T39 · Radio Silence

- Source: <https://web.archive.org/web/20211224152416id_/https://radiosilenceapp.com/>
- Skeleton: S1: "[QUOTED PHRASE FROM A NAMED SOURCE]" is a [ADJECTIVE] description of [PRODUCT]. Attribution: [NAME], on its own line.
- Clause map: The quoted fragment is the sentence's subject; a copular predicate judges it; the attribution follows on its own line.
- Connectors: None.
- Punctuation: Quotation marks around the fragment only.
- Tense: Present.
- Fits: QuotaDial has no testimonials and invents none (PRODUCT.md). Hold this for quoting real text, such as a phrase from docs/PRIVACY.md or a provider's own documentation, attributed to it.
- Watch: Never a quote nobody said.

#### T40 · Mosh

- Source: <https://web.archive.org/web/20211221060334id_/https://mosh.org/>
- Skeleton: Quote: "[A REAL, EXASPERATED REMARK]". Caption: [a dash in the source; cast as a caption line] actual [KIND OF SOURCE] from [WHERE]. S1: (Why you should [CONFIDENCE VERB] [PRODUCT] with [YOUR NEED]: we [CARE-TYPE VERB] about [THINGS] so [ADJECTIVE], even [EXPERTS] don't want to [VERB] about them.)
- Clause map: A quoted complaint, a caption naming its genuine source, then a bracketed sentence that turns the complaint into a reason to trust: why-fragment + colon + so ... even ... clause.
- Connectors: colon; so; even
- Punctuation: Quotation marks; caption line; the whole reasoning in brackets.
- Tense: Present.
- Fits: Turning a real hard edge into a trust argument (a provider's limit on how often it can be asked, as the reason QuotaDial asks at most every five minutes). Only with a real, citable quote.
- Watch: Without a real quote, use the bracketed reasoning shape alone.

## Markers (163)

Verdicts: 'idiom, verbatim' passed the idiom test (the bracket gives the route: a count of three or more sources, or a dictionary entry); 'word, verbatim' is grammar or a single plain dictionary word; 'shape only' means reuse the shape and never the words; 'device shape' is a rhetorical device, structural by nature, reused as a shape; 'do not use' is attested in the sources but untrue for QuotaDial or against PRODUCT.md. Sources is the number of unrelated sources that attest the marker.

### Metaphors and figurative phrases (49)

Highest priority: the strongest single human signal. Verbatim only where the verdict says so.

| ID | Marker | Verdict | Sources | Attested in | Dictionary | Note |
|---|---|---|---|---|---|---|
| M01 | under the hood | idiom, verbatim (dictionary) | 1 | Little Snitch | [under the hood](https://en.wiktionary.org/wiki/under_the_hood) | For what happens out of sight (the floor on requests, the held 0%). |
| M02 | out of the box | idiom, verbatim (3 sources) | 3 | Lunar, Hammerspoon, Sublime Merge | [out of the box](https://en.wiktionary.org/wiki/out_of_the_box) | Honest scope before set-up (T12); also what works with no settings. |
| M03 | keep an eye on | idiom, verbatim (dictionary) | 2 | TripMode, Little Snitch | [keep an eye on](https://en.wiktionary.org/wiki/keep_an_eye_on) | The reader watching a window, or the app watching one for them. |
| M04 | at a glance | idiom, verbatim (dictionary) | 2 | iStat Menus, TripMode | [at a glance](https://en.wiktionary.org/wiki/at_a_glance) | The menu-bar number. One source writes it with 'in'; use 'at'. |
| M05 | behind the scenes | idiom, verbatim (dictionary) | 2 | Radio Silence, Mosh | [behind the scenes](https://en.wiktionary.org/wiki/behind_the_scenes) | What the background part does; never to hint at anything hidden from the reader. |
| M06 | in the background | idiom, verbatim (dictionary) | 2 | Little Snitch, Mosh | [background](https://en.wiktionary.org/wiki/background) | The jargon list's plain replacement for 'daemon'; the dictionary route is the computing sense of 'background'. |
| M07 | a breeze / breeze through | idiom, verbatim (dictionary) | 2 | Little Snitch, Sublime Merge | [breeze through](https://en.wiktionary.org/wiki/breeze_through) | Easy set-up; spend once per page. |
| M08 | a snap (an easy task) | idiom, verbatim (dictionary) | 2 | Moom, Alfred | [snap](https://en.wiktionary.org/wiki/snap) | Same job as M07; pick one per page. |
| M09 | phone home | idiom, verbatim (dictionary) | 1 | Radio Silence | [phone home](https://en.wiktionary.org/wiki/phone_home) | Exactly the privacy point: what QuotaDial does not do (no analytics, no crash reports). |
| M10 | at your fingertips | idiom, verbatim (dictionary) | 1 | Alfred | [at one's fingertips](https://en.wiktionary.org/wiki/at_one%27s_fingertips) | Worn; only with a concrete object (the reset time, the command). |
| M11 | like there's no tomorrow | idiom, verbatim (dictionary) | 1 | TripMode | [like there's no tomorrow](https://en.wiktionary.org/wiki/like_there%27s_no_tomorrow) | Fits a long agent run burning through a window. |
| M12 | never look back | idiom, verbatim (dictionary) | 1 | TripMode | [look back](https://en.wiktionary.org/wiki/look_back) | After a one-time set-up step (T18). The dictionary entry is 'look back', in its sense of regret. |
| M13 | hassle-free | idiom, verbatim (dictionary) | 2 | Radio Silence, Volta | [hassle-free](https://en.wiktionary.org/wiki/hassle-free) | Prefer saying what the hassle was. |
| M14 | painless (easy) | idiom, verbatim (dictionary) | 2 | HTTPie, Sublime Merge | [painless](https://en.wiktionary.org/wiki/painless) | For a step, never the whole product. |
| M15 | out of the way | idiom, verbatim (dictionary) | 1 | Maccy | [out of the way](https://en.wiktionary.org/wiki/out_of_the_way) | Fits how quiet the menu-bar item is. |
| M16 | from the ground up | idiom, verbatim (dictionary) | 1 | Little Snitch | [from the ground up](https://en.wiktionary.org/wiki/from_the_ground_up) | Only for something truly built that way (one codebase for three systems). |
| M17 | in one go | idiom, verbatim (dictionary) | 1 | Little Snitch | [in one go](https://en.wiktionary.org/wiki/in_one_go) | British register; several accounts read at once. |
| M18 | down the road | idiom, verbatim (dictionary) | 1 | Mosh | [down the road](https://en.wiktionary.org/wiki/down_the_road) | For a later benefit; never to promise future features (PRODUCT.md labels future work as future). |
| M19 | does the right thing | idiom, verbatim (3 sources) | 3 | Radio Silence, HTTPie, Mosh | [do the right thing](https://en.wiktionary.org/wiki/do_the_right_thing) | Three sources plus the dictionary; name the right thing in the next clause. |
| M20 | get the job done | idiom, verbatim (dictionary) | 1 | Maccy | [get the job done](https://en.wiktionary.org/wiki/get_the_job_done) | Plain verdict; FAQ register. |
| M21 | one click / a single click | idiom, verbatim (6 sources) | 6 | Little Snitch, Radio Silence, Moom, ngrok, Tower, Transmit |  | Six sources; only where it is literally one click. |
| M22 | lots of / tons of | idiom, verbatim (4 sources) | 4 | Lunar, Hammerspoon, Transmit, Alfred | [lot](https://en.wiktionary.org/wiki/lot) | Colloquial quantity ('loads of' is the British variant one source uses); never for counts QuotaDial lacks. |
| M23 | underneath (what a thing is built on) | word, verbatim (dictionary) | 2 | Homebrew, Mosh | [underneath](https://en.wiktionary.org/wiki/underneath) | T21's frame: plain files underneath. |
| M24 | at hand | idiom, verbatim (dictionary) | 1 | Maccy | [at hand](https://en.wiktionary.org/wiki/at_hand) | The numbers kept within reach, one click from the menu bar or the tray. |
| M25 | no strings attached | idiom, verbatim (dictionary) | 1 | Alfred | [no strings attached](https://en.wiktionary.org/wiki/no_strings_attached) | Free needs no key and no account: a true use. |
| M26 | see for yourself | idiom, verbatim (dictionary) | 1 | Alfred | [see for oneself](https://en.wiktionary.org/wiki/see_for_oneself) | Beside the source code or docs/PRIVACY.md, where the reader really can check. |
| M27 | over and over | idiom, verbatim (dictionary) | 1 | Alfred | [over and over](https://en.wiktionary.org/wiki/over_and_over) | The repeated chore QuotaDial removes. |
| M28 | in the long run | idiom, verbatim (dictionary) | 1 | Alfred | [in the long run](https://en.wiktionary.org/wiki/in_the_long_run) | History (30 days, a year) and what it shows over time. |
| M29 | time will tell | idiom, verbatim (dictionary) | 1 | Mosh | [time will tell](https://en.wiktionary.org/wiki/time_will_tell) | Honest uncertainty about a future reset or a provider change. |
| M30 | in sync | idiom, verbatim (dictionary) | 2 | Lunar, Alfred | [in sync](https://en.wiktionary.org/wiki/in_sync) | The same numbers in the menu bar, the command, the API and MCP. |
| M31 | add up | idiom, verbatim (dictionary) | 1 | iTerm2 | [add up](https://en.wiktionary.org/wiki/add_up) | Small uses that accumulate in a five-hour window. |
| M32 | up to date / out of date | idiom, verbatim (dictionary) | 2 | Mosh, Dash | [up to date](https://en.wiktionary.org/wiki/up_to_date) | Central to 'honest numbers': every number says how old it is. |
| M33 | on the horizon | idiom, verbatim (dictionary) | 1 | iStat Menus | [on the horizon](https://en.wiktionary.org/wiki/on_the_horizon) | A reset or a run-out that is coming; fits alerts. |
| M34 | on your watch | idiom, verbatim (dictionary) | 1 | Radio Silence | [on one's watch](https://en.wiktionary.org/wiki/on_one%27s_watch) | Chiefly US idiom; at most once on the site. |
| M35 | in control / take control | idiom, verbatim (3 sources) | 3 | Bartender, TripMode, Little Snitch | [take control](https://en.wiktionary.org/wiki/take_control) | Generic in marketing; tie it to one concrete control. |
| M36 | you're the boss | idiom, verbatim (dictionary) | 1 | Alfred | [you're the boss](https://en.wiktionary.org/wiki/you%27re_the_boss) | Hands a decision to the reader (which window to show in the menu bar). |
| M37 | Shape: Distance measured as one small input: the feature is said to be a single small action away. | shape only | 2 | iStat Menus, Pastebot |  | Two sources, no dictionary entry: build a new image in the shape. |
| M38 | Shape: Availability timed to the reader's need: present at the moment of need, unseen otherwise. | shape only | 2 | Bartender, Pastebot |  | Two sources, no dictionary entry. |
| M39 | Shape: A resource-plus-appetite compound for greedy software (the dictionary carries the pattern through other compounds; the harvested one is one author's coinage). | shape only | 1 | TripMode |  | If coined, the resource word must pass the jargon list. |
| M40 | Shape: A vendor's server cast as a distant home base that apps secretly report back to. | shape only | 1 | Radio Silence |  | One source (a press quote). |
| M41 | Shape: A tagline claiming to impose order on mess, with a founding year. | shape only | 1 | Moom |  | One source. |
| M42 | Shape: A tool cast as a window onto a hidden world the reader could not otherwise see. | shape only | 1 | Little Snitch |  | One source; for QuotaDial the hidden world is the provider's limits. |
| M43 | Shape: Software cast as a thin span joining two systems. | shape only | 1 | Hammerspoon |  | One source; fits the MCP server between an agent and the numbers. |
| M44 | Shape: A comfort object that guards the reader against small slips. | shape only | 1 | Maccy |  | One source (a user quote). |
| M45 | Shape: A magic helper that appears from a container whenever the tool is used. | shape only | 1 | Oh My Zsh |  | One source (a user quote). |
| M46 | Shape: A new version cast as a vehicle that has driven into the future. | shape only | 1 | Transmit |  | One source. |
| M47 | Shape: Improvement said to reach every part of a place. | shape only | 1 | Transmit |  | One source, no entry. |
| M48 | Shape: An essence statement: 'at the heart of it, the product is only X'. | shape only | 1 | Hammerspoon |  | One source; no dictionary entry found for the harvested wording. |
| M49 | Shape: Doing exactly the one job asked, said as a plain verdict on the product. | shape only | 2 | Radio Silence, Maccy |  | Two sources for the wording; recast in new words. |

### Rhetorical devices (30)

Structural by nature: reuse the shape, never a source's words. Counts show how widely each shape is attested.

| ID | Marker | Verdict | Sources | Attested in | Dictionary | Note |
|---|---|---|---|---|---|---|
| D01 | Shape: The reader's own question as a heading, in the first person (the how-do-I and why-would-I-bother kind), answered in the paragraph below. | device shape | 5 | Bartender, Maccy, Mosh, Hammerspoon, iTerm2 |  | Fits the home page's demanded questions. |
| D02 | Shape: A short question in the reader's voice mid-section (want, need, looking for, not ready, got), answered at once by an imperative or a reassurance. | device shape | 9 | TripMode, Little Snitch, Moom, f.lux, Rainmeter, Oh My Zsh, iTerm2, Sublime Merge, Transmit |  | Never as a page or section opener (the banned rhetorical-question opener). |
| D03 | Shape: A pun on the product's own verb or category, split across a heading and its answer. | device shape | 3 | Moom, Oh My Zsh, Mosh |  | Once per site at most. |
| D04 | Shape: Owning up: a short admission in the first person, then the fact that explains it. | device shape | 4 | Lunar, Oh My Zsh, Mosh, Transmit |  | Fits unsigned builds and the request floor. |
| D05 | Shape: Understatement straight after a strong fact: a modest adverb plus a plain adjective. | device shape | 3 | Moom, Mosh, Transmit |  | Sells by restraint; once per page. |
| D06 | Shape: A long sentence followed by a verdict of two to four words that points back at it. | device shape | 3 | TripMode, Mosh, Transmit |  | Supplies the very short sentence each five-sentence span needs. |
| D07 | Shape: Three concrete scenes, places or times inside one sentence (never three adjectives). | device shape | 3 | TripMode, f.lux, Mosh |  | Concrete nouns keep it clear of the banned value tricolon. |
| D08 | Shape: Anaphora: three or four sentences opening on the same short frame, the last one escalating. | device shape | 3 | Mosh, Hammerspoon, Transmit |  | Once per page. |
| D09 | Shape: Personifying the machine, its apps or nature: they care, refuse, inform gently, misbehave, guide. | device shape | 6 | TripMode, Little Snitch, Radio Silence, Lunar, Mosh, Fork |  | Fits an agent run or a provider's limit. |
| D10 | Shape: Turning aside to one group of readers by name, with a reassurance or a nudge. | device shape | 2 | Moom, Oh My Zsh |  | Fits keyboard and screen-reader users, or Linux users. |
| D11 | Shape: A bracketed aside in a second, drier voice that clarifies or winks. | device shape | 5 | Lunar, BitBar, Homebrew, Mosh, Transmit |  | Fits naming the tray beside the menu bar, which differs by system. |
| D12 | Shape: Quoting the old chore in the reader's words, then retiring it in two words. | device shape | 1 | Homebrew |  | One source: shape only. |
| D13 | Shape: Addressing an object directly as a heading (apostrophe). | device shape | 1 | Little Snitch |  | One source: shape only. |
| D14 | Shape: A comic fantasy of the product's effect, undercut by a mundane alternative. | device shape | 1 | Oh My Zsh |  | One source: shape only. |
| D15 | Shape: Paradox or chiasmus in a heading or a line. | device shape | 3 | Little Snitch, iTerm2, Fork |  | Close to the banned antithesis family; at most once per site. |
| D16 | Shape: The restraint promise: what the product will not do, said as plainly as what it does. | device shape | 5 | Little Snitch, Radio Silence, Maccy, Homebrew, Mosh |  | The core of QuotaDial's sign-in and privacy sections; every restraint must be true in code. |
| D17 | Shape: A conditional lead naming the reader's situation (if you ...), then what the product does for that reader. | device shape | 14 | iStat Menus, Bartender, Little Snitch, Radio Silence, Lunar, Moom, Maccy, Itsycal, Oh My Zsh, HTTPie, Mosh, Hammerspoon, iTerm2, Transmit |  | The most widely attested lead shape in the harvest. |
| D18 | Shape: A closing 'that is what, how or why' sentence after the point is made. | device shape | 4 | TripMode, Maccy, HTTPie, Transmit |  | Once per page. |
| D19 | Shape: An honest limitation stated flat, then the fix or the workaround. | device shape | 5 | Bartender, Lunar, Mosh, Hammerspoon, Transmit |  | Fits builds that are not signed yet, and the providers' floors. |
| D20 | Shape: An infomercial aside that breaks the fourth wall mid-page. | device shape | 2 | Moom, Oh My Zsh |  | Two sources; once per site. |
| D21 | Shape: Echo the verb: what the product does becomes the chore the reader no longer has, in the same verb. | device shape | 1 | TripMode |  | One source; shape only (T18). |
| D22 | Shape: Even-escalation: a last, smaller, human capability introduced with 'even'. | device shape | 7 | iStat Menus, Lunar, Moom, Pastebot, Caffeine, Mosh, Dash |  | Seven sources. |
| D23 | Shape: A one-time set-up step, then permission to stop thinking about it, then the product carrying on alone. | device shape | 3 | TripMode, Lunar, f.lux |  | Three sources (T15). |
| D24 | Shape: A dated, hedged track record: as of a version and a date, as far as anyone knows, never a given bad thing. | device shape | 1 | Mosh |  | One source (T27). |
| D25 | Shape: A restraint fragment: 'No' plus a thing the product does not have or do. | device shape | 5 | iStat Menus, Little Snitch, Radio Silence, Maccy, Mosh |  | Fragment budget applies; never three in a row. |
| D26 | Shape: The hand-off clause: the reader does one step and the product does the remainder. | device shape | 2 | f.lux, Dash |  | Two sources. |
| D27 | Shape: The 'without it' judgement: the reader later cannot picture working without the product. | device shape | 3 | Moom, Maccy, Oh My Zsh |  | Three sources and a cliché; recast around one concrete loss. |
| D28 | Shape: An imperative promise that a named chore never happens again. | device shape | 2 | TripMode, Transmit |  | Two sources; only for a chore that truly ends. |
| D29 | Shape: A task said to be as simple as a smaller, familiar task. | device shape | 3 | Lunar, Moom, Homebrew |  | Three sources. |
| D30 | Shape: A bare verdict that the step is easy, straight after a how-to. | device shape | 4 | TripMode, Little Snitch, f.lux, Sublime Merge |  | Four sources; a claim, so show the steps first. |

### Editorial connectors (22)

How the sources join clauses. Single function words and stock connectives are grammar.

| ID | Marker | Verdict | Sources | Attested in | Dictionary | Note |
|---|---|---|---|---|---|---|
| C01 | But (opening a sentence) | word, verbatim (5 sources) | 5 | Little Snitch, Moom, f.lux, Mosh, Transmit |  | Never three sentence-initial 'But's in a row. |
| C02 | And (opening a sentence) | word, verbatim (5 sources) | 5 | Moom, BitBar, Mosh, ngrok, Transmit |  | One per section. |
| C03 | Just (softening an imperative) | word, verbatim (6 sources) | 6 | Bartender, TripMode, Lunar, Moom, Pastebot, Maccy |  | Only for a step that is genuinely small. |
| C04 | Unlike [X], | word, verbatim (3 sources) | 3 | Lunar, Maccy, Mosh |  | A cousin of the banned 'where others do X'; only to correct a belief the reader holds, once per page. |
| C05 | Whenever | word, verbatim (3 sources) | 3 | Little Snitch, Lunar, Mosh |  | Fits alert rules. |
| C06 | even if / even though / even when | word, verbatim (3 sources) | 3 | TripMode, Lunar, Mosh |  |  |
| C07 | Plus, | word, verbatim (dictionary) | 2 | iStat Menus, Transmit | [plus](https://en.wiktionary.org/wiki/plus) | Colloquial addition after a list. |
| C08 | So, (opening a sentence) | word, verbatim (dictionary) | 2 | Mosh, Transmit | [so](https://en.wiktionary.org/wiki/so) | Conversational opener (T01). |
| C09 | As well as [X], | word, verbatim (dictionary) | 2 | iStat Menus, ngrok | [as well as](https://en.wiktionary.org/wiki/as_well_as) |  |
| C10 | Once [you have done X], | word, verbatim (dictionary) | 2 | Moom, Oh My Zsh | [once](https://en.wiktionary.org/wiki/once) |  |
| C11 | Then (opening a sentence) | word, verbatim (dictionary) | 2 | f.lux, Mosh | [then](https://en.wiktionary.org/wiki/then) | Set-up sequences (T15, T38). |
| C12 | that means / this means | word, verbatim (grammar) | 2 | Lunar, Mosh |  | Editorial reasoning after a number. |
| C13 | Instead of [the chore], | word, verbatim (dictionary) | 1 | Radio Silence | [instead of](https://en.wiktionary.org/wiki/instead_of) | T22. |
| C14 | Frankly, | word, verbatim (dictionary) | 1 | TripMode | [frankly](https://en.wiktionary.org/wiki/frankly) | Candour; once per site. |
| C15 | Actually, | word, verbatim (dictionary) | 1 | Moom | [actually](https://en.wiktionary.org/wiki/actually) | Self-correction after a pun or a heading. |
| C16 | Alternatively, | word, verbatim (dictionary) | 1 | Alfred | [alternatively](https://en.wiktionary.org/wiki/alternatively) |  |
| C17 | in other words | idiom, verbatim (dictionary) | 1 | Mosh | [in other words](https://en.wiktionary.org/wiki/in_other_words) | Restating jargon plainly. |
| C18 | For example, | idiom, verbatim (dictionary) | 1 | Transmit | [for example](https://en.wiktionary.org/wiki/for_example) |  |
| C19 | Now, (a turn in time) | word, verbatim (dictionary) | 1 | Transmit | [now](https://en.wiktionary.org/wiki/now) | T34. |
| C20 | Shape: A two-word plain-summary signal before a one-line restatement of the whole product. | shape only | 1 | Transmit |  | One source, no dictionary entry for the harvested wording. |
| C21 | Shape: A concessive 'and' plus an affirmative, before naming what still works as before. | shape only | 1 | Transmit |  | One source. |
| C22 | In addition, | word, verbatim (dictionary) | 2 | Moom, Mosh | [in addition](https://en.wiktionary.org/wiki/in_addition) | Discouraged: it sits next to the banned opener 'Additionally'. |

### Colloquial verbs (27)

The plain verb in place of a precise noun phrase.

| ID | Marker | Verdict | Sources | Attested in | Dictionary | Note |
|---|---|---|---|---|---|---|
| V01 | check out | idiom, verbatim (6 sources) | 6 | Lunar, Maccy, Rainmeter, Oh My Zsh, Hammerspoon, iTerm2 | [check out](https://en.wiktionary.org/wiki/check_out) |  |
| V02 | take care of | idiom, verbatim (4 sources) | 4 | iStat Menus, Radio Silence, Mosh, Dash | [take care of](https://en.wiktionary.org/wiki/take_care_of) | Pairs with the hand-off shape (D26). |
| V03 | jump in / jump right in / jump on board | idiom, verbatim (3 sources) | 3 | Rainmeter, Oh My Zsh, Alfred | [jump in](https://en.wiktionary.org/wiki/jump_in) | One source's 'jump to' is literal navigation and is not counted. |
| V04 | set up | idiom, verbatim (4 sources) | 4 | iStat Menus, TripMode, Moom, Mosh | [set up](https://en.wiktionary.org/wiki/set_up) |  |
| V05 | try it / try it out | idiom, verbatim (5 sources) | 5 | TripMode, Moom, Oh My Zsh, Volta, Transmit | [try out](https://en.wiktionary.org/wiki/try_out) |  |
| V06 | hit (a key or a button) | word, verbatim (3 sources) | 3 | Maccy, Mosh, Fork | [hit](https://en.wiktionary.org/wiki/hit) |  |
| V07 | work around | idiom, verbatim (3 sources) | 3 | Lunar, HTTPie, Mosh | [work around](https://en.wiktionary.org/wiki/work_around) | Developer register; unsigned builds. |
| V08 | forget about it / forget about [X] | idiom, verbatim (3 sources) | 3 | TripMode, Lunar, f.lux | [forget about it](https://en.wiktionary.org/wiki/forget_about_it) | D23. |
| V09 | spot (notice) | word, verbatim (dictionary) | 2 | TripMode, Fork | [spot](https://en.wiktionary.org/wiki/spot) | Spotting the window that will run out first. |
| V10 | figure out | idiom, verbatim (dictionary) | 2 | iStat Menus, Little Snitch | [figure out](https://en.wiktionary.org/wiki/figure_out) |  |
| V11 | drop (a connection drops; a level drops below) | word, verbatim (dictionary) | 2 | Mosh, Hammerspoon | [drop](https://en.wiktionary.org/wiki/drop) |  |
| V12 | have a look / take a look | idiom, verbatim (dictionary) | 2 | BitBar, Alfred | [have a look](https://en.wiktionary.org/wiki/have_a_look) |  |
| V13 | find out | idiom, verbatim (dictionary) | 2 | Hammerspoon, Sublime Merge | [find out](https://en.wiktionary.org/wiki/find_out) |  |
| V14 | grab | word, verbatim (dictionary) | 1 | Moom | [grab](https://en.wiktionary.org/wiki/grab) | An imperative for a download step. |
| V15 | pop up | idiom, verbatim (dictionary) | 1 | Alfred | [pop up](https://en.wiktionary.org/wiki/pop_up) | Alerts as the system's notifications. |
| V16 | tweak | word, verbatim (dictionary) | 1 | Transmit | [tweak](https://en.wiktionary.org/wiki/tweak) | Alert rules in config.json. |
| V17 | cut down (on) | idiom, verbatim (dictionary) | 1 | Alfred | [cut down](https://en.wiktionary.org/wiki/cut_down) |  |
| V18 | ditch | word, verbatim (dictionary) | 1 | HTTPie | [ditch](https://en.wiktionary.org/wiki/ditch) | For a habit the reader drops. |
| V19 | track down | idiom, verbatim (dictionary) | 1 | Mosh | [track down](https://en.wiktionary.org/wiki/track_down) |  |
| V20 | show off | idiom, verbatim (dictionary) | 1 | Oh My Zsh | [show off](https://en.wiktionary.org/wiki/show_off) |  |
| V21 | keep on top of | idiom, verbatim (dictionary) | 1 | iStat Menus | [on top of](https://en.wiktionary.org/wiki/on_top_of) | Fits windows and their resets. |
| V22 | hack away (at) | idiom, verbatim (dictionary) | 1 | Homebrew | [hack away](https://en.wiktionary.org/wiki/hack_away) | T21; developer register. |
| V23 | twiddle (with) | word, verbatim (dictionary) | 1 | Radio Silence | [twiddle](https://en.wiktionary.org/wiki/twiddle) | Fits the point that the request floor is enforced in code, never a setting. |
| V24 | fill up | idiom, verbatim (dictionary) | 1 | Mosh | [fill up](https://en.wiktionary.org/wiki/fill_up) | Fits pace. |
| V25 | dive into / dive in | idiom, verbatim (dictionary) | 1 | iStat Menus | [dive in](https://en.wiktionary.org/wiki/dive_in) | T37: the full window. |
| V26 | get rid of | idiom, verbatim (dictionary) | 1 | Mosh | [get rid of](https://en.wiktionary.org/wiki/get_rid_of) |  |
| V27 | make for (result in) | idiom, verbatim (dictionary) | 1 | Mosh | [make for](https://en.wiktionary.org/wiki/make_for) | A plain judgement straight after a fact. |

### Natural hedges (23)

Honest uncertainty in plain words.

| ID | Marker | Verdict | Sources | Attested in | Dictionary | Note |
|---|---|---|---|---|---|---|
| H01 | usually | word, verbatim (3 sources) | 3 | Lunar, Mosh, Hammerspoon |  |  |
| H02 | typically | word, verbatim (3 sources) | 3 | TripMode, Mosh, Hammerspoon |  |  |
| H03 | in most cases / most often | idiom, verbatim (3 sources) | 3 | Little Snitch, Pastebot, Mosh |  |  |
| H04 | almost / nearly | word, verbatim (4 sources) | 4 | iStat Menus, HTTPie, Mosh, Transmit |  |  |
| H05 | probably | word, verbatim (3 sources) | 3 | f.lux, Mosh, ngrok |  |  |
| H06 | often | word, verbatim (5 sources) | 5 | Little Snitch, Moom, Pastebot, Oh My Zsh, Mosh |  |  |
| H07 | sometimes | word, verbatim (3 sources) | 3 | Little Snitch, Lunar, Mosh |  |  |
| H08 | may / might | word, verbatim (7 sources) | 7 | iStat Menus, Little Snitch, Caffeine, Oh My Zsh, Mosh, Hammerspoon, Dash |  |  |
| H09 | about (roughly) | word, verbatim (3 sources) | 3 | Maccy, Oh My Zsh, Mosh |  | Round in prose; exact in lists. |
| H10 | if you want / if you prefer | idiom, verbatim (9 sources) | 9 | iStat Menus, Bartender, TripMode, Radio Silence, Lunar, Itsycal, Mosh, Hammerspoon, Transmit |  | Nine sources. |
| H11 | a bit / a little | idiom, verbatim (dictionary) | 2 | Oh My Zsh, Mosh | [a bit](https://en.wiktionary.org/wiki/a_bit) |  |
| H12 | we think / we believe | word, verbatim (grammar) | 2 | iStat Menus, Mosh |  | Only for opinion, never for facts the app can show. |
| H13 | it's possible that / potentially | word, verbatim (grammar) | 2 | Little Snitch, f.lux |  |  |
| H14 | pretty much | idiom, verbatim (dictionary) | 1 | ngrok | [pretty much](https://en.wiktionary.org/wiki/pretty_much) |  |
| H15 | or so | idiom, verbatim (dictionary) | 1 | Transmit | [or so](https://en.wiktionary.org/wiki/or_so) |  |
| H16 | just about | idiom, verbatim (dictionary) | 1 | f.lux | [just about](https://en.wiktionary.org/wiki/just_about) |  |
| H17 | for now / at this time | idiom, verbatim (dictionary) | 1 | Mosh | [for now](https://en.wiktionary.org/wiki/for_now) | Builds that are not signed yet. |
| H18 | as far as [we know / anyone is aware] | idiom, verbatim (dictionary) | 1 | Mosh | [as far as](https://en.wiktionary.org/wiki/as_far_as) | D24. |
| H19 | a couple of | idiom, verbatim (dictionary) | 1 | iStat Menus | [couple](https://en.wiktionary.org/wiki/couple) | T29: rounding in prose. |
| H20 | a (small) handful | idiom, verbatim (dictionary) | 1 | Mosh | [handful](https://en.wiktionary.org/wiki/handful) |  |
| H21 | not that we know of | idiom, verbatim (dictionary) | 1 | Mosh | [not that I know of](https://en.wiktionary.org/wiki/not_that_I_know_of) | A short FAQ answer. |
| H22 | generally | word, verbatim (dictionary) | 1 | Mosh | [generally](https://en.wiktionary.org/wiki/generally) |  |
| H23 | Shape: A scope caveat of two words placed after a capability (for example, a support condition). | shape only | 1 | iStat Menus |  | One source. |

### Attested, but off-limits for QuotaDial (12)

Real human idioms the sources use that QuotaDial cannot: untrue for the app, or against PRODUCT.md.

| ID | Marker | Verdict | Sources | Attested in | Dictionary | Note |
|---|---|---|---|---|---|---|
| X01 | real time / real-time | do not use (attested idiom) | 4 | iStat Menus, Little Snitch, Radio Silence, ngrok | [real time](https://en.wiktionary.org/wiki/real_time) | Untrue: QuotaDial asks Anthropic at most every 5 minutes and ChatGPT at most every 2. |
| X02 | gold standard | do not use (attested idiom) | 1 | Transmit | [gold standard](https://en.wiktionary.org/wiki/gold_standard) | An unprovable superlative; no benchmarks exist (PRODUCT.md). |
| X03 | set the bar | do not use (attested idiom) | 1 | Sublime Merge | [set the bar](https://en.wiktionary.org/wiki/set_the_bar) | A comparative claim with no evidence. |
| X04 | the talk of the town | do not use (attested idiom) | 1 | Oh My Zsh | [talk of the town](https://en.wiktionary.org/wiki/talk_of_the_town) | Implies popularity; no user counts exist. |
| X05 | hands down | do not use (attested idiom) | 1 | Maccy | [hands down](https://en.wiktionary.org/wiki/hands_down) | A superlative in a testimonial voice; QuotaDial has no testimonials. |
| X06 | Shape: Speed bragged with a colloquial adjective or a doubled intensifier. | do not use (shape) | 5 | Bartender, Maccy, Volta, Sublime Merge, Transmit |  | No benchmark exists. |
| X07 | Shape: The product as a source of super-human powers for the reader or the bar. | do not use (shape) | 2 | Bartender, Oh My Zsh |  | Hype, and near the banned 'supercharge'. |
| X08 | Shape: A lively-community adjective in front of 'community'. | do not use (shape) | 2 | Rainmeter, Oh My Zsh |  | No community evidence to show. |
| X09 | Shape: A promise to improve the reader's life in general. | do not use (shape) | 2 | f.lux, Oh My Zsh |  | Fails question 1 of the six: no claim inside it. |
| X10 | Shape: A claim of hours saved or time not wasted. | do not use (shape) | 2 | Maccy, Alfred |  | Unmeasured. |
| X11 | Shape: Refund or guarantee language. | do not use (shape) | 2 | Radio Silence, Oh My Zsh |  | The site shows no price; Lemon Squeezy owns the checkout. |
| X12 | Shape: A speed claim measured in a tiny slice of time. | do not use (shape) | 1 | Maccy |  | No benchmark exists. |

## Counts

- Sources: 28 unrelated pages, snapshots dated 2020-06-10 to 2021-12-29; 19 of them supply templates.
- Templates: 40 (conversational_lead 9, short_punchy 8, editorial_reasoning 9, data_presentation 6, transition 6, quote_integration 2).
- Markers: 163 (metaphors 49, rhetorical devices 30, editorial connectors 22, colloquial verbs 27, natural hedges 23, off-limits 12).
- Attested idioms cleared for verbatim use: 68 (14 by a count of three or more sources, 54 by a dictionary entry; where both routes hold, the count is recorded).
- Plain words and grammar, verbatim: 37. Shape-only markers: 16. Device shapes: 30. Attested but off-limits: 12.

## Keeping it current

Treat this as a working set a person can edit: delete any template that reads badly by eye. No effectiveness score is kept, because nothing measures one. If the site's voice or its audience moves, harvest again rather than stretching this pool; a second page in the same niche is close to free while the library still fits.

