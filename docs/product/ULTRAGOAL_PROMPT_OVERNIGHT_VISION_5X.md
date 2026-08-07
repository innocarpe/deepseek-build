# Overnight ultragoal paste — **`vision-complete-5x`**

**Worktree:** `~/Projects/OpenSources/deepseek-build-vision-5x`  
**Branch:** `feat/vision-complete-5x` (tracks `origin/main` at create)  
**Runtime:** **Grok only** for this session and any children (no Claude/Codex spawn).

Copy **§ PASTE BELOW** into a new Grok Build / ultragoal session opened **in this worktree**.

---

## PASTE BELOW

```text
You are running an overnight ultragoal on DeepSeek Build.

## Plan
- Plan id (only active): vision-complete-5x
- Owner-bar v5.0.0 is DONE. Do NOT re-open G001–G012 / re-tag 5.0.0 or 5.1.0 as vision-complete.
- North star: docs/product/VISION.md + docs/architecture/HARNESS_PHILOSOPHY.md
  (Grok-class throughput + Reasonix cost/session + Deep Code native edit/tools)
- Board SSOT: docs/product/VISION_COMPLETE_5X_GOALS.md
- PR DAG: docs/product/WAVE_5x_VISION_PR_DAG.md
- Cold start detail: docs/product/ULTRAGOAL_PROMPT_COLD_START_VISION_5X.md
- PR planning: docs/product/ULTRAGOAL_PR_PLANNING.md + docs/contributing/* + skills/pr-authoring
- AGENTS.md + docs/product/SSOT.md

## SemVer floor (mandatory every release / every session start)
git fetch origin main
git show origin/main:Cargo.toml | rg 'version = "'
npm view @innocarpe/deepseek-build version
gh release list -R innocarpe/deepseek-build --limit 8

USED (do not plan as future feature cuts):
- 5.0.0 owner-bar cut
- 5.0.1 npm version/update fix
- 5.1.0 on main (theme/chrome) — may still be deploying

NEXT vision minors (unless floor moved — shift up, never reuse):
- 5.2.0 Deep Code Spec 45 snippet_id (VC002–VC006)
- 5.3.0 Reasonix assembly + effort-on-wire (VC007–VC009)
- 5.4.0 L3 Path A dogfood (VC010–VC013)
- 5.5.0 (or free 5.Y.0) vision freeze dual review (VC014–VC015)

Always full MAJOR.MINOR.PATCH. One bump unit per release.

## Path A only
Behavior claims need public deepseek-build / dsb → agent evidence (R0A), not library-only Path B.
Keep green: ./scripts/test-owner-bar.sh · ./scripts/check-path-a-linkage.sh · ./scripts/test-heart-regression.sh

## Execution rules (overnight)
1. Do not stop for applause. Story after story; continuous PRs.
2. Before coding each story: write PR unit plan (ULTRAGOAL_PR_PLANNING.md).
3. Atomic Conventional Commits; GitHub merge commit (no squash); English PR body; labels (github-pr skill / gh-public-english-gate).
4. Evidence under docs/product/evidence/.
5. Child agents/worktrees: Grok only (parent-runtime-child).
6. Disk: after large agent builds, rm -rf third_party/grok-build/target and workspace target if free space is low. Do not leave 40G+ trees overnight.
7. If blocked: document residual honestly; never claim vision-complete without V1–V4 criteria.

## Start order tonight
1. Floor check versions (above).
2. If 5.1.0 GitHub Release or npm still lagging main → VC001c only (finish 5.1.0 ship or 5.1.1 hot patch). Do not invent a new minor for packaging.
3. Else begin VC002 Spec 45 ADR + SnippetStore design, then VC003–VC006 toward 5.2.0.
4. After 5.2.0: Track B (5.3.0) and Track C (5.4.0) per DAG; freeze 5.5.0 only when board criteria met + dual review.

## First actions right now
- Read VISION_COMPLETE_5X_GOALS.md + WAVE_5x_VISION_PR_DAG.md
- Run floor check
- Report floor (main version, npm version, latest gh releases)
- Either VC001c or open VC002 PR unit plan and implement
```

---

## How to open the session

```bash
cd ~/Projects/OpenSources/deepseek-build-vision-5x
# Grok Build TUI / ultragoal in THIS directory
dsb
# or: grok …
```

Then paste **§ PASTE BELOW** as the first user message (or load ultragoal with that text).

## Worktree facts

| Item | Value |
|------|--------|
| Path | `/Users/WooseongKim/Projects/OpenSources/deepseek-build-vision-5x` |
| Branch | `feat/vision-complete-5x` |
| Base | `origin/main` @ create (includes vision SemVer rebase PR #122) |
| Remove later | `git -C ~/Projects/OpenSources/deepseek-build-owner-bar-5x worktree remove deepseek-build-vision-5x` |

## Optional: push branch early

```bash
cd ~/Projects/OpenSources/deepseek-build-vision-5x
git push -u origin feat/vision-complete-5x
```
