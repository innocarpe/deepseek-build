# Spec 100 — Sessions (minimum)

| Field | Value |
|-------|--------|
| Status | **ready-for-impl** (minimum for Wave A `0.5.0`; expand later) |
| Philosophy | HARNESS session replay + tool-pair repair (spec 15) |
| Gate | **G6a** |
| Tests | Automated required for store/load pairing |

## 1. Behavior (minimum)

1. Persist session as **JSONL** (or equivalent) under `~/.deepseek-build/sessions/` (override via `DEEPSEEK_BUILD_HOME`).  
2. Each line: role, content, optional reasoning_content, tool_calls, tool results, timestamps optional but **not** in stable API prefix.  
3. On load: run **tool-pair repair** (spec 15) before next API call.  
4. CLI: create/resume by id (exact flags as implemented; document in user-guide).  
5. Session id: opaque ulid/uuid; list recent sessions optional for minimum.
6. The `meta` line carries the **prefix shape** of the build that produced the
   transcript (spec 10 §1.5.1 rule 5), so a later process can attribute a
   prefix change for a continuing conversation. It is additive and optional: a
   file written before this field loads normally, and a resume then makes no
   attribution claim rather than guessing from an absent baseline. `meta` is
   never part of the API request.

## 2. Non-goals (minimum)

- Multi-device sync  
- Cloud backup  
- Fork/branch of sessions (later)

## 3. Test plan

| Test | Expect |
|------|--------|
| `roundtrip_jsonl` | write then load equal messages |
| `load_repairs_tool_pairs` | hole → interrupted placeholder |
| `path_under_home` | files only under build home |
| `prefix_snapshot_roundtrip` | meta carries the shape; save→load returns it |
| `legacy_meta_without_snapshot_loads` | pre-§1.5.1 file → no shape, no error |

## 4. Implementation notes

Shipped at product **0.5.0** on `main`; keep this spec as the contract for regressions.
