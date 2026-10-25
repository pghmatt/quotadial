/**
 * CHANGELOG.md in each shape a release gives it, built from the real file, so the changelog tests
 * hold on the version bump and after it as they do today: the file as it is; the file as the
 * version bump leaves it (docs/RELEASE.md, step 1), its Unreleased heading renamed to the release
 * after the newest one (0.3.0 while that is 0.2.0), dated the day after; and that with an empty
 * Unreleased heading above the new release.
 */

import { repositoryFile } from "../lib/markdown.ts";

/** One shape of the changelog, and what a failure calls it. */
export interface ChangelogCase {
  name: string;
  markdown: string;
}

const UNRELEASED = /^## \[Unreleased\]\n/m;
const NEWEST_RELEASE = /^## \[(\d+)\.(\d+)\.\d+\] - (\d{4}-\d{2}-\d{2})\n/m;

/** The heading the version bump gives the release after the newest one in `markdown`. */
function nextRelease(markdown: string): string {
  const newest = markdown.match(NEWEST_RELEASE);
  if (!newest) {
    throw new Error("The changelog names no release in the form ## [0.2.0] - 2026-10-24.");
  }
  const day = new Date(`${newest[3]}T00:00:00Z`);
  day.setUTCDate(day.getUTCDate() + 1);
  return `## [${newest[1]}.${Number(newest[2]) + 1}.0] - ${day.toISOString().slice(0, 10)}`;
}

/**
 * The shapes of `markdown` to run the changelog tests over. Without an Unreleased heading the file
 * is already as a version bump leaves it, so there is no bump to make.
 */
export function changelogCases(markdown: string = repositoryFile("CHANGELOG.md")): ChangelogCase[] {
  const cases: ChangelogCase[] = [{ name: "CHANGELOG.md", markdown }];
  let bumped = markdown;
  if (UNRELEASED.test(markdown)) {
    const heading = nextRelease(markdown);
    bumped = markdown.replace(UNRELEASED, `${heading}\n`);
    cases.push({ name: `CHANGELOG.md after the version bump, "${heading}"`, markdown: bumped });
  }
  const reopened = bumped.replace(NEWEST_RELEASE, (release) => `## [Unreleased]\n\n${release}`);
  if (reopened === bumped) {
    throw new Error("The changelog names no release to put an empty Unreleased heading above.");
  }
  cases.push({ name: "CHANGELOG.md after the version bump, with an empty Unreleased heading above it", markdown: reopened });
  return cases;
}
