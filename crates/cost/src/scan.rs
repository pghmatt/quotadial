//! A scan: list each provider's logs written in the last week, read what each gained since the
//! last scan, keep its requests, and add up the week at the bundled prices.

use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use chrono::{DateTime, TimeDelta, Utc};
use quotadial_core::text::plural;
use quotadial_core::{Cost, ModelCost, Tokens};

use crate::read::{self, MAX_LINE};
use crate::store::{FileState, Store};
use crate::{CostError, Price, Prices, Request, claude, codex};

/// The period a cost covers, up to its scan.
pub const WEEK: TimeDelta = TimeDelta::days(7);
/// How long requests are kept: a day past the week, so a late scan still has all of its week.
pub const KEEP: TimeDelta = TimeDelta::days(8);

/// Logs are looked for no deeper than this below a log folder. The deepest found are a workflow's
/// subagents, at `projects/<project>/<session>/subagents/workflows/<workflow>/`; Codex's sessions
/// are at `sessions/YYYY/MM/DD/`.
const DEPTH: usize = 8;
/// How much of a log is read before what was found is kept, so a long first reading keeps its
/// progress and its memory stays small.
const BUDGET: u64 = 256 * 1024 * 1024;

/// The providers whose logs QuotaDial reads, and how.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Claude,
    Codex,
}

impl Kind {
    fn of(provider: &str) -> Option<Self> {
        match provider {
            "claude" => Some(Self::Claude),
            "codex" => Some(Self::Codex),
            _ => None,
        }
    }

    /// Whether a file named `name` is one of the provider's logs.
    fn is_log(self, name: &str) -> bool {
        name.ends_with(".jsonl") && (self == Self::Claude || name.starts_with("rollout-"))
    }
}

/// Reads the providers' logs into `costs.db` and adds up their cost.
pub struct CostScan {
    store: Store,
    prices: Prices,
    claude: claude::Reader,
    codex: codex::Reader,
}

impl std::fmt::Debug for CostScan {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CostScan")
            .field("prices", &self.prices.updated)
            .finish_non_exhaustive()
    }
}

impl CostScan {
    /// A scan keeping what it reads in `path`, pricing with the table bundled in this release.
    ///
    /// # Errors
    ///
    /// When the database cannot be opened or made.
    pub fn open(path: &Path) -> Result<Self, CostError> {
        Self::with_prices(path, Prices::bundled())
    }

    /// A scan keeping what it reads in `path`, pricing with `prices`.
    ///
    /// # Errors
    ///
    /// When the database cannot be opened or made.
    pub fn with_prices(path: &Path, prices: Prices) -> Result<Self, CostError> {
        Ok(Self {
            store: Store::open(path)?,
            prices,
            claude: claude::Reader::new(),
            codex: codex::Reader::new(),
        })
    }

    /// Reads what the logs in `folders`, each provider's log folders as `(provider, folders)`,
    /// gained since the last scan, and returns each provider's cost over the week to `now`. A
    /// folder reached by several paths, such as through links, is read once. A provider none of
    /// whose folders exist gets no cost; a log that cannot be read is tried again next time.
    ///
    /// # Errors
    ///
    /// When `costs.db` cannot be read or written.
    pub fn scan(
        &self,
        folders: &[(String, Vec<PathBuf>)],
        now: DateTime<Utc>,
    ) -> Result<Vec<Cost>, CostError> {
        let since = now - WEEK;
        let mut unread = self.store.files()?;
        let mut walked = HashSet::new();
        let mut listed = HashSet::new();
        let mut scanned = Vec::new();
        let mut unrecorded = 0;
        for (provider, dirs) in folders {
            let Some(kind) = Kind::of(provider) else {
                continue;
            };
            let mut found = Vec::new();
            for dir in dirs {
                let Ok(dir) = fs::canonicalize(dir) else {
                    continue;
                };
                if walked.insert(dir.clone()) {
                    logs(&dir, kind, SystemTime::from(since), DEPTH, &mut found);
                }
                if !scanned.contains(provider) {
                    scanned.push(provider.clone());
                }
            }
            for (path, len) in found {
                let key = path.to_string_lossy().into_owned();
                if !listed.insert(key.clone()) {
                    continue;
                }
                let log = Log {
                    kind,
                    provider,
                    path: &path,
                    key: &key,
                    len,
                };
                let state = self.read(&log, unread.remove(&key), now)?;
                if kind == Kind::Codex && session(&state).unrecorded() {
                    unrecorded += 1;
                }
            }
        }
        let gone: Vec<&str> = unread.keys().map(String::as_str).collect();
        self.store.forget(&gone)?;
        self.store.prune(now - KEEP)?;
        self.costs(&scanned, since, now, unrecorded)
    }

