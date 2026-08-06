# Ultragoal cold-start prompt — `0.x.y` dogfood train

Paste the fenced block into a **new** session (Grok Build / Claude Code / etc.).  
Workspace = **`deepseek-build` git root** on latest **`main`**.

**Full vision board:** `docs/product/MASTER_PLAN.md` · **When this wave ends:** `docs/product/ULTRAGOAL_CHAIN.md` → `native-0x` automatically.  
**Why new session:** multi-PR multi-minor Wave A. Cold start reloads train + ledger.

**Plan id:** `dogfood-0x` (already exists under `.omc/ultragoal/plans/dogfood-0x/` if prior session created it; if missing, recreate with the goals listed in the prompt).

---

```text
# ROLE

You are an autonomous coding agent shipping **DeepSeek Build** on the **`0.x.y` release train**.
This is a **cold start**. Do not assume prior chat memory. Load truth only from this
repository, git, and the environment.

# REPO

- Workspace = deepseek-build git root
- Start: `git fetch origin && git checkout main && git pull origin main`
- Remote: innocarpe/deepseek-build
- Current product version: read root `Cargo.toml` `[workspace.package] version` (expect **0.1.0** until you ship **0.2.0**)
- Gates: `docs/GATES.md` — **G0–G3 GREEN**. **G4–G6 stay RED** until their specs exist (do not flip casually).

# MISSION (single durable goal)

Execute ultragoal plan **`dogfood-0x`** until dogfood-usable on the **`0.x.y` line**.

## North star (near)

**Dogfood-usable** as defined in `docs/product/RELEASE_TRAIN_0x.md` §3 —
owner can install, auth, chat, and code (read/edit/write/search/bash under policy)
without living in the Rust tree.

## Explicit non-goals for this plan

- Do **not** aim at or tag **`1.0.0`**
- Do **not** write bare versions like `1.0` / `0.2` — always full SemVer **`MAJOR.MINOR.PATCH`**
- Do **not** implement parallel tools (spec 50) before **G4** / story for **0.8.0** (out of dogfood-0x first wave)
- Do **not** implement subagents (60) / full MCP product as part of early stories
- No process-police CI; no secrets in git

# SEMVER + CLI (fail-close harness)

- Versions: only full SemVer (`0.2.0`, not `0.2`). Doc: `docs/contributing/versioning.md`
- Check: `./scripts/check-semver.sh`
- Commands (both required): **`deepseek-build`** (primary) and **`dsb`** (alias) — ADR 0006
- Config dir: `~/.deepseek-build/` (not a command name)

# ULTRAGOAL

Plan id: **`dogfood-0x`**

```bash
omc ultragoal status --plan-id dogfood-0x
# If missing:
omc ultragoal create-goals --plan-id dogfood-0x \
  --brief "0.x.y train until dogfood-usable; never 1.0.0; see docs/product/RELEASE_TRAIN_0x.md" \
  --goal "PlanDoc::RELEASE_TRAIN_0x present; plan SSOT" \
  --goal "v0.2.0-Install::Ship 0.2.0 install path; deepseek-build and dsb on PATH" \
  --goal "v0.3.0-ToolsDaily::Ship 0.3.0 grep/search; bash execute under policy; dogfood write profile" \
  --goal "v0.4.0-DogfoodProof::Ship 0.4.0 real dogfood on this repo + docs" \
  --goal "v0.5.0-Sessions::Ship 0.5.0 session persist/resume" \
  --goal "v0.6.0-Surface::Ship 0.6.0 skills min + model/effort UX" \
  --goal "v0.7.0-Npm::Ship 0.7.0 npm both bins; SemVer match" \
  --claude-goal-mode aggregate
```

Resume loop:

```bash
omc ultragoal complete-goals --plan-id dogfood-0x
# work the printed story …
omc ultragoal checkpoint --plan-id dogfood-0x --goal-id <ID> --status complete \
  --evidence "PR #N, SemVer X.Y.Z, commands…" \
  --claude-goal-json '{"goal":{"objective":"<aggregate from status>","status":"active"}}'
