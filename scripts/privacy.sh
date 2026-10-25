#!/usr/bin/env bash
# Copies docs/PRIVACY.md, what QuotaDial reads and sends, into README.md between its two privacy
# markers, one heading level down, so the README says exactly what the app's Help and the website
# say (D-106). Edit docs/PRIVACY.md, then run this.
#
# Usage:
#   scripts/privacy.sh           write the copy into README.md
#   scripts/privacy.sh --check   fail if README.md's copy differs from docs/PRIVACY.md
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source_file="$repo_root/docs/PRIVACY.md"
readme="$repo_root/README.md"
start='<!-- privacy: copied from docs/PRIVACY.md by scripts/privacy.sh; edit it there -->'
end='<!-- /privacy -->'

for marker in "$start" "$end"; do
  if [[ "$(grep -cxF -- "$marker" "$readme")" != 1 ]]; then
    echo "README.md needs this line exactly once: $marker" >&2
    exit 1
  fi
done

section="$(mktemp)"
written="$(mktemp)"
trap 'rm -f "$section" "$written"' EXIT
sed '1s/^# /## /' "$source_file" > "$section"

awk -v start="$start" -v end="$end" -v section="$section" '
  $0 == start { print; print ""; while ((getline line < section) > 0) print line; print ""; skipping = 1; next }
  $0 == end { skipping = 0 }
  !skipping { print }
' "$readme" > "$written"

if [[ "${1:-}" == "--check" ]]; then
  if ! diff -u "$readme" "$written"; then
    echo "README.md's privacy section differs from docs/PRIVACY.md: run scripts/privacy.sh." >&2
    exit 1
  fi
else
  cp "$written" "$readme"
fi
