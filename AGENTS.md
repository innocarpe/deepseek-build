# DeepSeek Build — agent contract

This file is standing instructions for any coding agent working in this repo.

## Current phase

**`0.x.y` release train** toward **dogfood-usable** (not `1.0.0` yet).  
Normative plan: [`docs/product/RELEASE_TRAIN_0x.md`](docs/product/RELEASE_TRAIN_0x.md).  
Current product version: **`0.2.0`** (installable CLI). Next target theme: **`0.3.0`** coding tools daily.

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

### Before claiming done

1. Branch: `<type>/<short-kebab>` (not `main`)
2. Conventional title + matching **kind** label on `gh pr create --label …`
3. Body meets **pr-body-standard.md** (Problem / What changed / Testing honesty / AI review / Security / Notes)
4. Milestone when known; cache-impact honest for agent/prompt/tool changes
5. Verify: `gh pr view --json title,labels,url`
6. **Would accept this PR from an external contributor as-is**

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
