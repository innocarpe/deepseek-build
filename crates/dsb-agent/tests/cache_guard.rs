//! Spec 10 §1.9 — cache regression bench (`cache_guard_*`, spec 10 §4.4).
//!
//! A scenario is a scripted turn sequence driven through the real agent turn
//! loop against a **prefix-accounting mock provider**. The mock derives
//! `prompt_cache_hit_tokens` / `prompt_cache_miss_tokens` from the requests it
//! receives — the leading messages this request shares with the previous one —
//! never from constants in a test. That derivation is what these tests score:
//! a mock that answers a fixed ratio fails `cache_guard_negative_control`.
//!
//! Spec 10 §1.9 scores two layers over a scenario:
//!
//! 1. exact — the number of distinct epochs is exactly 1 for a scenario that
//!    changes no §1.1 input, exactly 2 for one that changes a single input by
//!    design. Each rebuild is a later process resuming the conversation, which
//!    is the shape a real prefix move takes (spec 10 §1.5.1 rule 5).
//! 2. rate — `hit * 100 / (hit + miss)` per provider request, averaged over the
//!    last three requests, at or above `DSB_CACHE_GUARD_THRESHOLD` (default 90).
//!    The scored unit is one request, tool rounds included: the first request
//!    of a process legitimately carries nothing, which is the "turn 1 has no
//!    cache" floor the threshold is documented against.
//!
//! `cache_guard_negative_control` keeps the rate layer honest: a scenario whose
//! §1.1 input is perturbed before every turn must land **below** the threshold.
//! If it passes, the mock is not accounting from the request and the bench
//! fails — not the scenario (§1.9).
//!
//! The full scenario matrix is release-gated: it runs only when
//! `DSB_RELEASE_CACHE_GUARD=1` (wrapper: `scripts/cache-guard.sh`), so the
//! ordinary `cargo test --workspace` stays fast. The contract tests above run
//! always, and are what the mutation evidence in the PR exercised.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use dsb_agent::{Agent, AgentConfig, TurnEvent};
use dsb_context::{DEFAULT_SYSTEM_PROMPT, SkillIndexEntry};
use dsb_provider_deepseek::{
    ChatMessage, ChatRequest, ChatRequestBuilder, Client, ClientConfig, ModelId, ToolDefinition,
};
use serde_json::{Value, json};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

/// Bytes the mock charges per token. A fixed ratio is deliberate: the bench
/// scores a ratio between carried and fresh bytes, so the constant cancels.
const BYTES_PER_TOKEN: usize = 4;
/// Spec 10 §1.9 threshold — Reasonix's 90, kept deliberately for comparability.
const DEFAULT_THRESHOLD: u64 = 90;
/// The window the rate layer averages over (spec 10 §1.9: last 3 turns).
const TAIL_WINDOW: usize = 3;
/// Model id the mock answers with; the bench scores the curve, not routing.
const WIRE_MODEL: &str = "deepseek-v4-flash";
/// File the mock's `read` tool calls point at, inside the scenario workspace.
const NOTES_FILE: &str = "notes.txt";
const NOTES_BODY: &str = "\
alpha: the bench keeps this file small and deterministic.
beta: a tool result grows the volatile tail by a known amount.
gamma: the stable prefix above it must stay warm.
";

// ---------------------------------------------------------------------------
// Prefix-accounting mock provider
// ---------------------------------------------------------------------------

/// One recorded provider request with the accounting derived from its bytes.
#[derive(Debug, Clone)]
struct ReqRecord {
    /// Sum of the canonical message lengths — the prompt in mock bytes.
    total_bytes: usize,
    /// Leading messages carried over from the previous request, in mock bytes.
    hit_bytes: usize,
    /// Hash of the leading (stable prefix) message as it reached the mock —
    /// the wire's own view of how many distinct prefixes the scenario used.
    prefix_fingerprint: u64,
}

