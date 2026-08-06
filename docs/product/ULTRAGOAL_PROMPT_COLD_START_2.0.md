# Cold-start — DeepSeek Build **2.0** (Grok base)

**Use this after** [REPLAN_2.0.md](./REPLAN_2.0.md) is on `main`.  
Do **not** use Wave A–D overnight prompts as product SSOT anymore.

```text
# ROLE
You are shipping DeepSeek Build **2.0.0** — a Grok Build–class terminal coding agent
for DeepSeek. Base = open-source Grok Build. Overlay = Deep Code L1 + Reasonix L2.

# FINAL GOAL (immutable)
`dsb` / `deepseek-build` with no args opens a full coding agent (Grok-class TUI/runtime),
DeepSeek by default, first-run setup works, L1/L2 invariants hold.

# NON-GOALS
- Extending the 1.x thin clap REPL as if it were the product
- Claiming 2.0.0 from checklist alone without Grok base entry
- Multi-vendor identity, Gajae multi-stage core loop
- Deleting or unpublishing 1.x history

# WHERE WE ARE
- 1.x published (scaffold). Read REPLAN_2.0.md §0–§2.
- Product work = grokbase waves in WAVE_2x_PR_DAG.md
- Historical A–D chain is closed scaffold only — do not restart as product SSOT

# START
1. Read docs/product/REPLAN_2.0.md fully
2. Read docs/product/WAVE_2x_PR_DAG.md — pick next incomplete unit
3. Read local ../grok-build README + LICENSE + SOURCE_REV; cargo check -p xai-grok-pager-bin if possible
4. If no ADR yet: write docs/adr/0008-grok-build-base.md (fork vs subtree)
5. Ultragoal plan-id grokbase-2x (create if missing); complete next unit only
6. Never tag 2.0.0 until REPLAN §2 P0 green

# VERSIONING
- 1.x = legacy scaffold (freeze product features)
- 2.0.0-alpha/beta for integration
- 2.0.0 only when REPLAN §2 P0 green

# PR CULTURE
- ULTRAGOAL_PR_PLANNING.md + stack-merge-runbook.md
- Small PRs; path-gated CI; English on GitHub public text
- Parent runtime family only (Grok → grok children)
```
