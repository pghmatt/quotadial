//! `quotadial mcp`: an MCP server named `quotadial` on standard input and output, for agents such as
//! Claude Code and Codex. Its three tools only read:
//!
//! - `get_usage`: the report `quotadial status --json` prints, on every copy.
//! - `get_account_headroom`: the ranking `GET /v1/headroom` serves, on QuotaDial Pro.
//! - `list_alerts`: the alert rules, and what they told in the last week (D-093), on QuotaDial Pro.
//!
//! On QuotaDial Free the two Pro tools answer with a tool error naming the feature and pointing to
//! `get_usage`, so an agent can always read the budget (D-098, D-100).
//!
//! The numbers come from the running daemon, or from a direct fetch when none runs, exactly as for
//! `quotadial status`. The alert rules come from the daemon's history, as for `quotadial alerts`. Messages are JSON-RPC 2.0, one per line, as MCP's stdio transport defines.
//! Only the part of MCP that tools need is here (`initialize`, `ping`, `tools/list` and
//! `tools/call`), which is small enough to need no SDK.

use std::io;

use quotadial_core::{ProFeature, ProviderName, StatusReport};
use serde::Serialize;
use serde_json::{Map, Value, json};
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt};

use quotadial_daemon::{Alerts, Licensing};

use crate::client::Unanswered;
use crate::rank;

/// The MCP revisions this server speaks, newest first.
const PROTOCOL_VERSIONS: [&str; 4] = ["2025-11-25", "2025-06-18", "2025-03-26", "2024-11-05"];
/// The first revision whose tool results carry `structuredContent`.
const STRUCTURED_CONTENT_SINCE: &str = "2025-06-18";

// JSON-RPC 2.0's error codes.
const PARSE_ERROR: i64 = -32_700;
const INVALID_REQUEST: i64 = -32_600;
const METHOD_NOT_FOUND: i64 = -32_601;
const INVALID_PARAMS: i64 = -32_602;

/// What the server tells an agent about itself when it connects.
const INSTRUCTIONS: &str = "QuotaDial reports how much of each Claude and ChatGPT/Codex plan limit \
    the accounts on this machine have used, when each limit resets, and how old every number is. \
    Before starting a long job, call get_account_headroom (or get_usage for every window) to check \
    that the account has room left and when its tightest limit resets. On QuotaDial Free, \
    get_account_headroom and list_alerts answer that they are part of QuotaDial Pro, and get_usage \
    gives every window on any copy. While the QuotaDial daemon \
    runs, a window also has a pace: pct_per_hour, how fast it is filling, and runs_out_at, when it \
    reaches 100% if that comes before it resets. Providers are never asked more often than their \
    limits allow, so a number can be a few minutes old; its fetched_at says when it was fetched.";

/// Answers MCP requests from `input` on `output` until `input` ends. `providers` are the providers
/// an agent may narrow an answer to, by id, each described by its own name (P-020); `report` gets
/// the numbers, and `alerts` the alert rules and what they told, or why it could not; `licensing`
/// says whether the Pro tools answer.
///
/// # Errors
///
/// When `input` cannot be read or `output` written.
pub async fn serve<R, W, F, A>(
    input: R,
    mut output: W,
    providers: &[ProviderName],
    report: F,
    alerts: A,
    licensing: Licensing,
) -> io::Result<()>
where
    R: AsyncBufRead + Unpin,
    W: AsyncWrite + Unpin,
    F: AsyncFn() -> Result<StatusReport, Unanswered>,
    A: AsyncFn() -> Result<Alerts, String>,
{
    let sources = Sources { report, alerts };
    let mut server = Server {
        providers,
        version: None,
        licensing,
    };
    // Lines are read as bytes, so one that is not UTF-8 gets a parse error rather than ending the
    // server.
    let mut lines = input.split(b'\n');
    while let Some(line) = lines.next_segment().await? {
        if line.trim_ascii().is_empty() {
            continue;
        }
        if let Some(reply) = server.receive(&line, &sources).await {
            // Compact JSON escapes every newline, so each reply is exactly one line.
            let mut text = serde_json::to_string(&reply)?;
            text.push('\n');
            output.write_all(text.as_bytes()).await?;
            output.flush().await?;
        }
    }
    Ok(())
}

/// Where the tools' answers come from.
struct Sources<F, A> {
    report: F,
    alerts: A,
}

