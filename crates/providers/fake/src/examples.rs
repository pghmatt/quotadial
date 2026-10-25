//! The example accounts: what each window reads when the provider is made, how fast it fills, and
//! when it resets. They come from `fixtures/fake/example.json`, built in, or from a JSON file of the
//! same form that `QUOTADIAL_FAKE_ACCOUNTS` names, so a recording can show other numbers.
//!
//! A file is checked whole before anything is shown, and one that cannot be shown is refused with
//! one sentence naming what is wrong, so a picture is never made from half a file. A plan or a label
//! may not hold an address, so no example can look like someone's account.

use std::fs::File;
use std::io::Read;
use std::path::Path;

use chrono::{DateTime, TimeDelta, Utc};
use quotadial_core::{Source, Window, WindowKind, outside_text};
use serde::Deserialize;

/// The accounts built in: one like a Claude plan, one like a ChatGPT plan.
pub(crate) const BUILT_IN: &str = include_str!("../../../../fixtures/fake/example.json");

/// The most accounts a file may hold, more than any picture needs.
const MAX_ACCOUNTS: usize = 12;
/// The most windows an account may have.
const MAX_WINDOWS: usize = 8;
/// The longest plan, label or model name shown.
const MAX_NAME_CHARS: usize = 64;
/// The shortest window: a minute.
const MIN_WINDOW_SECONDS: u64 = 60;
/// The longest window: a year and a day.
const MAX_WINDOW_SECONDS: u64 = 366 * 24 * 60 * 60;
/// The fastest a window may fill, in points an hour: a whole window in an hour.
const MAX_PCT_PER_HOUR: f64 = 100.0;
/// The largest file read. The built-in one is under 2 KB.
const MAX_FILE_BYTES: u64 = 64 * 1024;

/// The example accounts, checked.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Examples {
    pub(crate) accounts: Vec<ExampleAccount>,
}

/// One example account: its plan and its windows.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ExampleAccount {
    pub(crate) plan: String,
    pub(crate) windows: Vec<ExampleWindow>,
}

/// One example window, as it reads when the provider is made.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ExampleWindow {
    kind: WindowKind,
    label: String,
    scope_model: Option<String>,
    /// Its length in seconds, from [`MIN_WINDOW_SECONDS`] to [`MAX_WINDOW_SECONDS`].
    window_seconds: i64,
    /// Its share used when the provider is made, from 0 to 100.
    used_pct: f64,
    /// How many seconds after the provider is made it first resets: at least 1, at most its length.
    resets_in_seconds: i64,
    /// How fast it fills, in points an hour, from 0 to [`MAX_PCT_PER_HOUR`].
    pct_per_hour: f64,
}

/// The file's form.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FileForm {
    accounts: Vec<AccountForm>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AccountForm {
    plan: String,
    windows: Vec<WindowForm>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WindowForm {
    kind: WindowKind,
    label: String,
    #[serde(default)]
    scope_model: Option<String>,
    window_seconds: u64,
    used_pct: f64,
    resets_in_seconds: u64,
    #[serde(default)]
    pct_per_hour: f64,
}

/// The examples in `text`, the file's JSON, or the first thing wrong with it.
pub(crate) fn parse(text: &str) -> Result<Examples, String> {
    let form: FileForm = serde_json::from_str(text)
        .map_err(|error| format!("it is not the example accounts' JSON ({error})"))?;
    if form.accounts.len() > MAX_ACCOUNTS {
        return Err(format!(
            "it lists {} accounts, and at most {MAX_ACCOUNTS} are shown",
            form.accounts.len()
        ));
    }
    let accounts = form
        .accounts
        .into_iter()
        .enumerate()
        .map(|(index, account)| checked_account(index + 1, account))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Examples { accounts })
}

/// The examples in the file at `path`, or why they cannot be used, naming the file and the fix.
pub(crate) fn read(path: &Path) -> Result<Examples, String> {
    let read = |path: &Path| -> Result<String, String> {
        let file = File::open(path).map_err(|error| format!("it could not be read ({error})"))?;
        let size = file
            .metadata()
            .map_err(|error| format!("it could not be read ({error})"))?
            .len();
        if size > MAX_FILE_BYTES {
            return Err(format!(
                "it is over {} KB, more than any example needs",
                MAX_FILE_BYTES / 1024
            ));
        }
        let mut text = String::new();
        // Taken no further than the limit, in case the file grows while it is read.
        file.take(MAX_FILE_BYTES)
            .read_to_string(&mut text)
            .map_err(|error| format!("it could not be read ({error})"))?;
        Ok(text)
    };
    read(path).and_then(|text| parse(&text)).map_err(|reason| {
        format!(
            "the example accounts in {} cannot be used: {reason}; fix the file, or unset {} to \
                 show the built-in examples",
            path.display(),
            crate::ACCOUNTS_VARIABLE
        )
    })
}

