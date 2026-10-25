/**
 * The repository's two guides, docs/INSTALL.md and docs/CLI.md, as /install and /cli show them:
 * rendered whole at build time, with their own words untouched. This file changes what is around
 * the words and nothing in them: each heading gets the anchor GitHub gives it (so every link the
 * guides already hold to a section keeps working), a link to another file in the repository points
 * at that file's page on this site or on GitHub, and each block (text, a table, a command, example
 * output) is told apart so the page can set it in the site's own components.
 */

import { posix } from "node:path";

import { Marked, Parser, type Token, type Tokens } from "marked";

import { markBreaks } from "../../lib/break-html.ts";
import { SITE } from "../../lib/data/site.ts";
import { repositoryFile, withoutTitle } from "../../lib/markdown.ts";
import { clickDetail, commandSystemOf, extensionOf, systemOf } from "../conversion/download-links.ts";

export type GuideName = "install" | "cli";

/** The guides' own files, and the page each has on this site. */
export const GUIDES: Record<GuideName, { file: string; path: string; from: string }> = {
  install: { file: "docs/INSTALL.md", path: "/install", from: "install-guide" },
  cli: { file: "docs/CLI.md", path: "/cli", from: "command-guide" },
};

/** The pages other repository texts have on this site; every other file points at GitHub. */
const PAGES: Record<string, string> = {
  "docs/INSTALL.md": "/install",
  "docs/CLI.md": "/cli",
  "docs/PRIVACY.md": "/privacy",
};

export interface GuideHeading {
  level: number;
  /** The heading's words, as plain text. */
  text: string;
  /** The heading as HTML, so a command in it keeps its code face. */
  html: string;
  /** GitHub's anchor for it: lower case, no punctuation, hyphens for spaces. */
  id: string;
}

export type GuideBlock =
  | { kind: "html"; html: string }
  /** `command` blocks hold something to run and carry a copy button; the rest are example output. */
  | { kind: "code"; code: string; command: boolean; name: string; nested: boolean }
  | { kind: "table"; caption: string; columns: string[]; rows: string[][] };

export interface GuideSection {
  heading: GuideHeading;
  blocks: GuideBlock[];
}

export interface Guide {
  /** The file's first heading, which the page sets as its H1. */
  title: string;
  /** The paragraph under it, as inline HTML. */
  intro: string;
  sections: GuideSection[];
  /** Every heading from the second level down, in the file's order. */
  headings: GuideHeading[];
}

const COMMAND_LANGUAGES = new Set(["sh", "bash", "zsh", "shell", "powershell"]);

/** GitHub's anchor for a heading's words: lower case, punctuation dropped, spaces as hyphens. */
export function slug(words: string): string {
  return words
    .toLowerCase()
    .replace(/[^\p{L}\p{N}\s-]/gu, "")
    .trim()
    .replace(/\s/g, "-");
}

/**
 * Where a link written in a repository text leads on this site: another guide or the privacy text
 * to its page, any other file in the repository to GitHub. A web address, a mail address and a
 * link within the text stay as they are. `from` is the folder the text sits in.
 */
