# VC014 — independent Grok review (user-guide + KNOWN_LIMITS vision pass)

| Field | Value |
|-------|--------|
| **Story** | **VC014** — `docs(product): user-guide + KNOWN_LIMITS vision pass` (vision **V4-docs**) |
| **Branch / worktree** | `vc014-vision-docs` @ `/Users/WooseongKim/Projects/deepseek-build/vc014-vision-docs` |
| **Plan-first head** | `2a20fb2` (docs-only plan; not sole READY proof) |
| **Docs units** | `37347d7` user-guide · `9f71e3e` KNOWN_LIMITS · `f66067c` whitespace |
| **Reviewed docs head (gates)** | `f66067c` (pre READY-tip content head; READY file lands after) |
| **Base** | `vc013-5-4-cut` @ `62077f4` (open PR **#145**) |
| **Reviewer** | Independent Grok review lane (static docs honesty + re-ran practical gates) |
| **Date** | 2026-08-08 |
| **Verdict** | **READY** |
| **Merge / publish** | **Do not merge.** **Do not publish.** **Do not bump SemVer.** |

### Head honesty (fail-close)

| Commit / artifact | Covered by this review |
|-------------------|------------------------|
| Plan `2a20fb2` | Floor + acceptance map + residual allow-list only |
| User-guide `37347d7` | Guides README / 04 / 10–14 Path A honesty |
| KNOWN_LIMITS `9f71e3e` | Residual allow-list + stack vs live floor |
| Whitespace `f66067c` | `git diff --check` clean |
| READY evidence + this review | Gate claims + residual non-closure |

**Provenance rule:** Behavior claims must cite VC006–VC013 Path A evidence. This story must **not** invent residual closure or re-label live npm/GitHub as **5.4.0**.

Docs-only tips after this file that only point at the review do **not** require a second code review.

---

## 1. Scope checked (story must-hold)

| Must-hold | Result |
|-----------|--------|
| Live floor re-check (main / npm / gh / stack) | **OK** — main/npm/gh **5.2.2**; stack **5.4.0** unchanged |
| Docs-only; no code feature; no SemVer bump | **OK** — cargo/package remain **5.4.0** |
| Dual CLI naming in user-facing docs | **OK** — README + L3 guides |
| Spec 45 Path A `snippet_id` honesty (not file_version-only) | **OK** — `10-tools.md` + KNOWN_LIMITS evidenced table |
| L2 assembly / effort / cache with scope | **OK** — `04-surface.md` + residual scope notes |
| L3 parallel / bg / subagent / worktree + opt-in/headless | **OK** — guides 11–14; VC010–VC013 pointers |
| Residual allow-list retained (no false close) | **OK** — V3-60-3, interactive worktree create, non-darwin, human-gated publish, stack lag |
| No stale “next feature minor 5.2.0+” residual framing | **OK** — removed from KNOWN_LIMITS residual story |
| No vision freeze claim | **OK** — explicit non-claims |
| No VC013 / PR #145 edit | **OK** |
| Stacked on `vc013-5-4-cut` / Depends on **#145** | **OK** — plan + branch base |
| Practical gates green; side-effects restored | **OK** — re-ran this lane |

---

## 2. What was verified (this review)

### 2.1 Verification method

1. **Static read** of plan, user-guide README/04/10–14, `KNOWN_LIMITS.md` against VC006–VC013 READY claims
2. **Residual scan** for over-claim language (freeze, residual closed, live 5.4.0 registry)
3. **Re-execution** of practical gates at docs head `f66067c`:
   - `git diff --check` base..HEAD → **PASS**
   - `./scripts/check-semver.sh` → **PASS** (5.4.0 cargo ≡ npm package field)
   - `./scripts/check-path-a-linkage.sh` → **PASS**
   - `./scripts/test-owner-bar.sh` → **PASS** (60/60); restored `OWNER_BAR_STATUS.tsv`
   - `./scripts/test-heart-regression.sh` → **PASS**
4. Floor probe: `origin/main` / npm / `gh release` Latest still **5.2.2**

```text
# Floor
origin/main Cargo version = 5.2.2
npm @innocarpe/deepseek-build = 5.2.2
gh release Latest = v5.2.2
stack Cargo/package = 5.4.0 (unchanged by VC014)

# Residuals must remain open in docs
V3-60-3 parent snippet expire (Path A)
interactive TTY worktree create
darwin-arm64-only prebuilt
human-gated publish
stack 5.4.0 != live 5.2.2
```

### 2.2 Content honesty sample

| Doc | Check |
|-----|-------|
| `10-tools.md` | Path A vs thin; Spec 45 `snippet_id`; VC006 pointer |
| `11-subagents.md` | VC011/VC013 dogfood; **V3-60-3 residual** explicit |
| `12-background-tasks.md` | VC010/VC013; no “wait for 4.0.0” residual |
| `13-worktrees.md` | opt-in; headless no-create; interactive create residual |
| `14-l3-throughput.md` | V3 table; 5.4.0 stack ≠ freeze |
| `04-surface.md` | effort / assembly / cache scope tables |
| `KNOWN_LIMITS.md` | evidenced stack table + residual allow-list + non-claims |

### 2.3 Stack base correctness

- Branch `vc014-vision-docs` forked at VC013 tip `62077f4`.
- Open PR base must be **`vc013-5-4-cut`** with **Depends on #145**.
- Do **not** target `origin/main` until #145 merges.

---

## 3. Residuals (honest)

| Residual | Notes |
|----------|-------|
| V3-60-3 Path A parent snippet expire | **Carry** — docs must not close |
| Interactive TTY worktree **create** | **Carry** — docs must not close |
| Non-darwin packaging / assets | **Carry** |
| Human-gated npm / GitHub publish | **Carry** |
| Live main/npm/GitHub still **5.2.2** | Expected until stack merges + release lane |
| Vision freeze **5.5.0** / VC015 | Out of story |

### Claim bound (reviewer must not over-read)

- **Docs pass** ≠ vision-complete freeze.
- **On-branch 5.4.0** ≠ shipped npm/GitHub Release.
- **Citing VC013 R0A** ≠ re-running L3 R0A in this story (not required for pure docs).
- **V3-60-3** remains residual despite V3-60-1/2 green.

---

## 4. Verdict

**READY** to open the stacked **`docs(product)`** PR for the V4-docs vision pass. **Do not merge** in this story. **Do not publish** npm or GitHub Release. **Do not bump SemVer.**
