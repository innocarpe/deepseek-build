# G003 MintFileVersion — evidence (owner-bar-5x)

| Field | Value |
|-------|--------|
| **Story** | G003 MintFileVersion (`G003-g003-mintfileversion`) |
| **WAVE unit** | **5x-H1-0** |
| **Date** | 2026-08-07 |
| **Base SHA** | `10b035c` (after G002) + this PR |
| **SemVer on disk** | `4.0.2` — not owner-bar complete |

## Done criteria

| Check | Result |
|-------|--------|
| `read_file` mints `file_version` = sha256(full file bytes) | **PASS** (unit + Path A wire) |
| Model-visible tool result includes `file_version: <hex>` | **PASS** |
| Path A public entry + scripted DeepSeek wire | **PASS** |
| `check-path-a-linkage` **NO_MINT** cleared | **PASS** (only DEAD_WIRING remains → G004) |
| No snippet_safe flip in this story | **PASS** (mint only) |

## What shipped

1. **`FileContent.file_version: Option<String>`** on tool output.
2. **Grok `read_file`** (Path A implementation) hashes file bytes with sha256 at read time.
3. **`to_prompt_format`** appends `file_version: <hex>` so the model / wire sees the token.
4. Unit test `current_read_file_mints_file_version_sha256`.
5. Scripted server improvements for read-file tool scenarios (`target_file` args, session-title skip).

## Commands

```bash
# Unit (crate)
cd third_party/grok-build
cargo test -p xai-grok-tools current_read_file_mints_file_version_sha256
# ok

# Linkage: NO_MINT gone
./scripts/check-path-a-linkage.sh
# DEAD_WIRING only (expected until G004)

# Path A R0A (public CLI + scripted server + rebuilt agent with mint)
# Agent: third_party/grok-build/target/debug/xai-grok-pager (built this session)
# → wire tool result contains:
#   file_version: 96b458e0a2bf038bf983342fd79e077feab262ae0d660d5f3846de99091c344b
# matching sha256(mint.txt)
```

## Artifacts

| Path | Role |
|------|------|
| [`PATH_A_R0_MINT_WIRE_last.jsonl`](./PATH_A_R0_MINT_WIRE_last.jsonl) | Slimmed wire with tool result |
| [`PATH_A_R0_MINT_META_last.txt`](./PATH_A_R0_MINT_META_last.txt) | Expected hash + exit |

## Explicit non-claims

- Does **not** enable `snippet_safe` default (G004).
- Does **not** green owner-bar aggregator.
- Install-path SIGKILL of product `~/.deepseek-build/bin/deepseek-build-agent` still noted (use hermetic copy / rebuilt binary).

## Next

**G004 SnippetLive** — apply snippet_safe on Standard toolset + dead-guard fix + negatives + liveness L1-45-0.
