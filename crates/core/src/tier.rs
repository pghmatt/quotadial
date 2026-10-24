//! What this copy of QuotaDial may do (D-090). Licensing comes in M5; until then every copy has every
//! feature. Each feature the Pro tier holds asks [`tier`] here, so licensing changes one place.

/// A tier of QuotaDial.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    /// The free tier.
    Free,
    /// The paid tier.
    Pro,
}

/// A feature only the Pro tier has, from the PRD's "Pro features".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProFeature {
    /// Alert rules beyond a threshold and a reset: a window running out before it resets, a time
    /// before a reset, and provider incidents.
    AlertRules,
    /// Alerts sent to webhooks and Slack as well as the system's notifications.
    AlertDelivery,
    /// History kept up to a year, rather than 30 days.
    LongHistory,
}

/// This copy's tier: Pro for every copy until M5's licensing.
#[must_use]
pub fn tier() -> Tier {
    Tier::Pro
}

impl Tier {
    /// Whether this tier has `feature`.
    #[must_use]
    pub fn allows(self, feature: ProFeature) -> bool {
        match (self, feature) {
            (Self::Pro, _) => true,
            (
                Self::Free,
                ProFeature::AlertRules | ProFeature::AlertDelivery | ProFeature::LongHistory,
            ) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_copy_is_pro_until_licensing_arrives() {
        assert_eq!(tier(), Tier::Pro);
        for feature in [
            ProFeature::AlertRules,
            ProFeature::AlertDelivery,
            ProFeature::LongHistory,
        ] {
            assert!(tier().allows(feature));
            assert!(!Tier::Free.allows(feature));
        }
    }
}
