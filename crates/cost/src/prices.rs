//! The price table: each model's price per million tokens, in US dollars, bundled with each release
//! from models.dev by `scripts/update-prices.sh` (D-090). QuotaDial never fetches prices itself.
//!
//! Prices are the API's standard ones: no batch, priority or regional rates. Following CodexBar's
//! `CostUsagePricing.swift`, a request whose context is over a model's `above.tokens` costs the
//! higher price for all of it, a cache price the table lacks is the input price, and Claude's
//! hour-long cache writes cost twice the input price, as Anthropic charges them.

use std::collections::HashMap;

use quotadial_core::Tokens;
use serde::Deserialize;

/// The table the release was built with.
const BUNDLED: &str = include_str!("../prices.json");

/// A model's price per million tokens, in US dollars.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Price {
    pub input: f64,
    pub output: f64,
    /// Reading from the prompt cache; input's price when the table has none.
    #[serde(default)]
    pub cache_read: Option<f64>,
    /// Writing to the prompt cache for five minutes; input's price when the table has none.
    #[serde(default)]
    pub cache_write: Option<f64>,
    /// A higher price for requests whose context is larger than its `tokens`.
    #[serde(default)]
    pub above: Option<Above>,
}

/// The price above a context size.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Above {
    /// The context size, in tokens, above which this price applies.
    pub tokens: u64,
    pub input: f64,
    pub output: f64,
    #[serde(default)]
    pub cache_read: Option<f64>,
    #[serde(default)]
    pub cache_write: Option<f64>,
}

impl Price {
    /// What one request of `tokens` costs, in US dollars, at the higher price when its context is
    /// over that price's size.
    #[must_use]
    pub fn of(&self, tokens: Tokens) -> f64 {
        let (input, output, cache_read, cache_write) = match &self.above {
            Some(above) if tokens.context() > above.tokens => (
                above.input,
                above.output,
                above.cache_read.unwrap_or(above.input),
                above.cache_write.unwrap_or(above.input),
            ),
            _ => (
                self.input,
                self.output,
                self.cache_read.unwrap_or(self.input),
                self.cache_write.unwrap_or(self.input),
            ),
        };
        let hour_long = tokens.cache_write_1h.min(tokens.cache_write);
        let per = |count: u64, price: f64| count as f64 * price / 1_000_000.0;
        per(tokens.input, input)
            + per(tokens.output, output)
            + per(tokens.cache_read, cache_read)
            + per(tokens.cache_write - hour_long, cache_write)
            + per(hour_long, input * 2.0)
    }
}

/// Each provider's models' prices.
#[derive(Debug, Clone, PartialEq)]
pub struct Prices {
    /// The day the table was made.
    pub updated: String,
    models: HashMap<(String, String), Price>,
}

#[derive(Deserialize)]
struct Table {
    updated: String,
    anthropic: HashMap<String, Price>,
    openai: HashMap<String, Price>,
}

impl Prices {
    /// The table bundled with this release.
    ///
    /// # Panics
    ///
    /// Never for a release build: the bundled table is checked by the tests.
    #[must_use]
    pub fn bundled() -> Self {
        Self::parse(BUNDLED).expect("the bundled price table is valid")
    }

    /// Reads a table in the shape `scripts/update-prices.sh` writes.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let table: Table = serde_json::from_str(text).ok()?;
        let models = table
            .anthropic
            .into_iter()
            .map(|(model, price)| (("claude".to_owned(), model), price))
            .chain(
                table
                    .openai
                    .into_iter()
                    .map(|(model, price)| (("codex".to_owned(), model), price)),
            )
            .collect();
        Some(Self {
            updated: table.updated,
            models,
        })
    }

    /// `provider`'s price for `model`, as the logs name it: exactly, or as the API names it under
    /// a cloud's spelling or with a date at its end, such as `claude-haiku-4-5-20251001`,
    /// `us.anthropic.claude-sonnet-4-5-20250929-v1:0` (Bedrock), `claude-opus-4-1@20250805`
    /// (Vertex) or `openai/gpt-5.5-2026-04-23`.
    #[must_use]
    pub fn of(&self, provider: &str, model: &str) -> Option<&Price> {
        let find = |model: &str| self.models.get(&(provider.to_owned(), model.to_owned()));
        let model = model.trim();
        if let Some(price) = find(model) {
            return Some(price);
        }
        let base = match provider {
            "claude" => claude_base(model),
            "codex" => codex_base(model),
            _ => return None,
        };
        find(base).or_else(|| find(without_date(base)))
    }
}

/// A Claude model without a cloud's spelling: Bedrock's region and `anthropic.` prefix and
/// `-v1:0` suffix, and Vertex's `@` date.
fn claude_base(model: &str) -> &str {
    let model = model.rsplit_once('.').map_or(model, |(_, tail)| {
        if tail.starts_with("claude-") {
            tail
        } else {
            model
        }
    });
    let model = model.split_once('@').map_or(model, |(head, _)| head);
    match model.rsplit_once("-v") {
        Some((head, version))
            if version
                .split_once(':')
                .is_some_and(|(major, minor)| is_digits(major) && is_digits(minor)) =>
        {
            head
        }
        _ => model,
    }
}

/// An OpenAI model without the `openai/` prefix some tools add.
fn codex_base(model: &str) -> &str {
    model.strip_prefix("openai/").unwrap_or(model)
}

