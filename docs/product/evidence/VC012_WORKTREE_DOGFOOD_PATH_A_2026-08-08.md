# VC012 — Worktree dogfood + bare `dsb` honesty (Path A)

| Field | Value |
|-------|--------|
| **Story** | **VC012** — public **Path A** worktree dogfood + bare `deepseek-build`/`dsb` session honesty (vision **V3-WT**) |
| **Plan** | `vision-complete-5x` |
| **Date** | 2026-08-08 |
| **Status** | **READY** — Path A worktree CLI dogfood + opt-in stamp + headless no-create green; gates green; unversioned; stacked on #143 |
| **SemVer** | **none** (this story does **not** bump product version) |
| **Depends on** | **VC011** subagent + worker cache Path A R0A (open PR **#143** `vc011-subagent-worker-cache`) |
| **Board** | [`VISION_COMPLETE_5X_GOALS.md`](../VISION_COMPLETE_5X_GOALS.md) · DAG [`WAVE_5x_VISION_PR_DAG.md`](../WAVE_5x_VISION_PR_DAG.md) |
| **Normative** | Spec **60** non-goal “mandatory worktree for every implement worker” · [`HARNESS_PHILOSOPHY.md`](../../architecture/HARNESS_PHILOSOPHY.md) optional worktree isolation · user-guide [`13-worktrees.md`](../../user-guide/13-worktrees.md) · L3 matrix worktree row · owner-bar **L3-WT-1/2** · vision **V3-WT** |
| **Prior** | G010 offline L3.0/L3.4 help + stamp honesty ([`G010_L3_UNDER_HEARTS_2026-08-07.md`](./G010_L3_UNDER_HEARTS_2026-08-07.md)); VC011 residual “worktree dogfood → VC012” ([`VC011_SUBAGENT_WORKER_CACHE_PATH_A_2026-08-08.md`](./VC011_SUBAGENT_WORKER_CACHE_PATH_A_2026-08-08.md)) |

**This file is the mandatory ultragoal PR unit plan for VC012 plus (later) implementation evidence.**
It does **not** claim VISION L3 / **5.4.0** freeze complete. It does **not** make worktree mandatory for implement workers (Spec 60 non-goal). Live interactive TTY worktree create remains residual unless a hermetic non-TTY path is proven.

---

## 0. Floor and dependency facts

### 0.1 Live floor (story open; 2026-08-08)

| Probe | Live result |
|-------|-------------|
| This worktree branch | `vc012-worktree-dogfood` (forked at VC011 tip `fd9b215`) |
| Stack base for feature commits / PR base | **`vc011-subagent-worker-cache`** (open PR **#143**); **not** `origin/main` until after #143 merges |
| `git show origin/main:Cargo.toml` version | **`5.2.2`** |
| `npm view @innocarpe/deepseek-build version` | **`5.2.2`** |
| `gh release list` Latest | **`v5.2.2`** |
| Working tree product version (stack tip) | **`5.3.0`** (from VC006 cut on stack; carried through VC007–VC011) |
| Board text residual | Track C target minor still **`5.4.0`** for VC010–VC013; this story is **unversioned** evidence only |
| G010 residual | L3-WT-1 was **help-only** (L3.0/L3.4); public-entry worktree dogfood beyond help was **weak** |
| VC011 residual | Explicitly deferred **VC012** worktree dogfood + bare `dsb` honesty |

### 0.2 Floor interpretation (fail-close)

- **Live product floor is `origin/main` = `5.2.2`.** npm + GitHub Release aligned at open.
- Stack already carries product **`5.3.0`**. **Do not bump SemVer** on VC012. Do **not** cut **`5.4.0`** here (that is VC013).
- Board’s “VC012 part of 5.4.0” remains true as **train membership**, not as this PR’s packaging action.
- **Open as one stacked PR** with base **`vc011-subagent-worker-cache`** and body **`Depends on #143`**. **Do not merge** this PR in-story.
- Claims must be **public Path A only**: `deepseek-build` / `dsb` → product agent (no `DEEPSEEK_BUILD_AGENT_BIN` override for the public-entry claim). Thin unit greens may support honesty but must not be sole proof.
- **Worktree remains opt-in.** Closing VC011’s residual “mandatory worktree for every implement worker → VC012” means **document + prove opt-in dogfood**, not force isolation on every implement spawn (Spec 60 non-goal).

---

## 1. Why this PR (one sentence)

Close the Grok L3 **V3-WT** gap that G010 left help/stamp-only and VC011 deferred: prove on **public Path A** that worktree CLI is dogfoodable via `deepseek-build`/`dsb`, that bare product launch is **single-session** with **`worktree_product=opt_in`**, and that headless `-p --worktree` **honestly does not create** a worktree — without claiming interactive TTY sole proof or a SemVer cut.

### 1.1 Scope amendment — minimal product top-level `--worktree` (plan delta)

**Discovery (implementation):** Pre-existing public docs and L3 honesty already described opt-in worktree as a product surface (`dsb --worktree=…`, user-guide 13, L3-WT / V3-WT). Inspection showed the **public product CLI did not parse** top-level `--worktree` / `--worktree-ref` (clap rejected them); only agent trailing args after `dsb agent -- …` reached the binary. That made the **documented public contract false** without a code change.

**Decision (minimal, documented):** Include a **thin product CLI forward** so the pre-existing public contract is true:

| Item | Acceptance |
|------|------------|
| Parse | Product accepts `--worktree [NAME]` / `-w [NAME]` and `--worktree-ref REF` on bare TTY and `agent` paths only |
| Forward | `tui_forward_flags` emits the same tokens to the agent after product stamps / before `exec` |
| Line-mode reject | `run` / `chat` / other non-TUI paths reject these flags (`reject_tui_only_flags`) — not silent no-ops |
| Dual syntax | **Still valid:** `deepseek-build agent -- --worktree NAME` (existing agent trailing path) |
| Parser tests | Unit: `tui_forward_flags_worktree_opt_in`, `reject_worktree_flags_on_line_mode` |
| R0A | `worktree-flag-forward` (stub argv after exec) + product `--help` lists `--worktree`; headless no-create uses product top-level + `-p` |

**Why not residual-only:** Recording “top-level flag absent” while docs claim `dsb --worktree` would ship a known contract lie. The delta is **minimal** (parse + forward + reject; no worktree create logic in product). Interactive TTY create remains residual (process boundary).

**Out of scope for this delta:** Changing vendor headless create behavior; making implement workers default to `isolation=worktree`; SemVer bump.

---

## 2. Call-path map (inspected before design)

| Layer | Path | Role |
|-------|------|------|
| Public product | `deepseek-build` / `dsb` → `agent_launch::exec_agent` → agent bin | **Path A** product default |
| Product top-level (scope amendment) | `--worktree [NAME]`, `-w`, `--worktree-ref` | Parsed by product CLI; forwarded on bare/`agent` only; rejected on line-mode |
| Public agent args | `dsb agent -- <agent flags…>` trailing args | Existing path: still forwards `--worktree`, `worktree` subcommand, `-p`, etc. |
| Bare TTY launch | `dsb` / `deepseek-build` (no worktree flags) | Single-session TUI; **does not** auto-create worktree |
| Agent flags | `-w, --worktree [NAME]`, `--worktree-ref` | Interactive create; **headless `-p` ignores create** (vendor honesty) |
| Agent subcommand | `worktree list\|show\|rm\|gc\|db` | Manage tracked worktrees (offline-capable list) |
| Product stamp | `stamp_path_a_l3` → `path_a_l3.txt` | `worktree_product=opt_in` + `bare_dsb_session=single` on every public launch |
| Offline L3 smoke | `scripts/test-l3-smoke.sh` L3.0/L3.4 | Raw agent help only; **not** public `deepseek-build` entry |
| User guide | [`docs/user-guide/13-worktrees.md`](../../user-guide/13-worktrees.md) | Must match actual product + agent syntax |
| KNOWN_LIMITS | bare `dsb` single-session / worktree opt-in | Present; keep aligned |
| Spec 60 | Non-goal: mandatory worktree for every implement worker | Isolation `worktree` remains **optional** on Path A `spawn_subagent` |

### Target VC012 R0A contract (Path A public product)

Public CLI → `agent_launch` → product agent (no `DEEPSEEK_BUILD_AGENT_BIN` for the public-entry claim) with hermetic home:

#### 2.1 Worktree CLI surface (V3-WT dogfood)

1. Resolve public `deepseek-build` (and `dsb` when present).
2. Hermetic `DEEPSEEK_BUILD_HOME` with a runnable agent binary under `bin/`.
3. Assert:
   - Product `deepseek-build --help` documents top-level `--worktree` (scope amendment §1.1).
   - `deepseek-build agent -- --help` documents agent `--worktree` / `worktree` subcommand.
   - `deepseek-build agent worktree --help` exits success-ish and mentions list/manage.
   - `deepseek-build agent worktree list --json --repo <git-repo>` exits 0 (empty array OK).
4. Dual-CLI: when `dsb` sits beside the public binary, run the same `worktree list --json` via `dsb agent …`.
5. Capture META + stdout tails under `docs/product/evidence/PATH_A_R0_VC012_*`.

#### 2.2 Opt-in honesty stamp (V3-WT / L3-WT-2)

1. Public-entry short hermetic agent turn (`-p`) via scripted DeepSeek (reuse `worker-cache-stamp` or a thin `worktree-opt-in-stamp` scenario).
2. Assert `path_a_l3.txt` under hermetic home contains:
   - `worktree_product=opt_in`
   - `bare_dsb_session=single`
   - (carry) `worker_epochs_match=true` + subagents enabled (L3 stamp integrity)
3. This is **public Path A** proof that product policy remains opt-in, not forced worktree on bare launch.

#### 2.3 Headless `--worktree` does not create (docs honesty) — **evidence-backed**

1. Snapshot `git worktree list --porcelain` for a disposable git repo **before**.
2. Run public `deepseek-build --worktree=vc012-headless-dogfood agent -p … --cwd <repo>` under hermetic home + scripted wire (product top-level flag).
3. Snapshot worktrees **after**.
4. Assert porcelain **identical** before/after, count unchanged, and name absent (headless honesty).
5. Turn still completes (scripted final token) so the claim is “ignored create”, not “launch failed”.
6. META records `claim_scope=headless_p_plus_product_worktree_no_git_worktree_create` and
   `process_boundary_residual=interactive_tty_worktree_create_not_asserted`.

#### 2.3b Product flag forward across process boundary (conservative bounded)

1. Hermetic home installs a **stub agent** that only records argv (no model, no worktree create).
2. Public `deepseek-build --worktree NAME --worktree-ref REF agent -p …` → product stamps then **exec**s agent.
3. Stub argv must contain `--worktree`, `NAME`, `--worktree-ref`, `REF`.
4. This bounds the product→agent handoff without claiming interactive TTY create.

#### 2.4 Docs honesty pass

1. User-guide **13** must state:
   - Bare `dsb` / `deepseek-build` is **single-session** (no auto worktree).
   - Worktree is **opt-in** via `--worktree` / interactive flows / `worktree` subcommand.
   - Headless `-p` **does not** create a worktree from `--worktree`.
   - Public entry examples use `deepseek-build agent …` / `dsb agent …` (not only raw agent bin).
2. `KNOWN_LIMITS` keep / tighten the bare-session residual row.
3. Do **not** claim implement workers always isolate to worktrees.

#### 2.5 Public-path only claims

| Allowed as Path A proof | Not Path A proof alone |
|-------------------------|------------------------|
| `deepseek-build agent worktree …` / `dsb agent worktree …` | Raw `xai-grok-pager worktree` without public CLI |
| `path_a_l3` stamp from public launch | Unit-only stamp tests without public entry |
| Headless no-create before/after git worktree diff | Interactive TTY create without harness |
| Dual CLI names when both bins present | README-only marketing |

### Explicit non-claims

| Non-claim | Residual |
|-----------|----------|
| Mandatory worktree for every implement worker | Spec 60 **non-goal**; isolation remains optional |
| Interactive TTY `--worktree` create sole green without harness | Needs real TTY / product interactive path; not this R0A sole claim |
| Live multi-agent fleet always using isolation=worktree | Product choice / optional spawn arg |
| SemVer / npm / GitHub Release **5.4.0** cut | **VC013** |
| Closing V3-60-3 parent snippet expire residual | Stays VC011 residual unless separate thin unit |
| Wall-clock proof worktree create is faster than shared cwd | Not required |

---

## 3. PR units (ordered atomic commits — **one** stacked PR)

This story ships as **one unversioned PR** with atomic Conventional Commits (not multiple stack slots).

### PR unit 1 — `docs(product): VC012 worktree dogfood Path A plan`

- **Intent:** Lock stack base, call-path map, acceptance matrix, SemVer non-claims before source edits.
- **Touches:** `docs/product/evidence/VC012_WORKTREE_DOGFOOD_PATH_A_2026-08-08.md`
- **Depends on:** VC011 stack tip (`fd9b215` / PR #143)
- **SemVer:** none
- **Tests:** n/a (docs)

### PR unit 1b — `feat(cli): minimal product --worktree forward` (scope amendment §1.1)

- **Intent:** Make documented public `dsb --worktree` / `--worktree-ref` true: parse + forward on bare/`agent`; reject on line-mode.
- **Touches:** `crates/dsb-cli/src/main.rs` (Cli fields, `tui_forward_flags`, `reject_tui_only_flags`, unit tests)
- **Depends on:** unit 1 (plan must record delta first — this amendment documents it)
- **SemVer:** none
- **Tests:** `cargo test -p dsb-cli tui_forward_flags_worktree reject_worktree_flags_on_line_mode`

### PR unit 2 — `test(scripts): hermetic Path A worktree dogfood harness`

- **Intent:** Public-entry R0A for worktree CLI surface, product flag-forward stub, opt-in stamp, headless no-create honesty.
- **Touches:** `scripts/test-path-a-vc012-r0a.sh`; optionally extend `scripts/lib/scripted_deepseek_server.py` if a dedicated stamp scenario is needed (prefer reusing `worker-cache-stamp` / short text).
- **Depends on:** unit 1b
- **SemVer:** none
- **Tests:** `./scripts/test-path-a-vc012-r0a.sh`

### PR unit 3 — `docs(user-guide): worktree public-entry + bare dsb honesty`

- **Intent:** Align user-guide 13 (+ KNOWN_LIMITS if needed) with proven Path A behavior and dual CLI names.
- **Touches:** `docs/user-guide/13-worktrees.md`; maybe `docs/product/KNOWN_LIMITS.md`
- **Depends on:** unit 2 (behavior proven first)
- **SemVer:** none
- **Tests:** n/a (docs)

### PR unit 4 — `docs(product): VC012 READY evidence + independent review`

- **Intent:** Fill READY section, wire/META pointers, gate table; land independent Grok review file.
- **Touches:** this evidence file (READY), `VC012_INDEPENDENT_REVIEW_2026-08-08.md`, `PATH_A_R0_VC012_*` artifacts
- **Depends on:** units 2–3 + green gates
- **SemVer:** none
- **Tests:** re-run R0A + gates in review lane

---

## 4. Acceptance matrix

| ID | Check | Evidence |
|----|-------|----------|
| **VC012-1** | Product `--help` lists top-level `--worktree`; agent help + list --json | R0A `worktree-cli-surface` |
| **VC012-2** | Dual CLI `dsb agent worktree list` when bin present | Same harness dual path |
| **VC012-3** | Product top-level `--worktree`/`--worktree-ref` in agent argv after exec | R0A `worktree-flag-forward` (stub) + unit parse/forward |
| **VC012-3b** | Line-mode `run`/`chat` reject product worktree flags | Unit `reject_worktree_flags_on_line_mode` |
| **VC012-4** | Public launch stamps `worktree_product=opt_in` + `bare_dsb_session=single` | R0A `worktree-opt-in-stamp` |
| **VC012-5** | Headless `-p` + product `--worktree` creates **no** new git worktree | R0A `worktree-headless-no-create` (porcelain identity) |
| **VC012-6** | Docs honesty matches actual syntax (product + agent paths) | user-guide 13 + KNOWN_LIMITS |
| **VC012-7** | Owner-bar / path-linkage / heart stay green | gate commands |
| **VC012-8** | No SemVer bump; stacked PR Depends on #143; not merged | `Cargo.toml` + `gh pr view` |
| **VC012-R1** | Interactive TTY worktree **create** after exec | **Residual** — process boundary; not asserted in hermetic R0A |
| **VC012-P** | Evidence META/WIRE `git_sha` = final source/docs head under test | Re-run R0A after final head; READY records SHA |

---

## 5. Gate commands (must stay green)

```bash
./scripts/check-path-a-linkage.sh
./scripts/test-owner-bar.sh
./scripts/test-heart-regression.sh
./scripts/test-path-a-vc012-r0a.sh
# support (not sole proof):
./scripts/test-l3-smoke.sh --offline-only
cargo test -p dsb-cli stamp_path_a_l3
```

Restore any generated TSV side-effects to HEAD if gates rewrite them. Clean vendor/agent `target/` build debris after agent builds if disk is tight (exact `target` dirs only).

---

## 6. READY evidence

**Status: READY** (implementation complete; independent review file sibling).

### 6.0 Provenance (fail-close)

| Field | Value |
|-------|--------|
| **Source/docs head under test** | `33552fb5bf465b4b909d22c00b07a2b3ab483ed3` |
| **Includes** | plan §1.1 scope amendment + CLI forward `bffc047` + harness + user-guide syntax |
| **R0A re-run** | `./scripts/test-path-a-vc012-r0a.sh --skip-build` after that head |
| **META/WIRE `git_sha`** | **must equal** source head above (all four scenario META files) |

Do **not** ship META that still cites plan-only `1a8e133` or a mid-stack READY tip without re-run.

### 6.1 Commands

| Command | Result |
|---------|--------|
| `./scripts/test-path-a-vc012-r0a.sh --skip-build` @ `33552fb` | **PASS** (cli-surface, flag-forward, opt-in-stamp, headless-no-create) |
| `cargo test -p dsb-cli tui_forward_flags_worktree` | **PASS** (support) |
| `cargo test -p dsb-cli reject_worktree_flags_on_line_mode` | **PASS** (support) |
| `cargo test -p dsb-cli stamp_path_a_l3` | **PASS** (support) |
| `./scripts/check-path-a-linkage.sh` | **PASS** |
| `./scripts/test-owner-bar.sh` | **PASS** (60/60; TSV side-effect restored) |
| `./scripts/test-heart-regression.sh` | **PASS** (PATH_A_E2E SKIP default; L3 offline PASS) |
| `git diff --check vc011-subagent-worker-cache...HEAD` | **0** (after final evidence commit) |
| SemVer on branch | **`5.3.0`** unchanged (no bump) |

### 6.2 Wire + stamp sample

| Scenario | Proof |
|----------|-------|
| `worktree-cli-surface` | Product `--help` lists `--worktree`; `agent -- --help` lists agent worktree flags; `agent worktree --help` + `list --json` via `deepseek-build` and `dsb` |
| `worktree-flag-forward` | Stub agent argv after `exec` contains product `--worktree`/`--worktree-ref` values; stamp present |
| `worktree-opt-in-stamp` | Public `-p` launch writes `path_a_l3` with `worktree_product=opt_in` + `bare_dsb_session=single` + `worker_epochs_match=true` |
| `worktree-headless-no-create` | Product `--worktree` + `-p`; scripted turn completes; git porcelain **identical** before/after; count/name unchanged |

Artifacts:

| Path | Role |
|------|------|
| [`PATH_A_R0_VC012_worktree-cli-surface_META_last.txt`](./PATH_A_R0_VC012_worktree-cli-surface_META_last.txt) | Public CLI worktree help/list |
| [`PATH_A_R0_VC012_worktree-flag-forward_META_last.txt`](./PATH_A_R0_VC012_worktree-flag-forward_META_last.txt) + ARGV | Bounded process-boundary flag forward |
| [`PATH_A_R0_VC012_worktree-opt-in-stamp_WIRE_last.jsonl`](./PATH_A_R0_VC012_worktree-opt-in-stamp_WIRE_last.jsonl) + META | Stamp honesty from public launch |
| [`PATH_A_R0_VC012_worktree-headless-no-create_WIRE_last.jsonl`](./PATH_A_R0_VC012_worktree-headless-no-create_WIRE_last.jsonl) + META | Headless no-create proof (porcelain) |
| [`PATH_A_R0_VC012_L3_last.txt`](./PATH_A_R0_VC012_L3_last.txt) | L3 stamp sample |
| [`VC012_INDEPENDENT_REVIEW_2026-08-08.md`](./VC012_INDEPENDENT_REVIEW_2026-08-08.md) | Independent review |

Path A L3 stamp sample:

```text
worker_epochs_match=true
subagents_enabled_in_config=true
worktree_product=opt_in
bare_dsb_session=single
```

### 6.3 What shipped

1. Product CLI **`--worktree` / `-w` / `--worktree-ref`** forward on bare TTY + `agent` paths (opt-in; rejected on line-mode).
2. **Conservative bounded** public R0A harness `scripts/test-path-a-vc012-r0a.sh` (four scenarios):
   - CLI surface, **stub flag-forward** (process boundary), opt-in stamp, **headless no-create porcelain**.
3. User-guide **13** + **KNOWN_LIMITS** honesty: bare single-session; headless no-create; implement workers not forced into worktree isolation.
4. Honest non-claims: no SemVer, no interactive TTY create sole green, no mandatory implement worktree, no **5.4.0** cut.

### 6.4 Explicit residuals at READY

| Residual | Recorded as |
|----------|-------------|
| Interactive TTY worktree **create** after `exec_agent` | META `process_boundary_residual=interactive_tty_worktree_create_not_asserted` (VC012-R1) |
| Optional `spawn_subagent` `isolation=worktree` live create | Spec 60 non-goal (not mandatory) |
| SemVer **5.4.0** cut | **VC013** |
| V3-60-3 Path A parent snippet expire | VC011 residual |

**Claim bound (no over-claim):** headless no-create is **git porcelain identity** on a disposable repo under public `-p` + product `--worktree`. Flag handoff is **stub argv after exec**. Neither asserts interactive TTY create.

### 6.5 Independent review

See [`VC012_INDEPENDENT_REVIEW_2026-08-08.md`](./VC012_INDEPENDENT_REVIEW_2026-08-08.md).

---

## 7. Stack / PR open checklist

- [x] Branch `vc012-worktree-dogfood` based on `vc011-subagent-worker-cache`
- [x] First commit = this plan (before source edits)
- [x] Atomic commits as §3 (plus READY docs)
- [ ] `gh pr create --base vc011-subagent-worker-cache` with **English** body + labels (`test` or `docs` + area label)
- [ ] Body includes **Depends on #143**
- [ ] `gh pr view --json title,labels,url` shows ≥1 kind label
- [ ] **Do not merge**; **do not bump SemVer** (still 5.3.0)

---

## 8. Orchestration provenance

| Field | Value |
|-------|--------|
| Run | `run_dae79ab6a005` |
| Task | `task_a1dc61b02a6e` |
| Dispatch | `ctx_5f3ae7ee52fb` |
| Worktree | `/Users/WooseongKim/Projects/deepseek-build/vc012-worktree-dogfood` |
| Stack head at open | `fd9b215` (VC011 tip) |
