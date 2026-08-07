# Owner-bar acceptance checklist (true product)

| Field | Value |
|-------|--------|
| **Status** | **Normative gate** — supersedes false “3.0.0 / 4.0.0 complete” claims for the **owner product bar** |
| **Audience** | Humans + agents planning / cutting majors (**`5.0.0` / `owner-bar-5x`**) |
| **Last updated** | 2026-08-07 |
| **On-disk SemVer today** | Read root `Cargo.toml` (do not hardcode; may lag this bar) |
| **Frozen P0 list** | **[OWNER_BAR_P0_LEDGER.md](./OWNER_BAR_P0_LEDGER.md)** — machine cut source |
| **Train** | [PRD-v5.md](./PRD-v5.md) · [OWNER_BAR_5X_GOALS.md](./OWNER_BAR_5X_GOALS.md) · [WAVE_5x_PR_DAG.md](./WAVE_5x_PR_DAG.md) · [ULTRAGOAL_PROMPT_COLD_START_5.0.md](./ULTRAGOAL_PROMPT_COLD_START_5.0.md) |
| **Plan reviews** | [evidence/OWNER_BAR_5X_PLAN_ADVERSARIAL_2026-08-07.md](./evidence/OWNER_BAR_5X_PLAN_ADVERSARIAL_2026-08-07.md) (Claude Opus + Codex gpt-5.6-sol) |
| **Related** | [VISION.md](./VISION.md) · [HARNESS_PHILOSOPHY.md](../architecture/HARNESS_PHILOSOPHY.md) · [HEART_3X_SPEC_BINDING.md](../architecture/HEART_3X_SPEC_BINDING.md) · specs under `docs/specs/` · [KNOWN_LIMITS.md](./KNOWN_LIMITS.md) |

---

## 0. Why this file exists

Past majors closed on **library APIs + unit tests + docs + tags** while the **default user path** stayed “Grok shell + DeepSeek paint.”

That pattern is **forbidden** for any release that claims the owner product:

> **DeepSeek Coding Agent + TUI** = Grok Build–class agent/TUI **and** Grok-class speed/parallelism **as product identity**, with **Reasonix** and **Deep Code** design advantages **controlling the real agent path** (not a side library, not thin `dsb run` only).

This checklist is the **single fail-close definition of “done.”**  
No PR, ultragoal, or SemVer cut may claim that product without every **P0** row green under the rules below.

**Historical tags** `v3.0.0` / `v4.0.0` / current `4.x` may exist on disk. They do **not** satisfy this bar. Treat them as **partial shell / partial L3 / unfused hearts** until this file says otherwise.

---

## 1. One-sentence product (unchanged)

From [VISION.md](./VISION.md) + owner restatement:

**DeepSeek Build** is a DeepSeek-native terminal coding agent that feels as fast as **Grok Build**, as cheap on long sessions as **Reasonix**, and as correctly tuned to DeepSeek V4 as **Deep Code CLI** — delivered as dual commands **`deepseek-build`** / **`dsb`**.

**Success feeling:** type `deepseek-build` or `dsb` → Grok-class full-screen agent opens → real multi-step coding is fast → cost stays sane → edits are safe → UI is readable (DeepSeek blue, not Grok low-contrast black).

---

## 2. Runtime under test (fail-close)

### 2.1 Only Path A counts for product P0

| Path | Entry | Counts as product? |
|------|-------|--------------------|
| **A. Default product** | Bare TTY `dsb` / `deepseek-build` → `agent_launch` → **`deepseek-build-agent`** (vendored Grok pager / shell / tools) | **YES — only this for P0** |
| **B. Thin / legacy** | `dsb run`, `dsb chat`, `repl-legacy`, pure `dsb-*` unit tests | **NO** — useful as reference impl / regression, **never** sole cut evidence |

```text
MUST PROVE ON:
  dsb | deepseek-build   (TTY or product headless agent flags)
    → deepseek-build-agent
    → xai-grok-shell loop
    → xai-grok-tools / workspace / sampler → api.deepseek.com

MUST NOT CLAIM FROM ONLY:
  cargo test -p dsb-agent / dsb-context / dsb-tools path_a_*
  dsb run | dsb chat
```