impl ReqRecord {
    fn prompt_tokens(&self) -> u64 {
        (self.total_bytes / BYTES_PER_TOKEN).max(1) as u64
    }

    fn hit_tokens(&self) -> u64 {
        (self.hit_bytes / BYTES_PER_TOKEN) as u64
    }

    fn miss_tokens(&self) -> u64 {
        self.prompt_tokens() - self.hit_tokens()
    }

    /// `hit * 100 / (hit + miss)` (spec 10 §1.9), integer division.
    fn rate(&self) -> u64 {
        100 * self.hit_tokens() / self.prompt_tokens()
    }
}

#[derive(Default)]
struct MockState {
    /// The previous request's canonical messages — the accounting baseline.
    previous_messages: Option<Vec<Vec<u8>>>,
    records: Vec<ReqRecord>,
    /// Tool-call rounds the mock still owes before a final answer.
    tool_rounds_left: u32,
    tool_call_seq: u32,
    /// Chain-of-thought echoed every turn (the reasoning round-trip).
    reasoning: Option<String>,
}

/// The accounting that makes this mock a mock of a *cache*: how much of this
/// request the previous request already carried, measured message by message.
/// The provider caches a token prefix, so the first message that differs is
/// where the carry stops; the negative control is what proves the boundaries
/// are real.
fn account(state: &mut MockState, body: &Value) -> Option<ReqRecord> {
    let messages = body.get("messages")?.as_array()?;
    let canonical: Vec<Vec<u8>> = messages
        .iter()
        .map(|message| serde_json::to_vec(message).ok())
        .collect::<Option<Vec<_>>>()?;
    let carried = match &state.previous_messages {
        Some(previous) => previous
            .iter()
            .zip(&canonical)
            .take_while(|(a, b)| a == b)
            .count(),
        None => 0,
    };
    let hit_bytes = canonical[..carried].iter().map(Vec::len).sum();
    let total_bytes = canonical.iter().map(Vec::len).sum();
    let mut hasher = DefaultHasher::new();
    canonical.first()?.hash(&mut hasher);
    let record = ReqRecord {
        total_bytes,
        hit_bytes,
        prefix_fingerprint: hasher.finish(),
    };
    state.previous_messages = Some(canonical);
    state.records.push(record.clone());
    Some(record)
}

struct BenchMock {
    state: Arc<Mutex<MockState>>,
}

impl Respond for BenchMock {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        let Ok(body) = serde_json::from_slice::<Value>(&request.body) else {
            return error_response("cache_guard mock: request body is not JSON");
        };
        let mut state = self.state.lock().expect("mock state");
        let Some(record) = account(&mut state, &body) else {
            return error_response("cache_guard mock: request carries no messages array");
        };
        let streaming = body.get("stream").and_then(Value::as_bool).unwrap_or(false);
        let reasoning = state.reasoning.clone();
        let read_call = if state.tool_rounds_left > 0 {
            state.tool_rounds_left -= 1;
            state.tool_call_seq += 1;
            Some(state.tool_call_seq)
        } else {
            None
        };
        drop(state);
        if streaming {
            sse_response(&record, reasoning.as_deref(), read_call)
        } else {
            json_response(&record, read_call)
        }
    }
}

fn error_response(message: &str) -> ResponseTemplate {
    ResponseTemplate::new(500).set_body_string(message.to_string())
}

fn read_arguments() -> String {
    json!({ "path": NOTES_FILE }).to_string()
}

fn usage_json(record: &ReqRecord) -> Value {
    let prompt = record.prompt_tokens();
    let completion = 8;
    json!({
        "prompt_tokens": prompt,
        "completion_tokens": completion,
        "total_tokens": prompt + completion,
        "prompt_cache_hit_tokens": record.hit_tokens(),
        "prompt_cache_miss_tokens": record.miss_tokens(),
    })
}

