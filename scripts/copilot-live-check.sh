#!/usr/bin/env bash
# Copilot's live check on the owner's Mac (#51, D-127): steps 1 to 11 of "The live check" in PR 2's
# notes in docs/TASK_TRACKER.md, run in order, saying what to click when macOS asks and writing every
# result by step number to fixtures/copilot/raw/results.md.
#
# No token is ever printed, logged or saved: each request takes the sign-in from `security ... -w` or
# `gh auth token` straight into `curl --config -` on standard input, never onto a command line.
# GitHub's answers stay in fixtures/copilot/raw/, which git ignores; only the redacted answer is
# written outside it, as fixtures/copilot/user-live.json. GitHub's usage endpoint is asked at most
# once every 5 minutes, as QuotaDial asks it.
#
# Usage:
#   scripts/copilot-live-check.sh            the check, about 25 minutes, most of it waiting
#   scripts/copilot-live-check.sh --dry-run  the same flow against a stand-in server on this Mac,
#                                            with made-up sign-ins and short waits: no Keychain, no
#                                            GitHub CLI or Copilot CLI, no cargo and nothing sent to
#                                            GitHub; its files go in a new temporary folder
set -uo pipefail

usage() {
  cat >&2 << 'USAGE'
usage: scripts/copilot-live-check.sh [--dry-run]
  With no option it runs the check on this Mac: about 25 minutes, most of it waiting.
  --dry-run runs the same flow against a stand-in server with made-up sign-ins: nothing
  reads the Keychain or GitHub CLI, nothing runs cargo and nothing is sent to GitHub.
USAGE
}

dry_run=no
case "${1:-}" in
  "") ;;
  --dry-run) dry_run=yes ;;
  -h | --help)
    usage
    exit 0
    ;;
  *)
    usage
    exit 2
    ;;
esac
if [[ $# -gt 1 ]]; then
  usage
  exit 2
fi

readonly SERVICE=copilot-cli
readonly REAL_ENDPOINT=https://api.github.com/copilot_internal/user
# VS Code's editor headers, as CodexBar sends them, for request C only.
readonly EDITOR_VERSION='Editor-Version: vscode/1.96.2'
readonly EDITOR_PLUGIN='Editor-Plugin-Version: copilot-chat/0.26.7'
readonly EDITOR_AGENT='User-Agent: GitHubCopilotChat/0.26.7'
# What the script reads from the person, and the last request's time, in seconds since 1970.
ANSWER=""
last_request_at=""
results=""
helper=""

dry() { [[ $dry_run == yes ]]; }

say() { printf '%s\n' "$@"; }

# One short line saying what the next step does.
step() { printf '\n%s\n' "$1"; }

# Stops the check with one plain sentence on what to do.
fail() {
  printf '\n%s\n' "$1" >&2
  if [[ -n $results && -f $results && -f $helper ]]; then
    record "" "Stopped: $1"
    printf 'What was found so far is in %s\n' "$results" >&2
  fi
  exit 1
}

# Appends its arguments, one line each, or standard input with none, to the results, with every
# login and organisation name found so far written as a placeholder and anything shaped like a token
# or an email address removed.
record() {
  if [[ $# -gt 0 ]]; then printf '%s\n' "$@"; else cat; fi | python3 "$helper" redact >> "$results"
}

# Adds a login or a name the results must not show, with the placeholder written in its place.
LIVE_CHECK_WORDS=""
export LIVE_CHECK_WORDS
forget() {
  [[ -n $1 ]] || return 0
  LIVE_CHECK_WORDS="${LIVE_CHECK_WORDS}${1}"$'\t'"${2}"$'\n'
  export LIVE_CHECK_WORDS
}

# Asks a yes or no question until the answer is one, setting ANSWER to yes or no.
ask() {
  local reply
  while :; do
    printf '%s (y/n) ' "$1"
    if ! IFS= read -r reply; then
      printf '\n'
      fail "No answer came, so the check stopped. Run it again to start over."
    fi
    [[ -t 0 ]] || printf '%s\n' "$reply"
    case "$reply" in
      [Yy] | [Yy][Ee][Ss])
        ANSWER=yes
        return 0
        ;;
      [Nn] | [Nn][Oo])
        ANSWER=no
        return 0
        ;;
      *) say "Please type y or n." ;;
    esac
  done
}

# Waits for Enter.
pause() {
  local reply
  printf '%s ' "${1:-Press Enter to go on.}"
  if ! IFS= read -r reply; then
    printf '\n'
    fail "No answer came, so the check stopped. Run it again to start over."
  fi
  [[ -t 0 ]] || printf '\n'
}

# Reads a line of free text into ANSWER; Enter alone leaves it empty.
ask_text() {
  printf '%s ' "$1"
  if ! IFS= read -r ANSWER; then
    ANSWER=""
  fi
  [[ -t 0 ]] || printf '%s\n' "$ANSWER"
}

# Runs a command for at most SECONDS, stopping it after that.
with_time_limit() {
  local seconds="$1" pid watcher status
  shift
  "$@" &
  pid=$!
  (sleep "$seconds" && kill "$pid" 2> /dev/null) > /dev/null 2>&1 &
  watcher=$!
  wait "$pid" 2> /dev/null
  status=$?
  kill "$watcher" 2> /dev/null
  wait "$watcher" 2> /dev/null
  return "$status"
}

# The real tools, which the dry run must never reach: each stops the check if it does.
run_security() {
  if dry; then fail "The dry run reached the Keychain, which it never may."; fi
  /usr/bin/security "$@"
}
run_gh() {
  if dry; then fail "The dry run reached GitHub CLI, which it never may."; fi
  gh "$@"
}
run_copilot() {
  if dry; then fail "The dry run reached the Copilot CLI, which it never may."; fi
  copilot "$@"
}
run_cargo() {
  if dry; then fail "The dry run reached cargo, which it never may."; fi
  cargo "$@"
}

# --- What the check needs -----------------------------------------------------------------------

[[ $(uname -s) == Darwin ]] ||
  fail "This check needs a Mac: run it on the Mac where the Copilot CLI or GitHub CLI is signed in."
script_dir=$(cd "$(dirname "$0")" && pwd) || fail "The script could not find its own folder."
repo=$(git -C "$script_dir" rev-parse --show-toplevel 2> /dev/null) ||
  fail "This script must stay in QuotaDial's scripts folder, inside its git checkout."
[[ -f $repo/crates/providers/copilot/Cargo.toml ]] ||
  fail "This checkout has no Copilot provider: switch to the feat/copilot branch and run this again."
command -v curl > /dev/null 2>&1 ||
  fail "curl is missing: install Apple's command line tools with xcode-select --install, then run this again."
python3 -c 'import json, re' > /dev/null 2>&1 ||
  fail "python3 is missing: install Apple's command line tools with xcode-select --install, then run this again."

if dry; then
  keychain_item=yes
  gh_installed=yes
  gh_signed_in=yes
  copilot_installed=yes
else
  [[ -x /usr/bin/security ]] ||
    fail "macOS's Keychain tool, /usr/bin/security, is missing, so this Mac cannot be checked."
  command -v cargo > /dev/null 2>&1 ||
    fail "cargo is missing: install Rust from https://rustup.rs, then run this again."
  # Its attributes only, which never asks: step 2 reads them again and records them.
  run_security find-generic-password -s "$SERVICE" > /dev/null 2>&1
  case $? in
    0) keychain_item=yes ;;
    44) keychain_item=no ;;
    36) fail "The login keychain is locked: unlock it in Keychain Access, then run this again." ;;
    *) fail "The Keychain could not be searched: restart the Mac, then run this again." ;;
  esac
  gh_installed=no
  gh_signed_in=no
  if command -v gh > /dev/null 2>&1; then
    gh_installed=yes
    if run_gh auth token --hostname github.com < /dev/null > /dev/null 2>&1; then
      gh_signed_in=yes
    fi
  fi
  copilot_installed=no
  if command -v copilot > /dev/null 2>&1; then copilot_installed=yes; fi
  if [[ $keychain_item == no && $gh_signed_in == no ]]; then
    fail "No Copilot sign-in was found: sign in to the Copilot CLI (run copilot, then /login) or to GitHub CLI (run gh auth login), then run this again."
  fi
