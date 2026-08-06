# Spec 50 — Parallelism & background (tools)

| Field | Value |
|-------|--------|
| Status | **ready-for-impl** |
| Philosophy | HARNESS L3 Grok throughput under L1/L2 (snippet + permission honesty) |
| Gate | **G4** |
| Tests | **Automated required:** independence classify, parallel read-only, serial mutate |

## 1. Behavior

### 1.1 Parallel independent tools (one model turn)

When the model returns **multiple** tool calls in one assistant message:

1. Classify each call as **read-only** or **mutating**.  
2. **Read-only** tools may run **concurrently**.  
3. **Mutating** tools run **serially** (after concurrent reads in that turn, or entirely serial if mixed with unsafe interleaving).  
4. Partial failure: one tool error must not drop sibling results already obtained; each call still produces a tool_result message.

### 1.2 Read-only vs mutating (minimum)

| Tool | Class |
|------|--------|
| `read`, `grep`, `skill`, `plan` (get) | read-only |
| `edit`, `write`, `bash` (any), `plan` (set/add/complete/clear), MCP | mutating (fail-closed) |

MCP and bash are always treated as mutating for scheduling (side effects unknown).

### 1.3 Snippet / permission honesty (L1)

- Parallel **reads** may use isolated snippet issue tables per worker **or** a synchronized store; product must not apply edits against a stale sibling snippet without version check (spec 45).  
- Permissions still run **per tool** before execution (spec 90).  
- Parallelism must **not** skip deny/ask.

### 1.4 Background shell (preview; full product **0.13.0**)

Spec 50 **documents** background shell + collect-by-id. Runtime may land in **0.13.0**. Minimum here:

| Concept | Meaning |
|---------|---------|
| `bash` with `background: true` | Start job, return `job_id` without waiting for full stdout |
| `bash_collect` / collect tool | Fetch stdout/stderr by `job_id` |

### 1.5 Cancel / partial failure

- Cancelling a turn should best-effort abort in-flight parallel reads.  
- Mutating tools that already applied are **not** auto-rolled back (document honesty).

## 2. Non-goals

- Spec 60 subagents (G5)  
- Worktree workers  
- Unlimited thread fan-out (cap product default, e.g. 8 concurrent)

## 3. Test plan

| ID | Case | Expect |
|----|------|--------|
| T1 | All-read batch | Concurrent path used; all results present |
| T2 | Mixed edit+read | Mutating serial; no lost results |
| T3 | One read fails | Sibling reads still return |
| T4 | Independence classifier | edit/write/bash → mutating |

## 4. Implementation map

| Area | Location |
|------|----------|
| Classifier + runner | `crates/dsb-agent` parallel tools |
| Gate | `docs/GATES.md` **G4** |

## 5. Ready-for-impl checklist

- [x] Independence rules  
- [x] L1 snippet/permission constraints  
- [x] Partial failure  
- [x] Automated tests listed  

**Status:** **ready-for-impl**.
