# Ultragoal brief — plan **`grokbase-2x`** (product 2.0.0)

Mission text for ultragoal / long autonomous runs. **Normative DoD:** [REPLAN_2.0.md](./REPLAN_2.0.md).  
**Full story board:** [GROKBASE_2X_GOALS.md](./GROKBASE_2X_GOALS.md).  
**PR units:** [WAVE_2x_PR_DAG.md](./WAVE_2x_PR_DAG.md).

## Mission

Ship **DeepSeek Build 2.0.0**: a **Grok Build–class** terminal coding agent for DeepSeek.

1. **Base** = open-source **Grok Build** runtime + full-screen TUI (not greenfield “Grok vibes”).  
2. **Default model path** = DeepSeek (provider/auth/setup).  
3. **Overlays** = Deep Code L1 (snippet, permissions) + Reasonix L2 (prefix/cache discipline).  
4. **Entry** = `dsb` / `deepseek-build` with no args opens the agent.  
5. **1.x** remains published **scaffold** — freeze product feature creep on thin REPL.

## Hard constraints

- Full SemVer only (`MAJOR.MINOR.PATCH`). Bands: docs → `2.0.0-alpha.N` → `2.0.0-beta.N` → **`2.0.0`**.  
- **Do not** tag `2.0.0` until REPLAN §2 **P0** is green (G012).  
- Dual CLI always: `deepseek-build` + `dsb`.  
- Fixed units only from WAVE_2x_PR_DAG — no overnight invention.  
- PR planning first ([ULTRAGOAL_PR_PLANNING.md](./ULTRAGOAL_PR_PLANNING.md)).  
- Parent runtime family only.  
- npm **publish** never agent-forced complete (ADR 0007).  
- Do **not** restart dogfood-0x / native-0x / throughput-0x / rc-1.0.0 as product SSOT.  
- Do **not** make multi-vendor or Gajae multi-stage planning the identity.

## Success (aggregate)

All **12** stories in plan `grokbase-2x` complete with evidence; success feeling in REPLAN §9 holds.
