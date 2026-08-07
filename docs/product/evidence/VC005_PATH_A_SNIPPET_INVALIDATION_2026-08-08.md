# VC005 — Path A write / bash snippet invalidation laws

| Field | Value |
|-------|--------|
| **Story** | **VC005** — Path A Spec 45 write bypass + bash mutation invalidation for session-local `snippet_id` |
| **Plan** | `vision-complete-5x` |
| **Date** | 2026-08-08 |
| **Status** | **PLAN** (mandatory unit plan before source edits; implementation evidence appended later) |
| **SemVer** | **none** (no version bump in this story; does **not** cut any release minor) |
| **Depends on** | **VC004** Path A `search_replace` hard `snippet_id` require (branch stack / open PR **#135** `vc004-snippet-id-require`) which depends on **VC003** mint (**PR #130 MERGED**) |
| **Board** | [`VISION_COMPLETE_5X_GOALS.md`](../VISION_COMPLETE_5X_GOALS.md) · DAG [`WAVE_5x_VISION_PR_DAG.md`](../WAVE_5x_VISION_PR_DAG.md) (read live on `origin/main` for floor; see §0) |
| **Normative design** | [`docs/adr/0010-spec-45-snippet-store.md`](../../adr/0010-spec-45-snippet-store.md) **§6 / §6.1 / §6.2** invalidation + write/bash laws |
| **Semantics SSOT** | [`docs/specs/45-snippet-edit.md`](../../specs/45-snippet-edit.md) §1.5 write bypass · §1.6 invalidation · test plan `write_*` / `bash_mutation_expires_snippets` |
| **Binding** | [`HEART_3X_SPEC_BINDING.md`](../../architecture/HEART_3X_SPEC_BINDING.md) · [`HARNESS_PHILOSOPHY.md`](../../architecture/HARNESS_PHILOSOPHY.md) §4.1 |

**This file is the mandatory ultragoal PR unit plan for VC005 plus (later) implementation evidence.**
It does **not** claim VISION L1 complete, owner-bar re-cut, Path A multi-edit R0A, resume/fork table restore (VC006), any SemVer cut, packaging/release of **5.2.1**, or public R0A wire proof.

---

## 0. Floor and dependency facts

### 0.1 Live floor (re-check at story open; 2026-08-08)

| Probe | Live result |
|-------|-------------|
| This worktree branch | `vc005-snippet-invalidation` (started from VC004 tip; implements VC005 only on top) |
| Stack base for feature commits | VC004 tip on branch (includes post-rebase VC004 evidence; **not** yet on `main` at open) |
| `git show origin/main:Cargo.toml` version | **`5.2.1`** |
| `package.json` on `origin/main` | **`5.2.1`** (inherited; **not** bumped by this story) |
| `npm view @innocarpe/deepseek-build version` | **`5.2.0`** — lag behind `main` |
| `gh release list` Latest | **`v5.2.0`** — GitHub Release lag; **owned by separate release-5.2.1 lane** |
| Board text residual | Still may document Spec 45 cut as **`5.2.0`** (VC006) in places — **stale vs live main floor** |
| VC003 | **on main** via #130; Path A mint remains prerequisite |
| VC004 | **open stack** PR **#135** (require gate); this story stacks after it |
| Thin Path B | `crates/dsb-tools` `SnippetStore` remains **reference/oracle**, not Path A proof |

### 0.2 Floor interpretation (fail-close)

- **`5.2.0` and `5.2.1` are already used** on the product line (main product version **`5.2.1`**). This story **must not** reuse or cut either.
- npm/GitHub release lag for **5.2.1** is **out of scope** here (separate Grok release lane). **Do not** duplicate packaging.
- Remaining Spec 45 completion after VC005 belongs to the **next free feature minor**. With main at **`5.2.1`**, that remains **`5.3.0`** unless a later board/npm re-check shows another free `5.Y.0`.
- Feature PRs (VC003–VC005) stay **unversioned**. Only a dedicated cut unit (historical VC006 slot, rebased) bumps SemVer.
- Owner-bar **`file_version` (sha256)** remains a **compatibility alias** of snippet `version`; do not remove it.
- Safe open base when stacking is gone: **`main` after VC004 merges**; until then this branch remains a VC004→VC005 stack.

---

## 1. Why this PR (one sentence)

Close Spec 45 / ADR 0010 **§6 write + bash laws** on Path A so (1) empty-old / write-spirit cannot overwrite an existing path without host force policy (`path_exists_use_edit`), (2) successful path mutations **eagerly expire** session snippets for that path, and (3) mutating bash **expires known paths or all session file snippets** (fail-closed), without claiming R0A or a SemVer cut.

---

## 2. Call-path map (inspected before design)

| Layer | Path | Role today (pre-VC005) |
|-------|------|------------------------|
| Public product | `deepseek-build` / `dsb` → `deepseek-build-agent` (vendored Grok) | **Path A** product default |
| Product Standard toolset | `xai-grok-shell` `FileToolset::Standard::tool_configs` | Injects `snippet_safe: true` + `empty_old_string_does_not_override: true` on `search_replace` |
| Write spirit on Path A | **No free standalone `write` tool** on product Path A | Empty `old_string` on `search_replace` is create / overwrite spirit (G005 honesty) |
| Edit impl | `xai-grok-tools` `implementations/grok_build/search_replace` | VC004 hard `snippet_id` require + scope replace; **no eager `expire_path` after success** |
| Overwrite guard today | `empty_old_string_does_not_override` / `snippet_safe` | Blocks empty-old overwrite of **non-empty** existing files via `FileAlreadyExists` — **not** ADR stable name `path_exists_use_edit`; empty existing file still overwritable |
| Session store (VC003) | `types/snippet_store.rs` `SessionSnippetStore` | `issue` / `get` / multi-id; **no `expire_path` / `expire_all`** |
| Bash tool | `implementations/grok_build/bash` | Runs terminal; **no snippet invalidation hook** |
| Lazy stale (G005) | content-hash at next edit | Bash mutate → next edit `snippet_stale` without table purge — residual vs ADR eager expire |
| Thin oracle | `crates/dsb-tools` `SnippetStore` + `tools::bash` | `write_new` → `path_exists_use_edit`; `edit` → `expire_path`; mutating bash → `expire_all` after permission; **not** Path A proof |
| Dual CLI | `deepseek-build` / `dsb` | Must both keep working; no install rename |

### Target VC005 contract (Path A, product `snippet_safe == true`)

#### 2.1 Successful mutation expiry (ADR §6)

| Event | Action |
|-------|--------|
| Successful non-create `search_replace` (edit) | **`expire_path`** for the mutated absolute/resolved path — all session ids for that path become unusable (`snippet_not_found` on reuse) |
| Successful force-overwrite write spirit (host policy only) | Write allowed → **`expire_path`** for that path |
| Create-new path (path did not exist) | No prior snippets expected; `expire_path` is a no-op if called |
| Fail-closed edit/write errors | **No** disk mutation and **no** expiry |

#### 2.2 Write bypass law (ADR §6.1; Path A write spirit = empty `old_string`)

| Case | Behavior |
|------|----------|
| Path **does not exist** | Create allowed (existing create path); **no** `snippet_id` required |
| Path **exists** (including empty file) under product `snippet_safe` | Default **deny** empty-old overwrite → stable identity **`path_exists_use_edit`** in model-visible error; **bytes unchanged** |
| Force overwrite | Only if **host/policy** `SearchReplaceParams.allow_force_write_overwrite` (or equivalent host-only flag) is true — **not** a free model boolean. If granted: revalidate existing FS/permission gates, write, **`expire_path`** |
| Legacy non-`snippet_safe` | Unchanged Grok empty-old / `empty_old_string_does_not_override` behavior (product Standard keeps `snippet_safe: true`) |

#### 2.3 Bash / external mutation (ADR §6.2)

After bash (`run_terminal_command` / Path A bash tool) is **actually dispatched** (validation passed; backend run started or completed) and the command is classified as potentially file-mutating:

| Knowledge of touched paths | Action |
|----------------------------|--------|
| **Known** set (heuristic path extraction from command tokens / redirects / common mutators) | **`expire_path`** for each resolved path under cwd/workspace |
| **Unknown** mutation set (classifier says may-mutate but paths unclear) | **Fail closed:** **`expire_all`** session file snippets (M2 product default, matches thin spirit) |
| Read-only / non-mutating classification | **No** snippet expiry |
| Validation / permission-style reject **before** dispatch | **No** snippet expiry |

External editors / user edits remain **lazy** via version check (`snippet_stale`) — no mtime watcher required.

**Out of scope for VC005:** subagent parent invalidation, worktree cross-bind, resume/fork table persistence (VC006), public R0A multi-edit heart proof, SemVer cut.

---

## 3. PR unit plan (four sections)

Per [`ULTRAGOAL_PR_PLANNING.md`](../ULTRAGOAL_PR_PLANNING.md). **VC005 is one feature PR** — invalidation/write/bash laws only; no resume/fork; no SemVer bump; no R0A claim.

### 3.1 PR units (ordered)

#### PR unit 1 — `docs(product): VC005 Path A snippet invalidation plan + evidence` **(this file)**
- **Intent:** Lock dependency on VC004/#135, map live write/bash contracts, atomic units, acceptance matrix, security/cache boundaries, non-claims, floor facts **before** source edits.
- **Touches:** `docs/product/evidence/VC005_PATH_A_SNIPPET_INVALIDATION_2026-08-08.md` only
- **Depends on:** VC004 require gate (stack tip / #135)
- **SemVer:** none

#### PR unit 2 — `feat(tools): Path A snippet expire + write/bash invalidation laws`
- **Intent:** Add session store `expire_path` / `expire_all`; after successful Path A edit/force-write expire path; under `snippet_safe` deny empty-old overwrite of existing paths with `path_exists_use_edit` unless host force flag; after dispatched mutating bash expire known paths or all; keep dual CLI + no new deps.
- **Touches:** primarily `third_party/grok-build/.../types/snippet_store.rs`, `.../search_replace/`, `.../bash/` (+ minimal exports); **no** SemVer files
- **Depends on:** unit 1
- **SemVer:** none

#### PR unit 3 — `test(tools): VC005 write/bash invalidation fail-closed regressions`
- **Intent:** Focused unit tests for every acceptance check below (expire after edit, path_exists_use_edit, force host-only, bash known/unknown/read-only, no partial write, session isolation, out-of-scope guards).
- **Touches:** tests under `xai-grok-tools` (`vc005_*`); thin `dsb-tools` only as oracle green
- **Depends on:** unit 2
- **SemVer:** none

#### Forward mapping (out of this PR)

| Unit | Story | Status here |
|------|-------|-------------|
| VC006 / Spec 45 cut | heart + multi-edit R0A + SemVer cut of remaining Spec 45 | **not implemented**; cut at **next free minor** (live → **`5.3.0`**, not reused **`5.2.0`/`5.2.1`**) |
| release-5.2.1 packaging | npm/GitHub lag close | **not this story** |

### 3.2 Sequential vs parallel

#### Sequential (must order)

1. **VC004 / #135** → **VC005 unit 1 (docs)** — require gate before invalidation laws that assume ids exist.
2. **unit 1** → **unit 2** (expire + write/bash laws) → **unit 3** (tests).
3. **VC005** → **VC006 cut** — heart/R0A + SemVer only after laws land.

#### Parallel (safe concurrent)

- None on the same Path A `SessionSnippetStore` / `search_replace` success path / bash dispatch surface.
- Pure docs that do not redefine ADR 0010 §6 may proceed independently.

```text
VC003 (mint, #130) ──► VC004 (require, #135) ──► VC005 (this) ──► Spec 45 cut @ next free minor (live → 5.3.0)
```

### 3.3 Atomic commits (on `vc005-snippet-invalidation`)

```text
docs(product): VC005 Path A snippet invalidation plan + evidence
feat(tools): Path A snippet expire + write/bash invalidation laws
test(tools): VC005 write/bash invalidation fail-closed regressions
```

Optional follow-ups only if justified after gates:

```text
style(tools): cargo fmt Path A snippet invalidation sources
docs(product): record VC005 Path A invalidation gate evidence
```

| Do | Do not |
|----|--------|
| One concern per commit | Mix VC006 resume/fork / R0A / SemVer into this branch |
| Keep `file_version` mint/alias | Remove `file_version` or break dual CLI |
| Session-local Resources store only | Process-global / cross-session table |
| English Conventional subjects | Bump `Cargo.toml` / package SemVer / claim `5.2.0` or `5.2.1` cut |
| Host-only force flag | Free model boolean that skips version/edit safety |

### 3.4 Chaining / stacking

| Pattern | Choice for VC005 |
|---------|------------------|
| **Base** | VC004 stack tip until #135 merges; then rebase onto **`main`** |
| **Branch** | `vc005-snippet-invalidation` |
| **Merge order** | #130 (VC003) → #135 (VC004) → VC005 → Spec 45 cut (next free minor) |
| **Conflict lock** | Path A `SessionSnippetStore` expiry + `search_replace` success/write path + bash post-dispatch invalidation owned by VC005; resume/fork reserved for VC006 |

**Planned PR title (when opened later):** `feat(tools): Path A write/bash snippet invalidation laws`
**Label kind:** `feat`
**Body:** Problem / What changed / Testing honesty / AI review / Security / Notes; **Depends on #135** (VC004); SemVer none; does **not** cut **`5.2.0`/`5.2.1`**; next free feature minor under live floor is **`5.3.0`**.

---

## 4. Acceptance criteria (VC005 only)

| ID | Criterion | Pass condition |
|----|-----------|----------------|
| **VC005-A1** | Store exposes expire APIs | Unit: `expire_path` removes only matching path ids; `expire_all` clears table |
| **VC005-A2** | Successful edit expires path snippets | Unit: mint ≥1 id for path, valid edit → store no longer contains those ids; reuse → `snippet_not_found` class |
| **VC005-A3** | Other paths preserved on expire_path | Unit: edit path A leaves path B ids intact |
| **VC005-A4** | Create-new without id still ok | Unit: empty `old_string` missing path works under `snippet_safe` |
| **VC005-A5** | Existing path empty-old denied | Unit: `snippet_safe` + empty old + existing path → error containing **`path_exists_use_edit`**; **bytes unchanged** |
| **VC005-A6** | Empty existing file also denied under snippet_safe | Unit: zero-byte existing path + empty old → `path_exists_use_edit`; no write |
| **VC005-A7** | Host force overwrite | Unit: `allow_force_write_overwrite` + empty old + existing → write succeeds + path snippets expired; model cannot set force via input args |
| **VC005-A8** | Bash known-path mutate expires path | Unit: mint, dispatch mutating cmd with extractable path → that path expired; other path may remain |
| **VC005-A9** | Bash unknown mutate expires all | Unit: mint multiple paths, unknown mutator → table empty |
| **VC005-A10** | Bash read-only does not expire | Unit: `ls`/`echo` without redirect keeps ids |
| **VC005-A11** | Pre-dispatch bash reject does not expire | Unit: validation error before backend run leaves store intact |
| **VC005-A12** | No partial write on write deny | Unit: A5/A6 assert original bytes |
| **VC005-A13** | Session isolation | Unit: expire/edit in Resources A does not clear Resources B |
| **VC005-A14** | No SemVer / no VC006 behavior | Diff has no version bump; no resume/fork table restore; no public R0A claim |

### Explicit non-claims (fail-close)

- Does **not** implement resume/fork snippet table persist/restore (VC006).
- Does **not** prove Path A multi-edit R0A / heart regression under real `snippet_id` tables (VC006 / R0A).
- Does **not** cut **`5.2.0`**, **`5.2.1`**, **`5.3.0`**, or any other SemVer; does **not** bump product version; does **not** package npm/GitHub release.
- Does **not** re-plan board tracks on `main` (board residual still lists Spec 45 cut as 5.2.0 in places).
- Does **not** claim public `deepseek-build`/`dsb` wire harness R0A unless that harness is run and captured with honest labels.
- Does **not** add mtime watchers or external-editor eager detection (lazy `snippet_stale` remains).
- Does **not** invent a free model `force=true` on write that skips Spec 45 edit safety.
- Thin `dsb-tools` greens are **oracle only**, not Path A proof.
- Owner-bar G005 spirit (lazy hash stale + empty-old non-empty guard) remains; this story **adds** eager table expiry + ADR error identity under `snippet_safe`.

---

## 5. Security / cache boundaries

| Concern | Rule |
|---------|------|
| Spec 10 stable prefix | Snippet table **must not** appear in stable-prefix bytes (session `Resources` only) |
| Cross-session leakage | IDs + expiry bind to owning session `Resources` / `SharedResources`; no process-global map |
| Permission | Existing Spec 90 / path policy gates stay before mutation where present; this story does not weaken them |
| Force overwrite | Host/policy resource only — never a model-trusted free boolean on tool input |
| Bash fail-closed | Unknown mutators expire **all** session file snippets rather than guessing |
| Pre-dispatch deny | Validation failures must **not** expire snippets |
| Dual CLI | No change to `deepseek-build` / `dsb` packaging names |
| Dependencies | **No new crates**; reuse VC003/VC004 helpers + existing sha256/uuid already in tree |

---

## 6. Validation commands

```bash
# Floor re-check
git fetch origin main
git show origin/main:Cargo.toml | rg 'version = "'
npm view @innocarpe/deepseek-build version
gh release list -R innocarpe/deepseek-build --limit 8

# Whitespace / conflict markers
git diff --check
git status --short
git diff --stat

# Path A unit (xai-grok-tools)
cd third_party/grok-build
cargo test -p xai-grok-tools --lib vc005
cargo test -p xai-grok-tools --lib vc004
cargo test -p xai-grok-tools --lib vc003
cargo test -p xai-grok-tools --lib snippet_safe
cargo test -p xai-grok-tools --lib search_replace::tests

# Format
cargo fmt --manifest-path third_party/grok-build/Cargo.toml -p xai-grok-tools -- --check

# Thin oracle still green (reference; not Path A proof) — from repo root
cargo test -p dsb-tools snippets
cargo test -p dsb-tools path_a_edit

# Required project gates
./scripts/test-owner-bar.sh
./scripts/check-path-a-linkage.sh
./scripts/test-heart-regression.sh

# Restore generated gate TSV side-effects to HEAD (do not commit)
git checkout HEAD -- docs/product/evidence/OWNER_BAR_STATUS.tsv \
  docs/product/evidence/PATH_A_R0_G010_HEART_REGRESSION_last.tsv 2>/dev/null || true
```

**R0A public wire:** not claimed in this story unless a later amendment appends wire artifacts with honest labels.

---

## 7. Implementation evidence (filled after code)

### 7.1 Atomic commits on `vc005-snippet-invalidation`

| Order | SHA (prefix) | Subject | Contents honesty |
|------:|--------------|---------|------------------|
| 1 | *(this commit)* | `docs(product): VC005 Path A snippet invalidation plan + evidence` | Plan only |
| 2 | _TBD_ | `feat(tools): Path A snippet expire + write/bash invalidation laws` | Contract/impl only |
| 3 | _TBD_ | `test(tools): VC005 write/bash invalidation fail-closed regressions` | Focused tests only |
| 4+ | _TBD_ | style / gate evidence if needed | no scope broaden |

**No VC006 behavior** in these commits: no resume/fork table restore, no public R0A harness, no SemVer bump, no release packaging.

### 7.2 What shipped (code) — _pending_

### 7.3 Acceptance matrix — _pending_

### 7.4 Commands actually run — _pending_

### 7.5 Independent adversarial review — _pending_

---

## 8. References

- ADR: [0010-spec-45-snippet-store](../../adr/0010-spec-45-snippet-store.md) §6.1–6.2
- Spec: [45-snippet-edit](../../specs/45-snippet-edit.md)
- VC004 evidence: [VC004_PATH_A_SNIPPET_ID_REQUIRE_2026-08-08.md](./VC004_PATH_A_SNIPPET_ID_REQUIRE_2026-08-08.md)
- VC003 evidence: [VC003_PATH_A_SNIPPET_ID_2026-08-08.md](./VC003_PATH_A_SNIPPET_ID_2026-08-08.md)
- Prior spirit: [G005_WRITE_BASH_INVALIDATE_2026-08-07.md](./G005_WRITE_BASH_INVALIDATE_2026-08-07.md)
- Path A code: `third_party/grok-build/.../search_replace/`, `.../bash/`, `.../types/snippet_store.rs`
- Thin oracle: `crates/dsb-tools/src/snippets.rs`, `tools.rs`
- Planning: [ULTRAGOAL_PR_PLANNING](../ULTRAGOAL_PR_PLANNING.md) · [WAVE_5x_VISION_PR_DAG](../WAVE_5x_VISION_PR_DAG.md)
