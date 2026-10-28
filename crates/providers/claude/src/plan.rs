//! Claude plan names, from what Claude Code's account file records, normalized to `max`, `pro`,
//! `team` and `enterprise`.
//!
//! Ported from CodexBar's `Providers/Claude/ClaudePlan.swift` (`fromRateLimitTier`).

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