fn checked_account(number: usize, account: AccountForm) -> Result<ExampleAccount, String> {
    let at = format!("account {number}");
    let plan = name(&account.plan, &format!("{at}: its plan"))?;
    if account.windows.is_empty() || account.windows.len() > MAX_WINDOWS {
        return Err(format!(
            "{at}: it has {} windows, and an example has from 1 to {MAX_WINDOWS}",
            account.windows.len()
        ));
    }
    let windows = account
        .windows
        .into_iter()
        .enumerate()
        .map(|(index, window)| checked_window(&format!("{at}, window {}", index + 1), window))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ExampleAccount { plan, windows })
}

fn checked_window(at: &str, window: WindowForm) -> Result<ExampleWindow, String> {
    let label = name(&window.label, &format!("{at}: its label"))?;
    let scope_model = window
        .scope_model
        .as_deref()
        .map(|model| name(model, &format!("{at}: its scope_model")))
        .transpose()?;
    if !(MIN_WINDOW_SECONDS..=MAX_WINDOW_SECONDS).contains(&window.window_seconds) {
        return Err(format!(
            "{at}: window_seconds must be from {MIN_WINDOW_SECONDS} to {MAX_WINDOW_SECONDS}"
        ));
    }
    if !(1..=window.window_seconds).contains(&window.resets_in_seconds) {
        return Err(format!(
            "{at}: resets_in_seconds must be from 1 to its window_seconds"
        ));
    }
    if !(0.0..=100.0).contains(&window.used_pct) {
        return Err(format!("{at}: used_pct must be from 0 to 100"));
    }
    if !(0.0..=MAX_PCT_PER_HOUR).contains(&window.pct_per_hour) {
        return Err(format!(
            "{at}: pct_per_hour must be from 0 to {MAX_PCT_PER_HOUR}"
        ));
    }
    // Both fit, being at most a year and a day.
    let seconds = |value: u64| i64::try_from(value).unwrap_or(i64::MAX);
    Ok(ExampleWindow {
        kind: window.kind,
        label,
        scope_model,
        window_seconds: seconds(window.window_seconds),
        used_pct: window.used_pct,
        resets_in_seconds: seconds(window.resets_in_seconds),
        pct_per_hour: window.pct_per_hour,
    })
}

/// `text` cleaned as any text from outside is, when it is a name an example may show: something
/// is left once it is cleaned, and it holds no address.
fn name(text: &str, what: &str) -> Result<String, String> {
    let cleaned = outside_text(text, MAX_NAME_CHARS).ok_or_else(|| format!("{what} is empty"))?;
    if cleaned.contains('@') {
        return Err(format!(
            "{what} holds an @, and an example never shows an address"
        ));
    }
    Ok(cleaned)
}

impl ExampleWindow {
    /// The window read at `now`, for a provider that started at `start`. It fills at its pace from
    /// its share at `start`, never past 100%, until its first reset; then each of its windows
    /// starts at 0% and fills at the same pace. Before `start`, as on a clock set back, it reads as
    /// at `start`; it is always said to be read at `now`.
    pub(crate) fn at(&self, start: DateTime<Utc>, now: DateTime<Utc>) -> Window {
        let read_at = now;
        let now = now.max(start);
        let seconds = |seconds: i64| TimeDelta::try_seconds(seconds).unwrap_or_default();
        let first_reset = start
            .checked_add_signed(seconds(self.resets_in_seconds))
            .unwrap_or(start);
        let (began, used_then, resets_at) = if now < first_reset {
            (start, self.used_pct, first_reset)
        } else {
            // The whole windows since the first reset: a window is at least a minute long.
            let windows = (now - first_reset).num_seconds() / self.window_seconds.max(1);
            let began = windows
                .checked_mul(self.window_seconds)
                .and_then(TimeDelta::try_seconds)
                .and_then(|since| first_reset.checked_add_signed(since))
                .unwrap_or(now);
            let resets_at = began
                .checked_add_signed(seconds(self.window_seconds))
                .unwrap_or(now);
            (began, 0.0, resets_at)
        };
        let hours = (now - began).num_milliseconds() as f64 / 3_600_000.0;
        let used_pct = (used_then + self.pct_per_hour * hours).clamp(0.0, 100.0);
        Window {
            kind: self.kind,
            label: self.label.clone(),
            used_pct: (used_pct * 10.0).round() / 10.0,
            resets_at: Some(resets_at),
            window_seconds: self.window_seconds.unsigned_abs(),
            scope_model: self.scope_model.clone(),
            source: Source::Example,
            fetched_at: read_at,
            unconfirmed: false,
            pace: None,
        }
    }
}

