// The contrast check (check-contrast.mjs) as `npm run build` runs it: it passes on the stylesheet as
// it is, and with each run it says the share picture's chart line on its own soft fill, in both
// themes, measured and not held (D-140), so that the figure is seen if it moves. What it holds, the
// line on its card at 3:1 among them, it holds by failing the build.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

test("the contrast check passes, and says the share picture's chart line on its own soft fill in both themes, measured and not held (D-140)", () => {
  const run = spawnSync(process.execPath, [fileURLToPath(new URL("./check-contrast.mjs", import.meta.url))], {
    encoding: "utf8",
  });
  assert.equal(run.status, 0, run.stderr);
  assert.match(run.stdout, /^Every colour pair meets WCAG AA in both themes \(\d+ pairs each\)\.$/m);
  // The figures are what the stylesheet gives, and not held: the line is judged on its card, its fill
  // being part of it, so only that each theme is measured is held here.
  const said = run.stdout.match(
    /^Measured, not held: the share picture's chart line on its own soft fill, which is part of it \(D-140\), (\d+\.\d\d):1 in light and (\d+\.\d\d):1 in dark\.$/m,
  );
  assert.ok(said, `the check does not say the line on its fill in both themes:\n${run.stdout}`);
  for (const figure of said.slice(1)) {
    assert.ok(Number(figure) >= 1, `${figure}:1 is not a contrast ratio`);
  }
});
