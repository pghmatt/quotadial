//! `docs/openapi.json`: the local API in OpenAPI 3.1, its schemas written from the very types the
//! API sends and takes, so the description cannot drift from what the API does. The API serves the
//! committed file; this test fails when the file and the code disagree, and
//! `QUOTADIAL_WRITE_OPENAPI=1 cargo test -p quotadial-api openapi` writes it again.

use std::path::PathBuf;

use quotadial_core::{ProFeature, StatusReport};
use quotadial_daemon::{Alerts, StoredRule};
use schemars::JsonSchema;
use schemars::generate::{Contract, SchemaSettings};
use serde_json::{Map, Value, json};

use crate::HeadroomReport;
use crate::server::{HistoryAnswer, NewRule, RuleChange};

/// Where the API is described, from this crate.
fn committed() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/openapi.json")
}

/// The schemas of what the API sends and takes, under `components/schemas`, keyed by type name.
struct Schemas {
    sent: schemars::SchemaGenerator,
    taken: schemars::SchemaGenerator,
}

impl Schemas {
    fn new() -> Self {
        let generator = |contract| {
            let mut settings = SchemaSettings::draft2020_12();
            settings.definitions_path = "/components/schemas".into();
            settings.contract = contract;
            settings.into_generator()
        };
        Self {
            sent: generator(Contract::Serialize),
            taken: generator(Contract::Deserialize),
        }
    }

    /// A reference to what the API sends as `T`.
    fn sent<T: JsonSchema>(&mut self) -> Value {
        self.sent.subschema_for::<T>().to_value()
    }

    /// A reference to what the API takes as `T`.
    fn taken<T: JsonSchema>(&mut self) -> Value {
        self.taken.subschema_for::<T>().to_value()
    }

    /// Every schema referred to, with the descriptions cleaned for people who use the API.
    fn all(mut self) -> Map<String, Value> {
        let mut all = self.sent.take_definitions(true);
        all.extend(self.taken.take_definitions(true));
        for schema in all.values_mut() {
            clean(schema);
        }
        all
    }
}

/// Makes every description in `schema` read as plain text: lines joined into paragraphs, decision
/// numbers such as "(D-096)" dropped, and Rust's link brackets taken off a name.
fn clean(schema: &mut Value) {
    match schema {
        Value::Object(object) => {
            if let Some(Value::String(description)) = object.get_mut("description") {
                *description = readable(description);
            }
            object.values_mut().for_each(clean);
        }
        Value::Array(items) => items.iter_mut().for_each(clean),
        _ => {}
    }
}

