//! Requests in Claude Code's session logs: `projects/<project>/<session>.jsonl` in each config
//! folder, and the subagents' `<session>/subagents/*.jsonl` beside them. Ported from CodexBar's
//! `CostUsageScanner+Claude.swift`.
//!
//! A request is an `assistant` line with `message.usage` and `message.model`. A response streamed
//! in parts is written once per part, each with the usage so far, under one `message.id` and
//! `requestId`, and a resumed session copies earlier lines into its new file, so that pair names
//! the request wherever it appears (`sessionId` stands in for a proxy's missing `requestId`). Only
//! these fields are read: the message's content is skipped unread.

use std::borrow::Cow;

use chrono::{DateTime, Utc};
use memchr::memmem::Finder;
use quotadial_core::Tokens;
use serde::Deserialize;

use crate::Request;

/// Reads Claude Code's log lines.
pub(crate) struct Reader {
    usage: Finder<'static>,
    assistant: Finder<'static>,
}

impl Reader {
    pub(crate) fn new() -> Self {
        Self {
            usage: Finder::new(b"\"usage\""),
            assistant: Finder::new(b"\"assistant\""),
        }
    }

    /// The request on `line`, if it holds one. A line is parsed only when it has both words a
    /// request's line has.
    pub(crate) fn request(&self, line: &[u8]) -> Option<Request> {
        if self.usage.find(line).is_none() || self.assistant.find(line).is_none() {
            return None;
        }
        let line: Line<'_> = serde_json::from_slice(line).ok()?;
        if line.kind.as_deref() != Some("assistant") {
            return None;
        }
        let message = line.message?;
        let usage = message.usage?;
        let model = message.model?;
        let at = DateTime::parse_from_rfc3339(line.timestamp.as_deref()?)
            .ok()?
            .with_timezone(&Utc);
        let cache_write = usage.cache_creation_input_tokens.unwrap_or(0);
        let tokens = Tokens {
            input: usage.input_tokens.unwrap_or(0),
            output: usage.output_tokens.unwrap_or(0),
            cache_read: usage.cache_read_input_tokens.unwrap_or(0),
            cache_write,
            cache_write_1h: usage
                .cache_creation
                .and_then(|creation| creation.ephemeral_1h_input_tokens)
                .unwrap_or(0)
                .min(cache_write),
        };
        // A proxy may log its own estimate as the response starts: no stop reason yet, input but
        // no output, and nothing about the cache. The real usage follows, or nothing does.
        let estimate = message.stop_reason == Field::Null
            && tokens.input > 0
            && tokens.output == 0
            && usage.cache_read_input_tokens.is_none()
            && usage.cache_creation_input_tokens.is_none();
        if tokens.total() == 0 || estimate {
            return None;
        }
        let named = |text: &Option<Cow<'_, str>>| {
            text.as_deref()
                .map(str::trim)
                .filter(|text| !text.is_empty())
                .map(str::to_owned)
        };
        let key = match (
            named(&message.id),
            named(&line.request_id),
            named(&line.session_id),
        ) {
            (Some(id), Some(request), _) => Some(format!("claude:{id}:{request}")),
            (Some(id), None, Some(session)) => Some(format!("claude:{session}:{id}")),
            _ => None,
        };
        Some(Request {
            key,
            at,
            model: model.trim().to_owned(),
            tokens,
        })
    }
}

#[derive(Deserialize)]
struct Line<'a> {
    #[serde(rename = "type", borrow, default)]
    kind: Option<Cow<'a, str>>,
    #[serde(borrow, default)]
    timestamp: Option<Cow<'a, str>>,
    #[serde(rename = "requestId", borrow, default)]
    request_id: Option<Cow<'a, str>>,
    #[serde(rename = "sessionId", borrow, default)]
    session_id: Option<Cow<'a, str>>,
    #[serde(borrow, default)]
    message: Option<Message<'a>>,
}

#[derive(Deserialize)]
struct Message<'a> {
    #[serde(borrow, default)]
    id: Option<Cow<'a, str>>,
    #[serde(borrow, default)]
    model: Option<Cow<'a, str>>,
    #[serde(default)]
    stop_reason: Field,
    #[serde(default)]
    usage: Option<Usage>,
}

#[derive(Deserialize)]
struct Usage {
    #[serde(default)]
    input_tokens: Option<u64>,
    #[serde(default)]
    output_tokens: Option<u64>,
    #[serde(default)]
    cache_creation_input_tokens: Option<u64>,
    #[serde(default)]
    cache_read_input_tokens: Option<u64>,
    #[serde(default)]
    cache_creation: Option<CacheCreation>,
}

#[derive(Deserialize)]
struct CacheCreation {
    #[serde(default)]
    ephemeral_1h_input_tokens: Option<u64>,
}

/// Whether a field was left out, written as null, or set.
#[derive(Debug, Default, PartialEq, Eq)]
enum Field {
    #[default]
    Missing,
    Null,
    Set,
}

