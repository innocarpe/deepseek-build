# Spec 10 — Cache contract (stable prefix)

| Field | Value |
|-------|--------|
| Status | **ready-for-impl** — §1.5.1 (overlay and Path A), §1.5.2, §1.9 (overlay bench and Path A assembly bench), and §1.10 enforced with tests |
| Philosophy | HARNESS §4.2, §5; Deep Code pillar B; Reasonix cache-first |
| Gate | Part of **G2** |
| Tests | **Automated golden + negative required** |

## 1. Behavior

The product builds each model request as:

```text
messages_to_api =
  stable_prefix_messages   // byte-stable across turns when inputs unchanged
  + volatile_tail_messages // user, tool results, dynamic reminders
```

### 1.1 Stable prefix contents (ordered)

1. System prompt body (product template; no wall-clock, no random IDs).  
2. Tool schemas document (canonical JSON; **sorted object keys** recursively).  
3. Skills **index only** (name + one-line description; deterministic sort by name).  
4. Small environment summary (OS family, cwd **as normalized path**, not hostname unless user opts in).  
5. Standing project instructions (discovered files; deterministic order — see §1.4).  

### 1.2 Volatile tail

- Current user turn  
- Assistant/tool messages for the active turn chain  
- Dynamic reminders, large tool outputs (may be snipped)  
- Anything containing timestamps used for UX only  

### 1.3 Byte stability

Define `stable_prefix_bytes = UTF-8 encoding of canonicalize(stable_prefix_messages)`.

**Canonicalize rules (normative):**

- JSON: keys sorted lexicographically; no insignificant whitespace beyond single separators as implemented in a single `serde_json::to_vec` / equivalent **documented** function.  
- Newlines: `\n` only inside stored strings.  
- Paths: prefer project-relative when under workspace root; else absolute normalized (no trailing slash except root).  
- No `SystemTime::now`, random UUIDs, or process id in stable sections.

**Invariant:** For identical inputs to the prefix builder, two consecutive builds produce **equal** `stable_prefix_bytes`.

### 1.4 Project instruction discovery (minimal for M1)

Load if present, in order, concatenate with clear separators:

1. `./DEEPSEEK.md` or `./DEEPSEEK_BUILD.md` (first found)  
2. `./AGENTS.md`  
3. `./.deepseek-build/instructions.md` if present  

Missing files skip. Content changes → **new cache epoch** (prefix hash changes; expected).

### 1.5 Epochs

An **epoch** is the hash of `stable_prefix_bytes`. Log `prefix_epoch=sha256_hex` per request (or first 16 hex chars). Mid-session tool schema change **must** bump epoch (new tools list → new prefix).

#### 1.5.1 Change attribution (normative)

An epoch says *that* the prefix moved. It must not be the only thing the log says.

A **prefix shape** is the per-component hashes over exactly the inputs the
builder consumed — the five components of §1.1:

| Component | Covers |
|-----------|--------|
| `system` | System prompt template body (§1.1 item 1) |
| `tools` | Tool schemas document (§1.1 item 2) |
| `skills` | Skills index (§1.1 item 3) |
| `environment` | OS family + normalized cwd (§1.1 item 4) |
| `project_instructions` | Standing project instructions (§1.1 item 5) |

Each component hash is SHA-256 over that component's bytes **as `build()`
emits them**, produced by the same document functions — so a component hash
cannot drift from the bytes whose name it carries.

Shape is **observational**. Computing it must not change
`stable_prefix_bytes`: a build with shape and a build without one produce
identical bytes and an identical epoch (§1.8 item 3).

When a rebuild is compared against a stored baseline (rule 5), log the reason
set — including when nothing moved, because `none` is the evidence that the
comparison actually ran:

```text
prefix_change=system,tools prev=<epoch_short> cur=<epoch_short>
prefix_change_detail=tools.added=write,skills.removed=old-skill
```

- Axis names: `system` · `tools` · `skills` · `environment` ·
  `project_instructions`. Sorted by that order, comma-joined.