fi

if dry; then
  out=$(mktemp -d "${TMPDIR:-/tmp}/copilot-live-check-dry-run.XXXXXX") ||
    fail "No temporary folder could be made for the dry run."
  fixture="$out/user-live.json"
  gap=3
else
  out="$repo/fixtures/copilot/raw"
  mkdir -p "$out" || fail "The folder $out could not be made."
  git -C "$repo" check-ignore -q "$out/results.md" ||
    fail "Git does not ignore fixtures/copilot/raw/ in this checkout, so nothing was saved: tell QuotaDial's Chat."
  fixture="$repo/fixtures/copilot/user-live.json"
  gap=300
fi
results="$out/results.md"
helper="$out/live-check-helper.py"
: > "$results"
# An earlier run's answers never stand in for this run's.
for name in a b c d; do
  : > "$out/$name.json"
  : > "$out/$name.headers"
  : > "$out/$name.code"
done

# The helper the steps share: it reads the Keychain's listing, GitHub CLI's file and GitHub's answers,
# writes the redacted fixture and what goes in the results, and never prints a token.
cat > "$helper" << 'PY'
"""Helpers for scripts/copilot-live-check.sh. None of them prints a token."""
import datetime
import json
import os
import re
import sys

TOKEN = re.compile(r"(?:gh[opusr]_|github_pat_)[A-Za-z0-9_*]{8,}")
EMAIL = re.compile(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9-]+(?:\.[A-Za-z0-9-]+)*")
LOGIN = re.compile(r"^[A-Za-z0-9][A-Za-z0-9-]{0,38}$")
HEADERS = (
    "content-type",
    "x-github-api-version-selected",
    "x-oauth-scopes",
    "x-accepted-oauth-scopes",
    "x-ratelimit-limit",
    "x-ratelimit-remaining",
    "x-ratelimit-used",
    "x-ratelimit-reset",
    "x-ratelimit-resource",
    "retry-after",
)


def read(path):
    with open(path, encoding="utf-8", errors="replace") as handle:
        return handle.read()


def write(path, text):
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)


def words():
    pairs = set()
    for line in os.environ.get("LIVE_CHECK_WORDS", "").splitlines():
        word, _, placeholder = line.partition("\t")
        if word.strip() and placeholder:
            pairs.add((word.strip(), placeholder))
    return sorted(pairs, key=lambda pair: -len(pair[0]))


def word_pattern(word):
    return r"(?<![A-Za-z0-9_-])" + re.escape(word) + r"(?![A-Za-z0-9_-])"


def redact_text(text):
    home = os.environ.get("HOME", "").rstrip("/")
    if home:
        text = re.sub(re.escape(home) + r"(?=/|\b|$)", "~", text)
    text = TOKEN.sub("<token removed>", text)
    text = EMAIL.sub("<email removed>", text)
    for word, placeholder in words():
        text = re.sub(word_pattern(word), lambda _: placeholder, text, flags=re.IGNORECASE)
    return text


def utc(seconds):
    moment = datetime.datetime.fromtimestamp(seconds, tz=datetime.timezone.utc)
    return moment.strftime("%Y-%m-%dT%H:%M:%SZ")


def leaves(value, path=()):
    if isinstance(value, dict):
        for key, inner in value.items():
            yield from leaves(inner, path + (str(key),))
    elif isinstance(value, list):
        for index, inner in enumerate(value):
            yield from leaves(inner, path + (str(index),))
    else:
        yield ".".join(path), value


def load(path):
    try:
        return json.loads(read(path))
    except ValueError:
        return None


def cmd_redact():
    sys.stdout.write(redact_text(sys.stdin.read()))


def cmd_redact_file(path):
    write(path, redact_text(read(path)))


def cmd_versions(repo):
    cargo = read(os.path.join(repo, "Cargo.toml"))
    section = re.search(r"^\[workspace\.package\]\s*$(.*?)(?=^\[|\Z)", cargo, re.M | re.S)
    version = section and re.search(r'^version\s*=\s*"([^"]+)"', section.group(1), re.M)
    api = read(os.path.join(repo, "crates/providers/copilot/src/api.rs"))
    api_version = re.search(r'API_VERSION: &str = "([^"]+)"', api)
    if not (version and api_version):
        return 1
    print(version.group(1), api_version.group(1))
    return 0


def holder(account):
    account = account.strip()
    if not account:
        return "", "empty, so the item names no account"
    if LOGIN.match(account):
        return account, "a login on its own"
    host, _, login = account.rpartition(":")
    if host and LOGIN.match(login):
        bare = host.replace("https://", "").replace("http://", "").rstrip("/")
        if bare.lower() == "github.com":
            return login, "github.com's address, then the login after a colon"
        return "", "another host than github.com, then a login"
    return "", "neither a login nor a host and a login"


def cmd_attributes(path):
    text = read(path)

    def blob(name):
        found = re.search(
            r'^\s*"%s"<blob>=(?:"(.*)"|0x[0-9A-Fa-f]+\s+"(.*)"|<NULL>)\s*$' % name, text, re.M
        )
        return (found.group(1) or found.group(2) or "") if found else ""

    modified = re.search(r'^\s*"mdat"<timedate>=\S+\s+"([^"]*)"', text, re.M)
    login, form = holder(blob("acct"))
    for key, value in (
        ("acct", blob("acct")),
        ("svce", blob("svce")),
        ("mdat", modified.group(1).replace("\\000", "") if modified else ""),
        ("login", login),
        ("form", form),
    ):
        print(key + "\t" + value)


def cmd_gh_status(source, target):
    """`gh auth status`'s text, from `source` or standard input for -, written to `target` without
    its token line, printing each login it names."""
    kept = []
    text = sys.stdin.read() if source == "-" else read(source)
    for line in text.splitlines():
        if re.search(r"\bToken:\s", line):
            continue
        kept.append(TOKEN.sub("<token removed>", line))
        found = re.search(r"Logged in to \S+ (?:account|as) ([A-Za-z0-9][A-Za-z0-9-]*)", line)
        if found:
            print("login\t" + found.group(1))
    write(target, "\n".join(kept) + "\n")


def cmd_hosts_layout(source, target):
    lines = read(source).splitlines()
    shown = [
        TOKEN.sub("<token removed>", re.sub(r"(oauth_token:).*", r"\1 <removed>", line))
        for line in lines
    ]
    write(target, "\n".join(shown) + "\n")
    block, inside = [], False
    for line in lines:
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        indent = len(line) - len(line.lstrip(" "))
        if indent == 0:
            inside = line.strip().strip("'\"").rstrip(":").strip("'\"") == "github.com"
            continue
        if inside:
            block.append((indent, line.strip()))
    if not block:
        print("finding\tIt has no github.com entry.")
        return
    top = min(indent for indent, _ in block)
    keys = [text.split(":", 1)[0].strip() for indent, text in block if indent == top]
    print("finding\tKeys directly under github.com: " + ", ".join(keys) + ".")
    for key in ("user", "oauth_token"):
        print("finding\t`%s` directly under github.com: %s." % (key, "yes" if key in keys else "no"))
    for indent, text in block:
        key, _, value = text.partition(":")
        if indent == top and key.strip() == "user" and value.strip():
            print("login\t" + value.strip().strip("'\""))
    users, under = [], False
    for indent, text in block:
        if indent == top:
            under = text.split(":", 1)[0].strip() == "users"
            continue
        if under:
            users.append((indent, text))
    if users:
        level = min(indent for indent, _ in users)
        held, current = [], None
        for indent, text in users:
            key = text.split(":", 1)[0].strip()
            if indent == level:
                current = [key, []]
                held.append(current)
                print("login\t" + key.strip("'\""))
            elif current is not None:
                current[1].append(key)
        described = [
            "%s (%s)" % (login, ", ".join(inner) if inner else "nothing under it")
            for login, inner in held
        ]
        print("finding\t`users:` holds " + "; ".join(described) + ".")
    else:
        print("finding\tThere is no `users:` under github.com.")


