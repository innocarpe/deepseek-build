# Cold-start prompt — Wave B `native-0x`

Use **only after** `dogfood-0x` is fully complete. Paste fenced block into a new session if needed.

```text
# ROLE
Autonomous agent for DeepSeek Build Wave B (DeepSeek-native surface).
Cold start: read repo only. Final goal: docs/product/MASTER_PLAN.md §1.

# MISSION
Execute ultragoal plan **native-0x** (create if missing) per
docs/product/prd/PRD-wave-B-native.md and MASTER_PLAN Wave B.
SemVer band **0.8.0–0.11.0**. Dual CLI. Full SemVer only. No 1.0.0 yet.
Include **DeepSeek blue readable default theme**.

# CREATE PLAN IF NEEDED
omc ultragoal create-goals --plan-id native-0x \
  --brief "Wave B DeepSeek-native: tools polish, permissions UX, theme v1, skills, MCP, plan light. 0.8.0-0.11.0. No parallel/subagents." \
  --goal "v0.8.0-Spec40::Spec 40 ready-for-impl + tool surface polish; ship 0.8.0" \
  --goal "v0.9.0-PermTheme::Interactive permissions + DeepSeek blue theme v1; ship 0.9.0" \
  --goal "v0.10.0-Skills::Skills product spec 70; ship 0.10.0" \
  --goal "v0.11.0-McpPlan::MCP + light plan with cache epochs; ship 0.11.0" \
  --claude-goal-mode aggregate

# START
git pull origin main
omc ultragoal complete-goals --plan-id native-0x
# For EACH story: write PR unit plan (ULTRAGOAL_PR_PLANNING.md) first
# sequential/parallel/stacking/atomic commits explicit
# Read HARNESS, GATES, SYSTEM_ARCHITECTURE, PRD-wave-B-native
# Ship vertical PRs; never G4 parallel tools in this wave
```
