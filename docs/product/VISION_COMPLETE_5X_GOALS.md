# Vision-complete train — **`5.x`** (after owner-bar `5.0.0`)

| Field | Value |
|-------|--------|
| **Plan id** | `vision-complete-5x` |
| **SemVer band** | **`5.2.0` – `5.Y.0`** (vision close-out). Floor = **`5.1.0`** aligned on **`main`**, GitHub Release, and npm |
| **Depends on** | **`v5.0.0` owner-bar CUT** (done) · **`5.0.1`+ product version fix** (npm) · **`5.1.0` theme/product chrome** (shipped: `main` + Release + npm) |
| **North star** | [VISION.md](./VISION.md) + [HARNESS_PHILOSOPHY.md](../architecture/HARNESS_PHILOSOPHY.md) |
| **PR planning** | [ULTRAGOAL_PR_PLANNING.md](./ULTRAGOAL_PR_PLANNING.md) · DAG [WAVE_5x_VISION_PR_DAG.md](./WAVE_5x_VISION_PR_DAG.md) |
| **Cold start** | [ULTRAGOAL_PROMPT_COLD_START_VISION_5X.md](./ULTRAGOAL_PROMPT_COLD_START_VISION_5X.md) |
| **Child runtime** | **Grok only** (parent = Grok Build) |

**Do not** plan releases as `5.0.1` or `5.1.0` — those targets are **already used / shipped**. Next **feature** minor for Deep Code snippet_id is **`5.2.0`**.

---

## 0. Honesty first

### What already shipped (do not re-target)

| Version | Meaning | Status (2026-08-07 check) |
|---------|---------|---------------------------|
| **`5.0.0` / `v5.0.0`** | Owner-bar P0 Path A complete | **Tagged** · npm · gate green |
| **`5.0.1`** | Product version/update alignment (npm line) | **On npm** `latest` as of check; may predate some main fixes |
| **`5.1.0`** | Product chrome (e.g. DeepSeek Night v2 default, tab icon, related) | **Aligned:** `main` = **`5.1.0`** · GitHub Release **`v5.1.0`** (darwin-arm64 asset) · npm **`@innocarpe/deepseek-build@5.1.0`** / **`latest` = `5.1.0`** |

### What `5.0.0` owner-bar closed (unchanged)

**Owner-bar P0 on Path A** — public CLI → agent, hearts + L3 machinery not dead-wired, dual CLI install, gate green, tag `v5.0.0`.

That is a **real product cut**. It is **not** “VISION north star 100%.”

### What the original ideal still demands

From [VISION.md](./VISION.md):

> feels as **fast as Grok Build**, as **cheap on long sessions as Reasonix**, and as **correctly tuned to DeepSeek V4 as Deep Code**.

| Pillar | Ideal end-state | Reality after 5.0.0–5.1.0 |
|--------|-----------------|---------------------------|
| **L1 Deep Code** | Spec 45 **snippet_id** session table; edit/write/bash bypass laws full; skills thrash-free | **file_version (sha256) equivalent** + snippet_safe on Path A; skills index OK; **snippet_id still residual** |
| **L2 Reasonix** | Grok assembly path Spec 10; repair; Flash/Pro + **effort on wire**; cache hits visible | Stamps + repair call site; **effort wire often null**; shell ≠ always `assemble_path_a_context` |
| **L3 Grok** | Parallel / bg / subagent / worktree dogfood; worker cache on real workers | Machinery + units + stamps; worktree opt-in; live extended residual |
| **Product identity** | Dual CLI, theme, install, version UX | Dual CLI + **5.1.0 theme v2 on main**; version fix in train; **keep owner-bar green** |

**This train closes that gap inside `5.x.y` with many small PRs — without re-burning 5.0.1 / 5.1.0.**

---

## 1. SemVer policy (rebased)

| Ship | When | Notes |
|------|------|--------|
| ~~`5.0.1`~~ | **Used** | npm; version/update line — **do not re-plan as future** |
| ~~`5.1.0`~~ | **Used / shipped** | `main` + GitHub Release + npm **`5.1.0`** — **do not re-plan as future** |
| **`5.1.x` patch** | Only for **deploy fix** of 5.1.0 (broken release, missing asset, hot banner) | Not for Spec 45 / L3 |
| **`5.2.0`** | Spec 45 **snippet_id** Path A (Deep Code primary contract) | First **vision** minor |
| **`5.3.0`** | Spec 10 assembly-in-Grok + effort-on-wire (+ cache visibility) | Reasonix minor |
| **`5.4.0`** | L3 Path A R0A (parallel / bg / subagent / worker / worktree dogfood) | Grok throughput minor |
| **`5.Y.0`** | Vision-complete freeze (dual review + CUT) | Prefer **`5.5.0`** if Y free; never below 5.2 |

**Rules**

