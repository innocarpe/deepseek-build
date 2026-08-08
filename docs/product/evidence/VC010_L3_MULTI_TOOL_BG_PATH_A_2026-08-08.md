# VC010 — L3 multi-tool parallel + background shell Path A R0A

| Field | Value |
|-------|--------|
| **Story** | **VC010** — hermetic **Path A** R0A for Spec 50 multi-tool **read-only parallel**, **mutate serial**, and **background shell collect-by-id** |
| **Plan** | `vision-complete-5x` |
| **Date** | 2026-08-08 |
| **Status** | **READY** — Path A R0A multi-tool + bg green; gates green; unversioned; stacked on #141 |
| **SemVer** | **none** (this story does **not** bump product version) |
| **Depends on** | **VC009** cache visibility (open PR **#141** `vc009-cache-visibility`); Track C board also lists soft depend on VC006 hearts |
| **Board** | [`VISION_COMPLETE_5X_GOALS.md`](../VISION_COMPLETE_5X_GOALS.md) · DAG [`WAVE_5x_VISION_PR_DAG.md`](../WAVE_5x_VISION_PR_DAG.md) |
| **Normative** | [`docs/specs/50-parallelism-background.md`](../../specs/50-parallelism-background.md) · [`HARNESS_PHILOSOPHY.md`](../../architecture/HARNESS_PHILOSOPHY.md) § L3 · owner-bar L3-50-* |
| **Prior** | G010 L3 stamp + offline smoke ([`G010_L3_UNDER_HEARTS_2026-08-07.md`](./G010_L3_UNDER_HEARTS_2026-08-07.md)); thin `dsb-agent` parallel + `dsb-tools` bg unit greens |

**This file is the mandatory ultragoal PR unit plan for VC010 plus (later) implementation evidence.**
It does **not** claim VISION L3 / **5.4.0** freeze complete. Thin Path B (`dsb-agent` loop / `dsb-tools` bg store) alone is **not** Path A R0A proof. Live API multi-tool dogfood remains residual unless a key is present and explicitly recorded.

---

## 0. Floor and dependency facts

### 0.1 Live floor (story open; 2026-08-08)

| Probe | Live result |
|-------|-------------|
| This worktree branch | `vc010-l3-parallel-bg` (forked at VC009 tip `3e8a5b5`) |
| Stack base for feature commits / PR base | **`vc009-cache-visibility`** (open PR **#141**); **not** `origin/main` until after #141 merges |
| `git show origin/main:Cargo.toml` version | **`5.2.2`** |
| `npm view @innocarpe/deepseek-build version` | **`5.2.2`** |
| `gh release list` Latest | **`v5.2.2`** |
| Working tree product version (stack tip) | **`5.3.0`** (from VC006 cut on stack; carried through VC007–VC009) |
| Board text residual | Track C target minor still **`5.4.0`** for VC010–VC013; this story is **unversioned** evidence only |
| G010 residual | Live multi-tool concurrent dogfood / live bg collect need API key (`--extended`); hermetic public-entry multi-tool + bg R0A was **weak** |

### 0.2 Floor interpretation (fail-close)

- **Live product floor is `origin/main` = `5.2.2`.** npm + GitHub Release aligned at open.
- Stack already carries product **`5.3.0`**. **Do not bump SemVer** on VC010. Do **not** cut **`5.4.0`** here (that is VC013).
- Board’s “VC010 part of 5.4.0” remains true as **train membership**, not as this PR’s packaging action.
- **Open as one stacked PR** with base **`vc009-cache-visibility`** and body **`Depends on #141`**. **Do not merge** this PR in-story.
- Claims must be **public Path A only**: `deepseek-build` / `dsb` → product agent (no `DEEPSEEK_BUILD_AGENT_BIN` override for the public-entry claim). Thin unit greens may support honesty but must not be sole proof.
- Fail closed if multi-tool batch, mutate application, or bg collect-by-id cannot be proven under hermetic scripted DeepSeek — record residual; do not invent green.

---

## 1. Why this PR (one sentence)

Close the Grok L3 throughput gap that G010 left weak: prove on **public Path A** (hermetic scripted wire) that a multi-tool turn can exercise **read-only multi-tool fan-out**, **mutating work remains serial/safe**, and **background shell + collect-by-id** returns stdout — without claiming live API dogfood or a SemVer cut.

---

## 2. Call-path map (inspected before design)

| Layer | Path | Role today (pre-VC010 residual) |
|-------|------|----------------------------------|
| Public product | `deepseek-build` / `dsb` → `deepseek-build-agent` (vendored Grok) | **Path A** product default |
| Product multi-tool dispatch | Grok shell `execute_tool_calls` / batch + same-path lock (`lock_path_for_args`) | Model-emitted multi `tool_calls` in one assistant message; concurrent where unlocked |
| Spec 50 classifier (product names) | `dsb_agent::is_mutating_tool` / `partition_indices` via `stamp_path_a_l3` name map | Launch stamp proves RO/mutate partition for `read_file` / `search_replace` / `run_terminal_command` |
| Thin Path B parallel | `dsb-agent` `handle_tool_calls_batch` | Unit + loop path — **not** Path A R0A |
| Background shell (Path A) | Grok `run_terminal_command` + `is_background` → `task_id` | Collect via `get_command_or_subagent_output` / `get_task_output` (`task_ids`) |
| Thin Path B bg | `dsb-tools` `bash` + `background: true` → `bash_collect` | Overlay / thin tools — **not** Path A R0A |
| Hermetic fixture | `scripts/lib/scripted_deepseek_server.py` | Has VC006 multi-step edit scenarios; **missing** multi-tool-in-one-message + bg collect scenarios |
| Public entry harness | `scripts/test-path-a-public-entry-e2e.sh` / VC006 R0A | Text-pong + snippet scenarios; not multi-tool parallel / bg collect |
| Offline L3 smoke | `scripts/test-l3-smoke.sh` | L3.2 live bg is env-gated; offline SKIP |

### Target VC010 R0A contract (Path A public agent)

Public CLI → `agent_launch` → product agent (no `DEEPSEEK_BUILD_AGENT_BIN` for the public-entry claim) with hermetic home + scripted DeepSeek wire:

#### 2.1 Multi-tool read-only batch (Spec 50 T1 spirit)

1. Seed workspace `a.txt` / `b.txt` with distinct markers.
2. Scripted model returns **two** `read_file` tool calls **in one** assistant message.
3. Agent executes both; both tool results return; final text token `multi-read-parallel-ok`.
4. Wire proves multi `tool_calls` in a single response; public entry still writes `path_a_l3.txt` (classifier stamp).

#### 2.2 Mutate serial / mixed honesty (Spec 50 T2 spirit)

1. Multi-read batch first (RO fan-out), then a **mutating** `search_replace` using minted `snippet_id` (Path A Spec 45 honesty).
2. Disk golden on the edited file; no claim that two concurrent edits raced safely without locks.
3. Final token `mixed-mutate-serial-ok`.

#### 2.3 Background shell collect-by-id (Spec 50 §1.4 / L3-50-3)

1. Scripted model calls `run_terminal_command` with **`is_background`: true** and `echo bg-ok-77` (short sleep allowed).
2. Tool result includes a **task id**; next scripted turn calls `get_command_or_subagent_output` with `task_ids` + positive `timeout_ms`.
3. Collected output contains **`bg-ok-77`**; final token `bg-collect-ok`.

#### 2.4 Public-path only claims

| Allowed as Path A proof | Not Path A proof alone |
|-------------------------|------------------------|
| `deepseek-build agent -p` / `dsb agent -p` under hermetic home | `cargo test -p dsb-agent parallel` sole green |
| Wire JSONL + META under `docs/product/evidence/` | Thin `dsb-tools` bg unit only |
| `path_a_l3.txt` stamp from public launch | Live L3.2 without key (SKIP) claimed PASS |

### Explicit non-claims

| Non-claim | Residual |
|-----------|----------|
| Live multi-tool concurrent dogfood without scripted model | Needs API key; optional `--extended` L3 smoke |
| Subagent + worker cache R0A | **VC011** |
| Worktree dogfood | **VC012** |
| SemVer / npm / GitHub Release **5.4.0** cut | **VC013** |
| Thin Path B is product default | Path A is Grok product agent |
| Wall-clock proof that RO tools always overlap in time | Assert multi-call batch + results + product dispatch/stamp honesty; wall-clock residual |

---

## 3. PR units (ordered atomic commits — **one** stacked PR)

This story ships as **one unversioned PR** with atomic Conventional Commits (not multiple stack slots).

### PR unit 1 — `docs(product): VC010 L3 multi-tool + bg Path A plan`

- **Intent:** Lock stack base, call-path map, acceptance matrix, SemVer non-claims before source edits.
- **Touches:** `docs/product/evidence/VC010_L3_MULTI_TOOL_BG_PATH_A_2026-08-08.md`
- **Depends on:** VC009 stack tip
- **SemVer:** none
- **Tests:** n/a (docs)

### PR unit 2 — `test(scripts): hermetic multi-tool + bg Path A fixture scenarios`

- **Intent:** Scripted DeepSeek emits multi-`tool_calls` RO batch, mixed mutate, and bg + collect-by-id sequences.
- **Touches:** `scripts/lib/scripted_deepseek_server.py`
- **Depends on:** unit 1
- **SemVer:** none
- **Tests:** exercised by unit 3 harness

### PR unit 3 — `test(scripts): VC010 Path A public-entry multi-tool + bg R0A`

- **Intent:** Public `deepseek-build`/`dsb` agent hermetic R0A for the three scenarios; wire + META artifacts.
- **Touches:** `scripts/test-path-a-vc010-r0a.sh`
- **Depends on:** unit 2
- **SemVer:** none
- **Tests:** `./scripts/test-path-a-vc010-r0a.sh`

### PR unit 4 — `test(agent): product-name classifier mapping for Spec 50 stamp honesty`

- **Intent:** Unit-test that product tool names map into RO/mutate partitions used by `stamp_path_a_l3` (public-path honesty support).
- **Touches:** `crates/dsb-agent/src/parallel.rs` (tests)
- **Depends on:** unit 1
- **SemVer:** none
- **Tests:** `cargo test -p dsb-agent partition` (support only; not sole R0A)

### PR unit 5 — `docs(product): VC010 READY evidence + independent review`

- **Intent:** Record gate results, wire samples, READY label, independent Grok review, residuals.
- **Touches:** evidence docs under `docs/product/evidence/`
- **Depends on:** units 2–4 green
- **SemVer:** none
- **Tests:** gates below

## Sequential vs parallel

### Sequential (must order)

1. unit 1 → unit 2 (plan before fixture)
2. unit 2 → unit 3 (harness needs scenarios)
3. units 2–4 → unit 5 (READY only after green)

### Parallel (safe concurrent)

- unit 4 ∥ unit 2/3 (disjoint: agent unit tests vs scripts) after unit 1

## Atomic commits (branch)

```text
docs(product): VC010 L3 multi-tool + bg Path A plan
test(scripts): hermetic multi-tool + bg Path A fixture scenarios
test(scripts): VC010 Path A public-entry multi-tool + bg R0A
test(agent): product-name classifier mapping for Spec 50 stamp honesty
docs(product): VC010 READY evidence and independent review
```

---

## 4. Acceptance matrix

| ID | Check | Evidence |
|----|-------|----------|
| **A1** | Multi-read multi-`tool_calls` in one scripted response on Path A | wire + final `multi-read-parallel-ok` |
| **A2** | Mixed path: multi-read then `search_replace` with `snippet_id`; disk golden | wire + META + file content |
| **A3** | Background `is_background` + collect `task_ids` yields `bg-ok-77` | wire + agent/collect content |
| **A4** | Public entry (no `DEEPSEEK_BUILD_AGENT_BIN`); `path_a_l3` stamp present | META |
| **A5** | Owner-bar + path-linkage + heart regression green | command logs |
| **A6** | No SemVer bump in this PR | `Cargo.toml` / `package.json` still **5.3.0** |
| **A7** | Independent Grok review READY | review evidence file |

---

## 5. Gates (required before READY)

```bash
./scripts/check-path-a-linkage.sh
./scripts/test-owner-bar.sh
./scripts/test-heart-regression.sh
./scripts/test-path-a-vc010-r0a.sh
# support (not sole proof):
cargo test -p dsb-agent partition is_mutating
cargo test -p dsb-tools bg
```

Restore any generated TSV side-effects to HEAD if gates rewrite them. Clean vendor/agent `target/` build debris after agent builds if needed.

---

## 6. READY evidence

**Status: READY** (implementation complete; independent review file sibling).

### 6.1 Commands

| Command | Result |
|---------|--------|
| `./scripts/test-path-a-vc010-r0a.sh --skip-build` | **PASS** (multi-read-parallel, mixed-mutate-serial, bg-collect-by-id) |
| `cargo test -p dsb-agent product_path_a_names_partition` | **PASS** (support only) |
| `cargo test -p dsb-tools bg` | **PASS** (support only) |
| `./scripts/check-path-a-linkage.sh` | **PASS** |
| `./scripts/test-owner-bar.sh` | **PASS** (60/60; TSV side-effect restored) |
| `./scripts/test-heart-regression.sh` | **PASS** (PATH_A_E2E SKIP default; L3 offline PASS) |
| SemVer on branch | **`5.3.0`** unchanged (no bump) |

### 6.2 Wire + stamp sample

| Scenario | Fixture batch | Public-entry proof |
|----------|---------------|--------------------|
| `multi-read-parallel` | `response_tool_calls` with **2× `read_file`** | final `multi-read-parallel-ok`; ≥2 tool results; `path_a_l3` stamp |
| `mixed-mutate-serial` | multi-read then `search_replace` with **a.txt** `snippet_id` | disk `a.txt=alpha-mutated`; final `mixed-mutate-serial-ok` |
| `bg-collect-by-id` | `run_terminal_command` `is_background:true` → `get_command_or_subagent_output` `task_ids` | tool output contains **`bg-ok-77`**; final `bg-collect-ok` |

Artifacts:

| Path | Role |
|------|------|
| [`PATH_A_R0_VC010_multi-read-parallel_WIRE_last.jsonl`](./PATH_A_R0_VC010_multi-read-parallel_WIRE_last.jsonl) | Multi-read RO batch wire |
| [`PATH_A_R0_VC010_mixed-mutate-serial_WIRE_last.jsonl`](./PATH_A_R0_VC010_mixed-mutate-serial_WIRE_last.jsonl) | Mixed mutate wire |
| [`PATH_A_R0_VC010_bg-collect-by-id_WIRE_last.jsonl`](./PATH_A_R0_VC010_bg-collect-by-id_WIRE_last.jsonl) | Bg + collect wire |
| Matching `*_META_last.txt` | Public-entry meta |
| [`PATH_A_R0_VC010_L3_last.txt`](./PATH_A_R0_VC010_L3_last.txt) | Spec 50 classifier stamp from public launch |

Path A L3 stamp sample:

```text
max_parallel_readonly=8
ro_indices=[0, 4]
mu_indices=[1, 2, 3, 5]
bash_mutating=true
worker_epochs_match=true
subagents_enabled_in_config=true
```

### 6.3 What shipped

1. Hermetic fixture multi-tool SSE + `response_tool_calls` wire records.
2. Public `deepseek-build`/`dsb` agent R0A harness `scripts/test-path-a-vc010-r0a.sh`.
3. Product-name → Spec 50 partition unit test (stamp honesty support).
4. Honest non-claims: no SemVer, no live API sole proof, no VC011 subagent R0A.

### 6.4 Explicit residuals at READY

- Live L3.1–L3.5 without API key remain **SKIP**.
- Wall-clock concurrency not claimed; multi-call batch + results + serial mutate path claimed.
- VC011 subagent + worker cache R0A, VC012 worktree, VC013 **5.4.0** cut remain out of scope.
- Product builder disables `enabled_background` when subagent types are emptied — R0A keeps subagent tools available; do not strip `Agent`/`spawn_subagent` in this harness.

### 6.5 Independent review

See [`VC010_INDEPENDENT_REVIEW_2026-08-08.md`](./VC010_INDEPENDENT_REVIEW_2026-08-08.md).

## 7. Stack / PR open checklist

- [x] Branch `vc010-l3-parallel-bg` based on `vc009-cache-visibility`
- [x] First commit = this plan (before source edits)
- [x] Atomic commits as §3 (plus small fix commits for R0A honesty)
- [ ] `gh pr create --base vc009-cache-visibility` with **English** body + labels (`test`, `area/orchestrator` or `area/tools`)
- [ ] Body includes **Depends on #141**
- [ ] `gh pr view --json title,labels,url` shows ≥1 kind label
- [x] **Do not merge**; **do not bump SemVer** (still 5.3.0)