- Detail sub-keys, only where the component exposes a finer axis:
  - `tools.added=<name>` · `tools.removed=<name>` · `tools.changed=<name>`
  - `skills.added=<name>` · `skills.removed=<name>` · `skills.changed=<name>`
  - `environment.os_family` · `environment.cwd`
- `prefix_change=none` when nothing moved.
- `prefix_change=unattributed` when the epoch moved but **no** component hash
  did. That is a coverage bug in this section, not a provider event (§3). It
  can only appear when the two builds' epochs are compared; a stored baseline
  carries the short epoch for exactly that comparison.

Rules:

1. Attribution is reported **where the rebuild happens**, not once per turn. A
   turn that reuses the session prefix logs `prefix_epoch=` and nothing more —
   and a resume with no stored baseline logs no `prefix_change=` line at all.
2. A component with no finer axis reports its name only. `project_instructions`
   reaches the builder already concatenated (§1.4) and re-parsing file
   boundaries out of it is not a contract; `system` likewise.
3. Detail carries axis names and tool/skill **names** — never instruction
   content: no prompt body, no skill description, no cwd value.
4. A comparison with no baseline makes **no claim**: the shape is recorded and
   no `prefix_change=` line is logged. Guessing from an absent baseline is how
   false attribution starts. A baseline that exists always produces a line,
   `none` included, because that is the evidence the comparison ran.
5. The shape of the build that produced a session's transcript is stored **with
   that session** (spec 100 storage; additive, absent in files written before
   this contract). That is where a rebuild for a continuing conversation
   happens: the next process builds a fresh prefix and compares it against the
   stored shape. Within a process the prefix is built once and reused, so a
   per-turn comparison would report `none` forever and name nothing.

**Landed (overlay):** shape, comparison, and resume-time logging (`dsb-context`,
`dsb-agent`, `dsb-cli`); §4.2 and §4.3 name the tests. That path is not Path A.
`xai-grok-shell` does not depend on `dsb-context`.

##### Path A (vendored turn)

Path A builds the prefix in `assemble_spec10_path_a_turn` and applies it from
`apply_spec10_to_conversation_request`, called from
`acp_session_impl/turn.rs` on every main turn. Attribution uses only the
documents that function already concatenates, hashed **before** concatenation:

| Axis | Bytes hashed |
|---|---|
| `system` | System prompt after `trim_end`, the bytes before `## Tools` |
| `tools` | `tools_document` |
| `skills` | `skills_document` |
| `environment` | `env_document` |
| `project_instructions` | The project-instructions bytes actually appended, or empty when that section is omitted |

No other axis. The volatile tail, the §1.10 placement
(`unchanged` / `first_write` / `appended`), and the wire `tools[]` array are
not attribution categories. The tools axis is the tools document inside the
stable body. A category this assembly cannot see is not added.

Rules for this path, against §1.5.1:

1. The baseline is the previous assembly of **this session in this process**.
   The first assembly has no baseline and logs no `prefix_change=` line
   (rule 4). A missing baseline is not a guess.
2. Assembly runs on every turn, so an unchanged epoch logs `prefix_epoch=`
   only and does **not** log `prefix_change=none` (rule 1). Rule 4's "a
   baseline always produces a line, `none` included" is the overlay resume
   comparison, which happens once. Repeating `none` on every Path A turn
   would name nothing.
3. When the epoch differs, log one block:

   ```text
   prefix_change=system,tools prev=<epoch_short> cur=<epoch_short>
   prefix_change_detail=tools.added=write
   ```

   Axes are in the §1.1 order above. The detail line is omitted when the
   moved component has no finer axis (`system`, `project_instructions`).
4. Finer axes, and only these: `tools.added=<name>`, `tools.removed=<name>`,
   `tools.changed=<name>`, `tools.reordered` (the tools document moved and
   every named tool hash matches), `skills.added=<name>`,
   `skills.removed=<name>`, `skills.changed=<name>`, `environment.os_family`,
   `environment.cwd`. Detail names the axis. It does not include prompt text,
   a skill description, or a cwd value (rule 3).
