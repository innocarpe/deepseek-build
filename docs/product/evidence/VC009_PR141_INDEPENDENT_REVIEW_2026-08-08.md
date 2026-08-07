# VC009 PR #141 — independent Grok review

| Field | Value |
|-------|--------|
| **PR** | [#141](https://github.com/innocarpe/deepseek-build/pull/141) `feat(shell): Path A cache-hit visibility for V2-cache` |
| **Reviewed code head** | `88faf079284381b6bfef9fc6f6458f6d816f0f9d` — last commit with product/test behavior under review (stamp gate harden in `scripts/test-path-a-public-entry-e2e.sh` + READY evidence package) |
| **Docs-only close-out heads** | `ad8a9e677d932de2ae131e8d08c3d669e4985f18` and any later **docs-only** commits that only land this report / evidence pointers / meta refresh — **not** a second code review |
| **Base** | `vc008-reasoning-effort` (open **#140**) |
| **Reviewer** | Fresh independent Grok review (dispatch task_198777849edd) |
| **Date** | 2026-08-08 |
| **Verdict (on code head `88faf07`)** | **READY** |
| **Merge** | **Do not merge** this PR in-story |

### Head honesty (fail-close)

| Commit | Kind | What independent review covers |
|--------|------|--------------------------------|
| **`88faf07`** | **Reviewed code head** | Hardening e2e stamp gate, feature stack through this tip, live re-verify (e2e/units/linkage) recorded below |
| **`ad8a9e6`** | **Docs-only close-out** | Adds this report file, points VC009 evidence at it, refreshes Path A meta/slim usage timestamps — **no** product/runtime code change |
| Later docs-only | **Docs-only** | Corrections such as this head-distinction — **do not** re-label as “reviewed final branch tip” without a new code review |

Do **not** claim the independent review re-audited every later tip that only adds markdown. Verdict attaches to **`88faf07`**. Docs-only tips may advance the PR branch without invalidating that verdict.

---

## 1. Scope checked (on `88faf07`)

| Area | Result |
|------|--------|
| Hardening (stamp + fixture e2e) | **OK** — hard FAIL if stamp missing, `cached_prompt_tokens<1`, or chip missing; hard FAIL if no `response_usage` with `prompt_cache_hit_tokens>0` |
| Path A call site | **OK** — `turn.rs` stamps `response.usage` before ledger record |
| Stream mapping unit | **OK** — re-ran `deepseek_prompt_cache_hit_tokens_map_to_cached_prompt_tokens` **PASS** |
| Stamp unit | **OK** — re-ran `path_a_cache_signal` (2) **PASS** |
| Public-entry e2e | **OK** — re-ran on **code head `88faf07`** **PASS** (`cache_usage_ok=2`, `cache_stamp_ok cached_prompt_tokens=80`) |
| Linkage | **OK** — `check-path-a-linkage` **PASS** |
| Evidence honesty | **OK** — gates recorded; residuals explicit; SemVer none |
| Labels | **OK** — `feat` + `area/cache` (count=2); title kind matches |
| Stack / merge policy | **OK** — body has **Depends on #140**, **Do not merge**, base not `main` |
| SemVer | **OK** — workspace stays **`5.3.0`**; no `Cargo.toml` / package bump in PR feature diff |

---

## 2. Prior NOT_READY items (from `019fddd4…`)

| Prior finding | Status on code head `88faf07` |
|---------------|------------------------------|
| P0: gates placeholders under READY | **Closed** — §6.1 records owner-bar 60/60 + heart PASS |
| P1: fixture-only hard assert | **Closed** — stamp hard-fail is Path A consumption gate |
| P1: stamp allows zero hits | **Closed** — requires `cached_prompt_tokens>0` |
| P1: soft stamp | **Closed** — missing stamp → `FAIL=1` |

---

## 3. Live re-verify (this review, code head `88faf07`)

```text
git_sha=88faf079284381b6bfef9fc6f6458f6d816f0f9d
test-path-a-public-entry-e2e: PASS
  cache_usage_ok=2 cache_samples=['hit=80/prompt=100', 'hit=80/prompt=100']
  cache_stamp_ok cached_prompt_tokens=80
check-path-a-linkage: PASS
cargo test -p xai-grok-shell --lib path_a_cache_signal → 2 ok
cargo test -p xai-grok-sampler --lib deepseek_prompt_cache → 1 ok
```

Artifacts consistent with claim: `PATH_A_CACHE_SIGNAL_last.txt` (`cached_prompt_tokens=80`, `cache_chip=cache 80%`); slim wire `PATH_A_R0_VC009_CACHE_USAGE_last.jsonl`.

---

## 4. Findings

### P0
**None.**

### P1
**None.**

### P2 (residual honesty — do not block READY)

1. **Stamp chip math ≠ pager `format_cache_hit_pct`** — local integer round-half-up vs pager f64 path; residual already in evidence §6.5.
2. **Board / DAG still map VC009 → `5.3.0` cut** — stale vs stack **5.3.0** from VC006; PR body + evidence residualize correctly; board files not updated (acceptable residual).
3. **Historical commit message** `e4eb4b7` still says “soft-check”; code head `88faf07` hardens — do not read intermediate commit as current gate policy.
4. **Evidence §5 log** still says unit 4 “soft/hard”; §6.3 correctly describes hard-fail — docs nit only.
5. **Stamp file I/O is fail-soft** (write errors never block turn) while e2e hard-fails if missing — correct product posture; residual for CI machines without rebuilt agent (fails closed until rebuild).
6. **Branch tip may be docs-only after `88faf07`** (`ad8a9e6`+); those commits do not re-open a code review requirement unless product/scripts change again.

---

## 5. Residual honesty checklist (must stay)

| Residual | Still honest? |
|----------|----------------|
| Hermetic ≠ live DeepSeek hit rates | Yes |
| TUI screenshot not re-dogfooded | Yes (UI #97; loggable stamp is hermetic close) |
| No SemVer / npm / Release this PR | Yes |
| Dual-call substitute out of scope | Yes |
| Stacked on #140; do not merge | Yes |
| Reviewed code head ≠ later docs-only tip | Yes — see table above |

---

## 6. Verdict

**READY** on **reviewed code head `88faf07`** for stack review and later merge **after** base **#140** (and stack) land. **Do not merge #141 now.**

Docs-only close-out **`ad8a9e6`** (and similar follow-ups) may sit on top of that head to land this report; they are **not** claimed as a re-review of product code.

V2-cache substance: loggable Path A cache-hit signal is proven on public-entry hermetic Path A with durable dual hard gates (fixture `response_usage` + agent stamp `cached_prompt_tokens>0`). User-visible chip path remains pre-shipped status line (honest residual, not over-claim).