fn sse_response(
    record: &ReqRecord,
    reasoning: Option<&str>,
    read_call: Option<u32>,
) -> ResponseTemplate {
    let mut chunks: Vec<Value> = Vec::new();
    if let Some(text) = reasoning {
        chunks.push(json!({
            "model": WIRE_MODEL,
            "choices": [{ "delta": { "reasoning_content": text } }],
        }));
    }
    match read_call {
        Some(seq) => {
            chunks.push(json!({
                "choices": [{ "delta": { "tool_calls": [{
                    "index": 0,
                    "id": format!("call_{seq}"),
                    "type": "function",
                    "function": { "name": "read", "arguments": read_arguments() },
                }] } }],
            }));
            chunks.push(json!({
                "choices": [{ "delta": {}, "finish_reason": "tool_calls" }],
            }));
        }
        None => chunks.push(json!({
            "choices": [{ "delta": { "content": "Done." }, "finish_reason": "stop" }],
        })),
    }
    chunks.push(json!({ "usage": usage_json(record) }));

    let mut body = String::new();
    for chunk in &chunks {
        body.push_str("data: ");
        body.push_str(&chunk.to_string());
        body.push_str("\n\n");
    }
    body.push_str("data: [DONE]\n\n");
    ResponseTemplate::new(200)
        .insert_header("content-type", "text/event-stream")
        .set_body_string(body)
}

fn json_response(record: &ReqRecord, read_call: Option<u32>) -> ResponseTemplate {
    let (message, finish) = match read_call {
        Some(seq) => (
            json!({
                "role": "assistant",
                "tool_calls": [{
                    "id": format!("call_{seq}"),
                    "type": "function",
                    "function": { "name": "read", "arguments": read_arguments() },
                }],
            }),
            "tool_calls",
        ),
        None => (json!({ "role": "assistant", "content": "Done." }), "stop"),
    };
    ResponseTemplate::new(200).set_body_json(json!({
        "model": WIRE_MODEL,
        "choices": [{ "message": message, "finish_reason": finish }],
        "usage": usage_json(record),
    }))
}

async fn start_mock(state: Arc<Mutex<MockState>>) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(BenchMock { state })
        .mount(&server)
        .await;
    server
}

// ---------------------------------------------------------------------------
// Scenarios
// ---------------------------------------------------------------------------

/// A change to one §1.1 input, applied when a later process rebuilds the
/// prefix — the resume shape a real prefix move takes (§1.5.1 rule 5).
#[derive(Debug, Clone)]
enum Mutation {
    /// §1.1 item 3 — a skill enters the index.
    AddSkill(String),
    /// §1.1 item 5 — standing project instructions change (AGENTS.md).
    ChangeInstructions(String),
}

#[derive(Debug, Clone)]
struct Turn {
    text: String,
    /// Applied before this turn: rebuild the prefix with this change.
    mutation: Option<Mutation>,
    /// Tool rounds the mock answers before a final answer for this turn.
    tool_rounds: u32,
}

#[derive(Debug, Clone, Copy)]
enum Expect {
    /// Both layers must pass, with exactly this many distinct epochs.
    Pass { epochs: usize },
    /// The rate layer must **fail** — the negative control (§1.9).
    LandBelowThreshold,
}

#[derive(Debug, Clone)]
struct Scenario {
    name: &'static str,
    reasoning: Option<&'static str>,
    turns: Vec<Turn>,
    expect: Expect,
}

/// ~250 chars of chain-of-thought, round-tripped on every assistant turn.
const REASONING: &str = "Let me weigh the constraints first: the visible answer must stay \
short, but the chain-of-thought is replayed on later turns, so it is part of the volatile \
tail the next request pays for. I will keep it stable, because a rewrite here would move \
bytes the provider already cached.";

/// One line of dialogue, ~70 bytes.
const DIALOGUE_LINE: &str = "keep the request prefix stable while the conversation grows. ";

fn turn(text: impl Into<String>, tool_rounds: u32) -> Turn {
    Turn {
        text: text.into(),
        mutation: None,
        tool_rounds,
    }
}

