# Ultragoal board — **`owner-bar-5x`** → **`5.0.0`**

**Plan id:** `owner-bar-5x`  
**DoD:** [PRD-v5.md](./PRD-v5.md) · [OWNER_BAR_ACCEPTANCE.md](./OWNER_BAR_ACCEPTANCE.md) · [OWNER_BAR_P0_LEDGER.md](./OWNER_BAR_P0_LEDGER.md)  
**PR units:** [WAVE_5x_PR_DAG.md](./WAVE_5x_PR_DAG.md)  
**Cold start:** [ULTRAGOAL_PROMPT_COLD_START_5.0.md](./ULTRAGOAL_PROMPT_COLD_START_5.0.md)  
**Plan reviews:** [evidence/OWNER_BAR_5X_PLAN_ADVERSARIAL_2026-08-07.md](./evidence/OWNER_BAR_5X_PLAN_ADVERSARIAL_2026-08-07.md)  
**Prior trains:** `heart-3x` / `fleet-4x` tags exist — **owner-bar NOT MET** (do not resume them as product SSOT)

---

## Rules (fail-close)

1. **One plate** until `v5.0.0` — do not invent a second plan-id mid-train.  
2. **No `--force` wipe** of in-progress ledger.  
3. **PR plan before code** every story ([ULTRAGOAL_PR_PLANNING.md](./ULTRAGOAL_PR_PLANNING.md)).  
4. **Child runtime = parent** (Grok session → `grok` only unless user explicitly crosses).  
5. **Full SemVer** only (`5.0.0`, never `5.0`).  
6. **CLI:** `deepseek-build` primary · `dsb` alias.  
7. **Evidence:** Path A R0A only for heart/L3 rows — never sole `cargo test -p dsb-*` / thin path.  
8. **Illegal statuses on cut:** SKIP, BLOCKED, N/A, NOT_RUN, XFAIL.  
9. **G001 first:** gate harness RED + honesty — no feature “green” before harness exists.  
10. **Mint before flip:** G003 before G004 (anti-brick).  
11. Merge: repo policy (**merge commits** if squash disabled). English GitHub text. Kind labels on PRs.

---

## Stories (G001 → G012)

| ID | Title | WAVE units | Band | Done when (Path A / mechanical) |
|----|-------|------------|------|----------------------------------|
| **G001** | TruthHarness | 5x-H0-1, 5x-H0-2 | docs+scripts | `./scripts/test-owner-bar.sh` **non-zero**; STATUS.tsv lists ledger rows mostly FAIL; self-test rejects fake evidence; 3.x/4.x demotion in versions/KNOWN_LIMITS/README as needed; **no product heart feature code** |
| **G002** | PathA-R0-Rig | 5x-H0-3 | scripts | Real agent via public CLI/`agent` entry + scripted DeepSeek server; wire transcript capture; no cargo-test-only evidence for this story |
| **G003** | MintFileVersion | 5x-H1-0 | alpha | Wire `read_file` result includes `file_version`=sha256(file) (or Spec 45 snippet_id) |
| **G004** | SnippetLive | 5x-H1-1 | alpha | Standard toolset applies snippet_safe; dead `!= Standard` guard fixed; negatives green; **L1-45-0 liveness** ≥3 edits / ≥2 files / exit 0 |
| **G005** | WriteBashInvalidate | 5x-H1-2 | alpha | write overwrite safety + bash invalidates versions (L1-45-7/8) |
| **G006** | PermsMatrix | 5x-H1-3 | alpha | Spec 90 matrix on Path A incl. headless + boundary + no bypass |
| **G007** | RepairDispatch | 5x-H2-1 | beta | Spec 15 one-pass repair on Grok dispatch; no invent/rename |
| **G008** | PrefixSkillsResume | 5x-H2-2 | beta | Spec 10 goldens from **captured wire** after G004 schema stable; skills index; resume/compaction |
| **G009** | RoutingEffort | 5x-H2-3 | beta | Flash default; Pro one-turn; precedence; effort wire; base_url |
| **G010** | L3UnderHearts | 5x-H3-1, 5x-H3-2 | rc | Parallel/bg/subagent/worktree R0A **and** full heart regression green |
| **G011** | InstallDualCLI | 5x-H4-1 | rc | Clean primary install; both commands; agent hash; theme/home |
| **G012** | FreezeReviewCut | 5x-H5-1 | **5.0.0** | Full ledger PASS; live DeepSeek R0A; dual independent adversarial reviews same SHA+manifest; tag **`v5.0.0`** only |

### DAG (ordering)

```text
G001 → G002 → G003 → G004 → G005 ─┐
         │                        │
         ├→ G006 ─────────────────┤
         ├→ G007 ─────────────────┼→ G008 (after G004 schema + prefer G007) → G010 → G012
         └→ G009 ─────────────────┤                         ▲
                                                            │
         G011 (packaging; may parallel after G002) ─────────┘
```

- **Strict:** G001→G002→G003→G004→G005; G010 after hearts G004–G009; G012 last.  
- **Parallel after G002:** G006 ∥ G007 ∥ G009 (disjoint files).  
- **G008 after G004** (tool schema in prefix); preferably after G007 too.  
- **G011** may progress in parallel once install scripts exist; cut still waits for G010.

---

## Create ledger (once, after G001 docs/scripts on main)

```bash
omc ultragoal create-goals --plan-id owner-bar-5x \
  --goal "G001 TruthHarness::test-owner-bar RED baseline + honesty demotion + gate selftest" \
  --goal "G002 PathA-R0-Rig::public entry + scripted DeepSeek + wire capture" \
  --goal "G003 MintFileVersion::read_file mints file_version on Path A wire" \
  --goal "G004 SnippetLive::snippet_safe default + negatives + liveness L1-45-0" \
  --goal "G005 WriteBashInvalidate::write safety + bash invalidates versions" \
  --goal "G006 PermsMatrix::Spec 90 Path A matrix headless+boundary+no bypass" \
  --goal "G007 RepairDispatch::Spec 15 one-pass repair on Grok dispatch" \
  --goal "G008 PrefixSkillsResume::Spec 10 wire goldens + skills + resume" \
  --goal "G009 RoutingEffort::Flash/Pro/effort/base_url on Path A wire" \
  --goal "G010 L3UnderHearts::parallel bg subagent worktree + heart regression" \
  --goal "G011 InstallDualCLI::clean install dual CLI agent theme home" \
  --goal "G012 FreezeReviewCut::ledger PASS dual review tag v5.0.0"
```

If plan exists with progress: **`omc ultragoal status --plan-id owner-bar-5x`** only — never `--force`.

```bash
omc ultragoal complete-goals --plan-id owner-bar-5x
```

---

## Operator loop

```bash
git fetch origin && git checkout main && git pull origin main
./scripts/test-owner-bar.sh || true   # expect non-zero until late train
omc ultragoal status --plan-id owner-bar-5x
omc ultragoal complete-goals --plan-id owner-bar-5x
# Active story only → WAVE_5x PR units → merge → checkpoint → complete-goals again
```

Stop when **12/12 complete** and `v5.0.0` tagged, or blocked with written evidence (not calendar wait).

---

## Non-goals (fail-close)

- Resuming `heart-3x` / `fleet-4x` as if owner-bar green  
- Claiming 5.0.0 from library `path_a_*` APIs alone  
- Closing stories with SKIP/BLOCKED credentials as PASS  
- Everyday `third_party/grok-build` full test (disk bomb)  
- Multi-vendor core / greenfield agent  
- Agent npm registry publish without human (ADR 0007)
