#!/usr/bin/env bash
# Builds docs/PRD-public.md from the private docs/PRD.md, following docs/PUBLIC_SCOPE.md. It removes:
#   - the "Competitive teardown" section, up to the next level-1 or level-2 heading
#   - every line with a dollar amount or the words "Pro price", "lifetime" or "per month"
#   - "Open questions" checklist items about price or open-sourcing
#
# Usage:
#   scripts/public-prd.sh           regenerate docs/PRD-public.md (needs the private docs/PRD.md)
#   scripts/public-prd.sh --check   fail if docs/PRD-public.md contains anything those rules remove
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
private_prd="$repo_root/docs/PRD.md"
public_prd="$repo_root/docs/PRD-public.md"
header='<!-- Generated from the private docs/PRD.md by scripts/public-prd.sh. Do not edit by hand; see docs/PUBLIC_SCOPE.md. -->'

# mode=keep prints the lines that stay public. mode=removed prints each removed line as
# "<line number><TAB><rule><TAB><text>".
apply_rules() {
  local mode="$1" file="$2"
  awk -v mode="$mode" '
    {
      sub(/\r$/, "")
      line = $0
      l = tolower(line)
      if (l ~ /^#+ /) {
        in_open = (l ~ /^#+ open questions/)
        if (l ~ /^##? /) in_teardown = (l ~ /^##? competitive teardown/)
      } else if (l ~ /^\*\*[^*]+\*\*[ \t]*$/) {
        in_open = (l ~ /^\*\*open questions\*\*/)
      }
      rule = ""
      if (in_teardown) {
        rule = "competitive teardown section"
      } else if (l ~ /[$] ?[0-9]/ || l ~ /pro price|lifetime|per month/) {
        rule = "pricing"
      } else if (in_open && l ~ /^[ \t]*[-*] \[[ x]\]/ && (l ~ /pric/ || l ~ /open[- ]?sourc/)) {
        rule = "open question about price or open-sourcing"
      }
      if (mode == "removed") {
        if (rule != "") printf "%d\t%s\t%s\n", NR, rule, line
        next
      }
      if (rule != "") next
      if (line ~ /^[ \t]*$/) {
        if (blank) next
        blank = 1
      } else {
        blank = 0
      }
      print line
    }
  ' "$file"
}

check_public() {
  if [[ ! -f "$public_prd" ]]; then
    echo "error: docs/PRD-public.md is missing. Run scripts/public-prd.sh next to the private PRD and commit the output." >&2
    return 1
  fi
  local leaks
  leaks="$(apply_rules removed "$public_prd")"
  if [[ -n "$leaks" ]]; then
    echo "error: docs/PRD-public.md contains private content (line, rule, text):" >&2
    printf '%s\n' "$leaks" >&2
    return 1
  fi
  echo "docs/PRD-public.md contains no private content."
}

generate() {
  if [[ ! -f "$private_prd" ]]; then
    echo "error: docs/PRD.md not found. The private PRD only exists on the maintainer's machine." >&2
    return 1
  fi
  if git -C "$repo_root" ls-files --error-unmatch docs/PRD.md >/dev/null 2>&1; then
    echo "error: docs/PRD.md is tracked by git. Untrack it (git rm --cached docs/PRD.md) before publishing anything." >&2
    return 1
  fi
  local tmp
  tmp="$(mktemp)"
  {
    printf '%s\n\n' "$header"
    apply_rules keep "$private_prd"
  } >"$tmp"
  cat "$tmp" >"$public_prd"
  rm -f "$tmp"
  echo "Wrote docs/PRD-public.md. Removed from the private PRD:"
  apply_rules removed "$private_prd" |
    awk -F '\t' '{ count[$2]++ } END { for (rule in count) printf "  %3d lines: %s\n", count[rule], rule }'
  check_public
}

case "${1:-}" in
  "") generate ;;
  --check) check_public ;;
  *)
    echo "usage: scripts/public-prd.sh [--check]" >&2
    exit 2
    ;;
esac
