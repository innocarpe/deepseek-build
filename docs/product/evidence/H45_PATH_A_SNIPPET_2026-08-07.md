# H45 Path A snippet-safe edit — G004 evidence

**Date:** 2026-08-07  
**Band:** `3.0.0-alpha.1`  
**Story:** G004 L1-Snippet · WAVE `3x-H1-1`  
**Binding:** [HEART_3X_SPEC_BINDING.md](../../architecture/HEART_3X_SPEC_BINDING.md)  
**Cases:** [HEART_3X_P0_TEST_PLAN.md](../HEART_3X_P0_TEST_PLAN.md) H45.*

## What shipped

| Layer | Change |
|-------|--------|
| Product contract | `crates/dsb-tools/src/path_a_edit.rs` — Grok-shaped request gate; free-form primary rejected when `require_snippet` |
| Grok edit path | `SearchReplaceParams.snippet_safe` + `SearchReplaceInput.file_version`; fail closed without version / on stale hash |
| Default agent toolset | Standard file toolset injects `snippet_safe=true` + `empty_old_string_does_not_override=true` |

## Test evidence

```bash
cargo test -p dsb-tools path_a
# vendor (from third_party/grok-build):
cargo test -p xai-grok-tools snippet_safe
cargo test -p dsb-tools   # H45.5 thin regression
```

| Case | Result |
|------|--------|
| H45.1 free-form without snippet | PASS (`path_a` + vendor without `file_version`) |
| H45.2 empty-old overwrite | PASS (path_a + snippet_safe forces guard) |
| H45.3 unique / ambiguous | PASS (path_a) |
| H45.4 stale version | PASS (path_a + vendor stale `file_version`) |
| H45.5 thin dsb-tools suite | PASS (`cargo test -p dsb-tools`) |

## Honest limits

- Session **snippet_id table inside Grok read_file** is not yet full Spec 45 mint/issue; Path A uses **`file_version` (sha256) as Spec 45 equivalent** per binding map, plus product `SnippetStore` adapter for tests/port.
- Model must pass `file_version` after read when `snippet_safe` is on; description schema field documents this.
- Dogfood live agent edit under DeepSeek is G005/H1.3 territory for permissions; this story is contract + tool path gate.

## SemVer

Product workspace **`3.0.0-alpha.1`** (Cargo + package.json).
