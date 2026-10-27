// How the window check holds a caption's words whole (whole-words.mjs): that each word of four
// letters or more is found wherever it is written, across the text nodes it is cut into; which
// captions a copy of the window should draw, in order; that the captions the window draws are held
// to those before any line is judged; and that a word's line boxes say whether it is drawn on one
// line or two, and excuse a word that is wider than its caption. The page's own code runs here
// against a small stand-in for the page, and the browser's part, which boxes a word is drawn in, is
// held by the window check itself.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { Script } from "node:vm";

import { inPage, tag, text } from "./page-stand-in.mjs";
import { between, randomTexts, seeded, seedOfThisRun } from "./random-text.mjs";
import { rules } from "./stylesheet.mjs";
import { PAGE_TEXT, matchesIn } from "./whole-shares.mjs";
import { CAPTIONS, CAPTION_WORDS, captionVerdict, expectedCaptions, wordVerdicts, wordsIn, wordsOf, wordsWithRuns } from "./whole-words.mjs";
import { crowded, free, many, pro } from "./window-fixtures.mjs";

test("a word is four letters or more in a row, found whole across the nodes it is written in, with where it starts and ends", () => {
  assert.deepEqual(wordsIn(["Weekly"]), [{ text: "Weekly", from: { node: 0, offset: 0 }, to: { node: 0, offset: 6 } }]);
  // A word cut by markup is one word, as a share cut into two nodes is one share.
  assert.deepEqual(wordsIn(["Wee", "kly (Claude"]), [
    { text: "Weekly", from: { node: 0, offset: 0 }, to: { node: 1, offset: 3 } },
    { text: "Claude", from: { node: 1, offset: 5 }, to: { node: 1, offset: 11 } },
  ]);
  // Empty nodes in the middle of a word change nothing.
  assert.deepEqual(wordsIn(["Wee", "", "kly"])[0], { text: "Weekly", from: { node: 0, offset: 0 }, to: { node: 2, offset: 3 } });
  // Any script, and a letter with a mark of its own.
  assert.deepEqual(wordsOf("Überlauf, café and 日本語です"), ["Überlauf", "café", "日本語です"]);
});

test("a word is four letters, not three: a short word, a figure and a sign are not one, and a hyphen ends a word", () => {
  for (const line of ["", "GPT", "and", "64%", "5-h", "a b c", "1234", "abc def", "x-ray", "(ABC)"]) {
    assert.deepEqual(wordsOf(line), [], JSON.stringify(line));
  }
  assert.deepEqual(wordsOf("5-hour"), ["hour"]);
  assert.deepEqual(wordsOf("Weekly (Claude and GPT)"), ["Weekly", "Claude"]);
  assert.deepEqual(wordsOf("latest 64%, highest 100%"), ["latest", "highest"]);
  assert.deepEqual(wordsOf("from the API 4 min ago"), ["from"]);
  assert.deepEqual(wordsOf("When it was used, by hour (weekly)"), ["When", "used", "hour", "weekly"]);
});

test("a word's run is the word and what is written against it, up to a space, across the nodes it is written in", () => {
  const run = (texts) => wordsWithRuns(texts).map((word) => [word.text, word.run]);
  assert.deepEqual(run(["Weekly (ClaudeSonnet)"]), [
    ["Weekly", { text: "Weekly", from: { node: 0, offset: 0 }, to: { node: 0, offset: 6 } }],
    ["ClaudeSonnet", { text: "(ClaudeSonnet)", from: { node: 0, offset: 7 }, to: { node: 0, offset: 21 } }],
  ]);
  // Cut by markup, a run is one run, as a word is.
  assert.deepEqual(run(["Wee", "kly (Claude", ")"]), [
    ["Weekly", { text: "Weekly", from: { node: 0, offset: 0 }, to: { node: 1, offset: 3 } }],
    ["Claude", { text: "(Claude)", from: { node: 1, offset: 4 }, to: { node: 2, offset: 1 } }],
  ]);
  // The words and shares beside a label, written as History writes them: each word is a run of its own.
  assert.deepEqual(wordsWithRuns(["latest ", "91", "%", ", highest", " ", "100", "%"]).map((word) => word.run.text), ["latest", "highest"]);
  // A line breaks at a space or a new line, and not at a no-break space; a hyphen and a stop do not end a run.
  assert.deepEqual(wordsWithRuns(["Claude Code\nfrom"]).map((word) => word.run.text), ["Claude", "Code", "from"]);
  assert.deepEqual(wordsWithRuns(["Claude\u00a0Code"]).map((word) => word.run.text), ["Claude\u00a0Code", "Claude\u00a0Code"]);
  assert.deepEqual(wordsWithRuns(["by hour (5-hour)."]).map((word) => word.run.text), ["hour", "(5-hour)."]);
});

