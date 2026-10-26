//! How each limit Antigravity CLI handed over is shown: as a window with its share used and its
//! reset, named from its id, or, for a limit with an amount left and no share, by its amount and
//! reset in words, never as a share made up for it (P-023).
//!
//! The names follow CodexBar's for Antigravity's limits (`Providers/Antigravity/
//! AntigravityStatusProbe.swift`, `quotaBucketKind`, `sessionCadenceAliases` and
//! `displayTitle(forQuotaGroup:)`, and `AntigravityQuotaFamilyVisibility.swift`, at 4654968e9):
//! an id ending `-5h`, `-session`, `-5-hour` or `-five-hour` is a five-hour limit and one ending
//! `-weekly` a weekly one, `_` read as `-`, in any case; what comes before is the group of models it
//! covers, `gemini` for Gemini and `3p` for Claude and GPT, Antigravity's two today. Any other id is
//! a limit QuotaDial does not know, shown by its name with no length (`other`, P-014).

use std::collections::HashSet;

use chrono::{DateTime, Utc};
use quotadial_core::{Source, Window, WindowKind, outside_text};

use crate::handover::{Handover, Limit};

/// A five-hour limit's length.
pub(crate) const FIVE_HOURS: u64 = 5 * 60 * 60;

/// A weekly limit's length.
pub(crate) const WEEK: u64 = 7 * 24 * 60 * 60;

/// The longest name shown for a group or a limit QuotaDial does not know, in characters.
const MOST_NAME_CHARS: usize = 64;

/// The lengths Antigravity's limits come in: the kind, the word that names it and its length, and
/// the endings of its ids.
const LENGTHS: [(WindowKind, &str, u64, &[&str]); 2] = [
    (
        WindowKind::Session,
        "5-hour",
        FIVE_HOURS,
        &["5h", "session", "5-hour", "five-hour"],
    ),
    (WindowKind::Weekly, "Weekly", WEEK, &["weekly"]),
];

/// What a handover shows.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct Shown {
    /// A window for each limit with a share left, Gemini's first, each group's five-hour limit
    /// before its weekly one.
    pub(crate) windows: Vec<Window>,
    /// Each limit with an amount left and no share, in the same order.
    pub(crate) amounts: Vec<Amount>,
}

/// A limit Antigravity CLI gives an amount left for and no share of.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Amount {
    /// Its name, as a window of it would be labelled.
    pub(crate) label: String,
    /// How much is left, in units Antigravity does not name.
    pub(crate) left: f64,
    /// When it resets, when Antigravity CLI says.
    pub(crate) resets_at: Option<DateTime<Utc>>,
}

/// What `handover`, handed over at `handed_over`, shows at `now`. A limit Antigravity CLI marks
/// turned off is left out, as Claude's provider leaves out a limit Anthropic sends as null: a share
/// shown for it would be read as a limit that counts. A limit whose reset has passed since is left
/// out too: what is left of it now is unknown until Antigravity CLI hands over its numbers again.
pub(crate) fn shown(handover: &Handover, handed_over: DateTime<Utc>, now: DateTime<Utc>) -> Shown {
    let mut named: Vec<(Name, &String, &Limit)> = handover
        .limits
        .iter()
        .filter(|(_, limit)| !limit.disabled)
        .map(|(id, limit)| (Name::of(id), id, limit))
        .collect();
    named.sort_by(|(a, a_id, _), (b, b_id, _)| a.rank().cmp(&b.rank()).then(a_id.cmp(b_id)));
    let mut shown = Shown::default();
    let mut labels = HashSet::new();
    for (name, _, limit) in named {
        let resets_at = limit.resets_at(handover.written_at);
        if resets_at.is_some_and(|at| at <= now) || !labels.insert(name.label.clone()) {
            continue;
        }
        match (limit.remaining_fraction, limit.remaining_amount) {
            (Some(fraction), _) => shown.windows.push(Window {
                kind: name.kind,
                label: name.label,
                used_pct: used(fraction),
                resets_at,
                window_seconds: name.seconds,
                scope_model: name.group.map(|group| group.to_lowercase()),
                source: Source::StatusLine,
                fetched_at: handed_over,
                unconfirmed: false,
                pace: None,
            }),
            (None, Some(left)) => shown.amounts.push(Amount {
                label: name.label,
                left,
                resets_at,
            }),
            (None, None) => {}
        }
    }
    shown
}

