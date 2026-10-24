//! Alert rules: what a user asks to be told about their limits, written in plain words, such as
//! `warn me at 80%`, and read by a small grammar here, with no model call (D-090, D-093).
//!
//! A rule has a condition and a scope. The conditions are the PRD's: a window reaching a share
//! used, a window resetting, a window that at its pace runs out before it resets, and a time before
//! a reset. A fifth, an account needing to sign in again, carries M3's notification over. The scope
//! narrows a rule to one provider or one kind of window; without one, a rule covers every window of
//! every account.

use std::fmt;

use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};

use crate::{Account, Pace, Snapshot, Status, StatusPage, Window, WindowKind, text};

/// The forms the grammar reads, offered when it cannot read a rule.
pub const EXAMPLES: [&str; 3] = [
    "warn me at 80%",
    "warn me when the weekly window will run out before it resets",
    "tell me 30 minutes before the session resets if it is above 50%",
];

/// The longest time before a reset a rule can ask for: a week, the longest window.
const MAX_MINUTES: u32 = 7 * 24 * 60;

/// What a rule watches for.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Condition {
    /// A window reaches `pct`% used.
    Threshold { pct: f64 },
    /// A window resets.
    Reset,
    /// At its pace, a window runs out before it resets (P-016).
    RunsOut,
    /// A window resets within `minutes`, while at least `above_pct`% of it is used.
    BeforeReset {
        minutes: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        above_pct: Option<f64>,
    },
    /// An account needs signing in again.
    SignIn,
    /// A provider's status page reports an incident (D-096).
    Incident,
}

/// Which accounts and windows a rule covers: all of them, unless it names a provider or a window.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scope {
    /// Only this provider's accounts, such as `claude`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    /// Only these windows: `session`, `weekly` or `daily` for that kind of window; otherwise a
    /// model, such as `opus`, or a window's label, such as `Weekly (Cowork)`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window: Option<String>,
}

/// A rule: its condition and its scope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rule {
    #[serde(flatten)]
    pub condition: Condition,
    #[serde(flatten)]
    pub scope: Scope,
}

/// Why a rule could not be read. The message names the forms the grammar reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleError(String);

impl fmt::Display for RuleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for RuleError {}

/// One time a rule's condition held.
#[derive(Debug, Clone, PartialEq)]
pub struct Occurrence {
    /// The window's label, or `None` for an account's sign-in.
    pub label: Option<String>,
    /// The window's model, for a per-model window.
    pub scope_model: Option<String>,
    /// Which time this is, so it is told once: the window's reset for most conditions, the reset
    /// that passed for a reset, and when the account last had numbers for a sign-in. `None` when
    /// there is no such time, as for a window that reports no reset.
    pub instance: Option<DateTime<Utc>>,
    /// The window's share used.
    pub used_pct: Option<f64>,
    /// When the window resets.
    pub resets_at: Option<DateTime<Utc>>,
    /// The window's pace, for a window that runs out.
    pub pace: Option<Pace>,
    /// Why the account needs signing in, naming the fix.
    pub note: Option<String>,
}