/// A model without a date at its end: Anthropic's `-20251001` or OpenAI's `-2026-04-23`.
fn without_date(model: &str) -> &str {
    if let Some((head, date)) = model.rsplit_once('-')
        && date.len() == 8
        && is_digits(date)
    {
        return head;
    }
    if let Some(cut) = model.len().checked_sub(11).filter(|cut| *cut > 0)
        && let Some(date) = model.get(cut..)
        && let [b'-', year @ .., b'-', m1, m2, b'-', d1, d2] = date.as_bytes()
        && year.len() == 4
        && year.iter().chain([m1, m2, d1, d2]).all(u8::is_ascii_digit)
    {
        return &model[..cut];
    }
    model
}

fn is_digits(text: &str) -> bool {
    !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens(input: u64, output: u64, cache_read: u64, cache_write: u64) -> Tokens {
        Tokens {
            input,
            output,
            cache_read,
            cache_write,
            cache_write_1h: 0,
        }
    }

    fn near(actual: f64, expected: f64) -> bool {
        (actual - expected).abs() < 1e-9
    }

    #[test]
    fn the_bundled_table_prices_the_models_in_use_today() {
        let prices = Prices::bundled();

        for (provider, model) in [
            ("claude", "claude-opus-5-5"),
            ("claude", "claude-sonnet-5-5"),
            ("claude", "claude-opus-5"),
            ("claude", "claude-haiku-4-5-20251001"),
            ("codex", "gpt-6-astra"),
            ("codex", "gpt-6.1-sol"),
            ("codex", "gpt-5.6-terra"),
            ("codex", "gpt-5.5"),
        ] {
            assert!(prices.of(provider, model).is_some(), "{model}");
        }
        assert_eq!(
            prices.of("claude", "gpt-6-astra"),
            None,
            "each provider's own"
        );
        assert_eq!(prices.of("claude", "<synthetic>"), None);
        assert_eq!(prices.of("gemini", "claude-opus-5"), None);
    }

    #[test]
    fn a_cloud_spelling_or_a_date_finds_the_api_price() {
        let prices = Prices::bundled();
        let sonnet = prices.of("claude", "claude-sonnet-5-5");
        let terra = prices.of("codex", "gpt-5.6-terra");
        assert!(sonnet.is_some() && terra.is_some());

        for model in [
            "claude-sonnet-5-5-20261001",
            "anthropic.claude-sonnet-5-5-v1:0",
            "us.anthropic.claude-sonnet-5-5-20261001-v1:0",
            "claude-sonnet-5-5@20261001",
            " claude-sonnet-5-5 ",
        ] {
            assert_eq!(prices.of("claude", model), sonnet, "{model}");
        }
        for model in ["openai/gpt-5.6-terra", "gpt-5.6-terra-2026-09-30"] {
            assert_eq!(prices.of("codex", model), terra, "{model}");
        }
        assert_eq!(prices.of("claude", "claude-sonnet-5-5-preview"), None);
        assert_eq!(prices.of("claude", "claude-sonnet-5-5-v1"), None);
    }

    #[test]
    fn a_request_costs_each_kind_of_token_at_its_price() {
        let price = Price {
            input: 4.0,
            output: 20.0,
            cache_read: Some(0.2),
            cache_write: Some(5.0),
            above: None,
        };

        // 4 + 10 + 0.4 + 0.5
        assert!(near(
            price.of(tokens(1_000_000, 500_000, 2_000_000, 100_000)),
            14.9
        ));
        let no_cache_prices = Price {
            cache_read: None,
            cache_write: None,
            ..price
        };
        assert!(
            near(no_cache_prices.of(tokens(0, 0, 1_000_000, 1_000_000)), 8.0),
            "the input price when the table has none"
        );
    }

    #[test]
    fn an_hour_long_cache_write_costs_twice_the_input_price() {
        let price = Price {
            input: 4.0,
            output: 20.0,
            cache_read: Some(0.2),
            cache_write: Some(5.0),
            above: None,
        };
        let written = Tokens {
            cache_write: 3_000_000,
            cache_write_1h: 2_000_000,
            ..Tokens::default()
        };

        // 1M for five minutes at 5, and 2M for an hour at 8.
        assert!(near(price.of(written), 5.0 + 16.0));
        let more_than_written = Tokens {
            cache_write_1h: 9_000_000,
            ..written
        };
        assert!(
            near(price.of(more_than_written), 24.0),
            "never more hour-long tokens than were written"
        );
    }

    #[test]
    fn a_large_context_costs_the_higher_price_for_all_of_it() {
        let price = Price {
            input: 10.0,
            output: 50.0,
            cache_read: Some(1.0),
            cache_write: Some(12.5),
            above: Some(Above {
                tokens: 272_000,
                input: 20.0,
                output: 75.0,
                cache_read: Some(2.0),
                cache_write: Some(25.0),
            }),
        };

        assert!(
            near(price.of(tokens(100_000, 10_000, 0, 0)), 1.5),
            "1 + 0.5"
        );
        assert!(
            near(price.of(tokens(100_000, 10_000, 172_000, 0)), 1.672),
            "272k is not over 272k"
        );
        assert!(
            near(price.of(tokens(100_000, 10_000, 200_000, 0)), 3.15),
            "2 + 0.75 + 0.4 over 272k"
        );
    }

    proptest::proptest! {
        #[test]
        fn no_price_table_makes_the_reader_panic(
            text in ".*",
            table in quotadial_core::testing::json(),
        ) {
            let _ = Prices::parse(&text);
            let _ = Prices::parse(&table.to_string());
            let shaped = serde_json::json!({ "updated": "2026-10-23", "anthropic": table });
            if let Some(prices) = Prices::parse(&shaped.to_string()) {
                let _ = prices;
            }
        }
    }
}
