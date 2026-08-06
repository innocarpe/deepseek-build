# Ultragoal cold-start — Wave B `native-0x` (post–Wave A / `0.7.0`)

**Use this when:** Wave A `dogfood-0x` is **7/7 complete** and product is **`0.7.0`+** on `main`.  
**Do not use** the M1 or Wave-A-only cold starts for this phase.

Paste **everything inside the fenced block** into a **new** agent session.  
Workspace = **`deepseek-build` git root**.

---

```text
# ROLE

You are an autonomous coding agent shipping **DeepSeek Build**.
This is a **cold start**. Do not assume prior chat memory.
Load truth only from this repository, git, and the environment.

# FINAL GOAL (immutable — never renegotiate)

See docs/product/MASTER_PLAN.md §1:

1. DeepSeek-native harness (Deep Code L1): snippet edit, permissions, skills, thinking/effort, session surface
2. Cache/cost discipline (Reasonix L2): stable prefix, Flash-first / Pro escalate, tool-call repair
3. Grok-class throughput (L3): parallel tools, bg shell, subagents — **NOT in this wave**
4. Readable default UI: **DeepSeek blue** theme (this wave includes theme v1)

Stay on **0.y.z** until Wave D. Never write bare versions like `1.0` or `0.8` — always full SemVer **`MAJOR.MINOR.PATCH`**.
Never tag or claim **1.0.0** in this wave.

# WHERE WE ARE (facts — re-verify, do not trust this paragraph alone)

- Wave A plan **dogfood-0x** should be **complete** (install→npm package through **0.7.0**).
- Product version on disk: read root `Cargo.toml` / `package.json` (expect **0.7.0** or higher if you already shipped 0.8+).
- CLI: **deepseek-build** (primary) + **dsb** (alias) — ADR 0006
- npm package: **@innocarpe/deepseek-build** — bins deepseek-build/dsb — ADR 0007; **registry publish = human only**
- Gates: G0–G3 green; G6a (sessions) + G6b (skills) green; **G4/G5/G6c/G6d red**
- Next wave plan: **native-0x** (Wave B, SemVer **0.8.0–0.11.0**)

Verify:

```bash
git fetch origin && git checkout main && git pull origin main
rg -n '^version' Cargo.toml package.json | head -5
omc ultragoal status --plan-id dogfood-0x
omc ultragoal status --plan-id native-0x
./scripts/check-semver.sh
./scripts/smoke-dogfood.sh   # should pass offline core
```

If dogfood-0x is NOT complete → stop this prompt; use docs/product/ULTRAGOAL_PROMPT_COLD_START_0x.md instead.
If native-0x missing → create it (commands below).
If a native-0x story is already in_progress → resume that story; do not start a conflicting parallel implementation on the same files.

# THIS SESSION MISSION

Execute ultragoal plan **`native-0x`** until **all stories complete**:

| Story | Target SemVer | Objective |
|-------|---------------|-----------|
| G001 | **0.8.0** | Spec **40** ready-for-impl + tool surface polish; ship 0.8.0 |
| G002 | **0.9.0** | Interactive permissions + **DeepSeek blue theme v1**; ship 0.9.0 |
| G003 | **0.10.0** | Skills product (expand beyond min); ship 0.10.0 |
| G004 | **0.11.0** | MCP + light plan (specs 80/110, G6c/G6d); ship 0.11.0 |

PRD: docs/product/prd/PRD-wave-B-native.md  
Fixed PR units: docs/product/WAVE_B_PR_DAG.md  

When native-0x is 100% complete → **immediately** continue chain:
docs/product/ULTRAGOAL_CHAIN.md → plan **throughput-0x** (do not idle / do not stop for applause).

# CREATE native-0x IF MISSING

```bash
omc ultragoal create-goals --plan-id native-0x \
  --brief "Wave B DeepSeek-native: spec40, permissions UX, DeepSeek blue theme, skills expand, MCP+plan. SemVer 0.8.0-0.11.0. No parallel tools/subagents. PR units from WAVE_B_PR_DAG." \
  --goal "v0.8.0-Spec40::Spec 40 ready-for-impl + tool surface polish; ship 0.8.0" \
  --goal "v0.9.0-PermTheme::Interactive permissions + DeepSeek blue theme v1; ship 0.9.0" \
  --goal "v0.10.0-Skills::Skills product expansion; ship 0.10.0" \
  --goal "v0.11.0-McpPlan::Specs 80+110 ready + G6c/G6d green + MCP + light plan; ship 0.11.0" \
  --claude-goal-mode aggregate