5. If the epoch differs and every component hash matches, log
   `prefix_change=unattributed`. That is a coverage bug in this section, not
   a sixth component and not a provider event (§3).
6. Computing the shape must not change `stable_prefix_bytes` or the epoch.
   The hashes are of the strings just concatenated.

The baseline is not written to the session file. Cross-process resume
attribution stays the overlay record (`report_prefix_change` on Path B). A
new Path A process has no baseline until its first assembly.

**Landed (Path A):** `observe_path_a_prefix_change` in
`spec10_path_a_assembly.rs`, called from `turn.rs`. §4.7 names the tests.

#### 1.5.2 Session-cumulative hit/miss (landed)

Per-turn cache evidence (§1.8) answers "how warm was this turn". The cost
question is the session's, and it needs a counter that survives compaction and
turn-level noise.

Log field, emitted once per turn when the session has any evidence:

```text
cache_session=hit=<n>,miss=<n>,rate=<pct>,reported=<n>,unreported=<n>
```

- `hit` / `miss`: summed prompt tokens over responses that carried cache
  fields. Denominators are token sums, never turn counts.
- `rate`: `hit * 100 / (hit + miss)`, integer, floor. `rate=na` when
  `hit + miss == 0`.
- `reported`: responses that carried cache fields. `unreported`: responses that
  did not. A response with no cache fields increments `unreported` and
  contributes **nothing** to `hit`, `miss`, or `rate` — counting it as a miss
  would invent a number the provider never sent.
- The counter is per session, reset on a new session, and never reset by a
  turn. "A new session" means a new conversation, not a new process: a
  resumed conversation is the same session (spec 100 §1.1 item 4 resumes by
  id), so its counter is **stored with the session** and continues. Keeping it
  in process memory only would make every resume silently restart the totals,
  which is the cost question this section exists to answer. A file written
  before this contract carries no counter and starts at zero — the evidence of
  those turns was never recorded, and inventing it would be a measurement of
  nothing.

**The accumulation unit is the model response, not the user turn.** A turn that
runs tool rounds makes several requests, and each carries its own usage, so the
unit a provider reports tokens in is the response. Counting per user turn would
either discard the tokens of every round but one or invent a "turn usage" the
provider never sent. This is what "denominators are token sums, never turn
counts" already implies; it is stated here because the implementation had to
resolve it.

Two consequences, both load-bearing:

- `reported + unreported` counts **responses**, so a turn with three tool
  rounds moves those counters by three. The line is still printed once per
  turn, after its rounds have been counted.
- A provider that sends only one half of the pair (a hit with no miss, as
  OpenRouter did) adds only what it sent. The missing half stays missing
  rather than defaulting to `0`, which would read as a measured value in
  the rate. Path A keeps `prompt_cache_miss_tokens` on `Usage` when the
  official host sends it, and leaves the field absent when the payload did.

**When the line is emitted.** The gate is "the session has any evidence" — once
`reported > 0`, every later turn logs, including its unreported ones, because
that is the only way the unreported count is ever observed. A session whose
responses *all* arrived without cache fields logs nothing at all: `rate=na`
with `reported=0` on every turn is not a measurement, and printing it would
read like one.

Surface: the REPL / `run` turn line (spec 20 routing line already prints there).
Path A logs the same line once per turn from its in-memory session ledger
(`shell.turn.cache_session`), after that turn's main-loop responses have been
counted. The pager's hit-percentage chip is unchanged. The Path A counter is
not written to the session file; a new process starts it at zero, the same
way Path A's existing hit sum does. The persisted counter remains the overlay
record on the REPL / `run` path.

### 1.6 Session replay

Persist turns as JSONL (or equivalent) under user state dir. On load, **repair tool pairs** (spec 15) before send.

### 1.7 Compaction (M1 stub)

