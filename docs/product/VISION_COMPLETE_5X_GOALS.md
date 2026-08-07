# Vision-complete train — **`5.x`** (after owner-bar `5.0.0`)

| Field | Value |
|-------|--------|
| **Plan id** | `vision-complete-5x` |
| **SemVer band** | **`5.0.1` – `5.y.z`** (no new major until a second product identity jump) |
| **Depends on** | **`v5.0.0` / owner-bar-5x CUT** (done) |
| **North star** | [VISION.md](./VISION.md) + [HARNESS_PHILOSOPHY.md](../architecture/HARNESS_PHILOSOPHY.md) |
| **PR planning** | [ULTRAGOAL_PR_PLANNING.md](./ULTRAGOAL_PR_PLANNING.md) · DAG [WAVE_5x_VISION_PR_DAG.md](./WAVE_5x_VISION_PR_DAG.md) |
| **Child runtime** | **Grok only** (parent = Grok Build) |

---

## 0. Honesty first

### What `5.0.0` closed

**Owner-bar P0 on Path A** — public CLI → agent, hearts + L3 machinery not dead-wired, dual CLI install, gate green, tag `v5.0.0`.

That is a **real product cut**. It is **not** “VISION north star 100%.”

### What the original ideal still demands

From [VISION.md](./VISION.md):

> feels as **fast as Grok Build**, as **cheap on long sessions as Reasonix**, and as **correctly tuned to DeepSeek V4 as Deep Code**.

| Pillar | Ideal end-state | `5.0.0` reality |
|--------|-----------------|-----------------|
| **L1 Deep Code** | Spec 45 **snippet_id** session table; edit/write/bash bypass laws full; skills thrash-free | **file_version (sha256) equivalent** + snippet_safe default on Path A; skills index OK; residual vs full snippet_id |
| **L2 Reasonix** | Grok assembly path **byte-stable** Spec 10 prefix; repair always; Flash/Pro + **effort on wire**; cache hits visible | Library + launch stamps + repair call site; wire system stable; **effort field often null**; shell prompt ≠ always `assemble_path_a_context` |
| **L3 Grok** | Parallel / bg / subagent / worktree **dogfoodable at Grok feel**, worker cache law on **real** workers | Machinery + units + stamps; worktree **opt-in**; live extended residual; not “always as fast as Grok” proven |
| **Product identity** | Dual CLI, theme, install, version UX = DeepSeek Build | Dual CLI + theme OK; version/update fix landed (**needs 5.0.1 agent prebuilt** for npm users) |

**This train exists to close that gap inside `5.x.y` with many small PRs — not to claim “already done.”**

---

## 1. Success definition (vision-complete)

Ship is **vision-complete** only when **all** of the following are true on **Path A** (public `dsb` / `deepseek-build` → product agent), with R0A evidence:

### V1 — Deep Code (L1) complete

| ID | Requirement | Evidence |
|----|-------------|---------|
| **V1-45-1** | Session snippet table with **`snippet_id`** (not only file_version) on Path A read | Wire + unit + multi-edit R0A |
| **V1-45-2** | Edit **requires** valid `snippet_id` (or documented dual-accept window with deadline) | Negative goldens |
| **V1-45-3** | Write create-only; overwrite uses same safety class as edit | R0A |
| **V1-45-4** | Bash mutation invalidates snippets for touched paths | R0A (G005 style extended) |
| **V1-90** | Perms matrix still green under new edit surface | Heart regression |
| **V1-70** | Skills index stable; body load thrash-free under multi-turn | Wire + unit |

### V2 — Reasonix (L2) complete

| ID | Requirement | Evidence |
|----|-------------|---------|
| **V2-10-1** | **Grok message assembly** (not only library stamp) uses Spec 10 stable prefix layout | Instrument or golden on assembly |
| **V2-10-2** | Compaction / resume does not thrash stable prefix | Two-turn + resume goldens |
| **V2-15** | One-pass repair on every Grok tool-call dispatch path | R0A bad-args |
| **V2-20** | Flash default + Pro one-shot/sticky per Spec 20 on TUI | Wire |
| **V2-30** | **`reasoning_effort` / thinking knobs appear on DeepSeek wire** when set | Wire assert |
| **V2-cache** | User-visible or loggable cache-hit signal (session or turn) | R0A or doctor |