    /// Reads what `log` gained since `before`, keeps it, and returns how far it was read. A log
    /// shorter than what was read of it was written again, and is read from its start.
    fn read(
        &self,
        log: &Log<'_>,
        before: Option<FileState>,
        now: DateTime<Utc>,
    ) -> Result<FileState, CostError> {
        let Log {
            kind,
            provider,
            path,
            key,
            len,
        } = *log;
        let before = before.unwrap_or_default();
        if before.len == len {
            return Ok(before);
        }
        let mut state = if before.read_to <= len {
            before
        } else {
            FileState::default()
        };
        let Ok(mut file) = File::open(path) else {
            return Ok(state);
        };
        loop {
            let mut current = session(&state);
            let mut requests: Vec<(u64, Request)> = Vec::new();
            let Ok(stop) = read::lines(
                &mut file,
                state.read_to,
                BUDGET,
                MAX_LINE,
                |offset, line| {
                    let request = match kind {
                        Kind::Claude => self.claude.request(line),
                        Kind::Codex => self.codex.request(&mut current, line),
                    };
                    if let Some(request) = request {
                        requests.push((offset, request));
                    }
                },
            ) else {
                return Ok(state);
            };
            state = FileState {
                len: if stop.end {
                    len.max(stop.at)
                } else {
                    state.len
                },
                read_to: stop.at,
                model: current.model,
                records: current.records,
                token_counts: current.token_counts,
            };
            self.store
                .record(provider, key, &state, &requests, now - KEEP)?;
            if stop.end {
                return Ok(state);
            }
        }
    }

    /// Each provider's cost from `since` to `until`, from the requests kept.
    fn costs(
        &self,
        providers: &[String],
        since: DateTime<Utc>,
        until: DateTime<Utc>,
        unrecorded: u64,
    ) -> Result<Vec<Cost>, CostError> {
        let mut found: HashMap<String, HashMap<String, Tally>> = HashMap::new();
        self.store.requests(since, until, |stored| {
            if !found.contains_key(stored.provider) {
                found.insert(stored.provider.to_owned(), HashMap::new());
            }
            let Some(models) = found.get_mut(stored.provider) else {
                return;
            };
            if !models.contains_key(stored.model) {
                let price = self.prices.of(stored.provider, stored.model).cloned();
                models.insert(stored.model.to_owned(), Tally::new(price));
            }
            if let Some(tally) = models.get_mut(stored.model) {
                tally.add(stored.tokens);
            }
        })?;
        Ok(providers
            .iter()
            .map(|provider| {
                let mut models: Vec<(String, Tally)> = found
                    .remove(provider)
                    .unwrap_or_default()
                    .into_iter()
                    .collect();
                models.sort_by(|(a_model, a), (b_model, b)| {
                    b.usd()
                        .unwrap_or(-1.0)
                        .total_cmp(&a.usd().unwrap_or(-1.0))
                        .then_with(|| b.tokens.total().cmp(&a.tokens.total()))
                        .then_with(|| a_model.cmp(b_model))
                });
                let mut notes = Vec::new();
                let mut cost = Cost {
                    provider: provider.clone(),
                    since,
                    until,
                    usd: 0.0,
                    requests: 0,
                    tokens: Tokens::default(),
                    models: Vec::new(),
                    prices_updated: self.prices.updated.clone(),
                    notes: Vec::new(),
                };
                for (model, tally) in models {
                    cost.usd += tally.usd().unwrap_or(0.0);
                    cost.requests += tally.requests;
                    cost.tokens += tally.tokens;
                    if tally.usd().is_none() {
                        let (count, are) = (
                            plural(tally.requests, "request"),
                            if tally.requests == 1 { "is" } else { "are" },
                        );
                        notes.push(if model == codex::NO_MODEL {
                            format!("{count} named no model, so {} left out.", it_or_they(tally.requests))
                        } else {
                            format!(
                                "The price table of {} has no price for {model}, so its {count} {are} left out.",
                                self.prices.updated,
                            )
                        });
                    }
                    cost.models.push(ModelCost {
                        model,
                        usd: tally.usd(),
                        requests: tally.requests,
                        tokens: tally.tokens,
                    });
                }
                if provider == "codex" && unrecorded > 0 {
                    notes.push(format!(
                        "{} from an older Codex CLI recorded no usage per request, so {} left out.",
                        plural(unrecorded, "session"),
                        it_or_they(unrecorded),
                    ));
                }
                cost.notes = notes;
                cost
            })
            .collect())
    }
}

