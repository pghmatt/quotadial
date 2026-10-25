//! Requests in the Codex CLI's session logs: `sessions/YYYY/MM/DD/rollout-*.jsonl` and
//! `archived_sessions/rollout-*.jsonl` in its folder.
//!
//! The CLI writes a `token_usage_record` for each model response: `payload.usage` with
//! `input_tokens` (the cached ones included), `cached_input_tokens`, `cache_write_input_tokens`
//! and `output_tokens` (reasoning included), under `payload.response_id`, which is the same in
//! every copy of the record. Its model is the one the session's latest `turn_context` names.
//!
//! Older CLIs wrote only running totals, in `token_count` events. Forked and subagent sessions
//! copy those from their parents, so adding them up counts work twice; CodexBar needs over a
//! thousand lines to undo that. A session with `token_count` events and no records is counted as
//! such and left out of the cost, which says so (D-097).

use std::borrow::Cow;

use chrono::{DateTime, Utc};
use memchr::memmem::Finder;
use quotadial_core::Tokens;
use serde::Deserialize;

use crate::Request;

/// The model of a request whose session has named none yet.
pub(crate) const NO_MODEL: &str = "unknown";

/// Reads the Codex CLI's log lines.
pub(crate) struct Reader {
    record: Finder<'static>,
    context: Finder<'static>,
    count: Finder<'static>,
}

/// What a session's log has shown so far.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Session {
    /// The model its latest `turn_context` named.
    pub(crate) model: Option<String>,
    /// Its `token_usage_record` lines.
    pub(crate) records: u64,
    /// Its `token_count` events.
    pub(crate) token_counts: u64,
}

impl Session {
    /// Whether the session was written by a CLI that records no requests.
    pub(crate) fn unrecorded(&self) -> bool {
        self.records == 0 && self.token_counts > 0
    }
}

impl Reader {
    pub(crate) fn new() -> Self {
        Self {
            record: Finder::new(b"\"token_usage_record\""),
            context: Finder::new(b"\"turn_context\""),
            count: Finder::new(b"\"token_count\""),
        }
    }

    /// The request on `line`, if it holds one, noting in `session` what the line shows of it. A
    /// line is parsed only when it has the word for one of the three kinds of line read.
    pub(crate) fn request(&self, session: &mut Session, line: &[u8]) -> Option<Request> {
        if self.record.find(line).is_none()
            && self.context.find(line).is_none()
            && self.count.find(line).is_none()
        {
            return None;
        }
        let line: Line<'_> = serde_json::from_slice(line).ok()?;
        let payload = line.payload?;
        match line.kind.as_deref()? {
            "turn_context" => {
                // An empty name clears the session's model, as the CLI means it to.
                session.model = payload
                    .model
                    .or(payload.model_name)
                    .map(|model| model.trim().to_owned())
                    .filter(|model| !model.is_empty());
                None
            }
            "event_msg" if payload.kind.as_deref() == Some("token_count") => {
                session.token_counts += 1;
                None
            }
            "token_usage_record" => {
                // Even a record that cannot be counted shows that the CLI writes them.
                session.records += 1;
                let response = payload
                    .response_id
                    .as_deref()
                    .map(str::trim)
                    .filter(|id| !id.is_empty())?;
                let tokens = payload.usage?.tokens()?;
                let at = DateTime::parse_from_rfc3339(line.timestamp.as_deref()?)
                    .ok()?
                    .with_timezone(&Utc);
                Some(Request {
                    key: Some(format!("codex:{response}")),
                    at,
                    model: session.model.clone().unwrap_or_else(|| NO_MODEL.to_owned()),
                    tokens,
                })
            }
            _ => None,
        }
    }
}

#[derive(Deserialize)]
struct Line<'a> {
    #[serde(rename = "type", borrow, default)]
    kind: Option<Cow<'a, str>>,
    #[serde(borrow, default)]
    timestamp: Option<Cow<'a, str>>,
    #[serde(borrow, default)]
    payload: Option<Payload<'a>>,
}

