# PRD v3 — DeepSeek Build **3.x** (heart fusion)

| Field | Value |
|-------|--------|
| **SemVer line** | **`3.0.0` – `3.x.y`** |
| Status | **`3.0.0` cut** — L1+L2 P0 heart fusion (see §3 + evidence) |
| Owner | @innocarpe |
| Last updated | 2026-08-07 |
| Index | [versions/README.md](./versions/README.md) |
| Depends on | 2.x Grok base + DeepSeek shell ([PRD-v2.md](./PRD-v2.md)) |
| Spine | [HARNESS_PHILOSOPHY.md](../architecture/HARNESS_PHILOSOPHY.md) |

---

## 1. Problem

2.x gives a **working Grok-derived full-screen agent** with DeepSeek entry/UI, but:

- **Deep Code heart (L1)** — snippet-safe edit, permission fail-closed, skills-as-context — is **not enforced on the Grok tool path**.  
- **Reasonix heart (L2)** — stable prefix/cache epoch, Flash-first / Pro escalate, tool-call repair — is **not the controlling loop under the real shell**.  

Owner original intent was never “Grok + DeepSeek paint.” It was:

```text
Grok Build base (L3 machine)
  + Deep Code L1 contracts
  + Reasonix L2 economics
  = DeepSeek Build
```

**3.0.0** is the major that makes that sentence true for **P0 heart fusion**.

---

## 2. Decision: one major vs split

| Option | Pros | Cons |
|--------|------|------|
| **A. All unfinished → single 3.0.0** | One story | Mega-PR risk; L3 polish mixed with contracts |
| **B. 3.0 = L1+L2 P0 fusion; 4.0 = L3 max** (**chosen**) | Clear layer cuts; matches HARNESS layers | Two majors |
| **C. 3.0 = L1 only; 4.0 = L2 only** | Very clean | Owner waits longer for “both hearts” |

### Chosen: **Option B**

| Major | Owns |
|-------|------|
| **`3.0.0`** | **L1 + L2 P0 fusion under Grok shell** (Deep Code + Reasonix hearts) |
| **`3.x` minors** | Skills thrash-free, Flash/Pro polish, more UI polish (if non-breaking) |
| **`4.0.0`** | **L3 productization** — worktree/subagent/bg as product identity, not residual | see [PRD-v4.md](./PRD-v4.md) |

**Do not** put “everything left forever” into 3.0.0.  
**Do** put all **heart** work that makes L1/L2 true under the real agent into 3.0.0 P0.

---

## 3. Product definition of done — **`3.0.0`**

### P0 (ship blockers)

1. **L1 snippet-safe edit** enforced on the **default Grok tool path** (not only `dsb chat` / `dsb-tools` MVP). Contract tests adapted from Spec 45.  
2. **L1 permissions** ask/deny/allow + **headless fail-closed** on that path. Spec 90 matrix TTY vs headless.  
3. **L2 prefix/epoch discipline** on the **real full-screen agent context assembly** (tests or golden bytes under Grok stack; not only thin-path `dsb-context`). Spec 10 spirit.  
4. **Tool-call repair** (Spec 15) active on default DeepSeek turns under the agent.  
5. **Flash-first / Pro escalate** (or DeepSeek model routing equivalent) documented + dogfoodable under agent. Spec 20 spirit.  
6. **Honesty:** README / KNOWN_LIMITS state that 2.x was shell cut; 3.0.0 is heart fusion.  
7. Tag **`v3.0.0` only** when P0 above green. Full SemVer only.

### P1 (same major if ready; else 3.x)

8. Skills index vs body without thrashing prefix (Spec 70).  
9. Deeper DeepSeek TUI polish beyond 2.x chrome.  
10. MCP live process manager beyond minimal (if not blocking P0).

### Precondition (before coding 3.0.0)

Run and record the **pre-3.x test matrix** ([PRE_3X_TEST_MATRIX.md](./PRE_3X_TEST_MATRIX.md)):

| Must be green | Script |
|---------------|--------|
| T0 product offline | `./scripts/test-product-offline.sh` |
| T4 agent DeepSeek live core (when key available) | `./scripts/test-deepseek-live.sh` |
| T1 vendor curated (before vendor edits) | `./scripts/test-grok-vendor-offline.sh` |

Orchestrator: `./scripts/test-pre3x-baseline.sh --all`  
Evidence: `docs/product/evidence/PRE3X_BASELINE_*.md`

Do **not** start L1/L2 fusion while T4.0 (agent still on Grok proxy) is red.

### Non-goals for 3.0.0

- Replacing Grok base with greenfield agent  
- Multi-vendor identity  
- Gajae multi-stage planning core  
- Full L3 “fleet OS” (→ 4.0.0)  

---

## 4. Architecture (target)

```text
dsb → deepseek-build-agent (Grok composition root)     [2.x already]
         │
         ├─ tools/edit ──► L1 snippet + permission policy   [3.0 NEW]
         ├─ context ─────► L2 stable prefix + epoch + repair [3.0 NEW]
         ├─ routing ─────► Flash default / Pro escalate      [3.0 NEW]
         └─ parallel/subagent/worktree ──► keep Grok L3      [2.x; max in 4.0]
```

**Conflict rule unchanged:** L3 never ships by violating L1/L2.

---

## 5. PR / ultragoal shape (plan before code)

| Artifact | Path |
|----------|------|
| Plan id | **`heart-3x`** |
| Board | [HEART_3X_GOALS.md](./HEART_3X_GOALS.md) (G001–G008) |
| PR DAG | [WAVE_3x_PR_DAG.md](./WAVE_3x_PR_DAG.md) |
| Cold start | [ULTRAGOAL_PROMPT_COLD_START_3.0.md](./ULTRAGOAL_PROMPT_COLD_START_3.0.md) |
| Chain | [ULTRAGOAL_CHAIN.md](./ULTRAGOAL_CHAIN.md) |
| Baseline | [PRE_3X_TEST_MATRIX.md](./PRE_3X_TEST_MATRIX.md) |

| Story | Content | SemVer band |
|-------|---------|-------------|
| G001–G002 | Prep harness + plan docs on main | `2.0.x` / docs |
| G003 | Spec map under Grok tools | docs/spec |
| G004–G005 | L1 snippet + permissions | `3.0.0-alpha.N` |
| G006–G007 | L2 prefix + repair / Flash-Pro | `3.0.0-beta.N` |
| G008 | Cut **3.0.0** | **3.0.0** |

Extend the DAG only via docs PR — do not invent overnight units.

---

## 6. Success feeling (3.0.0)

> Type `dsb` → DeepSeek TUI agent runs **fast like Grok**, but edits/permissions/cache  
> behave like a **DeepSeek-native harness** (Deep Code + Reasonix), not a painted Grok.

Until that is true, do not call heart fusion done.

---

## 7. Out of 3.0 → 4.0

See [PRD-v4.md](./PRD-v4.md): L3 productization (worktree fleets, subagent product UX, multi-wait orchestration as default identity).
