# Release train — Wave A only (`0.2.0`–`0.7.0`)

**Status:** Wave A **complete on `main`** (re-check `Cargo.toml`; patch line **`0.7.1`**).  
**Full vision board:** [MASTER_PLAN.md](./MASTER_PLAN.md)  
**SSOT priority:** [SSOT.md](./SSOT.md)  
**PR units (historical fixed DAG):** [WAVE_A_PR_DAG.md](./WAVE_A_PR_DAG.md)  
**After Wave A:** [ULTRAGOAL_CHAIN.md](./ULTRAGOAL_CHAIN.md) → **`native-0x`** (Wave B). Do **not** use this file for `0.8.0+`.

**SemVer:** full `MAJOR.MINOR.PATCH` only.  
**CLI:** `deepseek-build` · `dsb` (ADR 0006).

---

## 1. Intent

Ship user-visible **`0.y.0`** minors until **dogfood-usable** and npm **package** exists.  
**`1.0.0` is out of Wave A.** Parallel tools / subagents are **Wave C**, not this file.

---

## 2. Where we are now

| Item | Value |
|------|--------|
| Version | Read `Cargo.toml` (expect **`0.7.1`**) |
| Install | `./scripts/install.sh` + npm `@innocarpe/deepseek-build` ([01-install](../user-guide/01-install.md), [05-npm](../user-guide/05-npm.md)) |
| Tools | read/edit/write/grep/bash; `--dogfood` |
| Sessions | **0.5.0** JSONL persist/resume |
| Surface | **0.6.0** skills index min + thinking/effort UX |
| npm | **0.7.1** `@innocarpe/deepseek-build` published; dual bins; Rust needed for postinstall |

---

## 3. Dogfood-usable (executable)

Human checklist **and** machine check:

```bash
./scripts/smoke-dogfood.sh
# optional live:
# DEEPSEEK_API_KEY=… ./scripts/smoke-dogfood.sh
```

| # | Criterion | How verified |
|---|-----------|----------------|
| 1 | Bins installable / buildable | smoke builds dual bins |
| 2 | Auth possible | env `DEEPSEEK_API_KEY` or credentials file (live section) |
| 3 | `run`/`chat` work | help + optional live run |
| 4 | Tools: read/edit/write/search/bash under policy | `cargo test --workspace` |
| 5 | Workspace write profile | `--dogfood` in CLI |
| 6 | Documented smoke | README + user-guide |
| 7 | Full SemVer on both bins | smoke version check |

**Sessions / npm package** are Wave A **delivery** goals (`0.5.0`/`0.7.0`) but dogfood-usable **coding** can hold once 1–7 pass even if registry publish is pending.

---

## 4. Minors (Wave A only)

| SemVer | Theme | Status |
|--------|--------|--------|
| `0.1.0` | Engine preview | shipped |
| `0.2.0` | PATH install | shipped |
| `0.3.0` | Tools daily + `--dogfood` | shipped |
| `0.4.0` | Dogfood proof note | shipped |
| `0.5.0` | Sessions | shipped |
| `0.6.0` | Skills index + effort UX | shipped |
| `0.7.0` | npm package dual bins | shipped |
| `0.7.1` | help SemVer example + npm install docs nits | shipped |

**Not in this document:** `0.8.0+` — see MASTER_PLAN Waves B–D and [WAVE_B_PR_DAG.md](./WAVE_B_PR_DAG.md).

---

## 5. Ultragoal

Plan **`dogfood-0x`**: expect **7/7 complete** after `0.7.0`.  
Next plan: **`native-0x`**.

---

## 6. Agent rules

1. PR units first — [ULTRAGOAL_PR_PLANNING.md](./ULTRAGOAL_PR_PLANNING.md) + [stack-merge-runbook.md](../contributing/stack-merge-runbook.md).  
2. SemVer full triples; dual CLI.  
3. Do not re-open Wave A minors unless smoke fails.  
4. npm **publish** never required for agent story complete (ADR 0007).  

---

## 7. Progress log

| SemVer | Date | Notes |
|--------|------|--------|
| `0.1.0`–`0.7.0` | 2026-08-06 | Wave A complete on main (#18–#26) |
| `0.7.1` | 2026-08-06 | CLI `--help` SemVer example tracks `CARGO_PKG_VERSION`; npm install docs (Rust + first-build time); registry `@innocarpe/deepseek-build@0.7.1` |
