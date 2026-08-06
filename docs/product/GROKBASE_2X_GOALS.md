# Ultragoal board — plan id **`grokbase-2x`**

**One plate through product `2.0.0`.**  
Do not invent extra product plans overnight. Do not restart A–D as product SSOT.

| Normative | Path |
|-----------|------|
| Product DoD | [REPLAN_2.0.md](./REPLAN_2.0.md) §2 / §9 |
| Fixed PR units | [WAVE_2x_PR_DAG.md](./WAVE_2x_PR_DAG.md) |
| Cold start | [ULTRAGOAL_PROMPT_COLD_START_2.0.md](./ULTRAGOAL_PROMPT_COLD_START_2.0.md) |
| Brief | [ULTRAGOAL_BRIEF_2.0.md](./ULTRAGOAL_BRIEF_2.0.md) |
| Chain | [ULTRAGOAL_CHAIN.md](./ULTRAGOAL_CHAIN.md) |
| Local ledger | `.omc/ultragoal/plans/grokbase-2x/goals.json` |

---

## Final goal (immutable)

`dsb` / `deepseek-build` with **no args** opens a **Grok Build–class** full-screen coding agent, **DeepSeek by default**, first-run setup works, **L1/L2** minimums hold, install is dogfoodable.  
**Tag `v2.0.0` only when that is true** (REPLAN §2 P0).

---

## Story board (order is strict)

| # | Story title | WAVE_2x units | SemVer band | Done when |
|---|-------------|---------------|-------------|-----------|
| **G001** | ReplanOnMain | replan docs | docs | REPLAN + WAVE_2x + honesty wiring on `main` |
| **G002** | ADR0008-Base | 2x-W0-1 | docs | ADR-0008 merged (fork vs subtree + license + SOURCE_REV) |
| **G003** | W0-Spike | 2x-W0-2, 2x-W0-3 | docs | `GROK_BASE_SPIKE.md` + `cargo check -p xai-grok-pager-bin` evidence |
| **G004** | W1-Integrate | 2x-W1-1 | `2.0.0-alpha.N` | Grok tree integrated per ADR; build story documented/CI |
| **G005** | W1-EntryTUI | 2x-W1-2 | alpha | No-args TTY `dsb` opens full-screen agent (not thin REPL-only) |
| **G006** | W1-BrandAuth | 2x-W1-3, 2x-W1-4 | alpha | DeepSeek branding + setup/auth on new entry; W1 exit |
| **G007** | W2-DeepSeekDefault | 2x-W2-1, 2x-W2-2 | `2.0.0-beta.N` | Default models DeepSeek; provider wired into Grok HTTP path |
| **G008** | W2-EditLoop | 2x-W2-3 | beta | Real-repo edit/tool dogfood; W2 exit |
| **G009** | W3-L1-SnippetPerm | 2x-W3-1, 2x-W3-2 | beta | Snippet + permissions fail-closed under Grok tools + tests |
| **G010** | W3-L2-Prefix | 2x-W3-3 (+ optional 2x-W3-4) | beta | Prefix/epoch discipline tests; REPLAN P0 #4–5; W3 exit |
| **G011** | W4-InstallDocs | 2x-W4-1, 2x-W4-2, 2x-W4-3 | **2.0.0** prep | Install lands agent; docs/npm messaging match 2.0 |
| **G012** | W4-Cut-2.0.0 | 2x-W4-4 | **`2.0.0`** | Tag only with P0 green + release PR |

**P1 may slip past 2.0.0:** skills index thrash-free, Flash/Pro routing polish, DeepSeek blue TUI theme polish (REPLAN §2 P1).

---

## Create plan (if missing on a machine)