Binding history: [HEART_3X_SPEC_BINDING.md](../architecture/HEART_3X_SPEC_BINDING.md) §1  
(“Path B green alone does not cut the major”).

### 2.2 Evidence classes (ordered)

Every P0 item needs **at least one** of:

| Class | Code | Meaning |
|-------|------|---------|
| **R0A** Public Path A runtime | **required for hearts + L3 identity** | Evidence via **installed public** `deepseek-build` / `dsb` (or product `agent` subcommand through `agent_launch`) with `DEEPSEEK_BUILD_AGENT_BIN` unset unless recorded as non-cut. Wire/process tree + SHA + binary hash. **Not** raw `deepseek-build-agent` alone as final cut proof. |
| **R1** Link graph | required for code-bound hearts | Path A binary / Grok composition **depends on** or **embeds** the implementation |
| **R2** Unit / crate test | **insufficient alone** | Support only; **never** sole green for a P0 heart |
| **R3** Docs / matrix | **never sufficient** | Guides, PRD, evidence MD without R0A = not done |

**Anti-game rules (mechanical):**

1. **Call-site rule:** production call site on Path A — not only `lib.rs` export + `#[cfg(test)]`.  
2. **Nominal-fraud rule:** a symbol named `path_a_*` / `PathA*` is evidence of **nothing** without a non-test call site outside its defining file (`./scripts/check-path-a-linkage.sh`).  
3. **Default-path rule:** params only applied when `effective != Standard` while default is Standard = **FAIL**.  
4. **Mint-before-flip:** `read_file` must mint `file_version` / `snippet_id` **before** enabling `snippet_safe` reject path (else product bricks).  
5. **Liveness:** with safety on, Path A must still perform **≥3 successful edits across ≥2 files** (ledger **L1-45-0**).  
6. **No-skip:** cut statuses may only be `PASS` or `FAIL`. `SKIP` / `BLOCKED` / `N/A` / `NOT_RUN` / `XFAIL` = hard fail. Hermetic scripted-provider R0A mandatory during train; **live DeepSeek R0A mandatory at cut**.  
7. **Freshness:** no carried-forward MET; re-run against cut SHA + binary manifest.  
8. **Gate first:** `./scripts/test-owner-bar.sh` exists and is RED before feature stories close (G001).  
9. **No tag-first / no dual ledger:** docs singular; tag after all P0 PASS.  
10. **L3 never weakens L1/L2** ([HARNESS_PHILOSOPHY.md](../architecture/HARNESS_PHILOSOPHY.md) §3).  
11. **Dual independent adversarial reviews** on the same frozen SHA + binary/evidence manifest (G012). Neither reviewer is the implementation author.

### 2.3 Status legend (fill at cut time)

| Mark | Meaning |
|------|---------|
| **MET** | R0 (and R1 where required) green; evidence path recorded |
| **PARTIAL** | Real machinery exists but default path / dogfood / residual honesty fails |
| **MISSING** | Not controlling Path A |
| **N/A** | Explicit non-goal for this cut (must be listed in §9) |

**Baseline as of 2026-08-07 (adversarial dual-model + tree trace):** owner bar is **not met**. Rows below use that baseline until re-verified.

---

## 3. Master checklist — product identity

### 3.1 Shell & packaging (foundation)

| ID | Requirement | Spec / source | Evidence required | Baseline |
|----|-------------|---------------|-------------------|----------|
| **S1** | Dual CLI: **`deepseek-build`** primary + **`dsb`** alias, same behavior | ADR 0006, AGENTS.md | Both names install; `--help` / version match | **MET** (verify on cut) |
| **S2** | Bare TTY opens **full-screen Grok-derived agent** (not thin REPL as default) | HEART binding Path A | Process: `dsb` → `deepseek-build-agent` | **MET** |
| **S3** | Product home `~/.deepseek-build/`; `GROK_HOME` bridge does not strand config | agent_launch | Config seed + agent sees product home | **MET** |
| **S4** | Every DeepSeek model seed has **`base_url = https://api.deepseek.com`** (model-level; not endpoints-only) | ADR 0005, G001 | Live/offline: no Grok proxy 401 on agent path | **MET** |
| **S5** | Default UI readable (**DeepSeek blue** family), not Grok near-black low contrast | VISION, design | Screenshot or theme seed dogfood | **PARTIAL** — re-verify on cut |
| **S6** | Full SemVer **`MAJOR.MINOR.PATCH`** everywhere (no bare `5.0`) | versioning.md | `./scripts/check-semver.sh` | **MET** process |
| **S7** | Install real: cargo and/or npm prebuilt delivers **CLI + agent** for claimed platforms | ADR 0007, npm | Clean-machine smoke: both CLIs open TUI | **PARTIAL** — prove per release |
| **S8** | No default **YOLO**; product default `yolo = false` / Ask | Spec 90, NON_GOALS | Seed + headless deny/cancel without YOLO | **MET** |

