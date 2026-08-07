# Owner-bar P0 ledger (frozen for `owner-bar-5x` → `5.0.0`)

| Field | Value |
|-------|--------|
| **Status** | **Frozen for train** — change only via docs PR that updates this file + [OWNER_BAR_ACCEPTANCE.md](./OWNER_BAR_ACCEPTANCE.md) together |
| **Plan id** | `owner-bar-5x` |
| **Cut** | `5.0.0` / tag `v5.0.0` |
| **Last frozen** | 2026-08-07 |
| **Reviews** | [OWNER_BAR_5X_PLAN_ADVERSARIAL_2026-08-07.md](./evidence/OWNER_BAR_5X_PLAN_ADVERSARIAL_2026-08-07.md) |

Every row below is **P0** for cut. There is **no cut-time N/A**.  
Demote only by changing this ledger **before** G001 freezes baseline.

**Cut-day status** is **not** this table’s Baseline column. Baseline is historical diagnosis only.  
Live status: `docs/product/evidence/OWNER_BAR_STATUS.tsv` emitted by `./scripts/test-owner-bar.sh`.

---

## Status codes (machine)

| Code | Meaning |
|------|---------|
| `PASS` | R0A evidence green for this row on release SHA + binary manifest |
| `FAIL` | Not proven on Path A |
| `NOT_RUN` | Harness missing or not executed — **counts as FAIL for cut** |

`SKIP`, `BLOCKED`, `N/A`, `XFAIL`, `IGNORED` are **illegal** in cut manifest.

---

## Frozen P0 rows

| ID | Requirement (short) | Owning story | Required evidence class | Baseline 2026-08-07 |
|----|---------------------|--------------|-------------------------|---------------------|
| **S1** | Dual CLI same behavior | G011 | R0A install | FAIL until install e2e |
| **S2** | Bare TTY → full-screen agent | G001/G011 | R0A public entry | PASS-ish shell; re-prove |
| **S3** | Product home + GROK_HOME bridge | G011 | R0A | re-prove |
| **S4** | Model-level DeepSeek `base_url` | G009/G011 | R0A wire | re-prove |
| **S5** | Readable DeepSeek theme | G011 | R0A dogfood | re-prove |
| **S6** | Full SemVer discipline | G012 | process | re-prove |
| **S7** | Clean-machine install CLI+agent | G011 | R0A | FAIL |
| **S8** | Default non-YOLO | G006 | R0A | re-prove |
| **L1-45-0** | **Liveness:** ≥3 successful Path A edits / ≥2 files / exit 0 with safety on | G004 | R0A | FAIL |
| **L1-45-1** | Default tool path snippet-safe (params actually applied) | G004 | R0A+R1 | FAIL (dead Standard guard) |
| **L1-45-2** | `read_file` mints `file_version` (or Spec 45 `snippet_id`) on wire | G003 | R0A wire | FAIL (no mint) |
| **L1-45-3** | Edit rejects missing token | G004 | R0A negative | FAIL |
| **L1-45-4** | Edit rejects stale token | G004 | R0A negative | FAIL |
| **L1-45-5** | Ambiguous match no silent wrong replace | G004 | R0A negative | FAIL |
| **L1-45-6** | Empty old_string not free whole-file overwrite | G004 | R0A negative | FAIL |
| **L1-45-7** | `write` overwrite safety spirit | G005 | R0A | FAIL |
| **L1-45-8** | Bash mutation invalidates versions for touched paths | G005 | R0A | FAIL |
| **L1-90-1** | allow/deny/ask on mutate | G006 | R0A | re-prove |
| **L1-90-2** | Headless Ask→deny/cancel | G006 | R0A | re-prove |
| **L1-90-3** | Product default not YOLO | G006 | R0A | re-prove |
| **L1-90-4** | Workspace boundary matrix | G006 | R0A | FAIL |
| **L1-90-5** | Parallel/subagent cannot skip perms | G006/G010 | R0A | FAIL |
| **L1-70** | Skills index in stable prefix; body on demand | G008 | R0A | FAIL |
| **L1-30** | Effort/thinking controllable; default coding effort documented + wire-asserted | G009 | R0A wire | FAIL |
| **L1-100** | Session resume on product agent path | G008 | R0A | FAIL |
| **L2-10-1** | stable_prefix + volatile_tail on Path A assembly | G008 | R0A wire | FAIL |
| **L2-10-2** | Prefix order per Spec 10 | G008 | R0A golden | FAIL |
| **L2-10-3** | Unchanged inputs → byte-stable prefix across turns | G008 | R0A golden | FAIL |
| **L2-10-4** | No wall-clock in prefix | G008 | R0A negative | FAIL |
| **L2-10-5** | Compaction/resume preserve contract | G008 | R0A | FAIL |
| **L2-10-6** | Heart impl linked/called from Path A or honesty demotion | G008 | R1 | FAIL |
| **L2-15-1** | One repair pass before execute | G007 | R0A | FAIL |
| **L2-15-2** | Never invent required args / rename tool | G007 | R0A negative | FAIL |
| **L2-15-3** | Pairing holes on resume | G007/G008 | R0A | FAIL |
| **L2-15-4** | Repair on default agent dispatch (not thin only) | G007 | R0A+R1 | FAIL |
| **L2-20-1** | Default Flash wire model | G009 | R0A wire | re-prove |
| **L2-20-2** | Pro one-turn escalate then return (or sticky preset) | G009 | R0A | FAIL |
| **L2-20-3** | Turn model visibility | G009 | R0A | FAIL |
| **L2-20-4** | Precedence table | G009 | R0A | FAIL |
| **L2-20-5** | base_url on both models | G009 | R0A | re-prove |
| **L3-50-1** | RO parallel / mutate serial | G010 | R0A | FAIL |
| **L3-50-2** | Fail-closed classify unknown/bash/MCP | G010 | R0A | FAIL |
| **L3-50-3** | Background shell + collect-by-id | G010 | R0A | FAIL |
| **L3-50-4** | Bg/wait without secret flags | G010 | R0A | FAIL |
| **L3-60-1** | Subagents default-on | G010 | R0A | FAIL |
| **L3-60-2** | Explore + implement kinds | G010 | R0A | FAIL |
| **L3-60-3** | Worker cache law (prefix template) | G010 | R0A hash | FAIL |
| **L3-60-4** | Worker mutation invalidates parent snippets | G010 | R0A | FAIL |
| **L3-WT-1** | Worktree dogfoodable | G010 | R0A | FAIL |
| **L3-WT-2** | Honesty if worktree opt-in | G010/G012 | docs+R0 | FAIL |
| **L3-ID-1** | Claims match observed L3 behavior | G012 | docs scan | FAIL |
| **F1** | Path A binary links/embeds hearts | G012 | R1 graph | FAIL |
| **F2** | No dead-wiring of safety params | G004 | R0+rg | FAIL |
| **F3** | No Path B-only fusion claim | G001/G012 | honesty | FAIL hist. |
| **F4** | CUT cites Path A R0A only | G012 | CUT file | FAIL hist. |
| **F5** | Dual independent adversarial reviews same SHA/manifest | G012 | review arts | FAIL |
| **OB-1** | Public entry: installed `deepseek-build` + `dsb` (agent path via launch), no override required | G001/G011 | R0A | FAIL |
| **OB-2** | Manifest: every P0 has status; zero illegal statuses | G001/G012 | machine | FAIL until harness |
| **OB-3** | Freshness: all PASS on cut SHA + binary hash | G012 | machine | FAIL |
| **OB-4** | Gate self-test rejects fraudulent evidence | G001 | selftest | FAIL until harness |