test("however the text is cut into nodes, each word found is the text between its start and end, and none is missed", () => {
  // A new seed each run; the seed is in every failure, and QUOTADIAL_TEST_SEED=<seed> runs the same strings again.
  const seed = seedOfThisRun();
  const next = seeded(seed);
  // Letters, which make words, in Latin, accented and other scripts, and in a character that takes
  // two UTF-16 units; and what ends one: figures, a hyphen, brackets, a stop, spaces, and an emoji.
  const pieces = [
    ..."abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzWKLY",
    "é", "ü", "日", "本", "\u{20bb7}",
    "0", "5", "-", "(", ")", ".", ",", "%", " ", " ", "\n", "\u00a0", "\u{1f600}",
  ];
  // A space at which a line may break: not a no-break space.
  const breaks = (character) => character === undefined || /^[^\S\u00a0\u2007\u202f]$/u.test(character);
  const cases = 5000;
  let withWords = 0;
  for (let index = 0; index < cases; index += 1) {
    const texts = randomTexts(next, pieces, 30);
    const where = `seed ${seed}, case ${index}, ${JSON.stringify(texts)} (QUOTADIAL_TEST_SEED=${seed} runs it again)`;
    const wanted = [...texts.join("").matchAll(/\p{L}{4,}/gu)].map((found) => found[0]);
    const found = wordsIn(texts);
    assert.deepEqual(
      found.map((word) => word.text),
      wanted,
      `the words found are not those in the text: ${where}`,
    );
    for (const word of found) {
      assert.equal(between(texts, word.from, word.to), word.text, `a word's start and end give other text: ${where}`);
      assert.ok(word.from.offset < texts[word.from.node].length, `a word starts past the end of its node: ${where}`);
      assert.ok(word.to.offset >= 1 && word.to.offset <= texts[word.to.node].length, `a word ends outside its node: ${where}`);
    }
    // Each word's run holds the word, has no space a line may break at, and has one, or the text's
    // start or end, on either side of it.
    const all = texts.join("");
    const at = ({ node, offset }) => texts.slice(0, node).join("").length + offset;
    const withRuns = wordsWithRuns(texts);
    assert.deepEqual(withRuns.map(({ run, ...word }) => word), found, `the words with runs are not the words: ${where}`);
    for (const { run, ...word } of withRuns) {
      assert.equal(between(texts, run.from, run.to), run.text, `a run's start and end give other text: ${where}`);
      assert.ok(at(run.from) <= at(word.from) && at(word.to) <= at(run.to), `a run does not hold its word: ${where}`);
      assert.ok(![...run.text].some(breaks), `a run holds a space a line may break at: ${where}`);
      assert.ok(breaks(all[at(run.from) - 1]) && breaks(all[at(run.to)]), `a run stops short of a space: ${where}`);
    }
    withWords += found.length > 0 ? 1 : 0;
  }
  assert.ok(withWords >= cases / 10, `only ${withWords} of ${cases} strings held a word, seed ${seed}`);
});

/**
 * A copy of the window in the shape `expectedCaptions` reads: an account of the meter with the line
 * saying where its windows were read, or none, and an account of History with its charts and the
 * window its heatmap counts use in, or none.
 */