---

## 4. Master checklist — L1 Deep Code (must control Path A)

Philosophy: [HARNESS_PHILOSOPHY.md](../architecture/HARNESS_PHILOSOPHY.md) §4.

### 4.1 Spec 45 — Snippet-safe edit

| ID | Requirement | Evidence (Path A) | Baseline |
|----|-------------|-------------------|----------|
| **L1-45-0** | **Liveness:** with safety on, ≥3 successful Path A edits / ≥2 files / exit 0 (anti-brick) | R0A scripted session + diff | **MISSING** |
| **L1-45-1** | Primary edit path is **snippet-safe / version-scoped**, not free-form whole-file | Default tool registration on agent actually sets safety params; agent e2e | **MISSING** (dead wiring: Standard `tool_configs` not applied on default path) |
| **L1-45-2** | `read` (or product equivalent) **issues** a session token (`snippet_id` **or** documented `file_version` equivalent) the model can use — **must land before L1-45-1 flip** | Tool result fields on Path A wire; R0A | **MISSING** (no mint on Grok `read_file`) |
| **L1-45-3** | Edit **requires** valid token; missing → **reject**, do not execute | Negative e2e on Path A | **MISSING** (default `snippet_safe=false`) |
| **L1-45-4** | Stale version / file changed under agent → **reject** | Negative e2e | **MISSING** on default path |
| **L1-45-5** | Ambiguous / non-unique match → candidates or error, **no silent wrong replace** | Negative e2e | **PARTIAL** at best (Grok SR behavior; not full Spec 45) |
| **L1-45-6** | Empty `old_string` does **not** whole-file overwrite as free primary path | Negative e2e | **MISSING** as product default |
| **L1-45-7** | `write` overwrite of existing file obeys same version/safety spirit (create-new default) | Spec 45 bypass law | **MISSING** / unproven on Path A |
| **L1-45-8** | Bash mutations **invalidate** outstanding snippets for touched paths | Spec 45 + 90 | **MISSING** / unproven on Path A |

**Minimum ship shape (allowed equivalent):** Spec 45 full `snippet_id` **or** documented **file_version (sha256) + scope** with the same fail-closed properties — but it must be **default-on Path A**, not a helper that never runs.

### 4.2 Spec 90 — Permissions

| ID | Requirement | Evidence (Path A) | Baseline |
|----|-------------|-------------------|----------|
| **L1-90-1** | Mutating tools go through allow / deny / **ask** | Capability + prompt path | **MET** (Grok machinery; keep regression) |
| **L1-90-2** | Headless: Ask → **deny/cancel** unless explicit YOLO / pre-grant | Headless e2e | **MET** / re-verify |
| **L1-90-3** | Product default is **not** YOLO-only | Config seed | **MET** |
| **L1-90-4** | Workspace boundary: out-of-workspace high-risk paths ask/deny per matrix | Matrix tests on Path A | **PARTIAL** — prove matrix on agent path |
| **L1-90-5** | Parallel / subagent tools **do not** skip permission checks | Spec 50/60 honesty | **PARTIAL** — prove |

### 4.3 Spec 70 / 30 / 80 / 100 / 110 (L1 surface — P0 vs P1)

| ID | Requirement | P0 for owner-bar major? | Baseline |
|----|-------------|-------------------------|----------|
| **L1-70** | Skills as structured context: **index** in stable prefix; body load on demand (thrash-free spirit) | **P0** index + load path on Path A | **PARTIAL** |
| **L1-30** | Thinking / effort knobs first-class for DeepSeek wire (not only hidden env) | **P0** Flash coding default effort; UX dogfoodable | **PARTIAL** |
| **L1-80** | MCP mountable without breaking prefix/permission contracts | **P1** unless already product-critical | thin/Path A gap |
| **L1-100** | Sessions resume on product agent path | **P0** resume works on Path A | **PARTIAL** (Grok sessions exist; product honesty) |
| **L1-110** | Plan mode light (optional assist, not Gajae multi-stage trap) | **P1** | per NON_GOALS |

