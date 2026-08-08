# VC015 — independent adversarial review **Lane A** (head `01215c2`)

| Field | Value |
|-------|--------|
| **Lane** | **A** — fresh independent adversarial review |
| **Reviewer runtime** | Grok Build / **grok-4.5** high (parent = child; no Claude/Codex; no subagents) |
| **Date** | 2026-08-08 |
| **Target SHA (exact, frozen)** | `01215c25c6f9fc0fd33419be01f23a8a5c22b814` |
| **Target short** | `01215c2` — `test(scripts): stage-aware VC015 R0A skip-build resolution` |
| **Diff under review** | `8a6e951a5a6524f356353f76bdbe6800bfa36910..01215c25c6f9fc0fd33419be01f23a8a5c22b814` |
| **Worktree** | `/Users/WooseongKim/Projects/deepseek-build/vc015-review-head012-v1` · branch `vc015-review-head012-v1` |
| **Product on disk** | Cargo / package.json **`5.5.0`** (on-branch freeze packaging) |
| **Scope** | V1 Deep Code Spec **45** / `snippet_id`; V2 Reasonix Spec **10** / **30** / cache; integrity + SemVer/docs impact of **stage-aware VC015 harness + refreshed R0A evidence** |
| **Forbidden this lane** | fetch/pull/merge; moving remote tip; product code / version / PR / tag / npm / GitHub mutation; full builds; Claude/Codex/subagents |

**This file is read-only review evidence.** It does **not** self-count as dual review B. It does **not** merge, tag, or publish.

---

## 0. Mandate and fail-close rules applied

1. Review **only** exact target `01215c2` at worktree HEAD (no remote chase).  
2. Challenge freeze honesty: Path A sole proof vs thin Path B; staged binary identity vs packaging SemVer; residual over-claim.  
3. Spec anchors: Spec 45 (`snippet_id` mint/require/stale), Spec 10 (stable prefix / epoch / no snippets in prefix), Spec 30 (`reasoning_effort` on DeepSeek wire), Spec 60 T3 spirit (worker mutate → parent snippet safety) as closed by VC015 bar.  
4. Disk ~5 GiB free — **no** full cargo builds; focused read-only gates only.  
5. Write **only** this report; commit **only** this report on the review branch.

---

## 1. Target identity (verified)

| Probe | Result |
|-------|--------|
| `git rev-parse HEAD` | `01215c25c6f9fc0fd33419be01f23a8a5c22b814` |
| `git log -1 --oneline` | `01215c2 test(scripts): stage-aware VC015 R0A skip-build resolution` |
| Ancestry | `8a6e951` **is** ancestor of `01215c2` |
| Range log | single commit: harness + META/WIRE refresh only |
| Dirty product tree at review open | clean (only later this report file) |

### 1.1 Diff inventory (`8a6e951..01215c2`)

| Path | Change |
|------|--------|
| `scripts/test-path-a-vc015-r0a.sh` | Stage-aware CLI/agent resolution under `--skip-build`; prefer `PATH_A_R0A_*`; **never** fall back to `~/.deepseek-build` when `SKIP_BUILD=1`; META records versions/sha/env |
| `docs/product/evidence/PATH_A_R0_VC015_parent-worker-snippet-stale_META_last.txt` | Re-prove meta with staged identities |
| `docs/product/evidence/PATH_A_R0_VC015_parent-worker-snippet-stale_WIRE_last.jsonl` | Re-prove wire |

**No** crate / vendor product code / SemVer bump in this commit. Packaging **`5.5.0`** already landed earlier (`8eea500`).

---

## 2. Commands and evidence (this lane)

### 2.1 Read (SSOT / board / DAG / specs / freeze)

| Artifact | Used for |
|----------|----------|
| `docs/product/SSOT.md` | Priority: main+version → owner bar → PRD → harness → boards |
| `docs/product/VISION_COMPLETE_5X_GOALS.md` | V1–V4 acceptance IDs; freeze prefer **5.5.0** |
| `docs/product/WAVE_5x_VISION_PR_DAG.md` | VC015 freeze unit pointer |
| `docs/product/KNOWN_LIMITS.md` | Residual honesty + stack vs live floor |
| `docs/product/evidence/VC015_VISION_FREEZE_5_5_0_2026-08-08.md` | Freeze plan + READY claim + V3-60-3 mechanism honesty |
| `docs/specs/45-snippet-edit.md` | Mint / version / `snippet_stale` |
| `docs/specs/10-cache-contract.md` | Stable prefix; snippets **not** in prefix |
| `docs/specs/30-thinking-effort.md` | `reasoning_effort` on DeepSeek wire |
| `docs/specs/60-subagents.md` | T3 parent snippet expiry spirit vs VC015 bar |
| Prior stack evidence | VC006–VC009 presence (V1/V2 pillars); VC013 agent sha pin |

