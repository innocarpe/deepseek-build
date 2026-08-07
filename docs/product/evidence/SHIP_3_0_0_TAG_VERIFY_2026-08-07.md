# Ship 3.0.0 — G001 tag/main verify

**Date:** 2026-08-07  
**Plan:** `ship-3.0.0`

## Result: ALIGNED

| Ref | Commit |
|-----|--------|
| `origin/main` | `bcb92d2969cf9ef1c7a071cde8b4ff5f3023a661` |
| `v3.0.0` (peeled) | `bcb92d2969cf9ef1c7a071cde8b4ff5f3023a661` |
| Annotated tag object | `037487b5dfd188cbd7d36b1695accf1f17b22eeb` |

```bash
git fetch origin --tags
test "$(git rev-parse origin/main)" = "$(git rev-parse v3.0.0^{})"
```

Merge subject: `Merge pull request #83 from innocarpe/release/3.0.0`  
On-disk SemVer at that commit: **3.0.0**

**Action taken:** none (tag already correct on remote).
