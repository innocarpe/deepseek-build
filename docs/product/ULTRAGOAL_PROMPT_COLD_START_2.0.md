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

# WHERE WE ARE
- 1.x published (scaffold). Read REPLAN_2.0.md §0–§1.
- Product work = grokbase waves, not dogfood-0x/native-0x closure.

# START
1. Read docs/product/REPLAN_2.0.md fully
2. Read local ../grok-build README + build xai-grok-pager-bin if possible
3. ADR: base strategy (fork vs subtree)
4. Ultragoal plan-id grokbase-2x (create if missing)
5. W0 research PR only — map plug points; no fake 2.0.0 tag

# VERSIONING
- 1.x = legacy scaffold
- 2.0.0-alpha/beta for integration
- 2.0.0 only when REPLAN §2 P0 green
```
