# Design sources

Sources are **layered**, not a single global rank for every decision.  
Full conflict rules: [HARNESS_PHILOSOPHY.md](../architecture/HARNESS_PHILOSOPHY.md).

## Layer ownership

| Layer | Owner sources | Owns |
|-------|---------------|------|
| **L1** DeepSeek-native contracts | **Deep Code** (primary), Reasonix (cache co-owner) | Tool/edit shape, skills-as-context, side-effect permissions, official CLI surface habits |
| **L2** Cost & session economics | **Reasonix** (primary), Deep Code session layout | Prefix cache invariant, Flash/Pro, tool-call repair, long-session cost |
| **L3** Execution throughput | **Grok Build** (primary) | Parallel tools, background shell, subagents, worktrees, native tool speed |

### Hard rule

**L3 never overrides L1/L2.** Parallelism and subagents must obey cache stability, snippet edit safety, and permission honesty.

---

## 1. Deep Code CLI (L1 primary)

**Upstream:** https://github.com/lessweb/deepcode-cli  
**Architecture:** https://github.com/lessweb/deepcode-cli/blob/main/docs/architecture_en.md  
**DeepSeek docs:** agent integration “Deep Code”

### Philosophy we adopt

- Harness is **adapted to DeepSeek**, not “compatible enough” multi-vendor glue.  
- Tool schemas are **not neutral** (model habits matter).  
- Four pillars: **snippet edit repair**, **cache-aware context**, **skills as structured context**, **side-effect permissions**.  
- Small predictable built-in tool set; MCP dynamic.  
- Goal shape: better results than generic CLI + DeepSeek, at lower cost.

### Take / leave

| Take | Leave (for now) |
|------|------------------|
| Snippet_id edit contract | Node/TypeScript stack requirement |
| Cache-aware session layout + JSONL replay intent | VS Code parity day one |
| Skills index vs body; on-demand load | Every slash name identical forever |
| Side-effect scopes; bash declares effects | Multivendor Coding Plan as identity |
| Thinking + reasoning effort UX | Star-count chasing |

---

## 2. Reasonix (L2 primary; L1 cache co-owner)

**Local:** `../../../DeepSeek-Reasonix`  
**Upstream:** https://github.com/esengine/DeepSeek-Reasonix

### Philosophy we adopt

- **Cache-first** is an invariant: system + tools + standing memory prefix stays byte-stable across turns.  
- Dynamics ride the **turn tail**.  
- **Flash-first**, Pro escalate.  
- **Tool-call repair** for messy args.  
- Long sessions must stay **economically leave-on**.

### Take / leave

| Take | Leave (for now) |
|------|------------------|
| Prefix stability contract | Desktop / full plugin surface |
| Flash/Pro + effort culture | Every TOML dial |
| Tool-call repair | Multi-provider as primary identity |
| Cost/cache telemetry mindset | |

---

## 3. Grok Build (L3 primary)

**Local:** `../../../grok-build`

### Philosophy we adopt

- Wall-clock progress: parallel tools, background work, subagents, isolation.  
- Native-speed local tooling.  
- Modular package boundaries.

### Take / leave

| Take | Leave (for now) |
|------|------------------|
| Parallel tools, bg shell, multi-wait | Full monorepo hard-fork |
| Subagents + worktree isolation | xAI auth/telemetry/branding |
| Workflow-style fan-out patterns (later) | Tool shapes that break L1 DeepSeek fit |
| Hashline/anchors **only if** mapped to snippet semantics | Replacing snippet contract wholesale |

---

## Explicitly deferred

### Gajae-code

**Local:** `../../../gajae-code`

**v1 does not import:** deep-interview → ralplan → ultragoal, tmux teams, mobile answer loops as core surface.

Reason: wall-clock progress too poor vs north star; multi-stage planning stalls.

---

## Document precedence

```text
HARNESS_PHILOSOPHY (spine)
  > VISION + NON_GOALS
  > adr/ that does NOT claim supersession
  > specs/  (must cite philosophy sections; make contracts executable)
  > research/  (non-binding evidence)
  > “looks cool in another tool”
```

**Exception:** An ADR that **explicitly supersedes named HARNESS_PHILOSOPHY sections** outranks those sections only. See philosophy §1 amendment path.  
**Gates ledger:** [GATES.md](../GATES.md) is the authority for green/red implementation gates.

## Research notes

- [deepcode-cli.md](../research/deepcode-cli.md)  
- [reasonix.md](../research/reasonix.md)  
- [grok-build.md](../research/grok-build.md)  