impl Rule {
    /// Reads a rule written in plain words, such as the [`EXAMPLES`]. It reads:
    ///
    /// - a share used, such as `80%` or `80 percent`: a threshold, or with a time before a reset,
    ///   the share used it must be above;
    /// - a time, such as `30 minutes`, `2 h` or `an hour`, with a reset: a time before the reset;
    /// - `run out` or `pace`: a window running out before it resets;
    /// - `reset` alone: a window resetting;
    /// - `sign in`, `log in` or `expired`: an account needing to sign in again;
    /// - `incident`, `outage` or `down`: a provider's status page reporting an incident;
    /// - `session`, `weekly`, `daily`, a model such as `opus`, or a label in quotes: the window;
    /// - `claude` or `codex` (also `anthropic`, `chatgpt`, `openai`): the provider.
    ///
    /// # Errors
    ///
    /// When it finds none of these, more than one of a kind, or a share or time out of range.
    pub fn parse(written: &str) -> Result<Self, RuleError> {
        let unread = || {
            RuleError(format!(
                "QuotaDial could not read {:?} as a rule. Write it like one of these: \"{}\".",
                written.trim(),
                EXAMPLES.join("\", \"")
            ))
        };
        let (label, rest) = quoted(written);
        let words = words(&rest);
        let mut read = Read {
            label,
            ..Read::default()
        };
        for (index, word) in words.iter().enumerate() {
            let next = |ahead: usize| words.get(index + ahead).map(String::as_str);
            if let Ok(number) = word.parse::<f64>() {
                match next(1) {
                    Some("%" | "percent") => read.pct(number)?,
                    Some("per") if next(2) == Some("cent") => read.pct(number)?,
                    Some(unit) => {
                        if let Some(minutes) = in_minutes(unit) {
                            read.minutes(number * minutes)?;
                        }
                    }
                    None => {}
                }
                continue;
            }
            match word.as_str() {
                "a" | "an" | "one" if next(1).and_then(in_minutes).is_some() => {
                    if index == 0 || words[index - 1] != "half" {
                        read.minutes(next(1).and_then(in_minutes).unwrap_or_default())?;
                    }
                }
                "half" if matches!(next(1), Some("a" | "an")) && next(2) == Some("hour") => {
                    read.minutes(30.0)?;
                }
                "before" => read.before = true,
                "pace" => read.runs_out = true,
                "run" | "runs" | "running" | "ran" if next(1) == Some("out") => {
                    read.runs_out = true;
                }
                "sign" | "signs" | "signing" | "signed" | "log" | "logs" | "logging" | "logged"
                    if matches!(next(1), Some("in" | "out")) =>
                {
                    read.sign_in = true;
                }
                "login" | "signin" | "sign-in" | "log-in" | "expire" | "expires" | "expired" => {
                    read.sign_in = true;
                }
                "incident" | "incidents" | "outage" | "outages" | "down" | "degraded" => {
                    read.incident = true;
                }
                "session" | "5-hour" | "five-hour" => read.kind("session")?,
                "weekly" | "week" => read.kind("weekly")?,
                "daily" => read.kind("daily")?,
                "opus" | "sonnet" | "haiku" | "fable" => read.model(word)?,
                "claude" | "anthropic" => read.provider("claude")?,
                "codex" | "chatgpt" | "openai" => read.provider("codex")?,
                word if word.starts_with("reset") => read.reset = true,
                _ => {}
            }
        }
        let condition = match read {
            Read { sign_in: true, .. } if read.is_only_sign_in() => Condition::SignIn,
            Read { sign_in: true, .. } => return Err(unread()),
            Read { incident: true, .. } if read.is_only_incident() => Condition::Incident,
            Read { incident: true, .. } => return Err(unread()),
            Read {
                minutes: Some(minutes),
                reset: true,
                runs_out: false,
                ..
            } => Condition::BeforeReset {
                minutes,
                above_pct: read.pct,
            },
            Read {
                runs_out: true,
                pct: None,
                minutes: None,
                ..
            } => Condition::RunsOut,
            Read {
                reset: true,
                pct: None,
                minutes: None,
                before: false,
                ..
            } => Condition::Reset,
            Read {
                pct: Some(pct),
                minutes: None,
                reset: false,
                runs_out: false,
                ..
            } => Condition::Threshold { pct },
            _ => return Err(unread()),
        };
        Ok(Self {
            condition,
            scope: Scope {
                window: read.window(),
                provider: read.provider,
            },
        })
    }

    /// Where this rule's condition holds in `snapshot`, at `now`. `last_reset(window)` is when the
    /// window's reading before this one said it would reset, which tells that it has reset since.
    ///
    /// Only an `ok` snapshot's numbers are current enough to alert on, and an unconfirmed 0% never
    /// is; a sign-in rule looks for `auth_required` instead.
    pub fn occurrences(
        &self,
        snapshot: &Snapshot,
        last_reset: impl Fn(&Window) -> Option<DateTime<Utc>>,
        now: DateTime<Utc>,
    ) -> Vec<Occurrence> {
        if !self.scope.covers_account(&snapshot.account) {
            return Vec::new();
        }
        if self.condition == Condition::Incident {
            return Vec::new();
        }
        if self.condition == Condition::SignIn {
            return if snapshot.status == Status::AuthRequired {
                vec![Occurrence {
                    label: None,
                    scope_model: None,
                    instance: snapshot
                        .windows
                        .iter()
                        .map(|window| window.fetched_at)
                        .max(),
                    used_pct: None,
                    resets_at: None,
                    pace: None,
                    note: snapshot.error.clone(),
                }]
            } else {
                Vec::new()
            };
        }
        if snapshot.status != Status::Ok {
            return Vec::new();
        }
        snapshot
            .windows
            .iter()
            .filter(|window| self.scope.covers_window(window) && !window.unconfirmed)
            .filter_map(|window| {
                let instance = match &self.condition {
                    Condition::Threshold { pct } => {
                        (window.used_pct >= *pct).then_some(window.resets_at)?
                    }
                    Condition::RunsOut => {
                        window.pace.as_ref()?.runs_out_at?;
                        window.resets_at
                    }
                    Condition::BeforeReset { minutes, above_pct } => {
                        let resets_at = window.resets_at?;
                        let soon = now < resets_at
                            && resets_at - now <= TimeDelta::minutes(i64::from(*minutes));
                        let above = window.used_pct >= above_pct.unwrap_or(0.0);
                        (soon && above).then_some(Some(resets_at))?
                    }
                    Condition::Reset => {
                        let passed = last_reset(window)?;
                        (passed <= window.fetched_at).then_some(Some(passed))?
                    }
                    Condition::SignIn | Condition::Incident => return None,
                };
                Some(Occurrence {
                    label: Some(window.label.clone()),
                    scope_model: window.scope_model.clone(),
                    instance,
                    used_pct: Some(window.used_pct),
                    resets_at: window.resets_at,
                    pace: window.pace.clone(),
                    note: None,
                })
            })
            .collect()
    }

