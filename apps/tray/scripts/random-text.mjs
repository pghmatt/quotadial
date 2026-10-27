// What the property tests of the window check's scripts (whole-shares.test.mjs, whole-words.test.mjs)
// make their strings of: a seeded generator, so that the same seed makes the same strings and a
// failing input is found again from its seed, and text cut into nodes at random places, as Preact
// writes a text a node at a time.

/**
 * A seeded generator of numbers from 0 up to 1 (mulberry32), as code.test.mjs has: the same seed
 * makes the same strings, so a failing input is found again from its seed.
 */
export function seeded(seed) {
  let state = seed >>> 0;
  return () => {
    state = (state + 0x6d2b79f5) >>> 0;
    let mixed = Math.imul(state ^ (state >>> 15), state | 1);
    mixed ^= mixed + Math.imul(mixed ^ (mixed >>> 7), mixed | 61);
    return ((mixed ^ (mixed >>> 14)) >>> 0) / 2 ** 32;
  };
}

/**
 * The seed of a run of a property test: a new one each time, which every failure says, and
 * `QUOTADIAL_TEST_SEED=<seed>` runs the same strings again. It throws if that is not a whole number
 * from 0 to 2 ** 32 - 1.
 */
export function seedOfThisRun() {
  const given = process.env.QUOTADIAL_TEST_SEED;
  const seed = given === undefined ? Math.floor(Math.random() * 2 ** 32) : Number(given);
  if (!(Number.isInteger(seed) && seed >= 0 && seed < 2 ** 32)) {
    throw new Error(`QUOTADIAL_TEST_SEED is "${given}", not a whole number from 0 to 2 ** 32 - 1`);
  }
  return seed;
}

/**
 * A text of up to `longest` of `pieces`, chosen by `next` (a generator from `seeded`), cut at random
 * places into the text nodes it is written in: in the middle of a word or a share as often as not,
 * leaving some nodes empty. Never none.
 */
export function randomTexts(next, pieces, longest = 24) {
  let joined = "";
  for (let at = Math.floor(next() * longest); at > 0; at -= 1) {
    joined += pieces[Math.floor(next() * pieces.length)];
  }
  const texts = [];
  let from = 0;
  for (let cuts = Math.floor(next() * 6); cuts >= 0; cuts -= 1) {
    const to = cuts === 0 ? joined.length : Math.min(joined.length, from + Math.floor(next() * (joined.length - from + 1)));
    texts.push(joined.slice(from, to));
    from = to;
  }
  texts[texts.length - 1] += joined.slice(from);
  return texts;
}

/** What a match's start and end give back: the text from one to the other, across the nodes. */
export function between(texts, from, to) {
  if (from.node === to.node) {
    return texts[from.node].slice(from.offset, to.offset);
  }
  return texts[from.node].slice(from.offset) + texts.slice(from.node + 1, to.node).join("") + texts[to.node].slice(0, to.offset);
}