### V3 — Grok throughput (L3) complete under hearts

| ID | Requirement | Evidence |
|----|-------------|---------|
| **V3-50-1** | Multi-tool RO parallel + mutate serial on **live or hermetic multi-tool** Path A | Wire / log |
| **V3-50-2** | Background shell + collect-by-id dogfood Path A | R0A |
| **V3-60-1** | Explore + implement subagent dogfood Path A | R0A |
| **V3-60-2** | Worker reuses parent stable prefix template (hash) | R0A |
| **V3-60-3** | Worker mutation invalidates parent snippets | R0A |
| **V3-WT** | Worktree flow documented + one full dogfood; bare `dsb` honesty remains | R0A + docs |

### V4 — Product finish

| ID | Requirement | Evidence |
|----|-------------|---------|
| **V4-ver** | npm prebuilt agent shows **product SemVer**; no false update banner | Install smoke |
| **V4-plat** | Prebuilt platforms claimed in README actually ship (expand beyond darwin-arm64+linux-x64 if claimed) | Release assets |
| **V4-docs** | User-guide matches behavior; KNOWN_LIMITS only true residuals | Doc review |
| **V4-cut** | Dual adversarial review + CHANGELOG + tag **`v5.Y.0`** vision-complete minor | CUT doc |

**SemVer policy**

| Ship | When |
|------|------|
| **`5.0.1`** | Version/update fix + agent prebuilt in npm (patch) |
| **`5.1.0`** | Spec 45 snippet_id Path A (minor — user-visible edit contract) |
| **`5.2.0`** | Spec 10 assembly-in-Grok + effort-on-wire (minor) |
| **`5.3.0`** | L3 live dogfood under hearts (minor) |
| **`5.Y.0`** | Vision-complete cut (final minor of this train) |

Exact Y chosen at freeze; do **not** burn majors for residual close-out.

---

## 2. Gap inventory (actionable)

### Grok (L3) — keep base, deepen product use

| Gap | Severity | Close with |
|-----|----------|------------|
| Live multi-tool parallel R0A weak | High | Hermetic multi-tool scenario + live optional |
| Bg shell collect-by-id not product-dogfooded enough | High | Scripted + live scenario |
| Subagent spawn R0A weak | High | Path A headless spawn |
| Worker cache law only unit/stamp | High | Instrument real worker prefix |
| Worktree opt-in only | Product choice | Keep; dogfood one flow |
| “As fast as Grok” unmeasured | Med | Simple wall-clock multi-step harness (optional) |

### Reasonix (L2)

| Gap | Severity | Close with |
|-----|----------|------------|
| Grok system assembly ≠ full Spec 10 library path | High | Wire assembly to `assemble_path_a_context` or document demotion + force equivalent |
| `reasoning_effort` missing on chat_completions wire | High | Grok DeepSeek backend serialize field |
| Cache hit invisible | Med | Status row / log field |
| Compaction residual | Med | Goldens on compact path |

### Deep Code (L1)

| Gap | Severity | Close with |
|-----|----------|------------|
| No real `snippet_id` table (file_version only) | **Blocker for vision** | Spec 45 implementation on Path A tools |
| Write overwrite safety polish | Med | Align write with Spec 45 § write law |
| Skills body thrash polish | Low–Med | Load-on-demand metrics |
| Free-form edit still present as escape? | Med | Fail-close audit |

### Install / identity

| Gap | Severity | Close with |
|-----|----------|------------|
| npm `5.0.0` agent may predate version fix | High | **`5.0.1` release** |
| Multi-platform prebuilt limited | Med | Expand only if claimed |
| TUI “Beta” / Grok changelog bleed | Low | Branding PRs |