def cmd_headers(path):
    text = read(path).replace("\r", "")
    blocks = [block for block in text.split("\n\n") if block.strip()]
    if not blocks:
        print("none")
        return
    found = []
    for line in blocks[-1].splitlines()[1:]:
        name, _, value = line.partition(":")
        name, value = name.strip().lower(), value.strip()
        if name in HEADERS:
            if name == "x-ratelimit-reset" and value.isdigit():
                value += " (" + utc(int(value)) + ")"
            found.append(name + ": " + value)
    print("; ".join(found) if found else "none of the rate-limit headers")


def cmd_message(path):
    answer = load(path)
    if isinstance(answer, dict) and isinstance(answer.get("message"), str):
        print(answer["message"])
    elif read(path).strip():
        print("(an answer that is not GitHub's JSON error)")
    else:
        print("(an empty answer)")


def cmd_compare(first, second):
    one, two = load(first), load(second)
    if one is None or two is None:
        print("one of the answers is not JSON, so their fields cannot be compared")
        return
    first_paths = {path for path, _ in leaves(one)}
    second_paths = {path for path, _ in leaves(two)}
    if first_paths == second_paths:
        print("the same fields")
        return
    parts = []
    if second_paths - first_paths:
        parts.append("only in D: " + ", ".join(sorted(second_paths - first_paths)))
    if first_paths - second_paths:
        parts.append("only in the first: " + ", ".join(sorted(first_paths - second_paths)))
    print("different fields: " + "; ".join(parts))


# Keys whose values name an organisation or an enterprise, and ids that name a kind, not someone.
ORG_KEYS = ("org", "enterprise")
SAFE_ID_KEYS = ("quota_id",)


def cmd_fixture(source, target):
    """GitHub's answer at `source` written to `target` with the login, every organisation's login
    and name, and every id made up, an email address or token-like text taken out, and every other
    number and date kept. Prints what it replaced, for the results to leave out too."""
    answer = load(source)
    if not isinstance(answer, dict):
        return 1
    names, ids = {}, {}

    def organisation(real, kind):
        if real not in names:
            count = 1 + sum(1 for _, made in names.values() if made == kind)
            if kind == "login":
                fake = "example-org" if count == 1 else "example-org-%d" % count
            else:
                fake = "Example Org" if count == 1 else "Example Org %d" % count
            names[real] = (fake, kind)
        return names[real][0]

    def made_up_id(real):
        if isinstance(real, bool) or not isinstance(real, (int, str)):
            return real
        key = str(real)
        if key not in ids:
            numbers = sum(1 for made in ids.values() if isinstance(made, int))
            ids[key] = numbers + 1 if isinstance(real, int) else re.sub(r"[A-Za-z0-9]", "0", real)
        return ids[key]

    def walk(value, key, in_organisation):
        lowered = key.lower()
        inside = in_organisation or any(word in lowered for word in ORG_KEYS)
        if isinstance(value, dict):
            return {inner_key: walk(inner, inner_key, inside) for inner_key, inner in value.items()}
        if isinstance(value, list):
            return [
                organisation(item, "login") if inside and isinstance(item, str)
                else walk(item, key, inside)
                for item in value
            ]
        if lowered not in SAFE_ID_KEYS and (lowered == "id" or lowered.endswith("_id")):
            return made_up_id(value)
        if isinstance(value, str):
            if in_organisation and lowered in ("login", "slug"):
                return organisation(value, "login")
            if in_organisation and lowered in ("name", "display_name", "description"):
                return organisation(value, "name")
            if "@" in value:
                return "example-email-removed"
            if TOKEN.search(value):
                return "example-token-removed"
        return value

    real_login = answer.get("login") if isinstance(answer.get("login"), str) else ""
    redacted = {key: walk(value, key, False) for key, value in answer.items()}
    if real_login:
        redacted["login"] = "example-user"
    for key in ("name", "display_name", "full_name"):
        if isinstance(redacted.get(key), str):
            redacted[key] = "Example User"
    replacements = [(real_login, "example-user")] if real_login else []
    replacements += [(real, fake) for real, (fake, _) in names.items()]
    replacements += [(real, str(fake)) for real, fake in ids.items() if len(real) >= 6]
    replacements.sort(key=lambda pair: -len(pair[0]))

    def scrub(value):
        if isinstance(value, dict):
            return {key: scrub(inner) for key, inner in value.items()}
        if isinstance(value, list):
            return [scrub(inner) for inner in value]
        if isinstance(value, str):
            for real, fake in replacements:
                value = re.sub(word_pattern(real), lambda _: fake, value, flags=re.IGNORECASE)
        return value

    redacted = scrub(redacted)
    write(target, json.dumps(redacted, indent=2, sort_keys=True, ensure_ascii=False) + "\n")
    if real_login:
        print("login\t" + real_login)
    for real, (_, kind) in names.items():
        print(("org" if kind == "login" else "orgname") + "\t" + real)
    for real in ids:
        if len(real) >= 6:
            print("id\t" + real)
    return 0


def cmd_check_fixture(path):
    text = read(path)
    problems = []
    if TOKEN.search(text):
        problems.append("something shaped like a GitHub token")
    if "@" in text:
        problems.append("an @")
    for word, _ in words():
        if re.search(word_pattern(word), text, re.IGNORECASE):
            problems.append("a login, name or id from the real answer")
            break
    if problems:
        print("; ".join(problems))
        return 1
    return 0


def describe(key, value):
    shown = json.dumps(value)
    if key.endswith("_at") and isinstance(value, (int, float)) and not isinstance(value, bool):
        shown += " (" + utc(value) + ")"
    return "`%s` %s" % (key, shown)


def cmd_summary(path):
    answer = load(path)
    lines = ["- Has `login`: " + ("yes" if "login" in answer else "no")]
    for key in ("copilot_plan", "access_type_sku", "token_based_billing"):
        lines.append("- `%s`: %s" % (key, json.dumps(answer[key]) if key in answer else "absent"))
    snapshots = answer.get("quota_snapshots")
    if isinstance(snapshots, dict):
        for name in sorted(snapshots):
            snapshot = snapshots[name]
            if isinstance(snapshot, dict):
                fields = ", ".join(describe(key, snapshot[key]) for key in sorted(snapshot))
                lines.append("- `quota_snapshots.%s`: %s" % (name, fields))
    else:
        lines.append("- `quota_snapshots`: absent")
    for key in ("monthly_quotas", "limited_user_quotas", "limited_user_reset_date"):
        if key in answer:
            lines.append("- `%s`: %s" % (key, json.dumps(answer[key], sort_keys=True)))
    resets = [describe(path, value) for path, value in leaves(answer) if "reset" in path.lower()]
    lines.append("- Reset fields: " + ("; ".join(resets) if resets else "none"))
    organisations = answer.get("organization_login_list")
    if isinstance(organisations, list):
        lines.append("- Organisations in `organization_login_list`: %d" % len(organisations))
    lines.append("- Every field: " + ", ".join("`%s`" % path for path, _ in leaves(answer)))
    print("\n".join(lines))