1. Always full **`MAJOR.MINOR.PATCH`**.  
2. Read root **`Cargo.toml` / `package.json` on `main`** before every release unit — **never invent a version already on npm or `main`**.  
3. One SemVer bump unit per release; feature PRs prefer unversioned until cut PR.  
4. Do **not** re-tag `5.0.0` / `5.1.0` as “now vision complete.”

### Floor check (agents — run every session)

```bash
git show origin/main:Cargo.toml | rg 'version = "'
npm view @innocarpe/deepseek-build version
gh release list --limit 5
```

Next free **feature** minor = max(on-disk minor, npm minor) + 1 for the **next pillar cut**, or a **patch** only for release repairs.

---

## 2. Success definition (vision-complete)

Ship is **vision-complete** only when **all** of the following are true on **Path A** (public `dsb` / `deepseek-build` → product agent), with R0A evidence:

### V1 — Deep Code (L1) complete → target minor **`5.2.0`**

| ID | Requirement | Evidence |
|----|-------------|---------|
| **V1-45-1** | Session snippet table with **`snippet_id`** (not only file_version) on Path A read | Wire + unit + multi-edit R0A |
| **V1-45-2** | Edit **requires** valid `snippet_id` (or documented dual-accept window with deadline) | Negative goldens |
| **V1-45-3** | Write create-only; overwrite uses same safety class as edit | R0A |
| **V1-45-4** | Bash mutation invalidates snippets for touched paths | R0A |
| **V1-90** | Perms matrix still green under new edit surface | Heart regression |
| **V1-70** | Skills index stable; body load thrash-free under multi-turn | Wire + unit |

### V2 — Reasonix (L2) complete → target minor **`5.3.0`**

| ID | Requirement | Evidence |
|----|-------------|---------|
| **V2-10-1** | **Grok message assembly** uses Spec 10 stable prefix layout | Golden on assembly / wire |
| **V2-10-2** | Compaction / resume does not thrash stable prefix | Two-turn + resume goldens |
| **V2-15** | One-pass repair on every Grok tool-call dispatch path | R0A bad-args |
| **V2-20** | Flash default + Pro one-shot/sticky per Spec 20 on TUI | Wire |
| **V2-30** | **`reasoning_effort` / thinking knobs on DeepSeek wire** when set | Wire assert |
| **V2-cache** | User-visible or loggable cache-hit signal | R0A or doctor |

### V3 — Grok throughput (L3) complete → target minor **`5.4.0`**

| ID | Requirement | Evidence |
|----|-------------|---------|
| **V3-50-1** | Multi-tool RO parallel + mutate serial on Path A | Wire / log |
| **V3-50-2** | Background shell + collect-by-id dogfood Path A | R0A |
| **V3-60-1** | Explore + implement subagent dogfood Path A | R0A |
| **V3-60-2** | Worker reuses parent stable prefix template (hash) | R0A |
| **V3-60-3** | Worker mutation invalidates parent snippets | R0A |
| **V3-WT** | Worktree dogfood + bare `dsb` honesty | R0A + docs |

### V4 — Product finish → cut **`5.Y.0`** (prefer **`5.5.0`**)

| ID | Requirement | Evidence |
|----|-------------|---------|
| **V4-ver** | npm prebuilt agent shows **product SemVer**; no false update banner | Install smoke on **current** latest |
| **V4-plat** | Prebuilt platforms claimed in README actually ship | Release assets |
| **V4-docs** | User-guide matches behavior; KNOWN_LIMITS only true residuals | Doc review |
| **V4-owner-bar** | `./scripts/test-owner-bar.sh` still exit 0 | Gate |
| **V4-cut** | Dual adversarial review + CHANGELOG + tag **`v5.Y.0`** | CUT doc |

---

## 3. Gap inventory (actionable — same substance, new versions)

### Deep Code (L1) → **5.2.0**

| Gap | Severity | Close with |
|-----|----------|------------|
| No real `snippet_id` table (file_version only) | **Blocker** | Spec 45 Path A |
| Edit not snippet_id-required | High | VC004-class PR |
| Write/bash laws under snippet model | Med–High | VC005-class |
| Skills body thrash polish | Low–Med | minor after 5.2 |

### Reasonix (L2) → **5.3.0**

| Gap | Severity | Close with |
|-----|----------|------------|
| Grok assembly ≠ full Spec 10 library path | High | VC007 |
| `reasoning_effort` missing on wire | High | VC008 |
| Cache hit invisible | Med | VC009 |
| Compaction residual | Med | goldens |

### Grok (L3) → **5.4.0**

| Gap | Severity | Close with |
|-----|----------|------------|
| Live/hermetic multi-tool parallel R0A weak | High | VC010 |
| Bg collect-by-id dogfood weak | High | VC010 |
| Subagent + worker cache R0A weak | High | VC011 |
| Worktree dogfood | Med | VC012 |
| Wall-clock vs Grok unmeasured | Med | optional harness |

### Deploy floor (not vision pillars)