export function guideLink(href: string, from = "docs"): string {
  if (/^([a-z][a-z0-9+.-]*:|#|\/)/i.test(href)) {
    return href;
  }
  const [path, ...hash] = href.split("#");
  const target = posix.normalize(posix.join(from, path));
  const anchor = hash.length ? `#${hash.join("#")}` : "";
  return PAGES[target] ? `${PAGES[target]}${anchor}` : `${SITE.repository}/blob/main/${target}${anchor}`;
}

const FILE_LINK = /<a href="([^"]*\/releases\/latest\/download\/([^"/#?]+))"/g;

/**
 * A guide's inline HTML with what the page needs and the words do not say. A link that fetches a
 * release's file reports a `download_click` with the file and where it was, as /download's do;
 * and the words of a command or a link may break only at a space or at the marks `breakTokens`
 * names, so a file name, an address or a flag never splits inside a word.
 */
export function finish(html: string, from: string): string {
  const marked = html.replace(FILE_LINK, (_match, href: string, name: string) => {
    const system = ["dmg", "msi", "deb", "AppImage"].includes(extensionOf(name)) ? systemOf(name) : commandSystemOf(name);
    const attributes = Object.entries(clickDetail({ name }, from, system))
      .map(([key, value]) => `${key}="${value}"`)
      .join(" ");
    return `<a href="${href}" ${attributes}`;
  });
  return markBreaks(marked);
}

/** The words of a heading's Markdown, without its code marks. */
function plain(markdown: string): string {
  return markdown.replace(/`/g, "").trim();
}

/**
 * A list one of whose items holds a block (a command, example output) is "loose" in Markdown and
 * would set each item as a paragraph with space round it, unlike the lists beside it. Its items
 * are set tight, as the others are; an item of several paragraphs keeps them.
 */
function tighten(list: Tokens.List): void {
  list.loose = false;
  for (const item of list.items) {
    item.loose = false;
    if (item.tokens.filter((part) => part.type === "paragraph").length === 1) {
      item.tokens = item.tokens.map((part) => (part.type === "paragraph" ? ({ ...part, type: "text" } as Token) : part));
    }
  }
}

function hasCode(tokens: Token[]): boolean {
  return tokens.some(
    (token) =>
      token.type === "code" ||
      (token.type === "list" && (token as Tokens.List).items.some((item) => hasCode(item.tokens))) ||
      ("tokens" in token && Array.isArray(token.tokens) && hasCode(token.tokens as Token[])),
  );
}

/** Reads a guide's Markdown into its title, its opening paragraph and its sections. */
export function readGuide(markdown: string, from = "guide"): Guide {
  const marked = new Marked({ gfm: true, async: false });
  const { title, body } = withoutTitle(markdown);
  if (!title) {
    throw new Error("The guide has no title.");
  }
  const tokens = marked.lexer(body);
  marked.walkTokens(tokens, (token) => {
    if (token.type === "link") {
      (token as Tokens.Link).href = guideLink((token as Tokens.Link).href);
    }
    if (token.type === "list") {
      tighten(token as Tokens.List);
    }
  });
  const inline = (parts: Token[]) => finish(Parser.parseInline(parts), from);
  const used = new Map<string, number>();
  const headingOf = (token: Tokens.Heading): GuideHeading => {
    const base = slug(plain(token.text));
    const count = used.get(base) ?? 0;
    used.set(base, count + 1);
    return {
      level: token.depth,
      text: plain(token.text),
      html: marked.parseInline(token.text) as string,
      id: count === 0 ? base : `${base}-${count}`,
    };
  };

  let intro = "";
  const sections: GuideSection[] = [];
  const headings: GuideHeading[] = [];
  let blocks: GuideBlock[] = [];
  let context = "";

  const html = (parts: Token[]) => {
    const rendered = finish(marked.parser(parts as Tokens.Generic[]).trim(), from);
    if (rendered) {
      blocks.push({ kind: "html", html: rendered });
    }
  };
  const code = (token: Tokens.Code, nested: boolean) => {
    const language = (token.lang ?? "").split(/\s/)[0];
    blocks.push({
      kind: "code",
      code: token.text,
      command: COMMAND_LANGUAGES.has(language),
      name: `${context}, example output`,
      nested,
    });
  };

  for (const token of tokens) {
    if (token.type === "space") {
      continue;
    }
    if (token.type === "heading") {
      const heading = token as Tokens.Heading;
      const found = headingOf(heading);
      headings.push(found);
      context = found.text;
      if (heading.depth === 2) {
        blocks = [];
        sections.push({ heading: found, blocks });
      } else {
        blocks.push({ kind: "html", html: `<h${found.level} id="${found.id}">${found.html}</h${found.level}>` });
      }
      continue;
    }
    if (!sections.length) {
      if (token.type === "paragraph" && !intro) {
        intro = inline((token as Tokens.Paragraph).tokens).trim();
        continue;
      }
      throw new Error(`The guide has text before its first section that the page does not place: ${token.type}.`);
    }
    if (token.type === "code") {
      code(token as Tokens.Code, false);
    } else if (token.type === "table") {
      const table = token as Tokens.Table;
      blocks.push({
        kind: "table",
        caption: `${context}: ${table.header.map((cell) => plain(cell.text)).join(", ")}`,
        columns: table.header.map((cell) => plain(cell.text)),
        rows: table.rows.map((row) => row.map((cell) => inline(cell.tokens).trim())),
      });
    } else if (token.type === "list" && hasCode((token as Tokens.List).items.flatMap((item) => item.tokens))) {
      // A list item that holds a command or example output is split around it, so the block can be
      // set in the site's own component; the item's words stay in their list, the block beneath.
      const list = token as Tokens.List;
      // Items before and after one that holds a block stay together in a list of their own, so the
      // list's spacing is unchanged.
      let run: Tokens.ListItem[] = [];
      let numbered = Number(list.start || 1);
      const emit = (items: Tokens.ListItem[]) => {
        if (items.length) {
          html([{ ...list, start: list.ordered ? numbered : list.start, items } as Token]);
          numbered += items.length;
        }
      };
      for (const item of list.items) {
        if (!hasCode(item.tokens)) {
          run.push(item);
          continue;
        }
        let pending: Token[] = [];
        let opened = false;
        const flush = () => {
          if (!pending.length) {
            return;
          }
          if (opened) {
            html(pending);
          } else {
            run.push({ ...item, tokens: pending });
            emit(run);
            run = [];
            opened = true;
          }
          pending = [];
        };
        for (const part of item.tokens) {
          if (part.type === "code") {
            flush();
            code(part as Tokens.Code, true);
          } else {
            pending.push(part);
          }
        }
        flush();
        if (!opened) {
          numbered += 1;
        }
      }
      emit(run);
    } else {
      html([token]);
    }
  }
  return { title, intro, sections, headings };
}

/** A guide as the site shows it, read from the repository when the site is built. */
export function guide(name: GuideName, markdown: string = repositoryFile(GUIDES[name].file)): Guide {
  return readGuide(markdown, GUIDES[name].from);
}

/** Every link in a guide that leads to a heading, as the page it is on and its anchor. */
export function guideAnchors(markdown: string, own: string): { page: string; id: string }[] {
  const marked = new Marked({ gfm: true, async: false });
  const found: { page: string; id: string }[] = [];
  marked.walkTokens(marked.lexer(markdown), (token) => {
    if (token.type !== "link") {
      return;
    }
    const href = guideLink((token as Tokens.Link).href);
    const [page, id] = href.split("#");
    if (id !== undefined && (page === "" || Object.values(GUIDES).some((one) => one.path === page))) {
      found.push({ page: page === "" ? own : page, id });
    }
  });
  return found;
}
