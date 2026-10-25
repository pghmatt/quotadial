//! What a copy of QuotaDial may do (D-090, D-100). QuotaDial Free needs no key, and a licence key
//! adds QuotaDial Pro ([`crate::Standing`], D-099). Every Pro feature asks [`Tier::allows`] of the
//! tier the copy's licence gives, through the daemon crate's `Licensing`, the one switch. Where a
//! Free copy leaves a feature out, it says so in one line where the feature would be
//! ([`ProFeature::line`]), so nothing is silently gone.

use serde::{Deserialize, Serialize};

/// Where QuotaDial Pro is, as the lines that name a Pro feature give it.
pub const PRO_URL: &str = "quotadial.com/pro";

/// A tier of QuotaDial.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    /// QuotaDial Free: one account per provider, 30 days of history, threshold and reset alerts as
    /// the system's notifications, pace, status pages, and read-only status through the CLI, the
    /// local API and MCP (D-098).
    Free,
    /// QuotaDial Pro: everything.
    Pro,
}

/// A feature only QuotaDial Pro has.
// From the PRD's "Pro features", as decided on #49 (D-098).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum ProFeature {
    /// More than one account per provider.
    Accounts,
    /// History kept up to a year, rather than 30 days.
    LongHistory,
    /// Alert rules beyond a threshold and a reset: a window running out before it resets, a time
    /// before a reset, and provider incidents.
    AlertRules,
    /// Alerts sent to webhooks and Slack as well as the system's notifications.
    AlertDelivery,
    /// The week's API-equivalent cost of local use, from the CLIs' logs.
    CostScan,
    /// The accounts ranked by headroom, through the local API and MCP.
    Headroom,
    /// The alert rules through the local API and MCP.
    RulesApi,
}

impl ProFeature {
    /// Every Pro feature, in the order QuotaDial lists them.
    pub const ALL: [Self; 7] = [
        Self::Accounts,
        Self::LongHistory,
        Self::AlertRules,
        Self::AlertDelivery,
        Self::CostScan,
        Self::Headroom,
        Self::RulesApi,
    ];

    /// What the feature gives, as a list of what QuotaDial Pro adds puts it, such as "the week's
    /// cost at API prices".
    #[must_use]
    pub fn what(self) -> &'static str {
        match self {
            Self::Accounts => "every account on this machine, not one per provider",
            Self::LongHistory => "a year of history, not 30 days",
            Self::AlertRules => {
                "alerts before a window runs out, before a reset, and for provider incidents"
            }
            Self::AlertDelivery => "alerts sent to webhooks and Slack",
            Self::CostScan => "the week's cost at API prices",
            Self::Headroom => "headroom through the local API and MCP",
            Self::RulesApi => "alert rules through the local API and MCP",
        }
    }

    /// One line for where a Free copy leaves the feature out: what it is, that it is part of
    /// QuotaDial Pro, and where Pro is.
    #[must_use]
    pub fn line(self) -> String {
        let what = match self {
            Self::Accounts => "Showing every account is",
            Self::LongHistory => "Keeping a year of history, not 30 days, is",
            Self::AlertRules => {
                "Alerts before a window runs out, before a reset, and for provider incidents are"
            }
            Self::AlertDelivery => "Sending alerts to webhooks and Slack is",
            Self::CostScan => "The week's cost at API prices is",
            Self::Headroom => "Headroom through the local API and MCP is",
            Self::RulesApi => "Alert rules through the local API and MCP are",
        };
        format!("{what} part of QuotaDial Pro: {PRO_URL}")
    }
}

impl Tier {
    /// Whether this tier has `feature`.
    #[must_use]
    pub fn allows(self, feature: ProFeature) -> bool {
        // Every feature listed is QuotaDial Pro's; a later tier, such as Team, may hold some.
        matches!((self, feature), (Self::Pro, _))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pro_has_every_feature_and_free_none_of_them() {
        for feature in ProFeature::ALL {
            assert!(Tier::Pro.allows(feature));
            assert!(!Tier::Free.allows(feature));
        }
    }

    #[test]
    fn each_line_names_the_feature_and_where_pro_is() {
        assert_eq!(
            ProFeature::CostScan.line(),
            "The week's cost at API prices is part of QuotaDial Pro: quotadial.com/pro"
        );
        assert_eq!(
            ProFeature::AlertRules.line(),
            "Alerts before a window runs out, before a reset, and for provider incidents are \
             part of QuotaDial Pro: quotadial.com/pro"
        );
        for feature in ProFeature::ALL {
            assert!(feature.line().ends_with(PRO_URL), "{feature:?}");
            assert!(!feature.what().is_empty());
        }
    }

    #[test]
    fn a_feature_is_named_in_json_by_its_snake_case_name() {
        assert_eq!(
            serde_json::to_value(ProFeature::CostScan).expect("serializes"),
            "cost_scan"
        );
        assert_eq!(
            serde_json::to_value(Tier::Free).expect("serializes"),
            "free"
        );
    }
}
