#!/usr/bin/env bash
# Prints one release's notes from CHANGELOG.md: its section, without the heading, so no release
# note is written twice (D-101). The Release workflow puts them on the release it drafts.
#
# Usage:
#   scripts/release-notes.sh <version>     such as 0.3.0, without the v
set -euo pipefail

version="${1:?usage: scripts/release-notes.sh <version>}"
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

notes="$(awk -v heading="## [$version]" '
  index($0, heading) == 1 { found = 1; next }
  found && (/^## \[/ || /^\[[^]]+\]: /) { exit }
  found { lines[++n] = $0 }
  END {
    first = 1
    while (first <= n && lines[first] == "") first++
    last = n
    while (last >= first && lines[last] == "") last--
    for (i = first; i <= last; i++) print lines[i]
  }
' "$repo_root/CHANGELOG.md")"

if [[ -z "$notes" ]]; then
  echo "CHANGELOG.md has no notes for $version: add a \"## [$version]\" section before tagging." >&2
  exit 1
fi
printf '%s\n' "$notes"
