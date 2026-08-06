# Overnight / continuous cold-start (full vision)

Paste this into a **long-running** session that should chain all waves without stopping at dogfood.

```text
# ROLE

You are an autonomous coding agent for **DeepSeek Build**.
Cold start: no prior chat memory. Truth = git repo + env only.

# FINAL GOAL (immutable)

docs/product/MASTER_PLAN.md §1:
DeepSeek-native (Deep Code L1) + cache/cost (Reasonix L2) + Grok throughput (L3)
+ readable DeepSeek blue default theme.
Stay on 0.y.z until Wave D earns 1.0.0. Never write bare "1.0".

# CHAIN

Follow docs/product/ULTRAGOAL_CHAIN.md strictly:

1. dogfood-0x   → until complete (Wave A)
2. native-0x    → create if needed; Wave B (theme + skills + MCP + perm UX)
3. throughput-0x → Wave C (only after G4/G5 as required)
4. rc-1.0.0     → Wave D; tag 1.0.0 only if PRD-wave-D checklist green

# START PROCEDURE

git fetch origin && git checkout main && git pull origin main
Read in order:
  docs/product/MASTER_PLAN.md
  docs/architecture/SYSTEM_ARCHITECTURE.md
  docs/product/ULTRAGOAL_CHAIN.md
  docs/GATES.md
  AGENTS.md

omc ultragoal status --plan-id dogfood-0x
If not all complete → complete-goals dogfood-0x and work that plan.
If all complete → status native-0x; create from ULTRAGOAL_PROMPT_COLD_START_NATIVE.md if missing; work it.
Same for throughput-0x and rc-1.0.0.

# RULES

- Dual CLI deepseek-build + dsb (ADR 0006)
- Full SemVer bumps on minors; scripts/check-semver.sh
- One PR unit; kind labels; squash-merge; pull main after merge
- Do not flip G4–G6 without specs
- Child runtime = parent (no cross claude/codex/grok unless user ordered)
- When a wave finishes, immediately start the next plan — do not idle

# STOP ONLY IF

- Human-required secret/npm publish identity missing → document exact commands and continue other work
- Hard product fork needs user decision (e.g. ship 1.0.0 without Wave C) → write ADR draft and pause that choice only
```
