# Ultragoal cold start — **`vision-complete-5x`**

Paste this into a new agent session to resume without re-deriving versions.

---

## Immutable

- **Plan id:** `vision-complete-5x` only (active).  
- **Owner-bar** `v5.0.0` is **DONE** — do not re-open G001–G012.  
- **North star:** [VISION.md](./VISION.md) — Grok speed + Reasonix cost + Deep Code native fit.  
- **Board:** [VISION_COMPLETE_5X_GOALS.md](./VISION_COMPLETE_5X_GOALS.md)  
- **DAG:** [WAVE_5x_VISION_PR_DAG.md](./WAVE_5x_VISION_PR_DAG.md)  
- **Path A only** for behavior claims.  
- **Child runtime = Grok** (this parent).  
- **SemVer:** full `MAJOR.MINOR.PATCH`. **Never invent** a version already on `main` or npm.

## Floor (re-check every session)

```bash
git fetch origin main
git show origin/main:Cargo.toml | rg 'version = "'
npm view @innocarpe/deepseek-build version
gh release list -R innocarpe/deepseek-build --limit 8
```

**Known used (do not plan as future feature cuts):**

| Version | Role |
|---------|------|
| **5.0.0** | Owner-bar cut |
| **5.0.1** | Version/update fix (npm) |
| **5.1.0** | Product chrome on `main` (theme v2 etc.) — may still be **deploying** |

**Next vision minors (unless floor moved):**

| Version | Pillar |
|---------|--------|
| **5.2.0** | Deep Code Spec 45 `snippet_id` |
| **5.3.0** | Reasonix prefix assembly + effort on wire |
| **5.4.0** | L3 Path A dogfood under hearts |
| **5.5.0** (or free `5.Y.0`) | Vision-complete freeze |

If `main` or npm is already past these, **shift up** — never reuse.

## Stories

- **Done / shipping:** VC001, VC001b (± VC001c ops).  
- **Next implement:** VC002 → VC006 → ship **5.2.0**.  
- Then Track B **5.3.0**, Track C **5.4.0**, freeze **5.5.0**.

## Gates (must stay green)

```bash
./scripts/test-owner-bar.sh
./scripts/check-path-a-linkage.sh
./scripts/test-heart-regression.sh   # --with-e2e when agent built
```

## PR rules

- Unit plan before code ([ULTRAGOAL_PR_PLANNING.md](./ULTRAGOAL_PR_PLANNING.md)).  
- Atomic Conventional Commits · merge commit · English PR · labels.  
- Evidence under `docs/product/evidence/`.  
- Clean large `target/` after agent builds (disk).

## Stop conditions

- Hard block with no Path A path → document FAIL residual; do not claim vision-complete.  
- Do not re-tag 5.0.0 / 5.1.0 as vision-complete.

## First message to self

1. Floor check versions.  
2. If 5.1.0 Release/npm lag → VC001c only.  
3. Else start **VC002** Spec 45 ADR / store design toward **5.2.0**.  