impl<'de> Deserialize<'de> for Field {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(
            match Option::<serde::de::IgnoredAny>::deserialize(deserializer)? {
                Some(_) => Self::Set,
                None => Self::Null,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(text: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(text)
            .expect("valid timestamp")
            .with_timezone(&Utc)
    }

    /// An assistant line as Claude Code writes it, with made-up ids and content.
    fn line(usage: &str, stop_reason: &str) -> String {
        format!(
            r#"{{"parentUuid":"p-1","isSidechain":false,"sessionId":"session-1","message":{{"id":"msg_01","type":"message","role":"assistant","model":"claude-opus-5-5","content":[{{"type":"text","text":"Done."}}],"stop_reason":{stop_reason},"usage":{usage}}},"requestId":"req_01","type":"assistant","uuid":"u-1","timestamp":"2026-10-23T12:00:00.000Z"}}"#
        )
    }

    #[test]
    fn a_request_line_gives_its_tokens_model_time_and_key() {
        let reader = Reader::new();
        let usage = r#"{"input_tokens":10,"cache_creation_input_tokens":3000,"cache_read_input_tokens":50000,"output_tokens":700,"cache_creation":{"ephemeral_5m_input_tokens":1000,"ephemeral_1h_input_tokens":2000}}"#;

        let request = reader
            .request(line(usage, r#""end_turn""#).as_bytes())
            .expect("a request");

        assert_eq!(request.key.as_deref(), Some("claude:msg_01:req_01"));
        assert_eq!(request.at, at("2026-10-23T12:00:00Z"));
        assert_eq!(request.model, "claude-opus-5-5");
        assert_eq!(
            request.tokens,
            Tokens {
                input: 10,
                output: 700,
                cache_read: 50_000,
                cache_write: 3000,
                cache_write_1h: 2000,
            }
        );
    }

    #[test]
    fn a_proxy_without_a_request_id_is_named_by_its_session() {
        let reader = Reader::new();
        let text = line(r#"{"input_tokens":10,"output_tokens":5}"#, "null")
            .replace(r#""requestId":"req_01","#, "");

        let request = reader.request(text.as_bytes()).expect("a request");

        assert_eq!(request.key.as_deref(), Some("claude:session-1:msg_01"));
        let unnamed = text.replace(r#""sessionId":"session-1","#, "");
        assert_eq!(
            reader.request(unnamed.as_bytes()).expect("a request").key,
            None,
            "the scan names it by where it was found"
        );
    }

    #[test]
    fn lines_without_usage_to_count_are_not_requests() {
        let reader = Reader::new();
        let user = r#"{"type":"user","message":{"role":"user","content":"what is \"usage\" for the \"assistant\"?"},"timestamp":"2026-10-23T12:00:00.000Z"}"#;
        let zero = line(
            r#"{"input_tokens":0,"output_tokens":0,"cache_read_input_tokens":0}"#,
            r#""end_turn""#,
        )
        .replace("claude-opus-5-5", "<synthetic>");
        let estimate = line(r#"{"input_tokens":5000,"output_tokens":0}"#, "null");
        let no_time = line(r#"{"input_tokens":10,"output_tokens":5}"#, "null")
            .replace(r#","timestamp":"2026-10-23T12:00:00.000Z""#, "");

        for text in [
            user,
            zero.as_str(),
            estimate.as_str(),
            no_time.as_str(),
            "not json \"usage\" \"assistant\"",
        ] {
            assert_eq!(reader.request(text.as_bytes()), None, "{text}");
        }
        assert!(
            reader
                .request(
                    line(
                        r#"{"input_tokens":5000,"output_tokens":0,"cache_read_input_tokens":0}"#,
                        "null"
                    )
                    .as_bytes()
                )
                .is_some(),
            "cache counts make it the API's own usage"
        );
        assert!(
            reader
                .request(
                    line(
                        r#"{"input_tokens":5000,"output_tokens":0}"#,
                        r#""end_turn""#
                    )
                    .as_bytes()
                )
                .is_some(),
            "a finished response is never an estimate"
        );
    }

    /// A token count, often at the extremes a damaged log could hold.
    fn count() -> impl proptest::strategy::Strategy<Value = serde_json::Value> {
        use proptest::prelude::*;
        prop_oneof![
            any::<u64>().prop_map(serde_json::Value::from),
            Just(serde_json::Value::from(u64::MAX)),
            quotadial_core::testing::json(),
        ]
    }

    proptest::proptest! {
        #[test]
        fn no_line_in_a_log_makes_the_reader_panic(
            line in quotadial_core::testing::line(),
            input in count(),
            output in count(),
            cache_read in count(),
            cache_write in count(),
            hour in count(),
            id in ".{0,20}",
        ) {
            let reader = Reader::new();
            let _ = reader.request(&line);
            let shaped = serde_json::json!({
                "type": "assistant",
                "timestamp": "2026-10-25T01:12:00Z",
                "requestId": id,
                "message": {
                    "id": id,
                    "model": "claude-opus-5-5",
                    "usage": {
                        "input_tokens": input,
                        "output_tokens": output,
                        "cache_read_input_tokens": cache_read,
                        "cache_creation_input_tokens": cache_write,
                        "cache_creation": { "ephemeral_1h_input_tokens": hour },
                    },
                },
            });
            let _ = reader.request(shaped.to_string().as_bytes());
        }
    }
}
