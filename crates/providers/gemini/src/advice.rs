//! The one line QuotaDial shows for each way Gemini CLI signs in: why there are no numbers and,
//! for a Google sign-in, where Gemini CLI shows them.
//!
//! The command named is the one that shows the limits in Gemini CLI's own source (gemini-cli at
//! cd1b45c, read on 2026-10-25): `/model` opens the model dialog, which asks for the limits afresh
//! and shows each model's share used and when it resets (`modelCommand.ts`, `ModelDialog.tsx` and
//! `ModelQuotaDisplay.tsx` under `packages/cli/src/ui`). `/stats model`, which Gemini CLI's docs name
//! for this (`docs/resources/quota-and-pricing.md`), exists too, but shows a limit only after a
//! request in the same session and only while the model is set to auto (`ModelStatsDisplay.tsx`),
//! so a person who types it first sees "No API calls have been made in this session."

/// Signed in with Google: Google's terms forbid using Gemini CLI's sign-in from another tool
/// (Gemini CLI's FAQ and terms, read on #73), so QuotaDial reads none of it.
pub(crate) const GOOGLE: &str = "Google's terms do not let other tools use Gemini CLI's sign-in, \
                                 so QuotaDial shows no numbers for it. In Gemini CLI, `/model` \
                                 shows each model's share used and when it resets.";

/// Gemini CLI's settings name a Gemini API key, which is paid for by use, not by a plan.
pub(crate) const API_KEY: &str =
    "Gemini CLI uses a Gemini API key, which has no plan limits to show.";

/// Gemini CLI's settings name Vertex AI, which Google Cloud bills by use.
pub(crate) const VERTEX_AI: &str = "Gemini CLI uses Vertex AI, which has no plan limits to show.";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_line_is_one_plain_sentence_or_two() {
        for line in [GOOGLE, API_KEY, VERTEX_AI] {
            assert!(!line.contains('\n') && line.ends_with('.'), "{line}");
            assert!(
                !line.contains('\u{2014}') && !line.contains('\u{2013}'),
                "{line}"
            );
            // #52's plain words: never these in anything a user reads.
            for word in line
                .split(|c: char| !c.is_ascii_alphabetic())
                .map(str::to_ascii_lowercase)
            {
                assert!(
                    !matches!(
                        word.as_str(),
                        "quota" | "quotas" | "auth" | "credential" | "credentials"
                    ),
                    "{line}"
                );
            }
        }
    }

    #[test]
    fn a_google_sign_in_names_the_command_that_shows_its_limits() {
        assert!(GOOGLE.contains("`/model`"));
        assert!(GOOGLE.contains("Google's terms"));
    }
}