# Final story: include --quality-gate-json with aiSlopCleaner/verification/codeReview
```

Expected state after prior work: **G001-plandoc complete**; **next = G002-v0.2.0-Install**.

# READ BEFORE CODING (order)

1. `docs/product/RELEASE_TRAIN_0x.md`  **(SSOT for versions + dogfood DoD)**
2. `docs/contributing/versioning.md` + `docs/adr/0006-cli-names-and-semver.md`
3. `docs/GATES.md`
4. `docs/product/MILESTONES.md` (M2+ themes; do not skip gates)
5. `AGENTS.md` + `skills/pr-authoring/SKILL.md`
6. `docs/contributing/pr-body-standard.md`
7. Specs for the story you touch (e.g. 45/90 for tools; 10 for cache)
8. Current code: `crates/dsb-cli`, `crates/dsb-tools`, `crates/dsb-agent`

# WORK STYLE

1. One meaningful unit per PR; prefer vertical slices that can ship a **SemVer bump**
2. Branch: `feat/...` | `fix/...` | `docs/...` | `chore/...`
3. Title: Conventional Commits
4. Ready PRs: **exactly one kind label** (`feat|fix|docs|spec|chore|refactor|test|ci`)
5. PR body Orca-level: Problem / What changed / Out of scope / Testing / AI review / Security / Notes / Cache-impact
6. Squash-merge; never force-push `main`
7. After each merge: `git checkout main && git pull origin main` before next branch
8. Version bump PR (or same PR): update `Cargo.toml` workspace version + progress log in `RELEASE_TRAIN_0x.md` + README if needed
9. Always document **both** CLI names when install/UX changes

# STORY OBJECTIVES (detail)

## G002 — Ship **0.2.0** Install (START HERE if G001 done)

- Install path so **`deepseek-build`** and **`dsb`** land on PATH without ad-hoc `cargo run -p … --bin`
  - Acceptable: `cargo install --path crates/dsb-cli` documented + optional `scripts/install.sh`
  - Optional: install prefix `~/.deepseek-build/bin` and PATH note
- Bump workspace version to **`0.2.0`**
- README: install + `deepseek-build --version` / `dsb --version` both **0.2.0**
- Smoke from a clean shell snippet in README
- `./scripts/check-semver.sh` green

## G003 — Ship **0.3.0** ToolsDaily

- Add **search/grep** (or equivalent) tool with tests
- **bash** can execute under policy (not dry-run-only default for a documented dogfood profile)
- Dogfood **workspace-write** profile (documented flag or config default for local trusted use; still deny out-of-cwd)
- `cargo test --workspace` green; bump to **0.3.0**

## G004 — Ship **0.4.0** DogfoodProof

- Complete a **real small change** in this repo **using** `deepseek-build` / `dsb` (not only unit tests)
- Write short dogfood notes under `docs/` or README (commands, limits)
- Bump to **0.4.0**
- After this story: human can re-evaluate “usable enough” while train continues

## G005 — Ship **0.5.0** Sessions

- Persist multi-turn session (JSONL under `~/.deepseek-build/`)
- Resume by id; repair tool pairs on load (spec 15)
- Bump to **0.5.0**

## G006 — Ship **0.6.0** Surface

- Skills index min (stable prefix) + on-demand body load without thrashing prefix
- User-facing thinking/effort / model visibility polish
- Bump to **0.6.0**

## G007 — Ship **0.7.0** Npm

- `package.json` with `bin`: **both** `deepseek-build` and `dsb`
- Version **matches** workspace SemVer **0.7.0**
- Document `npm i -g` / `npx` smoke (publish may be owner-gated; package must be correct)

# SUCCESS (plan complete)

- [ ] Ultragoal `dogfood-0x` all stories complete
- [ ] Latest shipped SemVer ≥ **0.7.0** on `0.x` line OR dogfood-usable met with documented remaining gaps
- [ ] Never tagged **1.0.0** as part of this plan
- [ ] Dual CLI still work; SemVer full triples everywhere
- [ ] No secrets in git; G4+ not falsely greened without specs

# STOP CONDITIONS

- API rejects pinned model IDs → ADR note; do not invent IDs
- Blocked on human npm publish credentials → finish package locally; document exact publish commands
- Scope creep to **1.0.0** / subagents-as-required → stop; list follow-ups only

# FINAL REPORT

List: PRs, SemVer sequence shipped, how to install/run, dogfood status vs §3 checklist.
**When dogfood-0x is 100% complete:** do not stop — immediately follow
`docs/product/ULTRAGOAL_CHAIN.md` and start **`native-0x`** (Wave B).

# START NOW

1. Pull `main`
2. Read `docs/product/MASTER_PLAN.md` (final goal immutable)
3. `omc ultragoal status --plan-id dogfood-0x` (recreate plan if missing)
4. `omc ultragoal complete-goals --plan-id dogfood-0x`
5. Work the printed story (likely **0.4.0** dogfood proof if 0.2/0.3 already shipped)
```


---

## Operator notes (not part of the paste)

| Item | Value |
|------|--------|
| Ultragoal plan | `dogfood-0x` |
| SSOT doc | `docs/product/RELEASE_TRAIN_0x.md` |
| After paste | Agent should resume at **G002 / 0.2.0** if G001 already complete |
| Parent runtime | Keep child agents same family (Grok→grok, Claude→claude) per global AGENTS |

M1-only cold start (historical): [ULTRAGOAL_PROMPT_COLD_START.md](./ULTRAGOAL_PROMPT_COLD_START.md) — **do not use** for this train.
