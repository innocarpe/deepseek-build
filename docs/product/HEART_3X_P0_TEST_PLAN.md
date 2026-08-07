# Heart 3.x — P0 failure modes and red→green test plan

**Wave unit:** `3x-H0-2` · **Story:** G003 SpecMap · **Plan:** `heart-3x`  
**Binding map (H0-1):** [HEART_3X_SPEC_BINDING.md](../architecture/HEART_3X_SPEC_BINDING.md)  
**DoD:** [PRD-v3.md](./PRD-v3.md) §3 P0  
**Baseline (pre-heart):** [PRE_3X_TEST_MATRIX.md](./PRE_3X_TEST_MATRIX.md)

**Purpose:** Written cases that **`v3.0.0` cut must have green**.  
Each heart lists: failure modes (why users get a painted Grok), red state today, green evidence required, owning story.

**Path A** = default agent (`dsb` → `deepseek-build-agent`).  
**Path B** = thin (`dsb run` / `dsb-tools` / `dsb-context`). Path B green alone never ships 3.0.0.

Everyday regression (disk-safe):

```bash
./scripts/test-pre3x-baseline.sh --live   # when DEEPSEEK_API_KEY available
# or offline core:
./scripts/test-product-offline.sh
```

Do **not** run `test-grok-vendor-offline.sh --full` as everyday gate.

---

## Status legend

| State | Meaning |
|-------|---------|
| **RED** | Required for 3.0.0; not proven on Path A |
| **AMBER** | Partial (thin only, docs only, or dogfood missing) |
| **GREEN** | Path A automated and/or recorded dogfood meets DoD |
| **SKIP** | Blocked on env (API key, human OTP) — not a pass |

---

## Preconditions (must stay green while hearts land)

| ID | Case | Red if | Green evidence | Owner |
|----|------|--------|----------------|-------|
| **T4.0** | Agent models hit DeepSeek, not Grok proxy | Response/log shows `cli-chat-proxy.grok.com` or missing per-model `base_url` | Live agent turn uses `https://api.deepseek.com`; seed/repair in `agent_launch` | G001 / PRE_3X |
| **T0** | Product offline dual-bin + config seed | SemVer mismatch; missing bins | `./scripts/test-product-offline.sh` | ongoing |
| **Agent bin** | `deepseek-build-agent` installable | T2.1 fail “agent binary missing” | `./scripts/build-grok-pager.sh release && ./scripts/install.sh` | install docs |

---

## P0-1 — L1 snippet-safe edit (Spec 45 spirit) · Story **G004**

### Failure modes

| # | Failure | User-visible harm |
|---|---------|-------------------|
| F45.1 | Default Path A edit is free-form SearchReplace without session snippet / version scope | Model overwrites wrong region; no stale detection |
| F45.2 | Primary path is whole-file `old_string`/`new_string` without scope | Spec 45 bypass; “Grok paint” edits |
| F45.3 | Ambiguous multi-match applies first hit | Silent wrong edit |
| F45.4 | External/file change after read still applies | Stale write |
| F45.5 | Only Path B has snippet tests | False confidence for `dsb` TUI |

### Red → green checklist

| ID | Case | Expected green | How to prove |
|----|------|----------------|--------------|
| **H45.1** | Path A primary edit requires snippet-safe contract (snippet_id **or** documented equivalent version+scope issued by read/hashline) | Invalid/missing scope → **no mutation** | Automated contract test against Grok tool path or product wrapper |
| **H45.2** | Free-form whole-file as **default** primary is rejected or non-default | Fail closed / not default tool | Same suite; config/toolset assertion |
| **H45.3** | Zero match / multi match | `no_match` / candidates; no guess | Unit/integration |
| **H45.4** | Stale version | `snippet_stale` (or equivalent); no apply | Unit/integration |
| **H45.5** | Thin regression still green | `cargo test -p dsb-tools` snippet suite | CI local |

**Done when:** H45.1–H45.4 green on Path A + H45.5; band `3.0.0-alpha.N` per WAVE.

---

## P0-2 — L1 permissions (Spec 90 spirit) · Story **G005**

### Failure modes

| # | Failure | User-visible harm |
|---|---------|-------------------|
| F90.1 | Headless agent allows mutating tools without allowlist | Silent tree damage in CI/scripts |
| F90.2 | Product default is YOLO / always-allow | Policy theater |
| F90.3 | TTY never asks for high side-effect scopes | No human gate |
| F90.4 | CapabilityMode filters tools but no ask/deny audit path | Incomplete Spec 90 |
| F90.5 | Bash/file mutate outside workspace without deny | Escape |

### Red → green checklist

| ID | Case | Expected green | How to prove |
|----|------|----------------|--------------|
| **H90.1** | Headless + ask-class action | **Deny** (fail-closed) unless pre-approved policy | Matrix test Path A or product-enforced wrapper |
| **H90.2** | TTY + ask-class action | **Ask** (or documented allow after grant) | Matrix / dogfood |
| **H90.3** | Explicit deny policy | No execute; structured error to model | Automated |
| **H90.4** | Default config is not YOLO-only | Seeded config / docs + test | Config golden + code assert |
| **H90.5** | Dogfood: headless edit under DeepSeek with L1 policy on | Evidence note under `docs/product/evidence/` | G005 dogfood file |

**Done when:** H90.1–H90.5 green; H1 exit (L1 true on real agent tool path).

---

## P0-3 — L2 prefix / epoch (Spec 10 spirit) · Story **G006**

