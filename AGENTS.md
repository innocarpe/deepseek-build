# DeepSeek Build — agent contract

This file is standing instructions for any coding agent working in this repo.

## Current phase

**No active product ultragoal train.** The **`vision-complete-5x`** train is
completed on `main` at **`5.5.0`** and published to npm and GitHub Latest
(`5.5.0`, 2026-08-08). Do **not** re-plan
`5.0.1` through `5.5.0` as future feature cuts.

**Archived board:** [`docs/product/VISION_COMPLETE_5X_GOALS.md`](docs/product/VISION_COMPLETE_5X_GOALS.md) · DAG [`WAVE_5x_VISION_PR_DAG.md`](docs/product/WAVE_5x_VISION_PR_DAG.md)

**Completed trains:**
- **`vision-complete-5x`** → **`5.5.0`** (published to npm + GitHub Releases, 2026-08-08)
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
- **5.x** owner-bar **`5.0.0`** + vision-complete **`5.5.0` on `main`** — [PRD-v5](docs/product/PRD-v5.md) · [VISION_COMPLETE_5X_GOALS](docs/product/VISION_COMPLETE_5X_GOALS.md)


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

## Session output language (mandatory)

**Human-facing session text in this repository is Korean.** The maintainer reads
a session while it runs, so the rule covers everything a session puts on the
screen — not only the final answer.

| In scope | Example |
|----------|---------|
| Final answers | `핵심을 찾았습니다. 홈 전역이라 두 경우를 다 덮습니다` |
| Progress notes between tool calls | `이제 dsb가 지침을 읽는 자리 실측` |
| Tool-call descriptions (the one-liner on a call) | `README 구조 확인` |
| Plan lists, status reports, questions to the human | `계획 3단계로 갈까요?` |

**The opening line of a turn is in scope.** Sessions here habitually open with an
English sentence (`"I'll start by reading the brief."`) and switch later; that
opening line is exactly what this rule is for.

**Keep verbatim:** commands, paths, code, identifiers, API and package names,
error messages, and raw tool output. Translating those hurts readability — the
rule is about the sentences a session writes itself.

**Exceptions.** A turn that explicitly asks for another language wins. Repository
artifacts keep the repository's conventions: code, comments, commit messages, PR
bodies and the docs tree stay English.

Measured 2026-09-25 over this repo's session logs. Of the primary sessions that
had a standing Korean rule in context, 10 of 12 answered ≥99% of their text turns
in Korean. Sessions from the same day without one answered 0–1% (3 of 5). So the
rule works for the session a human is watching — but not automatically.

Two shapes still leaked, and they are different problems:

- **The opening line.** A turn starts with one English sentence and switches
  after the first tool call. That is the most common leak, which is why this
  section names it.
- **Delegated runs.** Subagent runs from the same batch, with near-identical
  English briefs, came back 0%, 0%, 1%, 98% — the same instruction produced very
  different adherence, so a rule document alone is a probabilistic control
  there. Delegated runs need the instruction in their brief as well
  (`skills/worktree-dispatch`).

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

## Control-tower checkout (parallel sessions)

Many agent sessions work on this repo at once. They are opened in the
**primary checkout** (the clone the maintainer works from), which acts as a
control tower: it directs, and **worktrees change code.**

This applies when sessions run in parallel through Orca worktrees. With a
single clone and no Orca, work as usual: branch, commit, PR
([`docs/contributing/`](docs/contributing/)).

| Place | Role | Writes |
|-------|------|--------|
| Primary checkout | Read, plan, review, dispatch, merge | **None.** Stays on `main`, clean; only `git pull --ff-only origin main` |
| Orca worktree | One unit of work | Code, commits, push — **one worktree = one branch = one PR** |

- **Do not edit or commit in the primary checkout.** Tower sessions share its
  index, so a stray edit or commit lands in another session's diff. A change to
  the harness itself (`AGENTS.md`, `skills/`) is a unit of work too: it gets a
  worktree.