    /// Where this incident rule holds among `pages`: each page it covers that reports an
    /// incident, keyed by when QuotaDial first saw it, so it is told once however long it lasts.
    #[must_use]
    pub fn incidents<'a>(&self, pages: &'a [StatusPage]) -> Vec<(&'a StatusPage, Occurrence)> {
        if self.condition != Condition::Incident {
            return Vec::new();
        }
        pages
            .iter()
            .filter(|page| {
                page.indicator.is_incident()
                    && self
                        .scope
                        .provider
                        .as_ref()
                        .is_none_or(|provider| *provider == page.provider)
            })
            .map(|page| {
                (
                    page,
                    Occurrence {
                        label: None,
                        scope_model: None,
                        instance: page.since,
                        used_pct: None,
                        resets_at: None,
                        pace: None,
                        note: Some(format!(
                            "{} says: {}.",
                            page.url
                                .trim_start_matches("https://")
                                .trim_end_matches('/'),
                            page.description
                        )),
                    },
                )
            })
            .collect()
    }

    /// What to say about `occurrence` for `account` at `now`: a title and one sentence, as a
    /// notification shows them.
    #[must_use]
    pub fn message(
        &self,
        account: &Account,
        occurrence: &Occurrence,
        now: DateTime<Utc>,
    ) -> (String, String) {
        let provider = text::provider_name(&account.provider);
        let name = &account.display_name;
        let window = occurrence.label.as_deref().unwrap_or_default();
        let used = occurrence
            .used_pct
            .map(|used_pct| format!("{:.0}% used", used_pct.floor()))
            .unwrap_or_default();
        // A window that reports no reset, as a session not yet started again, is not said to have
        // none.
        let resets = occurrence
            .resets_at
            .map(|at| format!(", {}", text::resets(Some(at), now)))
            .unwrap_or_default();
        match &self.condition {
            Condition::Threshold { pct } => (
                format!("{provider} {window} passed {}%", text::percent(*pct)),
                format!("{name}: {used}{resets}."),
            ),
            Condition::Reset => (
                format!("{provider} {window} has reset"),
                format!("{name}: {used} now{resets}."),
            ),
            Condition::RunsOut => {
                let pace = occurrence.pace.as_ref().map_or_else(String::new, |pace| {
                    format!(", {}", text::pace(pace, occurrence.resets_at, now))
                });
                (
                    format!("{provider} {window} runs out before it resets"),
                    format!("{name}: {used}{pace}."),
                )
            }
            Condition::BeforeReset { .. } => (
                format!("{provider} {window} resets soon"),
                format!("{name}: {used}{resets}."),
            ),
            Condition::Incident => (
                format!("{provider} has an incident"),
                occurrence.note.clone().unwrap_or_default(),
            ),
            Condition::SignIn => (
                format!("{provider} needs you to sign in again"),
                format!(
                    "{name}: {}",
                    occurrence
                        .note
                        .as_deref()
                        .unwrap_or("QuotaDial cannot read this account's usage until you sign in.")
                ),
            ),
        }
    }
}