---

## 3. Story board (execution order)

Do **not** stop for applause. One PR unit at a time; stack only when sequential.

| Story | Intent | Target SemVer | Depends |
|-------|--------|---------------|---------|
| **VC001** | Ship **`5.0.1`**: version/update fix in prebuilt agent + npm | **5.0.1** | none |
| **VC002** | Spec 45 design ADR if needed + session SnippetStore on Path A | — | VC001 prefer |
| **VC003** | Path A `read_file` mints `snippet_id` (+ keep file_version dual) | part of 5.1 | VC002 |
| **VC004** | Path A `search_replace` requires snippet_id (migration window) | **5.1.0** | VC003 |
| **VC005** | Write/bash laws + invalidate under snippet_id | 5.1.x | VC004 |
| **VC006** | Heart regression + owner-bar still green | 5.1.x | VC005 |
| **VC007** | Grok assembly uses Spec 10 product prefix (or demote honesty + max equivalence) | part of 5.2 | VC001 |
| **VC008** | `reasoning_effort` on DeepSeek wire | **5.2.0** | VC007 prefer |
| **VC009** | Cache-hit visibility (status or log) | 5.2.x | VC008 |
| **VC010** | L3 multi-tool + bg hermetic Path A R0A | part of 5.3 | VC006 |
| **VC011** | Subagent + worker cache R0A | part of 5.3 | VC010 |
| **VC012** | Worktree dogfood + docs honesty | 5.3.x | VC011 |
| **VC013** | Live extended matrix (key present) | 5.3.x | VC012 |
| **VC014** | User-guide + KNOWN_LIMITS rewrite (only true residuals) | — | VC013 |
| **VC015** | Dual adversarial review + CUT **`v5.Y.0` vision-complete** | **5.Y.0** | VC014 |

Parallelism (after VC001):

- **Track A (Deep Code):** VC002→VC006  
- **Track B (Reasonix):** VC007→VC009 (after VC001; soft-dep VC006 for heart re-prove)  
- **Track C (Grok L3):** VC010→VC013 (after VC006)

Do **not** parallel-edit same Grok tool files across tracks without stacks.

---

## 4. Overnight / continuous-PR rules

1. **Always PR unit plan** before coding a story ([ULTRAGOAL_PR_PLANNING.md](./ULTRAGOAL_PR_PLANNING.md)).  
2. **Atomic Conventional Commits**; **merge commit** on GitHub (no squash).  
3. **English** PR bodies + labels (`github-pr` skill).  
4. **Path A evidence** for behavior claims; library-only insufficient.  
5. **Heart regression** after each L1/L2/L3 behavior PR:  
   `./scripts/test-heart-regression.sh` (+ `--with-e2e` when agent present).  
6. **Disk:** do not leave full vendor `target/` overnight; clean after agent builds.  
7. **SemVer:** only release units bump version; full `MAJOR.MINOR.PATCH`.  
8. **No false complete:** if blocked, leave FAIL/NOT_RUN in a vision ledger row — never SKIP as pass.

### Vision ledger (optional TSV)

`docs/product/evidence/VISION_STATUS.tsv` — same spirit as owner-bar STATUS; fill as stories land. Gate script optional later (`scripts/test-vision-complete.sh`).

---

## 5. Explicit non-goals (still)

- Replacing Grok base with greenfield agent  
- Multi-vendor core  
- Gajae multi-stage planning harness  
- Claiming vision-complete from docs alone  
- Re-tagging `5.0.0` as “now really vision complete”  
- Forward calendar wait / paper dogfood instead of current evidence  

---

## 6. First actions (now)

1. Land this plan on `main` (docs PR).  
2. **VC001 `5.0.1`** — rebuild agent with version fix, package release assets, npm publish.  
3. Start **VC002/VC003** Spec 45 Path A (largest Deep Code gap).  

Board owner: continuous agent session until VC015 or hard block.
