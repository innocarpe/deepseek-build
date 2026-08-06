# Cold-start prompt — Wave C `throughput-0x`

Use **only after** `native-0x` complete. G4 before parallel code; G5 before subagents.

```text
# ROLE
Autonomous agent for DeepSeek Build Wave C (Grok-class throughput).
Cold start. Final goal: MASTER_PLAN.md §1 L3 without breaking L1/L2.

# MISSION
Plan **throughput-0x**, SemVer **0.12.0–0.14.0**, PRD-wave-C-throughput.md.
Worker cache law mandatory. No YOLO shell. Full SemVer. Dual CLI.

# CREATE PLAN IF NEEDED
omc ultragoal create-goals --plan-id throughput-0x \
  --brief "Wave C Grok throughput: spec50/G4 parallel, bg shell, spec60/G5 subagents. 0.12.0-0.14.0. Preserve L1/L2." \
  --goal "v0.12.0-Parallel::Spec 50 ready + G4 green + parallel tools; ship 0.12.0" \
  --goal "v0.13.0-BgShell::Background shell + collect IDs; ship 0.13.0" \
  --goal "v0.14.0-Subagents::Spec 60 + G5 + subagents/worktree + cache law tests; ship 0.14.0" \
  --claude-goal-mode aggregate

# START
git pull origin main
omc ultragoal complete-goals --plan-id throughput-0x
# PR units first (ULTRAGOAL_PR_PLANNING.md); stack sequential; atomic commits
```