/// What QuotaDial read a rule as, such as `when a Claude session window reaches 80%`, so people can
/// check the grammar read what they meant.
impl fmt::Display for Rule {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let provider = self.scope.provider.as_deref().map(text::provider_name);
        let window = match (provider, self.scope.window.as_deref()) {
            (None, None) => "any window".to_owned(),
            (Some(provider), None) => format!("any {provider} window"),
            (provider, Some(window @ ("session" | "weekly" | "daily"))) => {
                format!(
                    "a {}{window} window",
                    provider.map(|p| format!("{p} ")).unwrap_or_default()
                )
            }
            (provider, Some(name)) => format!(
                "the {}{} window",
                provider.map(|p| format!("{p} ")).unwrap_or_default(),
                capitalized(name)
            ),
        };
        match &self.condition {
            Condition::Threshold { pct } => {
                write!(formatter, "when {window} reaches {}%", text::percent(*pct))
            }
            Condition::Reset => write!(formatter, "when {window} resets"),
            Condition::RunsOut => {
                write!(formatter, "when {window} will run out before it resets")
            }
            Condition::BeforeReset { minutes, above_pct } => {
                let before = text::duration(TimeDelta::minutes(i64::from(*minutes)));
                write!(formatter, "{before} before {window} resets")?;
                match above_pct {
                    Some(pct) => write!(formatter, ", if it is above {}%", text::percent(*pct)),
                    None => Ok(()),
                }
            }
            Condition::SignIn => match provider {
                Some(provider) => write!(
                    formatter,
                    "when a {provider} account needs to sign in again"
                ),
                None => write!(formatter, "when an account needs to sign in again"),
            },
            Condition::Incident => match provider {
                Some(provider) => write!(
                    formatter,
                    "when {provider}'s status page reports an incident"
                ),
                None => write!(
                    formatter,
                    "when a provider's status page reports an incident"
                ),
            },
        }
    }
}

impl Scope {
    /// Whether the rule covers `account`.
    #[must_use]
    pub fn covers_account(&self, account: &Account) -> bool {
        self.provider
            .as_ref()
            .is_none_or(|provider| *provider == account.provider)
    }

    /// Whether the rule covers `window`.
    #[must_use]
    pub fn covers_window(&self, window: &Window) -> bool {
        let Some(name) = &self.window else {
            return true;
        };
        match name.as_str() {
            "session" => window.kind == WindowKind::Session,
            "weekly" => window.kind == WindowKind::Weekly,
            "daily" => window.kind == WindowKind::Daily,
            name => {
                window
                    .scope_model
                    .as_deref()
                    .is_some_and(|model| model.eq_ignore_ascii_case(name))
                    || window.label.eq_ignore_ascii_case(name)
            }
        }
    }
}

/// What the grammar has found so far.
#[derive(Debug, Default)]
struct Read {
    pct: Option<f64>,
    minutes: Option<u32>,
    reset: bool,
    before: bool,
    runs_out: bool,
    sign_in: bool,
    incident: bool,
    /// A window's label, in quotes.
    label: Option<String>,
    /// A model, such as `opus`.
    model: Option<String>,
    /// A kind of window: `session`, `weekly` or `daily`.
    kind: Option<String>,
    provider: Option<String>,
}

impl Read {
    fn pct(&mut self, pct: f64) -> Result<(), RuleError> {
        if !(pct > 0.0 && pct <= 100.0) {
            return Err(RuleError(
                "A rule's share used must be more than 0% and at most 100%.".to_owned(),
            ));
        }
        once(&mut self.pct, pct, "share used")
    }

    fn minutes(&mut self, minutes: f64) -> Result<(), RuleError> {
        if !(1.0..=f64::from(MAX_MINUTES)).contains(&minutes.round()) {
            return Err(RuleError(
                "A rule's time before a reset must be at least a minute and at most a week."
                    .to_owned(),
            ));
        }
        // In range, so the cast keeps the value.
        once(&mut self.minutes, minutes.round() as u32, "time")
    }

    fn kind(&mut self, kind: &str) -> Result<(), RuleError> {
        once(&mut self.kind, kind.to_owned(), "window")
    }

    fn model(&mut self, model: &str) -> Result<(), RuleError> {
        once(&mut self.model, model.to_owned(), "model")
    }

    /// The window named: a label in quotes, else a model, whose windows are weekly, else a kind.
    fn window(&self) -> Option<String> {
        self.label
            .clone()
            .or_else(|| self.model.clone())
            .or_else(|| self.kind.clone())
    }

    fn provider(&mut self, provider: &str) -> Result<(), RuleError> {
        once(&mut self.provider, provider.to_owned(), "provider")
    }

    /// Whether only an incident was asked for: an incident rule covers whole providers.
    fn is_only_incident(&self) -> bool {
        self.names_nothing_else() && !self.sign_in
    }

    /// Whether only a sign-in was asked for: a sign-in rule covers whole accounts.
    fn is_only_sign_in(&self) -> bool {
        self.names_nothing_else() && !self.incident
    }

