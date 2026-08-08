# VC005 — Path A write / bash snippet invalidation laws

| Field | Value |
|-------|--------|
| **Story** | **VC005** — Path A Spec 45 write bypass + bash mutation invalidation for session-local `snippet_id` |
| **Plan** | `vision-complete-5x` |
| **Date** | 2026-08-08 |
| **Status** | **IMPLEMENTATION** (runtime write/bash invalidation on Path A; unit evidence) |
| **SemVer** | **none** (no version bump in this story; does **not** cut any release minor) |
| **Depends on** | **VC004** Path A `search_replace` hard `snippet_id` require (branch stack / open PR **#135** `vc004-snippet-id-require`) which depends on **VC003** mint (**PR #130 MERGED**) |
| **Board** | [`VISION_COMPLETE_5X_GOALS.md`](../VISION_COMPLETE_5X_GOALS.md) · DAG [`WAVE_5x_VISION_PR_DAG.md`](../WAVE_5x_VISION_PR_DAG.md) (read live on `origin/main` for floor; see §0) |
| **Normative design** | [`docs/adr/0010-spec-45-snippet-store.md`](../../adr/0010-spec-45-snippet-store.md) **§6 / §6.1 / §6.2** invalidation + write/bash laws |
| **Semantics SSOT** | [`docs/specs/45-snippet-edit.md`](../../specs/45-snippet-edit.md) §1.5 write bypass · §1.6 invalidation · test plan `write_*` / `bash_mutation_expires_snippets` |
| **Binding** | [`HEART_3X_SPEC_BINDING.md`](../../architecture/HEART_3X_SPEC_BINDING.md) · [`HARNESS_PHILOSOPHY.md`](../../architecture/HARNESS_PHILOSOPHY.md) §4.1 |

**This file is the mandatory ultragoal PR unit plan for VC005 plus implementation evidence.**
It does **not** claim VISION L1 complete, owner-bar re-cut, Path A multi-edit R0A, resume/fork table restore (VC006), any SemVer cut, packaging/release of **5.2.0** / **5.2.1** / **5.2.2**, or public R0A wire proof.

---

## 0. Floor and dependency facts

### 0.1 Live floor (re-check at story open; 2026-08-08)

| Probe | Live result |
|-------|-------------|
| This worktree branch | `vc005-snippet-invalidation` (based on VC004 branch `vc004-snippet-id-require`; implements VC005 only on top) |
| Stack base for feature commits / PR base | **`vc004-snippet-id-require`** (open PR **#135**); **not** `origin/main` until after #135 merges |
| `git show origin/main:Cargo.toml` version | **Close-time floor `5.2.2`** at `d71c1b3` (historical open-time probe was **`5.2.1`**) |
| `package.json` on `origin/main` | **`5.2.2`** at close (historical open-time was **`5.2.1`**); **not** bumped by this story |
| `npm view @innocarpe/deepseek-build version` | **`5.2.1`** at close — lag behind `main` |
| `gh release list` Latest | **`v5.2.1`** at close — lag; **5.2.2 packaging is a separate release lane** (not this story) |
| Board text residual | Still may document Spec 45 cut as **`5.2.0`** (VC006) in places — **stale vs live main floor** |
| VC003 | **on main** via #130; Path A mint remains prerequisite |
| VC004 | **open stack** PR **#135** on `vc004-snippet-id-require` (require gate); VC005 opens stacked on that branch |
| Thin Path B | `crates/dsb-tools` `SnippetStore` remains **reference/oracle**, not Path A proof |

### 0.2 Floor interpretation (fail-close)

- **Close-time live floor is `origin/main` = `5.2.2` (`d71c1b3`).** Historical open-time probes recorded **`5.2.1`** and are preserved as history only.
- **`5.2.0`, `5.2.1`, and `5.2.2` are already used.** VC005 has **no SemVer bump** and **must not** reuse or cut any of them.
- npm **5.2.1** / GitHub Latest **v5.2.1** lag is **out of scope** here; **5.2.2 release packaging is a separate lane**. **Do not** duplicate packaging.
- Remaining Spec 45 completion after VC005 belongs to the **next free feature minor** → **`5.3.0`** under the current close-time floor (unless a later board/npm re-check shows another free `5.Y.0`).
- Feature PRs (VC003–VC005) stay **unversioned**. Only a dedicated cut unit (historical VC006 slot) bumps SemVer.
- Owner-bar **`file_version` (sha256)** remains a **compatibility alias** of snippet `version`; do not remove it.
- **Open as a stacked PR** with base **`vc004-snippet-id-require`** and body **`Depends on #135`**. **Do not** rebase onto `origin/main` before open. **Rebase / retarget after #135 merges.**

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
| VC006 / Spec 45 cut | heart + multi-edit R0A + SemVer cut of remaining Spec 45 | **not implemented**; cut at **next free minor** (live → **`5.3.0`**; not reused **`5.2.0`/`5.2.1`/`5.2.2`**) |
| release-5.2.2 packaging | npm/GitHub lag close (Latest still **5.2.1** / **v5.2.1** while main is **5.2.2**) | **not this story** (separate release lane) |

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
| English Conventional subjects | Bump `Cargo.toml` / package SemVer / claim **`5.2.0`**, **`5.2.1`**, or **`5.2.2`** cut |
| Host-only force flag | Free model boolean that skips version/edit safety |

### 3.4 Chaining / stacking

| Pattern | Choice for VC005 |
|---------|------------------|
| **Base (at open)** | **`vc004-snippet-id-require`** (stacked on open PR **#135**) — **not** `origin/main` |
| **Branch** | `vc005-snippet-invalidation` |
| **After #135 merges** | **Rebase / retarget** onto updated **`main`** (do **not** do this before open) |
| **Merge order** | #130 (VC003) → #135 (VC004) → VC005 → Spec 45 cut (next free minor) |
| **Conflict lock** | Path A `SessionSnippetStore` expiry + `search_replace` success/write path + bash post-dispatch invalidation owned by VC005; resume/fork reserved for VC006 |

**Planned PR title (when opened later):** `feat(tools): Path A write/bash snippet invalidation laws`
**Label kind:** `feat`
**Body:** Problem / What changed / Testing honesty / AI review / Security / Notes; **`Depends on #135`** (VC004); base **`vc004-snippet-id-require`**; SemVer **none**; does **not** reuse/cut **`5.2.0`/`5.2.1`/`5.2.2`**; next free feature minor under close-time floor is **`5.3.0`**.

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
- Does **not** cut or reuse **`5.2.0`**, **`5.2.1`**, **`5.2.2`**, **`5.3.0`**, or any other SemVer; does **not** bump product version; does **not** package npm/GitHub release (including the separate **5.2.2** packaging lane).
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
| 1 | `2a2f11c` | `docs(product): VC005 Path A snippet invalidation plan + evidence` | Plan only (this file first) |
| 2 | `464ba28` | `feat(tools): Path A snippet expire + write/bash invalidation laws` | Store expire + write/bash laws (force-guard skip completed with test lane) |
| 3 | `d1080a3` | `test(tools): VC005 write/bash invalidation fail-closed regressions` | Focused `vc005_*` unit tests (+ host force empty-old skip fix) |
| 4 | `ae67c64` | `style(tools): cargo fmt Path A snippet invalidation sources` | rustfmt only |
| 5 | `879e95f` | `fix(tools): harden Path A bash expire path match and redirects` | P1 fixes from independent review (path form + fd redirects) |
| 6 | tip (`git log -1`) | `docs(product): record VC005 Path A invalidation gate evidence` | Validation fill + adversarial READY |

**No VC006 behavior** in these commits: no resume/fork table restore, no public R0A harness, no SemVer bump, no release packaging.

### 7.2 What shipped (code)

| Piece | Location / behavior |
|-------|---------------------|
| Store expiry | `SessionSnippetStore::expire_path` / `expire_all` / `apply_bash_expire_plan` |
| Bash classifier | `bash_snippet_expire_plan` / `bash_command_may_mutate_files` / path extraction helpers (M2 heuristic; unknown → All) |
| Edit success | Path A `search_replace` after `EditsApplied` → `expire_path` |
| Write deny | Under `snippet_safe`, empty `old_string` + existing path (incl. empty file) → `path_exists_use_edit`; **no write**, **no expire** |
| Host force | `SearchReplaceParams.allow_force_write_overwrite` (host-only, default false) allows empty-old overwrite + expire; not a model input field |
| Bash hook | After backend accepts dispatch (FG run Ok / BG start Ok) apply plan; validation reject / backend Err → no expire |
| Dual CLI | Unchanged (`deepseek-build` / `dsb`) |
| Dependencies | **None** added |
| SemVer | **none** (no bump); close-time live floor **`5.2.2`** at `d71c1b3`; does not reuse **5.2.0/5.2.1/5.2.2**; next free feature minor **5.3.0** |

### 7.3 Acceptance matrix

| Check | Result | Evidence class |
|-------|--------|----------------|
| Evidence doc committed first | **PASS** — `2a2f11c` | commit |
| expire_path / expire_all APIs | **PASS** (`vc005_expire_path_*`, `vc005_expire_all_*`) | **unit** |
| Successful edit expires path snippets | **PASS** (`vc005_successful_edit_expires_path_snippets`) | **unit** |
| Other paths preserved | **PASS** (`vc005_expire_path_preserves_other_paths`) | **unit** |
| Create-new without id | **PASS** (`vc005_create_new_still_ok_without_snippet_id`) | **unit** |
| path_exists_use_edit deny + no partial write | **PASS** (`vc005_path_exists_use_edit_*`, `vc005_empty_existing_file_*`) | **unit** |
| Host force overwrite + expire | **PASS** (`vc005_host_force_overwrite_writes_and_expires`) | **unit** |
| Bash known / unknown / read-only | **PASS** (`vc005_bash_*` + store plan units) | **unit** |
| Pre-dispatch / backend fail no expire | **PASS** (`vc005_bash_pre_dispatch_*`, `vc005_bash_backend_error_*`) | **unit** |
| Session isolation | **PASS** (`vc005_session_isolation_*`) | **unit** |
| VC005 focused suite | **PASS** — **20** tests, 0 failed (post P1 fix) | **unit** |
| macOS private/tmp expire match | **PASS** (`vc005_expire_path_matches_macos_private_tmp_asymmetry`) | **unit** |
| fd/dev-null redirect not mutation | **PASS** (`vc005_bash_fd_redirect_is_not_file_mutation`) | **unit** |
| VC004 regressions | **PASS** — **9** tests | **unit** |
| VC003 mint regressions | **PASS** — **11** tests | **unit** |
| `snippet_safe` filter | **PASS** — **4** tests | **unit** |
| Broader `search_replace::tests` | **PASS** — **92** tests (incl. concise) | **unit** |
| Thin oracle | **PASS** — `dsb-tools` snippets **9** + path_a_edit **8** | thin oracle (**not** Path A proof) |
| Public Path A R0A wire | **not run / not claimed** | — |
| SemVer bump | **none** | no version files touched; live `origin/main` is **`5.2.2`** (do not cut) |

### 7.4 Commands actually run (exact)

```bash
# Floor (story open → close)
git fetch origin main
git show origin/main:Cargo.toml | rg 'version = "'
# open: 5.2.1 · close: 5.2.2
npm view @innocarpe/deepseek-build version
# close: 5.2.1 (lag behind main)
gh release list -R innocarpe/deepseek-build --limit 8
# Latest v5.2.1

# Focused + regressions
cd third_party/grok-build
CARGO_INCREMENTAL=0 RUSTC_WRAPPER= cargo test -p xai-grok-tools --lib vc005
# ok — 20 passed (after path-form + fd-redirect harden)
CARGO_INCREMENTAL=0 RUSTC_WRAPPER= cargo test -p xai-grok-tools --lib vc004
# ok — 9 passed
CARGO_INCREMENTAL=0 RUSTC_WRAPPER= cargo test -p xai-grok-tools --lib vc003
# ok — 11 passed
CARGO_INCREMENTAL=0 RUSTC_WRAPPER= cargo test -p xai-grok-tools --lib snippet_safe
# ok — 4 passed
CARGO_INCREMENTAL=0 RUSTC_WRAPPER= cargo test -p xai-grok-tools --lib search_replace::tests
# ok — 92 passed
cargo fmt --manifest-path Cargo.toml -p xai-grok-tools -- --check
# exit 0 (after style commit)

# Thin oracle (repo root)
cargo test -p dsb-tools snippets   # 9 passed
cargo test -p dsb-tools path_a_edit  # 8 passed

# Required gates
./scripts/test-owner-bar.sh
# exit 0 — ALL PASS (PASS=60 FAIL=0 NOT_RUN=0)
./scripts/check-path-a-linkage.sh
# exit 0 — PASS
./scripts/test-heart-regression.sh
# exit 0 — PASS
# SKIPs: live L3.1–L3.5 (no credentials); PATH_A_E2E (not requested)

# Restore TSV side-effects (not committed)
git checkout HEAD -- docs/product/evidence/OWNER_BAR_STATUS.tsv \
  docs/product/evidence/PATH_A_R0_G010_HEART_REGRESSION_last.tsv
```

**Honesty:** All Path A invalidation claims above are **unit tests inside `xai-grok-tools`**. No public `deepseek-build`/`dsb` agent wire harness (R0A) was run for this story. Gate TSV rewrites were **restored to HEAD** and **not** committed. Close-time live product floor on `origin/main` is **`5.2.2`** at **`d71c1b3`** (historical open-time probe **`5.2.1`** preserved above). VC005 has **no SemVer bump** and does **not** reuse **5.2.0/5.2.1/5.2.2**; remaining Spec 45 completion stays at the **next free feature minor (`5.3.0`)**. npm/GitHub Latest lag at **5.2.1**/**v5.2.1** and the **5.2.2 packaging lane** are separate. **Open stacked** against **`vc004-snippet-id-require`** with body **`Depends on #135`**; **do not** rebase onto `origin/main` before open — **rebase / retarget after #135 merges**.

### Required project gates (verified)

| Gate | Exit | Result | Honesty |
|------|------|--------|---------|
| `./scripts/test-owner-bar.sh` | **0** | **ALL PASS** — `PASS=60 FAIL=0 NOT_RUN=0` | Owner-bar green on HEAD |
| `./scripts/check-path-a-linkage.sh` | **0** | **PASS** | NOTE: third_party/grok-build has no dsb-* Cargo dep (expected until F1) |
| `./scripts/test-heart-regression.sh` | **0** | **PASS** | Live L3.1–L3.5 **SKIP** (no credentials); `PATH_A_E2E` **SKIP** |
| `cargo fmt … xai-grok-tools -- --check` | **0** | clean | after style commit |
| `git status` after restore | clean of TSV side-effects | — | worktree clean of gate artifacts |

### 7.5 Independent adversarial review (read-only Grok)

| Field | Value |
|-------|--------|
| Reviewer | Separate read-only Grok code-reviewer lane (not the implementer self-approve) |
| Scope | Branch `vc005-snippet-invalidation` / Path A write+bash invalidation + tests + evidence + Spec/ADR §6 |
| First-pass verdict | **NOT_READY** (0 P0, 2 P1) |
| First-pass P1 | (1) known-path expire miss after delete when mint `/private/tmp` vs extract `/tmp`; (2) `ls 2>&1` / `/dev/null` redirects → over-broad `expire_all` |
| Remediation | `879e95f` path normalize + fd/dev-null non-mutation + regressions |
| **Final verdict** | **READY** |
| **P0** | **none** |
| **P1** | **none remaining** (both fixed + unit-covered) |
| P2 residuals (non-blocking) | (1) force-overwrite success copy still says “created”; (2) triple FS read on empty-old path; (3) `path_exists_use_edit` is substring in `InvalidInput` not a dedicated enum; (4) residual bash classifier bluntness on unknown mutators (intentional M2 fail-close) |
| VC006/R0A/SemVer smuggling | **none** found |
| Honesty | Unit lane only; no R0A claim; close-time floor **5.2.2** (`d71c1b3`); no reuse of **5.2.0/5.2.1/5.2.2**; next free feature minor **5.3.0**; stacked open base **`vc004-snippet-id-require`** / **Depends on #135** |

P2 items are optional polish / later stories — **not** required to call VC005 implementation-ready under the unit evidence bar.

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
