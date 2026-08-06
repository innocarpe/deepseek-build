# PRD v1 — DeepSeek Build

| Field | Value |
|-------|--------|
| Status | Draft → active for M0–M6 |
| Owner | @innocarpe |
| Last updated | 2026-08-06 |
| Related | [VISION](./VISION.md) · [SOURCES](./SOURCES.md) · [NON_GOALS](./NON_GOALS.md) · [MILESTONES](./MILESTONES.md) |

---

## 1. Problem

Power users already juggle many coding agents (Claude Code, Codex, Grok Build, Reasonix, Deep Code, …). For this product owner:

- **Grok Build** is the wall-clock speed champion (parallel tools, subagents, native runtime).
- **Reasonix** and **Deep Code** are the DeepSeek-native options, each incomplete alone: cache/cost excellence vs official V4 surface (thinking, effort, skills, MCP, permissions).
- Generic multi-vendor harnesses (and heavy multi-stage planners) often feel **slow** and burn progress on ceremony.

There is no single CLI that is simultaneously:

1. DeepSeek V4–native (API knobs + harness habits),  
2. Cache- and cost-disciplined on long sessions, and  
3. Grok-class parallel execution speed.

## 2. Goals

### Primary goal

Ship a **DeepSeek-first terminal coding agent** whose default experience optimizes for **time-to-completed coding task**, not plan-document volume.

### Supporting goals

| ID | Goal | Measure (v1) |
|----|------|----------------|
| G1 | Flash/Pro routing is first-class | Documented presets; `/model` (or equivalent) works |
| G2 | Prefix/KV cache stays warm by design | Spec + telemetry of cache hit (or proxy) on multi-turn sessions |
| G3 | Parallel tool use without blocking the loop | Concurrent independent tools; background shell + wait |
| G4 | Official-surface parity (Deep Code class) | Thinking, effort, Skills, permissions, light plan, sessions |
| G5 | Docs-first open source | Specs before features; milestones; labeled PRs |

### Non-goals (v1)

See [NON_GOALS.md](./NON_GOALS.md). Summary:

- Gajae-style multi-stage planning / team / mobile harness  
- Full Grok hard-fork day one  
- Desktop / VS Code MVP  
- Multi-vendor as identity  

## 3. Users

| Persona | Need |
|---------|------|
| **Solo deep worker** | Long sessions on one repo; cares about cost and uninterrupted progress |
| **Speed-sensitive power user** | Used Grok Build; wants similar parallelism on DeepSeek pricing |
| **DeepSeek-native user** | Already on Reasonix or Deep Code; wants both cache discipline and richer orchestration |

Out of scope for v1 messaging: enterprises needing SSO fleets, multi-tenant gateways.

## 4. Product principles

1. **Progress over ceremony** — Prefer actions that change the tree or produce command evidence.  
2. **DeepSeek harness fit** — Tool schemas and prompts tuned for DeepSeek V4 habits (Deep Code philosophy).  
3. **Cache is a contract** — Stable system/tool/memory prefix; dynamic content on the turn tail (Reasonix).  
4. **Flash by default, Pro on purpose** — Cost and latency stay under control unless escalated.  
5. **Parallel when independent** — Orchestration from Grok patterns without uncached prompt explosions.  
6. **Specs before code** — Behavior contracts live in `docs/specs/`.

## 5. Design sources (binding)

**Layered ownership** (not a single global rank for every decision):

| Layer | Owner | Owns |
|-------|-------|------|
| L1 | Deep Code (+ Reasonix cache) | Tool/edit (snippets), skills-as-context, permissions, DeepSeek-native surface |
| L2 | Reasonix | Prefix cache invariant, Flash/Pro, tool-call repair |
| L3 | Grok Build | Parallelism, subagents, bg shell — **cannot override L1/L2** |

Normative spine: [HARNESS_PHILOSOPHY.md](../architecture/HARNESS_PHILOSOPHY.md).  
Summary table: [SOURCES.md](./SOURCES.md). Historical ADR: [0002](../adr/0002-source-priorities.md) (amended by layered model in philosophy doc).

## 6. Scope — v1 capabilities

### 6.1 Must have (MVP exit ≈ end of M5)

| Capability | Source bias | Spec (planned) |
|------------|-------------|----------------|
| DeepSeek chat + tool loop | Deep Code / Reasonix | 40, 10 |
| Flash default + Pro escalate | Reasonix + Deep Code | 20 |
| Thinking on/off + reasoning effort | Deep Code + API | 30 |
| Cache-stable prefix contract | Reasonix | 10 |
| Core tools: read, edit, shell, search | Grok + Deep Code | 40 |
| Parallel tools + background shell | Grok | 50 |
| Skills discovery (`.agents/skills` + project paths) | Deep Code | 70 |
| Permissions (ask / allow / deny classes) | Deep Code | 90 |
| Sessions: new / resume / fork (or equivalent) | Deep Code | 100 |
| Light plan mode | Deep Code | 110 |
| Subagents (at least explore + implement) | Grok | 60 |
| MCP client | Deep Code | 80 |
| Project surface `.deepseek-build/` | All | 120 |
| Interactive TUI or capable CLI loop | Grok UX bar | — |