### 2.2 Diff + static harness

```text
git log --oneline 8a6e951..01215c2
git diff --stat / --name-status 8a6e951..01215c2
git show 01215c2   # commit message + paths
# full script diff + current scripts/test-path-a-vc015-r0a.sh assert body
```

### 2.3 Focused read-only gates (no full build)

| Command | Result |
|---------|--------|
| `./scripts/check-semver.sh` | **PASS** — cargo ≡ npm **`5.5.0`** |
| `./scripts/check-path-a-linkage.sh` | **PASS** (F1 note: grok-build has no `dsb-*` dep — expected) |
| `./scripts/reorder-changelog.sh --check` | **PASS** |
| Offline Python re-assert of committed VC015 WIRE (same checks as harness) | **PASS** (mint + spawn implement-class + stale edit emit + fail-closed) |
| `df -h .` | ~**5.2 GiB** free — no full build attempted |
| `gh pr view 147` (read-only) | Open stacked PR; body mechanism honesty matches evidence (no edit) |

**Not re-run this lane (disk / mandate):** full `cargo build`, live DeepSeek R0A, owner-bar / heart full suite, npm reinstall smoke. Prior freeze evidence claims those green at packaging tip; this lane re-checked linkage/SemVer/changelog + offline wire truth for the **new** artifacts.

### 2.4 META identities at tip (committed)

```text
git_sha=8a6e951a5a6524f356353f76bdbe6800bfa36910
skip_build=1
cli=…/dsb-vc015-stage…/bin/deepseek-build
cli_version=deepseek-build 5.4.0
agent_resolved=…/xai-grok-pager
agent_sha256=a56897aa2cdab00eb2b47a53796007d55bc4aa73c983c9fc59ffe9f7de370e54
agent_version=deepseek-build 0.2.120 (c3857a9)
PATH_A_R0A_* set to staged bin dir
DEEPSEEK_BUILD_AGENT_BIN_unset=yes
scenario=parent-worker-snippet-stale
agent_exit=0
parent_seed='worker-mutated-parent\n'
final token includes parent-worker-snippet-stale-ok
```

Agent sha **matches VC013 cut pin** (`a56897aa…` / `c3857a9`).

### 2.5 Offline WIRE chain (committed)

| Step | Wire fact |
|------|-----------|
| Parent mint | `read_file` `parent_seed.txt` → `snippet_id: snp_01KZF6YD9HNNZ3M9C0X931X7XE` + `file_version` sha256 |
| Implement-class worker | `spawn_subagent` `general-purpose`; child `run_terminal_command` `printf 'worker-mutated-parent\n' > parent_seed.txt` |
| Parent stale edit | `search_replace` with **same** pre-mutation `snippet_id` + `should-not-apply-after-worker` |
| Fail-closed | tool result: **`snippet_stale: snippet version does not match current file content; re-read before edit`** |
| Disk honesty | META `parent_seed` = worker mutation; no `should-not-apply-after-worker` |
| Final token | `parent-worker-snippet-stale-ok` |
| Spec 30 sample | DeepSeek turns carry `reasoning_effort: "high"` (majority of parent/worker agent turns) |
| Spec 10 sample | DeepSeek requests include tool schemas; **`snippet_id` not embedded in system prefix** |
| L3 stamp | present; worker epochs match (`l3_worker_epochs_match=true`) |

---

## 3. Focus analysis

### 3.1 V1 — Deep Code Spec 45 / `snippet_id` (+ V3-60-3 bar)

| Claim | Evidence | Adversarial call |
|-------|----------|------------------|
| Session `snippet_id` mint on Path A read | Wire tool content `snippet_id: snp_…` after parent `read_file` | **Holds** |
| Edit requires / uses `snippet_id` | Parent `search_replace` args include `snippet_id` | **Holds** |
| Stale content → no apply | `snippet_stale` + disk unchanged for false edit | **Holds** |
| Public Path A entry | `deepseek-build agent -p`; `DEEPSEEK_BUILD_AGENT_BIN` unset; hermetic home agent copy | **Holds** (META) |
| Mechanism honesty vs Spec 60 T3 “expire table” | Close is **version mismatch reject**, not proven parent `expire_all` | **Honest residual** (KNOWN_LIMITS + VC015 §8.3) — **not** a false expire_all claim |

**Verdict on product mechanism:** V3-60-3 freeze bar (parent mint → implement worker mutates same path → parent pre-mutation edit rejected) is **supported by Path A wire**, not by thin `dsb-agent` unit alone.

