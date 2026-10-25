#!/usr/bin/env bash
# Sourced by the Release workflow's app build, before `tauri build` (docs/RELEASE.md). It turns on
# each kind of signing whose secrets are set and leaves the rest unsigned, so a release builds with
# all of them, some or none. Each secret and variable arrives as QD_<name> and is exported under its
# own name only when it has a value, because Tauri takes a variable that is set but empty as a
# setting. It leaves the Tauri config to pass with --config in SIGNING_CONFIG, and says what it
# signs.
#
# Usage, in the step that builds:
#   source scripts/release-signing.sh && npx tauri build --config "$SIGNING_CONFIG"

signing_parts=()

# Exports NAME from QD_NAME, when that has a value.
signing_export() {
  local from="QD_$1"
  if [[ -n "${!from:-}" ]]; then
    export "$1=${!from}"
  fi
}

# Whether every QD_<name> named has a value.
signing_has() {
  local name from
  for name in "$@"; do
    from="QD_$name"
    [[ -n "${!from:-}" ]] || return 1
  done
}

if signing_has TAURI_SIGNING_PRIVATE_KEY; then
  signing_export TAURI_SIGNING_PRIVATE_KEY
  signing_export TAURI_SIGNING_PRIVATE_KEY_PASSWORD
  signing_parts+=('"createUpdaterArtifacts":true')
  echo "Updates: signed with the updater key, for latest.json."
else
  echo "Updates: TAURI_SIGNING_PRIVATE_KEY is not set, so this release offers no update."
fi

case "${RUNNER_OS:-}" in
  macOS)
    if signing_has APPLE_CERTIFICATE APPLE_CERTIFICATE_PASSWORD APPLE_SIGNING_IDENTITY; then
      signing_export APPLE_CERTIFICATE
      signing_export APPLE_CERTIFICATE_PASSWORD
      signing_export APPLE_SIGNING_IDENTITY
      # Notarization needs the hardened runtime, which an ad-hoc signature does without.
      signing_parts+=('"macOS":{"hardenedRuntime":true}')
      if signing_has APPLE_ID APPLE_PASSWORD APPLE_TEAM_ID; then
        signing_export APPLE_ID
        signing_export APPLE_PASSWORD
        signing_export APPLE_TEAM_ID
        echo "macOS: signed with the Developer ID certificate and notarized."
      else
        echo "macOS: signed with the Developer ID certificate, but not notarized:" \
          "APPLE_ID, APPLE_PASSWORD and APPLE_TEAM_ID are not all set."
      fi
    else
      echo "macOS: not signed: APPLE_CERTIFICATE, APPLE_CERTIFICATE_PASSWORD and" \
        "APPLE_SIGNING_IDENTITY are not all set."
    fi
    ;;
  Windows)
    if signing_has AZURE_TENANT_ID AZURE_CLIENT_ID AZURE_CLIENT_SECRET \
      AZURE_SIGNING_ENDPOINT AZURE_SIGNING_ACCOUNT AZURE_SIGNING_PROFILE; then
      # They go into the config as JSON strings, and none ever holds a quote or a backslash.
      for name in AZURE_SIGNING_ENDPOINT AZURE_SIGNING_ACCOUNT AZURE_SIGNING_PROFILE; do
        from="QD_$name"
        if [[ "${!from}" =~ [\"\\[:cntrl:]] ]]; then
          echo "$name holds a quote, a backslash or a control character." >&2
          return 1
        fi
      done
      signing_export AZURE_TENANT_ID
      signing_export AZURE_CLIENT_ID
      signing_export AZURE_CLIENT_SECRET
      signing_parts+=("$(printf '"windows":{"signCommand":{"cmd":"artifact-signing-cli","args":["-e","%s","-a","%s","-c","%s","-d","QuotaDial","%%1"]}}' \
        "$QD_AZURE_SIGNING_ENDPOINT" "$QD_AZURE_SIGNING_ACCOUNT" "$QD_AZURE_SIGNING_PROFILE")")
      echo "Windows: signed with Azure Artifact Signing."
    else
      echo "Windows: not signed: AZURE_TENANT_ID, AZURE_CLIENT_ID, AZURE_CLIENT_SECRET and the" \
        "AZURE_SIGNING_ENDPOINT, AZURE_SIGNING_ACCOUNT and AZURE_SIGNING_PROFILE variables are" \
        "not all set."
    fi
    ;;
esac

# The quotes are the JSON's own, meant to reach Tauri as they are.
# shellcheck disable=SC2089
printf -v SIGNING_CONFIG '{"bundle":{%s}}' "$(
  IFS=,
  echo "${signing_parts[*]}"
)"
# shellcheck disable=SC2090
export SIGNING_CONFIG
