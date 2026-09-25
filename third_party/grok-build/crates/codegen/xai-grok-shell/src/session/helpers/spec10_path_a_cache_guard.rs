//! Spec 10 §1.9 on Path A request bytes (`cache_guard_path_a_*`).
//!
//! The overlay bench (`crates/dsb-agent/tests/cache_guard.rs`) scores
//! `dsb-context`. `xai-grok-shell` does not depend on that crate, so a guard
//! that stops there stays green when the bytes `turn.rs` sends are wrong.
//! This module scores the bytes Path A actually builds.
//!
//! # What is scored
//!
//! Each scenario request is:
//!
//! 1. `assemble_spec10_path_a_turn` on the inputs `apply_spec10_to_conversation_request`
//!    feeds it (`extract_base_system_prompt`, `discover_skills_index`,
//!    `discover_project_instructions`, `path_a_inputs_from_turn`).
//! 2. `place_stable_body` — the same placement that function writes onto the
//!    `ConversationRequest` at `acp_session_impl/turn.rs` before the sampler
//!    runs. The stamp and the telemetry lines around that call are not bytes,
//!    so they are not part of the score. `cache_guard_path_a_scored_bytes_match_apply_spec10`
//!    locks this wiring to `apply_spec10_to_conversation_request`.
//! 3. `ChatCompletionRequest::from`, which calls `conversation_to_chat_messages`.
//!    The mock accounts the serialized `messages` array, message by message,
//!    the same rule as the overlay mock. The `tools[]` field is a full
//!    replacement on every request (spec 10 §1.10 rule 5) and is not part of
//!    the carry; a tools change still moves the score because the tools
//!    document is inside the stable body. Sampler defaults (`model`,
//!    temperature) are not message bytes.
//!
//! Hit and miss tokens are `bytes / 4`. The constant cancels in the ratio.
//! It is the overlay's unit, not a provider bill.
//!
//! # Epochs
//!
//! Path A re-assembles the stable body on every request. A distinct epoch is
//! a distinct `epoch_sha256_hex` of that assembly. The mock cross-checks that
//! count against the fingerprint of the **latest system message** on the
//! mapped messages — spec 10 §1.10 rule 1, the effective prompt, which is the
//! body the epoch hashes. The overlay cross-checks the leading message
//! because Path B puts the only stable body there. On Path A a later body is
//! appended, so the leading message fingerprint stays **1** across an epoch
//! change. Requiring those two counts to be equal would reject the append
//! spec 10 §1.10 requires. Scenarios therefore assert both: latest-system
//! fingerprints equal epochs, and the leading fingerprint count stays 1.
//!
//! # Rate window
//!
//! `hit * 100 / (hit + miss)` per request, then the integer mean of the last
//! `min(3, requests)` rates — the same clamp as the overlay's `tail_rate`.
//! A scenario with fewer than 3 requests fails. The threshold is
//! `DSB_CACHE_GUARD_THRESHOLD` (default 90).
//!
//! Tool rounds are scripted `ConversationItem`s (reasoning sibling, assistant
//! tool call, tool result). This bench does not spawn a tool process. The
//! overlay test remains the one that executes tools.
//!
//! The contract tests run with the crate. The full scenario matrix runs only
//! when `DSB_RELEASE_CACHE_GUARD=1`. `scripts/cache-guard.sh` invokes this
//! module only when `xai-grok-shell` has already been compiled into the
//! vendored target, so the release wrapper does not start a cold build.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::sync::Arc;

use xai_grok_sampling_types::{
    ChatCompletionRequest, ChatRequestMessage, ConversationItem, ConversationRequest, Role,
    ToolCall, ToolSpec, synthesized_reasoning_item,
};

use super::{
    apply_spec10_to_conversation_request, assemble_spec10_path_a_turn,
    discover_project_instructions, discover_skills_index, extract_base_system_prompt,
    path_a_inputs_from_turn, place_stable_body,
};