const small = {
  meter: { accounts: [{ fetched: "from the API 3 min ago" }, { fetched: null }, { fetched: "from cache 14 min ago" }] },
  history: {
    7: {
      accounts: [
        {
          windows: [
            { label: "Weekly", latest: 63.9, highest: 99.99 },
            { label: "5-hour (Gemini)", latest: 0.4, highest: 12 },
          ],
          heatmap: { window: "5-hour (Gemini)", most: 9 },
        },
        { windows: [{ label: "Session", latest: 8, highest: 11 }], heatmap: { window: "Session", most: 0 } },
      ],
    },
  },
};

test("the captions a copy should draw: in the accounts the line for each card that has one, in History each chart's and each heatmap's", () => {
  assert.deepEqual(expectedCaptions(small, "accounts"), ["from the API 3 min ago", "from cache 14 min ago"]);
  // A chart's caption is its label, then the latest and the highest share, each rounded down as the
  // window writes it; a heatmap's, when the account was used at all, names its window in lower case; and
  // the heatmap's follows its account's charts.
  assert.deepEqual(expectedCaptions(small, "history"), [
    "Weekly latest 63%, highest 99%",
    "5-hour (Gemini) latest 0%, highest 12%",
    "When it was used, by hour (5-hour (gemini))",
    "Session latest 8%, highest 11%",
  ]);
  // History opens on 7 days; the other period is not asked for.
  assert.deepEqual(expectedCaptions({ ...small, history: { 7: small.history[7] } }, "history"), expectedCaptions(small, "history"));
  // Settings and Help have none.
  assert.deepEqual(expectedCaptions(small, "settings"), []);
  assert.deepEqual(expectedCaptions(small, "help"), []);
});

test("every copy has captions in the accounts and in History, each with a word the check can judge", () => {
  for (const [name, copy] of Object.entries({ pro, free, many, crowded })) {
    for (const panel of ["accounts", "history"]) {
      if (copy.panels && !copy.panels.includes(panel)) {
        continue;
      }
      const captions = expectedCaptions(copy, panel);
      assert.ok(captions.length > 0, `${name} has no captions in ${panel}, so nothing there is held`);
      for (const caption of captions) {
        assert.ok(wordsOf(caption).length > 0, `${name}'s caption ${JSON.stringify(caption)} holds no word of four letters`);
      }
    }
  }
  // One for each chart and each heatmap in History.
  const charts = (copy) => copy.history[7].accounts.reduce((sum, account) => sum + account.windows.length + (account.heatmap.most > 0 ? 1 : 0), 0);
  for (const copy of [pro, free, many, crowded]) {
    assert.equal(expectedCaptions(copy, "history").length, charts(copy));
  }
  // The accounts: those the app sends a line for, which is not the one whose windows each say their own.
  assert.equal(expectedCaptions(pro, "accounts").length, pro.meter.accounts.filter((account) => account.fetched).length);
  assert.ok(pro.meter.accounts.some((account) => !account.fetched), "the copy has an account that gets no line, which is not held");
  assert.equal(expectedCaptions(free, "accounts").at(-1), "from the example build 1 min ago");
});

