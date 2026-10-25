/**
 * The built site in out/, for the tests that read what a visitor is sent. They skip, saying why,
 * until `npm run build` has made it; CI's site job builds before it tests.
 */

import { existsSync, readdirSync, readFileSync } from "node:fs";

/** Why a test of the built site is skipped. */
export const NOT_BUILT = "run npm run build first: this reads the built page";

const OUT = new URL("../out/", import.meta.url);

/** Whether `npm run build` has made the site. */
export const built = existsSync(new URL("index.html", OUT));

/** One built file: its path in out/, such as "pro.html", and its text. */
export interface BuiltFile {
  path: string;
  text: string;
}

/** Every built file whose name ends with one of `endings`, by path; none before the build. */
export function builtFiles(...endings: string[]): BuiltFile[] {
  if (!built) {
    return [];
  }
  return readdirSync(OUT, { recursive: true, encoding: "utf8" })
    .filter((path) => endings.some((ending) => path.endsWith(ending)))
    .sort()
    .map((path) => ({ path, text: readFileSync(new URL(path, OUT), "utf8") }));
}