---

## 5. Master checklist — L2 Reasonix (must control Path A)

### 5.1 Spec 10 — Stable prefix / cache epoch

| ID | Requirement | Evidence (Path A) | Baseline |
|----|-------------|-------------------|----------|
| **L2-10-1** | Every main-agent DeepSeek request is **stable_prefix + volatile_tail** | Instrument or golden on **Grok message assembly**, not only `assemble_path_a_context` unit tests | **MISSING** |
| **L2-10-2** | Prefix contents ordered per Spec 10 (system, tools canonical, skills index, env, project instructions) | Golden / hash | **MISSING** on Path A |
| **L2-10-3** | Unchanged inputs → **byte-stable** prefix (canonicalize rules) across turns | Golden hash two turns | **MISSING** on Path A |
| **L2-10-4** | Volatile only: user/tool/dynamic (no wall-clock in prefix) | Negative golden | **MISSING** on Path A |
| **L2-10-5** | Compaction / resume preserves contract (no silent thrash of entire prefix every turn) | Resume dogfood + hash | **MISSING** / residual admitted in KNOWN_LIMITS |
| **L2-10-6** | Library `assemble_path_a_context` is either **called from Path A** or **deleted/demoted** as thin-only | Call-site or honesty doc | **MISSING** (library-only today) |

### 5.2 Spec 15 — Tool-call repair

| ID | Requirement | Evidence (Path A) | Baseline |
|----|-------------|-------------------|----------|
| **L2-15-1** | Before tool execute on Path A: parse → **one** repair pass → else structured error | R0: trailing comma / single-quote fixtures | **MISSING** (not Reasonix path; Grok has own normalize only) |
| **L2-15-2** | Never invent required args; never rename tool | Negative tests | **MISSING** on Path A as Spec 15 |
| **L2-15-3** | Session load: tool_call / tool_result pairing holes repaired or interrupted placeholders | Resume path | **PARTIAL** (thin has pairing; Path A unproven as Spec 15) |
| **L2-15-4** | Repair is **on the dispatch path** of default agent tools, not only `dsb-agent::repair` | Call-site R1+R0 | **MISSING** |

### 5.3 Spec 20 — Flash / Pro routing

| ID | Requirement | Evidence (Path A) | Baseline |
|----|-------------|-------------------|----------|
| **L2-20-1** | Default session model is **Flash** (`deepseek-v4-flash`) | Config + wire log / status | **MET** (static seed) |
| **L2-20-2** | **Pro escalate** dogfoodable (`/pro` or product equivalent) for **one turn** then return (or sticky preset per Spec 20) | R0 on TUI path | **MISSING** (router lives in thin `path_a_turn`) |
| **L2-20-3** | User-visible which wire model ran the turn | UI or log | **PARTIAL** |
| **L2-20-4** | Precedence: explicit user > sticky preset > auto > default Flash | Table tests on Path A | **MISSING** |
| **L2-20-5** | Both models always carry correct DeepSeek `base_url` | Live/offline | **MET** |

---

## 6. Master checklist — L3 Grok throughput (product identity)

Not “upstream has a flag.” **Product identity** = defaults + docs + dogfood on Path A without weakening L1/L2.

