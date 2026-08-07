# PRD v5 — DeepSeek Build **5.x** (owner-bar complete product)

| Field | Value |
|-------|--------|
| **SemVer line** | **`5.0.0` – `5.x.y`** |
| Status | **Active train planned:** `owner-bar-5x` |
| Owner | @innocarpe |
| Last updated | 2026-08-07 |
| Index | [versions/README.md](./versions/README.md) |
| **DoD (fail-close)** | [OWNER_BAR_ACCEPTANCE.md](./OWNER_BAR_ACCEPTANCE.md) + [OWNER_BAR_P0_LEDGER.md](./OWNER_BAR_P0_LEDGER.md) |
| PR units | [WAVE_5x_PR_DAG.md](./WAVE_5x_PR_DAG.md) |
| Ultragoal board | [OWNER_BAR_5X_GOALS.md](./OWNER_BAR_5X_GOALS.md) |
| Cold start | [ULTRAGOAL_PROMPT_COLD_START_5.0.md](./ULTRAGOAL_PROMPT_COLD_START_5.0.md) |
| Plan reviews | [evidence/OWNER_BAR_5X_PLAN_ADVERSARIAL_2026-08-07.md](./evidence/OWNER_BAR_5X_PLAN_ADVERSARIAL_2026-08-07.md) |

---

## 1. Problem

Tags **`3.0.0`** and **`4.0.0`** claimed heart fusion and L3 productization. Adversarial dual-model review (2026-08-07) proved:

| Claim | Reality on **Path A** (default `dsb` → `deepseek-build-agent`) |
|-------|----------------------------------------------------------------|
| L1 snippet-safe default | **Dead wiring** — `snippet_safe` built for Standard toolset but applied only when `effective != Standard` |
| `file_version` / snippet mint | **Missing** on Grok `read_file` — flipping safety alone **bricks edits** |
| L2 prefix / repair / Pro router | **Library / thin path only** — not in Grok compile graph |
| L3 product identity | Machinery exists; live R0 / worker cache law / claim honesty incomplete |
| Dual CLI + DeepSeek base_url + TUI | Real strengths — keep and re-prove |

Root failure mode: **Path B unit tests + docs + tags** closed majors while **Path A** stayed Grok shell + DeepSeek paint.

---

## 2. Why `5.0.0` (not re-tag 3/4)

| Option | Reject because |
|--------|----------------|
| Re-open 3.0.0 / 4.0.0 tags as “now really done” | Tags already published; rewrite history confuses users |
| Ship fusion as 4.0.2 | Owner bar is a **product identity** jump, not a patch |
| **New major `5.0.0`** | Honest: previous majors = partial; complete product is new major |

**Honesty table (claimed vs allowed):**

| Line | May claim | Must not claim |
|------|-----------|----------------|
| 2.x | Grok shell + DeepSeek entry | Heart fusion |
| 3.x | Heart fusion *attempt* / library hearts | Path A L1+L2 control |
| 4.x | L3 machinery / partial defaults | Complete fleet product identity |
| **5.0.0** | Owner-bar complete product | Only when ledger all PASS |

---

## 3. Product definition of done — **`5.0.0`**

Normative detail: [OWNER_BAR_P0_LEDGER.md](./OWNER_BAR_P0_LEDGER.md).

### P0 ship blockers (summary)

1. **Public Path A** — installed `deepseek-build` / `dsb` launch real agent; evidence via public entry (not raw-agent-only final proof).  
2. **L1 on Path A** — mint version → snippet-safe default → liveness (≥3 edits / ≥2 files) → write/bash invalidate.  
3. **L1 perms** — non-YOLO, headless fail-closed, boundary + no parallel/subagent bypass.  
4. **L2 on Path A** — prefix goldens from **captured wire**, repair on dispatch, Flash/Pro escalate + effort.  
5. **L3 under hearts** — parallel/bg/subagent/worktree re-proved **without** L1/L2 regression; worker cache law.  
6. **Install** — clean primary platform, dual CLI, agent present.  
7. **Mechanical gate** — `./scripts/test-owner-bar.sh` exit 0; no SKIP/BLOCKED/N/A; dual adversarial review same SHA+manifest.  
8. Tag **`v5.0.0`** only after formula in ledger § Cut.

### Explicit non-goals

- Greenfield agent replacing Grok base  
- Multi-vendor core  
- Gajae multi-stage planning harness  
- Claiming 5.0.0 from docs/library APIs alone  
- Cut-time N/A waivers  
- Everyday vendor-full cargo (disk bomb)  
- Agent-forced npm publish (ADR 0007 human-gated)

---

## 4. Architecture (unchanged layers)

```text
Path A (ONLY product path for P0):
  deepseek-build | dsb
    → agent_launch (seed, base_url repair, GROK_HOME)
    → deepseek-build-agent
    → xai-grok-shell / tools / workspace
    → DeepSeek API (chat_completions)

L1 Deep Code  controls edit + permissions + skills surface
L2 Reasonix   controls prefix + repair + Flash/Pro
L3 Grok       controls throughput WITHOUT violating L1/L2
```

Conflict: [HARNESS_PHILOSOPHY.md](../architecture/HARNESS_PHILOSOPHY.md).  
Path A vs B: [HEART_3X_SPEC_BINDING.md](../architecture/HEART_3X_SPEC_BINDING.md) — still true: Path B green ≠ product.

---

## 5. Ultragoal shape

| Plan id | Stories | Board |
|---------|---------|--------|
| **`owner-bar-5x`** | **G001–G012** | [OWNER_BAR_5X_GOALS.md](./OWNER_BAR_5X_GOALS.md) |

```text
G001 gate RED + honesty
  → G002 Path A R0 rig
  → G003 mint → G004 snippet live+liveness → G005 write/bash
  ∥ G006 perms ∥ G007 repair ∥ G009 routing (after rig; prefix after schema)
  → G008 prefix/skills/resume goldens
  → G010 L3 re-prove under hearts
  → G011 install
  → G012 freeze + dual review + tag v5.0.0
```

---

## 6. Success feeling

Type `deepseek-build` or `dsb` → Grok-class TUI opens → multi-step coding is **fast** → long sessions **affordable** → edits **safe and still work** → permissions honest → screen readable.

If any clause fails owner-bar R0A, **do not** ship `5.0.0`.

---

## 7. Status

| Item | State |
|------|-------|
| Plan package on disk | **This PR train** |
| Gate scripts | RED baseline required before feature stories close |
| Feature fusion | **Not started** under owner-bar rules |
| Tag `v5.0.0` | **Forbidden** until ledger all PASS |
