# VC014 — user-guide + KNOWN_LIMITS vision pass

| Field | Value |
|-------|--------|
| **Story** | **VC014** — `docs(product): user-guide + KNOWN_LIMITS vision pass` (vision **V4-docs**) |
| **Plan** | `vision-complete-5x` |
| **Date** | 2026-08-08 |
| **Status** | **PLAN** — plan-first gate only; docs not yet updated |
| **SemVer** | **none** (docs-only; do **not** bump product version) |
| **Depends on** | **VC013** L3 **5.4.0** cut (open PR **#145** `vc013-5-4-cut`) |
| **Board** | [`VISION_COMPLETE_5X_GOALS.md`](../VISION_COMPLETE_5X_GOALS.md) · DAG [`WAVE_5x_VISION_PR_DAG.md`](../WAVE_5x_VISION_PR_DAG.md) |
| **Normative** | V4-docs · dual CLI ADR 0006 · Spec 45 Path A · Spec 10/effort/cache honesty · L3 Path A R0A (VC010–VC013) · SSOT · `KNOWN_LIMITS` residual honesty |
| **Prior** | VC013 READY cut evidence · VC006 Spec 45 **5.3.0** stack cut · VC007–VC009 Reasonix Path A · VC010–VC012 L3 dogfood |

**This file is the mandatory ultragoal PR unit plan for VC014 plus (later) READY evidence.**  
It does **not** claim vision freeze **`5.5.0`**, npm publish, GitHub Release, or Path A residual closure without fresh proof. It does **not** edit VC013 or PR **#145**.

---

## 0. Floor and dependency facts

### 0.1 Live floor (story open; 2026-08-08)

| Probe | Live result |
|-------|-------------|
| This worktree branch | `vc014-vision-docs` @ `/Users/WooseongKim/Projects/deepseek-build/vc014-vision-docs` |
| Stack base for PR | **`vc013-5-4-cut`** (open PR **#145**); **not** `origin/main` until after #145 merges |
| Base tip at plan | **`62077f4`** (`docs: clean VC013 diff whitespace` — tip of VC013 branch) |
| Working tree product version (stack tip) | **`5.4.0`** (on-branch cut from VC013; **unmerged**) |
| `git show origin/main:Cargo.toml` version | **`5.2.2`** |
| `package.json` on `origin/main` | **`5.2.2`** (expected) |
| `npm view @innocarpe/deepseek-build version` | **`5.2.2`** |
| `gh release list` Latest | **`v5.2.2`** |
| Board DAG PR unit | VC014 — `docs(product): user-guide + KNOWN_LIMITS vision pass` |
| Stacked open train (sample) | … · #145 VC013 · **this** VC014 |

### 0.2 Floor interpretation (fail-close)

- **Live shipped floor is `origin/main` / npm / GitHub Latest = `5.2.2`.**
- **Stack product version is already `5.4.0`** (VC013 L3 cut). Do **not** re-cut **`5.4.0`**. Do **not** bump to **`5.5.0`**.
- **Open exactly one stacked PR** with base **`vc013-5-4-cut`** and body **`Depends on #145`**. **Do not merge.** **Do not publish** npm / tag / Release.
- Scope is **docs only**: user-facing guides + `KNOWN_LIMITS` + this evidence / independent review. **No code feature**, no CLI behavior change, no SemVer bump.
- Claims must match **actually shipped / on-branch Path A evidence** (VC006–VC013 stack), not aspirational 4.0.0 / pre-vision wording.
- **Do not close residuals** (especially **V3-60-3** parent snippet expire, interactive TTY worktree **create**) without **fresh Path A proof** — restate only.
- **Do not edit** VC013 evidence files or PR **#145**.
- **No Claude/Codex children** (Grok-only).

---

## 1. Why this PR (one sentence)

Refresh user-facing docs and `KNOWN_LIMITS` so they describe the real dual-CLI Path A product on the **5.4.0** vision stack (Spec 45 snippet safety, L2 assembly/effort/cache with honest scope, L3 parallel/bg/subagent/worktree dogfood + opt-in/headless limits), and keep only **evidenced residuals** — without claiming freeze, publish, or residual closure.

---

## 2. Acceptance map (V4-docs → evidence)

| ID | Requirement | How this story meets it |
|----|-------------|-------------------------|
| **V4-docs** | User-guide matches behavior; `KNOWN_LIMITS` only true residuals | Update guides **10–14** (+ README / surface as needed for dual CLI + L2 honesty); rewrite residual table |
| **Dual CLI** | `deepseek-build` primary + `dsb` alias documented consistently | User-guide README + L3 guides; no single-name-only claims |
| **Spec 45** | Snippet safety / `snippet_id` Path A (not file_version-only story) | `10-tools.md` + KNOWN_LIMITS honesty pointing at VC003–VC006 stack |
| **L2 honesty** | Assembly / effort / cache claims with **scope** | Surface + KNOWN_LIMITS: Spec 10 assembly, `reasoning_effort` wire, cache stamp/chip — cite VC007–VC009; no over-claim |
| **L3 honesty** | Parallel / bg / subagent / worktree dogfood + opt-in/headless | Guides **11–14** + VC010–VC013 pointers; bare `dsb` single-session; headless no-create |
| **Residuals only** | No stale 4.0.0 / “next 5.2.0+” wording as live residual | `KNOWN_LIMITS` residual list = evidenced carry only |
| **Floor honesty** | 5.4.0 on unmerged stack; live main/npm/GitHub = **5.2.2** | Evidence §0 + KNOWN_LIMITS ops residual |
| **Gates** | SemVer shape, path-linkage, owner-bar, heart as practical | Re-run; **no version change**; restore TSV side-effects |
| **Non-claims** | No freeze **5.5.0**; no residual close without Path A proof | Explicit non-claim section + independent review |

### 2.1 Residual allow-list (must remain honest)

| Residual | Source |
|----------|--------|
| **V3-60-3** Path A parent snippet expiry after worker mutation | VC011 / VC013 carry |
| Interactive TTY worktree **create** sole green | VC012 / VC013 carry |
| Non-darwin packaging / asset boundary (`darwin-arm64` only) | Install / ADR 0009 |
| Human-gated npm / GitHub publish | ADR 0007; release lane |
| On-branch **5.4.0** unmerged; live main/npm/GitHub Latest still **5.2.2** | Floor re-check |

### 2.2 Explicit non-claims

- Not vision freeze / **`5.5.0`** / VC015
- Not npm publish / GitHub Release for **5.4.0**
- Not closing **V3-60-3** or interactive worktree create without fresh Path A R0A
- Not inventing code features or changing CLI behavior
- Not editing VC013 / PR **#145** artifacts as this story’s proof surface
- Not claiming live main already ships **5.3.0** / **5.4.0**

### 2.3 Required practical checks (docs story)

| Check | Command | Notes |
|-------|---------|-------|
| Whitespace / conflict markers | `git diff --check` | Fail-close |
| SemVer shape (no accidental bump) | `./scripts/check-semver.sh` | Expect stack **5.4.0** cargo ≡ package.json; **not** a bump unit |
| Path A linkage | `./scripts/check-path-a-linkage.sh` | Must stay green |
| Owner-bar | `./scripts/test-owner-bar.sh` | Must stay green; restore TSV |
| Heart regression | `./scripts/test-heart-regression.sh` | Must stay green; restore TSV |

Hermetic L3 R0A re-prove is **not** required for pure docs unless this story changes agent/CLI sources (it must not). Cite VC013 re-prove at **`96a9b3c`** as provenance for L3 claims.

---

## 3. PR units (single stacked PR; atomic commits)

This story is **one PR** (exactly one unmerged stacked PR). Internally:

### PR unit 1 — `docs(product): VC014 user-guide + KNOWN_LIMITS plan and floor`

- **Intent:** Plan-first gate; record live floor, acceptance map, residual allow-list before any doc edit.
- **Touches:** `docs/product/evidence/VC014_USER_GUIDE_KNOWN_LIMITS_2026-08-08.md` (this file)
- **Depends on:** VC013 tip / #145
- **Parallelizable with:** none (first)
- **SemVer:** none
- **Tests:** none (docs only)

### PR unit 2 — `docs(user-guide): align Path A L1–L3 guides with 5.4.0 stack`

- **Intent:** Rewrite/refresh user-facing guides so dual CLI, Spec 45, L2 scope, and L3 dogfood + opt-in/headless match on-branch Path A evidence.
- **Touches (expected):**  
  `docs/user-guide/README.md`  
  `docs/user-guide/04-surface.md` (L2 effort/cache honesty)  
  `docs/user-guide/10-tools.md`  
  `docs/user-guide/11-subagents.md`  
  `docs/user-guide/12-background-tasks.md`  
  `docs/user-guide/13-worktrees.md`  
  `docs/user-guide/14-l3-throughput.md`
- **Depends on:** unit 1
- **SemVer:** none
- **Tests:** static consistency with VC006–VC013 claims; no over-claim

### PR unit 3 — `docs(product): refresh KNOWN_LIMITS for vision residuals`

- **Intent:** Drop stale 4.0.0 / “next feature minor 5.2.0+” residual framing; keep only evidenced residuals + floor honesty for unmerged **5.4.0**.
- **Touches:** `docs/product/KNOWN_LIMITS.md`
- **Depends on:** unit 2 (or after unit 1 if content is independent — prefer after guides so residual text matches guide honesty)
- **SemVer:** none
- **Tests:** residual allow-list §2.1 present; no false “vision complete”

### PR unit 4 — `docs(product): VC014 READY evidence + independent review`

- **Intent:** Fill READY provenance, gate table, residual table; land independent Grok review file.
- **Touches:** this evidence file (READY), `VC014_INDEPENDENT_REVIEW_2026-08-08.md`
- **Depends on:** units 2–3
- **SemVer:** none
- **Tests:** `git diff --check`; practical gates §2.3; English PR body gate at open

## Sequential

1. unit 1 → unit 2 → unit 3 → unit 4 → open PR (no merge)

## Parallel

- None (single worker, single stack slot; docs touch shared product truth).

---

## 4. Content contract (what the guides must say)

### 4.1 Dual naming

- Primary command **`deepseek-build`**, alias **`dsb`** (same binary behavior).
- Config home remains `~/.deepseek-build/` (path ≠ command name).
- Both report the **same** full `MAJOR.MINOR.PATCH` from product packaging.

### 4.2 Spec 45 (L1) — honest Path A

- Path A default agent uses **session `snippet_id`** mint/require/invalidation (VC003–VC006 stack; on-branch cut **5.3.0** packaging of Spec 45 Deep Code).
- Thin overlay `dsb-tools` / `dsb run` is a **different surface** (names may differ); do not conflate with Path A sole proof.
- Free-form whole-file primary edit remains fail-closed on Path A snippet contract.

### 4.3 L2 (Reasonix) — honest scope

| Topic | Claimable on Path A | Residual / scope note |
|-------|---------------------|------------------------|
| Spec 10 assembly | Grok Path A turns apply stable-prefix assembly (VC007) | Not every historical wire JSONL is post-VC007; do not claim full Spec 30 thinking body field |
| `reasoning_effort` | Product seed/repair + public-entry wire (VC008) | CLI override hermetic wire residual; session-title side model may omit effort |
| Cache signal | User-visible `cache N%` path + loggable Path A stamp (VC009) | Live provider hit rates are server policy; hermetic fixture ≠ live hit rate |

### 4.4 L3 (Grok throughput) — dogfood + limits

| Topic | Claimable | Limit |
|-------|-----------|--------|
| Multi-tool RO parallel + mutate serial | VC010 / VC013 hermetic Path A R0A | Cap / serial mutate honesty |
| Background shell + collect-by-id | VC010 / VC013 | Tool **names** differ thin vs agent path |
| Explore + implement-class subagents | VC011 / VC013 | Disable: `--no-subagents` / config / env |
| Worker stable-prefix epoch stamp | VC011 / VC013 `worker_epochs_match=true` | — |
| Parent snippet expire after worker mutation | **Not claimable as Path A sole green** | **V3-60-3 residual** |
| Worktree | Opt-in `--worktree` / product top-level forward (VC012) | Bare session single; headless `-p` **no create**; interactive TTY create residual |

### 4.5 Packaging / publish honesty

- Prebuilt platform: **`darwin-arm64`** only unless docs prove otherwise.
- npm / GitHub publish remain **human-gated**.
- Stack **5.4.0** ≠ live registry/GitHub Latest until merge + release lane.

---

## 5. Stack / PR open checklist (implementer)

- Branch: `vc014-vision-docs`
- Base: `vc013-5-4-cut`
- Body: **`Depends on #145`**
- Title: `docs(product): user-guide + KNOWN_LIMITS vision pass`
- Labels: **`docs`** + **`area/docs`** (existing repo labels; type/kind match)
- English public text; run `~/.local/bin/gh-public-english-gate`
- Verify: `gh pr view --json title,labels,baseRefName,headRefName,url,mergeable`
- **Do not merge**

---

## 6. READY evidence

**Status: PLAN only** — filled after units 2–4 land.

### 6.0 Provenance (to fill)

| Field | Value |
|-------|--------|
| Plan-first commit | *(this commit SHA)* |
| Docs source head (READY) | *(HEAD after unit 4)* |
| L3 behavior provenance (cited, not re-run) | VC013 cut head **`96a9b3c`** + META/WIRE re-prove |
| Spec 45 provenance | VC006 READY **5.3.0** stack cut |
| L2 provenance | VC007 / VC008 / VC009 READY |

### 6.1 Gate table (to fill)

| Command | Result |
|---------|--------|
| `git diff --check` | |
| `./scripts/check-semver.sh` | |
| `./scripts/check-path-a-linkage.sh` | |
| `./scripts/test-owner-bar.sh` | |
| `./scripts/test-heart-regression.sh` | |

### 6.2 Residuals at READY (expected = allow-list)

Same as §2.1 — must not shrink without Path A proof.

### 6.3 Artifact pointers

| Artifact | Role |
|----------|------|
| This file | Plan + READY |
| `VC014_INDEPENDENT_REVIEW_2026-08-08.md` | Independent Grok review |
| User-guide 10–14 + README + surface | Behavior honesty |
| `KNOWN_LIMITS.md` | Residual honesty |
| VC013 / VC006–VC012 evidence | Cited Path A provenance |

---

## 7. Implementer checklist

- [x] Floor re-check (`main` / npm / `gh release` / stack tip **5.4.0**)
- [x] Plan written **before** user-guide / KNOWN_LIMITS edits
- [ ] Unit 1 committed alone
- [ ] Unit 2 user-guide updates
- [ ] Unit 3 KNOWN_LIMITS refresh
- [ ] Unit 4 READY + independent review
- [ ] Practical gates green; TSV side-effects restored
- [ ] Single stacked English PR base `vc013-5-4-cut` · Depends on #145 · labels verified
- [ ] No merge / no publish / no SemVer bump / no VC013 edit