M1 may omit full compaction. If context overflows, fail with clear error **or** drop oldest **volatile** tail only — **never** mutate stable prefix in place without epoch bump. Full compaction policy → later ADR/spec 10 extension.

### 1.8 Cache evidence (with ADR 0005)

M1 acceptance:

1. Golden: `stable_prefix_bytes` equality test passes.  
2. Provider: parse cache hit/miss from usage when present; else dual-call substitute protocol logged.  
3. Attribution: §1.5.1 axes are named per component, with no false positive on
   an unchanged prefix and `unattributed` reserved for the coverage bug.  
4. Cumulative counter: §1.5.2 semantics, including the unreported-turn rule.  
5. Bench: §1.9 threshold. **Landed** — overlay `cache_guard_*` in
   `crates/dsb-agent/tests/cache_guard.rs`, and Path A `cache_guard_path_a_*`
   beside `spec10_path_a_assembly.rs`. Release wrapper
   `scripts/cache-guard.sh`; §4.4 names the tests.

### 1.9 Cache regression bench

`cache-first` is a claim about a curve, not a turn. The bench scores it.

**Scenarios.** A scenario is a scripted turn sequence (plain dialogue, long
dialogue, mixed message sizes, tool loop, long tool loop, with and without
reasoning round-trip) run against a **prefix-accounting mock provider**: the
mock derives `prompt_cache_hit_tokens` / `prompt_cache_miss_tokens` from the
request bytes it receives — how much of this request the previous request
already carried — not from constants in the test.

Two layers, both must pass:

1. **Exact layer — epochs.** Across a scenario that changes no §1.1 input, the
   number of distinct epochs is exactly **1**. A scenario that changes one
   input by design expects exactly **2**. This layer is not a threshold; it is
   the §1.3 invariant scored over a sequence instead of a pair.
2. **Rate layer — tokens.** Take `hit * 100 / (hit + miss)` per turn, then the
   average of the **last 3 turns**, and require **≥ 90%**. Floor: 3 turns is the
   smallest window that is not single-turn noise, and turn 1 legitimately has
   no cache. Threshold 90% is Reasonix's, kept deliberately: the economics it
   encodes (a tail that keeps 90% of prompt tokens warm) are the same, and a
   shared number makes the two projects comparable. Env-tunable as
   `DSB_CACHE_GUARD_THRESHOLD`.

**A scenario that cannot fail is not a scenario.** The harness must include a
negative control: one scenario that deliberately perturbs a §1.1 input mid-run
and must land **below** the threshold, and one that pins the exact epoch count.
If the negative control passes the rate layer, the mock is not accounting from
the request — fail the bench, not the scenario.

Wrapper: `scripts/cache-guard.sh`, release-gated like Reasonix's (env
`DSB_RELEASE_CACHE_GUARD=1`; skips when unset so it never slows the normal
suite).

**Landed (2026-09-26), two harnesses.**

* **Path B (overlay).** `crates/dsb-agent/tests/cache_guard.rs`. Scores the
  overlay builder and turn loop (`dsb-context` + `dsb-agent`), including
  live tool execution. §4.4 names those tests. `xai-grok-shell` does not
  depend on `dsb-context`, so this harness cannot see the bytes Path A sends.

* **Path A (vendored).** `spec10_path_a_cache_guard` in `xai-grok-shell`,
  wired from `spec10_path_a_assembly.rs`. A scenario request is the
  `messages` array of the `ChatCompletionRequest` produced after
  `assemble_spec10_path_a_turn` and `place_stable_body` — the placement
  `apply_spec10_to_conversation_request` writes at `turn.rs` before the
  sampler's `From` impl calls `conversation_to_chat_messages`. The mock
  accounts that array message by message (4 bytes = 1 token), same rule as
  the overlay mock. Distinct epochs are distinct `epoch_sha256_hex` values
  of those assemblies, cross-checked against the fingerprint of the latest
  system message on the wire (the effective prompt, §1.10 rule 1). The
  leading message stays one fingerprint across an append; counting epochs
  off the leading message would reject the append this contract requires.

