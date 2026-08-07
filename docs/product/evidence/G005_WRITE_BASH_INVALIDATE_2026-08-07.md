# G005 WriteBashInvalidate — evidence (owner-bar-5x)

| Field | Value |
|-------|--------|
| **Story** | G005 WriteBashInvalidate |
| **WAVE** | **5x-H1-2** |
| **Date** | 2026-08-07 |
| **Depends** | G003 mint + G004 snippet_safe wiring |

## Done criteria

| ID | Check | Result |
|----|-------|--------|
| **L1-45-7** | Empty-old `search_replace` (write spirit) cannot overwrite existing non-empty file when `empty_old_string_does_not_override` is on | **PASS** Path A R0A — content stayed `keep-me` |
| **L1-45-8** | Bash mutation invalidates outstanding `file_version` for touched path | **PASS** Path A R0A — `snippet_stale` after `run_terminal_command` mutate |

## Behavior (Path A)

Grok product edit path uses `search_replace` (not a separate free write tool):

- **Write overwrite** = empty `old_string` + existing non-empty file → rejected when Standard tool configs apply (`empty_old_string_does_not_override: true` from G004).
- **Bash invalidation** = content-hash versions: after bash rewrites file bytes, prior `file_version` fails `snippet_stale` (no separate version table required).

## Commands

```bash
# Unit (already on tree)
cargo test -p xai-grok-tools file_already_exists_nonempty snippet_safe_stale

# Path A R0A (public deepseek-build agent + scripted server)
# write-deny scenario → WRITE_DENY_PASS
# bash-stale scenario → BASH_STALE_PASS (tool: snippet_stale …)
```

## Artifacts

- `PATH_A_R0_G005_write-deny_WIRE_last.jsonl`
- `PATH_A_R0_G005_bash-stale_WIRE_last.jsonl`

## Wire excerpts

**bash-stale tool result:**

```text
snippet_stale: file_version does not match current file content; re-read before edit
```

File on disk after bash: `mutated-by-bash` (stale edit did not apply `should-fail`).

**write-deny:** existing.txt remained `keep-me` (overwrite blocked).

## Explicit non-claims

- Permissions matrix G006, repair G007 not claimed here.