/// The share used, in percent, of a limit with `fraction` left, to a millionth of a point, so
/// 0.9378 left is 6.22 used rather than 6.219999999999999.
fn used(fraction: f64) -> f64 {
    ((1.0 - fraction) * 100.0 * 1e6).round() / 1e6
}

/// How a limit is named, from its id.
#[derive(Debug, Clone, PartialEq)]
struct Name {
    kind: WindowKind,
    /// Such as `5-hour (Gemini)`.
    label: String,
    seconds: u64,
    /// The group of models it covers, such as `Gemini`.
    group: Option<String>,
}

impl Name {
    fn of(id: &str) -> Self {
        let key = id.trim().to_lowercase().replace('_', "-");
        for (kind, word, seconds, endings) in LENGTHS {
            for ending in endings {
                let rest = if key == *ending {
                    Some("")
                } else {
                    key.strip_suffix(ending)
                        .and_then(|rest| rest.strip_suffix('-'))
                };
                if let Some(rest) = rest {
                    let group = group(rest);
                    let label = match &group {
                        Some(group) => format!("{word} ({group})"),
                        None => word.to_owned(),
                    };
                    return Self {
                        kind,
                        label,
                        seconds,
                        group,
                    };
                }
            }
        }
        Self {
            kind: WindowKind::Other,
            label: humanize(id),
            seconds: 0,
            group: None,
        }
    }

    /// Where it comes in the list: Gemini's first, then Claude and GPT's, then the rest by name, and
    /// in each, the five-hour limit, the weekly one, then any other.
    fn rank(&self) -> (u8, String, u8) {
        let group = match self.group.as_deref() {
            Some(GEMINI) => 0,
            Some(CLAUDE_AND_GPT) => 1,
            _ => 2,
        };
        let length = match self.kind {
            WindowKind::Session => 0,
            WindowKind::Weekly => 1,
            _ => 2,
        };
        (group, self.group.clone().unwrap_or_default(), length)
    }
}

const GEMINI: &str = "Gemini";
const CLAUDE_AND_GPT: &str = "Claude and GPT";

/// The group of models an id's start names: `gemini` and `3p` (or `third-party`) as Antigravity
/// names its two, any other in words, and none for an id that is only its length.
fn group(start: &str) -> Option<String> {
    match start {
        "" => None,
        "gemini" => Some(GEMINI.to_owned()),
        "3p" | "third-party" => Some(CLAUDE_AND_GPT.to_owned()),
        other => Some(humanize(other)),
    }
}