Still not covered by either harness: a live API `prompt_cache_hit_tokens` /
`prompt_cache_miss_tokens` bill (the ratio is the mock's, and identical
bodies are not token-stable — §1.10), the full TUI turn loop (Path A tool
rounds are scripted conversation items, not a tool process), and sampler
fields outside `messages` (`model`, temperature, the `tools[]` array). A
green Path B run is not Path A evidence.

### 1.10 In-history stable-body update (Path A assembly)

`apply_spec10_to_conversation_request` builds the §1.1 stable body on every
Path A turn (`turn.rs`). U0.2 measured two shapes on Chat Completions.
Replacing the bytes of the leading system message returned
`cached_tokens = 0` on every sample. A later system message on an unchanged
leading system was accepted (HTTP 200) and did not force that zero, except
one identical insert that returned 0. Identical bodies were not
token-stable (641, 712, and 768 on one 792-token request). This section
therefore binds **bytes of system messages**, not a live token count.

The product template is still recovered from the earliest system message, as
the text before the `\n\n## Tools\n` marker. Every system message this
section writes is a full stable body, not a delta.

1. **The latest system message is the effective prompt.** The model treats
   the last system message as the current stable body. Earlier system
   messages stay in the transcript. This section does not delete them.
2. **Unchanged body.** When the newly assembled body equals the latest
   system message, ignoring trailing newlines, no system message is edited
   and none is added.
3. **First placement.** When no system message contains the `\n\n## Tools\n`
   marker, the body is written into the leading system message, or inserted
   at index 0 when the request has none. There is no earlier stable body
   to keep.
4. **Later change.** When a system message already contains that marker and
   the assembled body differs from the latest system message, every earlier
   system message stays byte-for-byte, and the new body is appended after
   the current items.
5. **What can move the body.** The tools document is inside the stable body
   (§1.1 item 2), so a tools, skills, environment, or project-instruction
   change appends under rule 4. The request `tools` array is still the full
   list on every request. This section does not define a tool-addition or
   tool-removal history message. Chat Completions has no field for one.
6. **Not this section.** `replace_or_insert_system_head` rewrites the stored
   leading system (model switch, memory) before this assembly runs. That
   rewrite is a different byte change. This section does not turn it into
   an append.
7. **Epoch.** The epoch stays the hash of the latest stable body (§1.5). An
   append changes the epoch. It does not change the bytes of the earlier
   system message.

A test that claims the append preserves a prefix derives the split from the
serialized messages: the shared byte-prefix length is the hit, and the
remainder is the miss. It does not hard-code a token count. The §1.9 Path A
bench scores the same idea over a scenario (`spec10_path_a_cache_guard`);
the §1.10 test keeps its own byte-prefix mock
(`in_history_update_appends_and_head_rewrite_breaks_the_byte_prefix`) so the
append rule does not depend on the bench.
## 2. Non-goals

- Guaranteeing 100% provider cache hits (server policy)  
- Stuffing full skill bodies into stable prefix  
- User-managed cache keys  

## 3. Failure modes

| Case | Behavior |
|------|----------|
| Non-deterministic field in system template | **Bug**; tests must catch |
| Tool schema key order shuffle | **Bug** |
| Compaction rewrites system tools mid-epoch without bump | **Bug** |
| Epoch moved, no component hash moved | `prefix_change=unattributed`; **bug in §1.5.1 coverage**; test must exist |
| Attribution names a component on an unchanged prefix | **Bug**; false positives make the line worthless |
| Detail line carries instruction content or a cwd value | **Bug**; §1.5.1 rule 3 |
| Turn without cache fields counted as a miss | **Bug**; §1.5.2 |
| `rate` computed over turn counts instead of token sums | **Bug**; §1.5.2 — a long turn would weigh the same as a short one |
| An all-unreported session printing `rate=na` per turn | **Bug**; it reads as a measurement of something |

## 4. Test plan (automated)