def cmd_status_summary(path):
    report = load(path)
    if not isinstance(report, dict):
        print("- `status --json` gave no JSON.")
        return
    lines = []
    for snapshot in report.get("snapshots", []):
        account = snapshot.get("account") or {}
        if account.get("provider") != "copilot":
            continue
        lines.append(
            "- Account `%s` (%s), plan %s, status %s"
            % (
                account.get("account_id"),
                account.get("display_name"),
                account.get("plan"),
                snapshot.get("status"),
            )
        )
        lines.append("  - sign_in: %s" % account.get("sign_in"))
        lines.append("  - sign_in_use: %s" % account.get("sign_in_use"))
        if snapshot.get("error"):
            lines.append("  - error: %s" % snapshot["error"])
        for window in snapshot.get("windows", []):
            lines.append(
                "  - window %s (%s): %s%% used, resets %s, over %s s, from %s%s"
                % (
                    window.get("label"),
                    window.get("kind"),
                    window.get("used_pct"),
                    window.get("resets_at"),
                    window.get("window_seconds"),
                    window.get("source"),
                    ", unconfirmed" if window.get("unconfirmed") else "",
                )
            )
    for error in report.get("errors", []):
        if error.get("provider") == "copilot":
            lines.append("- Error: %s" % error.get("message"))
    print("\n".join(lines) if lines else "- No Copilot account and no Copilot error in the report.")


def cmd_doctor_copilot(path):
    lines, keep = [], False
    for line in read(path).splitlines():
        if line.startswith("  ") and not line.startswith("   "):
            keep = line.strip().startswith("Copilot")
        elif not line.startswith("    "):
            keep = False
        if keep or "copilot" in line.lower():
            lines.append("    " + line)
    print("\n".join(lines) if lines else "    (doctor named no Copilot account)")


COMMANDS = {
    "redact": cmd_redact,
    "redact-file": cmd_redact_file,
    "versions": cmd_versions,
    "attributes": cmd_attributes,
    "gh-status": cmd_gh_status,
    "hosts-layout": cmd_hosts_layout,
    "headers": cmd_headers,
    "message": cmd_message,
    "compare": cmd_compare,
    "fixture": cmd_fixture,
    "check-fixture": cmd_check_fixture,
    "summary": cmd_summary,
    "status-summary": cmd_status_summary,
    "doctor-copilot": cmd_doctor_copilot,
}

if __name__ == "__main__":
    sys.exit(COMMANDS[sys.argv[1]](*sys.argv[2:]) or 0)
PY

versions=$(python3 "$helper" versions "$repo") ||
  fail "QuotaDial's version or Copilot's API version could not be read from this checkout: tell QuotaDial's Chat."