/// A log to read: whose it is, where it is, its key in `costs.db`, and its length when listed.
#[derive(Clone, Copy)]
struct Log<'a> {
    kind: Kind,
    provider: &'a str,
    path: &'a Path,
    key: &'a str,
    len: u64,
}

/// A model's requests so far.
struct Tally {
    price: Option<Price>,
    requests: u64,
    tokens: Tokens,
    usd: f64,
}

impl Tally {
    fn new(price: Option<Price>) -> Self {
        Self {
            price,
            requests: 0,
            tokens: Tokens::default(),
            usd: 0.0,
        }
    }

    /// Counts a request, priced on its own: a large context costs more for that request alone.
    fn add(&mut self, tokens: Tokens) {
        self.requests += 1;
        self.tokens += tokens;
        if let Some(price) = &self.price {
            self.usd += price.of(tokens);
        }
    }

    fn usd(&self) -> Option<f64> {
        self.price.as_ref().map(|_| self.usd)
    }
}

/// What a Codex session's reading has shown, from how far it was read.
fn session(state: &FileState) -> codex::Session {
    codex::Session {
        model: state.model.clone(),
        records: state.records,
        token_counts: state.token_counts,
    }
}

/// Adds each of `kind`'s logs below `dir` written since `since` to `found`, with its length. Links
/// are not followed below the folder itself, so a link cannot lead the walk in a circle.
fn logs(dir: &Path, kind: Kind, since: SystemTime, depth: usize, found: &mut Vec<(PathBuf, u64)>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_dir() {
            if depth > 0 {
                logs(&entry.path(), kind, since, depth - 1, found);
            }
        } else if file_type.is_file()
            && entry
                .file_name()
                .to_str()
                .is_some_and(|name| kind.is_log(name))
            && let Ok(metadata) = entry.metadata()
            && metadata.modified().is_ok_and(|modified| modified >= since)
        {
            found.push((entry.path(), metadata.len()));
        }
    }
}

/// `it is` for one, `they are` for more.
fn it_or_they(count: u64) -> &'static str {
    if count == 1 { "it is" } else { "they are" }
}

#[cfg(test)]
mod tests {
    use std::fs::OpenOptions;
    use std::io::Write;

    use super::*;