fn readable(description: &str) -> String {
    let joined = description
        .split("\n\n")
        .map(|paragraph| {
            paragraph
                .split('\n')
                .map(str::trim)
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect::<Vec<_>>()
        .join("\n\n");
    let mut text = String::with_capacity(joined.len());
    let mut rest = joined.as_str();
    while let Some(start) = rest.find(" (") {
        let (before, after) = rest.split_at(start);
        match after[2..].split_once(')') {
            Some((reference, tail)) if is_decision(reference) => {
                text.push_str(before);
                rest = tail;
            }
            _ => {
                text.push_str(&rest[..start + 2]);
                rest = &rest[start + 2..];
            }
        }
    }
    text.push_str(rest);
    text.replace("[`", "`").replace("`]", "`")
}

/// Whether `text` is only decision and proposal numbers, such as "D-096" or "D-094, P-016".
fn is_decision(text: &str) -> bool {
    text.split(", ").all(|part| {
        let mut chars = part.chars();
        matches!(chars.next(), Some('D' | 'P'))
            && chars.next() == Some('-')
            && chars.as_str().len() == 3
            && chars.all(|char| char.is_ascii_digit())
    })
}

/// A JSON answer: `schema` under `application/json`, described as `description`.
fn answer(description: &str, schema: &Value) -> Value {
    json!({
        "description": description,
        "content": { "application/json": { "schema": schema } }
    })
}

/// An answer with an [`Error`](#/components/schemas/Error) body.
fn error(description: &str) -> Value {
    answer(
        description,
        &json!({ "$ref": "#/components/schemas/Error" }),
    )
}

/// The API, as `docs/openapi.json` holds it.
fn spec() -> Value {
    let mut schemas = Schemas::new();
    let status = schemas.sent::<StatusReport>();
    let headroom = schemas.sent::<HeadroomReport>();
    let history = schemas.sent::<HistoryAnswer>();
    let alerts = schemas.sent::<Alerts>();
    let rule = schemas.sent::<StoredRule>();
    let pro_feature = schemas.sent::<ProFeature>();
    let new_rule = schemas.taken::<NewRule>();
    let rule_change = schemas.taken::<RuleChange>();
    let mut components = schemas.all();
    components.insert(
        "Error".into(),
        json!({
            "type": "object",
            "description": "What went wrong, in one sentence that says what to do about it.",
            "properties": {
                "error": { "type": "string" },
                "pro_feature": {
                    "allOf": [pro_feature],
                    "description": "On QuotaDial Free, the QuotaDial Pro feature the request needs."
                }
            },
            "required": ["error"]
        }),
    );
    let pro_only = |what: &str| {
        error(&format!(
            "On QuotaDial Free, {what} is part of QuotaDial Pro, and `pro_feature` names it. Also \
             any request whose `Host` is not `127.0.0.1:<port>` or `localhost:<port>`."
        ))
    };
    let rule_id = json!({
        "name": "id",
        "in": "path",
        "required": true,
        "description": "The rule's id, as `GET /v1/alerts` lists it.",
        "schema": { "type": "integer", "format": "int64" }
    });
    let unreadable = error("QuotaDial could not use its history, or stopped while answering.");
    json!({
        "openapi": "3.1.0",
        "info": {
            "title": "QuotaDial local API",
            "version": "1",
            "summary": "How much of your Claude and ChatGPT/Codex plan limits you have used, from the QuotaDial daemon on this machine.",
            "description": "The QuotaDial daemon answers on `127.0.0.1` only, at the port in `daemon.json` in QuotaDial's data folder, 47115 unless something else had it. Every request needs the token from the `token` file in the same folder, as `Authorization: Bearer <token>`; without it, or with another, the answer is 401. A request whose `Host` is not `127.0.0.1:<port>` or `localhost:<port>` gets 403, so a web page cannot reach the API through a name of its own. Any other path gets 404 and any other method 405, each with an `Error`. Status and history answer on every copy; headroom and the alert rules are part of QuotaDial Pro, and on QuotaDial Free they answer 403 naming the feature in `pro_feature`.",
            "license": { "name": "MIT", "identifier": "MIT" }
        },
        "externalDocs": {
            "description": "The local API in QuotaDial's documentation, with examples",
            "url": "https://github.com/pghmatt/quotadial/blob/main/docs/CLI.md#the-local-api"
        },
        "servers": [{
            "url": "http://127.0.0.1:{port}",
            "description": "The daemon on this machine",
            "variables": {
                "port": {
                    "default": "47115",
                    "description": "The port in `daemon.json` in QuotaDial's data folder."
                }
            }
        }],
        "security": [{ "token": [] }],
        "paths": {
            "/v1/status": {
                "get": {
                    "operationId": "getStatus",
                    "summary": "Every account's limits, as `quotadial status --json` gives them",
                    "responses": {
                        "200": answer("Each account's snapshot, the providers that gave none, and, while the daemon reads them, the status pages, the week's cost and what is wrong with QuotaDial itself.", &status),
                        "401": { "$ref": "#/components/responses/Unauthorized" },
                        "403": { "$ref": "#/components/responses/WrongHost" }
                    }
                }
            },
            "/v1/headroom": {
                "get": {
                    "operationId": "getHeadroom",
                    "summary": "Every account ranked by the room left in its tightest window",
                    "description": "The most headroom first; equal headroom goes to the account whose tightest window resets sooner, and accounts with no windows, or that cannot be used now, come last. Uses the numbers as last fetched. QuotaDial Pro.",
                    "responses": {
                        "200": answer("Every account, ranked.", &headroom),
                        "401": { "$ref": "#/components/responses/Unauthorized" },
                        "403": pro_only("headroom")
                    }
                }
            },
            "/v1/history": {
                "get": {
                    "operationId": "getHistory",
                    "summary": "Each window's readings since a time, as `quotadial history --json` gives them",
                    "parameters": [
                        {
                            "name": "since",
                            "in": "query",
                            "description": "From when, in RFC 3339, such as `2099-01-01T00:00:00Z`. 7 days ago when left out.",
                            "schema": { "type": "string", "format": "date-time" }
                        },
                        {
                            "name": "provider",
                            "in": "query",
                            "description": "Only this provider's accounts, such as `claude`.",
                            "schema": { "type": "string" }
                        },
                        {
                            "name": "account",
                            "in": "query",
                            "description": "Only this account: its id or its display name, in any case.",
                            "schema": { "type": "string" }
                        },
                        {
                            "name": "window",
                            "in": "query",
                            "description": "Only windows with this label, such as `Weekly`, in any case.",
                            "schema": { "type": "string" }
                        }
                    ],
                    "responses": {
                        "200": answer("Each account's window, each provider's in order of account name, with its readings oldest first.", &history),
                        "400": error("`since` is not a time in RFC 3339."),
                        "401": { "$ref": "#/components/responses/Unauthorized" },
                        "403": { "$ref": "#/components/responses/WrongHost" },
                        "500": unreadable
                    }
                }
            },
            "/v1/alerts": {
                "get": {
                    "operationId": "listAlerts",
                    "summary": "The alert rules, and what they told in the last week, as `quotadial alerts list --json` gives them",
                    "description": "QuotaDial Pro.",
                    "responses": {
                        "200": answer("Every rule, oldest first, and at most the newest 100 firings of the last week, oldest first.", &alerts),
                        "401": { "$ref": "#/components/responses/Unauthorized" },
                        "403": pro_only("the alert rules through the API"),
                        "500": unreadable
                    }
                },
                "post": {
                    "operationId": "addAlert",
                    "summary": "Adds an alert rule written in plain words",
                    "description": "Such as `warn me at 80%` or `tell me 30 minutes before the weekly window runs out`. QuotaDial Pro.",
                    "requestBody": {
                        "required": true,
                        "content": { "application/json": { "schema": new_rule } }
                    },
                    "responses": {
                        "201": answer("The rule as kept.", &rule),
                        "400": error("The body is not `{\"rule\": \"...\"}`, or the rule's words could not be read; the error says which part."),
                        "401": { "$ref": "#/components/responses/Unauthorized" },
                        "403": pro_only("adding alert rules through the API, or this kind of rule,"),
                        "500": unreadable
                    }
                }
            },
            "/v1/alerts/{id}": {
                "patch": {
                    "operationId": "setAlert",
                    "summary": "Turns an alert rule on or off",
                    "description": "QuotaDial Pro.",
                    "parameters": [rule_id.clone()],
                    "requestBody": {
                        "required": true,
                        "content": { "application/json": { "schema": rule_change } }
                    },
                    "responses": {
                        "200": answer("The rule as kept now.", &rule),
                        "400": error("The body is not `{\"enabled\": true}` or `{\"enabled\": false}`."),
                        "401": { "$ref": "#/components/responses/Unauthorized" },
                        "403": pro_only("changing alert rules through the API"),
                        "404": error("QuotaDial has no rule with this id."),
                        "500": unreadable
                    }
                },
                "delete": {
                    "operationId": "removeAlert",
                    "summary": "Removes an alert rule",
                    "description": "QuotaDial Pro.",
                    "parameters": [rule_id],
                    "responses": {
                        "204": { "description": "The rule is removed." },
                        "401": { "$ref": "#/components/responses/Unauthorized" },
                        "403": pro_only("removing alert rules through the API"),
                        "404": error("QuotaDial has no rule with this id."),
                        "500": error("QuotaDial could not use its history, or stopped while answering.")
                    }
                }
            },
            "/v1/openapi.json": {
                "get": {
                    "operationId": "getOpenApi",
                    "summary": "This description of the API",
                    "responses": {
                        "200": {
                            "description": "This document, the same as `docs/openapi.json` in QuotaDial's source.",
                            "content": { "application/json": { "schema": { "type": "object" } } }
                        },
                        "401": { "$ref": "#/components/responses/Unauthorized" },
                        "403": { "$ref": "#/components/responses/WrongHost" }
                    }
                }
            }
        },
        "components": {
            "securitySchemes": {
                "token": {
                    "type": "http",
                    "scheme": "bearer",
                    "description": "The 64 hex characters in the `token` file in QuotaDial's data folder, which the daemon makes on its first start, readable only by you."
                }
            },
            "responses": {
                "Unauthorized": {
                    "description": "The request carries no token, or another one.",
                    "headers": {
                        "WWW-Authenticate": { "schema": { "type": "string", "const": "Bearer" } }
                    },
                    "content": {
                        "application/json": { "schema": { "$ref": "#/components/schemas/Error" } }
                    }
                },
                "WrongHost": error("The request's `Host` is not `127.0.0.1:<port>` or `localhost:<port>`.")
            },
            "schemas": components
        }
    })
}

/// The spec as the file holds it: keys in order, two-space JSON and a final newline, the same
/// however `serde_json` was built.
fn text(spec: &Value) -> String {
    let mut text = serde_json::to_string_pretty(&sorted(spec)).expect("the spec is JSON");
    text.push('\n');
    text
}

fn sorted(value: &Value) -> Value {
    match value {
        Value::Object(object) => {
            let mut keys: Vec<&String> = object.keys().collect();
            keys.sort();
            Value::Object(
                keys.into_iter()
                    .map(|key| (key.clone(), sorted(&object[key])))
                    .collect(),
            )
        }
        Value::Array(items) => Value::Array(items.iter().map(sorted).collect()),
        other => other.clone(),
    }
}

#[test]
fn the_committed_spec_is_the_api() {
    let wanted = text(&spec());
    let path = committed();
    if std::env::var_os("QUOTADIAL_WRITE_OPENAPI").is_some() {
        std::fs::write(&path, &wanted).expect("writes docs/openapi.json");
    }
    let found = std::fs::read_to_string(&path).unwrap_or_default();
    assert!(
        found == wanted,
        "docs/openapi.json is not the API the code describes; write it again with \
         QUOTADIAL_WRITE_OPENAPI=1 cargo test -p quotadial-api openapi"
    );
}

#[test]
fn descriptions_leave_out_what_only_the_code_needs() {
    assert_eq!(
        readable("What the status pages say (D-096). Left out\nof a direct fetch."),
        "What the status pages say. Left out of a direct fetch."
    );
    assert_eq!(readable("One.\n\nTwo,\nthree."), "One.\n\nTwo, three.");
    assert_eq!(
        readable("Where it was sent (D-094, P-016)."),
        "Where it was sent."
    );
    assert_eq!(
        readable("One model's part of a [`Cost`]."),
        "One model's part of a `Cost`."
    );
    // Brackets that hold words stay.
    assert_eq!(
        readable("Its share (from 0 to 100)."),
        "Its share (from 0 to 100)."
    );
}

#[test]
fn every_route_is_described_and_every_reference_resolves() {
    let spec = spec();
    let operations: Vec<String> = spec["paths"]
        .as_object()
        .expect("paths")
        .iter()
        .flat_map(|(path, methods)| {
            methods
                .as_object()
                .expect("methods")
                .keys()
                .map(move |method| format!("{} {path}", method.to_uppercase()))
        })
        .collect();
    assert_eq!(
        operations,
        [
            "GET /v1/alerts",
            "POST /v1/alerts",
            "DELETE /v1/alerts/{id}",
            "PATCH /v1/alerts/{id}",
            "GET /v1/headroom",
            "GET /v1/history",
            "GET /v1/openapi.json",
            "GET /v1/status",
        ]
    );
    let mut references = Vec::new();
    collect_references(&spec, &mut references);
    for reference in references {
        let pointer = reference.trim_start_matches('#');
        assert!(
            spec.pointer(pointer).is_some(),
            "{reference} is referred to but not described"
        );
    }
}

#[test]
fn every_description_is_about_the_api_not_the_code() {
    let mut descriptions = Vec::new();
    collect_descriptions(&spec(), &mut descriptions);
    for description in descriptions {
        assert!(
            !["::", "PRD", "#4", "#5", "](", "\n"]
                .iter()
                .any(|code| description.split("\n\n").any(|part| part.contains(code))),
            "this description speaks of the code; move that part to a // comment: {description}"
        );
    }
}

#[test]
fn descriptions_use_quotadials_words() {
    // #52: one term for one thing, and never "quota", "auth" or "credential" in anything a user
    // reads. A name in backticks or quotes, such as a Keychain item's, stays as it is.
    let mut descriptions = Vec::new();
    collect_descriptions(&spec(), &mut descriptions);
    for description in descriptions {
        assert_eq!(
            quotadial_core::text::unplain_words(&description),
            Vec::<String>::new(),
            "say a window, a reset or a sign-in in their place: {description}"
        );
    }
}

fn collect_descriptions(value: &Value, found: &mut Vec<String>) {
    match value {
        Value::Object(object) => {
            if let Some(Value::String(description)) = object.get("description") {
                found.push(description.clone());
            }
            object
                .values()
                .for_each(|value| collect_descriptions(value, found));
        }
        Value::Array(items) => items
            .iter()
            .for_each(|value| collect_descriptions(value, found)),
        _ => {}
    }
}

fn collect_references(value: &Value, found: &mut Vec<String>) {
    match value {
        Value::Object(object) => {
            if let Some(Value::String(reference)) = object.get("$ref") {
                found.push(reference.clone());
            }
            object
                .values()
                .for_each(|value| collect_references(value, found));
        }
        Value::Array(items) => items
            .iter()
            .for_each(|value| collect_references(value, found)),
        _ => {}
    }
}
