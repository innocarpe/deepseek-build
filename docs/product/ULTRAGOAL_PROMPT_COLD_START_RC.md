# Cold-start prompt — Wave D `rc-1.0.0`

Use after Wave B (required) and Wave C (recommended). Only tag **1.0.0** when PRD-wave-D checklist is green.

```text
# ROLE
Autonomous agent for DeepSeek Build Wave D (RC → 1.0.0).
Cold start. Do not tag 1.0.0 early. Full SemVer only.

# MISSION
Plan **rc-1.0.0** per PRD-wave-D-rc.md and MASTER_PLAN Wave D.
Minors 0.15.0, 0.16.0, then 1.0.0 when exit criteria pass.

# CREATE PLAN IF NEEDED
omc ultragoal create-goals --plan-id rc-1.0.0 \
  --brief "Wave D RC: harden, user-guide, CI smoke, then 1.0.0 only if checklist green." \
  --goal "v0.15.0-Harden::CI smoke + regression; ship 0.15.0" \
  --goal "v0.16.0-Docs::user-guide + known-limits + CHANGELOG; ship 0.16.0" \
  --goal "v1.0.0-Release::All PRD-wave-D exit criteria; tag 1.0.0 + npm/binaries" \
  --claude-goal-mode aggregate

# START
git pull origin main
omc ultragoal complete-goals --plan-id rc-1.0.0
# PR units first (ULTRAGOAL_PR_PLANNING.md); stack release docs vs code if needed
# Final story requires quality-gate-json for ultragoal
```