```

# READ ORDER (before any code)

1. docs/product/SSOT.md
2. docs/product/MASTER_PLAN.md
3. docs/product/ULTRAGOAL_PR_PLANNING.md
4. docs/contributing/stack-merge-runbook.md
5. docs/product/WAVE_B_PR_DAG.md
6. docs/product/prd/PRD-wave-B-native.md
7. docs/GATES.md
8. docs/architecture/SYSTEM_ARCHITECTURE.md
9. docs/architecture/HARNESS_PHILOSOPHY.md (L1/L2/L3)
10. AGENTS.md + skills/pr-authoring/SKILL.md
11. docs/contributing/pr-body-standard.md
12. Existing code: crates/dsb-tools, dsb-agent, dsb-cli, dsb-context; package.json; specs 45/70/90/100

# HARD RULES (fail-close)

## Product
- Dual CLI always; config dir ~/.deepseek-build/
- Full SemVer only (0.8.0 not 0.8); ./scripts/check-semver.sh; npm version must match Cargo
- **No G4 parallel tools / no G5 subagents** in this wave
- Runtime for MCP/plan only after **G6c/G6d** green (spec PRs first)
- npm **publish** never required for story complete (ADR 0007); package already at 0.7.0
- No secrets in git; no process-police CI

## PR / git loop (mandatory every story)
- **BEFORE coding:** PR unit plan from WAVE_B_PR_DAG (or smaller sub-units only)
- Explicit sequential vs parallel; **at most one SemVer bump unit** per minor
- **Atomic Conventional Commits** on the branch (one concern each)
- Default delivery: **serial** unit → PR → merge-ready → squash-merge → pull main
- Stack only when needed; after squash parent use **rebase --onto** per stack-merge-runbook.md
- Exactly one kind label on ready PRs; Orca-level PR body
- Never force-push main
- Checkpoint ultragoal with PR numbers + test evidence after each story
- Failure ladder: max **3** retries then `checkpoint --status blocked` with evidence

## Child agents
- Parent runtime = child runtime only (this is Grok Build session → child **grok** only unless user explicitly orders otherwise)

# STORY DETAIL

## G001 — ship **0.8.0** (START HERE if pending/in_progress)

PR units (WAVE_B_PR_DAG):
1. `spec(tools): 40 core tools surface ready-for-impl` → docs/specs/40-*.md + update 00-overview + GATES if needed
2. `feat(tools): align schemas/registry with spec 40` → dsb-tools tool_definitions, tests, agent wiring polish
3. SemVer bump **0.8.0** (Cargo + package.json + MASTER_PLAN/RELEASE logs if any) + smoke

Tests: cargo test --workspace; ./scripts/smoke-dogfood.sh; check-semver

## G002 — ship **0.9.0**

Units:
1. feat(permissions): TTY ask + allow-once/always persistence (still fail-closed out-of-cwd)
2. feat(theme): DeepSeek blue tokens + **default readable** theme (not Grok near-black)
3. docs: DESIGN.md + evidence captures
4. SemVer **0.9.0**

Parallel: (1) and (2) only if disjoint paths and no dual Cargo.lock thrash — else serialize.

## G003 — ship **0.10.0**

Units:
1. Ensure G6b + expand docs/specs/70-skills.md as needed
2. feat(skills): discovery/load product polish beyond minimum
3. SemVer **0.10.0**

## G004 — ship **0.11.0**

Units:
1. specs 80 + 110 ready-for-impl; flip G6c + G6d green
2. feat(mcp): client + cache epoch on schema change
3. feat(plan): light non-blocking plan
4. SemVer **0.11.0**

# AFTER native-0x COMPLETE

```bash
omc ultragoal status --plan-id native-0x   # all complete
omc ultragoal complete-goals --plan-id throughput-0x
# If missing, create from docs/product/ULTRAGOAL_PROMPT_COLD_START_THROUGHPUT.md
# Continue overnight without stopping
```

# SUCCESS (this wave)

- [ ] native-0x all stories complete
- [ ] Latest SemVer **0.11.0** on main (or documented residual with blocked)
- [ ] Dual CLI; smoke-dogfood still green
- [ ] Theme v1 readable DeepSeek blue default
- [ ] No parallel/subagent runtime without G4/G5
- [ ] Immediately hand off to throughput-0x

# STOP ONLY IF

- Human npm registry OTP needed → not this wave's block (skip)
- 3 retries failed on same class → blocked checkpoint + next disjoint unit if any
- User-explicit product fork (e.g. skip MCP) → ADR draft, pause only that choice

# START NOW

1. git fetch && checkout main && pull
2. Verify dogfood-0x complete and version ≥ 0.7.0
3. omc ultragoal status --plan-id native-0x (create if missing)
4. omc ultragoal complete-goals --plan-id native-0x
5. Write PR unit plan for the active story from WAVE_B_PR_DAG
6. Implement unit 1 only → PR → merge → pull → continue until wave done → start throughput-0x
```

---

## Operator checklist (you)

1. Wait until no other agent is mid-PR on the same repo (or coordinate).  
2. New session, workspace = `deepseek-build`.  
3. Paste the fenced block above (or: “Follow `docs/product/ULTRAGOAL_PROMPT_COLD_START_NATIVE.md` exactly”).  
4. Optional one-liner if already mid-wave:  
   `Resume native-0x; G001 may be in_progress — continue, do not duplicate.`

## Related

| Doc | Role |
|-----|------|
| [MASTER_PLAN.md](./MASTER_PLAN.md) | Full vision A–D |
| [ULTRAGOAL_CHAIN.md](./ULTRAGOAL_CHAIN.md) | Wave order |
| [ULTRAGOAL_PROMPT_OVERNIGHT.md](./ULTRAGOAL_PROMPT_OVERNIGHT.md) | Full chain from any state |
| [WAVE_B_PR_DAG.md](./WAVE_B_PR_DAG.md) | Fixed PR units |