| Gap | Severity | Close with |
|-----|----------|------------|
| ~~`5.1.0` Release/npm lag vs `main`~~ | Ops | **Closed** — main / Release / npm aligned at **`5.1.0`** (VC001c complete) |
| Hotfix on 5.1 line | Ops | **`5.1.1`+ patch only** (only if ship breaks; not required now) |

---

## 4. Story board (rebased)

| Story | Intent | Target SemVer | Depends | Status |
|-------|--------|---------------|---------|--------|
| **VC001** | Product SemVer / update fix ship | ~~5.0.1~~ | — | **DONE** (npm 5.0.1; fix PR #117) |
| **VC001b** | Theme / chrome **5.1.0** | ~~5.1.0~~ | — | **DONE** (on `main`; e.g. theme v2) |
| **VC001c** | Finish **5.1.0** GitHub Release + npm if lagging | **5.1.0** (same) or **5.1.1** patch | VC001b | **COMPLETE / SKIP** — Release + npm **`5.1.0`/`latest`** aligned ([evidence](./evidence/VC001C_5_1_0_SHIP_2026-08-07.md) §8) |
| **VC002** | Spec 45 ADR + SnippetStore design | none | after 5.1.0 stable (**floor met**) | may proceed / pending |
| **VC003** | Path A `read_file` mints `snippet_id` | none | VC002 | pending |
| **VC004** | Path A edit requires `snippet_id` | part of **5.2.0** | VC003 | pending |
| **VC005** | Write/bash snippet invalidation | part of **5.2.0** | VC004 | pending |
| **VC006** | Heart regression under snippet_id | **5.2.0** cut unit | VC005 | pending |
| **VC007** | Spec 10 assembly on Grok Path A turns | part of **5.3.0** | VC006 prefer | pending |
| **VC008** | `reasoning_effort` on DeepSeek wire | part of **5.3.0** | VC007 prefer | pending |
| **VC009** | Cache-hit visibility | **5.3.0** cut unit | VC008 soft | pending |
| **VC010** | L3 multi-tool + bg Path A R0A | part of **5.4.0** | VC006 | pending |
| **VC011** | Subagent + worker cache Path A R0A | part of **5.4.0** | VC010 | pending |
| **VC012** | Worktree dogfood + docs honesty | part of **5.4.0** | VC011 | pending |
| **VC013** | Live extended matrix when key present | **5.4.0** cut unit | VC012 | pending |
| **VC014** | User-guide + KNOWN_LIMITS vision pass | none | VC013 | pending |
| **VC015** | Dual review + CUT **`v5.Y.0`** (prefer **5.5.0**) | **5.Y.0** | VC014 | pending |

### Parallel tracks (after 5.1.0 floor stable)

```text
5.1.0 shipped (main + Release + npm) ──► do not plan 5.0.1 / 5.1.0 as future goals
        │
        ▼
   VC002–VC006  →  tag/ship 5.2.0   (Deep Code)
        │
        ├─► VC007–VC009 → 5.3.0   (Reasonix)
        │
        └─► VC010–VC013 → 5.4.0   (Grok L3; after 5.2 hearts green)
                │
                ▼
           VC014–VC015 → 5.5.0 (or free 5.Y.0) vision freeze
```

Do **not** parallel-edit the same Grok tool files across tracks without stacks.

---

## 5. Overnight / continuous-PR rules

1. **PR unit plan** before coding ([ULTRAGOAL_PR_PLANNING.md](./ULTRAGOAL_PR_PLANNING.md)).  
2. **Atomic Conventional Commits**; GitHub **merge commit**.  
3. **English** PR bodies + labels.  
4. **Path A evidence** for behavior claims.  
5. **Heart + owner-bar stay green:**  
   `./scripts/test-heart-regression.sh` · `./scripts/test-owner-bar.sh`  
6. **Disk:** clean vendor `target/` after agent builds.  
7. **Version floor check** every release unit (§1).  
8. **No false complete** — vision ledger rows FAIL until R0A.

Optional: `docs/product/evidence/VISION_STATUS.tsv`.

---

## 6. Explicit non-goals

- Replacing Grok base with greenfield agent  
- Multi-vendor core  
- Gajae multi-stage planning harness  
- Claiming vision-complete from docs alone  
- Re-tagging `5.0.0` / `5.1.0` as vision complete  
- Planning **`5.0.1` or `5.1.0` as future feature targets**  

---

## 7. First actions (rebased “now”)

1. **5.1.0 floor is closed** (main + GitHub Release + npm **`5.1.0`/`latest`**). **VC001c COMPLETE / SKIP** — do not re-run packaging catch-up unless ship breaks.
2. **VC002** Spec 45 design — start **5.2.0** track (floor met).
3. Do **not** open a PR titled “ship 5.1.0 theme” or re-plan **`5.1.0`** as a future feature target.

Board owner: continuous session until VC015 or hard block.
