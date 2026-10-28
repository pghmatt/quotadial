//! How the two windows Claude Code's status line handed over are shown (D-145).
//!
//! They keep the identity they had when Claude's limits came from Anthropic's usage address, as
//! `usage.rs`'s `KNOWN_BUCKETS` gave it (the manager's rule 2 on #73): `five_hour` is the session
//! window, "Session", 18,000 seconds long, and `seven_day` the weekly one, "Weekly", 604,800
//! seconds. A reading from the status line is a reading of the same window as one from the usage
//! address was, so a person's History, its pace and their alert rules carry across with no seam.
//!
//! A window whose reset has passed since the handover is left out, as D-126 leaves out
//! Antigravity's: what is left of it now is unknown until Claude Code hands its numbers over again.

use chrono::{DateTime, Utc};
use quotadial_core::{Source, Window, WindowKind};

use crate::handover::{FIVE_HOUR, Handover, SEVEN_DAY};

/// Each window by its name, with its kind, its label and its length in seconds, in the order
/// QuotaDial shows them.
pub(crate) const NAMED: [(&str, WindowKind, &str, u64); 2] = [
    (FIVE_HOUR, WindowKind::Session, "Session", 5 * 60 * 60),
    (SEVEN_DAY, WindowKind::Weekly, "Weekly", 7 * 24 * 60 * 60),
];

/// What `handover`, handed over at `handed_over`, shows at `now`: a window for each it holds that
/// has not reset since, from the status line, read at `handed_over`.
pub(crate) fn shown(
    handover: &Handover,
    handed_over: DateTime<Utc>,
    now: DateTime<Utc>,
) -> Vec<Window> {
    NAMED
        .iter()
        .filter_map(|(name, kind, label, seconds)| {
            let limit = handover.windows.get(name)?;
            if limit.resets_at.is_some_and(|at| at <= now) {
                return None;
            }
            Some(Window {
                kind: *kind,
                label: (*label).to_owned(),
                used_pct: limit.used_percentage,
                resets_at: limit.resets_at,
                window_seconds: *seconds,
                scope_model: None,
                source: Source::StatusLine,
                fetched_at: handed_over,
                unconfirmed: false,
                pace: None,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::handover::Limit;

    fn at(rfc3339: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(rfc3339)
            .expect("a time")
            .with_timezone(&Utc)
    }

    fn handover() -> Handover {
        Handover {
            written_at: at("2026-10-28T17:30:00Z"),
            windows: BTreeMap::from([
                (
                    SEVEN_DAY,
                    Limit {
                        used_percentage: 41.2,
                        resets_at: Some(at("2026-11-02T15:30:00Z")),
                    },
                ),
                (
                    FIVE_HOUR,
                    Limit {
                        used_percentage: 23.5,
                        resets_at: Some(at("2026-10-28T22:30:00Z")),
                    },
                ),
            ]),
        }
    }

    #[test]
    fn the_two_windows_keep_the_identity_they_had_from_the_usage_address() {
        // As `usage.rs`'s `KNOWN_BUCKETS` had them: id, kind, label and length (the manager's
        // rule 2).
        assert_eq!(
            NAMED,
            [
                ("five_hour", WindowKind::Session, "Session", 18_000),
                ("seven_day", WindowKind::Weekly, "Weekly", 604_800),
            ]
        );
        let handed_over = at("2026-10-28T17:29:00Z");

        let windows = shown(&handover(), handed_over, at("2026-10-28T17:30:00Z"));

        assert_eq!(
            windows,
            [
                Window {
                    kind: WindowKind::Session,
                    label: "Session".into(),
                    used_pct: 23.5,
                    resets_at: Some(at("2026-10-28T22:30:00Z")),
                    window_seconds: 18_000,
                    scope_model: None,
                    source: Source::StatusLine,
                    fetched_at: handed_over,
                    unconfirmed: false,
                    pace: None,
                },
                Window {
                    kind: WindowKind::Weekly,
                    label: "Weekly".into(),
                    used_pct: 41.2,
                    resets_at: Some(at("2026-11-02T15:30:00Z")),
                    window_seconds: 604_800,
                    scope_model: None,
                    source: Source::StatusLine,
                    fetched_at: handed_over,
                    unconfirmed: false,
                    pace: None,
                },
            ]
        );
    }

    #[test]
    fn a_window_whose_reset_has_passed_since_the_handover_is_left_out() {
        let handed_over = at("2026-10-28T17:30:00Z");
        let labels = |now: &str| -> Vec<String> {
            shown(&handover(), handed_over, at(now))
                .into_iter()
                .map(|window| window.label)
                .collect()
        };

        assert_eq!(labels("2026-10-28T22:29:59Z"), ["Session", "Weekly"]);
        assert_eq!(labels("2026-10-28T22:30:00Z"), ["Weekly"], "at its reset");
        assert_eq!(labels("2026-11-02T15:30:00Z"), Vec::<String>::new());
    }

    #[test]
    fn a_window_with_no_reset_stays_until_numbers_come_again() {
        let mut handover = handover();
        handover
            .windows
            .get_mut(SEVEN_DAY)
            .expect("the weekly window")
            .resets_at = None;

        let windows = shown(
            &handover,
            at("2026-10-28T17:30:00Z"),
            at("2027-10-28T17:30:00Z"),
        );

        assert_eq!(windows.len(), 1);
        assert_eq!(windows[0].label, "Weekly");
        assert_eq!(windows[0].resets_at, None);
    }
}
