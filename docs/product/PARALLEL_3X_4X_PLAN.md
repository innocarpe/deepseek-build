# Parallel plan: **3.0.0 heart** + **4.0.0 L3** (through cut)

**Status:** Normative operating plan for dual-track work  
**Audience:** Owner + agents (heart-3x train and optional L3 prep worktree)  
**Last updated:** 2026-08-07  
**Active ultragoal (code train):** `heart-3x` → tag **`3.0.0`**  
**Follow-on ultragoal (after 3.0.0 green):** `fleet-4x` → tag **`v4.0.0`** (name fixed here; create ledger only after gate)

Related:

| Doc | Role |
|-----|------|
| [PRD-v3.md](./PRD-v3.md) · [HEART_3X_GOALS.md](./HEART_3X_GOALS.md) · [WAVE_3x_PR_DAG.md](./WAVE_3x_PR_DAG.md) | 3.0.0 heart fusion SSOT |
| [PRD-v4.md](./PRD-v4.md) · [WAVE_4x_PR_DAG.md](./WAVE_4x_PR_DAG.md) | 4.0.0 L3 productization SSOT |
| [PRE_3X_TEST_MATRIX.md](./PRE_3X_TEST_MATRIX.md) | Everyday regression (not vendor-full) |
| [ULTRAGOAL_CHAIN.md](./ULTRAGOAL_CHAIN.md) | Which plan is “active” |
| [HARNESS_PHILOSOPHY.md](../architecture/HARNESS_PHILOSOPHY.md) | L3 must not override L1/L2 |

---

## 1. Goal of this document

Answer, while **3.0.0 is mid-flight**:

1. How to finish **3.0.0** without thrash.  
2. How to **parallel-prep 4.0.0** so wall-clock to **v4.0.0** is minimized.  
3. What may land on **`main` now** vs what must wait.  
4. Worktree / PR / merge rules (this repo: **merge commits**, squash disabled).

This plan defines **how to execute Lane B in parallel** (docs, smoke scripts, evidence)
while heart-3x runs. It does **not** start a second full ultragoal **code** train
(`fleet-4x`) until gates in §6 pass — but Lane B is **not** “docs forever later”;
build those assets **now** in a **separate worktree**, never by stashing 3.0 WIP.

---

## 2. North-star vs major cuts

Owner identity (unchanged):

```text
Grok Build machine (L3)
  + Deep Code heart (L1)
  + Reasonix heart (L2)
  = DeepSeek Build
```

| Cut | Makes true | Does **not** claim |
|-----|------------|-------------------|
| **2.x** (done) | Grok shell + DeepSeek entry/UI/API | Hearts fused |
| **3.0.0** | L1+L2 **P0** on default Grok agent path | L3 as product identity |
| **4.0.0** | L3 mechanisms as **product defaults** + docs + evidence | Multi-vendor / greenfield agent |

**Efficiency principle:**  
- **Lane A** closes the identity **hearts** (blocking for “DeepSeek Build” sentence).  
- **Lane B** accumulates **L3 readiness assets** that do not fight Lane A.  
- **Lane C** holds risky L3 **code experiments** off `main` until hearts green.

---

## 3. Three lanes (operating model)

```text
                    ┌─────────────────────────────────────┐
  Lane A (primary)  │  heart-3x → main → tag v3.0.0       │
  bandwidth ★★★★★   │  L1 snippet/perms, L2 prefix/repair │
                    └─────────────────────────────────────┘
                                      │
                    ┌─────────────────┴───────────────────┐
  Lane B (parallel) │  worktree → PRs to main             │
  bandwidth ★★☆     │  guides, test-l3-smoke, gap, evidence│
                    └─────────────────────────────────────┘
                                      │
                    ┌─────────────────┴───────────────────┐
  Lane C (optional) │  worktree + exp/* branch (no merge) │
  bandwidth ★       │  profile defaults, fleet UX trials  │
                    └─────────────────────────────────────┘
                                      │
                         after 3.0.0 green
                                      ▼
                    ┌─────────────────────────────────────┐
  Lane D (serial)   │  fleet-4x ultragoal → tag v4.0.0    │
                    │  absorb C + execute WAVE_4x code    │
                    └─────────────────────────────────────┘
```

| Lane | Ultragoal? | Default PR base | Owner focus |
|------|------------|-----------------|-------------|
| **A** | **Yes** `heart-3x` | `main` | Almost all agent hours |
| **B** | No (prep only) | **`main`** | Second session / low duty cycle |
| **C** | No | **do not merge** | Spike only; rebase on `main` weekly |
| **D** | **Yes** `fleet-4x` | `main` | Starts only after §6 gate |

**Do not** make a long-lived `train/l3-4x` the base of all 4.0 PRs while 3.0 is hot — rebase cost dominates.