    /// A price table with round prices, so each cost below can be worked out by hand.
    const PRICES: &str = r#"{
        "updated": "2026-10-23",
        "anthropic": {
            "claude-opus-5-5": {"input": 4, "output": 20, "cache_read": 0.5, "cache_write": 5}
        },
        "openai": {
            "gpt-6-astra": {"input": 10, "output": 50, "cache_read": 1,
                            "above": {"tokens": 272000, "input": 20, "output": 75, "cache_read": 2}}
        }
    }"#;

    fn at(text: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(text)
            .expect("valid timestamp")
            .with_timezone(&Utc)
    }

    /// A Claude Code assistant line, with made-up ids and content.
    fn claude_line(id: &str, time: &str, input: u64, output: u64) -> String {
        format!(
            r#"{{"sessionId":"s-1","message":{{"id":"{id}","role":"assistant","model":"claude-opus-5-5","content":[{{"type":"text","text":"ok"}}],"stop_reason":"end_turn","usage":{{"input_tokens":{input},"output_tokens":{output},"cache_read_input_tokens":0,"cache_creation_input_tokens":0}}}},"requestId":"req-{id}","type":"assistant","timestamp":"{time}"}}"#
        )
    }

    fn codex_context(model: &str) -> String {
        format!(
            r#"{{"timestamp":"2026-10-23T09:00:00.000Z","type":"turn_context","payload":{{"turn_id":"t-1","model":"{model}"}}}}"#
        )
    }

    fn codex_record(id: &str, input: u64, cached: u64, output: u64) -> String {
        format!(
            r#"{{"timestamp":"2026-10-23T09:00:05.000Z","type":"token_usage_record","payload":{{"thread_id":"th-1","response_id":"{id}","usage":{{"input_tokens":{input},"cached_input_tokens":{cached},"output_tokens":{output},"reasoning_output_tokens":0}}}}}}"#
        )
    }

    const CODEX_COUNT: &str = r#"{"timestamp":"2026-10-23T09:00:05.000Z","type":"event_msg","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":5000,"output_tokens":100}}}}"#;

    fn write(path: &Path, lines: &[String]) {
        fs::create_dir_all(path.parent().expect("a folder")).expect("makes the folder");
        fs::write(
            path,
            lines
                .iter()
                .map(|line| format!("{line}\n"))
                .collect::<String>(),
        )
        .expect("writes");
    }

    fn append(path: &Path, text: &str) {
        OpenOptions::new()
            .append(true)
            .open(path)
            .expect("opens")
            .write_all(text.as_bytes())
            .expect("appends");
    }

    fn scan(dir: &Path) -> CostScan {
        CostScan::with_prices(
            &dir.join("costs.db"),
            Prices::parse(PRICES).expect("a valid table"),
        )
        .expect("opens")
    }

    fn near(actual: f64, expected: f64) -> bool {
        (actual - expected).abs() < 1e-9
    }

    #[test]
    fn a_week_of_claude_logs_counts_each_request_once() {
        let temp = tempfile::tempdir().expect("temp dir");
        let claude = temp.path().join("home/.claude");
        let session = claude.join("projects/-work/s-1.jsonl");
        write(
            &session,
            &[
                // A response streamed in two parts, and a second response.
                claude_line("m1", "2026-10-23T10:00:00.000Z", 1_000_000, 10_000),
                claude_line("m1", "2026-10-23T10:00:01.000Z", 1_000_000, 50_000),
                claude_line("m2", "2026-10-23T11:00:00.000Z", 500_000, 0),
                r#"{"type":"user","message":{"role":"user","content":"hello"}}"#.to_owned(),
            ],
        );
        // Subagents' logs, one of them a workflow's, and a resumed session that copied the first
        // response.
        write(
            &claude.join("projects/-work/s-1/subagents/agent-a.jsonl"),
            &[claude_line("m3", "2026-10-23T10:30:00.000Z", 0, 100_000)],
        );
        write(
            &claude.join("projects/-work/s-1/subagents/workflows/wf_1/agent-b.jsonl"),
            &[claude_line("m4", "2026-10-23T10:40:00.000Z", 0, 50_000)],
        );
        write(
            &claude.join("projects/-work/s-2.jsonl"),
            &[claude_line(
                "m1",
                "2026-10-23T10:00:01.000Z",
                1_000_000,
                50_000,
            )],
        );
        // An account folder whose projects link to the same folder, as Codus keeps them.
        let account = temp.path().join("home/.codus/accounts/.pending-1");
        fs::create_dir_all(&account).expect("makes the folder");
        #[cfg(unix)]
        std::os::unix::fs::symlink(claude.join("projects"), account.join("projects"))
            .expect("links");
        let folders = vec![(
            "claude".to_owned(),
            vec![claude.join("projects"), account.join("projects")],
        )];

        let scanner = scan(temp.path());
        let costs = scanner
            .scan(&folders, at("2026-10-24T00:00:00Z"))
            .expect("scans");

        assert_eq!(
            scanner.store.files().expect("reads").len(),
            4,
            "the linked folder's logs are read once"
        );
        assert_eq!(costs.len(), 1);
        let cost = &costs[0];
        assert_eq!(cost.provider, "claude");
        assert_eq!(cost.requests, 4, "m1 once, m2, m3 and m4");
        assert_eq!(cost.tokens.input, 1_500_000);
        assert_eq!(cost.tokens.output, 200_000);
        // m1: 4 + 1; m2: 2; m3: 2; m4: 1.
        assert!(near(cost.usd, 10.0), "{}", cost.usd);
        assert_eq!(cost.since, at("2026-10-17T00:00:00Z"));
        assert_eq!(cost.until, at("2026-10-24T00:00:00Z"));
        assert_eq!(cost.prices_updated, "2026-10-23");
        assert_eq!(cost.models.len(), 1);
        assert_eq!(cost.models[0].model, "claude-opus-5-5");
        assert!(cost.notes.is_empty(), "{:?}", cost.notes);
    }

    #[test]
    fn codex_records_are_priced_by_the_turn_s_model_and_a_large_context_costs_more() {
        let temp = tempfile::tempdir().expect("temp dir");
        let codex = temp.path().join("home/.codex");
        write(
            &codex.join("sessions/2026/10/23/rollout-2026-10-23T09-00-00-a.jsonl"),
            &[
                r#"{"timestamp":"2026-10-23T09:00:00.000Z","type":"session_meta","payload":{"id":"a"}}"#.to_owned(),
                codex_record("before-any-turn", 1000, 0, 0),
                codex_context("gpt-6-astra"),
                // 100k fresh and 100k cached: 1 + 0.1 + 0.5.
                codex_record("r1", 200_000, 100_000, 10_000),
                // 300k is over 272k, all at the higher price: 1 + 0.5 + 0.75.
                codex_record("r2", 300_000, 250_000, 10_000),
                CODEX_COUNT.to_owned(),
            ],
        );
        // A session from an older CLI, with running totals only.
        write(
            &codex.join("archived_sessions/rollout-2026-10-20T09-00-00-b.jsonl"),
            &[CODEX_COUNT.to_owned(), CODEX_COUNT.to_owned()],
        );
        let folders = vec![(
            "codex".to_owned(),
            vec![codex.join("sessions"), codex.join("archived_sessions")],
        )];

        let costs = scan(temp.path())
            .scan(&folders, at("2026-10-24T00:00:00Z"))
            .expect("scans");

        let cost = &costs[0];
        assert_eq!(cost.provider, "codex");
        assert_eq!(cost.requests, 3);
        assert!(near(cost.usd, 1.6 + 2.25), "{}", cost.usd);
        assert_eq!(cost.models[0].model, "gpt-6-astra");
        assert_eq!(cost.models[1].model, codex::NO_MODEL);
        assert_eq!(cost.models[1].usd, None);
        assert_eq!(
            cost.notes,
            [
                "1 request named no model, so it is left out.",
                "1 session from an older Codex CLI recorded no usage per request, so it is left out.",
            ]
        );
    }

    #[test]
    fn a_log_is_read_once_and_then_only_what_it_gained() {
        let temp = tempfile::tempdir().expect("temp dir");
        let projects = temp.path().join(".claude/projects");
        let session = projects.join("-work/s-1.jsonl");
        write(
            &session,
            &[claude_line("m1", "2026-10-23T10:00:00.000Z", 1_000_000, 0)],
        );
        // A line still being written waits for its end.
        append(
            &session,
            &claude_line("m2", "2026-10-23T10:05:00.000Z", 1_000_000, 0),
        );
        let folders = vec![("claude".to_owned(), vec![projects.clone()])];
        let costs = scan(temp.path());
        let now = at("2026-10-24T00:00:00Z");

        assert_eq!(costs.scan(&folders, now).expect("scans")[0].requests, 1);
        let read_to = costs.store.files().expect("reads")[&fs::canonicalize(&session)
            .expect("exists")
            .to_string_lossy()
            .into_owned()]
            .read_to;
        assert!(read_to > 0);

        append(&session, "\n");
        append(
            &session,
            &format!(
                "{}\n",
                claude_line("m3", "2026-10-23T10:10:00.000Z", 1_000_000, 0)
            ),
        );
        let cost = &costs.scan(&folders, now).expect("scans")[0];
        assert_eq!(cost.requests, 3);
        assert!(near(cost.usd, 12.0));

        // Written again, shorter: read again from the start, so its new request counts, and the
        // ones it held before still count once.
        write(
            &session,
            &[
                claude_line("m1", "2026-10-23T10:00:00.000Z", 1_000_000, 0),
                claude_line("m4", "2026-10-23T10:15:00.000Z", 1_000_000, 0),
            ],
        );
        assert_eq!(costs.scan(&folders, now).expect("scans")[0].requests, 4);
    }

    #[test]
    fn only_the_week_counts() {
        let temp = tempfile::tempdir().expect("temp dir");
        let projects = temp.path().join(".claude/projects");
        let old = projects.join("-work/old.jsonl");
        write(
            &old,
            &[claude_line("m1", "2026-10-01T10:00:00.000Z", 1_000_000, 0)],
        );
        File::options()
            .write(true)
            .open(&old)
            .expect("opens")
            .set_modified(SystemTime::from(at("2026-10-01T10:00:00Z")))
            .expect("dates it");
        write(
            &projects.join("-work/new.jsonl"),
            &[
                claude_line("m2", "2026-10-10T10:00:00.000Z", 1_000_000, 0),
                // Kept, as it is within a day of the week, but not counted.
                claude_line("m4", "2026-10-16T12:00:00.000Z", 1_000_000, 0),
                claude_line("m3", "2026-10-23T10:00:00.000Z", 1_000_000, 0),
            ],
        );
        let folders = vec![
            ("claude".to_owned(), vec![projects.clone()]),
            (
                "codex".to_owned(),
                vec![temp.path().join(".codex/sessions")],
            ),
            ("gemini".to_owned(), vec![projects]),
        ];

        let costs = scan(temp.path())
            .scan(&folders, at("2026-10-24T00:00:00Z"))
            .expect("scans");

        assert_eq!(
            costs.len(),
            1,
            "no folder for codex; gemini's logs are not read"
        );
        assert_eq!(costs[0].requests, 1, "m3 alone is in the week");
    }

    #[test]
    fn a_week_with_nothing_in_it_costs_nothing() {
        let temp = tempfile::tempdir().expect("temp dir");
        let sessions = temp.path().join(".codex/sessions");
        fs::create_dir_all(&sessions).expect("makes the folder");

        let costs = scan(temp.path())
            .scan(
                &[("codex".to_owned(), vec![sessions])],
                at("2026-10-24T00:00:00Z"),
            )
            .expect("scans");

        assert_eq!(costs.len(), 1);
        assert_eq!(costs[0].requests, 0);
        assert!(near(costs[0].usd, 0.0));
        assert!(costs[0].models.is_empty());
    }
}