/// The first whole minute at or after `at`, when examples made at `at` start: real limits reset on
/// whole minutes, and the first reading then shows each reset as the file gives it, rather than a
/// minute less.
pub(crate) fn next_minute(at: DateTime<Utc>) -> DateTime<Utc> {
    let seconds = at.timestamp();
    let whole = seconds.rem_euclid(60) == 0 && at.timestamp_subsec_nanos() == 0;
    let minute = if whole {
        seconds
    } else {
        (seconds.div_euclid(60) + 1).saturating_mul(60)
    };
    DateTime::from_timestamp(minute, 0).unwrap_or(at)
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use serde_json::json;

    use super::*;

    fn at(rfc3339: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(rfc3339)
            .expect("a time")
            .with_timezone(&Utc)
    }

    /// One account, named Max, with one window, written as given.
    fn file(window: &str) -> String {
        format!(r#"{{ "accounts": [{{ "plan": "Max", "windows": [{window}] }}] }}"#)
    }

    const SESSION: &str = r#"{ "kind": "session", "label": "Session", "window_seconds": 18000,
        "used_pct": 38, "resets_in_seconds": 7800, "pct_per_hour": 8.5 }"#;

    fn session() -> ExampleWindow {
        parse(&file(SESSION)).expect("parses").accounts[0].windows[0].clone()
    }

    #[test]
    fn the_built_in_examples_are_a_claude_plan_and_a_chatgpt_plan() {
        let examples = parse(BUILT_IN).expect("the built-in examples parse");

        let plans: Vec<&str> = examples
            .accounts
            .iter()
            .map(|account| account.plan.as_str())
            .collect();
        assert_eq!(plans, ["Claude Max 5x", "ChatGPT Pro"]);
        let labels = |account: &ExampleAccount| -> Vec<String> {
            account
                .windows
                .iter()
                .map(|window| window.label.clone())
                .collect()
        };
        assert_eq!(
            labels(&examples.accounts[0]),
            ["Weekly", "Weekly (Opus)", "Session"]
        );
        assert_eq!(labels(&examples.accounts[1]), ["Weekly", "Session"]);
        for window in examples
            .accounts
            .iter()
            .flat_map(|account| &account.windows)
        {
            assert!(
                window.pct_per_hour > 0.0,
                "{} fills, so the daemon finds it a pace",
                window.label
            );
        }
    }

    #[test]
    fn a_window_reads_as_given_when_the_provider_is_made() {
        let start = at("2099-01-01T00:00:00Z");

        let window = session().at(start, start);

        assert_eq!(window.kind, WindowKind::Session);
        assert_eq!(window.label, "Session");
        assert_eq!(window.used_pct, 38.0);
        assert_eq!(window.resets_at, Some(at("2099-01-01T02:10:00Z")));
        assert_eq!(window.window_seconds, 18_000);
        assert_eq!(window.scope_model, None);
        assert_eq!(window.source, Source::Example);
        assert_eq!(window.fetched_at, start);
        assert!(!window.unconfirmed);
        assert_eq!(window.pace, None);
    }

    #[test]
    fn a_window_fills_at_its_pace_and_starts_again_at_each_reset() {
        let start = at("2099-01-01T00:00:00Z");
        let window = session();

        let later = window.at(start, at("2099-01-01T02:00:00Z"));
        assert_eq!(later.used_pct, 55.0, "38% and two hours at 8.5%/h");
        assert_eq!(later.resets_at, Some(at("2099-01-01T02:10:00Z")));

        let reset = window.at(start, at("2099-01-01T02:10:00Z"));
        assert_eq!(reset.used_pct, 0.0, "a new window begins at the reset");
        assert_eq!(reset.resets_at, Some(at("2099-01-01T07:10:00Z")));

        let next = window.at(start, at("2099-01-01T08:10:00Z"));
        assert_eq!(next.used_pct, 8.5, "an hour into the third window");
        assert_eq!(next.resets_at, Some(at("2099-01-01T12:10:00Z")));
        assert_eq!(next.fetched_at, at("2099-01-01T08:10:00Z"));
    }

    #[test]
    fn a_window_never_reads_past_full_nor_before_the_provider_was_made() {
        let start = at("2099-01-01T00:00:00Z");
        let weekly = parse(&file(
            r#"{ "kind": "weekly", "label": "Weekly", "window_seconds": 604800, "used_pct": 91,
                 "resets_in_seconds": 140400, "pct_per_hour": 2 }"#,
        ))
        .expect("parses")
        .accounts[0]
            .windows[0]
            .clone();

        assert_eq!(weekly.at(start, at("2099-01-01T10:00:00Z")).used_pct, 100.0);
        let early = weekly.at(start, at("2098-12-31T00:00:00Z"));
        assert_eq!(
            early.used_pct, 91.0,
            "a clock set back reads as at the start"
        );
        assert_eq!(early.resets_at, Some(at("2099-01-02T15:00:00Z")));
        assert_eq!(
            early.fetched_at,
            at("2098-12-31T00:00:00Z"),
            "and says when the clock said it was read"
        );
    }

    #[test]
    fn examples_start_at_the_next_whole_minute() {
        assert_eq!(
            next_minute(at("2099-01-01T00:00:30.25Z")),
            at("2099-01-01T00:01:00Z")
        );
        assert_eq!(
            next_minute(at("2099-01-01T00:01:00Z")),
            at("2099-01-01T00:01:00Z")
        );
        assert_eq!(
            next_minute(at("2099-01-01T00:01:00.000001Z")),
            at("2099-01-01T00:02:00Z")
        );
        assert_eq!(
            next_minute(at("1969-12-31T23:59:30Z")),
            at("1970-01-01T00:00:00Z"),
            "before 1970 too"
        );
    }

    #[test]
    fn a_file_that_cannot_be_shown_is_refused_naming_what_is_wrong() {
        let refused = |window: &str| parse(&file(window)).expect_err("refused");

        assert_eq!(
            refused(&SESSION.replace("38", "101")),
            "account 1, window 1: used_pct must be from 0 to 100"
        );
        assert_eq!(
            refused(&SESSION.replace("7800", "18001")),
            "account 1, window 1: resets_in_seconds must be from 1 to its window_seconds"
        );
        assert_eq!(
            refused(&SESSION.replace("18000", "59")),
            "account 1, window 1: window_seconds must be from 60 to 31622400"
        );
        assert_eq!(
            refused(&SESSION.replace("8.5", "-1")),
            "account 1, window 1: pct_per_hour must be from 0 to 100"
        );
        assert_eq!(
            refused(&SESSION.replace(r#""Session""#, r#""me@example.com""#)),
            "account 1, window 1: its label holds an @, and an example never shows an address"
        );
        assert_eq!(
            refused(&SESSION.replace(r#""Session""#, r#""  ""#)),
            "account 1, window 1: its label is empty"
        );
        assert!(
            refused(&SESSION.replace(r#""used_pct""#, r#""email": "x", "used_pct""#))
                .starts_with("it is not the example accounts' JSON (unknown field `email`"),
            "a field the form does not have is refused, not passed over"
        );
        assert_eq!(
            parse(r#"{ "accounts": [{ "plan": "a@b.com", "windows": [] }] }"#)
                .expect_err("refused"),
            "account 1: its plan holds an @, and an example never shows an address"
        );
        assert_eq!(
            parse(r#"{ "accounts": [{ "plan": "Max", "windows": [] }] }"#).expect_err("refused"),
            "account 1: it has 0 windows, and an example has from 1 to 8"
        );
    }

    #[test]
    fn no_accounts_is_an_empty_picture_and_too_many_are_refused() {
        assert_eq!(
            parse(r#"{ "accounts": [] }"#).expect("parses"),
            Examples {
                accounts: Vec::new()
            }
        );
        let account = format!(r#"{{ "plan": "Max", "windows": [{SESSION}] }}"#);
        let thirteen = format!(r#"{{ "accounts": [{}] }}"#, vec![account; 13].join(","));
        assert_eq!(
            parse(&thirteen).expect_err("refused"),
            "it lists 13 accounts, and at most 12 are shown"
        );
    }

    #[test]
    fn a_file_that_cannot_be_used_names_itself_and_the_fix() {
        let temp = tempfile::tempdir().expect("temp dir");
        let path = temp.path().join("examples.json");

        let missing = read(&path).expect_err("no file");
        assert!(
            missing.starts_with(&format!(
                "the example accounts in {} cannot be used: it could not be read (",
                path.display()
            )),
            "{missing}"
        );
        assert!(
            missing.ends_with(
                "; fix the file, or unset QUOTADIAL_FAKE_ACCOUNTS to show the built-in examples"
            ),
            "{missing}"
        );

        std::fs::write(&path, vec![b' '; 64 * 1024 + 1]).expect("writes");
        let large = read(&path).expect_err("too large");
        assert!(large.contains(": it is over 64 KB, more than"), "{large}");

        std::fs::write(&path, BUILT_IN).expect("writes");
        assert_eq!(
            read(&path),
            parse(BUILT_IN),
            "a file reads as the same text built in"
        );
    }

    /// A window in the file's form: mostly with values it may hold, so that most files can be read
    /// and the windows in them are read too, and sometimes with anything at all.
    fn window_form() -> impl Strategy<Value = serde_json::Value> {
        let length_and_reset = prop_oneof![
            9 => (60u64..=31_622_400).prop_flat_map(|length| (Just(length), 1..=length)),
            1 => (any::<u64>(), any::<u64>()),
        ];
        let share = || prop_oneof![9 => 0.0..=100.0, 1 => any::<f64>()];
        (
            prop::sample::select(&["session", "weekly", "model_weekly", "credits", "yearly"][..]),
            prop_oneof![Just("Weekly".to_owned()), "[A-Za-z ()]{1,20}", ".{0,20}"],
            proptest::option::of(prop_oneof!["[a-z]{1,10}", ".{0,20}"]),
            length_and_reset,
            share(),
            share(),
        )
            .prop_map(|(kind, label, model, (length, resets_in), used, pace)| {
                json!({
                    "kind": kind, "label": label, "scope_model": model, "window_seconds": length,
                    "used_pct": used, "resets_in_seconds": resets_in, "pct_per_hour": pace,
                })
            })
    }

    /// Anything a file could hold: any JSON, any text, or the file's form with any values in it.
    fn any_file() -> impl Strategy<Value = String> {
        prop_oneof![
            quotadial_core::testing::json().prop_map(|value| value.to_string()),
            quotadial_core::testing::json().prop_map(|value| file(&value.to_string())),
            prop::collection::vec(window_form(), 1..4).prop_map(|windows| {
                json!({ "accounts": [{ "plan": "Max", "windows": windows }] }).to_string()
            }),
            ".{0,200}",
        ]
    }

    proptest! {
        #[test]
        fn no_file_makes_the_reader_panic(body in any_file(), minutes in 0i64..10_000_000) {
            // A file can be refused for what it holds; nothing in it makes reading it panic.
            if let Ok(examples) = parse(&body) {
                let start = at("2026-10-25T01:12:00Z");
                let now = start + TimeDelta::minutes(minutes);
                for window in examples.accounts.iter().flat_map(|account| &account.windows) {
                    let read = window.at(start, now);
                    prop_assert!((0.0..=100.0).contains(&read.used_pct));
                    prop_assert!(read.resets_at.is_some_and(|reset| reset > now));
                    prop_assert_eq!(read.fetched_at, now);
                    prop_assert_eq!(read.source, Source::Example);
                }
            }
        }

        #[test]
        fn the_next_minute_is_whole_and_never_earlier(seconds in -10_000_000_000i64..10_000_000_000, nanos in 0u32..1_000_000_000) {
            let Some(time) = DateTime::from_timestamp(seconds, nanos) else {
                return Ok(());
            };
            let minute = next_minute(time);
            prop_assert!(minute >= time);
            prop_assert!(minute - time < TimeDelta::minutes(1));
            prop_assert_eq!(minute.timestamp().rem_euclid(60), 0);
            prop_assert_eq!(minute.timestamp_subsec_nanos(), 0);
        }
    }
}
