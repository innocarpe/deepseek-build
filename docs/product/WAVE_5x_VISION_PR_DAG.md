# WAVE — vision-complete `5.x` PR DAG (SemVer rebased)

> [!IMPORTANT]
> **ARCHIVED / COMPLETED.** All listed vision PR units have merged to `main`
> through **PR #147**, producing source version **`5.5.0`**. npm and GitHub
> Latest remain **`5.2.2`** until the release lane publishes `5.5.0`.

**Plan:** [VISION_COMPLETE_5X_GOALS.md](./VISION_COMPLETE_5X_GOALS.md)
**Rules:** [ULTRAGOAL_PR_PLANNING.md](./ULTRAGOAL_PR_PLANNING.md)
**Floor:** `main` **`5.5.0`** · do **not** schedule **`5.0.1`–`5.5.0`** as future feature cuts

## Graph

```text
[DONE] VC001  5.0.1 version/update (npm)
[SHIP] VC001b 5.1.0 product chrome on main ──► VC001c finish Release/npm if lagging
                    │
                    ▼
        ┌───────────┼───────────┐
        ▼           ▼           ▼
   Track A       Track B     Track C
   Deep Code     Reasonix    Grok L3
   VC002–006     VC007–009   VC010–013
   → 5.3.0       → 5.3.0     → 5.4.0
   (5.2.x published floor line; one 5.3.0 source bump for A+B)
        │           │           │
        └───────────┴───────────┘
                    ▼
              VC014 docs
                    ▼
              VC015 freeze v5.5.0 (or free 5.Y.0)
```

## PR units

### ~~VC001~~ — DONE (`5.0.1`)
Product SemVer / update check fix (PR #117 + npm 5.0.1).

### ~~VC001b~~ — ON MAIN (`5.1.0`)
Theme v2 / chrome (e.g. PR #119). **Not** a future planning target.

### PR unit VC001c — `chore(release): finish 5.1.0 assets + npm` (only if lagging)
- **Intent:** GitHub Release `v5.1.0` + npm `@…@5.1.0` match `main`
- **SemVer:** **5.1.0** (same) or **5.1.1** if hot-fix needed
- **Depends on:** VC001b
- **Skip if:** `npm view` and `gh release view v5.1.0` already green

### PR unit VC002 — `spec(45): Path A snippet_id ADR + store design`
- **SemVer:** none
- **Depends on:** 5.1.0 floor stable (prefer)
- **Status:** merged — PR #125

### PR unit VC003 — `feat(tools): mint snippet_id on Path A read_file`
- **Depends on:** VC002
- **Status:** merged — PR #130

### PR unit VC004 — `feat(tools): require snippet_id on Path A search_replace`
- **Depends on:** VC003
- **Status:** merged — PR #135

### PR unit VC005 — `feat(tools): write/bash snippet invalidation laws`
- **Depends on:** VC004
- **Status:** merged — PR #137

### PR unit VC006 — `test+chore(release): 5.3.0 Deep Code snippet_id cut`
- **SemVer:** **5.3.0** final cut after intermediate `5.2.x` publish line
- **Depends on:** VC005
- **Tests:** heart regression + Path A multi-edit R0A
- **Status:** merged — PR #138

### PR unit VC007 — `feat(context): Spec 10 assembly on Grok Path A turns`
- **Depends on:** VC006 prefer
- **Status:** merged — PR #139

### PR unit VC008 — `feat(provider): reasoning_effort on DeepSeek wire`
- **Depends on:** VC007 prefer
- **Status:** merged — PR #140

### PR unit VC009 — `feat(shell): Path A cache-hit visibility for V2-cache`
- **SemVer:** part of the **5.3.0** source line; no second bump after VC006
- **Depends on:** VC008
- **Status:** merged — PR #141

### PR unit VC010 — `test(l3): multi-tool parallel + bg Path A R0A`
- **Depends on:** VC006
- **Status:** merged — PR #142

### PR unit VC011 — `test(l3): subagent + worker cache Path A R0A`
- **Depends on:** VC010
- **Status:** merged — PR #143

### PR unit VC012 — `docs+test(worktree): dogfood flow`
- **Depends on:** VC011
- **Status:** merged — PR #144

### PR unit VC013 — `test+chore(release): 5.4.0 L3 cut`
- **SemVer:** **5.4.0**
- **Depends on:** VC012
- **Status:** merged — PR #145

### PR unit VC014 — `docs(product): user-guide + KNOWN_LIMITS vision pass`
- **Depends on:** VC013
- **Status:** merged — PR #146

### PR unit VC015 — `chore(release): vision-complete freeze v5.5.0`
- **SemVer:** **5.5.0** (or next free `5.Y.0` if 5.5.0 already taken — re-check)
- **Depends on:** VC014 + dual adversarial review
- **Status:** merged — PR #147; release publish pending

## Merge policy

- GitHub **merge commit** only
- Labels mandatory · English public text
- Stack with `--base` when sequential
- Before any version bump: re-read `Cargo.toml` + `npm view` + `gh release list`