fn mutated_turn(text: impl Into<String>, mutation: Mutation, tool_rounds: u32) -> Turn {
    Turn {
        text: text.into(),
        mutation: Some(mutation),
        tool_rounds,
    }
}

/// `turns` user turns that change no §1.1 input: the epoch layer must see a
/// single epoch for the whole scenario.
fn plain_dialogue(name: &'static str, turns: usize, reasoning: bool) -> Scenario {
    Scenario {
        name,
        reasoning: reasoning.then_some(REASONING),
        turns: (1..=turns)
            .map(|i| turn(format!("Turn {i}: {DIALOGUE_LINE}"), 0))
            .collect(),
        expect: Expect::Pass { epochs: 1 },
    }
}

/// One user turn answered with `rounds` tool rounds (the tail grows through
/// the tool loop, one request per round).
fn tool_loop(name: &'static str, rounds: u32, reasoning: bool) -> Scenario {
    Scenario {
        name,
        reasoning: reasoning.then_some(REASONING),
        turns: vec![turn("Read the notes file repeatedly, then answer.", rounds)],
        expect: Expect::Pass { epochs: 1 },
    }
}

/// Mixed message sizes: every third user turn is several times longer than the
/// others, so the fresh-tail share moves without any §1.1 input changing.
fn mixed_message_sizes() -> Scenario {
    let turns = (1..=10)
        .map(|i| {
            let text = if i % 3 == 2 {
                format!("Turn {i}: {}", DIALOGUE_LINE.repeat(6))
            } else {
                format!("Turn {i}: {DIALOGUE_LINE}")
            };
            turn(text, 0)
        })
        .collect();
    Scenario {
        name: "mixed-message-sizes",
        reasoning: Some(REASONING),
        turns,
        expect: Expect::Pass { epochs: 1 },
    }
}

/// A single §1.1 input changed by design part-way through: exactly two epochs,
/// and the tail recovers, because only the turn that rebuilt pays for it.
fn input_change_by_design() -> Scenario {
    let mut turns: Vec<Turn> = (1..=6)
        .map(|i| turn(format!("Turn {i}: {DIALOGUE_LINE}"), 0))
        .collect();
    turns[1] = mutated_turn(
        format!("Turn 2: {DIALOGUE_LINE}"),
        Mutation::ChangeInstructions(
            "Bench workspace instructions: the prefix moved on purpose.\n".to_string(),
        ),
        0,
    );
    Scenario {
        name: "input-change-by-design",
        reasoning: Some(REASONING),
        turns,
        expect: Expect::Pass { epochs: 2 },
    }
}

/// The negative control (§1.9): a §1.1 input is perturbed before **every**
/// turn, so each request starts a new prefix and nothing is ever carried. If
/// this scenario meets the threshold, the mock is not accounting from the
/// request and the bench must fail.
fn negative_control() -> Scenario {
    let mut turns: Vec<Turn> = vec![turn(format!("Turn 1: {DIALOGUE_LINE}"), 0)];
    for i in 2..=6 {
        turns.push(mutated_turn(
            format!("Turn {i}: {DIALOGUE_LINE}"),
            Mutation::AddSkill(format!("noise-{i}")),
            0,
        ));
    }
    Scenario {
        name: "negative-control-perturbed-prefix",
        reasoning: Some(REASONING),
        turns,
        expect: Expect::LandBelowThreshold,
    }
}

/// The release matrix: the scenario flavors spec 10 §1.9 names (plain / long
/// dialogue, mixed sizes, tool loop, long tool loop, with and without the
/// reasoning round-trip) plus the two controls.
fn scenario_matrix() -> Vec<Scenario> {
    vec![
        plain_dialogue("plain-dialogue", 6, true),
        plain_dialogue("plain-dialogue-no-reasoning", 6, false),
        plain_dialogue("long-dialogue", 14, true),
        mixed_message_sizes(),
        tool_loop("tool-loop", 14, true),
        tool_loop("tool-loop-no-reasoning", 14, false),
        tool_loop("long-tool-loop", 24, true),
        tool_loop("long-tool-loop-no-reasoning", 24, false),
        input_change_by_design(),
        negative_control(),
    ]
}

