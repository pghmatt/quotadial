#!/usr/bin/env bash
# Writes crates/cost/prices.json, the price table the cost scan bundles (D-090, D-097), from
# models.dev. Run it when a release is made, so the table stays current without QuotaDial ever
# fetching prices itself. It keeps the Anthropic and OpenAI models' prices per million tokens in US
# dollars: input, output, cache reads and writes, and a higher price above a context size, where
# models.dev lists one.
#
# Usage:
#   scripts/update-prices.sh               fetch models.dev and rewrite the table
#   scripts/update-prices.sh <api.json>    rewrite the table from a copy of models.dev's api.json
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
table="$repo_root/crates/cost/prices.json"

if [[ $# -ge 1 ]]; then
  source_json="$(cat "$1")"
else
  source_json="$(curl --fail --silent --show-error --max-time 60 https://models.dev/api.json)"
fi

jq --sort-keys --arg day "$(date -u +%Y-%m-%d)" '
  # The price above a context size: the smallest context tier, or the older context_over_200k.
  def above:
    ([(.tiers // [])[] | select(.tier.type == "context")] | min_by(.tier.size)) as $tier
    | if $tier then $tier | {tokens: .tier.size, input, output, cache_read, cache_write}
      elif .context_over_200k then
        .context_over_200k | {tokens: 200000, input, output, cache_read, cache_write}
      else null end;
  def price:
    {input, output, cache_read, cache_write}
    + (above as $above | if $above then {above: $above} else {} end)
    | with_entries(select(.value != null))
    | if .above then .above |= with_entries(select(.value != null)) else . end;
  {
    source: "https://models.dev/api.json",
    updated: $day,
    anthropic: (.anthropic.models | with_entries(select(.value.cost.input != null) | .value = (.value.cost | price))),
    openai: (.openai.models | with_entries(select(.value.cost.input != null) | .value = (.value.cost | price)))
  }
' <<<"$source_json" >"$table.tmp"
mv "$table.tmp" "$table"
echo "Wrote $(jq '.anthropic | length' "$table") Anthropic and $(jq '.openai | length' "$table") OpenAI prices to ${table#"$repo_root/"}."
