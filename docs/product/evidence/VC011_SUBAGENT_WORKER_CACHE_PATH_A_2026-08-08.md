# VC011 — Subagent + worker cache Path A R0A

| Field | Value |
|-------|--------|
| **Story** | **VC011** — hermetic **Path A** R0A for Spec **60** explore + implement-class subagents, worker cache law (stable prefix epoch), and parent snippet invalidation after worker mutation |
| **Plan** | `vision-complete-5x` |
| **Date** | 2026-08-08 |
| **Status** | **PLAN** — plan-first; implementation + READY evidence follow in later atomic commits |
| **SemVer** | **none** (this story does **not** bump product version) |
| **Depends on** | **VC010** L3 multi-tool + bg Path A R0A (open PR **#142** `vc010-l3-parallel-bg`) |
| **Board** | [`VISION_COMPLETE_5X_GOALS.md`](../VISION_COMPLETE_5X_GOALS.md) · DAG [`WAVE_5x_VISION_PR_DAG.md`](../WAVE_5x_VISION_PR_DAG.md) |
| **Normative** | [`docs/specs/60-subagents.md`](../../specs/60-subagents.md) · Spec 45 snippet invalidation · [`HARNESS_PHILOSOPHY.md`](../../architecture/HARNESS_PHILOSOPHY.md) § L3 under L2 · owner-bar L3-60-* · vision V3-60-1/2/3 |
| **Prior** | G010 L3 stamp + thin units ([`G010_L3_UNDER_HEARTS_2026-08-07.md`](./G010_L3_UNDER_HEARTS_2026-08-07.md)); VC010 multi-tool + bg Path A R0A ([`VC010_L3_MULTI_TOOL_BG_PATH_A_2026-08-08.md`](./VC010_L3_MULTI_TOOL_BG_PATH_A_2026-08-08.md)) |

**This file is the mandatory ultragoal PR unit plan for VC011 plus (later) implementation evidence.**  
It does **not** claim VISION L3 / **5.4.0** freeze complete. Thin Path B (`dsb-agent` `subagent` tool units) alone is **not** Path A R0A proof. Live API `spawn_subagent` dogfood remains residual unless a key is present and explicitly recorded.

---

## 0. Floor and dependency facts

### 0.1 Live floor (story open; 2026-08-08)

| Probe | Live result |
|-------|-------------|
| This worktree branch | `vc011-subagent-worker-cache` (forked at VC010 tip `9d6f1d0`) |
| Stack base for feature commits / PR base | **`vc010-l3-parallel-bg`** (open PR **#142**); **not** `origin/main` until after #142 merges |
| `git show origin/main:Cargo.toml` version | **`5.2.2`** |
| `npm view @innocarpe/deepseek-build version` | **`5.2.2`** |
| `gh release list` Latest | **`v5.2.2`** |
| Working tree product version (stack tip) | **`5.3.0`** (from VC006 cut on stack; carried through VC007–VC010) |
| Board text residual | Track C target minor still **`5.4.0`** for VC010–VC013; this story is **unversioned** evidence only |
| G010 residual | Live `spawn_subagent` dogfood needs API key (`--extended`); hermetic public-entry explore/implement R0A was **weak** |
| VC010 residual | Explicitly deferred **VC011** subagent + worker cache Path A R0A |

### 0.2 Floor interpretation (fail-close)

- **Live product floor is `origin/main` = `5.2.2`.** npm + GitHub Release aligned at open.
- Stack already carries product **`5.3.0`**. **Do not bump SemVer** on VC011. Do **not** cut **`5.4.0`** here (that is VC013).
- Board’s “VC011 part of 5.4.0” remains true as **train membership**, not as this PR’s packaging action.
- **Open as one stacked PR** with base **`vc010-l3-parallel-bg`** and body **`Depends on #142`**. **Do not merge** this PR in-story.
- Claims must be **public Path A only**: `deepseek-build` / `dsb` → product agent (no `DEEPSEEK_BUILD_AGENT_BIN` override for the public-entry claim). Thin unit greens may support honesty but must not be sole proof.
- Fail closed if explore/implement-class spawn, worker epoch match, or parent snippet invalidation after worker mutation cannot be proven under hermetic scripted DeepSeek — record residual; do not invent green.

---

## 1. Why this PR (one sentence)

Close the Grok L3 Spec **60** gap that G010 left stamp/unit-only and VC010 deferred: prove on **public Path A** (hermetic scripted wire) that the product can **spawn explore + implement-class subagents**, that **worker cache law** (stable prefix epoch equality) is visible on the public launch path, and that **worker mutation invalidates parent snippets** — without claiming live API sole proof or a SemVer cut.

---

## 2. Call-path map (inspected before design)

| Layer | Path | Role today (pre-VC011 residual) |
|-------|------|----------------------------------|
| Public product | `deepseek-build` / `dsb` → `deepseek-build-agent` (vendored Grok) | **Path A** product default |
| Path A subagent surface | Grok tool **`spawn_subagent`** (aliases `Agent` / `Task`) | Types: `explore`, `general-purpose` (implement-class), `plan` |
| Product docs | [`docs/user-guide/11-subagents.md`](../../user-guide/11-subagents.md) | Path A ≠ thin `dsb-tools` `subagent` helper |
| Spec 60 thin Path B | `dsb-agent` tool **`subagent`** `kind=explore\|implement` | In-process heuristic workers; unit greens exist |
| Worker cache law (product stamp) | `dsb_cli::agent_launch::stamp_path_a_l3` → `worker_stable_prefix` dual build | Writes `path_a_l3.txt` `worker_epochs_match=true` on every public launch |
| Parent after worker (thin) | `dsb_agent::parent_after_worker` → `snippets.expire_all()` when `mutated` | Unit `implement_write_mutates`; **not** Path A R0A alone |
| Path A snippet invalidation | Spec 45 Path A write/bash laws (VC005); content/file_version stale | Worker-mutation → **parent table expire** is Spec 60 product default (expire all) |
| Hermetic fixture | `scripts/lib/scripted_deepseek_server.py` | Has VC010 multi-tool + bg; **missing** spawn_subagent explore/implement scenarios |
| Public entry harness | VC010 `scripts/test-path-a-vc010-r0a.sh` | Multi-tool + bg; not subagent R0A |
| Offline L3 smoke | `scripts/test-l3-smoke.sh` | L3.5 live spawn is `--extended` / env-gated |

### Target VC011 R0A contract (Path A public agent)

Public CLI → `agent_launch` → product agent (no `DEEPSEEK_BUILD_AGENT_BIN` for the public-entry claim) with hermetic home + scripted DeepSeek wire:

#### 2.1 Explore subagent dogfood (V3-60-1 / Spec 60 T1 spirit)

1. Seed workspace with a distinctive marker file (`explore-marker.txt`).
2. Scripted parent model emits **`spawn_subagent`** with `subagent_type: explore` (or product-accepted explore alias) and a task that only **reads/lists**.
3. Child worker (same hermetic `base_url` / config) completes under scripted wire; parent receives a structured spawn/collect result.
4. Final parent text token **`explore-subagent-ok`**.
5. Honesty: explore must not be the sole proof of mutating paths.

#### 2.2 Implement-class subagent mutation (V3-60-1 / Spec 60 T3 spirit)

1. Seed workspace file that parent has (or will have) a Path A snippet for.
2. Scripted parent emits **`spawn_subagent`** with implement-class type **`general-purpose`** (product map: full tools; Spec 60 “implement” kind equivalent on Path A).
3. Child mutates a known path (write/create or search_replace under yolo hermetic policy).
4. Disk golden proves mutation; final token **`implement-subagent-ok`**.

#### 2.3 Worker cache law on public path (V3-60-2 / Spec 60 T2)

1. Every public-entry scenario must write **`path_a_l3.txt`** under hermetic `DEEPSEEK_BUILD_HOME`.
2. Assert **`worker_epochs_match=true`**, dual `worker_epoch_a/b` non-error, kinds `explore` + `implement` present, **`subagents_enabled_in_config=true`**.
3. Support-only (not sole proof): `cargo test -p dsb-agent cache_law` / `worker_stable_prefix` units remain green.

#### 2.4 Parent snippet invalidation after worker mutation (V3-60-3 / Spec 60 T3)

1. Prefer **Path A public** proof: parent mints/holds a snippet for a path; implement-class worker mutates that path; parent’s subsequent edit with the pre-mutation `snippet_id` is rejected **or** product stamp/log proves table expire.
2. If Path A Grok does not yet expire the parent snippet table on child mutation, **fail-closed residual** + keep thin-path unit `implement_write_mutates` + `parent_after_worker` as support evidence (must not claim Path A sole green for V3-60-3).
3. Product default per Spec 60: expire **all** workspace snippets after implement mutation when wired.

#### 2.5 Public-path only claims

| Allowed as Path A proof | Not Path A proof alone |
|-------------------------|------------------------|
| `deepseek-build agent -p` / `dsb agent -p` under hermetic home | `cargo test -p dsb-agent subagent` sole green |
| Wire JSONL + META under `docs/product/evidence/` | Thin `ToolName::Subagent` dispatch only |
| `path_a_l3.txt` stamp from public launch | Live L3.5 without key (SKIP) claimed PASS |
| `spawn_subagent` / `Agent` on product tool surface | Stripping subagent tools (also disables bg per product builder) |

### Explicit non-claims

| Non-claim | Residual |
|-----------|----------|
| Live multi-model concurrent subagent fleets without scripted model | Needs API key; optional `--extended` L3.5 |
| Nested unbounded subagent trees | Spec 60 non-goal |
| Mandatory worktree for every implement worker | **VC012** |
| SemVer / npm / GitHub Release **5.4.0** cut | **VC013** |
| Thin Path B `subagent` tool is product default | Path A is Grok `spawn_subagent` |
| Wall-clock proof workers always overlap | Not required |

---

## 3. PR units (ordered atomic commits — **one** stacked PR)

This story ships as **one unversioned PR** with atomic Conventional Commits (not multiple stack slots).

### PR unit 1 — `docs(product): VC011 subagent + worker cache Path A plan`

- **Intent:** Lock stack base, call-path map, acceptance matrix, SemVer non-claims before source edits.
- **Touches:** `docs/product/evidence/VC011_SUBAGENT_WORKER_CACHE_PATH_A_2026-08-08.md`
- **Depends on:** VC010 stack tip (`9d6f1d0` / PR #142)
- **SemVer:** none
- **Tests:** n/a (docs)

### PR unit 2 — `test(scripts): hermetic Path A spawn_subagent fixture scenarios`

- **Intent:** Scripted DeepSeek emits parent `spawn_subagent` explore + implement-class sequences (and child turns when shared base_url is used).
- **Touches:** `scripts/lib/scripted_deepseek_server.py`
- **Depends on:** unit 1
- **SemVer:** none
- **Tests:** exercised by unit 3 harness

### PR unit 3 — `test(scripts): VC011 Path A public-entry subagent + worker cache R0A`

- **Intent:** Public `deepseek-build`/`dsb` agent hermetic R0A for explore spawn, implement-class mutate, stamp cache law; wire + META artifacts.
- **Touches:** `scripts/test-path-a-vc011-r0a.sh`
- **Depends on:** unit 2
- **SemVer:** none
- **Tests:** `./scripts/test-path-a-vc011-r0a.sh`

### PR unit 4 — `test(agent): Spec 60 worker cache + parent_after_worker honesty support`

- **Intent:** Keep/strengthen unit coverage for explore deny-write, epoch equality, implement mutate → parent snippet expire (support only).
- **Touches:** `crates/dsb-agent/src/subagent.rs` (tests) as needed
- **Depends on:** unit 1
- **SemVer:** none
- **Tests:** `cargo test -p dsb-agent subagent` (support only; not sole R0A)

### PR unit 5 — `docs(product): VC011 READY evidence + independent review`

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
docs(product): VC011 subagent + worker cache Path A plan
test(scripts): hermetic Path A spawn_subagent fixture scenarios
test(scripts): VC011 Path A public-entry subagent + worker cache R0A
test(agent): Spec 60 worker cache + parent_after_worker honesty support
docs(product): VC011 READY evidence and independent review
```

---

## 4. Acceptance matrix

| ID | Check | Evidence |
|----|-------|----------|
| **A1** | Public Path A explore `spawn_subagent` dogfood | wire + final `explore-subagent-ok` |
| **A2** | Public Path A implement-class (`general-purpose`) spawn mutates disk | wire + disk golden + `implement-subagent-ok` |
| **A3** | Public launch `path_a_l3` stamp: `worker_epochs_match=true`, kinds, subagents enabled | META + stamp file |
| **A4** | Worker mutation → parent snippet invalidation **or** explicit residual + thin unit green | R0A disk/edit reject **or** residual section + unit |
| **A5** | Public entry (no `DEEPSEEK_BUILD_AGENT_BIN`) | META |
| **A6** | Owner-bar + path-linkage + heart regression green | command logs |
| **A7** | No SemVer bump in this PR | `Cargo.toml` / `package.json` still **5.3.0** |
| **A8** | Independent Grok review READY | review evidence file |

---

## 5. Gates (required before READY)

```bash
./scripts/check-path-a-linkage.sh
./scripts/test-owner-bar.sh
./scripts/test-heart-regression.sh
./scripts/test-path-a-vc011-r0a.sh
# support (not sole proof):
cargo test -p dsb-agent subagent
cargo test -p dsb-cli stamp_path_a_l3
```

Restore any generated TSV side-effects to HEAD if gates rewrite them. Clean vendor/agent `target/` build debris after agent builds if disk is tight (exact `target` dirs only).

---

## 6. READY evidence

**Status: PLAN** — fill after implementation is green.

### 6.1 Commands

| Command | Result |
|---------|--------|
| `./scripts/test-path-a-vc011-r0a.sh` | _pending_ |
| `cargo test -p dsb-agent subagent` | _pending_ (support) |
| `./scripts/check-path-a-linkage.sh` | _pending_ |
| `./scripts/test-owner-bar.sh` | _pending_ |
| `./scripts/test-heart-regression.sh` | _pending_ |
| SemVer on branch | **`5.3.0`** (must remain) |

### 6.2 Wire + stamp sample

_pending implementation_

### 6.3 What shipped

_pending_

### 6.4 Explicit residuals at READY

- Live L3.5 `spawn_subagent` without API key remains **SKIP**.
- Worktree dogfood → **VC012**.
- SemVer **5.4.0** → **VC013**.
- Product builder disables `enabled_background` when subagent types are emptied — R0A keeps subagent tools available (same constraint as VC010).

### 6.5 Independent review

_pending_ sibling file `VC011_INDEPENDENT_REVIEW_2026-08-08.md`.

---

## 7. Stack / PR open checklist

- [ ] Branch `vc011-subagent-worker-cache` based on `vc010-l3-parallel-bg`
- [x] First commit = this plan (before source edits)
- [ ] Atomic commits as §3
- [ ] `gh pr create --base vc010-l3-parallel-bg` with **English** body + labels (`test`, `area/orchestrator` or `area/tools`)
- [ ] Body includes **Depends on #142**
- [ ] `gh pr view --json title,labels,url` shows ≥1 kind label
- [ ] **Do not merge**; **do not bump SemVer** (still 5.3.0)

---

## 8. Orchestration provenance

| Field | Value |
|-------|--------|
| Run | `run_7900ce143c26` |
| Task | `task_5b59f74b11ac` |
| Dispatch | `ctx_e6e96f805580` |
| Worktree | `/Users/WooseongKim/Projects/deepseek-build/vc011-subagent-worker-cache` |
| Stack head at open | `9d6f1d0` (VC010 tip) |