### 3.2 V2 — Reasonix Spec 10 / 30 / cache

| ID | At target | Notes |
|----|-----------|-------|
| **Spec 10** assembly spirit | **OK for this R0A** | Stable system + tools present; snippet table not in system prefix; L3 worker epoch equality stamp present |
| **Spec 10** full golden suite | **Not re-executed** this lane | Prior **VC007** evidence remains on tree; `01215c2` does not touch assembly code |
| **Spec 30** `reasoning_effort` | **OK on this wire** | `high` on DeepSeek agent turns in VC015 WIRE |
| **Spec 30** full thinking body object | **Not claimed** | KNOWN_LIMITS L2 scope note remains correct |
| **Cache visibility** | **Not re-proved by VC015 scenario** | VC009 usage fixture still present (`prompt_cache_hit_tokens`); L3 epoch match is worker-cache stamp, not user cache chip |
| **01215c2 impact** | **None negative** | Harness/evidence only; no Spec 10/30/cache product regression surface in the diff |

### 3.3 Stage-aware harness / evidence integrity (`01215c2`)

**What improved (integrity positive):**

1. Under `--skip-build`, **no** silent resolution to `~/.deepseek-build` (older **5.2.x / 5.3.0** risk → false green / false red).  
2. Explicit staged pair env: `PATH_A_R0A_CLI` / `PATH_A_R0A_AGENT` / `PATH_A_R0A_BIN_DIR`.  
3. META now records `cli_version`, `agent_sha256`, `agent_version`, staged env, `skip_build` — **reproducible identity**.  
4. Re-prove uses agent sha **pinned to VC013** (`a56897aa…`), matching freeze evidence agent identity.

**Honesty about staged SemVer vs freeze packaging:**

| Layer | Value |
|-------|-------|
| On-disk freeze packaging | **`5.5.0`** (`Cargo.toml` / `package.json`) |
| R0A CLI under staged re-prove | **`deepseek-build 5.4.0`** |
| R0A agent | `0.2.120 (c3857a9)` sha `a56897aa…` |

Product crates between L3 cut and freeze are **packaging/docs/test** for the freeze unit (`8eea500` bump-only for SemVer fields). Using the **VC013-staged Path A pair** to re-prove snippet safety is **acceptable** for behavior, **if** docs do not claim the R0A binary itself printed **`5.5.0`**. Committed META is honest (`cli_version=… 5.4.0`). Freeze narrative still correctly says on-branch **`5.5.0`** packaging ≠ shipped registry.

**Residual harness gaps (non-blocking):**

- Under `--skip-build`, CLI may still resolve `ROOT/target/release/deepseek-build` without a hard pin to staged pair if env unset (agent path also falls through to tree). Operator must set `PATH_A_R0A_*` for disk-constrained re-prove; fail-closed message steers correctly when neither tree nor staged pair exists.  
- META `git_sha=8a6e951` is the **pre-commit** tip of the re-prove, not `01215c2` (expected chicken-egg: evidence generated then committed). Reviewers must treat **target** as packaging commit + **artifact identity** as staged pair, not assume META `git_sha == review SHA`.

### 3.4 SemVer / documentation integrity at tip

| Check | Result |
|-------|--------|
| Full SemVer form | **`5.5.0`** (never `5.5`) |
| cargo ≡ package.json | **PASS** |
| CHANGELOG `5.5.0` section | Present; dual review external; no publish claim |
| versions README decision-log | **`5.5.0`** freeze row + PR **#147** |
| Live vs stack | Docs claim live still **5.2.2** until merge+publish — consistent with freeze non-ship |
| PR **#147** body (read-only) | Mechanism honesty + residuals table align with VC015 evidence; labels present (`chore`/`test`/`docs`/areas) |

**Documentation drift residuals (pre-existing at tip; not introduced by product code in `01215c2`, still visible at review SHA):**

1. `KNOWN_LIMITS.md` floor-rule sentence still says monorepo tip “carries **`5.4.0`**” while Cargo is **`5.5.0`** (table rows above correctly list 5.5.0 freeze).  
2. Explicit non-claims still couple “V3-60-3 … without fresh Path A R0A” in a sentence that can be misread against the residual table that marks V3-60-3 **closed via `snippet_stale`**. Residual table + VC015 §8.3 remain the clearer SSOT.  
3. Board story table in `VISION_COMPLETE_5X_GOALS.md` still shows many VC rows as “pending” (historical board lag vs stacked READY evidence).  
4. VC015 freeze evidence §8.0 does not yet narrate the **stage-aware** `/tmp/…stage…` pair; identity is in META + commit `01215c2` message.

None of these re-open the Path A **`snippet_stale`** wire proof.