**P1 (explicitly not cut-blocking):** L1-80 MCP full, L1-110 plan mode polish, multi-platform secondary installs beyond primary.

---

## Story → rows (ownership)

| Story | Rows owned (primary) |
|-------|----------------------|
| G001 | F3 partial, OB-2 harness, OB-4, honesty demotion |
| G002 | R0A rig substrate (enables all later R0) |
| G003 | L1-45-2 |
| G004 | L1-45-0,1,3,4,5,6 · F2 |
| G005 | L1-45-7,8 |
| G006 | L1-90-* · S8 |
| G007 | L2-15-* |
| G008 | L2-10-* · L1-70 · L1-100 |
| G009 | L2-20-* · L1-30 · S4 |
| G010 | L3-50/60/WT · L1-90-5 recheck · heart regression |
| G011 | S1–S7 · OB-1 |
| G012 | F1–F5 · L3-ID-1 · OB-3 · tag |

---

## Cut formula (normative)

```text
CUT(v5.0.0) =
  Frozen(this ledger) ∧ Frozen(GitSHA, BinaryManifest)
  ∧ ∀ row ∈ this ledger:
      status(row)=PASS ∧ R0A(row) ∧ Fresh(row, GitSHA, BinaryManifest)
      ∧ (code-bound row ⇒ R1(row))
  ∧ ./scripts/test-owner-bar.sh exit 0
  ∧ LiveDeepSeekR0 (cut day; not SKIP)
  ∧ DualIndependentAdversarialReview(SHA, Manifest)
  ∧ Tag after all of the above
```
