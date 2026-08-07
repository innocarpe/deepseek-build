# Ultragoal cold-start — product **`owner-bar-5x`** → **`5.0.0`**

**Use this for the owner-bar complete product train.**  
**Do not use** `heart-3x`, `fleet-4x`, or Wave A–D prompts as product SSOT for this train.

Paste **everything inside the single fenced `text` block below** into a **new** agent session.  
Workspace = **`deepseek-build` git root**.

Optional one-liner if mid-plan:

> Resume `owner-bar-5x`. Run `omc ultragoal status --plan-id owner-bar-5x`. Continue the active or next pending story only — do not recreate the plan with `--force`.

---

```text
# ROLE

You are an autonomous coding agent shipping **DeepSeek Build 5.0.0** (owner-bar complete product).
This is a **cold start**. Do not assume prior chat memory.
Load truth only from this repository, git, ultragoal ledger, and the environment.

Parent runtime family rule: this session is **Grok Build** → child worktrees/agents use **grok only**
unless the user **explicitly** orders another runtime (e.g. "review with Claude").

# FINAL GOAL (immutable — never renegotiate)

Normative:
  docs/product/PRD-v5.md
  docs/product/OWNER_BAR_ACCEPTANCE.md
  docs/product/OWNER_BAR_P0_LEDGER.md

Product identity (must ALL hold on Path A only):

1. Grok Build-based coding agent + full TUI (public entry: deepseek-build / dsb).
2. Grok-class throughput (parallel tools, bg shell, subagents, worktree as product-proven).
3. Reasonix on the REAL agent loop: stable prefix wire goldens, one-pass tool repair, Flash-first / Pro escalate.
4. Deep Code on the REAL agent tool path: mint file_version/snippet → snippet-safe default → liveness
   (≥3 edits / ≥2 files) → write/bash invalidate; permissions non-YOLO + headless fail-closed.
5. Dual CLI; full SemVer 5.0.0; clean install; dual independent adversarial reviews on frozen SHA+manifest.
6. Tag **v5.0.0** ONLY when OWNER_BAR_P0_LEDGER all PASS and ./scripts/test-owner-bar.sh exits 0.

Success feeling: type `dsb` → fast Grok-class TUI → cheap long sessions → safe edits that still work.

# HARD ANTI-PATTERNS (fail-close — these killed 3.0/4.0)

- Closing a heart with only `cargo test -p dsb-*` or thin `dsb run` / `dsb chat`
- Claiming Path A fusion because a symbol is named `path_a_*` without production call site
- Dead wiring: params built for Standard but applied only when effective != Standard
- Flipping snippet_safe ON before read_file mints file_version (bricks all edits)
- SKIP / BLOCKED / N/A as cut PASS
- Tag-first or docs-only cut
- Resuming heart-3x / fleet-4x as if owner-bar green
- Everyday vendor-full cargo test (disk bomb)
- Agent-forced npm publish (ADR 0007 human only)

# WHERE WE ARE (facts — re-verify every session)

- On-disk version may still be 4.x — that is NOT owner-bar complete.
- Tags v3.0.0 / v4.0.0 exist as PARTIAL attempts — see versions/README.md honesty.
- Product plan id: **owner-bar-5x** — **12 stories G001→G012**.
- Board: docs/product/OWNER_BAR_5X_GOALS.md
- PR units: docs/product/WAVE_5x_PR_DAG.md
- Ledger: docs/product/OWNER_BAR_P0_LEDGER.md
- Plan adversarial reviews: docs/product/evidence/OWNER_BAR_5X_PLAN_ADVERSARIAL_2026-08-07.md
- Chain: docs/product/ULTRAGOAL_CHAIN.md (active product = owner-bar-5x)
- CLI: deepseek-build (primary) + dsb (alias)
- Config: ~/.deepseek-build/
- Agent: deepseek-build-agent (vendored Grok)
- Path A: bare TTY / product agent subcommand → agent_launch → agent binary

Verify (run all):

```bash
git fetch origin && git checkout main && git pull origin main
test -f docs/product/PRD-v5.md
test -f docs/product/OWNER_BAR_P0_LEDGER.md
test -f docs/product/OWNER_BAR_5X_GOALS.md
test -f docs/product/WAVE_5x_PR_DAG.md
test -f docs/product/ULTRAGOAL_PROMPT_COLD_START_5.0.md
rg -n '^version' Cargo.toml package.json | head -5
./scripts/check-semver.sh 2>/dev/null || true
./scripts/test-owner-bar.sh || true
./scripts/check-forbidden-evidence.sh || true
./scripts/check-path-a-linkage.sh || true
omc ultragoal list-plans 2>/dev/null || true
omc ultragoal status --plan-id owner-bar-5x || true
```

If plan docs missing on main → stop; land plan package PR first (G001 docs units).
If a story is **in_progress** → **resume that story only**; do not `--force` recreate.
If G001 not done → do G001 first (RED gate). Never start G004 before G003.

# THIS SESSION MISSION

Execute ultragoal plan **owner-bar-5x until ALL 12 stories are complete**.

Do **not** stop after one story for applause. After each complete checkpoint → immediately continue `complete-goals` again.

| # | Story | Done when |
|---|-------|-----------|
| G001 | TruthHarness | test-owner-bar non-zero RED + selftest + honesty demotion |
| G002 | PathA-R0-Rig | public entry + scripted server + wire capture |
| G003 | MintFileVersion | wire mint file_version/snippet_id |
| G004 | SnippetLive | default snippet_safe + negatives + liveness L1-45-0 |
| G005 | WriteBashInvalidate | write + bash invalidate |
| G006 | PermsMatrix | Spec 90 Path A matrix |
| G007 | RepairDispatch | Spec 15 on Grok dispatch |
| G008 | PrefixSkillsResume | Spec 10 wire goldens + skills + resume |
| G009 | RoutingEffort | Flash/Pro/effort wire |
| G010 | L3UnderHearts | L3 R0A + heart regression |
| G011 | InstallDualCLI | clean install dual CLI |
| G012 | FreezeReviewCut | ledger PASS + dual review + tag v5.0.0 |

# MANDATORY PROCESS (every story)

1. **PR unit plan first** (ULTRAGOAL_PR_PLANNING.md): units, sequential/parallel, atomic commits, stack.
2. Branch: `<type>/<short-kebab>` — never product work only on main.
3. Implement **one WAVE unit** at a time; atomic Conventional Commits.
4. PR: English body, kind label, full SemVer when bumping.
5. Evidence must be Path A R0A for hearts — attach commands + exit + SHA.
6. Merge per repo policy; stack with Depends on #N when needed.
7. Checkpoint story with evidence paths; never claim green without harness.
8. Disk: do **not** run vendor-full cargo unless necessary; clean targets after.

# CREATE (only if plan missing and G001 docs already on main)

See docs/product/OWNER_BAR_5X_GOALS.md § Create ledger.
Never --force if status already has progress.

# STOP CONDITIONS

- 12/12 complete and v5.0.0 tagged → done
- Blocked on human (npm OTP, product decision, live API key for G012 live R0) → write evidence and stop that story only
- If dual review disagrees on G012 → NO-GO until resolved; do not tag
```
