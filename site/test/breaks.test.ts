import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import { breakPieces, breakTokens } from "../lib/breaks.ts";

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

test("a command is spaces and words, and each word keeps its flags and versions in one piece", () => {
  const tokens = breakTokens("shasum -a 256 -c SHA256SUMS.txt --ignore-missing");
  assert.deepEqual(
    tokens.filter((token) => "pieces" in token).map((token) => ("pieces" in token ? token.pieces : [])),
    [["shasum"], ["-a"], ["256"], ["-c"], ["SHA256SUMS.txt"], ["--ignore-missing"]],
  );
  assert.ok(tokens.some((token) => "space" in token && token.space === " "), "the spaces stay places to break");
  const [url] = breakTokens("https://github.com/pghmatt/quotadial/releases/latest/download/QuotaDial_universal.dmg");
  assert.ok("pieces" in url && url.pieces.includes("latest/") && url.pieces.length > 5, "an address is one word in many pieces");
});

test("Breakable sets every piece unbreakable, so the browser breaks only at a space or a mark", () => {
  const source = readFileSync(new URL("../components/Breakable.tsx", import.meta.url), "utf8");
  assert.match(source, /<span className="whitespace-nowrap">\{piece\}<\/span>/);
  assert.match(source, /<wbr \/>/);
  assert.doesNotMatch(source, /overflow-wrap:\s*anywhere|\[overflow-wrap:anywhere\]/);
});
