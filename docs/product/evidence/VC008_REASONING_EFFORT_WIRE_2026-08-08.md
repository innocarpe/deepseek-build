# VC008 — `reasoning_effort` on DeepSeek Path A wire

| Field | Value |
|-------|--------|
| **Story** | **VC008** — product Path A chat/completions bodies carry `reasoning_effort` when set (default coding **high**) |
| **Plan** | `vision-complete-5x` |
| **Date** | 2026-08-08 |
| **Status** | **READY** — seed/repair + CLI forward + hermetic wire assert green; unversioned; stacked on #139 |
| **SemVer** | **none** (this story does **not** bump product version) |
| **Depends on** | **VC007** Spec 10 Path A assembly stack (open PR **#139** `vc007-context-assembly`) |
| **Board** | [`VISION_COMPLETE_5X_GOALS.md`](../VISION_COMPLETE_5X_GOALS.md) · DAG [`WAVE_5x_VISION_PR_DAG.md`](../WAVE_5x_VISION_PR_DAG.md) |
| **Normative** | [`docs/specs/30-thinking-effort.md`](../../specs/30-thinking-effort.md) · ADR [`0005-deepseek-provider-contract.md`](../../adr/0005-deepseek-provider-contract.md) · Spec 20 routing defaults |
| **Prior evidence** | G009 routing/effort stamp + residual honesty ([`G009_ROUTING_EFFORT_2026-08-07.md`](./G009_ROUTING_EFFORT_2026-08-07.md)); product `dsb-provider-deepseek` request builder goldens |

**This file is the mandatory ultragoal PR unit plan for VC008 plus implementation evidence.**
It does **not** claim VISION L2 Reasonix complete (that still needs VC009 cache-hit visibility / cut). Thin Path B provider-builder goldens alone are **not** Path A wire proof.

---

## 0. Floor and dependency facts

### 0.1 Live floor (story open; 2026-08-08)

| Probe | Live result |
|-------|-------------|
| This worktree branch | `vc008-reasoning-effort` (forked at VC007 tip) |
| Stack base for feature commits / PR base | **`vc007-context-assembly`** (open PR **#139**); **not** `origin/main` until after #139 merges |
| `git show origin/main:Cargo.toml` version | **`5.2.2`** |
| Working tree product version (stack tip) | **`5.3.0`** (from VC006 cut on stack) |
| Board text residual | Reasonix cut (VC009) may still say **`5.3.0`** — **stale** vs floor + VC006 already on **`5.3.0`**; residual cut minor **`5.4.0`** |
| G009 residual | Path A chat_completions body often omitted / null `reasoning_effort` even when router/CLI stamp effort |

### 0.2 Floor interpretation (fail-close)

- **Live product floor is `origin/main` = `5.2.2`.**
- Stack already carries product **`5.3.0`** from VC006. **Do not reuse or re-cut `5.3.0`.**
- VC008 is **unversioned** — no SemVer bump, no npm, no GitHub Release packaging.
- **Open as a stacked PR** with base **`vc007-context-assembly`** and body **`Depends on #139`**. **Do not merge** this PR in-story.
- Fail closed if public Path A runtime cannot put non-null `reasoning_effort` on DeepSeek chat/completions wire — record residual; do not invent green.

---

## 1. Why this PR (one sentence)

Close the Reasonix L2 gap where **Path A** DeepSeek chat/completions requests still ship **without** top-level `reasoning_effort`, even though Spec 30 / ADR 0005 and G009 router+CLI defaults already intend coding effort **high**.

---

## 2. Call-path map (inspected before design)

| Layer | Path | Role today (pre-VC008) |
|-------|------|------------------------|
| Public product | `deepseek-build` / `dsb` → `deepseek-build-agent` (vendored Grok) | **Path A** product default |
| Product model seed | `dsb-cli` `ensure_product_agent_config*` | Seeds flash/pro with `api_backend` + `base_url` — **no effort fields** |
| Product CLI flag | `dsb` / `deepseek-build --effort low\|high\|max` | Applied on thin Path B loop only; **`tui_forward_flags` does not forward** to Grok agent |
| Grok model → sampler | `sampling_config_for_model` → `SamplerConfig.reasoning_effort = info.reasoning_effort` | Copies model catalog default when set |
| Grok request builder | `xai-chat-state` request_builder | `ConversationRequest.reasoning_effort` from sampling config |
| Grok chat_completions | `ChatCompletionRequest::from(ConversationRequest)` | Top-level `reasoning_effort` (omit when unset) |
| Thin Path B | `dsb-agent` + `dsb-provider-deepseek` `ChatRequestBuilder` | Already serializes effort + thinking — **not** public Path A |

### Target VC008 wire contract (Path A)

1. Default product DeepSeek models (flash + pro) carry **`supports_reasoning_effort = true`** and **`reasoning_effort = "high"`** (Spec 30 coding default).
2. Existing product homes **repair** injects those keys when missing (idempotent; does not clobber explicit user effort).
3. Product CLI `--effort <low|high|max>` is **forwarded** to the Grok agent as `--reasoning-effort <value>`.
4. Hermetic Path A public-entry e2e asserts at least one DeepSeek chat/completions body has **non-null** string `reasoning_effort` (default **high** unless override).
5. Gates remain green; TSV side-effects restored.

### Explicit non-claims

| Non-claim | Residual |
|-----------|----------|
| Full Spec 30 `thinking: { type: enabled\|disabled }` body field on Grok chat_completions | Grok `ChatCompletionRequest` has no separate `thinking` field; DeepSeek thinking is driven by product Path B provider or future wire extension |
| VC009 cache-hit visibility / Reasonix packaging cut | Separate story |
| SemVer / npm / Release for this unit | Unversioned |
| Owner-bar L1-30 already “full green without residual” pre-wire | G009 admitted PARTIAL wire |

---

## 3. PR units (ordered atomic)

### PR unit 1 — `docs(product): VC008 reasoning_effort wire plan + floor`

- **Intent:** Lock stack base, call-path map, acceptance matrix, non-claims before code.
- **Touches:** `docs/product/evidence/VC008_REASONING_EFFORT_WIRE_2026-08-08.md`
- **Depends on:** VC007 stack tip
- **SemVer:** none
- **Tests:** n/a (docs)

### PR unit 2 — `feat(cli): seed + repair Path A reasoning_effort defaults`

- **Intent:** Product config seed/repair puts Spec 30 coding effort on flash/pro so sampler stamps wire.
- **Touches:** `crates/dsb-cli/src/agent_launch.rs` (+ unit tests)
- **Depends on:** unit 1
- **SemVer:** none
- **Tests:** `cargo test -p dsb-cli product_config` / agent_launch config tests

### PR unit 3 — `feat(cli): forward --effort to Grok agent`

- **Intent:** Product CLI effort flag reaches Path A agent as `--reasoning-effort`.
- **Touches:** `crates/dsb-cli/src/main.rs` (`tui_forward_flags` + tests)
- **Depends on:** unit 2 (logical; same PR OK)
- **SemVer:** none
- **Tests:** `cargo test -p dsb-cli tui_forward_flags`

### PR unit 4 — `test(scripts): Path A wire assert for reasoning_effort`

- **Intent:** Hermetic public-entry e2e seeds effort fields and fails closed if DeepSeek wire lacks non-null effort.
- **Touches:** `scripts/test-path-a-public-entry-e2e.sh` (and optional slim evidence capture)
- **Depends on:** units 2–3
- **SemVer:** none
- **Tests:** `./scripts/test-path-a-public-entry-e2e.sh` (needs rebuilt agent when config-only is insufficient — config seed is primary)

### PR unit 5 — `docs(product): VC008 READY evidence + adversarial close-out`

- **Intent:** Record commands, wire sample, review verdict, residuals.
- **Touches:** this evidence file status + optional `PATH_A_R0_VC008_*` artifacts
- **Depends on:** units 2–4 green + independent Grok review
- **SemVer:** none

## Sequential vs parallel

```text
Sequential: unit1 → unit2 → unit3 → unit4 → unit5
Parallel: none (single stacked PR; same review lens)
```

## Atomic commits (planned)

1. `docs(product): VC008 reasoning_effort wire plan and floor`
2. `feat(cli): seed and repair Path A reasoning_effort defaults`
3. `feat(cli): forward --effort to Grok agent as --reasoning-effort`
4. `test(scripts): assert reasoning_effort on Path A public-entry wire`
5. `docs(product): VC008 READY evidence and adversarial close-out`

---

## 4. Acceptance matrix

| ID | Check | Evidence |
|----|-------|----------|
| **V2-30-1** | Default flash/pro product seed includes `reasoning_effort = "high"` + `supports_reasoning_effort = true` | Unit + seed file content |
| **V2-30-2** | Repair injects missing effort keys without clobbering explicit user values | Unit |
| **V2-30-3** | `--effort high` (etc.) forwarded on TUI/agent path | Unit on `tui_forward_flags` |
| **V2-30-4** | Path A hermetic wire: ≥1 DeepSeek chat/completions body has non-null `reasoning_effort` string | Public-entry e2e |
| **V2-30-5** | Owner-bar / heart / path-a-linkage stay green | Scripts |

---

## 5. Implementation notes (locked)

1. **Root cause:** product `[model.deepseek-v4-*]` stanzas never set `reasoning_effort`; Grok copies `info.reasoning_effort` into sampler → request. Unset → field omitted on wire (G009 residual).
2. **Do not** vendor-patch Grok chat_completions serializer for this story — product config + CLI forward are sufficient.
3. **chat_completions auto-support:** Grok only auto-enables `supports_reasoning_effort` for Messages backend; product must set the flag **explicitly** for DeepSeek chat_completions (needed for CLI override / menus; default effort field still needs seed).
4. **thinking body field** remains residual on Path A Grok wire (no field on `ChatCompletionRequest`).

---

## 6. Evidence log (implementation)

### 6.1 Commands

```bash
cargo test -p dsb-cli -- product_config_seed
cargo test -p dsb-cli -- repair_injects_reasoning
cargo test -p dsb-cli -- tui_forward_flags
./scripts/check-path-a-linkage.sh
./scripts/test-path-a-public-entry-e2e.sh
./scripts/test-owner-bar.sh
./scripts/test-heart-regression.sh
```

### 6.2 Results

| Check | Result |
|-------|--------|
| `product_config_seed` | **PASS** |
| `repair_injects_reasoning_effort_defaults_without_clobber` | **PASS** |
| `tui_forward_flags` (incl. effort → `--reasoning-effort`) | **PASS** (7 tests) |
| `check-path-a-linkage.sh` | **PASS** |
| `test-path-a-public-entry-e2e.sh` | **PASS** — `wire_models flash=2 pro=0 effort_ok=2 samples=['deepseek-v4-flash:high', …]` |
| `test-owner-bar.sh` | **PASS** (60/60; TSV restored) |
| `test-heart-regression.sh` | **PASS** (TSV restored) |

### 6.3 Wire sample

Slim capture: [`PATH_A_R0_VC008_EFFORT_WIRE_last.jsonl`](./PATH_A_R0_VC008_EFFORT_WIRE_last.jsonl)

| n | model | reasoning_effort |
|---|-------|------------------|
| 0 | `grok-4.5` (session-title side-call) | null / omitted |
| 1 | `deepseek-v4-flash` | **`high`** |
| 2 | `deepseek-v4-flash` | **`high`** |

Full wire: [`PATH_A_R0_WIRE_last.jsonl`](./PATH_A_R0_WIRE_last.jsonl) · meta: [`PATH_A_R0_VC008_META_last.txt`](./PATH_A_R0_VC008_META_last.txt)

### 6.4 What shipped

1. Product config **seed** for flash/pro: `supports_reasoning_effort = true`, `reasoning_effort = "high"`.
2. Product config **repair** injects those keys when missing; does not clobber explicit user effort.
3. Product CLI `--effort` forwarded on TUI/agent path as `--reasoning-effort`.
4. Hermetic public-entry e2e asserts ≥1 DeepSeek body with non-null `reasoning_effort` string.

### 6.5 Independent review

_(filled after Grok-only adversarial review — required before merge readiness)_

### 6.6 Residuals (honest)

| Residual | Notes |
|----------|--------|
| Path A Grok `thinking: { type }` body field | Not on Grok `ChatCompletionRequest`; Spec 30 Path B provider still sends both |
| Session-title side model `grok-4.5` | May omit effort — not DeepSeek product wire |
| VC009 cache-hit visibility / Reasonix cut | Separate story; SemVer residual cut **`5.4.0`** under current floor |
| VC007 turn-prefix stamp soft warn on this agent binary | Unrelated; agent binary on host may lag VC007 wire rewrite until stack rebuild |
