# Ultragoal chain — overnight continuity

**Purpose:** When one wave plan completes, the next starts without re-deriving the final goal.  
**Major lines:** [versions/README.md](./versions/README.md) · [PRD-v4.md](./PRD-v4.md)  
**Scaffold history board:** [MASTER_PLAN.md](./MASTER_PLAN.md)

---

## Active product chain

**No active product ultragoal train.** Latest major cut: **`v4.0.0`** (L3). On-disk / npm: read root SemVer (may be `4.0.x` patches).

### Completed trains

| Plan id | Role |
|---------|------|
| **`fleet-4x`** | 4.0.0 L3 productization — **complete** (tag `v4.0.0`, PR #85; install/UX `4.0.1`/`4.0.2`) |
| **`heart-3x`** | 3.0.0 heart fusion — **complete** (tag `v3.0.0`) |
| **`ship-3.0.0`** | 3.0.0 ship closeout (tag/release/npm verify) — **complete** |
| **`l3-prep-lane-b`** | Parallel L3 prep during 3.0 — **complete** |
| **`grokbase-2x`** | 2.x Grok base + DeepSeek shell |

Cut evidence: [CUT_4_0_0_2026-08-07.md](./evidence/CUT_4_0_0_2026-08-07.md) · [CUT_3_0_0_2026-08-07.md](./evidence/CUT_3_0_0_2026-08-07.md)

---

## Historical scaffold chain (archive)

| Plan id | Wave |
|---------|------|
| `dogfood-0x` / `native-0x` / `throughput-0x` / `rc-1.0.0` | A–D scaffold |

---

## Operator loop (post-cut)

```bash
git fetch origin && git checkout main && git pull origin main
./scripts/test-product-offline.sh
./scripts/test-l3-smoke.sh --offline-only
# hearts
cargo test -p dsb-tools path_a
# ledger (read-only unless a new train starts)
omc ultragoal status --plan-id fleet-4x 2>/dev/null || true
```

## Continuity rules

1. Do not reopen `fleet-4x` as active product SSOT; extend 4.x via minors or a new plan-id.  
2. Keep Path A hearts (`yolo = false`).  
3. PR planning: [ULTRAGOAL_PR_PLANNING.md](./ULTRAGOAL_PR_PLANNING.md).  
4. Merge commits (squash disabled).  
5. Full SemVer only; dual CLI names.  