    /// Whether the rule names no share, time, reset, running out or window.
    fn names_nothing_else(&self) -> bool {
        self.pct.is_none()
            && self.minutes.is_none()
            && !self.reset
            && !self.runs_out
            && self.window().is_none()
    }
}

/// Keeps `value` in `slot`, unless the rule already named a different one.
fn once<T: PartialEq>(slot: &mut Option<T>, value: T, what: &str) -> Result<(), RuleError> {
    match slot {
        Some(kept) if *kept != value => Err(RuleError(format!(
            "A rule can name one {what}; write a rule for each."
        ))),
        _ => {
            *slot = Some(value);
            Ok(())
        }
    }
}

/// A label written in quotes, and the rest of the text.
fn quoted(written: &str) -> (Option<String>, String) {
    let unquoted = || (None, written.to_owned());
    let Some(open) = written.find(['"', '\u{201c}']) else {
        return unquoted();
    };
    let start = open + written[open..].chars().next().map_or(1, char::len_utf8);
    let Some(length) = written[start..].find(['"', '\u{201d}']) else {
        return unquoted();
    };
    let end = start + length;
    let label = written[start..end].trim();
    if label.is_empty() {
        return unquoted();
    }
    let after = end + written[end..].chars().next().map_or(1, char::len_utf8);
    (
        Some(label.to_owned()),
        format!("{} {}", &written[..open], &written[after..]),
    )
}

/// The text's words in lower case, with `%` a word of its own and a number run into its unit, as
/// in `30min`, split from it.
fn words(text: &str) -> Vec<String> {
    let mut words = Vec::new();
    for raw in text
        .to_lowercase()
        .split(|c: char| !(c.is_alphanumeric() || matches!(c, '%' | '.' | '-')))
    {
        for part in raw.split_inclusive('%') {
            let (word, percent) = part
                .strip_suffix('%')
                .map_or((part, false), |word| (word, true));
            let word = match word.trim_matches(['.', '-']) {
                // The five-hour session, not five hours.
                "5h" => "5-hour",
                word => word,
            };
            let digits = word
                .find(|c: char| !(c.is_ascii_digit() || c == '.'))
                .unwrap_or(word.len());
            if digits > 0 && digits < word.len() && !word.contains('-') {
                words.push(word[..digits].to_owned());
                words.push(word[digits..].to_owned());
            } else if !word.is_empty() {
                words.push(word.to_owned());
            }
            if percent {
                words.push("%".to_owned());
            }
        }
    }
    words
}

/// `name` with its first letter in capitals, as `Opus` for `opus`.
fn capitalized(name: &str) -> String {
    let mut letters = name.chars();
    letters.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(letters).collect()
    })
}