// ---------------------------------------------------------------------------
// Runner
// ---------------------------------------------------------------------------

/// Mutable §1.1 inputs a scenario's rebuilds are applied to.
struct ScenarioInputs {
    workspace: PathBuf,
    system_prompt: String,
    skills: Vec<SkillIndexEntry>,
    tools: Vec<ToolDefinition>,
    instructions: Option<String>,
}

impl ScenarioInputs {
    fn base(workspace: &Path) -> Self {
        Self {
            workspace: workspace.to_path_buf(),
            system_prompt: DEFAULT_SYSTEM_PROMPT.to_string(),
            skills: vec![SkillIndexEntry {
                name: "bench-skill".to_string(),
                description: "A skill the bench keeps in the index.".to_string(),
            }],
            tools: dsb_tools::tool_definitions(),
            instructions: Some(
                "Bench workspace instructions: keep the prefix stable.\n".to_string(),
            ),
        }
    }

    fn apply(&mut self, mutation: &Mutation) {
        match mutation {
            Mutation::AddSkill(name) => self.skills.push(SkillIndexEntry {
                name: name.clone(),
                description: format!("skill {name}"),
            }),
            Mutation::ChangeInstructions(text) => self.instructions = Some(text.clone()),
        }
    }

    fn write_instructions(&self) {
        let path = self.workspace.join("AGENTS.md");
        match &self.instructions {
            Some(text) => std::fs::write(path, text).expect("write AGENTS.md"),
            None => {
                let _ = std::fs::remove_file(path);
            }
        }
    }

    fn agent_config(&self) -> AgentConfig {
        AgentConfig {
            workspace_root: self.workspace.clone(),
            system_prompt: self.system_prompt.clone(),
            tools: self.tools.clone(),
            skills_index: self.skills.clone(),
            max_tool_rounds: 64,
            show_model: false,
            ..AgentConfig::default()
        }
    }
}

/// What one scenario run measured, both layers.
struct Outcome {
    scenario: &'static str,
    records: Vec<ReqRecord>,
    /// Epoch (short) of every prefix build the run used, in order.
    epochs: Vec<String>,
    generations: usize,
    cache_evidence_events: usize,
}

impl Outcome {
    fn requests(&self) -> usize {
        self.records.len()
    }

    fn rates(&self) -> Vec<u64> {
        self.records.iter().map(ReqRecord::rate).collect()
    }

    /// Mean of the last `TAIL_WINDOW` request rates, clamped to what ran —
    /// the same clamp Reasonix's `tailAverage` applies. Every bench scenario
    /// produces at least three requests, so the clamp is a safety net.
    fn tail_rate(&self) -> u64 {
        let rates = self.rates();
        let n = rates.len().min(TAIL_WINDOW);
        if n == 0 {
            return 0;
        }
        rates[rates.len() - n..].iter().sum::<u64>() / n as u64
    }

    fn distinct_epochs(&self) -> Vec<String> {
        distinct_strings(&self.epochs)
    }

    fn distinct_fingerprints(&self) -> Vec<String> {
        let fingerprints: Vec<String> = self
            .records
            .iter()
            .map(|record| record.prefix_fingerprint.to_string())
            .collect();
        distinct_strings(&fingerprints)
    }

    fn report(&self) -> String {
        let rates: Vec<String> = self.rates().iter().map(u64::to_string).collect();
        let last = match self.records.last() {
            Some(record) => format!(
                " last=hit{}/prompt{}",
                record.hit_tokens(),
                record.prompt_tokens()
            ),
            None => String::new(),
        };
        format!(
            "scenario={} requests={} generations={} epochs={} wire_prefixes={} cache_evidence={} tail_rate={}% rates=[{}]{}",
            self.scenario,
            self.requests(),
            self.generations,
            self.distinct_epochs().len(),
            self.distinct_fingerprints().len(),
            self.cache_evidence_events,
            self.tail_rate(),
            rates.join(","),
            last,
        )
    }
}

