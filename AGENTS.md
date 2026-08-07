# DeepSeek Build — agent contract

This file is standing instructions for any coding agent working in this repo.

## Current phase

**Active product ultragoal:** **`vision-complete-5x`** — close [VISION.md](docs/product/VISION.md) north star inside **`5.x.y`** (Deep Code + Reasonix + Grok throughput). Owner-bar **`v5.0.0`** is **done**; **`5.1.0` is on `main` / shipping** — do **not** re-plan `5.0.1` or `5.1.0` as future feature cuts. Next vision minors: **`5.2.0` → `5.3.0` → `5.4.0` → freeze `5.5.0`** (re-check floor every session).

**Board:** [`docs/product/VISION_COMPLETE_5X_GOALS.md`](docs/product/VISION_COMPLETE_5X_GOALS.md) · DAG [`WAVE_5x_VISION_PR_DAG.md`](docs/product/WAVE_5x_VISION_PR_DAG.md) · cold start [`ULTRAGOAL_PROMPT_COLD_START_VISION_5X.md`](docs/product/ULTRAGOAL_PROMPT_COLD_START_VISION_5X.md)

**Completed trains:**  
- **`owner-bar-5x`** → **`v5.0.0`** (owner-bar complete product)  
- **`fleet-4x`** → **`v4.0.0`** L3 productization *attempt* · patches **`4.0.1`** / **`4.0.2`** / **`4.0.x`**  
- **`heart-3x`** → **`v3.0.0`** heart fusion *attempt*  

**Pointers:**  
- Owner-bar cut: [`docs/product/evidence/CUT_5_0_0_2026-08-07.md`](docs/product/evidence/CUT_5_0_0_2026-08-07.md)  
- Gate: `./scripts/test-owner-bar.sh` (must stay green) · `./scripts/check-path-a-linkage.sh` · heart: `./scripts/test-heart-regression.sh`  
- Chain: [`docs/product/ULTRAGOAL_CHAIN.md`](docs/product/ULTRAGOAL_CHAIN.md)  

**Major product lines (PRDs):** [`docs/product/versions/README.md`](docs/product/versions/README.md)  
- **1.x** scaffold — [PRD-v1](docs/product/PRD-v1.md)  
- **2.x shipped shell** — [PRD-v2](docs/product/PRD-v2.md)  
- **3.x tagged hearts *attempt*** — [PRD-v3](docs/product/PRD-v3.md) (`3.0.0`) — **not owner-bar green**  
- **4.x tagged L3 *attempt*** — [PRD-v4](docs/product/PRD-v4.md) (`4.0.0`+) — **not owner-bar green**  
- **5.x** owner-bar **`5.0.0`** + active vision-complete train — [PRD-v5](docs/product/PRD-v5.md) · [VISION_COMPLETE_5X_GOALS](docs/product/VISION_COMPLETE_5X_GOALS.md)  


**SSOT priority:** [`docs/product/SSOT.md`](docs/product/SSOT.md)  
**Historical board:** [`docs/product/MASTER_PLAN.md`](docs/product/MASTER_PLAN.md) · replan [`REPLAN_2.0.md`](docs/product/REPLAN_2.0.md)  
**PR planning:** [`docs/product/ULTRAGOAL_PR_PLANNING.md`](docs/product/ULTRAGOAL_PR_PLANNING.md) ·  
[`docs/contributing/stack-merge-runbook.md`](docs/contributing/stack-merge-runbook.md)  
**Merge on GitHub:** **merge commit** (squash disabled on this repo).  
**Architecture:** [`docs/architecture/HARNESS_PHILOSOPHY.md`](docs/architecture/HARNESS_PHILOSOPHY.md) ·  
[`docs/architecture/SYSTEM_ARCHITECTURE.md`](docs/architecture/SYSTEM_ARCHITECTURE.md)  
**SemVer on disk:** read root `Cargo.toml` (do not hardcode). Re-check ultragoal / major PRD each session.

## SemVer — fail-close (mandatory)

**Always** use full Semantic Version form **`MAJOR.MINOR.PATCH`** (e.g. `0.1.0`, `1.0.0`).

| Forbidden | Required |
|-----------|----------|
| `1.0`, `v1`, `0.2`, “ship one-point-oh” as a version id | `1.0.0`, `0.2.0`, tag `v1.0.0` |

Normative: [`docs/contributing/versioning.md`](docs/contributing/versioning.md) · ADR [0006](docs/adr/0006-cli-names-and-semver.md).  
Check: `./scripts/check-semver.sh`

Do **not** claim a release is ready as “1.0”; say **`1.0.0`** only when install + smoke criteria are met.