/// Bytes the mock charges per token. Same unit as the overlay bench.
const BYTES_PER_TOKEN: usize = 4;
/// Spec 10 §1.9 threshold — Reasonix's 90, kept for comparability.
const DEFAULT_THRESHOLD: u64 = 90;
/// The window the rate layer averages over (spec 10 §1.9: last 3 turns).
const TAIL_WINDOW: usize = 3;

/// One line of the product template. Repeated so the stable body dwarfs a
/// fresh tail: a single appended user line, assistant reply, or tool result
/// stays well under 10% of the serialized prompt.
const TEMPLATE_LINE: &str =
    "Product template line: the stable prefix stays free of clocks and random ids.\n";
const TEMPLATE_REPEATS: usize = 160;

const DIALOGUE_LINE: &str = "keep the request prefix stable while the conversation grows. ";

/// ~250 chars of chain-of-thought, folded into the following assistant by
/// `conversation_to_chat_messages`.
const REASONING: &str = "Let me weigh the constraints first: the visible answer must stay \
short, but the chain-of-thought is replayed on later turns, so it is part of the volatile \
tail the next request pays for. I will keep it stable, because a rewrite here would move \
bytes the provider already cached.";

const TOOL_RESULT: &str = "alpha: deterministic tool result for the bench.\n";

// ---------------------------------------------------------------------------
// Prefix-accounting mock
// ---------------------------------------------------------------------------

/// One scored request. Tokens are derived from serialized message bytes.
#[derive(Debug, Clone)]
struct ReqRecord {
    total_bytes: usize,
    hit_bytes: usize,
    /// Fingerprint of `messages[0]` — the leading system message Path A keeps.
    leading_fingerprint: u64,
    /// Fingerprint of the latest system message — the effective prompt (§1.10).
    effective_fingerprint: u64,
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
    previous_messages: Option<Vec<Vec<u8>>>,
}

fn fingerprint(bytes: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    hasher.finish()
}