fn distinct_strings(items: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for item in items {
        if !out.contains(item) {
            out.push(item.clone());
        }
    }
    out
}

async fn run_scenario(scenario: &Scenario) -> Outcome {
    let dir = tempfile::tempdir().expect("scenario workspace");
    std::fs::write(dir.path().join(NOTES_FILE), NOTES_BODY).expect("notes file");

    let state = Arc::new(Mutex::new(MockState {
        reasoning: scenario.reasoning.map(str::to_string),
        ..MockState::default()
    }));
    let server = start_mock(state.clone()).await;
    let client = Arc::new(
        Client::new(ClientConfig::new("cache-guard-key").with_base_url(server.uri()))
            .expect("mock client"),
    );

    let mut inputs = ScenarioInputs::base(dir.path());
    inputs.write_instructions();

    let mut tail: Vec<ChatMessage> = Vec::new();
    let mut epochs: Vec<String> = Vec::new();
    let mut generations = 0usize;
    let mut cache_evidence_events = 0usize;
    let mut agent: Option<Agent> = None;

    for turn in &scenario.turns {
        if let Some(mutation) = &turn.mutation {
            inputs.apply(mutation);
        }
        if agent.is_none() || turn.mutation.is_some() {
            // A rebuild: a later process resuming the conversation.
            inputs.write_instructions();
            let mut fresh = Agent::new(client.clone(), inputs.agent_config()).expect("agent");
            fresh.load_transcript(tail.clone());
            epochs.push(fresh.prefix_epoch_short().to_string());
            generations += 1;
            agent = Some(fresh);
        }
        state.lock().expect("mock state").tool_rounds_left = turn.tool_rounds;
        let running = agent.as_mut().expect("agent built above");
        running
            .run_turn(&turn.text, |event| {
                if matches!(event, TurnEvent::CacheEvidence(_)) {
                    cache_evidence_events += 1;
                }
            })
            .await
            .expect("turn completes");
        tail = running.transcript_tail().to_vec();
    }

    let records = state.lock().expect("mock state").records.clone();
    Outcome {
        scenario: scenario.name,
        records,
        epochs,
        generations,
        cache_evidence_events,
    }
}

// ---------------------------------------------------------------------------
// Scoring
// ---------------------------------------------------------------------------

fn guard_threshold() -> u64 {
    match std::env::var("DSB_CACHE_GUARD_THRESHOLD") {
        Ok(raw) => raw.trim().parse::<u64>().unwrap_or(DEFAULT_THRESHOLD),
        Err(_) => DEFAULT_THRESHOLD,
    }
}

fn release_gate_enabled() -> bool {
    std::env::var("DSB_RELEASE_CACHE_GUARD").is_ok_and(|value| value == "1")
}