## CLI names — dual command (mandatory)

| Command | Role |
|---------|------|
| **`deepseek-build`** | Primary public command |
| **`dsb`** | Short alias (same binary behavior) |

Both are built from `dsb-cli`. Prefer documenting **`deepseek-build`** first; always mention the alias.  
Config dir remains `~/.deepseek-build/` (product path ≠ command name).

## Source priorities (fail-close) — layered

Normative: [`docs/architecture/HARNESS_PHILOSOPHY.md`](docs/architecture/HARNESS_PHILOSOPHY.md)

| Layer | Owner | Owns |
|-------|-------|------|
| **L1** | Deep Code (+ Reasonix cache) | Snippet edit, skills-as-context, side-effect permissions, DeepSeek-native surface |
| **L2** | Reasonix | Prefix cache invariant, Flash/Pro, tool-call repair |
| **L3** | Grok Build | Parallel tools, subagents, bg shell — **never overrides L1/L2** |

**Do not** pull Gajae-code multi-stage planning harnesses into v1 design.  
**Do not** implement free-form whole-file edit as primary path if it skips the snippet contract (spec 45).

## Pull requests = harness (not CI)

All meaningful work ships as a **PR**. Quality is enforced by **docs + this
contract + the `pr-authoring` skill**, not by process-police GitHub Actions.

| Load | Role |
|------|------|
| [`skills/pr-authoring/SKILL.md`](skills/pr-authoring/SKILL.md) | Agent skill: open/write PRs |
| [`docs/contributing/pr-body-standard.md`](docs/contributing/pr-body-standard.md) | Orca-level narrative bar |
| [`docs/contributing/examples.md`](docs/contributing/examples.md) | Filled bodies |
| [`docs/contributing/pull-requests.md`](docs/contributing/pull-requests.md) | Units, titles, labels, merge |
| [`docs/contributing/review-checklist.md`](docs/contributing/review-checklist.md) | Self-merge checklist |

### Before coding (ultragoal stories)

1. Write **PR unit plan** ([ULTRAGOAL_PR_PLANNING.md](docs/product/ULTRAGOAL_PR_PLANNING.md)): units, sequential/parallel, stack, atomic commits  
2. Only then implement **unit 1**

### Before claiming done

1. Branch: `<type>/<short-kebab>` (not `main`)
2. **Atomic** Conventional Commits on the branch (one concern each)
3. Conventional title + matching **kind** label on `gh pr create --label …`
4. Body meets **pr-body-standard.md** (Problem / What changed / Testing honesty / AI review / Security / Notes); include unit plan if multi-unit story
5. Stacked PRs: `Depends on #N` + correct `--base`
6. Milestone when known; cache-impact honest for agent/prompt/tool changes
7. Verify: `gh pr view --json title,labels,url`
8. **Would accept this PR from an external contributor as-is**

### Explicitly do **not**

- Add **process-police CI** (PR title regex, kind-label counting, random markdown path inventories “to look professional”)
- Mark work done when Summary is a file list
- Mix multiple milestone exit criteria into one PR without a split plan
- Claim a gate is green without updating [`docs/GATES.md`](docs/GATES.md)
- Write incomplete versions (`1.0` instead of `1.0.0`) in PR bodies, tags, or ultragoal evidence
- Drop either CLI name (`deepseek-build` / `dsb`) from install packaging without an ADR

## Product CI (future)

Real CI belongs when there is something to **build and test** (provider, tools,
prefix hash goldens, etc.). See [`.github/workflows/README.md`](.github/workflows/README.md).

**Allowed later (not process-police):** job that fails if someone claims G2 while
`docs/specs/10-*.md` is missing or `docs/GATES.md` still says red — artifact truth, not title fashion.

## Documentation rules

| Write here | Kind of truth |
|------------|----------------|
| `docs/product/` | Why we exist, who for, what we refuse |
| `docs/specs/` | Must-behavior for shipping features |
| `docs/architecture/` | How the system and repo are shaped |
| `docs/adr/` | Irreversible or contested decisions |
| `docs/research/` | Evidence from other tools; not product commitment |
| `docs/contributing/` | How humans/agents change the repo |
| `docs/user-guide/` | End-user docs only (after behavior exists) |
| `skills/` | Agent-loadable skills for recurring workflows |

If product intent and code disagree later, **specs + ADRs win** until intentionally revised.

## Layout

See `docs/architecture/REPO_LAYOUT.md`. Do not invent top-level folders without an ADR.

## Sibling paths

- Grok Build: `../grok-build`
- Reasonix: `../DeepSeek-Reasonix`