test("a caption is what the copy says, in order, with its words, or the verdict says which is not", () => {
  const caption = (text, ...words) => ({ what: "p.source", text, room: 90, words: words.map((word) => ({ text: word, boxes: [] })) });
  const drawn = [caption("from the API 3 min ago", "from"), caption("Weekly latest 8%, highest 11%", "Weekly", "latest", "highest")];
  const wanted = ["from the API 3 min ago", "Weekly latest 8%, highest 11%"];
  assert.equal(captionVerdict(drawn, wanted), false);
  assert.equal(captionVerdict([], []), false);
  // A caption that is not the copy's, one too few and one too many, and none at all.
  assert.match(captionVerdict([drawn[0], caption("Weekly latest 9%, highest 11%", "Weekly", "latest", "highest")], wanted), /^the window's caption 2 is "Weekly latest 9%, highest 11%", where its copy has "Weekly latest 8%, highest 11%" \(2 drawn, 2 wanted\)$/);
  assert.match(captionVerdict([drawn[0]], wanted), /^the window's caption 2 is no caption, where its copy has "Weekly latest 8%, highest 11%" \(1 drawn, 2 wanted\)$/);
  assert.match(captionVerdict(drawn, [wanted[0]]), /^the window's caption 2 is "Weekly latest 8%, highest 11%", where its copy has none \(2 drawn, 1 wanted\)$/);
  assert.match(captionVerdict([], wanted), /caption 1 is no caption/);
  // The same order matters.
  assert.match(captionVerdict(drawn, [wanted[1], wanted[0]]), /caption 1 is "from the API 3 min ago"/);
  // A caption whose words were not found, or are not its own, so that nothing is judged there.
  assert.match(captionVerdict([caption("from the API 3 min ago"), drawn[1]], wanted), /^the words found in the caption "from the API 3 min ago" are "", not "from"$/);
  assert.match(captionVerdict([drawn[0], caption("Weekly latest 8%, highest 11%", "Weekly", "highest")], wanted), /"Weekly highest", not "Weekly latest highest"/);
});

/** A box, in CSS pixels. */
const box = (top, bottom, left = 0, right = 20) => ({ top, bottom, left, right });

test("a word on one line is whole, whether its range is one box or two", () => {
  const captions = [{ what: "figcaption", text: "Weekly latest 8%", room: 97, words: [
    { text: "Weekly", boxes: [box(0, 16, 0, 40)] },
    { text: "latest", boxes: [box(18, 34, 0, 20), box(18, 34, 20, 38)] },
  ] }];
  assert.deepEqual(wordVerdicts(captions), [false, false]);
  assert.deepEqual(wordVerdicts([]), []);
});

test('a word on two lines is broken, "Wee" over "kly", unless it is wider than its whole caption, which has to break', () => {
  const caption = (room, ...boxes) => ({ what: "figcaption", text: "Weekly latest 8%", room, words: [{ text: "Weekly", boxes }] });
  // 40 px of word in 97 px of caption: it fits a line of its own, so breaking it is a verdict.
  assert.deepEqual(wordVerdicts([caption(97, box(0, 16, 0, 20), box(16, 32, 0, 20))]), [
    '"Weekly" is broken over 2 lines, though it fits its caption (figcaption "Weekly latest 8%")',
  ]);
  assert.match(wordVerdicts([caption(97, box(0, 16, 0, 8), box(16, 32, 0, 8), box(32, 48, 0, 24))])[0], /^"Weekly" is broken over 3 lines/);
  // A word as wide as the caption fits it, to half a pixel, and a word wider does not: it has to break.
  assert.match(wordVerdicts([caption(97, box(0, 16, 0, 60), box(16, 32, 0, 37))])[0], /^"Weekly" is broken over 2 lines/);
  assert.match(wordVerdicts([caption(97, box(0, 16, 0, 60), box(16, 32, 0, 37.4))])[0], /^"Weekly" is broken over 2 lines/);
  assert.deepEqual(wordVerdicts([caption(97, box(0, 16, 0, 60), box(16, 32, 0, 38))]), [false]);
  assert.deepEqual(wordVerdicts([caption(97, box(0, 16, 0, 97), box(16, 32, 0, 23))]), [false]);
});

test("a word in a run wider than its caption, as a model's name in its brackets can be, may break inside the word; in a run that fits, it may not", () => {
  // Windows' CI at 100% (#102's first run): the Many copy's "(ClaudeSonnetFourPointFiveExtendedContextPreview)"
  // was wider than its caption and the name alone was not, so the browser broke the run where the
  // line ended, inside the name. The widths are those this Mac's Chrome draws, in a caption between
  // the name's 298.6 px and the 303.1 px of the name with its opening bracket.
  const name = "ClaudeSonnetFourPointFiveExtendedContextPreview";
  const text = `Weekly (${name}) latest 12%, highest 12%`;
  const caption = (room) => ({ what: "figcaption", text, room, words: [{
    text: name,
    boxes: [box(15, 30, 4.6, 293.9), box(30, 45, 0, 9.3)],
    run: { text: `(${name})`, boxes: [box(15, 30, 0, 293.9), box(30, 45, 0, 13.9)] },
  }] });
  assert.deepEqual(wordVerdicts([caption(300.84)]), [false]);
  // In a caption the whole run fits, nothing made the browser break the name, so it is a verdict, which names the run.
  assert.deepEqual(wordVerdicts([caption(310)]), [
    `"${name}" is broken over 2 lines, though "(${name})" fits its caption (figcaption ${JSON.stringify(text)})`,
  ]);
});

test("a word with no box to measure is a verdict, and the verdicts come in the order the words are", () => {
  const found = [
    { what: "p.source", text: "from the API 3 min ago", room: 90, words: [{ text: "from", boxes: [box(0, 14, 0, 24)] }] },
    { what: "figcaption", text: "Weekly latest 8%", room: 97, words: [
      { text: "Weekly", boxes: [box(0, 16, 0, 20), box(16, 32, 0, 20)] },
      { text: "latest", boxes: [] },
    ] },
  ];
  assert.deepEqual(wordVerdicts(found), [
    false,
    '"Weekly" is broken over 2 lines, though it fits its caption (figcaption "Weekly latest 8%")',
    '"latest" has no box to measure (figcaption "Weekly latest 8%")',
  ]);
  // The order of the boxes does not matter, as a share's does not.
  const apart = [box(0, 16, 0, 20), box(16, 32, 0, 20)];
  assert.match(wordVerdicts([{ what: "figcaption", text: "x", room: 97, words: [{ text: "Weekly", boxes: apart.toReversed() }] }])[0], /broken over 2 lines/);
});

// The page's own code, run against a stand-in for the page (page-stand-in.mjs).

test("the code for the page is valid, names the captions it is given and carries what the tests hold", () => {
  const code = CAPTION_WORDS(CAPTIONS);
  assert.doesNotThrow(() => new Script(code));
  assert.ok(code.includes(JSON.stringify(CAPTIONS)));
  assert.ok(code.includes(String(wordsIn)));
  assert.ok(code.includes(String(wordsWithRuns)));
  assert.ok(code.includes(String(matchesIn)));
  assert.ok(code.includes(PAGE_TEXT));
});

/** History's caption as the window writes it: the label, and the shares round the words after it. */
function chartCaption(label, latest, highest, width = 97) {
  const share = (figure) => tag("span", { classes: ["nobreak"] }, text(String(figure)), text("%"));
  return tag(
    "figcaption",
    { display: "flex", width },
    tag("span", { display: "block" }, text(label)),
    tag("span", { classes: ["detail"], display: "block" }, text("latest "), share(latest), text(", highest"), text(" "), share(highest)),
  );
}

test("in the page, a caption is its blocks apart, its words are found across the nodes, and each has the boxes of its range", () => {
  const caption = chartCaption("Weekly", 91, 100);
  const asked = [];
  const [found] = inPage(CAPTION_WORDS(CAPTIONS), [caption], (start, end) => {
    asked.push([start[0].textContent, start[1], end[0].textContent, end[1]]);
    return [{ top: 0, bottom: 16, left: 0, right: 30 }];
  });
  // The label and the words round the shares are blocks of their own, so a word never runs from one
  // into the next, and the text has a space where they meet.
  assert.equal(found.text, "Weekly latest 91%, highest 100%");
  assert.equal(found.what, "figcaption");
  assert.equal(found.room, 97);
  assert.deepEqual(found.words.map((word) => word.text), ["Weekly", "latest", "highest"]);
  assert.deepEqual(found.words.map((word) => word.boxes), [[{ top: 0, bottom: 16, left: 0, right: 30 }], [{ top: 0, bottom: 16, left: 0, right: 30 }], [{ top: 0, bottom: 16, left: 0, right: 30 }]]);
  // Each word is a run of its own here, measured as the word is.
  assert.deepEqual(found.words.map((word) => word.run), ["Weekly", "latest", "highest"].map((run) => ({ text: run, boxes: [{ top: 0, bottom: 16, left: 0, right: 30 }] })));
  // "latest " is its own node, 6 letters from offset 0; ", highest" starts after the comma and the
  // space. Each word's range is asked for, then its run's.
  const once = [["Weekly", 0, "Weekly", 6], ["latest ", 0, "latest ", 6], [", highest", 2, ", highest", 9]];
  assert.deepEqual(asked, once.flatMap((range) => [range, range]));
});

test("in the page, a model's name in its brackets is measured as the run it is, and breaks inside the name only where the run is wider than the caption", () => {
  const name = "ClaudeSonnetFourPointFiveExtendedContextPreview";
  const label = `Weekly (${name})`;
  // The browser's boxes, as this Mac's Chrome draws them in a caption 300.84 px wide: the run goes
  // under "Weekly" and breaks before "w)", inside the name. Every other word is on one line.
  const drawnIn = (start, end) => {
    const range = [start[0].textContent, start[1], end[1]].join(" ");
    if (range === `${label} 8 55`) {
      return [{ top: 15, bottom: 30, left: 4.6, right: 293.9 }, { top: 30, bottom: 45, left: 0, right: 9.3 }];
    }
    if (range === `${label} 7 56`) {
      return [{ top: 15, bottom: 30, left: 0, right: 293.9 }, { top: 30, bottom: 45, left: 0, right: 13.9 }];
    }
    return [{ top: 0, bottom: 15, left: 0, right: 40 }];
  };
  const [narrow] = inPage(CAPTION_WORDS(CAPTIONS), [chartCaption(label, 12, 12, 300.84)], drawnIn);
  assert.deepEqual(narrow.words.map((word) => [word.text, word.run.text]), [["Weekly", "Weekly"], [name, `(${name})`], ["latest", "latest"], ["highest", "highest"]]);
  assert.deepEqual(wordVerdicts([narrow]), [false, false, false, false]);
  assert.equal(captionVerdict([narrow], [`${label} latest 12%, highest 12%`]), false);
  // In a caption the run fits, the same boxes are a name broken for nothing.
  const [wide] = inPage(CAPTION_WORDS(CAPTIONS), [chartCaption(label, 12, 12, 310)], drawnIn);
  assert.deepEqual(wordVerdicts([wide]), [
    false,
    `"${name}" is broken over 2 lines, though "(${name})" fits its caption (figcaption "${label} latest 12%, highest 12%")`,
    false,
    false,
  ]);
});

test("in the page, a word broken by the browser is a word on two lines, and the verdicts say so", () => {
  const caption = chartCaption("Weekly", 91, 91);
  const found = inPage(CAPTION_WORDS(CAPTIONS), [caption], (start) =>
    start[0].textContent === "Weekly"
      ? [{ top: 0, bottom: 16, left: 0, right: 20 }, { top: 16, bottom: 32, left: 0, right: 20 }]
      : [{ top: 0, bottom: 16, left: 40, right: 80 }],
  );
  assert.deepEqual(wordVerdicts(found), [
    '"Weekly" is broken over 2 lines, though it fits its caption (figcaption "Weekly latest 91%, highest 91%")',
    false,
    false,
  ]);
  assert.equal(captionVerdict(found, ["Weekly latest 91%, highest 91%"]), false);
});

test("in the page, each caption is read apart, in the page's order, and what is not shown or is for a screen reader is left out", () => {
  const source = tag("p", { classes: ["source"], display: "block", width: 90 }, text("from the API 3 min ago"));
  const heatmap = tag("figcaption", { classes: ["detail"], display: "block", width: 97 }, text("When it was used, by hour ("), text("weekly"), text(")"));
  const hidden = tag("figcaption", { display: "flex", shown: false }, tag("span", { display: "block" }, text("Session")));
  const aside = tag("figcaption", { classes: ["visually-hidden"], display: "block" }, text("Mon 20: used 9:00 to 18:00"));
  const drawn = () => [{ top: 0, bottom: 14, left: 0, right: 30 }];
  const found = inPage(CAPTION_WORDS(CAPTIONS), [source, chartCaption("5-hour", 12, 12), heatmap, hidden, aside], drawn);
  assert.deepEqual(found.map((caption) => caption.text), [
    "from the API 3 min ago",
    "5-hour latest 12%, highest 12%",
    "When it was used, by hour (weekly)",
    "",
    "",
  ]);
  assert.deepEqual(found.map((caption) => caption.what), ["p.source", "figcaption", "figcaption.detail", "figcaption", "figcaption.visually-hidden"]);
  assert.deepEqual(found.map((caption) => caption.words.map((word) => word.text)), [
    ["from"],
    ["hour", "latest", "highest"],
    ["When", "used", "hour", "weekly"],
    [],
    [],
  ]);
  // A caption the window does not draw is not one it is held to, so the copy's has to be missing it.
  assert.match(captionVerdict(found, ["from the API 3 min ago", "5-hour latest 12%, highest 12%", "When it was used, by hour (weekly)", "Session latest 8%, highest 8%"]), /caption 4 is "", where its copy has "Session latest 8%, highest 8%"/);
});

/**
 * The lines style.css lets break anywhere, which D-139 names with why: each can carry text from
 * outside with no spaces in it, which would otherwise be cut off at the window's edge (#52). Nothing
 * else may, and the body may not: QuotaDial's own words, a caption's included, keep the browser's
 * default, which breaks between words and leaves a word whole.
 */
const MAY_BREAK_ANYWHERE = [
  ".card-head",
  ".label",
  ".headline",
  ".incident",
  ".note",
  ".problems",
  ".costs",
  ".sign-in",
  ".settings .card",
  ".help .card",
  ".update-offer",
  ".history > p",
  ".account-history > .detail",
  ".heatmap figcaption",
  "code",
];

test("style.css lets only the lines that can carry text from outside with no spaces in it break anywhere, and not the body (D-139)", () => {
  const breaking = rules().flatMap((rule) =>
    rule.declarations
      .filter((declaration) => /^(overflow-wrap|word-wrap|word-break|line-break|hyphens)\s*:/.test(declaration))
      .map((declaration) => ({ selectors: rule.selectors, declaration })),
  );
  // Every rule of that kind in the stylesheet is `overflow-wrap: anywhere`, and on these lines.
  assert.deepEqual([...new Set(breaking.map((one) => one.declaration))], ["overflow-wrap: anywhere"]);
  assert.deepEqual(breaking.flatMap((one) => one.selectors).toSorted(), MAY_BREAK_ANYWHERE.toSorted());
  // Nothing that reaches every line: not the page, nor the body that holds the window.
  for (const wide of ["*", "html", "body", ":root", "main", ".popover", ".scroll", ".content", ".card", "p", "li", "span", "div"]) {
    assert.ok(!MAY_BREAK_ANYWHERE.includes(wide), `${wide} may not break words anywhere`);
  }
});

test("a caption's label may break anywhere and the caption wraps, so that the label and the words beside it never squeeze each other", () => {
  const css = (name) => readFileSync(new URL(`../src/${name}`, import.meta.url), "utf8");
  // History's caption is a flex row of the window's label, which can be a model's name, and the
  // words and shares beside it. Two items that may both shrink share the room in proportion and, when
  // each may break anywhere, give up letters of a short word. A label that may break anywhere
  // therefore has the caption wrap, and the words beside it not break anywhere.
  assert.match(css("history.tsx"), /<span class="label">\{chart\.label\}<\/span>\s*<span class="detail">/);
  const caption = rules().filter((rule) => rule.selectors.includes(".chart figcaption")).flatMap((rule) => rule.declarations);
  assert.ok(caption.includes("display: flex"));
  assert.ok(caption.includes("flex-wrap: wrap"), "a caption that does not wrap squeezes its label and the words beside it");
  assert.ok(MAY_BREAK_ANYWHERE.includes(".label"));
  assert.ok(!MAY_BREAK_ANYWHERE.some((selector) => selector.includes("figcaption") && selector.includes("detail")), "the words and shares beside a caption's label may not break anywhere");
  // The card's own caption is QuotaDial's words alone, in a paragraph of its own.
  assert.ok(!MAY_BREAK_ANYWHERE.includes(".source"));
});