fn check_scenario(outcome: &Outcome, expect: Expect, threshold: u64) -> Result<(), String> {
    if outcome.requests() < TAIL_WINDOW {
        return Err(format!(
            "scenario ran {} request(s); the last-{} window needs at least that many",
            outcome.requests(),
            TAIL_WINDOW
        ));
    }
    if outcome.cache_evidence_events != outcome.requests() {
        return Err(format!(
            "{} request(s) but {} cache-evidence event(s) — the product did not read cache fields on every request",
            outcome.requests(),
            outcome.cache_evidence_events
        ));
    }
    let epochs = outcome.distinct_epochs();
    if epochs.len() != outcome.generations {
        return Err(format!(
            "{} prefix build(s) produced {} distinct epoch(s) — a rebuild did not move the prefix",
            outcome.generations,
            epochs.len()
        ));
    }
    let wire_prefixes = outcome.distinct_fingerprints();
    if wire_prefixes.len() != epochs.len() {
        return Err(format!(
            "{} epoch(s) on the builder but {} distinct leading messages on the wire — the epoch claim and the request bytes disagree",
            epochs.len(),
            wire_prefixes.len()
        ));
    }
    match expect {
        Expect::Pass { epochs: expected } => {
            if epochs.len() != expected {
                return Err(format!(
                    "expected exactly {expected} distinct epoch(s), measured {}",
                    epochs.len()
                ));
            }
            let rate = outcome.tail_rate();
            if rate < threshold {
                return Err(format!(
                    "last-{TAIL_WINDOW} average {rate}% is below the {threshold}% threshold"
                ));
            }
        }
        Expect::LandBelowThreshold => {
            let rate = outcome.tail_rate();
            if rate >= threshold {
                return Err(format!(
                    "negative control scored {rate}% against a {threshold}% threshold — the mock is not accounting from the request (spec 10 §1.9); fail the bench, not the scenario"
                ));
            }
        }
    }
    Ok(())
}

fn expect_scenario(outcome: &Outcome, expect: Expect, threshold: u64) {
    if let Err(problem) = check_scenario(outcome, expect, threshold) {
        panic!("cache guard: {problem}\n  {}", outcome.report());
    }
    eprintln!("CACHE_GUARD_RESULT: {}", outcome.report());
}

