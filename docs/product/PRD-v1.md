# PRD v1 — DeepSeek Build **1.x** (scaffold line)

| Field | Value |
|-------|--------|
| **SemVer line** | **`1.0.0` – `1.x.y`** |
| Status | **Shipped / frozen for product features** (legacy scaffold) |
| Tags / npm | `v1.0.0`, `v1.1.0` · `@innocarpe/deepseek-build@1.0.0` / `1.1.0` |
| Owner | @innocarpe |
| Last updated | 2026-08-07 |
| Index | [versions/README.md](./versions/README.md) |
| Related | [MASTER_PLAN.md](./MASTER_PLAN.md) · [prd/](./prd/) (waves A–D) · [REPLAN_2.0.md](./REPLAN_2.0.md) |

> **Honesty:** 1.x is a **useful research + contract scaffold**.  
> It is **not** the product the owner ordered (Grok-class full-screen agent).  
> Product cut moved to **2.0.0** — see [PRD-v2.md](./PRD-v2.md).

---

## 1. Problem (as understood for 1.x)

Need a DeepSeek-first terminal agent with specs, dual CLI (`deepseek-build` / `dsb`), and install/npm path — while architecture (L1/L2/L3) was still being locked.

## 2. Goals (1.x)

| ID | Goal | Outcome |
|----|------|---------|
| G1 | Dual CLI + SemVer harness | Shipped |
| G2 | DeepSeek provider + thinking/effort surface | Shipped (thin agent) |
| G3 | Specs + gates + PR harness | Shipped as docs/skill |
| G4 | Thin interactive agent (`chat` / `run`) | Shipped |
| G5 | First-run setup / credentials 0600 | Shipped in **1.1.0** |

### Non-goals for 1.x (even if later required)

- Full-screen Grok Build base runtime  
- L1/L2 fused into a Grok shell  
- Claiming “product complete” as Grok-class agent  

## 3. What shipped (claimed vs real)

| Claim | Reality on 1.x |
|-------|----------------|
| “Coding agent CLI” | **Thin clap REPL** + tools on `dsb-agent` / `dsb-tools` |
| “Grok-class” | **Not** Grok runtime — MVP heuristics only |
| “1.0.0 product done” | **Scaffold train** complete; owner intent **not** met |
| npm package | Installable; still useful for contracts/experiments |

## 4. Architecture (1.x)

```text
dsb / deepseek-build
  → dsb-cli (clap + thin REPL / run)
  → dsb-agent + dsb-provider-deepseek + dsb-tools + dsb-context + dsb-config
```

Layer intent already documented in [HARNESS_PHILOSOPHY.md](../architecture/HARNESS_PHILOSOPHY.md), but **1.x does not implement L3 Grok base**.

## 5. Freeze policy (1.x)

| Allowed | Forbidden |
|---------|-----------|
| Critical security / install bugs | New product features on thin REPL |
| Docs that mark 1.x as legacy | Claiming 1.x is Grok-base product |
| npm history remains installable | Unpublish / force-delete 1.x |

## 6. Exit / handoff

- Scaffold chronology: Waves A–D under [prd/](./prd/)  
- Product direction supersession: [REPLAN_2.0.md](./REPLAN_2.0.md)  
- Next product major: [PRD-v2.md](./PRD-v2.md)  

## 7. Release log (1.x)

| Version | Notes |
|---------|--------|
| **1.0.0** | Scaffold train RC-style cut (thin agent) |
| **1.1.0** | First-run setup / auth onboarding |
