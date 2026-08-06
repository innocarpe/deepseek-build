# PRD v1 — DeepSeek Build

| Field | Value |
|-------|--------|
| Status | Draft → active for M0–M6 |
| Owner | @innocarpe |
| Last updated | 2026-08-06 |
| Related | [MASTER_PLAN](./MASTER_PLAN.md) · [VISION](./VISION.md) · [SOURCES](./SOURCES.md) · [NON_GOALS](./NON_GOALS.md) · [MILESTONES](./MILESTONES.md) · [staged PRDs](./prd/) |

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
| G2 | Prefix/KV cache stays warm by design | Spec 10 golden prefix bytes **and** provider cache evidence when API exposes it (else documented substitute: dual-call cost/latency protocol in provider ADR) |
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

| Capability | Source bias | Spec (planned) | Milestone |
|------------|-------------|----------------|-----------|
| DeepSeek chat + streaming provider | Deep Code / Reasonix | provider + 40 slice | M1 |
| Cache-stable prefix + golden bytes | Reasonix + Deep Code B | 10 | M1 |
| Tool-call / tool-result repair | Reasonix | 15 | **M1 (must)** |
| Flash default + Pro escalate (provider + flags) | Reasonix + Deep Code | 20 | M1 |
| Thinking + effort **API wiring** | Deep Code + API | 30 | M1 (after G1b) |
| Thinking + effort **UX** (`/model`, presets polish) | Deep Code | 30 UX | M3 |
| Snippet-scoped edit (+ safe write) | Deep Code A | 45 | M2 |
| Core tools: read, write, edit, shell, search | Deep Code set + L3 speed | 40 | M2 |
| **Minimum** side-effect permissions (shell/file) | Deep Code D | 90 min | **M2 (with tools)** |
| Parallel tools + background shell | Grok L3 | 50 | M2 |
| Full permissions polish | Deep Code D | 90 | M3 |
| Skills as structured context | Deep Code C | 70 | M3 |
| Sessions: new / resume / fork | Deep Code | 100 | M5 |
| Light plan mode | Deep Code | 110 | M5 |
| Subagents (+ worker cache law) | Grok under L2 | 60 | M4 |
| MCP client (+ schema epoch rules) | Deep Code | 80 | M5 |
| Project surface `.deepseek-build/` | All | 120 | M3 |
| Interactive TUI or capable CLI loop | Grok UX bar | — | M2–M5 |

### 6.2 Should have (M6 preview)

- User-visible cache hit / cost indicators in TUI (M1 still requires provider-side evidence per G2 — not “proxy only”)  
- `/preset flash|balanced|max` polish  
- Notify hook after turn  
- Worktree isolation for subagents  
- Headless/CI **product** test suite  
- Adversarial acceptance suite expansion

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
2. Cache remains effective; user can see cost / cache evidence (not proxy-only).  
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
| Models (v1 focus) | `deepseek-v4-flash`, `deepseek-v4-pro` ([ADR 0005](../adr/0005-deepseek-provider-contract.md)) |
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

## 14. Launch definition — gates before runtime code

See also [HARNESS_PHILOSOPHY §11](../architecture/HARNESS_PHILOSOPHY.md).

**PRD / foundation complete:**

- [x] Public GitHub repo with OSS scaffolding  
- [x] This PRD + milestones published  
- [x] Harness philosophy (L1/L2/L3 + Deep Code four pillars)  

**M1 code may start only when ([GATES.md](../GATES.md)):**

- [x] **G0** Harness philosophy  
- [x] **G1** Toolchain ADR (`docs/adr/0004-toolchain.md`) — green on merge of preflight PR  
- [x] **G1b** Provider contract ADR (`docs/adr/0005-deepseek-provider-contract.md`)  
- [x] **G2** Specs **10, 15, 20, 30** ready-for-impl  

**Forbidden:** opening `feat` work that implements M2+ (snippet edit, shell, parallel) without G3+.

---

## Changelog

| Date | Change |
|------|--------|
| 2026-08-06 | Initial PRD v1 from product design discussions |
| 2026-08-06 | Layered sources; tool-call repair → M1 must; gates before code; Codex adversarial review amendments |