---

## 4. Severity-ranked findings

| Sev | ID | Finding | Disposition |
|-----|----|---------|-------------|
| **Medium** | F1 | `KNOWN_LIMITS` floor-rule prose lags stack tip (**5.4.0** wording vs **`5.5.0`** Cargo) | **Residual docs debt** — not product false green; fix in a docs PR; **not freeze-mechanism BLOCK** |
| **Medium** | F2 | Staged R0A CLI reports **`5.4.0`** while freeze packaging is **`5.5.0`** | **Accept with honesty** — packaging-only bump; agent sha = VC013 pin; META records real CLI version; do not claim R0A printed 5.5.0 |
| **Low** | F3 | META `git_sha=8a6e951` ≠ review target `01215c2` | **Accept** chicken-egg of committing evidence; pin review to target SHA + META identities |
| **Low** | F4 | `--skip-build` can still pick tree `target/release` if env unset | **Residual harness polish** — home fallback removed (main integrity win); operators should set `PATH_A_R0A_*` |
| **Low** | F5 | Vision board story statuses lag READY stack | **Board lag residual** — SSOT still owns priority over stale “pending” rows |
| **Info** | F6 | Spec 60 default `expire_all` still not Path A sole-proven | **Honesty residual** (documented) — freeze bar met via Spec 45 version gate |
| **Info** | F7 | Stage-aware skip-build resolution | **Integrity improvement** — prevents older `~/.deepseek-build` false results |

**No High/Critical** findings on product Path A snippet mechanism, SemVer shape, or silent ship claim.

---

## 5. Criterion roll-up (Lane A)

| Area | Call |
|------|------|
| V1 Spec 45 mint/require/stale (via VC015 R0A + stack VC006) | **GREEN** at target |
| V3-60-3 Path A parent after implement worker | **GREEN** — `snippet_stale` wire + disk |
| V2 Spec 10 spirit on this wire + prior VC007 | **GREEN** / prior pillar intact |
| V2 Spec 30 effort on this wire + prior VC008 | **GREEN** / scope residual OK |
| V2 cache visibility | **Prior VC009 intact**; not this scenario’s sole duty |
| SemVer integrity (`5.5.0` full form; cargo≡npm) | **GREEN** |
| `01215c2` harness integrity | **Net positive**; residuals F2–F4 only |
| On-branch ≠ shipped; dual review external | **Honored** |
| Dual review complete | **No** — Lane A only; Lane B independent |

---

## 6. Explicit verdict

### **READY**

Lane A adversarial review of exact target  
`01215c25c6f9fc0fd33419be01f23a8a5c22b814`  
finds **no blocking integrity, SemVer, or Spec 45 / 10 / 30 / cache regression** introduced by the stage-aware VC015 harness/evidence change.

- Path A V3-60-3 close via **`snippet_stale`** remains wire-backed.  
- Stage-aware `--skip-build` is an **integrity fix** (no home-bin false pair).  
- Freeze packaging **`5.5.0`** remains on-branch-only; not conflated with live **5.2.2**.  
- Residuals **F1–F6** must stay visible; none re-open the mechanism bar.

**Not claimed by this verdict:** dual-review complete; merge; tag; npm/GitHub publish; parent `expire_all` sole green; live registry **5.5.0**.

---

## 7. Residuals (carry)

| Residual | Owner |
|----------|--------|
| `KNOWN_LIMITS` floor-rule **5.4.0** prose lag | Docs follow-up |
| Explicit parent `expire_all` after spawn (optional Spec 60 default) | Honesty residual |
| Interactive TTY worktree **create** sole green | VC012 carry |
| Non-darwin prebuilts | ADR 0009 |
| Human-gated publish | ADR 0007 |
| Live main/npm/GitHub still **5.2.2** | Stack lag until merge+publish |
| Dual adversarial **Lane B** | External independent |
| Board “pending” rows vs stacked READY evidence | Board refresh later |
| META git_sha pre-commit vs packaging tip | Review process note |

---

## 8. Non-actions (this lane honored)

- No fetch / pull / merge  
- No product code, version bump, PR edit, tag, npm, or GitHub write  
- No Claude/Codex; no subagents  
- No full build / full owner-bar re-run (disk)  
- Only this report file authored for commit on the review branch  

---

## 9. Sign-off

| Field | Value |
|-------|--------|
| **Lane** | A |
| **Target** | `01215c25c6f9fc0fd33419be01f23a8a5c22b814` |
| **Verdict** | **READY** |
| **Blocking findings** | **None** |
| **Report path** | `docs/product/evidence/VC015_REVIEW_HEAD012_LANE_A_2026-08-08.md` |