### 6.2 Should have (M6 preview)

- Cache hit / cost indicators in UI  
- `/preset flash|balanced|max`  
- Tool-call repair  
- Notify hook after turn  
- Worktree isolation for subagents  
- Headless/CI mode stub  

### 6.3 Could have (post-v1)

- Workflow DAG scripting  
- ACP editor integration  
- VS Code extension  
- Plugin marketplace  

### 6.4 Won’t have (v1)

- Gajae interview/ralplan/ultragoal/tmux teams  
- “Always Pro, max effort” as silent default for all turns  
- Silent YOLO on all shell/file ops as only mode  

## 7. User journeys

### J1 — First run

1. Install (path TBD by toolchain ADR).  
2. Configure DeepSeek API key (wizard or config file).  
3. `cd` project → start agent.  
4. Ask for a small change; tools run; files update.  
5. See model (Flash) and that work progressed without a multi-page plan.

### J2 — Hard task with Pro

1. Default Flash exploration.  
2. User or router escalates to Pro for architecture decision.  
3. Implementation returns to Flash where safe.  
4. Optional light `/plan` produces a short checklist, then execution continues.

### J3 — Long session cost

1. Multi-hour session with stable tools/skills/memory prefix.  
2. Cache remains effective; user can see cost or hit proxy.  
3. Compaction (when added) does not thrash the stable prefix.

### J4 — Parallel investigation

1. User asks for a multi-area bugfix.  
2. Agent launches parallel reads/greps (and later explore subagents).  
3. Results merge; edits apply without serial “one file at a time” stalls.

## 8. UX outline

Inspired by Deep Code slash surface + Grok speed:

| Command / control | Role |
|-------------------|------|
| `/model` | Model, thinking, effort |
| `/plan` | Light planning mode |
| `/new` `/resume` `/fork` | Session lifecycle |
| `/skills` `/mcp` | Extensibility |
| `/pro` or preset | One-shot / session routing |
| Esc / interrupt | Cancel in-flight turn |
| Background shell | Non-blocking long commands |

Exact names may differ; behavior is what ships against specs.

## 9. Technical constraints

| Constraint | Note |
|------------|------|
| API | `https://api.deepseek.com` (OpenAI-compatible where applicable) |
| Models (v1 focus) | `deepseek-v4-flash`, `deepseek-v4-pro` |
| Layout | See [REPO_LAYOUT.md](../architecture/REPO_LAYOUT.md) |
| Language | **Not locked** — ADR required before `crates/` fills |
| License | Apache-2.0 |
| Secrets | Never commit keys; user home config |

## 10. Success metrics (v1 qualitative + light quantitative)

| Metric | Target |
|--------|--------|
| Task progress feel | Maintainer dogfood: multi-file tasks complete without “stuck planning” |
| Cost discipline | Flash default; Pro turns are intentional and visible |
| Cache discipline | Spec 10 implemented; no mid-session rewrite of tool schema prefix |
| OSS hygiene | Labels, milestones, Orca-level PR bodies; PR harness (`skills/pr-authoring`) |
| Spec coverage | MVP specs 10–50, 70, 90, 100 written before their implementation PRs |

Quantitative latency benchmarks are **M6+**, after a runnable agent exists.

## 11. Risks

| Risk | Mitigation |
|------|------------|
| Hard-forking Grok is too large | Slim core; port patterns not the monorepo |
| Parallel subagents kill cache | Shared stable worker prefix; Flash workers; summary merge |
| Scope creep from other agents | NON_GOALS + milestone gate |
| Language churn | One toolchain ADR; freeze for MVP |
| DeepSeek API changes | Provider crate isolation; pin documented models |

## 12. Milestones

Authoritative list: **[MILESTONES.md](./MILESTONES.md)** (mirrored as GitHub Milestones).

| ID | Name | Outcome |
|----|------|---------|
| M0 | Product foundation | Repo, OSS kit, PRD, specs index, labels |
| M1 | Provider + cache + routing | Minimal DeepSeek loop + Flash/Pro + cache contract |
| M2 | Core tools + parallelism | Productive single-agent coding |
| M3 | DeepSeek surface | Thinking, effort, skills, permissions |
| M4 | Subagents + worktree | Grok-class fan-out |
| M5 | Sessions + plan + MCP | Deep Code-class product surface |
| M6 | Preview polish | Dogfood release candidate |

## 13. Open decisions (need ADRs)

1. Implementation language / toolchain (Rust vs Go vs other)  
2. Binary / package name (`dsb` vs `deepseek-build` vs other)  
3. Config path schema (`~/.deepseek-build/` vs compatibility with Deep Code settings)  
4. Compaction strategy vs cache stability  
5. Whether worktree isolation is M4 must or should  

## 14. Launch definition — “PRD v1 done”

v1 is **accepted for implementation** when:

- [x] Public GitHub repo with OSS scaffolding  
- [x] This PRD + milestones published  
- [ ] Toolchain ADR accepted  
- [ ] Specs 10, 20, 30, 40, 50 drafted to “ready for impl” quality  
- [ ] M1 implementation started on `main` or versioned branch  

---

## Changelog

| Date | Change |
|------|--------|
| 2026-08-06 | Initial PRD v1 from product design discussions |