Names below are the real test names. `cargo test -p dsb-context -p dsb-agent`
runs the overlay rows. §4.6's counter tests live in
`crates/dsb-agent/src/cache_totals.rs`, with the turn-loop and persist→resume
cases in `crates/dsb-agent/src/loop_.rs`. §4.4's Path A rows, §4.5, and §4.7
run inside `xai-grok-shell` (`cargo test -p xai-grok-shell --lib`, from
`third_party/grok-build`; the bench filter is `cache_guard_path_a`).

### 4.1 Pre-existing (unchanged, must stay green)

| Test | Expect |
|------|--------|
| `prefix_stable_across_two_builds` | bytes equal |
| `prefix_changes_when_tool_added` | bytes differ; epoch differs |
| `prefix_no_timestamp` | fixture system template without clock |
| `sorted_tool_schema_keys_same_bytes` | permuting input map → same bytes |
| Negative: inject `Utc::now` into builder path | test fails if someone reintroduces |

### 4.2 §1.3 bytes and §1.5.1 shape

| Test | Expect |
|------|--------|
| `stable_prefix_bytes_golden_lock` | fixture epoch pinned to `c4b3cc9b…a20e` / 463 bytes; a change that moves prefix bytes fails with "this is cache-breaking for live sessions" |
| `shape_is_observational_not_byte_affecting` | two builds of one input → equal bytes, epoch, **and** shape |
| `shape_names_one_axis_per_changed_component` | each of the five components mutated in turn → exactly its own axis, no false positive |
| `attribute_detail_holds_names_not_content` | skill/instruction text absent from the detail line |
| `system_change_names_system_only` | prompt body changed → `system`, no detail (no finer axis) |
| `tool_names_added_removed_changed` | → `tools` + `tools.added=`/`changed=`/`removed=` with names |
| `tool_reorder_names_reordered` | tool order moved, entries identical → `tools.reordered` |
| `skill_names_added_removed_changed` | → `skills` + the exact `skills.<axis>=<name>` |
| `environment_axes_are_named_not_valued` | cwd change → `environment.cwd`; neither value appears |
| `multiple_axes_sorted_in_spec_order` | §1.1 order in the label |
| `unchanged_shape_reports_none` | → `none`, no detail |
| `epoch_moved_without_component_change_is_unattributed` | → `unattributed` (coverage bug, §3) |
| `log_block_carries_epochs_and_detail_only_when_present` | `prev=`/`cur=` present; detail line only when there is detail |
| `sub_hash_is_16_hex_chars` | truncation is what §5 says |

### 4.3 §1.5.1 rule 5 (session-carried baseline)

| Test | Expect |
|------|--------|
| `prefix_snapshot_roundtrip` | meta survives write→load with the shape intact |
| `save_without_snapshot_preserves_the_existing_baseline` | a plain save does not drop the baseline |
| `legacy_meta_without_snapshot_loads` | pre-§1.5.1 file loads; no baseline is invented |
| `resume_reports_prefix_change_by_axis` | stored instruction text vs changed file → `project_instructions`, no text in the line |
| `resume_with_unchanged_inputs_reports_none` | same inputs, two processes → `none` |
| `a_changed_tool_schema_is_named_tools_on_resume` | changed tool description → `tools` |

### 4.4 §1.9 cache guard (landed)

| Test | Expect |
|------|--------|
| `cache_guard_mock_accounts_from_request_bytes` | the mock computes the carry from the request: zero on the first request, the whole previous prompt on an append, zero when the leading message moved, everything on a replay |
| `cache_guard_unchanged_prefix_is_one_epoch_and_passes` | no §1.1 input changes → exactly 1 epoch and the last-3 average holds the threshold |
| `cache_guard_by_design_change_is_two_epochs` | one §1.1 input changed by design (a rebuild = a resume) → exactly 2 epochs, and the tail recovers above the threshold |
| `cache_guard_negative_control` | a perturbed prefix lands below the threshold, an unchanged one does not |
| `cache_guard_tool_loop_stays_above_threshold` | the tool-loop shape, real tool execution included → exactly 1 epoch and the last-3 average holds |
| `cache_guard_mixed_message_sizes_hold_the_threshold` | mixed message sizes move the fresh-tail share without a §1.1 change → exactly 1 epoch and the last-3 average holds |
| `cache_guard_release_suite` | the full scenario matrix; gated by `DSB_RELEASE_CACHE_GUARD=1`, skips when unset |

