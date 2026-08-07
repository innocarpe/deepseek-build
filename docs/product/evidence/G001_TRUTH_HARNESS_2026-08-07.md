# G001 TruthHarness — evidence (owner-bar-5x)

| Field | Value |
|-------|--------|
| **Story** | G001 TruthHarness (`G001-g001-truthharness`) |
| **WAVE units** | 5x-H0-1 (honesty demotion residual), 5x-H0-2 (RED gate substrate verify) |
| **Date** | 2026-08-07 |
| **Base SHA (pre-PR)** | `d957af85dedcb57146c36f2b69a707c1f94ef8f5` (main after PR #89 plan package) |
| **SemVer on disk** | `4.0.2` — **not** owner-bar complete |

## Done criteria (mechanical)

| Check | Result |
|-------|--------|
| `./scripts/test-owner-bar.sh` exit non-zero | **PASS (RED)** exit 1 |
| STATUS.tsv ledger rows | **60 FAIL**, 0 PASS |
| `./scripts/test-owner-bar.sh --selftest` | **PASS** exit 0 |
| `./scripts/check-forbidden-evidence.sh` | **PASS** |
| `./scripts/check-path-a-linkage.sh` | **FAIL (expected RED)** — DEAD_WIRING + NO_MINT |
| Honesty demotion (3.x/4.x not owner-bar green) | versions/KNOWN_LIMITS/SSOT from PR #89; residual Agents/GATES/README fixed this PR |
| Product heart feature code in G001 | **None** (docs + gate verify only) |

## Commands (captured)

```text
$ git rev-parse HEAD
d957af85dedcb57146c36f2b69a707c1f94ef8f5

$ ./scripts/test-owner-bar.sh
=== test-owner-bar ===
git_sha=d957af85dedcb57146c36f2b69a707c1f94ef8f5
wrote …/OWNER_BAR_STATUS.tsv (60 rows, all FAIL)
DEAD_WIRING: tool_configs applied only when effective != Standard
NO_MINT: read_file has no file_version/snippet_id issuance
check-path-a-linkage: FAIL (2 issues) — expected RED until fusion
summary: PASS=0 FAIL=60 linkage_exit=1 forbidden_exit=0
test-owner-bar: RED (expected until owner-bar-5x fusion complete)
# exit 1

$ ./scripts/test-owner-bar.sh --selftest
selftest: correctly detects incomplete coverage (2 < 60)
selftest: illegal status token: SKIP|BLOCKED|N/A|NOT_RUN|XFAIL|IGNORED
selftest: linkage check exits non-zero on current tree (expected RED)
selftest: aggregator exits non-zero (expected RED baseline)
test-owner-bar --selftest: PASS (gate substrate ok)
# exit 0

$ ./scripts/check-forbidden-evidence.sh
check-forbidden-evidence: PASS
```

## Artifacts

- Live status: [`OWNER_BAR_STATUS.tsv`](./OWNER_BAR_STATUS.tsv) (all FAIL / `no_R0A_harness_yet`)
- Gate scripts (landed PR #89): `scripts/test-owner-bar.sh`, `scripts/check-path-a-linkage.sh`, `scripts/check-forbidden-evidence.sh`
- Plan package (landed PR #89): PRD-v5, OWNER_BAR_*, WAVE_5x, cold-start, adversarial plan review

## Explicit non-claims

- Does **not** claim Path A fusion, mint, snippet_safe, or any heart PASS.
- Does **not** promote SKIP/BLOCKED/N/A as cut PASS.
- Does **not** green owner-bar; baseline is intentionally **RED**.

## Next story

**G002 PathA-R0-Rig** — public entry + scripted DeepSeek + wire capture (`scripts/test-path-a-public-entry-e2e.sh` + `scripts/lib/scripted_deepseek_server.*`).