struct Server<'a> {
    providers: &'a [ProviderName],
    /// The revision agreed at `initialize`.
    version: Option<&'static str>,
    /// Whether the copy has the Pro tools.
    licensing: Licensing,
}

/// A JSON-RPC error: its code and message.
type Failure = (i64, String);

impl Server<'_> {
    /// The reply to one line, if it needs one: notifications and responses get none.
    async fn receive<F, A>(&mut self, line: &[u8], sources: &Sources<F, A>) -> Option<Value>
    where
        F: AsyncFn() -> Result<StatusReport, Unanswered>,
        A: AsyncFn() -> Result<Alerts, String>,
    {
        let Ok(message) = serde_json::from_slice::<Value>(line) else {
            return Some(failure(
                Value::Null,
                (PARSE_ERROR, "The message is not valid JSON.".to_owned()),
            ));
        };
        match message {
            // Batches are part of the 2025-03-26 revision.
            Value::Array(batch) if batch.is_empty() => Some(failure(
                Value::Null,
                (INVALID_REQUEST, "The batch is empty.".to_owned()),
            )),
            Value::Array(batch) => {
                let mut replies = Vec::new();
                for message in batch {
                    replies.extend(self.handle(message, sources).await);
                }
                (!replies.is_empty()).then_some(Value::Array(replies))
            }
            message => self.handle(message, sources).await,
        }
    }

    async fn handle<F, A>(&mut self, message: Value, sources: &Sources<F, A>) -> Option<Value>
    where
        F: AsyncFn() -> Result<StatusReport, Unanswered>,
        A: AsyncFn() -> Result<Alerts, String>,
    {
        let invalid =
            |message: &str| Some(failure(Value::Null, (INVALID_REQUEST, message.to_owned())));
        let Value::Object(message) = message else {
            return invalid("A message must be a JSON object.");
        };
        if message.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
            return invalid("A message must say \"jsonrpc\": \"2.0\".");
        }
        // This server sends no requests, so a response from the client answers nothing.
        if message.contains_key("result") || message.contains_key("error") {
            return None;
        }
        let Some(method) = message.get("method").and_then(Value::as_str) else {
            return invalid("A request must name its method.");
        };
        // A notification, such as `notifications/initialized`, needs no reply.
        let id = message.get("id")?;
        if !(id.is_string() || id.is_i64() || id.is_u64()) {
            return invalid("A request's id must be a string or an integer.");
        }
        let params = message.get("params").unwrap_or(&Value::Null);
        let result = match method {
            "initialize" => Ok(self.initialize(params)),
            "ping" => Ok(json!({})),
            "tools/list" => Ok(json!({ "tools": tools(self.providers) })),
            "tools/call" => self.call(params, sources).await,
            _ => Err((
                METHOD_NOT_FOUND,
                format!("QuotaDial's MCP server has no method {method:?}."),
            )),
        };
        Some(match result {
            Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
            Err(error) => failure(id.clone(), error),
        })
    }

    /// Agrees on the client's revision when this server speaks it, and otherwise offers the newest
    /// this server speaks, as MCP's version negotiation asks.
    fn initialize(&mut self, params: &Value) -> Value {
        let requested = params.get("protocolVersion").and_then(Value::as_str);
        let version = PROTOCOL_VERSIONS
            .into_iter()
            .find(|version| Some(*version) == requested)
            .unwrap_or(PROTOCOL_VERSIONS[0]);
        self.version = Some(version);
        json!({
            "protocolVersion": version,
            "capabilities": { "tools": { "listChanged": false } },
            "serverInfo": { "name": "quotadial", "version": env!("CARGO_PKG_VERSION") },
            "instructions": INSTRUCTIONS,
        })
    }

    async fn call<F, A>(&self, params: &Value, sources: &Sources<F, A>) -> Result<Value, Failure>
    where
        F: AsyncFn() -> Result<StatusReport, Unanswered>,
        A: AsyncFn() -> Result<Alerts, String>,
    {
        let Some(name) = params.get("name").and_then(Value::as_str) else {
            return Err((
                INVALID_PARAMS,
                "tools/call needs the name of a tool.".to_owned(),
            ));
        };
        let empty = Map::new();
        let arguments = match params.get("arguments") {
            None | Some(Value::Null) => &empty,
            Some(Value::Object(arguments)) => arguments,
            Some(_) => {
                return Err((
                    INVALID_PARAMS,
                    "A tool's arguments must be a JSON object.".to_owned(),
                ));
            }
        };
        let (narrows, pro) = match name {
            "get_usage" => (true, None),
            "get_account_headroom" => (true, Some(ProFeature::Headroom)),
            "list_alerts" => (false, Some(ProFeature::RulesApi)),
            _ => {
                return Err((
                    INVALID_PARAMS,
                    format!(
                        "QuotaDial has no tool {name:?}; its tools are get_usage, \
                         get_account_headroom and list_alerts."
                    ),
                ));
            }
        };
        if let Some(feature) = pro.filter(|feature| !self.licensing.allows(*feature)) {
            return Ok(tool_error(&format!(
                "{} get_usage gives every window, with its reset, on any copy.",
                feature.line()
            )));
        }
        // A wrong argument is the tool's error, not the protocol's, so the agent sees it and can
        // correct the call.
        let provider = match self.provider(arguments, narrows) {
            Ok(provider) => provider,
            Err(message) => return Ok(tool_error(&message)),
        };
        if name == "list_alerts" {
            return Ok(match (sources.alerts)().await {
                Ok(alerts) => self.tool_result(&alerts),
                Err(message) => tool_error(&message),
            });
        }
        let report = match (sources.report)().await {
            Ok(report) => only(report, provider),
            Err(unanswered) => return Ok(tool_error(&unanswered.to_string())),
        };
        Ok(if name == "get_usage" {
            self.tool_result(&report)
        } else {
            self.tool_result(&rank(&report))
        })
    }

    /// The `provider` argument, checked against the providers QuotaDial has. Any other argument is
    /// refused, naming the ones the tool takes.
    fn provider<'a>(
        &self,
        arguments: &'a Map<String, Value>,
        narrows: bool,
    ) -> Result<Option<&'a str>, String> {
        let takes = if narrows {
            "Its only argument is provider."
        } else {
            "It takes no arguments."
        };
        if let Some(unknown) = arguments
            .keys()
            .find(|key| !(narrows && key.as_str() == "provider"))
        {
            return Err(format!("The tool has no argument {unknown:?}. {takes}"));
        }
        match arguments.get("provider") {
            None | Some(Value::Null) => Ok(None),
            Some(Value::String(provider))
                if self.providers.iter().any(|named| named.id == *provider) =>
            {
                Ok(Some(provider.as_str()))
            }
            Some(other) => Err(format!(
                "provider must be one of {}, not {other}.",
                self.providers
                    .iter()
                    .map(|named| named.id.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
        }
    }

    /// A tool's answer: its JSON as text, as every revision has it, with its fields in the order
    /// `quotadial status --json` prints them, and as `structuredContent` from the revision that
    /// added it.
    fn tool_result(&self, answer: &impl Serialize) -> Value {
        let (Ok(text), Ok(structured)) =
            (serde_json::to_string(answer), serde_json::to_value(answer))
        else {
            return tool_error("QuotaDial could not encode its answer.");
        };
        let mut result = json!({
            "content": [{ "type": "text", "text": text }],
            "isError": false,
        });
        if self
            .version
            .is_none_or(|version| version >= STRUCTURED_CONTENT_SINCE)
        {
            result["structuredContent"] = structured;
        }
        result
    }
}

/// `report` with only `provider`'s accounts, errors, status page, cost and what QuotaDial Free
/// leaves out of it, or all of them. QuotaDial's own problems are every provider's.
fn only(report: StatusReport, provider: Option<&str>) -> StatusReport {
    let Some(provider) = provider else {
        return report;
    };
    StatusReport {
        snapshots: report
            .snapshots
            .into_iter()
            .filter(|snapshot| snapshot.account.provider == provider)
            .collect(),
        errors: report
            .errors
            .into_iter()
            .filter(|error| error.provider == provider)
            .collect(),
        status_pages: report
            .status_pages
            .into_iter()
            .filter(|page| page.provider == provider)
            .collect(),
        costs: report
            .costs
            .into_iter()
            .filter(|cost| cost.provider == provider)
            .collect(),
        left_out: report
            .left_out
            .into_iter()
            .filter(|left_out| left_out.provider.as_deref().is_none_or(|of| of == provider))
            .collect(),
        problems: report.problems,
        providers: report
            .providers
            .into_iter()
            .filter(|named| named.id == provider)
            .collect(),
    }
}

/// The three tools, for `tools/list`. The `provider` argument takes each provider's id, described
/// by the provider's own name, so no table of names is kept here (P-020).
fn tools(providers: &[ProviderName]) -> Value {
    let ids: Vec<&str> = providers.iter().map(|named| named.id.as_str()).collect();
    let each = providers
        .iter()
        .map(|named| format!("{} for {}", named.id, named.name))
        .collect::<Vec<_>>()
        .join(", ");
    let narrowed = json!({
        "type": "object",
        "properties": {
            "provider": {
                "type": "string",
                "enum": ids,
                "description": format!("Only this provider's accounts, by its id: {each}."),
            },
        },
        "additionalProperties": false,
    });
    let read_only = json!({ "readOnlyHint": true });
    json!([
        {
            "name": "get_usage",
            "description": "Usage of every Claude and ChatGPT/Codex plan limit for the accounts on \
                            this machine. For each account: its status, and for every window the \
                            share used (used_pct), when it resets (resets_at), where the number \
                            came from (source), when it was fetched (fetched_at) and, while the \
                            daemon runs, its pace (pace.pct_per_hour, and pace.runs_out_at when it \
                            runs out before it resets). While the daemon runs, costs gives what \
                            the week's local use of each provider would have cost at API prices \
                            (usd, requests, tokens and models), on QuotaDial Pro; on QuotaDial \
                            Free, left_out names what the copy leaves out. errors lists each \
                            provider or account that gave no numbers: provider is the provider's \
                            id, and message says what happened and what to do and leaves the \
                            provider out, since providers gives each provider's own name for its \
                            id. The same JSON as `quotadial status --json`.",
            "inputSchema": narrowed,
            "annotations": read_only,
        },
        {
            "name": "get_account_headroom",
            "description": "The accounts ranked by headroom, the room left in each account's \
                            tightest window: headroom_pct is 100 minus that window's share used, \
                            and tightest is the window, with when it resets and its pace. The most \
                            headroom comes first; costs is the week's cost of each provider at API \
                            prices, while the daemon runs; and errors is what gave no numbers, as \
                            in get_usage. Use it before a long job to check that an account has \
                            room left. Part of QuotaDial Pro; on QuotaDial Free use get_usage.",
            "inputSchema": narrowed,
            "annotations": read_only,
        },
        {
            "name": "list_alerts",
            "description": "The alert rules the user set, each with the words it was written \
                            in, what QuotaDial read it as (kind, pct, minutes, provider, window) \
                            and whether it is on, and what the rules told in the last week: each \
                            firing's title, text, account, window and time. Part of QuotaDial \
                            Pro.",
            "inputSchema": { "type": "object", "properties": {}, "additionalProperties": false },
            "annotations": read_only,
        },
    ])
}

fn tool_error(message: &str) -> Value {
    json!({
        "content": [{ "type": "text", "text": message }],
        "isError": true,
    })
}

fn failure(id: Value, (code, message): Failure) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};
    use quotadial_core::{Account, ProviderError, Snapshot, Source, Status, Window, WindowKind};

    use super::*;

    fn at(text: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(text)
            .expect("valid timestamp")
            .with_timezone(&Utc)
    }

    fn snapshot(provider: &str, label: &str, used_pct: f64) -> Snapshot {
        Snapshot {
            account: Account {
                provider: provider.into(),
                provider_name: format!("The {provider} provider"),
                account_id: format!("{provider}-user"),
                display_name: "user@example.com".into(),
                plan: None,
                org: None,
                sign_in: None,
                sign_in_use: "QuotaDial reads this test's sign-in.".into(),
            },
            windows: vec![Window {
                kind: WindowKind::Weekly,
                label: label.into(),
                used_pct,
                resets_at: Some(at("2099-01-07T00:00:00Z")),
                window_seconds: 604_800,
                scope_model: None,
                source: Source::Api,
                fetched_at: at("2099-01-01T00:00:00Z"),
                unconfirmed: false,
                pace: None,
            }],
            extra_usage: None,
            status: Status::Ok,
            error: None,
        }
    }

    fn report() -> StatusReport {
        StatusReport {
            snapshots: vec![
                snapshot("claude", "Weekly", 70.0),
                snapshot("codex", "Weekly", 20.0),
            ],
            errors: vec![ProviderError {
                provider: "codex".into(),
                message: "request failed: Could not reach ChatGPT.".into(),
            }],
            status_pages: Vec::new(),
            costs: Vec::new(),
            left_out: Vec::new(),
            problems: Vec::new(),
            providers: vec![provider("claude", "Claude"), provider("codex", "Codex")],
        }
    }

    fn provider(id: &str, name: &str) -> quotadial_core::ProviderName {
        quotadial_core::ProviderName {
            id: id.into(),
            name: name.into(),
        }
    }

    /// Sends `input` to a server on QuotaDial Pro answering from `answer` and returns its replies,
    /// checking that each is one line of JSON.
    async fn exchange_raw(input: &str, answer: Result<StatusReport, Unanswered>) -> Vec<Value> {
        exchange_on(true, input, answer).await
    }

    /// [`exchange_raw`] on QuotaDial Pro, or on QuotaDial Free with no key.
    async fn exchange_on(
        pro: bool,
        input: &str,
        answer: Result<StatusReport, Unanswered>,
    ) -> Vec<Value> {
        let temp = tempfile::tempdir().expect("temp dir");
        let dir = quotadial_daemon::DataDir::new(temp.path().join("quotadial"));
        let licensing = if pro {
            crate::tests::pro(&dir)
        } else {
            Licensing::new(Some(&dir))
        };
        let mut output = Vec::new();
        serve(
            input.as_bytes(),
            &mut output,
            &[provider("claude", "Claude"), provider("codex", "Codex")],
            async || answer.clone(),
            async || Ok(alerts()),
            licensing,
        )
        .await
        .expect("serves");
        let text = String::from_utf8(output).expect("UTF-8");
        assert!(text.is_empty() || text.ends_with('\n'), "{text}");
        text.lines()
            .map(|line| serde_json::from_str(line).expect("one JSON message per line"))
            .collect()
    }

    async fn exchange(messages: &[Value]) -> Vec<Value> {
        let input: String = messages
            .iter()
            .map(|message| format!("{message}\n"))
            .collect();
        exchange_raw(&input, Ok(report())).await
    }

    fn request(id: u64, method: &str, params: Value) -> Value {
        json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params })
    }

    fn initialize(version: &str) -> Value {
        request(
            0,
            "initialize",
            json!({
                "protocolVersion": version,
                "capabilities": {},
                "clientInfo": { "name": "test", "version": "1.0" },
            }),
        )
    }

    fn call(id: u64, name: &str, arguments: Value) -> Value {
        request(
            id,
            "tools/call",
            json!({ "name": name, "arguments": arguments }),
        )
    }

    #[tokio::test]
    async fn the_server_agrees_on_a_revision_and_names_itself_quotadial() {
        for (requested, agreed) in [
            ("2025-06-18", "2025-06-18"),
            ("2024-11-05", "2024-11-05"),
            ("2099-01-01", "2025-11-25"),
        ] {
            let replies = exchange(&[initialize(requested)]).await;
            let result = &replies[0]["result"];
            assert_eq!(result["protocolVersion"], agreed, "{requested}");
            assert_eq!(result["serverInfo"]["name"], "quotadial");
            assert_eq!(
                result["capabilities"],
                json!({ "tools": { "listChanged": false } })
            );
            assert!(
                result["instructions"]
                    .as_str()
                    .expect("instructions")
                    .contains("get_account_headroom")
            );
        }
    }

    #[tokio::test]
    async fn the_three_read_only_tools_are_listed() {
        let replies = exchange(&[
            initialize("2025-11-25"),
            json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }),
            request(1, "tools/list", json!({})),
        ])
        .await;

        assert_eq!(replies.len(), 2, "the notification gets no reply");
        let tools = replies[1]["result"]["tools"].as_array().expect("tools");
        let names: Vec<&str> = tools
            .iter()
            .map(|tool| tool["name"].as_str().expect("a name"))
            .collect();
        assert_eq!(names, ["get_usage", "get_account_headroom", "list_alerts"]);
        for tool in tools {
            assert_eq!(tool["annotations"]["readOnlyHint"], true);
            assert_eq!(tool["inputSchema"]["type"], "object");
            assert!(
                !tool["description"]
                    .as_str()
                    .expect("a description")
                    .is_empty()
            );
        }
        assert_eq!(
            tools[0]["inputSchema"]["properties"]["provider"]["enum"],
            json!(["claude", "codex"])
        );
        assert_eq!(
            tools[0]["inputSchema"]["properties"]["provider"]["description"],
            "Only this provider's accounts, by its id: claude for Claude, codex for Codex.",
            "each provider as it names itself (P-020)"
        );
    }

    #[tokio::test]
    async fn the_tools_say_an_errors_message_leaves_its_provider_out() {
        let replies = exchange(&[
            initialize("2025-11-25"),
            request(1, "tools/list", json!({})),
        ])
        .await;
        let tools = replies[1]["result"]["tools"].as_array().expect("tools");
        let described = |name: &str| {
            tools
                .iter()
                .find(|tool| tool["name"] == name)
                .and_then(|tool| tool["description"].as_str())
                .expect("a described tool")
                .to_owned()
        };

        // The provider is in `provider`, by its id, and `providers` names it (D-128).
        let usage = described("get_usage");
        for words in [
            "errors lists",
            "provider is the provider's id",
            "message says what happened and what to do and leaves the provider out",
            "providers gives each provider's own name",
        ] {
            assert!(usage.contains(words), "{words:?} in {usage}");
        }
        let headroom = described("get_account_headroom");
        assert!(
            headroom.contains("errors is what gave no numbers, as in get_usage"),
            "{headroom}"
        );
        for tool in tools {
            let description = tool["description"].as_str().expect("a description");
            // #52's plain words: never "quota", "auth" or "credential" outside a name.
            assert_eq!(
                quotadial_core::text::unplain_words(description),
                Vec::<String>::new(),
                "{description}"
            );
        }
    }

    #[tokio::test]
    async fn get_usage_answers_with_the_status_report() {
        let replies = exchange(&[initialize("2025-11-25"), call(1, "get_usage", json!({}))]).await;

        let result = &replies[1]["result"];
        assert_eq!(result["isError"], false);
        let expected = serde_json::to_value(report()).expect("encodes");
        assert_eq!(result["structuredContent"], expected);
        let text = result["content"][0]["text"].as_str().expect("text");
        assert_eq!(
            serde_json::from_str::<Value>(text).expect("JSON text"),
            expected
        );
        assert!(
            text.starts_with(
                r#"{"snapshots":[{"account":{"provider":"claude","provider_name":"The claude provider","account_id""#
            ),
            "the text keeps the order `quotadial status --json` prints: {text}"
        );
    }

    #[tokio::test]
    async fn get_account_headroom_answers_with_the_ranking() {
        let replies = exchange(&[call(1, "get_account_headroom", json!({}))]).await;

        let answer = &replies[0]["result"]["structuredContent"];
        assert_eq!(
            *answer,
            serde_json::to_value(rank(&report())).expect("encodes")
        );
        assert_eq!(answer["accounts"][0]["account"]["provider"], "codex");
        assert_eq!(answer["accounts"][0]["headroom_pct"], 80.0);
    }

    #[tokio::test]
    async fn on_free_the_pro_tools_say_so_and_get_usage_still_answers() {
        let input: String = [
            call(1, "get_account_headroom", json!({})),
            call(2, "list_alerts", json!({})),
            call(3, "get_usage", json!({})),
        ]
        .iter()
        .map(|message| format!("{message}\n"))
        .collect();
        let replies = exchange_on(false, &input, Ok(report())).await;

        for (reply, line) in replies[..2].iter().zip([
            "Headroom through the local API and MCP is part of QuotaDial Pro: quotadial.com/pro",
            "Alert rules through the local API and MCP are part of QuotaDial Pro: \
             quotadial.com/pro",
        ]) {
            assert_eq!(reply["result"]["isError"], true, "{reply}");
            assert_eq!(
                reply["result"]["content"][0]["text"],
                format!("{line} get_usage gives every window, with its reset, on any copy.")
            );
        }
        assert_eq!(
            replies[2]["result"]["isError"], false,
            "status reads on any copy"
        );
    }

    #[tokio::test]
    async fn a_provider_narrows_the_answer() {
        let replies = exchange(&[
            call(1, "get_usage", json!({ "provider": "claude" })),
            call(2, "get_account_headroom", json!({ "provider": "codex" })),
        ])
        .await;

        let usage = &replies[0]["result"]["structuredContent"];
        assert_eq!(usage["snapshots"].as_array().expect("snapshots").len(), 1);
        assert_eq!(usage["snapshots"][0]["account"]["provider"], "claude");
        assert_eq!(usage["errors"], json!([]));
        let headroom = &replies[1]["result"]["structuredContent"];
        assert_eq!(headroom["accounts"].as_array().expect("accounts").len(), 1);
        assert_eq!(headroom["errors"][0]["provider"], "codex");
    }

    /// One rule, as the history keeps it, and nothing told.
    fn alerts() -> Alerts {
        Alerts {
            rules: vec![quotadial_daemon::StoredRule {
                id: 1,
                written: "warn me at 80%".into(),
                enabled: true,
                rule: quotadial_core::Rule::parse("warn me at 80%").expect("reads"),
                created_at: at("2099-01-01T00:00:00Z"),
                waits: false,
            }],
            firings: Vec::new(),
        }
    }

    #[test]
    fn narrowing_to_a_provider_keeps_its_status_page_cost_and_name_only() {
        let mut report = report();
        let page = |provider: &str| quotadial_core::StatusPage {
            provider: provider.into(),
            indicator: quotadial_core::Indicator::Minor,
            description: "Degraded".into(),
            url: format!("https://status.{provider}.example"),
            since: None,
            checked_at: at("2099-01-01T00:00:00Z"),
        };
        let cost = |provider: &str| quotadial_core::Cost {
            provider: provider.into(),
            since: at("2098-12-25T00:00:00Z"),
            until: at("2099-01-01T00:00:00Z"),
            usd: 1.5,
            requests: 2,
            tokens: quotadial_core::Tokens::default(),
            models: Vec::new(),
            prices_updated: "2026-10-23".into(),
            notes: Vec::new(),
        };
        report.status_pages = vec![page("claude"), page("codex")];
        report.costs = vec![cost("claude"), cost("codex")];

        let narrowed = only(report, Some("codex"));

        assert_eq!(narrowed.status_pages, [page("codex")]);
        assert_eq!(narrowed.costs, [cost("codex")]);
        assert_eq!(narrowed.providers, [provider("codex", "Codex")]);
    }

    #[tokio::test]
    async fn list_alerts_gives_the_rules_and_what_they_told() {
        let replies = exchange(&[call(1, "list_alerts", json!({}))]).await;

        assert_eq!(
            replies[0]["result"]["structuredContent"],
            json!({
                "rules": [{
                    "id": 1,
                    "written": "warn me at 80%",
                    "enabled": true,
                    "kind": "threshold",
                    "pct": 80.0,
                    "created_at": "2099-01-01T00:00:00Z",
                }],
                "firings": [],
            })
        );
    }

    #[tokio::test]
    async fn a_wrong_argument_is_a_tool_error_the_agent_can_correct() {
        let replies = exchange(&[
            call(1, "get_usage", json!({ "provider": "gemini" })),
            call(2, "get_usage", json!({ "account": "user@example.com" })),
            call(3, "list_alerts", json!({ "provider": "claude" })),
        ])
        .await;

        let messages: Vec<&str> = replies
            .iter()
            .map(|reply| {
                assert_eq!(reply["result"]["isError"], true);
                reply["result"]["content"][0]["text"]
                    .as_str()
                    .expect("text")
            })
            .collect();
        assert_eq!(
            messages,
            [
                "provider must be one of claude, codex, not \"gemini\".",
                "The tool has no argument \"account\". Its only argument is provider.",
                "The tool has no argument \"provider\". It takes no arguments.",
            ]
        );
    }

    #[tokio::test]
    async fn a_daemon_that_does_not_answer_is_a_tool_error() {
        let unanswered = Unanswered("the QuotaDial daemon is running, but ...".into());
        let input = format!("{}\n", call(1, "get_usage", json!({})));

        let replies = exchange_raw(&input, Err(unanswered.clone())).await;

        assert_eq!(replies[0]["result"]["isError"], true);
        assert_eq!(
            replies[0]["result"]["content"][0]["text"],
            unanswered.to_string()
        );
    }

    #[tokio::test]
    async fn revisions_before_structured_content_get_text_only() {
        let replies = exchange(&[initialize("2025-03-26"), call(1, "get_usage", json!({}))]).await;

        let result = &replies[1]["result"];
        assert!(result.get("structuredContent").is_none());
        assert!(result["content"][0]["text"].is_string());
    }

    #[tokio::test]
    async fn protocol_errors_follow_json_rpc() {
        let input = [
            "not json".to_owned(),
            request(1, "resources/list", json!({})).to_string(),
            request(2, "tools/call", json!({ "name": "delete_everything" })).to_string(),
            request(3, "tools/call", json!({})).to_string(),
            json!({ "jsonrpc": "2.0", "id": 4 }).to_string(),
            json!({ "jsonrpc": "1.0", "id": 5, "method": "ping" }).to_string(),
            json!({ "jsonrpc": "2.0", "id": null, "method": "ping" }).to_string(),
            json!({ "jsonrpc": "2.0", "id": "seven", "method": "ping" }).to_string(),
            json!({ "jsonrpc": "2.0", "id": 8, "result": {} }).to_string(),
            "[]".to_owned(),
            "42".to_owned(),
            String::new(),
        ]
        .join("\n");

        let replies = exchange_raw(&input, Ok(report())).await;

        let summary: Vec<(Value, Value)> = replies
            .iter()
            .map(|reply| {
                (
                    reply["id"].clone(),
                    reply
                        .get("error")
                        .map_or_else(|| reply["result"].clone(), |error| error["code"].clone()),
                )
            })
            .collect();
        assert_eq!(
            summary,
            [
                (Value::Null, json!(-32_700)),
                (json!(1), json!(-32_601)),
                (json!(2), json!(-32_602)),
                (json!(3), json!(-32_602)),
                (Value::Null, json!(-32_600)),
                (Value::Null, json!(-32_600)),
                (Value::Null, json!(-32_600)),
                (json!("seven"), json!({})),
                (Value::Null, json!(-32_600)),
                (Value::Null, json!(-32_600)),
            ],
            "a response from the client and a blank line get no reply"
        );
    }

    #[tokio::test]
    async fn a_line_that_is_not_utf_8_gets_a_parse_error_and_the_server_goes_on() {
        let mut output = Vec::new();
        let input = [
            b"{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"ping\"}\r\n".as_slice(),
            b"\xff\xfe\n",
            b"{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"ping\"}",
        ]
        .concat();

        serve(
            input.as_slice(),
            &mut output,
            &[provider("claude", "Claude")],
            async || Ok(report()),
            async || Ok(alerts()),
            Licensing::default(),
        )
        .await
        .expect("serves");

        let replies: Vec<Value> = String::from_utf8(output)
            .expect("UTF-8")
            .lines()
            .map(|line| serde_json::from_str(line).expect("JSON"))
            .collect();
        assert_eq!(
            replies[0]["result"],
            json!({}),
            "a CRLF line ending is fine"
        );
        assert_eq!(replies[1]["error"]["code"], -32_700);
        assert_eq!(replies[2]["id"], 2, "the last line needs no newline");
    }

    #[tokio::test]
    async fn a_batch_gets_a_batch_of_replies() {
        let batch = json!([
            request(1, "ping", json!({})),
            { "jsonrpc": "2.0", "method": "notifications/initialized" },
            call(2, "list_alerts", json!({})),
        ]);

        let replies = exchange(&[batch]).await;

        let replies = replies[0].as_array().expect("a batch");
        assert_eq!(replies.len(), 2);
        assert_eq!(replies[0]["id"], 1);
        assert_eq!(replies[1]["id"], 2);
    }

    /// A line an agent might send: arbitrary bytes or JSON, or a JSON-RPC message with arbitrary
    /// parts.
    fn message() -> impl proptest::strategy::Strategy<Value = Vec<u8>> {
        use proptest::prelude::*;
        let method = prop_oneof![
            prop::sample::select(
                &[
                    "initialize",
                    "notifications/initialized",
                    "ping",
                    "tools/list",
                    "tools/call",
                ][..]
            )
            .prop_map(str::to_owned),
            ".{0,20}",
        ];
        let tool = prop_oneof![
            prop::sample::select(&["get_usage", "get_account_headroom", "list_alerts"][..])
                .prop_map(str::to_owned),
            ".{0,20}",
        ];
        let call = (
            quotadial_core::testing::json(),
            method,
            tool,
            quotadial_core::testing::json(),
            quotadial_core::testing::json(),
        )
            .prop_map(|(id, method, tool, arguments, params)| {
                let params = if method == "tools/call" {
                    json!({ "name": tool, "arguments": arguments })
                } else {
                    params
                };
                json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params })
                    .to_string()
                    .into_bytes()
            });
        prop_oneof![quotadial_core::testing::line(), call]
    }

    #[test]
    fn no_input_makes_the_server_panic_and_every_answer_is_json() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("a runtime");
        proptest::proptest!(|(lines in proptest::collection::vec(message(), 0..6))| {
            let input = lines.join(&b"\n"[..]);
            let mut output = Vec::new();
            runtime
                .block_on(serve(
                    input.as_slice(),
                    &mut output,
                    &[provider("claude", "Claude")],
                    async || Ok(report()),
                    async || Ok(alerts()),
                    Licensing::default(),
                ))
                .expect("serves");
            for line in String::from_utf8(output).expect("UTF-8").lines() {
                proptest::prop_assert!(serde_json::from_str::<Value>(line).is_ok(), "{line}");
            }
        });
    }
}