Runner: `crates/dsb-agent/tests/cache_guard.rs`; wrapper
`scripts/cache-guard.sh`.

Path A, same contract, different bytes. The mock reads the Chat Completions
`messages` produced from `assemble_spec10_path_a_turn` + `place_stable_body`.
Tool rounds are scripted items, not a live tool process. The release-suite
test is gated by the same env var; the wrapper runs it only when
`xai-grok-shell` has already been compiled into the vendored target
(`libxai_grok_shell-*.rlib`, or the `xai_grok_shell-*` test harness).

| Test | Expect |
|------|--------|
| `cache_guard_path_a_mock_accounts_from_request_bytes` | the mock computes the carry from the mapped messages: zero on the first request, the whole previous prompt on an append, zero when the leading message moved, everything on a replay |
| `cache_guard_path_a_scored_bytes_match_apply_spec10` | the bench's placement matches `apply_spec10_to_conversation_request` on the same workspace |
| `cache_guard_path_a_tail_window_clamps_and_rejects_a_short_run` | the rate is the integer mean of the last `min(3, requests)` rates; fewer than 3 requests fail |
| `cache_guard_path_a_unchanged_prefix_is_one_epoch_and_passes` | no §1.1 input changes → exactly 1 epoch, one leading fingerprint, and the last-3 average holds the threshold |
| `cache_guard_path_a_by_design_change_is_two_epochs` | one §1.1 input changed by design → exactly 2 epochs (latest system message moves, leading message does not) and the tail recovers above the threshold |
| `cache_guard_path_a_negative_control` | a perturbed prefix lands below the threshold, an unchanged one does not |
| `cache_guard_path_a_tool_loop_stays_above_threshold` | scripted tool-call / tool-result rounds → exactly 1 epoch and the last-3 average holds |
| `cache_guard_path_a_mixed_message_sizes_hold_the_threshold` | mixed message sizes move the fresh-tail share without a §1.1 change → exactly 1 epoch and the last-3 average holds |
| `cache_guard_path_a_release_suite` | the full scenario matrix; gated by `DSB_RELEASE_CACHE_GUARD=1`, skips when unset |

Runner: `third_party/grok-build/crates/codegen/xai-grok-shell/src/session/helpers/spec10_path_a_cache_guard.rs`.

### 4.5 §1.10 wire placement

| Test | Expect |
|------|--------|
| `in_history_update_appends_and_head_rewrite_breaks_the_byte_prefix` | A later stable-body change leaves the leading system bytes intact and appends the new body. A byte-prefix mock, derived from the serialized messages, counts the old body inside the shared prefix. Replacing the leading system with that same new body does not. A second apply of the same body adds nothing. |

### 4.6 §1.5.2 session counter (landed)

