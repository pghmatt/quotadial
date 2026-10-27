// A stand-in for the page, with as much of the DOM as the code the window check runs in it asks for
// (`SHARES` in whole-shares.mjs, `CAPTION_WORDS` in whole-words.mjs), so that the tests of that code need no
// browser: they run it here against a small page they build, and the browser's part, which boxes a
// piece of text is drawn in, is held by the window check itself.
import { runInNewContext } from "node:vm";

/** A text node. */
export const text = (textContent) => ({ textContent, parentElement: null });

/**
 * An element with `children`. Drawn `display` and `shown` (an element is shown when it and all above
 * it are) are what the page's own styles and layout would say, and `width` how wide it is drawn.
 */
export function tag(name, { classes = [], display = "inline", shown = true, width = 0 } = {}, ...children) {
  const element = {
    tagName: name.toUpperCase(),
    classList: classes,
    style: { display, visibility: "visible" },
    shown,
    children,
    parentElement: null,
    get textContent() {
      return children.map((child) => child.textContent).join("");
    },
    getBoundingClientRect() {
      return { width };
    },
    getClientRects() {
      for (let above = element; above; above = above.parentElement) {
        if (!above.shown) {
          return [];
        }
      }
      return [{}];
    },
    // The selectors the code asks with: a class, or a list of tags.
    closest(selector) {
      const wanted = selector.split(",").map((part) => part.trim());
      for (let above = element; above; above = above.parentElement) {
        if (wanted.some((part) => (part.startsWith(".") ? above.classList.includes(part.slice(1)) : above.tagName.toLowerCase() === part))) {
          return above;
        }
      }
      return null;
    },
  };
  for (const child of children) {
    child.parentElement = element;
  }
  return element;
}

/** Every text node under `node`, in order. */
const textsUnder = (node) => (node.children ? node.children.flatMap(textsUnder) : [node]);

/**
 * What `code`, which the window check would run in the page, answers in a page whose roots, found by
 * whatever selector it asks for, are `roots`, with `drawnIn(start, end)` giving the boxes a range from
 * `start` to `end`, each `[node, offset]`, is drawn in.
 */
export function inPage(code, roots, drawnIn) {
  const page = {
    querySelectorAll: () => roots,
    createTreeWalker: (root) => {
      const nodes = textsUnder(root);
      let at = 0;
      return { nextNode: () => nodes[at++] ?? null };
    },
    createRange: () => {
      const range = {
        setStart: (node, offset) => (range.start = [node, offset]),
        setEnd: (node, offset) => (range.end = [node, offset]),
        getClientRects: () => drawnIn(range.start, range.end),
      };
      return range;
    },
  };
  const result = runInNewContext(code, {
    document: page,
    getComputedStyle: (element) => element.style,
    NodeFilter: { SHOW_TEXT: 4 },
  });
  // From another realm, as the page's own answer is.
  return JSON.parse(JSON.stringify(result));
}
