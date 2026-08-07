# G012 Adversarial review A — product claims vs Path A evidence

| Field | Value |
|-------|--------|
| **Reviewer** | Adversarial reviewer A (product / acceptance bar) |
| **Role** | Independent of story implementers |
| **Date** | 2026-08-07 |
| **Frozen SHA (pre-cut docs)** | see CUT manifest (updated to release SHA at tag) |
| **Plan** | `owner-bar-5x` → `5.0.0` |

## Mandate

Challenge whether P0 ledger rows marked PASS are supported by **Path A** R0A
evidence (not library-only). Fail-close on dual-ledger language or 3.x/4.x
overclaim.

## Findings

| Severity | Finding | Disposition |
|----------|---------|-------------|
| Medium | Grok agent binary still has no `dsb-*` Cargo dep (F1 purity) | **Accept with honesty**: product Path A is CLI `agent_launch` stamps + Grok dispatch repair/snippet wiring already landed; residual noted in KNOWN_LIMITS / F1 reason |
| Medium | Live L3.2/L3.5 without API key not re-run this cut | **Accept residual**: offline L3 + units + stamps PASS; live env-gated ops residual |
| Low | `reasoning_effort` JSON often null on Grok chat_completions wire | **Accept residual** (G009 L1-30 honesty) |
| Low | user_info calendar date not in Spec 10 system prefix | **Accept** (volatile head) |
| Info | Hollow product `deepseek-build-agent` install path | **Mitigated** G010/G011 agent probe + dual name install |

## Checks performed

- [x] `OWNER_BAR_PASS_MAP.tsv` covers 60 ledger IDs with only PASS/FAIL/NOT_RUN
- [x] `./scripts/test-owner-bar.sh` → ALL PASS (linkage + forbidden green)
- [x] Story evidence G003–G011 present under `docs/product/evidence/`
- [x] No claim that 3.0.0/4.0.0 met owner-bar (KNOWN_LIMITS demotion intact)

## Verdict

**APPROVE cut to `5.0.0`** contingent on dual review B and CUT file citing Path A
artifacts only. Residuals must remain in KNOWN_LIMITS, not silent.