/// How much of this request the previous request already carried, measured
/// message by message. The first message that differs ends the carry. A mock
/// that returns a constant ratio fails
/// `cache_guard_path_a_mock_accounts_from_request_bytes`.
fn account(state: &mut MockState, messages: &[ChatRequestMessage]) -> Result<ReqRecord, String> {
    if messages.is_empty() {
        return Err("cache guard mock: request carries no messages".to_string());
    }
    let canonical = messages
        .iter()
        .map(|message| {
            serde_json::to_vec(message).map_err(|err| format!("serialize message: {err}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let carried = match &state.previous_messages {
        Some(previous) => canonical
            .iter()
            .zip(previous.iter())
            .take_while(|(current, prior)| current == prior)
            .count(),
        None => 0,
    };
    let hit_bytes = canonical.iter().take(carried).map(Vec::len).sum();
    let total_bytes = canonical.iter().map(Vec::len).sum();
    let leading = canonical
        .first()
        .ok_or_else(|| "cache guard mock: missing leading message".to_string())?;
    let effective_idx = messages
        .iter()
        .rposition(|message| message.role == Role::System)
        .ok_or_else(|| "cache guard mock: request has no system message".to_string())?;
    let effective = canonical
        .get(effective_idx)
        .ok_or_else(|| "cache guard mock: latest system message missing from bytes".to_string())?;
    let record = ReqRecord {
        total_bytes,
        hit_bytes,
        leading_fingerprint: fingerprint(leading),
        effective_fingerprint: fingerprint(effective),
    };
    state.previous_messages = Some(canonical);
    Ok(record)
}

// ---------------------------------------------------------------------------
// Request bytes: assemble + place + Chat Completions mapping
// ---------------------------------------------------------------------------

fn product_template() -> String {
    TEMPLATE_LINE.repeat(TEMPLATE_REPEATS)
}

fn base_tools() -> Vec<ToolSpec> {
    vec![ToolSpec {
        name: "read_file".into(),
        description: Some("Read a workspace file".into()),
        parameters: serde_json::json!({
            "type": "object",
            "properties": { "path": { "type": "string" } },
            "required": ["path"]
        }),
    }]
}

fn base_request() -> ConversationRequest {
    ConversationRequest {
        items: vec![ConversationItem::system(product_template())],
        tools: base_tools(),
        model: Some("deepseek-v4-flash".into()),
        ..ConversationRequest::default()
    }
}

fn workspace_cwd(workspace: &Path) -> &str {
    workspace
        .to_str()
        .expect("scenario workspace path is utf-8")
}

/// The byte-producing steps of `apply_spec10_to_conversation_request`.
/// Stamp and telemetry are omitted; the parity test below checks the items.
fn assemble_onto(
    request: &mut ConversationRequest,
    workspace: &Path,
) -> super::Spec10PathAAssembled {
    let system_prompt = request
        .items
        .iter()
        .find_map(|item| match item {
            ConversationItem::System(sys) => Some(sys.content.as_ref().to_string()),
            _ => None,
        })
        .unwrap_or_default();
    let base = extract_base_system_prompt(&system_prompt);
    let volatile_count = request
        .items
        .iter()
        .filter(|item| !matches!(item, ConversationItem::System(_)))
        .count();
    let inputs = path_a_inputs_from_turn(
        &base,
        &request.tools,
        workspace_cwd(workspace),
        discover_skills_index(workspace, None),
        discover_project_instructions(workspace),
        volatile_count,
    );
    let assembled = assemble_spec10_path_a_turn(&inputs);
    place_stable_body(&mut request.items, assembled.stable_body.as_str());
    assembled
}

fn mapped_messages(request: &ConversationRequest) -> Vec<ChatRequestMessage> {
    ChatCompletionRequest::from(request.clone()).messages
}

fn latest_system_text(messages: &[ChatRequestMessage]) -> Option<String> {
    messages
        .iter()
        .rev()
        .find(|message| message.role == Role::System)
        .map(|message| message.text_content())
}

fn write_base(workspace: &Path) {
    std::fs::write(
        workspace.join("AGENTS.md"),
        "Bench workspace instructions: keep the prefix stable.\n",
    )
    .expect("write AGENTS.md");
    let skill = workspace.join("skills").join("bench-skill");
    std::fs::create_dir_all(&skill).expect("skill dir");
    std::fs::write(
        skill.join("SKILL.md"),
        "---\ndescription: A skill the bench keeps in the index.\n---\n# bench\n",
    )
    .expect("write SKILL.md");
}

fn apply_mutation(workspace: &Path, mutation: &Mutation) {
    match mutation {
        Mutation::AddSkill(name) => {
            let skill = workspace.join("skills").join(name);
            std::fs::create_dir_all(&skill).expect("skill dir");
            std::fs::write(
                skill.join("SKILL.md"),
                format!("---\ndescription: skill {name}\n---\n# {name}\n"),
            )
            .expect("write SKILL.md");
        }
        Mutation::ChangeInstructions(text) => {
            std::fs::write(workspace.join("AGENTS.md"), text).expect("write AGENTS.md");
        }
    }
}

fn push_tool_exchange(items: &mut Vec<ConversationItem>, seq: u32, reasoning: Option<&str>) {
    if let Some(text) = reasoning {
        items.push(ConversationItem::Reasoning(synthesized_reasoning_item(
            text,
        )));
    }
    let id = format!("call_{seq}");
    items.push(ConversationItem::assistant_tool_calls(vec![ToolCall {
        id: Arc::<str>::from(id.as_str()),
        name: "read_file".into(),
        arguments: Arc::<str>::from("{\"path\":\"notes.txt\"}"),
    }]));
    items.push(ConversationItem::tool_result(id, TOOL_RESULT));
}

fn push_final_assistant(items: &mut Vec<ConversationItem>, reasoning: Option<&str>) {
    if let Some(text) = reasoning {
        items.push(ConversationItem::Reasoning(synthesized_reasoning_item(
            text,
        )));
    }
    items.push(ConversationItem::assistant("Done."));
}

fn score_current(
    request: &mut ConversationRequest,
    workspace: &Path,
    state: &mut MockState,
    epochs: &mut Vec<String>,
    records: &mut Vec<ReqRecord>,
) {
    let assembled = assemble_onto(request, workspace);
    let messages = mapped_messages(request);
    let latest = latest_system_text(&messages).unwrap_or_default();
    assert_eq!(
        latest, assembled.stable_body,
        "the mapped latest system message must be the assembled stable body"
    );
    epochs.push(assembled.epoch_sha256_hex);
    let record = account(state, &messages).unwrap_or_else(|problem| panic!("{problem}"));
    records.push(record);
}

// ---------------------------------------------------------------------------
// Scenarios
// ---------------------------------------------------------------------------

/// A change to one §1.1 input, applied before that turn's assembly.
/// Path A appends the new stable body (§1.10); it does not rewrite the head.
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
    mutation: Option<Mutation>,
    /// Tool rounds scripted before the final assistant line for this turn.
    tool_rounds: u32,
}

#[derive(Debug, Clone, Copy)]
enum Expect {
    /// Both layers must pass, with exactly this many distinct epochs.
    Pass { epochs: usize },
    /// The rate layer must fail — the negative control (§1.9).
    LandBelowThreshold,
}

#[derive(Debug, Clone)]
struct Scenario {
    name: &'static str,
    reasoning: Option<&'static str>,
    turns: Vec<Turn>,
    expect: Expect,
}

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

fn tool_loop(name: &'static str, rounds: u32, reasoning: bool) -> Scenario {
    Scenario {
        name,
        reasoning: reasoning.then_some(REASONING),
        turns: vec![turn("Read the notes file repeatedly, then answer.", rounds)],
        expect: Expect::Pass { epochs: 1 },
    }
}

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

/// One §1.1 input changed by design on the second turn. Exactly two epochs.
/// The turn that appends the new body pays for it; the last three recover.
fn input_change_by_design() -> Scenario {
    let turns = (1..=6)
        .map(|i| {
            if i == 2 {
                mutated_turn(
                    format!("Turn {i}: {DIALOGUE_LINE}"),
                    Mutation::ChangeInstructions(
                        "Bench workspace instructions: the prefix moved on purpose.\n".to_string(),
                    ),
                    0,
                )
            } else {
                turn(format!("Turn {i}: {DIALOGUE_LINE}"), 0)
            }
        })
        .collect();
    Scenario {
        name: "input-change-by-design",
        reasoning: Some(REASONING),
        turns,
        expect: Expect::Pass { epochs: 2 },
    }
}

/// A §1.1 input is perturbed before every turn after the first, so each
/// request appends a new stable body. The last-3 average must land below
/// the threshold. If it does not, the mock is not accounting from the bytes.
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
// Runner and scoring
// ---------------------------------------------------------------------------

struct Outcome {
    scenario: &'static str,
    records: Vec<ReqRecord>,
    /// Epoch of every scored request, in order. Repeats collapse in
    /// [`Outcome::distinct_epochs`].
    epochs: Vec<String>,
    /// Prefix builds: the initial assembly, plus one per §1.1 mutation.
    generations: usize,
}

impl Outcome {
    fn requests(&self) -> usize {
        self.records.len()
    }

    fn rates(&self) -> Vec<u64> {
        self.records.iter().map(ReqRecord::rate).collect()
    }

    /// Mean of the last `min(TAIL_WINDOW, requests)` rates. A scenario with
    /// fewer than [`TAIL_WINDOW`] requests is rejected by [`check_scenario`];
    /// the clamp is what the mean uses when someone calls it directly.
    fn tail_rate(&self) -> u64 {
        let rates = self.rates();
        let n = rates.len().min(TAIL_WINDOW);
        if n == 0 {
            return 0;
        }
        let start = rates.len() - n;
        rates.iter().skip(start).sum::<u64>() / n as u64
    }

    fn distinct_epochs(&self) -> Vec<String> {
        distinct_strings(&self.epochs)
    }

    fn distinct_effective_prefixes(&self) -> Vec<String> {
        let fingerprints = self
            .records
            .iter()
            .map(|record| record.effective_fingerprint.to_string())
            .collect::<Vec<_>>();
        distinct_strings(&fingerprints)
    }

    fn distinct_leading_prefixes(&self) -> Vec<String> {
        let fingerprints = self
            .records
            .iter()
            .map(|record| record.leading_fingerprint.to_string())
            .collect::<Vec<_>>();
        distinct_strings(&fingerprints)
    }

    fn report(&self) -> String {
        let rates = self
            .rates()
            .iter()
            .map(u64::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let last = match self.records.last() {
            Some(record) => format!(
                " last=hit{}/prompt{}",
                record.hit_tokens(),
                record.prompt_tokens()
            ),
            None => String::new(),
        };
        format!(
            "scenario={} requests={} generations={} epochs={} effective_prefixes={} leading_prefixes={} tail_rate={}% rates=[{}]{}",
            self.scenario,
            self.requests(),
            self.generations,
            self.distinct_epochs().len(),
            self.distinct_effective_prefixes().len(),
            self.distinct_leading_prefixes().len(),
            self.tail_rate(),
            rates,
            last,
        )
    }
}

fn distinct_strings(items: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for item in items {
        if !out.contains(item) {
            out.push(item.clone());
        }
    }
    out
}

fn run_scenario(scenario: &Scenario) -> Outcome {
    let dir = tempfile::tempdir().expect("scenario workspace");
    write_base(dir.path());
    let mut request = base_request();
    let mut state = MockState::default();
    let mut epochs = Vec::new();
    let mut records = Vec::new();
    let mut generations = 0usize;
    let mut saw_prefix = false;

    for turn in &scenario.turns {
        if let Some(mutation) = &turn.mutation {
            apply_mutation(dir.path(), mutation);
        }
        if !saw_prefix || turn.mutation.is_some() {
            generations += 1;
            saw_prefix = true;
        }
        request
            .items
            .push(ConversationItem::user(turn.text.as_str()));
        score_current(
            &mut request,
            dir.path(),
            &mut state,
            &mut epochs,
            &mut records,
        );
        for seq in 1..=turn.tool_rounds {
            push_tool_exchange(&mut request.items, seq, scenario.reasoning);
            score_current(
                &mut request,
                dir.path(),
                &mut state,
                &mut epochs,
                &mut records,
            );
        }
        push_final_assistant(&mut request.items, scenario.reasoning);
    }

    Outcome {
        scenario: scenario.name,
        records,
        epochs,
        generations,
    }
}

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
            "scenario ran {} request(s); the last-{TAIL_WINDOW} window needs at least that many",
            outcome.requests()
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
    let effective = outcome.distinct_effective_prefixes();
    if effective.len() != epochs.len() {
        return Err(format!(
            "{} epoch(s) on the assembly but {} distinct latest system message(s) on the wire — the epoch claim and the request bytes disagree",
            epochs.len(),
            effective.len()
        ));
    }
    let leading = outcome.distinct_leading_prefixes();
    if leading.len() != 1 {
        return Err(format!(
            "{} distinct leading message(s); Path A records an epoch change as an appended system message (spec 10 §1.10), so the first message stays one fingerprint",
            leading.len()
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
        panic!(
            "cache guard: {problem}\n  {} threshold={threshold}%",
            outcome.report()
        );
    }
    eprintln!(
        "CACHE_GUARD_RESULT: {} threshold={threshold}%",
        outcome.report()
    );
}

fn record_with_rate(rate: u64) -> ReqRecord {
    // 400 bytes → 100 tokens. `hit_bytes = rate * 4` yields that rate.
    ReqRecord {
        total_bytes: 400,
        hit_bytes: (rate as usize) * 4,
        leading_fingerprint: 1,
        effective_fingerprint: 1,
    }
}

fn system_texts(items: &[ConversationItem]) -> Vec<String> {
    items
        .iter()
        .filter_map(|item| match item {
            ConversationItem::System(sys) => Some(sys.content.as_ref().to_string()),
            _ => None,
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// The mock is the instrument. These assertions fail for a mock that answers
/// from a constant ratio: nothing is carried on the first request, an append
/// carries the whole previous prompt, a moved leading message zeroes the
/// carry, and a replay carries everything.
#[test]
fn cache_guard_path_a_mock_accounts_from_request_bytes() {
    let dir = tempfile::tempdir().expect("scenario workspace");
    write_base(dir.path());
    let mut request = base_request();
    request.items.push(ConversationItem::user("one"));
    let mut state = MockState::default();
    let mut epochs = Vec::new();
    let mut records = Vec::new();
    score_current(
        &mut request,
        dir.path(),
        &mut state,
        &mut epochs,
        &mut records,
    );
    let first = records.last().expect("first request").clone();
    assert_eq!(
        first.hit_tokens(),
        0,
        "nothing can be carried on the first request"
    );
    assert_eq!(first.miss_tokens(), first.prompt_tokens());

    request.items.push(ConversationItem::assistant("a"));
    request.items.push(ConversationItem::user("two"));
    score_current(
        &mut request,
        dir.path(),
        &mut state,
        &mut epochs,
        &mut records,
    );
    let second = records.last().expect("second request").clone();
    assert_eq!(
        second.hit_tokens(),
        first.prompt_tokens(),
        "an appended request must carry the whole previous prompt"
    );
    assert!(second.miss_tokens() > 0);

    // The rewrite is applied to the mapped messages, not through
    // `place_stable_body`. That function would repair a head that lost the
    // tools marker. This case is the mock's boundary, same as the overlay.
    let mapped = mapped_messages(&request);
    let mut rewritten = Vec::new();
    let mut rest = mapped.into_iter();
    let _old_head = rest.next();
    rewritten.push(ChatRequestMessage::system("REWRITTEN_HEAD"));
    rewritten.extend(rest);
    let third = account(&mut state, &rewritten).expect("rewritten request");
    assert_eq!(
        third.hit_tokens(),
        0,
        "a moved leading message must zero the carry"
    );
    assert_eq!(third.miss_tokens(), third.prompt_tokens());

    let fourth = account(&mut state, &rewritten).expect("replay");
    assert_eq!(
        fourth.hit_tokens(),
        fourth.prompt_tokens(),
        "an identical replay must carry the whole prompt"
    );
    assert_eq!(fourth.miss_tokens(), 0);

    let rates = [first.rate(), second.rate(), third.rate(), fourth.rate()];
    eprintln!(
        "CACHE_GUARD_RESULT: scenario=mock-accounting rates=[{}]",
        rates
            .iter()
            .map(u64::to_string)
            .collect::<Vec<_>>()
            .join(",")
    );
}

/// The bench's placement matches `apply_spec10_to_conversation_request`.
/// That function also stamps an epoch file when `DEEPSEEK_BUILD_HOME` is set;
/// the bytes compared here do not include the stamp.
#[test]
fn cache_guard_path_a_scored_bytes_match_apply_spec10() {
    let dir = tempfile::tempdir().expect("scenario workspace");
    write_base(dir.path());
    let cwd = workspace_cwd(dir.path());
    let mut via_apply = base_request();
    via_apply.items.push(ConversationItem::user("hello"));
    let mut via_bench = via_apply.clone();
    apply_spec10_to_conversation_request(&mut via_apply, cwd, Some(dir.path()), None);
    let assembled = assemble_onto(&mut via_bench, dir.path());
    assert_eq!(
        system_texts(&via_apply.items),
        system_texts(&via_bench.items),
        "bench placement drifted from apply_spec10_to_conversation_request"
    );
    let messages = mapped_messages(&via_bench);
    assert_eq!(
        latest_system_text(&messages).as_deref(),
        Some(assembled.stable_body.as_str())
    );
}

/// Last-3 mean clamps at `min(3, requests)`. Fewer than 3 requests fail
/// before either layer is scored.
#[test]
fn cache_guard_path_a_tail_window_clamps_and_rejects_a_short_run() {
    let records = [0u64, 10, 80, 90, 100]
        .into_iter()
        .map(record_with_rate)
        .collect();
    let outcome = Outcome {
        scenario: "tail-window",
        records,
        epochs: vec!["epoch-a".to_string()],
        generations: 1,
    };
    assert_eq!(
        outcome.tail_rate(),
        90,
        "mean of the last 3 rates (80, 90, 100), not of the whole run"
    );
    assert!(check_scenario(&outcome, Expect::Pass { epochs: 1 }, 90).is_ok());

    let short = Outcome {
        scenario: "short",
        records: vec![record_with_rate(0), record_with_rate(100)],
        epochs: vec!["epoch-a".to_string()],
        generations: 1,
    };
    assert_eq!(
        short.tail_rate(),
        50,
        "with only two rates the clamp averages both"
    );
    let problem = check_scenario(&short, Expect::Pass { epochs: 1 }, 90).unwrap_err();
    assert!(
        problem.contains("last-3"),
        "a short scenario must fail closed: {problem}"
    );
}

#[test]
fn cache_guard_path_a_unchanged_prefix_is_one_epoch_and_passes() {
    let scenario = plain_dialogue("plain-dialogue", 6, true);
    let outcome = run_scenario(&scenario);
    expect_scenario(&outcome, scenario.expect, guard_threshold());
}

#[test]
fn cache_guard_path_a_by_design_change_is_two_epochs() {
    let scenario = input_change_by_design();
    let outcome = run_scenario(&scenario);
    expect_scenario(&outcome, scenario.expect, guard_threshold());
}

/// Perturbing a §1.1 input before every later turn lands below the threshold.
/// The same runner, with no perturbation, holds the threshold — the layer is
/// not failing every scenario.
#[test]
fn cache_guard_path_a_negative_control() {
    let perturbed = run_scenario(&negative_control());
    expect_scenario(&perturbed, Expect::LandBelowThreshold, guard_threshold());

    let unchanged = run_scenario(&plain_dialogue("plain-dialogue", 6, true));
    expect_scenario(&unchanged, Expect::Pass { epochs: 1 }, guard_threshold());
}

#[test]
fn cache_guard_path_a_tool_loop_stays_above_threshold() {
    let scenario = tool_loop("tool-loop", 14, true);
    let outcome = run_scenario(&scenario);
    expect_scenario(&outcome, scenario.expect, guard_threshold());
}

#[test]
fn cache_guard_path_a_mixed_message_sizes_hold_the_threshold() {
    let scenario = mixed_message_sizes();
    let outcome = run_scenario(&scenario);
    expect_scenario(&outcome, scenario.expect, guard_threshold());
}

/// Full matrix. Unset `DSB_RELEASE_CACHE_GUARD` skips, so an ordinary
/// `cargo test -p xai-grok-shell --lib` does not pay for the long scenarios.
#[test]
fn cache_guard_path_a_release_suite() {
    if !release_gate_enabled() {
        eprintln!(
            "cache_guard_path_a_release=skipped (set DSB_RELEASE_CACHE_GUARD=1 to run the release cache guard)"
        );
        return;
    }
    let threshold = guard_threshold();
    for scenario in scenario_matrix() {
        let outcome = run_scenario(&scenario);
        if let Err(problem) = check_scenario(&outcome, scenario.expect, threshold) {
            panic!(
                "cache guard release suite: {problem}\n  {} threshold={threshold}%",
                outcome.report()
            );
        }
        eprintln!(
            "CACHE_GUARD_RESULT: {} threshold={threshold}%",
            outcome.report()
        );
    }
}
