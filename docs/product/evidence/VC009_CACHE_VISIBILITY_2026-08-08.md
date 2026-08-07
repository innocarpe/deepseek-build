# VC009 — Path A cache-hit visibility (+ Reasonix packaging honesty)

| Field | Value |
|-------|--------|
| **Story** | **VC009** — user-visible **or** loggable cache-hit signal on product Path A; close V2-cache; package Reasonix residual honestly |
| **Plan** | `vision-complete-5x` |
| **Date** | 2026-08-08 |
| **Status** | **PLAN** — unit plan locked; implementation in progress |
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

_(filled as units land)_

---

## 6. READY evidence

_(filled at close)_