fn request(messages: Vec<ChatMessage>) -> ChatRequest {
    ChatRequestBuilder::new(ModelId::Flash)
        .stream(false)
        .messages(messages)
        .build()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// The mock is the instrument: these assertions fail for any mock that answers
/// from constants instead of from the request bytes (zero carry on the first
/// request, full carry on a replay, zero carry when the prefix message moved).
#[tokio::test]
async fn cache_guard_mock_accounts_from_request_bytes() {
    let state = Arc::new(Mutex::new(MockState::default()));
    let server = start_mock(state).await;
    let client =
        Client::new(ClientConfig::new("cache-guard-key").with_base_url(server.uri())).unwrap();

    let first = vec![ChatMessage::system("SYSTEM"), ChatMessage::user("one")];
    let r1 = client
        .chat(request(first.clone()))
        .await
        .expect("first chat");
    let u1 = r1.usage.as_ref().expect("usage on the first response");
    let p1 = u1.prompt_tokens.expect("prompt tokens");
    assert_eq!(
        u1.cache_hit_tokens,
        Some(0),
        "nothing can be carried on the first request"
    );
    assert_eq!(u1.cache_miss_tokens, Some(p1));

    // A strict extension of the previous request carries all of it.
    let mut second = first.clone();
    second.push(ChatMessage::assistant("a"));
    second.push(ChatMessage::user("two"));
    let r2 = client
        .chat(request(second.clone()))
        .await
        .expect("second chat");
    let u2 = r2.usage.as_ref().expect("usage");
    assert_eq!(
        u2.cache_hit_tokens,
        Some(p1),
        "an appended request must carry the whole previous prompt"
    );
    assert!(u2.cache_miss_tokens.expect("miss") > 0);

    // The leading message changed: the carry stops at message one.
    let mut third = second.clone();
    third[0] = ChatMessage::system("SYSTEM_V2");
    let r3 = client
        .chat(request(third.clone()))
        .await
        .expect("third chat");
    let u3 = r3.usage.as_ref().expect("usage");
    assert_eq!(
        u3.cache_hit_tokens,
        Some(0),
        "a moved prefix message must zero the carry"
    );
    assert_eq!(u3.cache_miss_tokens, u3.prompt_tokens);

    // A byte-identical replay carries everything.
    let r4 = client.chat(request(third)).await.expect("replay");
    let u4 = r4.usage.as_ref().expect("usage");
    assert_eq!(
        u4.cache_hit_tokens, u4.prompt_tokens,
        "an identical replay must carry the whole prompt"
    );
    assert_eq!(u4.cache_miss_tokens, Some(0));

    eprintln!(
        "CACHE_GUARD_RESULT: scenario=mock-accounting rates=[{}]",
        [r1, r2, r3, r4]
            .iter()
            .map(|r| {
                let u = r.usage.as_ref().expect("usage");
                let hit = u.cache_hit_tokens.unwrap_or(0);
                let prompt = u.prompt_tokens.unwrap_or(0).max(1);
                100 * hit / prompt
            })
            .map(|rate| rate.to_string())
            .collect::<Vec<_>>()
            .join(",")
    );
}

/// §1.9 exact layer + rate layer on a scenario that changes nothing: one
/// epoch for the whole run, and the rate curve holds the threshold.
#[tokio::test]
async fn cache_guard_unchanged_prefix_is_one_epoch_and_passes() {
    let scenario = plain_dialogue("plain-dialogue", 6, true);
    let outcome = run_scenario(&scenario).await;
    expect_scenario(&outcome, scenario.expect, guard_threshold());
}

/// §1.9 exact layer on a scenario that changes one §1.1 input by design:
/// exactly two epochs, and the tail recovers above the threshold.
#[tokio::test]
async fn cache_guard_by_design_change_is_two_epochs() {
    let scenario = input_change_by_design();
    let outcome = run_scenario(&scenario).await;
    expect_scenario(&outcome, scenario.expect, guard_threshold());
}

/// §1.9 negative control — spec §4.4: *a perturbed prefix lands below the
/// threshold, an unchanged one does not*. The perturbed side proves the rate
/// layer can fail (a constant mock would pass it); the unchanged side proves
/// the layer is not failing everything.
#[tokio::test]
async fn cache_guard_negative_control() {
    let perturbed = run_scenario(&negative_control()).await;
    expect_scenario(&perturbed, Expect::LandBelowThreshold, guard_threshold());

    let unchanged = run_scenario(&plain_dialogue("plain-dialogue", 6, true)).await;
    expect_scenario(&unchanged, Expect::Pass { epochs: 1 }, guard_threshold());
}

/// The tool loop, always-on: the schema of every scenario shape is measured by
/// the ordinary suite, tool execution included, so the release matrix only
/// extends counts (and the no-reasoning flag) over verified ground.
#[tokio::test]
async fn cache_guard_tool_loop_stays_above_threshold() {
    let scenario = tool_loop("tool-loop", 14, true);
    let outcome = run_scenario(&scenario).await;
    expect_scenario(&outcome, scenario.expect, guard_threshold());
}

/// Mixed message sizes, always-on for the same reason: the fresh-tail share
/// moves without any §1.1 input changing, and the epoch layer must not notice.
#[tokio::test]
async fn cache_guard_mixed_message_sizes_hold_the_threshold() {
    let scenario = mixed_message_sizes();
    let outcome = run_scenario(&scenario).await;
    expect_scenario(&outcome, scenario.expect, guard_threshold());
}

/// The full scenario matrix, release-gated by `DSB_RELEASE_CACHE_GUARD=1`
/// (wrapper: `scripts/cache-guard.sh`). Unset, this reports a skip so the
/// ordinary suite never pays for the bench.
#[tokio::test]
async fn cache_guard_release_suite() {
    if !release_gate_enabled() {
        eprintln!(
            "cache_guard_release=skipped (set DSB_RELEASE_CACHE_GUARD=1 to run the release cache guard)"
        );
        return;
    }
    let threshold = guard_threshold();
    for scenario in scenario_matrix() {
        let outcome = run_scenario(&scenario).await;
        if let Err(problem) = check_scenario(&outcome, scenario.expect, threshold) {
            panic!(
                "cache guard release suite: {problem}\n  {}",
                outcome.report()
            );
        }
        eprintln!("CACHE_GUARD_RESULT: {}", outcome.report());
    }
}
