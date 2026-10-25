import assert from "node:assert/strict";
import { test } from "node:test";

import { breakPieces } from "../lib/breaks.ts";

test("an address breaks after its slashes and its words' hyphens, never inside a word", () => {
  const url = "https://github.com/pghmatt/quotadial/releases/latest/download/quotadial-aarch64-apple-darwin.tar.gz";
  const pieces = breakPieces(url);
  assert.equal(pieces.join(""), url, "nothing is added or lost");
  assert.ok(pieces.includes("latest/"), "latest stays whole");
  assert.ok(pieces.includes("quotadial-"), "a file name breaks after a hyphen inside it");
  assert.ok(pieces.every((piece) => !/^\/|^-/.test(piece)), "no piece starts with the mark it broke after");
});

test("a command's flag and a version stay whole", () => {
  const pieces = breakPieces("shasum -a 256 -c SHA256SUMS.txt --ignore-missing");
  assert.equal(pieces.join(""), "shasum -a 256 -c SHA256SUMS.txt --ignore-missing");
  assert.equal(pieces.length, 1, "nothing in it breaks inside a word");
  assert.deepEqual(breakPieces("quotadial-v0.2.0-x86_64-pc-windows-msvc.zip"), [
    "quotadial-",
    "v0.2.0-",
    "x86_",
    "64-",
    "pc-",
    "windows-",
    "msvc.zip",
  ]);
});

test("spaces and line ends are kept, and a mark at the end makes no empty piece", () => {
  assert.equal(breakPieces("sudo mkdir -p /usr/local/bin\nsudo ln -sf a b").join(""), "sudo mkdir -p /usr/local/bin\nsudo ln -sf a b");
  assert.deepEqual(breakPieces("folder/"), ["folder/"]);
  assert.deepEqual(breakPieces(""), []);
});