### Failure modes

| # | Failure | User-visible harm |
|---|---------|-------------------|
| F10.1 | Every agent turn rebuilds unstable system dump | Cache miss economics; slower than Grok |
| F10.2 | Only thin `dsb-context` has epoch tests | Path A unmeasured |
| F10.3 | Snippet tables / timestamps / random IDs in stable prefix | Epoch thrash |
| F10.4 | Tool schema key order non-canonical | Silent epoch churn |

### Red → green checklist

| ID | Case | Expected green | How to prove |
|----|------|----------------|--------------|
| **H10.1** | Two consecutive Path A context assemblies with identical inputs | Equal stable-prefix bytes or equal epoch hash | Golden / hash-stable test under Grok stack |
| **H10.2** | Deliberate system/tool/skills-index change | Epoch **changes** | Same harness |
| **H10.3** | Volatile tail (user turn, tool results) does not alter stable epoch | Epoch stable across turns when prefix inputs fixed | Integration |
| **H10.4** | Thin `dsb-context` still green | `cargo test -p dsb-context` | Regression |

**Done when:** H10.1–H10.3 on Path A; band `3.0.0-beta.N`.

---

## P0-4 — Tool-call repair (Spec 15) · Story **G007**

### Failure modes

| # | Failure | User-visible harm |
|---|---------|-------------------|
| F15.1 | Malformed tool JSON executes anyway | Arbitrary tool havoc |
| F15.2 | No repair; model stuck after trailing comma | Fragile DeepSeek turns |
| F15.3 | Repair invents required args or renames tool | Spec violation |
| F15.4 | Repair only on Path B | Default agent unprotected |

### Red → green checklist

| ID | Case | Expected green | How to prove |
|----|------|----------------|--------------|
| **H15.1** | Trailing comma / single-quoted args | One repair → execute **or** structured error after 1 attempt | Unit against agent dispatch path |
| **H15.2** | Missing required arg after repair | **No execute**; error to model | Negative test |
| **H15.3** | Never rename tool / invent required fields | Asserted in tests | Unit |
| **H15.4** | Default DeepSeek agent turns use repair | Integration or mock multi-turn tool | G007 evidence |

**Done when:** H15.1–H15.4 green on Path A.

---

## P0-5 — Flash-first / Pro escalate (Spec 20 spirit) · Story **G007**

### Failure modes

| # | Failure | User-visible harm |
|---|---------|-------------------|
| F20.1 | Default model is Pro or Grok id | Cost / wrong product |
| F20.2 | Escalate sticky forever after one `/pro` | Silent cost | 
| F20.3 | Wire model not visible per turn | No dogfoodability |
| F20.4 | `base_url` missing on one model table | Partial T4.0 regress |

### Red → green checklist

| ID | Case | Expected green | How to prove |
|----|------|----------------|--------------|
| **H20.1** | Session default Flash (`deepseek-v4-flash`) | Config + live/offline assert | Config golden; live when key |
| **H20.2** | One-shot Pro then return (unless sticky max) | Documented + testable switch | Unit/routing under agent or product docs + dogfood |
| **H20.3** | Turn shows which wire model ran | Log/UI field | Dogfood or integration |
| **H20.4** | Both model tables have `base_url` | Seed/repair tests | `dsb-cli` product_config tests + live T4.0 |

**Done when:** H20.1–H20.4; H2 exit with repair.

---

## P0-6 — Honesty docs · Story **G008**

| ID | Case | Green evidence |
|----|------|----------------|
| **HDOC.1** | README / KNOWN_LIMITS: 2.x = shell cut; 3.0.0 = heart fusion | Docs match behavior |
| **HDOC.2** | Claimed-vs-shipped in PRD-v3 updated | Table green only for true items |
| **HDOC.3** | Pre-3x + heart suites green for cut | `test-pre3x-baseline.sh --live` + heart tests; evidence file |
| **HDOC.4** | SemVer **3.0.0** + tag **`v3.0.0`** | Cargo + package.json + CHANGELOG; **never** bare `3.0` |
| **HDOC.5** | npm registry publish | Human only (ADR 0007) — agent must not force publish |

---

## Aggregate cut gate (G008)

All of the following:

1. Preconditions T4.0 (when key) + T0 not regressed.  
2. **H45.*** green (G004).  
3. **H90.*** green (G005).  
4. **H10.*** green (G006).  
5. **H15.*** + **H20.*** green (G007).  
6. **HDOC.*** done; tag **`v3.0.0`** only then.

### Snapshot template (update at cut)

```text
Preconditions:  ?/3
H45 snippet:    ?/5
H90 perms:      ?/5
H10 prefix:     ?/4
H15 repair:     ?/4
H20 routing:    ?/4
HDOC honesty:   ?/5
```

---

## Mapping to WAVE_3x

| WAVE unit | Cases | Story |
|-----------|-------|-------|
| 3x-H1-1 | H45.* | G004 |
| 3x-H1-2, 3x-H1-3 | H90.* | G005 |
| 3x-H2-1 | H10.* | G006 |
| 3x-H2-2 | H15.* | G007 |
| 3x-H2-3 | H20.* | G007 |
| 3x-H3-* | HDOC.* + full aggregate | G008 |

---

## Out of this plan

- T5 extended Grok surface completeness (optional, not P0).  
- Spec 70 skills thrash-free full (3.x minor if non-breaking).  
- L3 fleet productization (4.x).  
- Vendor-full offline as everyday gate.