- **Create, hand off and clean up worktrees with
  [`skills/worktree-dispatch`](skills/worktree-dispatch/SKILL.md)** — branch
  naming, opening an agent tab in the tree, the first-run trust prompt that
  swallows a brief, and removal after merge.
- **Operate on a worktree by path; do not `cd` into it and stay.**

  | Tool | Target it with |
  |------|----------------|
  | `git` | `git -C "$WT" …` |
  | `cargo`, `scripts/*.sh`, `npm` | subshell: `(cd "$WT" && cargo test -p dsb-cli)` |
  | `gh` | `--repo innocarpe/deepseek-build` — `gh` has no `-C` and otherwise reads the repo from cwd. `gh pr create` also takes its head branch from cwd, so add `--head <branch>` |

- **GitHub credentials per command.** When more than one `gh` account is logged
  in on the machine, pass the token of the account that can push here (for the
  maintainer, `innocarpe`) on each call:
  `GH_TOKEN="$(gh auth token --user innocarpe)" gh pr view 123 --repo innocarpe/deepseek-build`.
  Never `gh auth switch` — it changes the active account for every other
  session on the machine. `git push` goes to `origin`.
- **Vendored Grok builds run one at a time.** Each worktree has its own
  `target/`, and a cold build of `third_party/grok-build` takes 30–60+ min
  ([release-cycle.md](docs/contributing/release-cycle.md)); parallel builds
  starve each other. Anything that runs `cargo` in `third_party/grok-build` —
  directly or through a script (`build-grok-pager.sh`, `install.sh`,
  `test-grok-vendor-offline.sh`, the `test-path-a-*` scripts, …; check with
  `rg -l grok-build scripts/`) — goes serially across all worktrees. Units that
  only build and test `crates/` can run in parallel.
- **Leave other sessions' worktrees alone** — their files, branches and
  terminals. Ask the owning session or report instead.
- **Open work sessions as `deepseek-build` (`dsb`), not another coding agent.**
  Every session that changes this repo runs under the product this repo ships:
  its TUI, its tools, its cache behaviour. Claude Code, Codex and similar spend
  capacity that belongs to this product, and they hide the product's own gaps
  from the people who would fix them. Reading, planning and review may use any
  tool; the session that *writes the change* is `deepseek-build`.

## One session, one unit

The opening message, or the brief a dispatch handed over, is this session's
**one unit**. The order is [`skills/session-unit`](skills/session-unit/SKILL.md).
Worktree commands stay in [`worktree-dispatch`](skills/worktree-dispatch/SKILL.md);
the PR stays in [`pr-authoring`](skills/pr-authoring/SKILL.md).

- **Write the done-condition in one sentence before editing.** Take it from
  the opening. Do not later swap it for a smaller goal that is only what this
  session can finish easily, and do not add work the opening did not name.
- **Carry the unit through without waiting for another prompt** when the
  opening asked for the work: implement, run the checks that change needs,
  one-concern commits, then the PR. Stop short of push or PR only when the
  opening said to stop there. Merge only when the opening granted it, using
  the merge method this file states under **Merge on GitHub**.
- **A defect in a file this unit is already changing**, which no other
  session is editing, is part of finishing — its own commit, same unit.
  Anything the opening did not name (a new behavior, a fresh investigation,
  a drive-by in a file this unit is not already changing) is the **next**
  unit. Do not start it here. Open a new worktree and a `deepseek-build`
  tab for it, with that unit's done-condition as the first line of the
  brief. Report the tab. Do not ask whether to open it.
- **When the unit is done, say so first.** Name whether the done-condition
  holds, the evidence, and any unit you handed off. Do not offer another
  unit in this session.

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

`CLAUDE.md` is a symlink to this file, and `.claude/skills` and `.agents/skills`
are symlinks to `skills/` ([ADR 0011](docs/adr/0011-agent-harness-links.md)), so
Claude Code, Codex and DeepSeek Build load this contract and every skill in any
checkout or worktree. Edit `AGENTS.md` and `skills/`, never the links.

## Sibling paths

- Grok Build: `../grok-build`
- Reasonix: `../DeepSeek-Reasonix`
