# Spec 110 — Light plan mode

| Field | Value |
|-------|--------|
| Status | **ready-for-impl** |
| Philosophy | HARNESS §8 — plan unblocks execution; not Gajae multi-stage religion |
| Gate | **G6d** |
| Tests | **Automated required:** update/list plan, non-blocking, session-local |

## 1. Behavior

### 1.1 Intent

Provide a **light, non-blocking** checklist the model can update while continuing to execute tools. Plan must **not** stall the agent loop waiting for human interview stages.

### 1.2 Tool surface

Built-in tool **`plan`** (canonical name; advertised in tool definitions when plan product is enabled):

| Arg | Required | Type | Notes |
|-----|----------|------|-------|
| `action` | **yes** | string | `get` \| `set` \| `add` \| `complete` \| `clear` |
| `items` | no | string[] | For `set` / `add` |
| `index` | no | integer | 0-based for `complete` |

### 1.3 Semantics

| Action | Behavior |
|--------|----------|
| `get` | Return current checklist JSON |
| `set` | Replace entire checklist with `items` (all open) |
| `add` | Append `items` as open |
| `complete` | Mark `index` complete (idempotent) |
| `clear` | Empty checklist |

Each item:

```json
{ "text": "…", "done": false }
```

### 1.4 Storage

- **Session-local** in-process by default (volatile with the agent).  
- Optional: persist under session JSONL metadata later — not required for G6d.  
- Plan content is **volatile** (not in stable prefix). Updating plan must **not** change prefix epoch.

### 1.5 Non-blocking

- `plan` tool returns immediately.  
- No human confirmation gate on plan updates.  
- Execution of other tools continues in the same turn chain without waiting on plan approval.

### 1.6 UX (CLI optional)

Product may print a short plan summary on `set`/`complete` via turn events; not required for gate.

## 2. Non-goals

- Multi-stage ralplan / ultragoal orchestration as product identity  
- Blocking “must plan before code” modes as default  
- Shared multi-user plan servers  

## 3. Test plan

| ID | Case | Expect |
|----|------|--------|
| T1 | set + get | items round-trip |
| T2 | complete index | done=true |
| T3 | clear | empty |
| T4 | invalid index | structured error, no panic |
| T5 | not in stable prefix | plan text absent from prefix builder |

## 4. Implementation map

| Area | Location |
|------|----------|
| Plan store + tool | `crates/dsb-tools` |
| Gate | `docs/GATES.md` **G6d** |

## 5. Ready-for-impl checklist

- [x] Tool args  
- [x] Non-blocking semantics  
- [x] Volatile storage  
- [x] Automated tests listed  

**Status:** **ready-for-impl**.