| Test | Expect |
|------|--------|
| `cache_totals_accumulate_and_track_unreported` | token sums accumulate; an unreported response moves only `unreported`; `rate=80` for 120/150 |
| `an_unreported_turn_does_not_contaminate_the_totals` | same totals with and without a gap — the gap shows only in `unreported` |
| `rate_is_na_when_no_tokens_were_reported` | `hit + miss == 0` → `na`, not `0` |
| `the_line_is_gated_on_evidence_anywhere_in_the_session` | `has_evidence()` false while every response is unreported |
| `rate_floors_integer_division` | 1/3 → 33, 2/3 → 66, 100/101 → 99 |
| `a_missing_half_is_not_defaulted_to_a_zero_miss` | hit-only response → miss stays 0, rate 100 |
| `substitute_protocol_counts_as_unreported` | dual-call latency measurement is not a cache-field response |
| `a_new_session_starts_from_zero` | a fresh counter is a fresh session |
| `rate_survives_token_sums_past_u64_multiplication` | `u128` arithmetic; no wrap |
| `cache_session_line_is_emitted_once_per_turn_and_accumulates` | the line through a real turn loop: once per turn, and turn 2's missing fields leave `rate=80` alone |
| `an_all_unreported_session_logs_nothing` | a session with no evidence prints no line |
| `a_new_agent_starts_the_counter_at_zero` | two agents do not share a counter |
| `cache_totals_roundtrip_with_the_session` | the counter survives write→load, so a resume continues it |
| `save_without_totals_preserves_the_existing_counter` | a plain save does not restart a resumed session's totals |
| `legacy_meta_without_cache_totals_loads` | a pre-§1.5.2 file loads; no counter is invented |
| `a_resumed_session_continues_its_cache_totals` | the persist→resume pair: both halves of "a resume continues the count" |

Runner: `crates/dsb-agent/src/cache_totals.rs`, with the turn-loop and
persist→resume cases in `crates/dsb-agent/src/loop_.rs`.

### 4.7 Path A attribution (landed)

| Test | Expect |
|------|--------|
| `path_a_shape_is_the_concatenated_documents` | The stable body is the five documents concatenated, and the tools/system hashes are those documents |
| `path_a_first_sight_and_unchanged_epoch_have_no_change_line` | No baseline, and a repeated epoch, both return no `prefix_change=` line |
| `path_a_tool_add_names_tools_added` | A new tool is `tools` / `tools.added=<name>`, and the session observer agrees |
| `path_a_system_change_names_system_only` | A system-template change names `system` and has no detail line |
| `path_a_tool_reorder_names_reordered` | Reordering tools moves the epoch and the detail is `tools.reordered` |
| `path_a_skill_description_names_the_skill_not_the_text` | `skills.changed=<name>`; the description text is not in the line |
| `path_a_environment_detail_names_the_axis_not_the_cwd` | `environment.cwd`; the cwd value is not in the line |
| `path_a_project_instructions_have_no_finer_axis` | `project_instructions` with no detail line and no instruction text |
| `path_a_axes_stay_in_section_order` | Moved axes join in §1.1 order |
| `path_a_epoch_moved_without_component_change_is_unattributed` | Epoch differs, component hashes do not: `unattributed`, not a new axis |

## 5. Implementation notes

- Crate: `dsb-context` (ADR 0004) — builder, shape, classifier.  
- Hash: SHA-256 of raw bytes for epoch logging; component hashes use the same
  function over the same document strings, so attribution and epoch cannot
  disagree. Detail sub-axis hashes are truncated to 16 hex chars; they name a
  change, they do not authenticate it.  
- `dsb-agent` carries the shape on the built prefix and persists it with the
  session; `dsb-cli` prints the lines. That is the overlay resume path.
- Path A carries its own shape inside `assemble_spec10_path_a_turn`, over the
  five documents that function concatenates. `turn.rs` logs
  `prefix_change=` only when that epoch differs from the previous assembly
  of the same session in this process. The Path A baseline is not persisted.  
- The §1.5.2 counter is `crates/dsb-agent/src/cache_totals.rs`, carried on
  `Agent` and stored in the session's `meta` line (spec 100 §1 item 7) beside
  the prefix shape. It is folded in at the model response and printed once per
  turn, after that turn's rounds. Storing it is what makes "reset on a new
  session" true of a *conversation*: a resume continues the count.  
- Path A does not call `dsb-agent`. It keeps its own §1.5.2 counter on the
  in-memory session ledger (`xai-chat-state` `CacheSessionTotals`) and logs
  the line from `emit_turn_completed`. `prompt_cache_miss_tokens` is a field
  on Chat Completions `Usage`, so serde no longer drops it. The pager chip
  still reads `cached_read_tokens` / `input_tokens`.
