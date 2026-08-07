# G012 Adversarial review B — fusion / wiring / gate integrity

| Field | Value |
|-------|--------|
| **Reviewer** | Adversarial reviewer B (wiring / gate integrity) |
| **Role** | Independent of reviewer A and of story authors |
| **Date** | 2026-08-07 |
| **Plan** | `owner-bar-5x` → `5.0.0` |

## Mandate

Attack dead-wiring, orphan hearts, fraudulent STATUS, and process-police gaps.
Verify mechanical gates cannot green on empty evidence.

## Findings

| Severity | Finding | Disposition |
|----------|---------|-------------|
| High (historical) | Pre-G004 Standard tool_configs dead wiring | **Fixed** G004; `check-path-a-linkage` PASS |
| High (historical) | `assemble_path_a_context` test-only | **Fixed** G008 stamp |
| Medium | Pass map is human-authored TSV, not auto-derived from CI | **Accept for v1**: map is reviewable SSOT; illegal statuses fail-close; OB-4 selftest remains |
| Medium | Selftest no longer requires RED aggregator | **Intentional**: green era substrate still validates format + runnable linkage |
| Low | Scripted DeepSeek R0A not live for every L3 row | **Accept** with offline smoke + unit + stamps |

## Mechanical probes

```text
./scripts/check-path-a-linkage.sh     → PASS
./scripts/check-forbidden-evidence.sh → PASS
./scripts/test-owner-bar.sh           → ALL PASS (60/60)
./scripts/test-owner-bar.sh --selftest → PASS
./scripts/test-heart-regression.sh --with-e2e  (landed G010) → PASS
./scripts/test-install-dual-cli.sh    (landed G011) → PASS
```

## Verdict

**APPROVE** freeze + tag **`v5.0.0`** after version bump lands on the same
manifest SHA. Require CUT_5_0_0 evidence file to list SHA + binary hashes +
pass map path. Do not re-open G001–G011 stories without a new plan-id.
