# Spec 60 — Subagents (+ worker cache law)

| Field | Value |
|-------|--------|
| Status | **ready-for-impl** |
| Philosophy | HARNESS L3 under L2: workers share stable template; no unique cold prefixes |
| Gate | **G5** |
| Tests | **Automated required:** worker cache law, explore read-only, invalidate snippets on path touch |

## 1. Behavior

### 1.1 Worker kinds (minimum)

| Kind | Tools | Default model |
|------|-------|---------------|
| `explore` | read-only: `read`, `grep`, `skill` | Flash |
| `implement` | full built-ins under parent policy | Flash (Pro optional later) |

### 1.2 Worker cache law (normative)

1. Workers **must** reuse the same **stable prefix template** as the parent (system + tool schemas + skills index + env summary + project instructions) when inputs are unchanged.  
2. Workers must **not** invent unique system prompts that thrash the cache epoch.  
3. Worker volatile tails are isolated (their own tool transcripts).  
4. When an implement worker mutates paths, parent **snippet table expires** for touched paths (or all workspace snippets — product default: expire all).

### 1.3 Spawn API (tool)

Built-in tool **`subagent`**:

| Arg | Required | Notes |
|-----|----------|-------|
| `kind` | yes | `explore` \| `implement` |
| `task` | yes | Natural language task for the worker |
| `max_rounds` | no | Default 4 |

Returns JSON: `{ ok, kind, summary, tool_rounds, mutated }`.

v1 may run the worker **in-process** synchronously (no OS process fork). Worktree isolation is optional later.

### 1.4 Permissions

- Explore workers: policy forces write/delete/bash deny.  
- Implement workers: inherit parent policy (including headless/grants).  
- Parent still owns user-facing ask prompts.

## 2. Non-goals

- Nested subagent trees of unbounded depth  
- Multi-vendor worker fleets  
- Mandatory worktree for every implement worker  

## 3. Test plan

| ID | Case | Expect |
|----|------|--------|
| T1 | explore cannot write | write denied |
| T2 | cache law epoch | parent and worker stable epoch equal for same inputs |
| T3 | implement mutates → parent snippets expire | snippet gone after worker write |
| T4 | unknown kind | structured error |

## 4. Ready-for-impl checklist

- [x] Worker kinds  
- [x] Cache law  
- [x] Spawn tool  
- [x] Tests listed  

**Status:** **ready-for-impl**.
