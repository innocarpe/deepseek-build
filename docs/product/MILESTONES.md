# Development milestones

These map 1:1 to **GitHub Milestones** on the repository.  
Issues and PRs should set a milestone whenever possible (`milestone-aligned` label optional).

North star for every milestone: **wall-clock progress on real coding tasks**.

---

## M0 — Product foundation

| | |
|--|--|
| **Goal** | Repo is a credible open-source project; product truth is written |
| **Exit criteria** | Public `main`; LICENSE/NOTICE/CONTRIBUTING/SECURITY/CoC; labels; issue/PR templates; PR process harness; PRD-v1 + this file; docs tree live |
| **Primary docs** | `docs/product/*`, `docs/architecture/REPO_LAYOUT.md`, ADR 0001–0002 |
| **Not in M0** | Runnable agent binary |

**Work items (examples)**

- [x] Scaffold `docs/`, `crates/`, skills surface  
- [x] OSS kit + labels catalog  
- [x] PRD-v1 + milestones  
- [x] Sync labels to GitHub  
- [x] GitHub Milestones M0–M6 created  
- [ ] Optional: close M0 on GitHub when CI is green  


---

## M1 — Provider + cache + routing

| | |
|--|--|
| **Goal** | Minimal DeepSeek-native loop that is already cheaper/faster to leave running than a naive client |
| **Exit criteria** | Headless or TUI-thin loop: user message → model → (optional tools stub) → response; Flash default; Pro escalate; cache contract documented **and** implemented for system/tools prefix stability; API key config without committing secrets |
| **Specs** | `10-cache-contract`, `20-model-routing`, `30-thinking-effort` (draft→ready), provider bits of `40` |
| **Sources** | Reasonix (cache, Flash/Pro), Deep Code (thinking/effort knobs), Grok (loop shape only) |

**Work items (examples)**

- [ ] Toolchain ADR (language, package name)  
- [ ] Specs 10 / 20 / 30 ready  
- [ ] DeepSeek provider client (streaming)  
- [ ] Stable prefix builder + tests (“byte-stable across turns”)  
- [ ] Flash/Pro switch + effort flags  
- [ ] Smoke: one multi-turn session with measurable prefix reuse intent  

---

## M2 — Core tools + parallelism

| | |
|--|--|
| **Goal** | Agent can complete small–medium repo tasks without subagents |
| **Exit criteria** | Tools: read, search/grep, edit, shell; parallel independent tool calls in one turn; background shell + collect output; project instructions (`AGENTS.md` or equivalent) loaded into **cache-safe** slots |
| **Specs** | `40-tools`, `50-parallelism` |
| **Sources** | Grok Build (parallel + bg), Deep Code (tool set pragmatism) |

**Work items (examples)**

- [ ] Specs 40 / 50 ready  
- [ ] Tool runtime + permission hooks stub  
- [ ] Parallel dispatch + ordering of results  
- [ ] Background shell task IDs  
- [ ] Dogfood: implement a small feature in this repo using the agent  

---

## M3 — DeepSeek surface (thinking, skills, permissions)

| | |
|--|--|
| **Goal** | Feel like an official-class DeepSeek CLI, not a generic OpenAI wrapper |
| **Exit criteria** | Thinking mode + effort exposed; Skills discovery (Deep Code–compatible paths); permission policies for shell/file/network; `/model` (or equivalent) UX |
| **Specs** | `30` complete, `70-skills`, `90-permissions`, config parts of `120` |
| **Sources** | Deep Code primary; Reasonix for not breaking cache when skills index is stable |

**Work items (examples)**

- [ ] Skills loader + index line in stable prefix  
- [ ] Permission engine (ask/allow/deny classes)  
- [ ] Thinking/effort wiring verified against DeepSeek API docs  
- [ ] User-facing docs draft in `docs/user-guide/` (install, auth, model)  

---

## M4 — Subagents + worktree

| | |
|--|--|
| **Goal** | Grok-class fan-out without destroying cost model |
| **Exit criteria** | Spawn explore (read-only) and implement (write) subagents; parent continues; results summarized; **cache strategy for workers documented**; optional worktree isolation for write workers |
| **Specs** | `60-subagents` (+ worktree note in architecture) |
| **Sources** | Grok Build; Reasonix cache caveats |

**Work items (examples)**

- [ ] Spec 60 ready (include cache rules for children)  
- [ ] Subagent lifecycle + cancel  
- [ ] Flash-default workers; Pro optional for review worker  
- [ ] Worktree backend (may be MVP-simple: git worktree)  
- [ ] Dogfood: parallel explore on a mid-size codebase  

---

## M5 — Sessions + plan + MCP

| | |
|--|--|
| **Goal** | Product-complete Deep Code–class surface on top of fast core |
| **Exit criteria** | Session new/resume/fork; light plan mode that **does not** block execution indefinitely; MCP client; notify hook optional |
| **Specs** | `100-sessions`, `110-plan-mode`, `80-mcp` |
| **Sources** | Deep Code |

**Work items (examples)**

- [ ] Session store under user state dir  
- [ ] `/plan` checklist → continue agent loop  
- [ ] MCP config + tool merge without thrashing cache (schema stability rules)  
- [ ] End-to-end demo script / recording  

**MVP product bar:** M1–M5 exit criteria met → **v1 feature-complete candidate**.

---

## M6 — Preview polish

| | |
|--|--|
| **Goal** | Dogfoodable public preview |
| **Exit criteria** | Install path documented; changelog; known-limitations; cost/cache UI hints; basic regression smoke in CI; tagged prerelease |
| **Sources** | All |

**Work items (examples)**

- [ ] `docs/user-guide` filled for shipped commands  
- [ ] Performance / cost notes  
- [ ] Prerelease tag + GitHub Release notes  
- [ ] Issue triage backlog groomed  

---

## Milestone hygiene

1. Every implementation PR sets a **GitHub Milestone**.  
2. Spec PRs should land **in the same milestone** as the code they unlock (or the previous one).  
3. If work slips, **move the issue** — do not silently expand exit criteria.  
4. Expanding v1 scope requires an ADR + PRD changelog entry.

## Dependency graph

```text
M0 ──► M1 ──► M2 ──► M3 ──► M4 ──► M5 ──► M6
              │       │       │
              │       └───────┴── specs may draft early in M0–M1
              └── tools before subagents (M2 before M4)
```

Parallelism allowed:

- Spec writing for M3–M5 during M1–M2  
- Research clones (deepcode-cli) anytime under `docs/research/`