/// How many minutes one of `unit` is.
fn in_minutes(unit: &str) -> Option<f64> {
    match unit {
        "m" | "min" | "mins" | "minute" | "minutes" => Some(1.0),
        "h" | "hr" | "hrs" | "hour" | "hours" => Some(60.0),
        "d" | "day" | "days" => Some(24.0 * 60.0),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::{Source, Window};

    use super::*;

    fn rule(written: &str) -> Rule {
        Rule::parse(written).unwrap_or_else(|error| panic!("{written:?}: {error}"))
    }

    fn scope(provider: Option<&str>, window: Option<&str>) -> Scope {
        Scope {
            provider: provider.map(str::to_owned),
            window: window.map(str::to_owned),
        }
    }

    #[test]
    fn the_examples_read_as_the_three_kinds_they_show() {
        assert_eq!(
            Rule::parse(EXAMPLES[0]),
            Ok(Rule {
                condition: Condition::Threshold { pct: 80.0 },
                scope: Scope::default(),
            })
        );
        assert_eq!(
            Rule::parse(EXAMPLES[1]),
            Ok(Rule {
                condition: Condition::RunsOut,
                scope: scope(None, Some("weekly")),
            })
        );
        assert_eq!(
            Rule::parse(EXAMPLES[2]),
            Ok(Rule {
                condition: Condition::BeforeReset {
                    minutes: 30,
                    above_pct: Some(50.0),
                },
                scope: scope(None, Some("session")),
            })
        );
    }

    #[test]
    fn rules_read_however_they_are_worded() {
        let cases = [
            (
                "Warn me when Claude's weekly window passes 90 percent",
                Condition::Threshold { pct: 90.0 },
                scope(Some("claude"), Some("weekly")),
            ),
            (
                "tell me when the session resets",
                Condition::Reset,
                scope(None, Some("session")),
            ),
            (
                "an hour before codex's weekly resets",
                Condition::BeforeReset {
                    minutes: 60,
                    above_pct: None,
                },
                scope(Some("codex"), Some("weekly")),
            ),
            (
                "ping me half an hour before the 5h session resets",
                Condition::BeforeReset {
                    minutes: 30,
                    above_pct: None,
                },
                scope(None, Some("session")),
            ),
            (
                "2h before reset if over 75.5%",
                Condition::BeforeReset {
                    minutes: 120,
                    above_pct: Some(75.5),
                },
                Scope::default(),
            ),
            (
                "notify me when an account needs to sign in again",
                Condition::SignIn,
                Scope::default(),
            ),
            (
                "tell me when my ChatGPT sign-in expires",
                Condition::SignIn,
                scope(Some("codex"), None),
            ),
            (
                "warn me when \u{201c}Weekly (Cowork)\u{201d} reaches 75%",
                Condition::Threshold { pct: 75.0 },
                scope(None, Some("Weekly (Cowork)")),
            ),
            (
                "warn me when the fable weekly limit hits 95%",
                Condition::Threshold { pct: 95.0 },
                scope(None, Some("fable")),
            ),
            (
                "the 5-hour session at 80%",
                Condition::Threshold { pct: 80.0 },
                scope(None, Some("session")),
            ),
            (
                "tell me if I am running out",
                Condition::RunsOut,
                Scope::default(),
            ),
        ];
        for (written, condition, scope) in cases {
            assert_eq!(rule(written), Rule { condition, scope }, "{written:?}");
        }
    }

    #[test]
    fn an_incident_rule_watches_the_status_pages_it_covers() {
        assert_eq!(
            rule("tell me when claude has an incident"),
            Rule {
                condition: Condition::Incident,
                scope: scope(Some("claude"), None),
            }
        );
        assert_eq!(rule("warn me about outages").condition, Condition::Incident);
        assert!(Rule::parse("tell me when the session has an outage").is_err());
        assert!(Rule::parse("tell me about outages and when I need to sign in").is_err());
        assert_eq!(
            rule("tell me when codex is down").to_string(),
            "when Codex's status page reports an incident"
        );

        let page = |provider: &str, indicator| StatusPage {
            provider: provider.into(),
            indicator,
            description: "Partial System Degradation".into(),
            url: "https://status.openai.com".into(),
            since: Some(at(-30)),
            checked_at: at(0),
        };
        let pages = [
            page("claude", crate::Indicator::Maintenance),
            page("codex", crate::Indicator::Minor),
        ];
        let any = rule("warn me about outages");
        let held = any.incidents(&pages);
        assert_eq!(held.len(), 1, "maintenance is no incident");
        assert_eq!(held[0].0.provider, "codex");
        assert_eq!(held[0].1.instance, Some(at(-30)), "keyed by when it began");
        assert_eq!(
            any.message(&account("codex"), &held[0].1, at(0)),
            (
                "Codex has an incident".to_owned(),
                "status.openai.com says: Partial System Degradation.".to_owned()
            )
        );
        assert!(
            rule("tell me when claude is down")
                .incidents(&pages)
                .is_empty()
        );
        assert!(
            rule("warn me at 80%").incidents(&pages).is_empty(),
            "only incident rules"
        );
    }

    #[test]
    fn what_cannot_be_read_names_the_forms_that_can() {
        let unread = Rule::parse("make it stop").expect_err("not a rule");
        assert_eq!(
            unread.to_string(),
            "QuotaDial could not read \"make it stop\" as a rule. Write it like one of these: \
             \"warn me at 80%\", \"warn me when the weekly window will run out before it resets\", \
             \"tell me 30 minutes before the session resets if it is above 50%\"."
        );
        for written in [
            "",
            "tell me before the session resets",
            "warn me at 90% before the weekly resets",
            "30 minutes",
            "warn me at 80% or when it runs out",
            "tell me when the opus window needs signing in",
        ] {
            assert!(
                Rule::parse(written)
                    .expect_err(written)
                    .to_string()
                    .contains("Write it like one of these"),
                "{written:?}"
            );
        }
    }

    #[test]
    fn two_of_a_kind_or_a_number_out_of_range_is_refused() {
        let refused = |written: &str| Rule::parse(written).expect_err(written).to_string();

        assert_eq!(
            refused("warn me at 80% and 90%"),
            "A rule can name one share used; write a rule for each."
        );
        assert_eq!(
            refused("when the session or the weekly reaches 80%"),
            "A rule can name one window; write a rule for each."
        );
        assert_eq!(
            refused("warn me at 150%"),
            "A rule's share used must be more than 0% and at most 100%."
        );
        assert_eq!(
            refused("8 days before the weekly resets"),
            "A rule's time before a reset must be at least a minute and at most a week."
        );
    }

    #[test]
    fn a_rule_says_what_it_was_read_as() {
        let read = |written: &str| rule(written).to_string();

        assert_eq!(read("warn me at 80%"), "when any window reaches 80%");
        assert_eq!(
            read("warn me when claude's session passes 80.5%"),
            "when a Claude session window reaches 80.5%"
        );
        assert_eq!(
            read("tell me when codex resets"),
            "when any Codex window resets"
        );
        assert_eq!(
            read("opus will run out"),
            "when the Opus window will run out before it resets"
        );
        assert_eq!(
            read(EXAMPLES[2]),
            "30 min before a session window resets, if it is above 50%"
        );
        assert_eq!(
            read("90 minutes before \"Weekly (Cowork)\" resets"),
            "1 h 30 min before the Weekly (Cowork) window resets"
        );
        assert_eq!(
            read("tell me when my chatgpt login expires"),
            "when a Codex account needs to sign in again"
        );
    }

    #[test]
    fn rules_keep_their_shape_in_json() {
        let rule = rule(EXAMPLES[2]);
        let value = serde_json::to_value(&rule).expect("serializes");

        assert_eq!(
            value,
            serde_json::json!({
                "kind": "before_reset",
                "minutes": 30,
                "above_pct": 50.0,
                "window": "session",
            })
        );
        assert_eq!(
            serde_json::from_value::<Rule>(value).expect("deserializes"),
            rule
        );
        assert_eq!(
            serde_json::to_value(Rule::parse("tell me when codex resets").expect("reads"))
                .expect("serializes"),
            serde_json::json!({ "kind": "reset", "provider": "codex" })
        );
    }

    fn at(minutes: i64) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2099-01-01T00:00:00Z")
            .expect("valid timestamp")
            .with_timezone(&Utc)
            + TimeDelta::minutes(minutes)
    }

    fn account(provider: &str) -> Account {
        Account {
            provider: provider.into(),
            account_id: "user-1".into(),
            display_name: "user@example.com".into(),
            plan: None,
            org: None,
            sign_in: None,
        }
    }

    fn window(kind: WindowKind, label: &str, used_pct: f64, resets_in: i64) -> Window {
        Window {
            kind,
            label: label.into(),
            used_pct,
            resets_at: Some(at(resets_in)),
            window_seconds: 18_000,
            scope_model: None,
            source: Source::Api,
            fetched_at: at(0),
            unconfirmed: false,
            pace: None,
        }
    }

    fn snapshot(provider: &str, windows: Vec<Window>) -> Snapshot {
        Snapshot {
            account: account(provider),
            windows,
            extra_usage: None,
            status: Status::Ok,
            error: None,
        }
    }

    /// The labels and instances where `written` holds in `snapshot` at minute 0, with no reset
    /// passed.
    fn holds(written: &str, snapshot: &Snapshot) -> Vec<(Option<String>, Option<DateTime<Utc>>)> {
        rule(written)
            .occurrences(snapshot, |_| None, at(0))
            .into_iter()
            .map(|occurrence| (occurrence.label, occurrence.instance))
            .collect()
    }

    #[test]
    fn a_threshold_holds_for_each_window_at_or_over_it_once_per_reset() {
        let snapshot = snapshot(
            "claude",
            vec![
                window(WindowKind::Session, "Session", 80.0, 120),
                window(WindowKind::Weekly, "Weekly", 79.9, 6000),
            ],
        );

        assert_eq!(
            holds("warn me at 80%", &snapshot),
            [(Some("Session".to_owned()), Some(at(120)))],
            "keyed by the window's reset"
        );
        assert!(holds("warn me when weekly passes 80%", &snapshot).is_empty());
        assert!(holds("warn me when codex passes 50%", &snapshot).is_empty());

        let mut unconfirmed = snapshot.clone();
        unconfirmed.windows[0].unconfirmed = true;
        assert!(
            holds("warn me at 0.5%", &unconfirmed).len() == 1,
            "only the weekly"
        );

        let mut stale = snapshot.clone();
        stale.status = Status::Stale;
        assert!(
            holds("warn me at 80%", &stale).is_empty(),
            "numbers that are not current"
        );
    }

    #[test]
    fn a_window_that_runs_out_before_its_reset_holds() {
        let mut filling = window(WindowKind::Weekly, "Weekly", 60.0, 6000);
        filling.pace = Some(Pace {
            pct_per_hour: 2.0,
            runs_out_at: Some(at(1200)),
        });
        let mut lasting = window(WindowKind::Session, "Session", 10.0, 120);
        lasting.pace = Some(Pace {
            pct_per_hour: 1.0,
            runs_out_at: None,
        });
        let snapshot = snapshot("claude", vec![filling, lasting]);

        assert_eq!(
            holds("warn me if anything will run out", &snapshot),
            [(Some("Weekly".to_owned()), Some(at(6000)))]
        );
    }

    #[test]
    fn a_time_before_a_reset_holds_from_that_time_until_the_reset() {
        let rule = rule("tell me 30 minutes before the session resets if it is above 50%");
        let session = |used_pct| {
            snapshot(
                "claude",
                vec![window(WindowKind::Session, "Session", used_pct, 120)],
            )
        };
        let holds_at = |snapshot: &Snapshot, minute| {
            !rule.occurrences(snapshot, |_| None, at(minute)).is_empty()
        };

        assert!(!holds_at(&session(60.0), 89), "31 minutes before");
        assert!(holds_at(&session(60.0), 90));
        assert!(holds_at(&session(60.0), 119));
        assert!(!holds_at(&session(60.0), 120), "it has reset");
        assert!(!holds_at(&session(40.0), 100), "not above 50%");
    }

    #[test]
    fn a_reset_holds_once_the_last_reading_s_reset_has_passed() {
        let rule = rule("tell me when a window resets");
        let after = snapshot(
            "claude",
            vec![window(WindowKind::Session, "Session", 2.0, 300)],
        );

        let passed = rule.occurrences(&after, |_| Some(at(-5)), at(0));
        assert_eq!(passed.len(), 1);
        assert_eq!(
            passed[0].instance,
            Some(at(-5)),
            "keyed by the reset that passed"
        );

        assert!(
            rule.occurrences(&after, |_| Some(at(5)), at(0)).is_empty(),
            "a reset still to come, as an expired window's moving reset is"
        );
        assert!(
            rule.occurrences(&after, |_| None, at(0)).is_empty(),
            "no reading before"
        );
    }

    #[test]
    fn a_sign_in_holds_until_the_account_works_again_keyed_by_its_last_numbers() {
        let rule = rule("tell me when an account needs to sign in again");
        let mut expired =
            Snapshot::auth_required(account("claude"), "Run `claude` to sign in again.");
        expired.windows = vec![window(WindowKind::Session, "Session", 40.0, 120)];

        let held = rule.occurrences(&expired, |_| None, at(0));

        assert_eq!(held.len(), 1);
        assert_eq!(held[0].instance, Some(at(0)), "when it last had numbers");
        assert!(
            rule.occurrences(&snapshot("claude", Vec::new()), |_| None, at(0))
                .is_empty()
        );
    }

    #[test]
    fn each_condition_says_what_happened_and_names_the_account() {
        let account = account("codex");
        let occurrence = Occurrence {
            label: Some("Weekly".into()),
            scope_model: None,
            instance: Some(at(3000)),
            used_pct: Some(91.4),
            resets_at: Some(at(3000)),
            pace: Some(Pace {
                pct_per_hour: 4.0,
                runs_out_at: Some(at(129)),
            }),
            note: None,
        };
        let message = |written: &str| rule(written).message(&account, &occurrence, at(0));

        assert_eq!(
            message("warn me at 90%"),
            (
                "Codex Weekly passed 90%".to_owned(),
                "user@example.com: 91% used, resets in 2 d 2 h.".to_owned()
            )
        );
        assert_eq!(
            message("tell me when it runs out").1,
            "user@example.com: 91% used, at 4.0%/h, runs out in 2 h 9 min, 1 d 23 h before it \
             resets."
        );
        assert_eq!(
            message("tell me when it resets").0,
            "Codex Weekly has reset"
        );
        let idle = Occurrence {
            used_pct: Some(0.0),
            resets_at: None,
            ..occurrence.clone()
        };
        assert_eq!(
            rule("tell me when it resets")
                .message(&account, &idle, at(0))
                .1,
            "user@example.com: 0% used now.",
            "a session not started again has no reset to name"
        );
        assert_eq!(
            message("an hour before it resets").0,
            "Codex Weekly resets soon"
        );
        let expired = Occurrence {
            note: Some("Run `codex` to sign in again.".into()),
            ..occurrence
        };
        assert_eq!(
            rule("when I need to log in").message(&account, &expired, at(0)),
            (
                "Codex needs you to sign in again".to_owned(),
                "user@example.com: Run `codex` to sign in again.".to_owned()
            )
        );
    }
}
