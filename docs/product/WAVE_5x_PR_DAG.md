# Wave 5.x PR DAG — owner-bar complete product → **`5.0.0`**

**Plan id:** `owner-bar-5x` — board: [OWNER_BAR_5X_GOALS.md](./OWNER_BAR_5X_GOALS.md)  
**PRD:** [PRD-v5.md](./PRD-v5.md)  
**Ledger:** [OWNER_BAR_P0_LEDGER.md](./OWNER_BAR_P0_LEDGER.md)  
**Do not invent overnight units** — extend this file in a docs PR if the DAG must change.

Prior DAGs (`WAVE_3x`, `WAVE_4x`) are **historical**. This DAG owns the only train allowed to claim the full owner product.

---

## Legend

| Field | Meaning |
|-------|---------|
| **Unit** | One mergeable PR |
| **Depends** | Must merge before start (or stack) |
| **Band** | SemVer band when unit ships |
| **Evidence** | Mandatory artifact — “tests pass” is **not** evidence |

Default: **serial** unless marked parallel-safe.  
Merge: repo setting (prefer **merge commit** if squash disabled).  
SemVer: full **`MAJOR.MINOR.PATCH` only**.  
Every PR body: **Problem / What changed / Testing (commands + stdout summary) / AI review / Security / Notes** + evidence path + candidate SHA.

---

## H0 — Gate + truth (must be first)

| ID | Unit | Depends | Band | Evidence |
|----|------|---------|------|----------|
| **5x-H0-1** | Honesty demotion + PRD-v5 / chain / versions point to owner-bar-5x | — | docs | README/KNOWN_LIMITS/versions/SSOT/ULTRAGOAL_CHAIN state 3.x/4.x not owner-bar green; PRD-v5 linked |
| **5x-H0-2** | Gate substrate: `test-owner-bar.sh`, forbidden-evidence, path-a-linkage, STATUS.tsv RED | 5x-H0-1 | scripts | Script **exits non-zero**; TSV covers ledger IDs; self-test rejects R2-only / skip / stale fixtures |
| **5x-H0-3** | Path A R0 rig: public entry + scripted DeepSeek + wire capture | 5x-H0-2 | scripts | `test-path-a-public-entry-e2e.sh` + scripted server; process tree shows agent via launch; wire JSON artifact |

**H0 exit:** No feature story can claim green without the harness. Baseline is **RED**.

**Parallel after 5x-H0-2:** none for H0-3 (rig before hearts).

---

## H1 — L1 edit + perms (alpha)

| ID | Unit | Depends | Band | Evidence |
|----|------|---------|------|----------|
| **5x-H1-0** | `read_file` mints `file_version` (or snippet_id) on Path A wire | 5x-H0-3 | `5.0.0-alpha.N` | Wire transcript: token == sha256(file); L1-45-2 |
| **5x-H1-1** | snippet_safe default on Standard + dead-guard fix + negatives + **liveness** | 5x-H1-0 | alpha | rg: no `effective != Standard` exclusion of tool_configs in production; L1-45-0…6 R0A; binary strings optional |
| **5x-H1-2** | write overwrite safety + bash invalidates versions | 5x-H1-1 | alpha | L1-45-7/8 R0A |
| **5x-H1-3** | Permissions matrix Path A (TTY/headless/boundary/no bypass) | 5x-H0-3 | alpha | L1-90-* R0A; may **parallel** 5x-H1-0..2 if files disjoint |

**H1 exit:** Edits are safe **and still work** on the default product path.

**Parallel-safe:** `5x-H1-3` ∥ `5x-H1-0` only if no shared tool files; prefer after mint if editing same tools crate.

---

## H2 — L2 Reasonix on Path A (beta)

| ID | Unit | Depends | Band | Evidence |
|----|------|---------|------|----------|
| **5x-H2-1** | Tool-call repair on Grok dispatch (Spec 15) | 5x-H0-3 | `5.0.0-beta.N` | Malformed fixtures: one repair then error; L2-15-*; **no invent/rename** |
| **5x-H2-2** | Prefix/skills/resume goldens from **captured wire** (Spec 10 + 70 + 100) | 5x-H1-1, prefer 5x-H2-1 | beta | Two-turn byte-stable prefix; skills index; resume; L2-10-*; L1-70; L1-100 |
| **5x-H2-3** | Flash/Pro + effort on Path A wire (Spec 20 + 30) | 5x-H0-3 | beta | Flash default; Pro one-turn; precedence; base_url; L2-20-*; L1-30 |

**H2 exit:** Reasonix economics **control** default agent turns (wire proof).

**Parallel after 5x-H0-3:** `5x-H2-1` ∥ `5x-H2-3` if disjoint; `5x-H2-2` waits for tool schema from H1-1.

---

## H3 — L3 under hearts (rc)

| ID | Unit | Depends | Band | Evidence |
|----|------|---------|------|----------|
| **5x-H3-1** | Parallel tools + background shell R0A + heart regression | H1 exit + H2 exit | `5.0.0-rc.N` | L3-50-*; full L1/L2 re-run PASS |
| **5x-H3-2** | Subagents + worktree + worker cache law + snippet invalidate | 5x-H3-1 | rc | L3-60-*; L3-WT-*; parent/worker prefix hashes |

**H3 exit:** Throughput is product identity **without** breaking hearts.

---

## H4 — Install product

| ID | Unit | Depends | Band | Evidence |
|----|------|---------|------|----------|
| **5x-H4-1** | Clean primary-platform install dual CLI + agent provenance | 5x-H0-3 (may parallel hearts) | rc | S1–S7; OB-1; package inventory; both CLIs resolve agent without required override |

Cut still requires H3 green before G012.

---

## H5 — Freeze / review / cut

| ID | Unit | Depends | Band | Evidence |
|----|------|---------|------|----------|
| **5x-H5-1** | Owner-bar green + dual adversarial review + SemVer **5.0.0** + tag `v5.0.0` | H3 exit + 5x-H4-1 | **5.0.0** | `test-owner-bar.sh` exit 0; live DeepSeek R0A; two independent reviews same SHA+manifest; CHANGELOG; docs singular; npm human-gated |

**H5 exit = ship `5.0.0`.** Do not tag earlier. Dual review **required**.

---

## Explicitly out of this DAG

| Out | Notes |
|-----|--------|
| Re-cutting 3.0.0 / 4.0.0 as owner-bar complete | Honesty forbids |
| Thin-path-only hearts as product | Path B reference OK; not cut evidence |
| Secondary platforms beyond primary | P1 after 5.0.0 unless already free |
| MCP full / plan mode polish | P1 |

---

## Everyday commands (not vendor-full)

```bash
./scripts/test-owner-bar.sh
./scripts/check-forbidden-evidence.sh
./scripts/check-path-a-linkage.sh
# After G002+:
./scripts/test-path-a-public-entry-e2e.sh
```