```bash
omc ultragoal create-goals --plan-id grokbase-2x --claude-goal-mode aggregate \
  --brief-file docs/product/ULTRAGOAL_BRIEF_2.0.md \
  --goal "G001-ReplanOnMain::REPLAN_2.0 + WAVE_2x_PR_DAG + cold-start 2.0 + SSOT/versioning honesty on main (docs #55 family). Evidence: merge SHA." \
  --goal "G002-ADR0008-Base::ADR-0008 Grok Build base strategy (A fork vs B subtree), Apache-2.0 attribution, SOURCE_REV pin, how dsb binary is produced. Merged on main." \
  --goal "G003-W0-Spike::docs/architecture/GROK_BASE_SPIKE.md: crate map, auth/provider/config plug points; cargo check -p xai-grok-pager-bin on ../grok-build with pass/fail + toolchain notes." \
  --goal "G004-W1-Integrate::Integrate Grok tree per ADR-0008 (fork layout or subtree pin). Tree builds in CI or documented CI plan. SemVer 2.0.0-alpha.N allowed." \
  --goal "G005-W1-EntryTUI::dual bins deepseek-build + dsb entry = Grok pager composition root. No-args TTY opens full-screen coding agent (not thin REPL-only). Evidence: smoke note." \
  --goal "G006-W1-BrandAuth::DeepSeek Build branding (not Grok product name) + first-run setup/auth on new entry (reuse 1.x credentials story, 0600). W1 exit: open agent shell dogfoodable. Ship alpha band." \
  --goal "G007-W2-DeepSeekDefault::Default provider/models = DeepSeek (base URL, model ids). Port/adapt dsb-provider-deepseek or equivalent into Grok HTTP path. Live or recorded chat turn evidence. SemVer 2.0.0-beta.N." \
  --goal "G008-W2-EditLoop::Edit/tool loop works on a real repo via Grok tools (read/edit/bash). Owner-style dogfood note. W2 exit: dsb → chat → real code changes with DeepSeek." \
  --goal "G009-W3-L1-SnippetPerm::Snippet-safe edit policy + permission model (ask/deny/allow; headless fail-closed) under Grok tools. Contract tests ported/adapted from Spec 20/30 family. Evidence: tests + TTY/headless matrix." \
  --goal "G010-W3-L2-Prefix::Prefix/cache epoch discipline (Reasonix L2) under real shell. Tests for stable prefix or documented Grok-equivalent. Optional P1 skills/Flash-Pro may slip. W3 exit: REPLAN §2 P0 items 4–5 green." \
  --goal "G011-W4-InstallDocs::Install path (npm and/or install.sh) produces dsb that opens agent; README/package.json/KNOWN_LIMITS/user-guide rewrite for 2.0 reality; 1.x marked legacy in messaging." \
  --goal "G012-W4-Cut-2.0.0::Tag v2.0.0 ONLY when REPLAN §2 P0 all green. Release PR + tag + CHANGELOG. npm publish human-gated residual OK. Success feeling: dsb opens Grok-class DeepSeek agent."
```

If the plan already exists with these 12 stories, **do not** `--force` recreate (would wipe status). Use `omc ultragoal status --plan-id grokbase-2x`.

---

## Operator loop (until 12/12)

```bash
git fetch origin && git checkout main && git pull origin main
omc ultragoal status --plan-id grokbase-2x
omc ultragoal complete-goals --plan-id grokbase-2x
# Implement active story only; PR units from WAVE_2x_PR_DAG + ULTRAGOAL_PR_PLANNING
# After merge + evidence:
omc ultragoal checkpoint --plan-id grokbase-2x --goal-id <id> --status complete \
  --evidence "PR #…; tests…; commands…" \
  --claude-goal-json '<fresh aggregate /goal snapshot>'
# Immediately complete-goals again — do not idle between stories
```

When **12/12 complete**, product train is done. No next product plan is required for the original owner intent (optional P1 polish can be a later `2.x` plan).

---

## Hard rules

1. **1.x freeze** — no thin-REPL product features as “progress” (REPLAN §5).  
2. **Never tag 2.0.0** before G012 P0 evidence.  
3. **Parent runtime = parent family** (Grok session → grok children).  
4. **npm publish** human-gated (ADR 0007).  
5. **English** on all GitHub public text.  
6. **PR planning first** every story ([ULTRAGOAL_PR_PLANNING.md](./ULTRAGOAL_PR_PLANNING.md)).