---

## 4. Touch-set firewall (avoid merge hell)

### 4.1 Lane A hot zones (Lane B/C must not “win” here)

Assume heart-3x owns these until **v3.0.0** is tagged (extend via WAVE_3x only):

| Area | Examples |
|------|----------|
| Snippet / edit contract | Grok `search_replace` path, Spec 45 adapters, `dsb-tools` snippet policy on agent path |
| Permissions | Spec 90 matrix, headless fail-closed, product default permission mode |
| Context / prefix / epoch | Agent context assembly, Spec 10 spirit |
| Repair / routing | Spec 15 repair, Flash/Pro escalate under agent |
| SemVer **3.0.0** cut | Cargo/package version, CHANGELOG ship notes for 3.0.0 |

If a 4.0 idea needs these files: **write a design note in Lane B**, implement in **Lane D** (or cherry-pick after 3.0).

### 4.2 Lane B safe zones (land on `main` now)

| Area | Examples |
|------|----------|
| Product docs | PRD-v4 refinement, WAVE_4x draft, this file, research notes |
| Inventory | What Grok L3 tools/flows already exist under DeepSeek |
| Evidence | Headless dogfood logs that **do not change** product defaults |
| Guides | User-guide drafts for worktree/subagent/bg as *documentation only* |
| Test matrix extension | Optional T5-style cases **documented** as future 4.0 gates |

### 4.3 Lane C quarantine (experiment only)

| Area | Rule |
|------|------|
| Default agent profiles favoring parallel/worktree | Exp branch only until 3.0 green + heart regression |
| Product “fleet-first” UX chrome | Same |
| Changing YOLO / allow-all for “speed” | **Forbidden** (HARNESS) |

---

## 5. Phased timeline (realistic, calendar-free)

Phases are **gates**, not calendar waits.

### Phase 0 — Now (3.0 mid-flight) ★ you are here

| Track | Work | Exit |
|-------|------|------|
| **A** | heart-3x G003→… per WAVE_3x | Continuous |
| **B** | Land PARALLEL plan + WAVE_4x **draft** + L3 gap inventory PR(s) on `main` | Docs merged |
| **C** | Optional: one exp worktree for inventory scripts only | No merge required |

**Everyday gate (both lanes):**  
`./scripts/test-pre3x-baseline.sh --live` when API key present (no vendor-full).

### Phase 1 — Through **v3.0.0** tag

| Track | Work | Exit |
|-------|------|------|
| **A** | H1 L1 → H2 L2 → H3 cut (`G004`–`G008`) | Tag **`v3.0.0`** |
| **B** | Keep gap map updated when heart PRs reveal real injection points | Map matches shipped 3.0 |
| **C** | Rebase exp onto post-3.0 `main` once; drop bitrot | Ready to harvest |

**Hard rule:** No `fleet-4x` create-goals until Phase 1 exit (unless PRD-v3/v4 explicitly amended).

### Phase 2 — 3.0.0 green → 4.0.0 train start (short)

| Work | Exit |
|------|------|
| Finalize WAVE_4x (draft → ready-for-impl) | Docs PR on `main` |
| Cold-start prompt `ULTRAGOAL_PROMPT_COLD_START_4.0.md` | On `main` |
| `omc ultragoal create-goals --plan-id fleet-4x` | Ledger exists |
| Absorb Lane C patches that still apply | PRs to `main` with heart regression green |

### Phase 3 — Execute **fleet-4x** → tag **v4.0.0**

Follow [WAVE_4x_PR_DAG.md](./WAVE_4x_PR_DAG.md):

1. L3 capability matrix (code paths + product defaults).  
2. Product defaults / profiles (parallel, bg, subagent) **without** weakening L1/L2.  
3. Worktree + subagent dogfood as product features + evidence.  
4. User-guide + README honesty for “throughput” identity.  
5. Heart regression + pre-3x live still green.  
6. SemVer **4.0.0** + tag **`v4.0.0`**.

### Phase 4 — After 4.0.0 (out of scope of this plan)

3.x minors polish, further L3 depth, optional platform work — new boards only via docs PR.

---

## 6. Gate: when may `fleet-4x` start?

**All** of the following:

1. Tag **`v3.0.0`** on `main` (or maintainer written waiver on PRD-v3).  
2. [WAVE_4x_PR_DAG.md](./WAVE_4x_PR_DAG.md) status is **ready-for-impl** (not draft-only).  
3. Heart regression: Spec 45/90/10/15/20 spirit tests still green under agent path.  
4. `./scripts/test-pre3x-baseline.sh --live` green (or documented env skip).  
5. ULTRAGOAL_CHAIN updated: active plan = `fleet-4x`.

Until then: Lane B only + optional Lane C.