#[derive(Deserialize)]
struct Payload<'a> {
    #[serde(rename = "type", borrow, default)]
    kind: Option<Cow<'a, str>>,
    #[serde(borrow, default)]
    model: Option<Cow<'a, str>>,
    #[serde(borrow, default)]
    model_name: Option<Cow<'a, str>>,
    #[serde(borrow, default)]
    response_id: Option<Cow<'a, str>>,
    #[serde(default)]
    usage: Option<Usage>,
}

#[derive(Deserialize)]
struct Usage {
    input_tokens: u64,
    output_tokens: u64,
    #[serde(default)]
    cached_input_tokens: u64,
    #[serde(default)]
    cache_write_input_tokens: u64,
    #[serde(default)]
    reasoning_output_tokens: u64,
}

impl Usage {
    /// The tokens by kind, with the cache's taken out of the input. `None` when the counts
    /// contradict each other, as CodexBar treats them: more cached or reasoning tokens than input
    /// or output.
    fn tokens(&self) -> Option<Tokens> {
        if self.cached_input_tokens > self.input_tokens
            || self.reasoning_output_tokens > self.output_tokens
        {
            return None;
        }
        let cache_read = self.cached_input_tokens;
        let cache_write = self
            .cache_write_input_tokens
            .min(self.input_tokens - cache_read);
        Some(Tokens {
            input: self.input_tokens - cache_read - cache_write,
            output: self.output_tokens,
            cache_read,
            cache_write,
            cache_write_1h: 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Lines as the Codex CLI writes them, with made-up ids and values.
    const CONTEXT: &str = r#"{"timestamp":"2026-10-23T12:00:00.000Z","ordinal":3,"type":"turn_context","payload":{"turn_id":"turn-1","cwd":"/work","approval_policy":"never","model":"gpt-6-astra","effort":"high","summary":"auto"}}"#;
    const RECORD: &str = r#"{"timestamp":"2026-10-23T12:00:05.000Z","ordinal":9,"type":"token_usage_record","payload":{"thread_id":"thread-1","turn_id":"turn-1","session_id":"session-1","response_id":"resp_01","usage":{"input_tokens":300000,"cached_input_tokens":250000,"cache_write_input_tokens":0,"output_tokens":1200,"reasoning_output_tokens":800,"total_tokens":301200},"turn_token_usage":{"input_tokens":300000,"output_tokens":1200},"thread_token_usage":{"input_tokens":900000,"output_tokens":5000}}}"#;
    const COUNT: &str = r#"{"timestamp":"2026-10-23T12:00:05.000Z","ordinal":10,"type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{"input_tokens":900000,"cached_input_tokens":700000,"output_tokens":5000},"last_token_usage":{"input_tokens":300000,"cached_input_tokens":250000,"output_tokens":1200}},"rate_limits":{"primary":{"used_percent":12.0,"window_minutes":300}}}}"#;

    #[test]
    fn a_record_is_a_request_of_the_model_the_turn_named() {
        let reader = Reader::new();
        let mut session = Session::default();

        assert_eq!(reader.request(&mut session, CONTEXT.as_bytes()), None);
        let request = reader
            .request(&mut session, RECORD.as_bytes())
            .expect("a request");

        assert_eq!(request.key.as_deref(), Some("codex:resp_01"));
        assert_eq!(request.model, "gpt-6-astra");
        assert_eq!(
            request.at,
            DateTime::parse_from_rfc3339("2026-10-23T12:00:05Z").expect("valid")
        );
        assert_eq!(
            request.tokens,
            Tokens {
                input: 50_000,
                output: 1200,
                cache_read: 250_000,
                cache_write: 0,
                cache_write_1h: 0,
            },
            "the cached tokens come out of the input; reasoning is in the output"
        );
        assert_eq!(request.tokens.context(), 300_000);
        assert_eq!(
            session,
            Session {
                model: Some("gpt-6-astra".to_owned()),
                records: 1,
                token_counts: 0,
            }
        );
    }

    #[test]
    fn cache_writes_come_out_of_the_input_too() {
        let usage = Usage {
            input_tokens: 1000,
            output_tokens: 10,
            cached_input_tokens: 600,
            cache_write_input_tokens: 300,
            reasoning_output_tokens: 0,
        };
        assert_eq!(
            usage.tokens(),
            Some(Tokens {
                input: 100,
                output: 10,
                cache_read: 600,
                cache_write: 300,
                cache_write_1h: 0,
            })
        );
        let more_written_than_left = Usage {
            cache_write_input_tokens: 900,
            ..usage
        };
        assert_eq!(
            more_written_than_left
                .tokens()
                .map(|tokens| tokens.cache_write),
            Some(400)
        );
        let more_cached_than_read = Usage {
            cached_input_tokens: 1001,
            ..usage
        };
        assert_eq!(more_cached_than_read.tokens(), None);
    }

    #[test]
    fn token_counts_are_counted_not_added_up() {
        let reader = Reader::new();
        let mut session = Session::default();

        assert_eq!(reader.request(&mut session, COUNT.as_bytes()), None);
        assert_eq!(reader.request(&mut session, COUNT.as_bytes()), None);

        assert_eq!(session.token_counts, 2);
        assert!(session.unrecorded());
        assert!(reader.request(&mut session, RECORD.as_bytes()).is_some());
        assert!(!session.unrecorded());
    }

    #[test]
    fn a_record_before_any_turn_names_no_model() {
        let reader = Reader::new();
        let mut session = Session::default();

        let request = reader
            .request(&mut session, RECORD.as_bytes())
            .expect("a request");
        assert_eq!(request.model, NO_MODEL);

        let cleared = CONTEXT.replace(r#""model":"gpt-6-astra""#, r#""model":"""#);
        reader.request(&mut session, CONTEXT.as_bytes());
        reader.request(&mut session, cleared.as_bytes());
        assert_eq!(session.model, None, "an empty name clears it");
    }

    #[test]
    fn other_lines_and_broken_records_are_not_requests() {
        let reader = Reader::new();
        let mut session = Session::default();
        let message = r#"{"timestamp":"2026-10-23T12:00:01.000Z","type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"why is \"token_usage_record\" missing?"}]}}"#;
        let no_id = RECORD.replace(r#""response_id":"resp_01","#, "");
        let no_usage = RECORD.replace(r#""input_tokens":300000,"cached"#, r#""cached"#);

        for text in [
            message,
            no_id.as_str(),
            no_usage.as_str(),
            "{\"turn_context\"",
        ] {
            assert_eq!(
                reader.request(&mut session, text.as_bytes()),
                None,
                "{text}"
            );
        }
        assert_eq!(
            session,
            Session {
                records: 1,
                ..Session::default()
            },
            "a record without an id still shows the CLI writes records"
        );
    }

    proptest::proptest! {
        #[test]
        fn no_line_in_a_log_makes_the_reader_panic(
            lines in proptest::collection::vec(quotadial_core::testing::line(), 0..6),
            input in proptest::prelude::any::<u64>(),
            output in proptest::prelude::any::<u64>(),
            cached in proptest::prelude::any::<u64>(),
            written in proptest::prelude::any::<u64>(),
            reasoning in proptest::prelude::any::<u64>(),
            model in ".{0,20}",
        ) {
            let reader = Reader::new();
            let mut session = Session::default();
            for line in &lines {
                let _ = reader.request(&mut session, line);
            }
            let context = serde_json::json!({
                "timestamp": "2026-10-25T01:12:00Z",
                "type": "turn_context",
                "payload": { "model": model },
            });
            let record = serde_json::json!({
                "timestamp": "2026-10-25T01:12:01Z",
                "type": "event_msg",
                "payload": {
                    "type": "token_usage_record",
                    "response_id": "r1",
                    "usage": {
                        "input_tokens": input,
                        "output_tokens": output,
                        "cached_input_tokens": cached,
                        "cache_write_input_tokens": written,
                        "reasoning_output_tokens": reasoning,
                    },
                },
            });
            let _ = reader.request(&mut session, context.to_string().as_bytes());
            let _ = reader.request(&mut session, record.to_string().as_bytes());
        }
    }
}