| ID | Requirement | Spec | Evidence (Path A) | Baseline |
|----|-------------|------|-------------------|----------|
| **L3-50-1** | Multi-tool turn: **read-only parallel**, **mutating serial** | 50 | R0 concurrent reads; serial edits | **PARTIAL** (machinery exists; product dogfood incomplete) |
| **L3-50-2** | Classification fail-closed: unknown / bash / MCP treated mutating for schedule | 50 | Unit + R0 | **PARTIAL** |
| **L3-50-3** | Background shell + collect-by-id works as product feature | 50 | R0 live or hermetic | **PARTIAL** |
| **L3-50-4** | Auto-background / wait patterns usable without secret flags | 50 | Dogfood | **PARTIAL** |
| **L3-60-1** | Subagents **enabled by product default** | 60 | Config + spawn e2e | **PARTIAL** (seed/enabled; live probes weak) |
| **L3-60-2** | Explore (read-only) + implement (mutating) kinds or product equivalents | 60 | R0 | **PARTIAL** |
| **L3-60-3** | **Worker cache law:** workers reuse parent stable prefix template | 60 + 10 | Prefix hash parent vs worker | **MISSING** until L2-10 on Path A |
| **L3-60-4** | Worker path mutation **invalidates** parent snippets | 60 + 45 | R0 | **MISSING** until L1-45 |
| **L3-WT-1** | Worktree isolation **product-documented** and dogfoodable | product choice | R0 at least one flow | **PARTIAL** (opt-in OK if honesty clear) |
| **L3-WT-2** | If worktree remains opt-in, **KNOWN_LIMITS + README** say bare `dsb` is single-session | honesty | Doc sync | **PARTIAL** |
| **L3-ID-1** | Marketing / PRD does not claim “fleet OS complete” without L3-50/60 R0 | PRD | Doc sync | **MISSING** at prior 4.0 cut |

---

## 7. Master checklist — fusion integrity (the gap that killed 3.0/4.0)

| ID | Requirement | Evidence | Baseline |
|----|-------------|----------|----------|
| **F1** | Path A binary **links or embeds** heart implementations (shared crate **or** ported Grok-local authority). Zero “API exists only in dsb-*” for claimed hearts. | `Cargo.toml` dep graph of pager-bin / shell + call-sites | **MISSING** |
| **F2** | No production heart is **dead-wired** (computed params never applied on default toolset) | Code review rule + e2e | **MISSING** (snippet_safe dead) |
| **F3** | Thin Path B may keep hearts for MVP/tools, but release notes **must not** say Path A fusion if only Path B is green | Honesty | **FAILED** historically |
| **F4** | CUT evidence cites **Path A R0** commands and outputs, not only `cargo test -p dsb-* path_a` | CUT template | **FAILED** for 3.0/4.0 cuts |
| **F5** | Adversarial re-check (second model or human) of this checklist before major tag | Review log | **required for 5.0.0** |

---

## 8. Go / no-go formula

### 8.1 Owner-bar major (**`5.0.0`** / plan `owner-bar-5x`)

**Normative machine list:** [OWNER_BAR_P0_LEDGER.md](./OWNER_BAR_P0_LEDGER.md) (not free-form “§3–§7 vibes”).

**GO only if ALL of:**

1. Every ledger P0 row is **PASS** with **R0A** + freshness on cut SHA + binary manifest.  
2. **F1–F5** PASS.  
3. `./scripts/test-owner-bar.sh` exit 0.  
4. **Live DeepSeek** Path A R0A on cut day (not SKIP). Hermetic R0A green throughout.  
5. Install smoke: dual CLI + agent on primary platform.  
6. Docs singular.  
7. Dual independent adversarial reviews same SHA+manifest.  
8. Full SemVer tag **only after** 1–7.

**NO-GO if any of:**

- Heart proven only under `dsb run` / unit tests / `path_a_*` name  
- Default path skips safety params or bricks edits (no liveness)  
- L3 “done” from guides alone  
- Any SKIP/BLOCKED/N/A in cut manifest  
- Residual in KNOWN_LIMITS contradicts cut claim without demoting the claim  

### 8.2 What current `4.x` is allowed to claim (honesty)

Until this file is green:

| May claim | Must not claim |
|-----------|----------------|
| Real Grok-derived DeepSeek TUI | Full Reasonix control of default loop |
| DeepSeek routing / base_url seed | Spec 45 primary edit on default path |
| Dual CLI / packaging progress | Heart fusion complete (3.0 narrative) |
| L3 machinery present / partial defaults | L3 product identity complete (4.0 narrative) |
| Thin-path heart reference implementations | “Path A APIs shipped” as user-facing fusion |

---

## 9. Explicit non-goals (do not block owner bar)

From [NON_GOALS.md](./NON_GOALS.md) + HARNESS:

