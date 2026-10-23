//! Claude plan names, normalized to the ids Claude Code uses in its credentials: `max`, `pro`,
//! `team`, `enterprise`.
//!
//! Ported from CodexBar's `Providers/Claude/ClaudePlan.swift` (`fromRateLimitTier`).

/// A `subscriptionType` from Claude Code's credentials, such as `max`.
pub(crate) fn from_subscription_type(value: &str) -> Option<String> {
    let plan = value.trim().to_lowercase();
    (!plan.is_empty()).then_some(plan)
}

/// An `organizationType` from Claude Code's account config, such as `claude_max`.
pub(crate) fn from_organization_type(value: &str) -> Option<String> {
    let value = value.trim().to_lowercase();
    match value.strip_prefix("claude_") {
        Some(plan) if !plan.is_empty() => Some(plan.to_owned()),
        _ => from_tier(&value).map(str::to_owned),
    }
}

/// A rate-limit tier such as `default_claude_max_20x`.
pub(crate) fn from_tier(value: &str) -> Option<&'static str> {
    let tier = value.trim().to_lowercase();
    ["max", "pro", "team", "enterprise"]
        .into_iter()
        .find(|plan| tier.contains(plan))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subscription_types_are_trimmed_and_lowercased() {
        assert_eq!(from_subscription_type(" Max ").as_deref(), Some("max"));
        assert_eq!(from_subscription_type("  "), None);
    }

    #[test]
    fn organization_types_drop_the_claude_prefix() {
        assert_eq!(from_organization_type("claude_max").as_deref(), Some("max"));
        assert_eq!(
            from_organization_type("claude_enterprise").as_deref(),
            Some("enterprise")
        );
        assert_eq!(from_organization_type("team_plan").as_deref(), Some("team"));
        assert_eq!(from_organization_type("api"), None);
    }

    #[test]
    fn tiers_name_their_plan() {
        assert_eq!(from_tier("default_claude_max_20x"), Some("max"));
        assert_eq!(from_tier("claude_pro"), Some("pro"));
        assert_eq!(from_tier("default_raven"), None);
    }
}