version=${versions%% *}
api_version=${versions##* }

# --- How the steps talk to the Keychain, GitHub CLI and GitHub ----------------------------------

# `path` with the home folder written ~.
tilde() {
  case "$1" in
    "$HOME"/*) printf '~%s\n' "${1#"$HOME"}" ;;
    *) printf '%s\n' "$1" ;;
  esac
}

upper() { printf '%s' "$1" | tr '[:lower:]' '[:upper:]'; }

# What `security`'s exit status says.
read_meaning() {
  case "$1" in
    0) echo "read, and sent nowhere" ;;
    51) echo "refused (errSecAuthFailed)" ;;
    128) echo "refused (errSecUserCanceled)" ;;
    36) echo "the keychain is locked or could not ask" ;;
    44) echo "there is no such item" ;;
    *) echo "another error" ;;
  esac
}

# The first line of a tool's error output, in brackets, or nothing when it said nothing.
said() {
  local line=""
  [[ -s $1 ]] && IFS= read -r line < "$1"
  if [[ -n $line ]]; then printf ' (it said: %s)' "$line"; fi
}

# Reads the Copilot CLI's secret as QuotaDial reads it, onto standard output, which only ever goes
# to /dev/null or into the pipe to curl.
read_item() {
  if [[ -n $acct ]]; then
    run_security find-generic-password -a "$acct" -w -s "$SERVICE"
  else
    run_security find-generic-password -w -s "$SERVICE"
  fi
}

# Prints the sign-in SOURCE names on standard output, for the pipe into curl and nowhere else: the
# Copilot CLI's from the Keychain (keychain) or GitHub CLI's from `gh auth token` (gh).
sign_in() {
  if dry; then
    case "$1" in
      keychain) printf '%s\n' dry_run_keychain_token ;;
      gh) printf '%s\n' dry_run_gh_token ;;
    esac
    return 0
  fi
  case "$1" in
    keychain) read_item ;;
    gh) run_gh auth token --hostname github.com < /dev/null ;;
  esac
}

# Turns the sign-in on standard input into curl's settings, the Authorization header and the URL
# together, so that with no sign-in curl has nowhere to send a request. Exits 1 when none came and
# 3 when what came is not a token, printing nothing either way.
as_curl_settings() {
  LC_ALL=C awk -v url="$1" '
    !seen && NF { seen = 1; token = $0 }
    END {
      if (!seen) exit 1
      gsub(/^[ \t\r]+|[ \t\r]+$/, "", token)
      if (token !~ /^[!-~]+$/ || token ~ /["\\]/) exit 3
      printf "header = \"Authorization: token %s\"\nurl = \"%s\"\n", token, url
    }'
}

# The headers of the header set NAME, as curl's arguments, in HEADER_ARGS.
headers_for() {
  case "$1" in
    quotadial)
      HEADER_ARGS=(-H 'Accept: application/json' -H "X-GitHub-Api-Version: $api_version"
        -H "User-Agent: quotadial/$version")
      ;;
    quotadial-unversioned)
      HEADER_ARGS=(-H 'Accept: application/json' -H "User-Agent: quotadial/$version")
      ;;
    editor)
      HEADER_ARGS=(-H 'Accept: application/json' -H "X-GitHub-Api-Version: $api_version"
        -H "$EDITOR_VERSION" -H "$EDITOR_PLUGIN" -H "$EDITOR_AGENT")
      ;;
    editor-unversioned)
      HEADER_ARGS=(-H 'Accept: application/json' -H "$EDITOR_VERSION" -H "$EDITOR_PLUGIN"
        -H "$EDITOR_AGENT")
      ;;
  esac
}

describe_headers() {
  case "$1" in
    quotadial)
      echo "QuotaDial's: \`Accept: application/json\`, \`X-GitHub-Api-Version: $api_version\`, \`User-Agent: quotadial/$version\`"
      ;;
    quotadial-unversioned)
      echo "QuotaDial's without \`X-GitHub-Api-Version\`: \`Accept: application/json\`, \`User-Agent: quotadial/$version\`"
      ;;
    editor)
      echo "VS Code's in place of QuotaDial's \`User-Agent\`: \`Accept: application/json\`, \`X-GitHub-Api-Version: $api_version\`, \`$EDITOR_VERSION\`, \`$EDITOR_PLUGIN\`, \`$EDITOR_AGENT\`"
      ;;
    editor-unversioned)
      echo "VS Code's without \`X-GitHub-Api-Version\`: \`Accept: application/json\`, \`$EDITOR_VERSION\`, \`$EDITOR_PLUGIN\`, \`$EDITOR_AGENT\`"
      ;;
  esac
}

describe_source() {
  case "$1" in
    keychain) echo "the Copilot CLI's, read from the Keychain item \`copilot-cli\` as QuotaDial reads it" ;;
    gh) echo "GitHub CLI's, from \`gh auth token\`" ;;
  esac
}

# Sends request NAME to the usage endpoint with the sign-in SOURCE and the header set HEADERS,
# keeping the answer as NAME.json and its headers as NAME.headers in the raw folder. Sets
# REQUEST_CODE to the HTTP status, "not sent" or "no answer", REQUEST_NOTE to why, and REQUEST_AT
# to when it went.
request() {
  local name="$1" source="$2" set="$3" statuses
  headers_for "$set"
  : > "$out/$name.json"
  : > "$out/$name.headers"
  : > "$out/$name.code"
  sign_in "$source" 2> "$out/$name.sign-in.txt" |
    as_curl_settings "$endpoint" |
    curl -sS --config - --proto "$proto" --max-redirs 0 --max-time 60 "${HEADER_ARGS[@]}" \
      -D "$out/$name.headers" -o "$out/$name.json" -w '%{http_code}' \
      > "$out/$name.code" 2> "$out/$name.curl.txt"
  statuses=("${PIPESTATUS[@]}")
  REQUEST_NOTE=""
  REQUEST_AT=""
  if [[ ${statuses[1]} == 1 ]]; then
    REQUEST_CODE="not sent"
    if [[ $source == keychain ]]; then
      REQUEST_NOTE="the sign-in could not be read: security exited with status ${statuses[0]}, $(read_meaning "${statuses[0]}")$(said "$out/$name.sign-in.txt")"
    else
      REQUEST_NOTE="the sign-in could not be read: gh auth token exited with status ${statuses[0]}$(said "$out/$name.sign-in.txt")"
    fi
    return 0
  fi
  if [[ ${statuses[1]} == 3 ]]; then
    REQUEST_CODE="not sent"
    REQUEST_NOTE="what the sign-in holds is not a plain token"
    return 0
  fi
  last_request_at=$(date +%s)
  REQUEST_AT=$(date -u +%Y-%m-%dT%H:%M:%SZ)
  if [[ ${statuses[2]} != 0 ]]; then
    REQUEST_CODE="no answer"
    REQUEST_NOTE="curl exited with status ${statuses[2]}$(said "$out/$name.curl.txt")"
  else
    REQUEST_CODE=$(cat "$out/$name.code")
  fi
}

# request, saying on screen what happened.
send() {
  say "Sending request $(upper "$1")."
  request "$@"
  case "$REQUEST_CODE" in
    "not sent") say "Not sent: $REQUEST_NOTE." ;;
    "no answer") say "No answer: $REQUEST_NOTE." ;;
    *) say "$answerer answered HTTP $REQUEST_CODE." ;;
  esac
}

# Writes what request NAME (SOURCE, HEADERS) gave to the results, with ASKED, whether macOS asked.
record_request() {
  local name="$1" source="$2" set="$3" asked="${4:-}"
  record "- Sign-in: $(describe_source "$source")" "- Headers: $(describe_headers "$set")"
  case "$REQUEST_CODE" in
    "not sent") record "- Not sent: $REQUEST_NOTE." ;;
    "no answer") record "- Sent at $REQUEST_AT, no answer: $REQUEST_NOTE." ;;
    *)
      record "- Sent at $REQUEST_AT; answer: HTTP $REQUEST_CODE, kept as \`$name.json\` and \`$name.headers\`" \
        "- Its headers: $(python3 "$helper" headers "$out/$name.headers")"
      if [[ $REQUEST_CODE != 200 ]]; then
        record "- $answerer said: $(python3 "$helper" message "$out/$name.json")"
      fi
      ;;
  esac
  if [[ -n $asked ]]; then record "- Did macOS ask about \`copilot-cli\`: $asked"; fi
}

refused() {
  case "$1" in
    401 | 403 | 404) return 0 ;;
    *) return 1 ;;
  esac
}

# Waits until 5 minutes after the last request sent to GitHub (seconds in the dry run), counting
# down on one line, so the usage endpoint is asked at most once every 5 minutes, as QuotaDial asks.
wait_for_floor() {
  local target now left shown=""
  [[ -n $last_request_at ]] || return 0
  target=$((last_request_at + gap))
  now=$(date +%s)
  ((now < target)) || return 0
  say "Waiting before $1, so GitHub is asked at most once every 5 minutes. It is not stuck."
  while now=$(date +%s) && ((now < target)); do
    left=$((target - now))
    if [[ -t 1 ]]; then
      printf '\r  %d:%02d left ' $((left / 60)) $((left % 60))
    elif [[ -z $shown ]] || ((left % 30 == 0)); then
      printf '  %d:%02d left\n' $((left / 60)) $((left % 60))
      shown=yes
    fi
    sleep 1
  done
  if [[ -t 1 ]]; then printf '\r  0:00 left\n'; fi
}

# --- The dry run's stand-ins --------------------------------------------------------------------

if dry; then
  answerer="The stand-in server"
  mkdir -p "$out/gh" "$out/data"
  printf '%s\n' 'github.com:' '    users:' '        dry-run-user:' '    git_protocol: https' \
    '    user: dry-run-user' '    oauth_token: dry_run_gh_token' > "$out/gh/hosts.yml"
  cat > "$out/stand-in-server.py" << 'PY'
"""GitHub's usage endpoint as the dry run sees it, on 127.0.0.1 only. The made-up GitHub CLI
sign-in is answered; the made-up Copilot CLI sign-in is refused unless VS Code's editor headers
come with it, so that request C runs; anything else is refused as a bad sign-in. Each request is
logged by its headers, the sign-in named, never written."""
import calendar
import http.server
import json
import sys
import time

PORT_FILE, LOG_FILE = sys.argv[1], sys.argv[2]
SIGN_INS = {
    "token dry_run_gh_token": "the made-up GitHub CLI sign-in",
    "token dry_run_keychain_token": "the made-up Copilot CLI sign-in",
}
NOW = time.gmtime()
NEXT_YEAR, NEXT_MONTH = (NOW.tm_year + 1, 1) if NOW.tm_mon == 12 else (NOW.tm_year, NOW.tm_mon + 1)
RESET_AT = calendar.timegm((NEXT_YEAR, NEXT_MONTH, 1, 0, 0, 0))
RESET_DATE = "%04d-%02d-01" % (NEXT_YEAR, NEXT_MONTH)
UNLIMITED = {
    "entitlement": 0,
    "overage_count": 0,
    "overage_permitted": False,
    "percent_remaining": 100,
    "remaining": 0,
    "unlimited": True,
}
ANSWER = {
    "login": "dry-run-user",
    "access_type_sku": "dry_run_sku",
    "analytics_tracking_id": "0123456789abcdef0123456789abcdef",
    "assigned_date": "2026-01-01T00:00:00Z",
    "can_signup_for_limited": False,
    "chat_enabled": True,
    "copilot_plan": "individual_pro",
    "token_based_billing": True,
    "notification_address": "dry-run-user@example.invalid",
    "organization_login_list": ["dry-run-org"],
    "organization_list": [{"id": 4242, "login": "dry-run-org", "name": "Dry Run Org"}],
    "endpoints": {"api": "https://api.individual.githubcopilot.com"},
    "quota_reset_date": RESET_DATE,
    "quota_reset_date_utc": RESET_DATE + "T00:00:00.000Z",
    "quota_snapshots": {
        "chat": dict(UNLIMITED, quota_id="chat"),
        "completions": dict(UNLIMITED, quota_id="completions"),
        "premium_interactions": {
            "credits_used": 375,
            "entitlement": 1500,
            "overage_count": 0,
            "overage_permitted": True,
            "percent_remaining": 75.0,
            "quota_id": "premium_interactions",
            "quota_remaining": 1125,
            "quota_reset_at": RESET_AT,
            "remaining": 1125,
            "unlimited": False,
        },
    },
}


class Handler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        sign_in = self.headers.get("Authorization", "")
        named = SIGN_INS.get(sign_in, "something else" if sign_in else "no sign-in")
        sent = [
            "%s: %s" % (name, self.headers[name])
            for name in ("Accept", "X-GitHub-Api-Version", "User-Agent", "Editor-Version",
                         "Editor-Plugin-Version")
            if self.headers.get(name)
        ]
        if self.path != "/copilot_internal/user":
            code, body = 404, {"message": "Not Found"}
        elif sign_in == "token dry_run_gh_token" or (
            sign_in == "token dry_run_keychain_token" and self.headers.get("Editor-Version")
        ):
            code, body = 200, ANSWER
        elif sign_in == "token dry_run_keychain_token":
            code, body = 404, {"message": "Not Found",
                               "documentation_url": "https://docs.github.com/rest"}
        else:
            code, body = 401, {"message": "Bad credentials",
                               "documentation_url": "https://docs.github.com/rest"}
        with open(LOG_FILE, "a", encoding="utf-8") as log:
            log.write("GET %s with %s; %s; answered %d\n" % (self.path, named, "; ".join(sent), code))
        data = json.dumps(body).encode()
        self.send_response(code)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(data)))
        self.send_header("X-RateLimit-Limit", "5000")
        self.send_header("X-RateLimit-Remaining", "4990")
        self.send_header("X-RateLimit-Used", "10")
        self.send_header("X-RateLimit-Reset", str(int(time.time()) + 3600))
        self.send_header("X-RateLimit-Resource", "core")
        if self.headers.get("X-GitHub-Api-Version"):
            self.send_header("X-GitHub-Api-Version-Selected", self.headers["X-GitHub-Api-Version"])
        self.end_headers()
        self.wfile.write(data)

    def log_message(self, *_):
        pass


SERVER = http.server.HTTPServer(("127.0.0.1", 0), Handler)
with open(PORT_FILE, "w", encoding="utf-8") as port:
    port.write(str(SERVER.server_address[1]))
SERVER.serve_forever()
PY
  cat > "$out/stand-in-quotadial.py" << 'PY'
"""QuotaDial's CLI as the dry run sees it in step 10: made-up output in the real one's shapes."""
import json
import sys

ACCOUNT = {
    "provider": "copilot",
    "provider_name": "Copilot",
    "account_id": "dry-run-user",
    "display_name": "dry-run-user",
    "plan": "individual_pro",
    "sign_in": "~/.config/gh/hosts.yml",
    "sign_in_use": "QuotaDial reads this account's sign-in from ~/.config/gh/hosts.yml. It only "
    "reads it and never changes it.",
}
WINDOW = {
    "kind": "credits",
    "label": "AI credits",
    "used_pct": 25.0,
    "resets_at": "2026-11-01T00:00:00Z",
    "window_seconds": 2592000,
    "source": "cache",
    "fetched_at": "2026-10-26T12:00:00Z",
    "unconfirmed": False,
}
ARGS = sys.argv[1:]
if ARGS == ["status"]:
    print("Copilot dry-run-user (individual_pro)\n  AI credits  25% used, resets in 6 days")
elif ARGS == ["status", "--json"]:
    REPORT = {
        "snapshots": [{"account": ACCOUNT, "windows": [WINDOW], "status": "ok"}],
        "errors": [],
        "providers": [{"id": "copilot", "name": "Copilot"}],
    }
    print(json.dumps(REPORT, indent=2))
elif ARGS == ["doctor"]:
    print("QuotaDial 0.0.0, a stand-in for the dry run\nAccounts (1):\n"
          "  Copilot dry-run-user: ok\n    sign-in: " + ACCOUNT["sign_in_use"] + "\n"
          "    AI credits: 25% used, from cache")
else:
    sys.exit(2)
PY
  : > "$out/stand-in.log"
  : > "$out/stand-in.port"
  python3 "$out/stand-in-server.py" "$out/stand-in.port" "$out/stand-in.log" 2> "$out/stand-in.err" &
  server=$!
  trap 'kill "$server" 2> /dev/null' EXIT
  for _ in $(seq 50); do
    [[ -s $out/stand-in.port ]] && break
    sleep 0.1
  done
  [[ -s $out/stand-in.port ]] || fail "The dry run's stand-in server did not start."
  endpoint="http://127.0.0.1:$(cat "$out/stand-in.port")/copilot_internal/user"
  proto="=http"
  gh_dir="$out/gh"
  quotadial=(python3 "$out/stand-in-quotadial.py")
else
  answerer="GitHub"
  endpoint=$REAL_ENDPOINT
  proto="=https"
  if [[ -n ${GH_CONFIG_DIR:-} ]]; then
    gh_dir=$GH_CONFIG_DIR
  elif [[ -n ${XDG_CONFIG_HOME:-} ]]; then
    gh_dir=$XDG_CONFIG_HOME/gh
  else
    gh_dir=$HOME/.config/gh
  fi
fi
trap 'printf "\n"; fail "Stopped with Control-C. Run it again to start over."' INT

# --- The steps ----------------------------------------------------------------------------------

say "" "QuotaDial's Copilot live check (#51)."
if dry; then
  say "Dry run: a stand-in server on this Mac answers for GitHub, with made-up sign-ins and waits of" \
    "$gap seconds. Nothing reads the Keychain, GitHub CLI or the Copilot CLI, nothing runs cargo," \
    "and nothing is sent to GitHub."
else
  say "It takes about 25 minutes, most of it waiting: GitHub is asked at most once every 5 minutes," \
    "as QuotaDial asks it. No token is shown or saved. Keep this window open: it says whenever it" \
    "needs you, and macOS may ask about the Keychain along the way."
fi
say "The results go in $(tilde "$results")."
pause "Press Enter to start."
run_note=""
if dry; then run_note=", a dry run against a stand-in server with made-up sign-ins"; fi
record "# Copilot live check (#51)" "" \
  "- Run at $(date -u +%Y-%m-%dT%H:%M:%SZ) on $(git -C "$repo" rev-parse --abbrev-ref HEAD) at $(git -C "$repo" rev-parse --short HEAD)$run_note" \
  "- Requests go at least $gap seconds apart, to $endpoint"

step "Step 1: the versions of macOS, GitHub CLI and the Copilot CLI, asking no one."
macos=$(sw_vers -productVersion 2> /dev/null) || macos=unknown
if dry; then
  gh_version="not run in the dry run"
  copilot_version="not run in the dry run"
else
  gh_version="not installed"
  if [[ $gh_installed == yes ]]; then
    gh_version=$(with_time_limit 30 run_gh --version 2>&1 < /dev/null | head -n 1)
  fi
  copilot_version="not installed"
  if [[ $copilot_installed == yes ]]; then
    copilot_version=$(with_time_limit 30 run_copilot --version 2>&1 < /dev/null | head -n 1)
  fi
fi
say "macOS $macos; GitHub CLI: $gh_version; Copilot CLI: $copilot_version."
record "" "## 1. Versions" "" "- macOS: $macos" "- GitHub CLI: $gh_version" \
  "- Copilot CLI: $copilot_version" \
  "- QuotaDial's requests: \`User-Agent: quotadial/$version\`, \`X-GitHub-Api-Version: $api_version\`"

step "Step 2: finding the Copilot CLI's sign-in in the Keychain by its attributes, reading no secret."
record "" "## 2. The Copilot CLI's Keychain item, by its attributes" ""
acct=""
if [[ $keychain_item == yes ]]; then
  if dry; then
    printf '%s\n' 'class: "genp"' 'attributes:' '    "acct"<blob>="https://github.com:dry-run-user"' \
      '    "mdat"<timedate>=0x32303236313032363132303030305A00  "20261026120000Z\000"' \
      '    "svce"<blob>="copilot-cli"' > "$out/attributes.txt"
  else
    run_security find-generic-password -s "$SERVICE" > "$out/attributes.txt" 2> "$out/attributes.err"
  fi
  svce="" mdat="" item_login="" form=""
  while IFS=$'\t' read -r key value; do
    case "$key" in
      acct) acct=$value ;;
      svce) svce=$value ;;
      mdat) mdat=$value ;;
      login) item_login=$value ;;
      form) form=$value ;;
    esac
  done < <(python3 "$helper" attributes "$out/attributes.txt")
  forget "$item_login" "<login>"
  say "Found it: its account attribute is ${acct:-empty}."
  ask "Did macOS ask anything just then?"
  record "- Found: yes, service \`$svce\`" "- \`acct\`: \`${acct:-<NULL>}\`: $form" \
    "- \`mdat\`: $mdat" "- Did macOS ask: $ANSWER"
else
  say "There is no Copilot CLI sign-in in the Keychain, so every step that reads it is skipped."
  record "- Found: no (security exited with status 44: there is no such item)."
fi

step "Step 3: reading the Copilot CLI's sign-in three times, sending it nowhere, to see what macOS asks."
record "" "## 3. Whether reading the item makes macOS ask" ""
if [[ $keychain_item == yes ]]; then
  say "macOS may show a window saying that security wants to use your confidential information" \
    "stored in \"copilot-cli\" in your keychain, with a box for your Mac password and three" \
    "buttons: Always Allow, Deny and Allow." \
    "  - The first time it asks, click Deny." \
    "  - Every time after that, type your Mac password and click Allow." \
    "  - Never click Always Allow: it would let any program on this Mac read the sign-in."
  pause "Press Enter when you are ready."
  asked_reads="" read_statuses=""
  for n in 1 2 3; do
    case $n in
      1) say "Read 1 of 3. If macOS asks, click Deny." ;;
      2) say "Read 2 of 3. If macOS asks, type your Mac password and click Allow." ;;
      3) say "Read 3 of 3, to see whether macOS asks again. If it does, type your Mac password and click Allow." ;;
    esac
    if dry; then
      : > "$out/read-$n.err"
      status=0
      if [[ $n == 1 ]]; then status=128; fi
    else
      read_item > /dev/null 2> "$out/read-$n.err"
      status=$?
    fi
    say "security exited with status $status: $(read_meaning "$status")."
    ask "Did macOS ask?"
    asked_reads="$asked_reads$ANSWER "
    read_statuses="$read_statuses$status "
    record "- Read $n: exit $status, $(read_meaning "$status"); macOS asked: $ANSWER$(said "$out/read-$n.err")"
  done
  read -r asked_1 asked_2 asked_3 <<< "$asked_reads"
  read -r status_1 status_2 _ <<< "$read_statuses"
  if [[ $asked_1 == yes && $status_1 != 0 ]]; then record "- A Deny gives exit $status_1."; fi
  # What an Allow lasts, told only when read 2 was allowed after macOS asked.
  if [[ $asked_1 == no && $asked_2 == no && $asked_3 == no ]]; then
    record "- macOS never asked: the item lets \`security\` read it without asking."
  elif [[ $status_2 == 0 && $asked_2 == yes && $asked_3 == yes ]]; then
    record "- macOS asked again after an Allow: an Allow lasts one read, so QuotaDial would ask every 5 minutes."
  elif [[ $status_2 == 0 && $asked_2 == yes && $asked_3 == no ]]; then
    record "- macOS did not ask again after an Allow."
  else
    record "- These three reads say nothing of what an Allow lasts: see each one above."
  fi
else
  say "Skipped: there is no Copilot CLI sign-in in the Keychain."
  record "- Skipped: there is no Copilot CLI sign-in in the Keychain."
fi

step "Step 4: where GitHub CLI keeps its sign-in, with its token left out."
record "" "## 4. Where GitHub CLI keeps its sign-in" ""
if [[ $gh_installed == yes ]]; then
  while IFS=$'\t' read -r key value; do
    if [[ $key == login ]]; then forget "$value" "<login>"; fi
  done < <(
    if dry; then
      printf '%s\n' 'github.com' "  Logged in to github.com account dry-run-user ($(tilde "$gh_dir/hosts.yml"))" \
        '  - Active account: true' '  - Git operations protocol: https' \
        '  - Token: (hidden, as gh shows it, and left out here)' "  - Token scopes: 'gist', 'read:org', 'repo'"
    else
      run_gh auth status --hostname github.com 2>&1 < /dev/null
    fi | python3 "$helper" gh-status - "$out/gh-status.txt"
  )
  say "gh auth status, with its token line left out:"
  sed 's/^/  /' "$out/gh-status.txt"
  record "- \`gh auth status\`, with its token line left out:" "" '```'
  record < "$out/gh-status.txt"
  record '```'
  hosts="$gh_dir/hosts.yml"
  if [[ -f $hosts ]]; then
    findings=""
    while IFS=$'\t' read -r key value; do
      case "$key" in
        login) forget "$value" "<login>" ;;
        finding) findings="$findings- $value"$'\n' ;;
      esac
    done < <(python3 "$helper" hosts-layout "$hosts" "$out/hosts-layout.txt")
    say "GitHub CLI's file, $(tilde "$hosts"), with its token left out:"
    sed 's/^/  /' "$out/hosts-layout.txt"
    record "" "- \`$(tilde "$hosts")\`, with its token left out:" "" '```'
    record < "$out/hosts-layout.txt"
    record '```' ""
    printf '%s' "$findings" | record
  else
    say "There is no $(tilde "$hosts"): GitHub CLI keeps its sign-in elsewhere, such as the keychain."
    record "" "- There is no \`$(tilde "$hosts")\`: GitHub CLI keeps its sign-in elsewhere."
  fi
else
  say "Skipped: GitHub CLI is not installed."
  record "- Skipped: GitHub CLI is not installed, so request B is skipped too."
fi

step "Step 5: request A, the Copilot CLI's sign-in with QuotaDial's headers."
record "" "## 5. Request A: the Copilot CLI's sign-in, QuotaDial's headers" ""
a_code="not sent"
if [[ $keychain_item == yes ]]; then
  say "If macOS asks about \"copilot-cli\", type your Mac password and click Allow."
  send a keychain quotadial
  a_code=$REQUEST_CODE
  ask "Did macOS ask about \"copilot-cli\" just then?"
  record_request a keychain quotadial "$ANSWER"
else
  say "Skipped: there is no Copilot CLI sign-in in the Keychain."
  record "- Not sent: there is no Copilot CLI sign-in in the Keychain."
fi

step "Step 6: request B, GitHub CLI's sign-in with QuotaDial's headers."
record "" "## 6. Request B: GitHub CLI's sign-in, QuotaDial's headers" ""
b_code="not sent"
if [[ $gh_signed_in == yes ]]; then
  wait_for_floor "request B"
  send b gh quotadial
  b_code=$REQUEST_CODE
  record_request b gh quotadial
else
  say "Skipped: GitHub CLI is not signed in."
  record "- Not sent: GitHub CLI is not signed in."
fi

step "Step 7: request C, only if A or B was refused."
record "" "## 7. Request C: a refused sign-in again, with VS Code's editor headers" ""
c_code="not sent"
c_source=""
if refused "$a_code"; then
  c_source=keychain
elif refused "$b_code"; then
  c_source=gh
fi
if [[ -n $c_source ]]; then
  say "A or B was refused, so C sends that sign-in again with VS Code's headers in place of QuotaDial's User-Agent."
  wait_for_floor "request C"
  asked=""
  if [[ $c_source == keychain ]]; then
    say "If macOS asks about \"copilot-cli\", type your Mac password and click Allow."
  fi
  send c "$c_source" editor
  c_code=$REQUEST_CODE
  if [[ $c_source == keychain ]]; then
    ask "Did macOS ask about \"copilot-cli\" just then?"
    asked=$ANSWER
  fi
  record_request c "$c_source" editor "$asked"
else
  say "Skipped: neither A nor B was refused."
  record "- Not sent: neither A nor B was refused (A: $a_code; B: $b_code)."
fi

step "Step 8: request D, a sign-in that worked, without X-GitHub-Api-Version."
record "" "## 8. Request D: a sign-in that worked, without \`X-GitHub-Api-Version\`" ""
worked=""
d_code="not sent"
if [[ $a_code == 200 ]]; then
  worked=a d_source=keychain d_set=quotadial-unversioned
elif [[ $b_code == 200 ]]; then
  worked=b d_source=gh d_set=quotadial-unversioned
elif [[ $c_code == 200 ]]; then
  worked=c d_source=$c_source d_set=editor-unversioned
fi
if [[ -n $worked ]]; then
  say "Request $(upper "$worked") worked, so D sends its sign-in and headers again without X-GitHub-Api-Version."
  wait_for_floor "request D"
  asked=""
  if [[ $d_source == keychain ]]; then
    say "If macOS asks about \"copilot-cli\", type your Mac password and click Allow."
  fi
  send d "$d_source" "$d_set"
  d_code=$REQUEST_CODE
  if [[ $d_source == keychain ]]; then
    ask "Did macOS ask about \"copilot-cli\" just then?"
    asked=$ANSWER
  fi
  record_request d "$d_source" "$d_set" "$asked"
  if [[ $d_code == 200 ]]; then
    record "- Its fields beside request $(upper "$worked")'s: $(python3 "$helper" compare "$out/$worked.json" "$out/d.json")"
  fi
else
  say "Skipped: no request was answered with HTTP 200."
  record "- Not sent: no request was answered with HTTP 200."
fi

step "Step 9: the answer's shape, saved as a redacted fixture."
record "" "## 9. The answer's shape, as a redacted fixture" ""
answer=""
if [[ $a_code == 200 ]]; then
  answer=a
elif [[ $b_code == 200 ]]; then
  answer=b
elif [[ $c_code == 200 ]]; then
  answer=c
elif [[ $d_code == 200 ]]; then
  answer=d
fi
if [[ -n $answer ]]; then
  candidate="$out/user-live.candidate.json"
  : > "$candidate"
  while IFS=$'\t' read -r key value; do
    case "$key" in
      login) forget "$value" "<login>" ;;
      org) forget "$value" "<organisation>" ;;
      orgname) forget "$value" "<organisation's name>" ;;
      id) forget "$value" "<id>" ;;
    esac
  done < <(python3 "$helper" fixture "$out/$answer.json" "$candidate")
  [[ -s $candidate ]] ||
    fail "Request $(upper "$answer")'s answer is not JSON, so no fixture was written: tell QuotaDial's Chat."
  if grep -n -i -E 'gho_|ghu_|ghp_|github_pat_|@' "$candidate" > /dev/null; then
    fail "The redacted answer still holds something shaped like a token or an email address, so it was not saved as a fixture: tell QuotaDial's Chat."
  fi
  problem=$(python3 "$helper" check-fixture "$candidate") ||
    fail "The redacted answer still holds $problem, so it was not saved as a fixture: tell QuotaDial's Chat."
  cp "$candidate" "$fixture" || fail "The fixture could not be written to $(tilde "$fixture")."
  say "Saved $(tilde "$fixture") from request $(upper "$answer")'s answer, with the login, organisations and ids made up."
  record "- Written from request $(upper "$answer")'s answer to \`${fixture#"$repo"/}\`; searching it for \`gho_|ghu_|ghp_|github_pat_|@\` found nothing."
  python3 "$helper" summary "$fixture" | record
  ask_text "If you can, open the Copilot CLI in another window, type /usage and type here what it shows for this month (Enter alone skips):"
  record "- What the Copilot CLI's /usage shows for the month: ${ANSWER:-not given}"
  if dry; then
    record "- \`cargo test -p quotadial-core --test fixtures\`: not run in the dry run."
  else
    say "Checking every fixture, the new one too: cargo test -p quotadial-core --test fixtures"
    if run_cargo test --locked -p quotadial-core --test fixtures; then
      record "- \`cargo test -p quotadial-core --test fixtures\`: passed."
    else
      record "- \`cargo test -p quotadial-core --test fixtures\`: failed; the fixture needs another look."
    fi
  fi
else
  say "No request was answered with HTTP 200, so there is no answer to save."
  record "- No request was answered with HTTP 200, so there is no answer to save."
fi

step "Step 10: QuotaDial itself, built with Copilot, in a data folder of its own."
record "" "## 10. QuotaDial itself, 5 minutes after the last request" ""
if dry; then
  data_dir="$out/data"
  record "- Not built in the dry run: a stand-in for QuotaDial's CLI ran instead."
else
  say "Building QuotaDial with Copilot: cargo build --locked -p quotadial --features copilot"
  if ! run_cargo build --locked -p quotadial --features copilot; then
    record "- \`cargo build --locked -p quotadial --features copilot\` failed."
    fail "QuotaDial did not build, so step 10 could not run: tell QuotaDial's Chat."
  fi
  target=$(run_cargo metadata --locked --format-version 1 --no-deps 2> /dev/null |
    python3 -c 'import json, sys; print(json.load(sys.stdin)["target_directory"])') || target="$repo/target"
  quotadial=("$target/debug/quotadial")
  [[ -x ${quotadial[0]} ]] || fail "The built QuotaDial is not at ${quotadial[0]}: tell QuotaDial's Chat."
  data_dir=$(mktemp -d "${TMPDIR:-/tmp}/quotadial-live-check.XXXXXX") ||
    fail "No data folder could be made for QuotaDial."
  record "- Built with \`cargo build --locked -p quotadial --features copilot\`."
fi
record "- Its data folder: \`$(tilde "$data_dir")\`, new and its own"
wait_for_floor "QuotaDial's own run"
say "If macOS asks about \"copilot-cli\", type your Mac password and click Allow. If it asks about" \
  "another item, answer as you would for QuotaDial." "Running: quotadial status"
QUOTADIAL_DATA_DIR="$data_dir" "${quotadial[@]}" status 2>&1 < /dev/null | tee "$out/status.txt"
status_exit=${PIPESTATUS[0]}
ask "Did macOS ask about \"copilot-cli\" during that?"
first_asked=$ANSWER
say "Running: quotadial status --json, then quotadial doctor, which show what status just fetched."
QUOTADIAL_DATA_DIR="$data_dir" "${quotadial[@]}" status --json > "$out/status.json" 2> "$out/status-json.err" < /dev/null
json_exit=$?
QUOTADIAL_DATA_DIR="$data_dir" "${quotadial[@]}" doctor 2>&1 < /dev/null | tee "$out/doctor.txt"
doctor_exit=${PIPESTATUS[0]}
ask "Did macOS ask about \"copilot-cli\" during those two?"
record "- \`status\`: exit $status_exit, kept as \`status.txt\`; macOS asked: $first_asked" \
  "- \`status --json\`: exit $json_exit, kept as \`status.json\`; \`doctor\`: exit $doctor_exit, kept as \`doctor.txt\`; macOS asked during these two: $ANSWER" \
  "- Copilot in \`status --json\`:"
python3 "$helper" status-summary "$out/status.json" | record
record "- Copilot in \`doctor\`:" ""
python3 "$helper" doctor-copilot "$out/doctor.txt" | record

step "Step 11: the results."
record "" "## 11. Next" "" \
  "- QuotaDial's Chat reads these results, records them as a decision and finishes #81."
if dry; then
  record "" "## The stand-in server's log (dry run only)" "" '```'
  record < "$out/stand-in.log"
  record '```'
fi
python3 "$helper" redact-file "$results"
if grep -q -E 'gh[opusr]_[A-Za-z0-9_]{8,}|github_pat_[A-Za-z0-9_]{8,}' "$results"; then
  : > "$results"
  fail "The results held something shaped like a token, so they were emptied: tell QuotaDial's Chat."
fi
say "" "Results: $results" "Done. Tell QuotaDial's Chat it has finished."
