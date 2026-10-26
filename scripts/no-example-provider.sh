#!/usr/bin/env bash
# Fails when a built `quotadial` names the example build's provider (D-117), whose made-up accounts
# are for pictures of the app and must never reach a user: by its id, `fake`, as JSON names a
# provider, or by its own name, `Example`, as text does (P-020). The Release workflow runs it on
# every CLI it ships. The CLI runs in a scratch home and data folder, where it finds no sign-in and so
# asks no one; a missing sign-in is fine here, since only what the CLI names is checked. Each run is
# stopped after a minute rather than left to hang.
#
# Usage:
#   scripts/no-example-provider.sh target/release/quotadial
set -euo pipefail

if [[ $# -ne 1 || ! -f "$1" ]]; then
  echo "usage: scripts/no-example-provider.sh <the built quotadial>" >&2
  exit 2
fi
cli="$1"
limit=60

scratch="$(mktemp -d "${RUNNER_TEMP:-${TMPDIR:-/tmp}}/quotadial-release-check.XXXXXX")"
trap 'rm -rf "$scratch"' EXIT
mkdir -p "$scratch/home" "$scratch/data"

# A path as the CLI's own system writes it: on Windows the CLI is not an MSYS program.
native() {
  if command -v cygpath > /dev/null 2>&1; then
    cygpath -w "$1"
  else
    printf '%s\n' "$1"
  fi
}
home="$(native "$scratch/home")"
data="$(native "$scratch/data")"

# Runs the CLI with "$@", its output in $scratch/out and $scratch/err, where no sign-in can be
# found: a scratch home, no token in the environment, Gemini CLI's and GitHub CLI's folders in that
# home rather than where GEMINI_CLI_HOME or GH_CONFIG_DIR may point, and either build's data folder
# in scratch.
run() {
  env -u CLAUDE_CODE_OAUTH_TOKEN -u CLAUDE_SECURESTORAGE_CONFIG_DIR -u ANTHROPIC_API_KEY \
    -u OPENAI_API_KEY -u QUOTADIAL_FAKE_ACCOUNTS -u GEMINI_CLI_HOME \
    -u GH_CONFIG_DIR -u GH_TOKEN -u GITHUB_TOKEN -u COPILOT_GITHUB_TOKEN \
    HOME="$home" USERPROFILE="$home" \
    XDG_DATA_HOME="$home" XDG_CACHE_HOME="$home" XDG_CONFIG_HOME="$home" \
    CLAUDE_CONFIG_DIR="$(native "$scratch/home/.claude")" \
    CODEX_HOME="$(native "$scratch/home/.codex")" \
    QUOTADIAL_DATA_DIR="$data" QUOTADIAL_FAKE_DATA_DIR="$data" \
    "$cli" "$@" > "$scratch/out" 2> "$scratch/err" < /dev/null &
  local pid=$! waited=0
  while kill -0 "$pid" 2> /dev/null; do
    if ((waited >= limit * 10)); then
      kill "$pid" 2> /dev/null || true
      wait "$pid" 2> /dev/null || true
      echo "error: \`quotadial $*\` did not finish within $limit seconds, so it was stopped." >&2
      return 1
    fi
    sleep 0.1
    waited=$((waited + 1))
  done
  # Its exit status says whether it found sign-ins, which do not matter here.
  wait "$pid" || true
}

# The example build's provider, by its id or its name.
example='fake|Example'

# Checks what `quotadial $1` printed: something that starts as $2 says, and no example provider.
check() {
  local command="$1" printed="$2"
  if ! grep -q "$printed" "$scratch/out"; then
    echo "error: \`quotadial $command\` printed nothing to check. What it said:" >&2
    cat "$scratch/err" >&2
    return 1
  fi
  if grep -qwE "$example" "$scratch/out" "$scratch/err"; then
    echo "error: \`quotadial $command\` names the example build's provider, fake or Example (D-117):" >&2
    grep -hwE "$example" "$scratch/out" "$scratch/err" >&2
    return 1
  fi
  echo "\`quotadial $command\` names no example provider."
}

failed=0
run status --json
check "status --json" '"snapshots"' || failed=1
run doctor
check doctor '^QuotaDial ' || failed=1
exit "$failed"