| Non-goal | Notes |
|----------|--------|
| Gajae multi-stage planning harness | Out |
| Multi-vendor first-class (Claude/GPT core) | DeepSeek-first |
| Desktop / VS Code MVP | CLI/TUI first |
| YOLO as default | Forbidden |
| Full Grok hard-fork branding rewrite | Adapt vendor; product chrome OK |
| Process-police CI for PR title fashion | Product CI only for real build/test truth |
| Perfect byte-identity with thin `dsb-context` **if** Path A has its own golden that meets Spec 10 | Equivalence of **contract**, not necessarily same function name |

Anything else that would delay Path A fusion for “nice polish” is **P1**, not a fake P0.

---

## 10. Verification command template (cut day)

Agents **must** attach outputs (or env-BLOCKED with reason) for:

```bash
# Identity
./scripts/check-semver.sh
cargo metadata --no-deps -q | head   # version truth
# Dual CLI (installed or target/)
deepseek-build --version; dsb --version

# Path A process composition (document how agent is resolved)
# e.g. which deepseek-build-agent; strings / rg on binary optional

# Hearts — REPLACE with real Path A harnesses as they land
# FORBIDDEN as sole evidence:
#   cargo test -p dsb-agent path_a
#   cargo test -p dsb-context path_a
# REQUIRED examples (evolve into scripts/test-owner-bar.sh):
#   ./scripts/test-path-a-snippet-e2e.sh
#   ./scripts/test-path-a-prefix-golden.sh
#   ./scripts/test-path-a-repair-e2e.sh
#   ./scripts/test-path-a-routing-e2e.sh
#   ./scripts/test-l3-smoke.sh --live   # when key present

# Dead-wiring scan (must stay clean)
rg -n 'snippet_safe' third_party/grok-build/crates/codegen/xai-grok-shell/src/tools/config.rs
rg -n 'effective != .*Standard' third_party/grok-build/crates/codegen/xai-grok-shell/src/agent -g '*.rs'
# Prove Standard path actually applies tool_configs OR product default is not Standard

# Call-site scan for claimed hearts
rg -n 'assemble_path_a_context|prepare_path_a_tool_call|path_a_default_router' \
  third_party/grok-build crates --glob '*.rs'
# Production hits outside tests required OR honesty demotion
```

Ship a single **`./scripts/test-owner-bar.sh`** (or equivalent) before `5.0.0` that fails closed on R2-only hearts.

---

## 11. Mapping: checklist → train stories (normative)

**Do not use W8-last.** Gate substrate is **G001 first**. Full map: [OWNER_BAR_5X_GOALS.md](./OWNER_BAR_5X_GOALS.md) · [WAVE_5x_PR_DAG.md](./WAVE_5x_PR_DAG.md).

| Story | Primary IDs | Notes |
|-------|-------------|--------|
| **G001** | OB-2/4, F3 honesty, gate RED | Scripts + STATUS.tsv + demotion |
| **G002** | R0A rig | Public entry + scripted server |
| **G003** | L1-45-2 | **Mint before flip** |
| **G004** | L1-45-0/1/3–6, F2 | Live safety + liveness |
| **G005** | L1-45-7/8 | write/bash |
| **G006** | L1-90-*, S8 | perms matrix |
| **G007** | L2-15-* | repair dispatch |
| **G008** | L2-10-*, L1-70, L1-100 | prefix after schema stable |
| **G009** | L2-20-*, L1-30, S4 | routing/effort |
| **G010** | L3-* | under hearts |
| **G011** | S1–S7, OB-1 | install |
| **G012** | F1–F5, OB-3, tag | dual review + cut |

---

## 12. Sign-off block (fill only when green)

```text
Owner-bar cut: __________ (e.g. 5.0.0)
Date: __________
Path A evidence bundle: docs/product/evidence/OWNER_BAR_<ver>_<date>.md
test-owner-bar.sh: PASS / FAIL
Adversarial review 1: PASS / FAIL (model/tool)
Adversarial review 2: PASS / FAIL
KNOWN_LIMITS residuals remaining: (list or none)
Owner: __________
```

Until this block is filled with **PASS**, do **not** say the product is complete.

---

## 13. Change control

- Changing P0 scope requires **owner approval** + update to this file in the **same PR** as any PRD that claims a new major.  
- Agents must not invent a softer bar in ultragoal goals to make a train “finish.”  
- If this file and a PRD disagree: **this file wins for “is the product done?”**; PRD wins only for scoped non-goals explicitly listed in §9.
