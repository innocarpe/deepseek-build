# VC009 — Path A cache-hit visibility (+ Reasonix packaging honesty)

| Field | Value |
|-------|--------|
| **Story** | **VC009** — user-visible **or** loggable cache-hit signal on product Path A; close V2-cache; package Reasonix residual honestly |
| **Plan** | `vision-complete-5x` |
| **Date** | 2026-08-08 |
| **Status** | **READY** — hermetic cache usage + Path A stamp + stream mapping green; unversioned; stacked on #140 |
| **SemVer** | **none** (this story does **not** bump product version) |
| **Depends on** | **VC008** `reasoning_effort` wire (open PR **#140** `vc008-reasoning-effort`) |
| **Board** | [`VISION_COMPLETE_5X_GOALS.md`](../VISION_COMPLETE_5X_GOALS.md) · DAG [`WAVE_5x_VISION_PR_DAG.md`](../WAVE_5x_VISION_PR_DAG.md) |
| **Normative** | [`docs/specs/10-cache-contract.md`](../../specs/10-cache-contract.md) §1.8 · ADR [`0005-deepseek-provider-contract.md`](../../adr/0005-deepseek-provider-contract.md) · VISION long-session cost · V2-cache |
| **Prior** | Status-line UI (#97 / 4.0.4); sampling-types `prompt_cache_hit_tokens` → `cached_prompt_tokens`; VC007/VC008 Reasonix stack |

**This file is the mandatory ultragoal PR unit plan for VC009 plus implementation evidence.**
It does **not** claim VISION freeze complete. It does **not** re-cut SemVer **`5.3.0`** (already on this stack from VC006). Thin Path B `dsb-provider-deepseek` usage goldens alone are **not** Path A visibility proof.

---

## 0. Floor and dependency facts

### 0.1 Live floor (story open; 2026-08-08)

| Probe | Live result |
|-------|-------------|
| This worktree branch | `vc009-cache-visibility` (forked at VC008 tip) |
| Stack base for feature commits / PR base | **`vc008-reasoning-effort`** (open PR **#140**); **not** `origin/main` until after #140 merges |
| `git show origin/main:Cargo.toml` version | **`5.2.2`** |
| `npm view @innocarpe/deepseek-build version` | **`5.2.2`** |
| `gh release list` Latest | **`v5.2.2`** |
| Working tree product version (stack tip) | **`5.3.0`** (from VC006 cut on stack) |
| Board text residual | VC009 / Reasonix cut still may say **`5.3.0`** — **stale** vs floor + VC006 already on **`5.3.0`** |
| Prior UI | DeepSeek bottom status row (`cache N%` + balance) already in vendor patches / #97 |

### 0.2 Floor interpretation (fail-close)

- **Live product floor is `origin/main` = `5.2.2`.** npm + GitHub Release aligned at open.
- Stack already carries product **`5.3.0`** from VC006. **Do not reuse or re-cut `5.3.0`.**
- Board’s “VC009 = 5.3.0 Reasonix cut” is **stale packaging text**. This story is **unversioned** visibility + packaging **honesty**. Residual dedicated packaging cut (if any) is the next free minor after floor re-check — historically noted as **`5.4.0`**, which **collides** with the L3 board minor; do not invent a cut here.
- **Open as a stacked PR** with base **`vc008-reasoning-effort`** and body **`Depends on #140`**. **Do not merge** this PR in-story.
- Fail closed if Path A cannot show a **user-visible format** or a **loggable/stampable** cache-hit signal under hermetic usage — record residual; do not invent green.

---

## 1. Why this PR (one sentence)

Close the Reasonix L2 **V2-cache** gap: product Path A must surface a **user-visible** (`cache N%` status chip path) or **loggable/stampable** cache-hit signal when DeepSeek usage reports `prompt_cache_hit_tokens`, with hermetic fixture + Path A stamp evidence — not library-only goldens.

---

## 2. Call-path map (inspected before design)

| Layer | Path | Role today (pre-VC009 residual) |
|-------|------|----------------------------------|
| Public product | `deepseek-build` / `dsb` → `deepseek-build-agent` (vendored Grok) | **Path A** product default |
| Wire usage (DeepSeek) | top-level `prompt_cache_hit_tokens` on chat/completions usage | Provider field (ADR 0005) |
| Mapping | `xai-grok-sampling-types` `Usage` → `TokenUsage.cached_prompt_tokens` | Unit-tested; stream via `chat_completions` `u.into()` |
| Ledger | chat-state `cached_read_tokens` from `TokenUsage` | Session totals for status |
| User-visible UI | pager bottom row `format_cache_hit_pct` + `x.ai/deepseek/status` | **Shipped** #97 / patches 0001–0005 |
| Loggable turn | `shell.turn.inference_done` / telemetry `cached_prompt_tokens` | Present but not Path A hermetic-stamped |
| Hermetic fixture | `scripts/lib/scripted_deepseek_server.py` usage | **Missing** `prompt_cache_hit_tokens` — never exercises cache field |
| Thin Path B | `dsb-provider-deepseek` `CacheEvidence` | Correct oracle — **not** Path A proof |

### Target VC009 contract (Path A)

1. Hermetic scripted DeepSeek responses include **`prompt_cache_hit_tokens`** (and miss) on usage.
2. Response usage is **recorded** on the wire transcript (response-side lines) so e2e can assert fixture honesty without live API.
3. Path A turn completion **stamps** `path_a_cache_signal.txt` under `DEEPSEEK_BUILD_HOME` when usage is present (loggable/doctor-style signal; fail-soft).
4. Stream unit proves DeepSeek `prompt_cache_hit_tokens` maps to `TokenUsage.cached_prompt_tokens` on chat_completions path.
5. Status chip formatter unit remains green (`cache N%`).
6. Gates remain green; TSV side-effects restored.
7. SemVer packaging: **none**; board residual for cut minor documented honestly.

### Explicit non-claims

| Non-claim | Residual |
|-----------|----------|
| Guaranteed provider cache hits on live DeepSeek | Server policy (Spec 10 non-goal) |
| Live dual-call substitute protocol on Path A | ADR 0005 Path B / when fields absent |
| SemVer / npm / GitHub Release cut this story | Unversioned; residual packaging cut after floor re-check |
| Full VISION L2 freeze | Requires residual honesty on board SemVer + any live dogfood residual |
| TUI screenshot dogfood this story | Formatter unit + stamp + mapping; live UI already shipped |

---

## 3. PR units (ordered atomic)

### PR unit 1 — `docs(product): VC009 cache visibility plan + floor`

- **Intent:** Lock stack base, call-path map, acceptance matrix, SemVer non-claims before code.
- **Touches:** `docs/product/evidence/VC009_CACHE_VISIBILITY_2026-08-08.md`
- **Depends on:** VC008 stack tip
- **SemVer:** none
- **Tests:** n/a (docs)

### PR unit 2 — `feat(scripts): hermetic DeepSeek cache usage on Path A fixture`

- **Intent:** Scripted server emits `prompt_cache_hit_tokens` / miss and records response usage on wire.
- **Touches:** `scripts/lib/scripted_deepseek_server.py`
- **Depends on:** unit 1
- **SemVer:** none
- **Tests:** exercised by Path A e2e + manual python import if needed

### PR unit 3 — `feat(shell): Path A cache signal stamp + stream mapping test`

- **Intent:** Stamp loggable cache signal under product home; unit-test chat_completions DeepSeek cache mapping.
- **Touches:** vendored shell helper/turn path; `xai-grok-sampler` chat_completions test
- **Depends on:** unit 2 (logical)
- **SemVer:** none
- **Tests:** `cargo test -p xai-grok-sampler …` / shell stamp unit; e2e after agent rebuild

### PR unit 4 — `test(scripts): Path A e2e asserts V2-cache signal`

- **Intent:** Public-entry e2e fails closed if response usage lacks cache fields or stamp is empty when agent rebuilt.
- **Touches:** `scripts/test-path-a-public-entry-e2e.sh`
- **Depends on:** units 2–3
- **SemVer:** none
- **Tests:** `./scripts/test-path-a-public-entry-e2e.sh`

### PR unit 5 — `docs(product): VC009 READY evidence + adversarial close-out`

- **Intent:** Record commands, stamp/wire sample, review verdict, residuals.
- **Touches:** this evidence file status + optional `PATH_A_R0_VC009_*` artifacts
- **Depends on:** units 2–4 green + independent Grok review
- **SemVer:** none

## Sequential

1. unit 1 → unit 2 → unit 3 → unit 4 → unit 5

## Parallel

- none (single stacked PR for this supervised story)

---

## 4. Acceptance matrix

| ID | Criterion | Evidence |
|----|-----------|----------|
| V2-cache-1 | Hermetic responses include DeepSeek cache usage fields | Wire response_usage lines |
| V2-cache-2 | Mapping DeepSeek field → `cached_prompt_tokens` on stream path | Unit test |
| V2-cache-3 | Loggable/stampable Path A signal under product home | `path_a_cache_signal.txt` |
| V2-cache-4 | User-visible format path still green | `format_cache_hit_pct` unit |
| V2-cache-5 | Public entry e2e green with cache asserts | `test-path-a-public-entry-e2e.sh` |
| Package-1 | No illegal SemVer re-cut of 5.3.0 | Cargo.toml stays 5.3.0; story unversioned |
| Gates | heart + owner-bar + path-a linkage | scripts |

---

## 5. Implementation log

| Unit | Commit | Notes |
|------|--------|-------|
| 1 plan | `74e14ff` | Floor + acceptance matrix |
| 2 fixture | `f3e0aa6` | `prompt_cache_hit_tokens` + `response_usage` wire |
| 3 stamp + stream | `c5809f5` | `path_a_cache_signal` + chat_completions mapping test |
| 4 e2e | `e4eb4b7` | Public-entry assert cache usage + stamp soft/hard |
| 5 READY | this section | Evidence + review |

---

## 6. READY evidence

### 6.1 Commands

| Command | Result |
|---------|--------|
| `cargo test -p xai-grok-sampler deepseek_prompt_cache_hit` (vendor) | **PASS** |
| `cargo test -p xai-grok-shell --lib path_a_cache_signal` (vendor) | **PASS** (2) |
| `cargo test -p xai-grok-pager --lib cache_hit_pct` (vendor) | **PASS** (2) |
| `cargo test -p dsb-provider-deepseek parses_prompt_cache` | **PASS** |
| `./scripts/build-grok-pager.sh release` | **PASS** (agent with stamp) |
| `cargo build -p dsb-cli --release` | **PASS** (CLI 5.3.0) |
| `./scripts/test-path-a-public-entry-e2e.sh` | **PASS** (`cache_usage_ok=2`, `cache_stamp_ok cached_prompt_tokens=80`) |
| `./scripts/check-path-a-linkage.sh` | **PASS** |
| `./scripts/test-owner-bar.sh` | **PASS** (60/60; TSV restored) |
| `./scripts/test-heart-regression.sh` | **PASS** (TSV restored; PATH_A_E2E SKIP default without `--with-e2e`) |

### 6.2 Wire + stamp sample

Slim capture: [`PATH_A_R0_VC009_CACHE_USAGE_last.jsonl`](./PATH_A_R0_VC009_CACHE_USAGE_last.jsonl)

| n | model | reasoning_effort | response `prompt_cache_hit_tokens` |
|---|-------|------------------|-------------------------------------|
| 1 | `deepseek-v4-flash` | **high** | **80** / prompt 100 |
| 2 | `deepseek-v4-flash` | **high** | **80** / prompt 100 |

Path A stamp ([`PATH_A_CACHE_SIGNAL_last.txt`](./PATH_A_CACHE_SIGNAL_last.txt)):

```text
path_a_cache_signal=present
prompt_tokens=100
cached_prompt_tokens=80
cache_hit_pct=80
cache_chip=cache 80%
source=path_a_turn_usage
```

### 6.3 What shipped

1. Hermetic DeepSeek fixture emits **`prompt_cache_hit_tokens` / miss** and records **`response_usage`** wire lines.
2. Path A turn completion **stamps** `path_a_cache_signal.txt` under `DEEPSEEK_BUILD_HOME` (loggable signal + chip string).
3. Chat completions stream unit maps DeepSeek cache field → `TokenUsage.cached_prompt_tokens`.
4. Public-entry e2e **hard-fails** without `response_usage` cache hits **and** without Path A stamp `cached_prompt_tokens>0` + `cache_chip`.
5. User-visible status chip path remains the shipped pager `format_cache_hit_pct` (unit re-green; no re-implementation).
6. **No SemVer bump** — packaging honesty: do not re-cut **`5.3.0`**.

### 6.4 Independent review (Grok-only)

| Field | Value |
|-------|--------|
| **Reviewer 1** | Independent Grok critic (`oh-my-claudecode:critic`, subagent `019fddd4-4c70-7491-a91b-601adbc62099`) |
| **Initial verdict** | **NOT_READY** — READY claimed with gate placeholders; stamp gate soft/fixture-only hard path hollow |
| **P0 closed** | owner-bar **60/60 PASS** + heart regression **PASS** recorded in §6.1 |
| **P1 closed** | e2e hard-fails if stamp missing or `cached_prompt_tokens<1` or chip missing (Path A consumption, not fixture-only) |
| **Close-out (after harden)** | **READY** after P0/P1 fixes + re-green e2e (`cache_stamp_ok cached_prompt_tokens=80`) |
| **Reviewer 2 (final head)** | Fresh independent Grok review of PR **#141** head `88faf07` — full write-up [`VC009_PR141_INDEPENDENT_REVIEW_2026-08-08.md`](./VC009_PR141_INDEPENDENT_REVIEW_2026-08-08.md) |
| **Final-head verdict** | **READY** — P0/P1 none; P2 residuals only; **do not merge** until stack base #140 lands |
| **P2 residual** | stamp chip arithmetic is local (not calling pager `format_cache_hit_pct`); board SemVer text still stale |

Implementer self-notes are **not** independent review. Reviewer 2 re-verified hardening, units, e2e, labels, and residual honesty on the final head.

### 6.5 Residuals (honest)

| Residual | Notes |
|----------|--------|
| Live DeepSeek cache hit rates | Server policy; hermetic fixture is not live proof |
| TUI screenshot dogfood | Chip format unit + stamp; live UI shipped in #97 |
| SemVer / npm / Release cut | Unversioned; board “5.3.0 Reasonix cut” is **stale** vs stack **5.3.0** from VC006 |
| Dual-call substitute on Path A | ADR 0005 Path B / when fields absent — not this story |
| Stamp chip vs pager formatter drift | P2; stamp uses integer round-half-up; pager uses f64 format — residual |
| Agent binary without rebuild | Stamp missing → e2e **FAIL** (hard); rebuild agent with VC009 stamp code |