---

## 7. Worktree recipe

### 7.1 Lane A (heart) — usually the primary clone

```bash
# already heart-3x working tree / session
omc ultragoal status --plan-id heart-3x
# PR base: main; merge: gh pr merge --merge
```

### 7.2 Lane B (prep) — second worktree

```bash
git fetch origin
git worktree add ../deepseek-build-l3-prep origin/main -b docs/l3-4x-prep
cd ../deepseek-build-l3-prep
# Child runtime = same as parent (Grok → grok only)
# Small PRs → base main → merge commit
```

### 7.3 Lane C (exp) — third worktree (optional)

```bash
git worktree add ../deepseek-build-l3-exp origin/main -b exp/l3-profile-defaults
# Never open “ready” product PR to main until Phase 2
# Weekly: git fetch && git rebase origin/main
```

**Disk:** do not run vendor-full tests in every worktree. Prefer installed `deepseek-build-agent` + live scripts. Clean `third_party/grok-build/target` after heavy local builds.

---

## 8. PR / merge policy (this repo)

| Item | Rule |
|------|------|
| GitHub merge | **Merge commit** (`gh pr merge --merge`) — squash **disabled** |
| Kind labels | Required on every PR |
| Public text | English only |
| Stacking | Prefer **serial**: merge A → pull main → branch B (overnight-friendly) |
| Long 4.0 integration base | **Discouraged** during Phase 0–1 |
| SemVer | Full `MAJOR.MINOR.PATCH` only; no 4.0.0 bump in Phase 0–1 |

---

## 9. Bandwidth split (suggested)

While heart-3x is active, default human/agent time:

| Lane | Share | Rationale |
|------|-------|-----------|
| A heart-3x | **~80–90%** | Blocks identity sentence |
| B main prep docs | **~10–20%** | Cheap wall-clock savings later |
| C exp | **0–10%** | Only if idle; never starves A |

If only one agent session: **A only**. Lane B is for a **second** session or human docs PR between heart merges.

---

## 10. Definition of done (this parallel plan)

This plan document is “working” when:

1. Team follows lanes without inventing a second code ultragoal mid-3.0.  
2. Lane B assets exist on `main` (WAVE_4x draft + gap map).  
3. After `v3.0.0`, `fleet-4x` can start **within one session** using cold-start 4.0 + WAVE_4x.  
4. Tag **`v4.0.0`** only with WAVE_4x H-cut evidence and hearts still green.

---

## 11. Anti-patterns

| Anti-pattern | Why bad |
|--------------|---------|
| Dual full ultragoals (heart + fleet) at once | Context split; L1/L2 regressions for “speed” |
| 4.0 default-profile PR to main before 3.0 | Touch-set collision; honesty failure |
| vendor-full as everyday gate | Disk bomb; slows both lanes |
| Calendar wait / “paper live” as evidence | Not allowed in product claims |
| Claiming 4.0.0 from docs alone | Tag requires product defaults + dogfood |

---

## 12. Immediate next actions (checklist)

### Lane A (heart session — already running)

- [ ] Continue `heart-3x` from active story (do not recreate plan with `--force`)  
- [ ] PR units only from WAVE_3x  
- [ ] Keep `./scripts/test-pre3x-baseline.sh --live` green on heart-impacting PRs  

### Lane B (execute in **separate worktree**, never stash 3.0 WIP)

- [x] This file (`PARALLEL_3X_4X_PLAN.md`)  
- [x] [WAVE_4x_PR_DAG.md](./WAVE_4x_PR_DAG.md) draft on `main`  
- [x] L3 gap inventory + code pointers  
- [x] User guides 11–14 (subagent / bg / worktree / throughput)  
- [x] `./scripts/test-l3-smoke.sh`  
- [x] [LANE_B_L3_PREP_GOALS.md](./LANE_B_L3_PREP_GOALS.md) + cold-start [ULTRAGOAL_PROMPT_LANE_B_L3.md](./ULTRAGOAL_PROMPT_LANE_B_L3.md)  
- [x] Offline smoke green (`--offline-only`)  
- [x] Live smoke: **BLOCKED** until owner restores `~/.deepseek-build/credentials.json` or `DEEPSEEK_API_KEY` (not a code gap)  
- [ ] Phase 2 only: full cold-start 4.0 text after `v3.0.0`  

**Lane B prep train status:** closable at **B008** once this checklist and ledger match (live deferred as blocked, not incomplete code).

### Lane C

- [ ] Only if needed; no merge to main in Phase 0–1  

---

## 13. Revision

Amend this file via **docs PR** when:

- heart-3x DAG changes touch sets, or  
- 3.0.0 ships and Phase 2 starts (flip ULTRAGOAL_CHAIN; mark WAVE_4x ready-for-impl).
