# Development milestones

These map 1:1 to **GitHub Milestones** on the repository.  
Issues and PRs should set a milestone whenever possible (`milestone-aligned` label optional).

North star for every milestone: **wall-clock progress on real coding tasks**  
**without** violating L1/L2 harness invariants ([HARNESS_PHILOSOPHY](../architecture/HARNESS_PHILOSOPHY.md)).

**SemVer release train (how we ship):** stay on **`0.y.z`** until dogfood-usable —  
see [**RELEASE_TRAIN_0x.md**](./RELEASE_TRAIN_0x.md). Do not treat **`1.0.0`** as the near finish line.

**Invariant gates (every L3-heavy milestone must restate):**

- Snippet edit safety **before any edit tool ships** (no conditional dodge)  
- Cache byte-stability / epoch rules **before M1 claims cache success**  
- Side-effect permissions **before shell mutates the tree**  
- Worker cache law **before subagent fan-out**  

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
| **Specs** | `10` cache, `15` tool-call repair, `20` routing, `30` thinking/effort API (+ provider slice of `40` **read-only tools only**) |
| **Sources** | L1/L2 first; Grok only for loop shape |
| **Gate** | **G0+G1+G1b+G2** — philosophy; toolchain ADR; **provider contract ADR**; specs 10/15/20/30 ready-for-impl ([GATES.md](../GATES.md)) |
| **Failure if** | Implementation PRs land with TODO specs; golden prefix **without** cache evidence protocol; repair deferred to “later” |

**Work items (examples)**

- [x] Harness philosophy doc (Deep Code four pillars + Reasonix + Grok L3)  
- [x] Toolchain ADR (0004)  
- [x] Provider contract ADR (0005)  
- [x] Specs 10 / 15 / 20 / 30 **ready-for-impl**  
- [x] DeepSeek provider client (streaming)  
- [x] Stable prefix builder + **golden byte tests**  
- [x] Tool-call repair on provider path (**M1 must**, not M6)  
- [x] Flash/Pro + effort **flags** (API); polished `/model` UX can wait for M3  
- [x] Smoke: multi-turn session with **golden prefix equality and** cache evidence protocol (per provider ADR; not golden-only)  

**Not in M1:** full snippet edit, parallel fan-out, subagents, MCP, full TUI polish.

---

## M2 — Core tools + parallelism

| | |
|--|--|
| **Goal** | Agent can complete small–medium repo tasks without subagents |
| **Exit criteria** | Tools: read, search/grep, edit, shell; parallel independent tool calls in one turn; background shell + collect output; project instructions (`AGENTS.md` or equivalent) loaded into **cache-safe** slots |
| **Specs** | **`45` snippet-edit**, **`90` minimum permissions**, then `40`, then `50` |
| **Sources** | Deep Code A+D first; Grok L3 only after L1 tool/perm contracts exist |
| **Gate** | **G3** (45 + min 90) before mutating tools/shell; **G4** (50) before parallel dispatch |
| **Failure if** | Free-form whole-file edit as primary path; YOLO shell; parallel tools without 50 |

**Work items (examples)**

- [x] Spec 45 snippet edit ready (before free-form edit)  
- [x] Spec 90 **minimum** (path scopes + bash side-effect declare + ask/deny) — **before** shell  
- [ ] Specs 40 / 50 ready  
- [ ] Tool runtime implementing snippet contract + write bypass law  
- [ ] Parallel dispatch + ordering / cancel / partial failure (50)  
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