/// `"gemini-4-argon"` as `"Gemini 4 argon"`. The id is Antigravity's text, so it is cleaned as any
/// text from outside is.
fn humanize(id: &str) -> String {
    let words = outside_text(&id.replace(['-', '_'], " "), MOST_NAME_CHARS)
        .unwrap_or_else(|| id.to_owned());
    let mut chars = words.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => words,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    fn at(rfc3339: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(rfc3339)
            .expect("a time")
            .with_timezone(&Utc)
    }

    fn now() -> DateTime<Utc> {
        at("2026-10-25T22:30:00Z")
    }

    fn limit(fraction: Option<f64>, amount: Option<f64>, reset: &str) -> Limit {
        Limit {
            remaining_fraction: fraction,
            remaining_amount: amount,
            reset_time: Some(at(reset)),
            reset_in_seconds: None,
            disabled: false,
        }
    }

    fn handover(limits: &[(&str, Limit)]) -> Handover {
        Handover {
            written_at: now(),
            email: None,
            plan_tier: None,
            limits: limits
                .iter()
                .map(|(id, limit)| ((*id).to_owned(), limit.clone()))
                .collect::<BTreeMap<_, _>>(),
        }
    }

    #[test]
    fn each_limit_is_named_from_its_id_as_codexbar_names_antigravity_s() {
        for (id, kind, label, seconds, group) in [
            (
                "gemini-5h",
                WindowKind::Session,
                "5-hour (Gemini)",
                FIVE_HOURS,
                Some("Gemini"),
            ),
            (
                "gemini-weekly",
                WindowKind::Weekly,
                "Weekly (Gemini)",
                WEEK,
                Some("Gemini"),
            ),
            (
                "3p-5h",
                WindowKind::Session,
                "5-hour (Claude and GPT)",
                FIVE_HOURS,
                Some("Claude and GPT"),
            ),
            (
                "3P_WEEKLY",
                WindowKind::Weekly,
                "Weekly (Claude and GPT)",
                WEEK,
                Some("Claude and GPT"),
            ),
            (
                "third-party-five-hour",
                WindowKind::Session,
                "5-hour (Claude and GPT)",
                FIVE_HOURS,
                Some("Claude and GPT"),
            ),
            (
                "gemini-session",
                WindowKind::Session,
                "5-hour (Gemini)",
                FIVE_HOURS,
                Some("Gemini"),
            ),
            (
                "gemini-4-argon-weekly",
                WindowKind::Weekly,
                "Weekly (Gemini 4 argon)",
                WEEK,
                Some("Gemini 4 argon"),
            ),
            ("weekly", WindowKind::Weekly, "Weekly", WEEK, None),
            ("5h", WindowKind::Session, "5-hour", FIVE_HOURS, None),
            ("gemini-25h", WindowKind::Other, "Gemini 25h", 0, None),
            ("ai-credits", WindowKind::Other, "Ai credits", 0, None),
            ("-", WindowKind::Other, "-", 0, None),
        ] {
            assert_eq!(
                Name::of(id),
                Name {
                    kind,
                    label: label.to_owned(),
                    seconds,
                    group: group.map(str::to_owned)
                },
                "{id}"
            );
        }
    }

    #[test]
    fn a_share_left_is_shown_as_the_share_used_with_its_reset_and_where_it_came_from() {
        let handed_over = at("2026-10-25T22:29:00Z");
        let shown = shown(
            &handover(&[(
                "gemini-weekly",
                limit(Some(0.9378), None, "2026-10-31T07:50:32Z"),
            )]),
            handed_over,
            now(),
        );

        assert_eq!(
            shown.windows,
            [Window {
                kind: WindowKind::Weekly,
                label: "Weekly (Gemini)".into(),
                used_pct: 6.22,
                resets_at: Some(at("2026-10-31T07:50:32Z")),
                window_seconds: WEEK,
                scope_model: Some("gemini".into()),
                source: Source::StatusLine,
                fetched_at: handed_over,
                unconfirmed: false,
                pace: None,
            }]
        );
        assert!(shown.amounts.is_empty());
        assert_eq!(used(1.0), 0.0);
        assert_eq!(used(0.0), 100.0);
        assert_eq!(used(0.5), 50.0);
    }

    #[test]
    fn a_reset_given_in_seconds_counts_from_when_the_command_took_the_handover() {
        let mut gemini = limit(Some(0.5), None, "2026-10-25T00:00:00Z");
        gemini.reset_time = None;
        gemini.reset_in_seconds = Some(3600);
        // A file whose modified time is earlier than the time its command recorded.
        let handed_over = at("2026-10-25T22:00:00Z");

        let shown = shown(&handover(&[("gemini-5h", gemini)]), handed_over, now());

        assert_eq!(shown.windows[0].resets_at, Some(at("2026-10-25T23:30:00Z")));
        assert_eq!(shown.windows[0].fetched_at, handed_over);
    }

    #[test]
    fn a_limit_marked_turned_off_is_left_out() {
        let mut off = limit(Some(1.0), None, "2026-10-26T02:00:00Z");
        off.disabled = true;
        let mut amount_off = limit(None, Some(5.0), "2026-10-26T02:00:00Z");
        amount_off.disabled = true;

        let shown = shown(
            &handover(&[
                ("3p-5h", off),
                ("3p-weekly", amount_off),
                ("gemini-5h", limit(Some(0.5), None, "2026-10-26T02:00:00Z")),
            ]),
            now(),
            now(),
        );

        let labels: Vec<&str> = shown.windows.iter().map(|w| w.label.as_str()).collect();
        assert_eq!(labels, ["5-hour (Gemini)"]);
        assert!(shown.amounts.is_empty());
    }

    #[test]
    fn a_limit_that_has_reset_since_the_handover_is_left_out() {
        let handed_over = at("2026-10-25T10:00:00Z");
        let shown = shown(
            &handover(&[
                ("gemini-5h", limit(Some(0.1), None, "2026-10-25T14:00:00Z")),
                (
                    "gemini-weekly",
                    limit(Some(0.6), None, "2026-10-31T00:00:00Z"),
                ),
                ("3p-weekly", limit(None, Some(3.0), "2026-10-25T22:30:00Z")),
            ]),
            handed_over,
            now(),
        );

        let labels: Vec<&str> = shown.windows.iter().map(|w| w.label.as_str()).collect();
        assert_eq!(
            labels,
            ["Weekly (Gemini)"],
            "the five-hour limit reset at 14:00"
        );
        assert!(
            shown.amounts.is_empty(),
            "a reset at this very moment has passed"
        );
    }

    #[test]
    fn an_amount_with_no_share_is_never_a_window() {
        let shown = shown(
            &handover(&[
                (
                    "3p-weekly",
                    limit(None, Some(120.0), "2026-10-31T00:00:00Z"),
                ),
                (
                    "gemini-weekly",
                    limit(Some(0.25), Some(40.0), "2026-10-31T00:00:00Z"),
                ),
            ]),
            now(),
            now(),
        );

        assert_eq!(
            shown.amounts,
            [Amount {
                label: "Weekly (Claude and GPT)".into(),
                left: 120.0,
                resets_at: Some(at("2026-10-31T00:00:00Z")),
            }]
        );
        assert_eq!(shown.windows.len(), 1, "a share wins over an amount");
        assert_eq!(shown.windows[0].used_pct, 75.0);
    }

    #[test]
    fn gemini_comes_first_each_five_hour_limit_before_its_weekly_one_and_a_name_shows_once() {
        let reset = "2026-10-31T00:00:00Z";
        let shown = shown(
            &handover(&[
                ("3p-weekly", limit(Some(0.5), None, reset)),
                ("3p-5h", limit(Some(0.5), None, reset)),
                ("ai-credits", limit(Some(0.5), None, reset)),
                ("gemini-weekly", limit(Some(0.5), None, reset)),
                ("gemini_weekly", limit(Some(0.1), None, reset)),
                ("gemini-5h", limit(Some(0.5), None, reset)),
            ]),
            now(),
            now(),
        );

        let labels: Vec<&str> = shown.windows.iter().map(|w| w.label.as_str()).collect();
        assert_eq!(
            labels,
            [
                "5-hour (Gemini)",
                "Weekly (Gemini)",
                "5-hour (Claude and GPT)",
                "Weekly (Claude and GPT)",
                "Ai credits",
            ]
        );
        assert_eq!(
            shown.windows[1].used_pct, 50.0,
            "the first of two ids named alike, by id, shows"
        );
    }
}
