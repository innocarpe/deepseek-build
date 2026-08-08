# VC004 — Path A `search_replace` requires session-local `snippet_id`

| Field | Value |
|-------|--------|
| **Story** | **VC004** — Path A public `search_replace` requires a valid session-local Spec 45 `snippet_id` |
| **Plan** | `vision-complete-5x` |
| **Date** | 2026-08-08 |
| **Status** | **IMPLEMENTATION** (runtime require on Path A edit path; unit evidence) |
| **SemVer** | **none** (no version bump in this story; does **not** cut any release minor) |
| **Depends on** | **PR #130 MERGED** / VC003 Path A mint (`snippet_id` + `SessionSnippetStore` on `Resources`) — rebased onto `origin/main` `@4696e28` |
| **Board** | [`VISION_COMPLETE_5X_GOALS.md`](../VISION_COMPLETE_5X_GOALS.md) · DAG [`WAVE_5x_VISION_PR_DAG.md`](../WAVE_5x_VISION_PR_DAG.md) (read live on `origin/main` for floor; see §0) |
| **Normative design** | [`docs/adr/0010-spec-45-snippet-store.md`](../../adr/0010-spec-45-snippet-store.md) §5 edit contract |
| **Semantics SSOT** | [`docs/specs/45-snippet-edit.md`](../../specs/45-snippet-edit.md) |
| **Binding** | [`HEART_3X_SPEC_BINDING.md`](../../architecture/HEART_3X_SPEC_BINDING.md) · [`HARNESS_PHILOSOPHY.md`](../../architecture/HARNESS_PHILOSOPHY.md) §4.1 |

**This file is the mandatory ultragoal PR unit plan for VC004 plus implementation evidence.**
It does **not** claim VISION L1 complete, owner-bar re-cut, Path A multi-edit R0A, write/bash invalidation (VC005), resume/fork (VC006), any SemVer cut, or public R0A wire proof.

---

## 0. Floor and dependency facts

### 0.1 Historical live floor (re-check; post-rebase onto `origin/main`)

| Probe | Live result |
|-------|-------------|
| This worktree branch | `vc004-snippet-id-require` (started from PR #130 tip; **rebased onto `origin/main`**) |
| Rebase base | **`origin/main` `@4696e28`** (includes **#130 MERGED** VC003 + later main commits; product floor **5.2.1**) |
| Working tree product `Cargo.toml` / `package.json` after rebase | **`5.2.1`** (inherited from main; **not** bumped by this story) |
| `git show origin/main:Cargo.toml` version | **`5.2.1`** |
| `package.json` on `origin/main` | **`5.2.1`** |
| npm latest / GitHub Latest release (honesty) | still lag at **`5.2.0` / `v5.2.0`** — **owned by separate release-5.2.1 lane**; this story does **not** package/deploy or cut |
| Board text residual | Still may document Spec 45 cut as **`5.2.0`** (VC006) in places — **stale vs live main floor** |
| VC003 | **on main** via #130; Path A mint remains prerequisite for this require gate |
| Thin Path B | `crates/dsb-tools` `SnippetStore` remains **reference/oracle**, not Path A proof |

### 0.1a Pre-merge refresh after #127 landed

The table above is historical evidence from the VC004 rebase at `origin/main`
`@4696e28`. Before merging PR #135, the branch was merge-forwarded with current
`origin/main` `@d1f942f`, which includes PR #127 and product floor **`5.2.2`**.

| Probe | Current result |
|-------|----------------|
| Merge-forward base | **`origin/main` `@d1f942f`** |
| Working tree product `Cargo.toml` / `package.json` after merge-forward | **`5.2.2`** inherited from main; **not** bumped by this story |
| npm latest / GitHub Latest release | **`5.2.2` / `v5.2.2`** |
| SemVer impact | **none** — this PR remains an unversioned feature unit |

### 0.2 Floor interpretation (fail-close)

- **`5.2.0`, `5.2.1`, and `5.2.2` are already used** on the product line (current main product version **`5.2.2`**). This story **must not** reuse or cut any of them.
- Remaining Spec 45 completion (VC004 → VC005 → cut unit) belongs to the **next free feature minor**. With main at **`5.2.2`**, that remains **`5.3.0`** unless a later board/npm re-check shows another free `5.Y.0`.
- Feature PRs (VC003–VC005) stay **unversioned**. Only a dedicated cut unit (historical VC006 slot, rebased) bumps SemVer.
- Owner-bar **`file_version` (sha256)** remains a **compatibility alias** of snippet `version`; do not remove it from wire/output.
- Safe open base after #130 merge: **`main`** (`origin/main` `@4696e28` at rebase time).

---

## 1. Why this PR (one sentence)

Require a **valid session-local `snippet_id`** on Path A `search_replace` (when product `snippet_safe` is on) so free-form / hash-only primary edit is fail-closed and edits are authorized only for the recorded path / range / version from a prior text `read_file`.

---

## 2. Call-path map (inspected before design)

| Layer | Path | Role today (pre-VC004) |
|-------|------|------------------------|
| Public product | `deepseek-build` / `dsb` → `deepseek-build-agent` (vendored Grok) | **Path A** product default |
| Product Standard toolset | `xai-grok-shell` `FileToolset::Standard::tool_configs` | Injects `snippet_safe: true` + `empty_old_string_does_not_override: true` on `search_replace` |
| Edit impl | `xai-grok-tools` `implementations/grok_build/search_replace` | `run_search_replace` → create vs `handle_replacement` |
| Edit args today | `SearchReplaceInput` | `file_path`, `old_string`, `new_string`, `replace_all`, optional **`file_version`** — **no `snippet_id`** |
| `snippet_safe` gate today | Before mutation | Non-create edits require matching full-file **`file_version`** sha256; missing → free-form reject; mismatch → `snippet_stale` |
| Match scope today | Whole file | Literal (optional unicode-normalized) match on **entire file**, not session snippet range |
| Session store (VC003) | `types/snippet_store.rs` `SessionSnippetStore` on `Resources` | Minted by text `read_file`; fields: id, path, start/end line, version, scope, preview, encoding, issued_at_turn |
| Thin oracle | `crates/dsb-tools` `path_a_edit` / `SnippetStore::edit` | Already requires `snippet_id` + scope replace — **not** Path A proof |
| Dual CLI | `deepseek-build` / `dsb` | Must both keep working; no install rename |

### Exact pre-VC004 product edit contract (honesty)

1. **Create path:** `old_string == ""` and path missing → write new file; **no** `file_version` / `snippet_id` required (write-create spirit; full write laws remain VC005).
2. **Existing file + `snippet_safe`:** requires **`file_version`** equal to `hex(sha256(current full file bytes))`.
3. **Missing `file_version`:** `InvalidInput` free-form primary reject; **no disk write**.
4. **Stale `file_version`:** `InvalidInput` with `snippet_stale:…`; **no disk write**.
5. **Match:** whole-file literal match; multi-match without `replace_all` fails closed; empty-old overwrite of non-empty file fail-closed when guard / `snippet_safe` on.
6. **Session `snippet_id`:** minted on read (VC003) but **ignored** by edit until this story.

### Target VC004 contract (Path A, `snippet_safe == true`)

| Case | Behavior |
|------|----------|
| Non-create edit **missing** `snippet_id` | Fail closed; **no write** |
| **Malformed** id (not ADR `snp_` + 26 Crockford ULID) | Fail closed; **no write** |
| **Unknown** id (not in this session store) | Fail closed; **no write** |
| **Path mismatch** (edit `file_path` ≠ recorded snippet path after resolve) | Fail closed; **no write** |
| **Stale / version mismatch** (recorded `version` ≠ current full-file sha256; optional wire `file_version` if present must also match current) | Fail closed; **no write** |
| **Valid** id | Authorizes **only** recorded path + inclusive line range + version; replace runs **inside that scope**; returns normal `EditsApplied` / existing error classes for no-match / multi-match |
| After successful mutation | Old ids become **stale** on next check via version drift; **re-read/mint** is the way to obtain a fresh id (eager path expire is **VC005**, not claimed here) |
| Create (`old_string` empty + new path) | Unchanged: no `snippet_id` required |
| `snippet_safe == false` (non-product legacy) | Unchanged free-form path (tests / non-Standard); product Standard keeps `snippet_safe: true` |

**Dual-accept decision (explicit):** Prefer **hard `snippet_id`** on product Path A once VC003 mints IDs (ADR 0010 §5.1). Do **not** keep `file_version`-only authorization as product primary. Keep `file_version` on the wire as optional compatibility alias / extra check; do not remove mint or schema field.

---

## 3. PR unit plan (four sections)

Per [`ULTRAGOAL_PR_PLANNING.md`](../ULTRAGOAL_PR_PLANNING.md). **VC004 is one feature PR** — edit require only; no write/bash expire; no SemVer bump; no R0A.

### 3.1 PR units (ordered)

#### PR unit 1 — `docs(product): VC004 Path A snippet_id require plan + evidence` **(this file)**
- **Intent:** Lock dependency on VC003 / #130, map live edit contract, atomic units, acceptance, non-claims, floor facts **before** source edits.
- **Touches:** `docs/product/evidence/VC004_PATH_A_SNIPPET_ID_REQUIRE_2026-08-08.md` only
- **Depends on:** VC003 mint (PR #130 head)
- **SemVer:** none

#### PR unit 2 — `feat(tools): require session snippet_id on Path A search_replace`
- **Intent:** Add optional `snippet_id` arg; when `snippet_safe`, non-create edits require a store-valid id authorizing path/range/version; fail closed otherwise; scope-limit match to recorded range; preserve create path + unrelated semantics + dual CLI.
- **Touches:** primarily `third_party/grok-build/.../search_replace/` (+ any `SearchReplaceInput` construction sites that need the new field); reuse `SessionSnippetStore` / `is_valid_snippet_id` from VC003
- **Depends on:** unit 1
- **SemVer:** none

#### PR unit 3 — `test(tools): VC004 snippet_id require regressions`
- **Intent:** Focused unit tests for acceptance checks below (valid, missing, malformed/unknown, stale/version, path mismatch, no partial write, session isolation as feasible).
- **Touches:** `search_replace` tests in `xai-grok-tools` (update existing `snippet_safe` + `file_version` cases to the new primary)
- **Depends on:** unit 2
- **SemVer:** none

#### Forward mapping (out of this PR)

| Unit | Story | Status here |
|------|-------|-------------|
| VC005 | write create-only / force overwrite + bash expire laws | **not implemented** |
| VC006 / Spec 45 cut | heart + multi-edit R0A + SemVer cut of remaining Spec 45 | **not implemented**; cut at **next free minor** (live → **`5.3.0`**, not reused **`5.2.0`**) |

### 3.2 Sequential vs parallel

#### Sequential (must order)

1. **VC003 / PR #130** → **VC004 unit 1 (docs)** — mint before require.
2. **unit 1** → **unit 2** (edit require + scope) → **unit 3** (tests).
3. **VC004** → **VC005** — invalidation laws after require exists.

#### Parallel (safe concurrent)

- None on the same Path A `search_replace` / `SearchReplaceInput` surface.
- Pure docs that do not redefine ADR 0010 edit semantics may proceed independently.

```text
VC003 (mint, #130) ──► VC004 (require, no SemVer) ──► VC005 ──► Spec 45 cut @ next free minor (live → 5.3.0)
```

### 3.3 Atomic commits (on `vc004-snippet-id-require`)

```text
docs(product): VC004 Path A snippet_id require plan + evidence
feat(tools): require session snippet_id on Path A search_replace
test(tools): VC004 snippet_id require and fail-closed regressions
```

| Do | Do not |
|----|--------|
| One concern per commit | Mix VC005 write/bash expire into this branch |
| Keep `file_version` mint/alias | Remove `file_version` or break dual CLI |
| Session-local Resources store only | Process-global / cross-session table |
| English Conventional subjects | Bump `Cargo.toml` / package SemVer / claim `5.2.0` cut |

### 3.4 Chaining / stacking

| Pattern | Choice for VC004 |
|---------|------------------|
| **Base** | **`main`** after **#130 MERGED** (rebased onto `origin/main` `@4696e28`) |
| **Branch** | `vc004-snippet-id-require` |
| **Merge order** | #130 (VC003) → VC004 → VC005 → Spec 45 cut (next free minor) |
| **Conflict lock** | Path A `search_replace` + `SearchReplaceInput` + session snippet lookup owned by VC004; write/bash expire reserved for VC005 |

**Planned PR title (when opened later):** `feat(tools): require snippet_id on Path A search_replace`
**Label kind:** `feat`
**Body:** Problem / What changed / Testing honesty / AI review / Security / Notes; **Depends on #130**; SemVer none; does **not** cut **`5.2.0`**.

---

## 4. Acceptance criteria (VC004 only)

| ID | Criterion | Pass condition |
|----|-----------|----------------|
| **VC004-A1** | Valid session `snippet_id` authorizes edit | Unit: mint via store (or read), edit with id → `EditsApplied`; file content matches expected scope change |
| **VC004-A2** | Missing `snippet_id` fails closed | Unit: `snippet_safe` + non-create + no id → error; **bytes unchanged** |
| **VC004-A3** | Malformed / unknown id fails closed | Unit: bad shape and/or absent from this session store → error; no write |
| **VC004-A4** | Stale / version mismatch fails closed | Unit: mutate file after mint (or wrong version) → error; no write |
| **VC004-A5** | Path mismatch fails closed | Unit: id for path A used with edit path B → error; no write on either |
| **VC004-A6** | Scope authorization | Unit: match only inside recorded `[start_line, end_line]`; occurrence outside range does not authorize a whole-file free edit |
| **VC004-A7** | No partial write | Unit: every fail-closed case leaves original file bytes intact |
| **VC004-A8** | Session isolation | Unit: id minted in Resources A is unknown in Resources B |
| **VC004-A9** | Create path preserved | Unit: empty `old_string` new file still works without `snippet_id` under product guards |
| **VC004-A10** | No SemVer / no VC005–VC006 behavior | Diff has no version bump; no bash expire / write-force laws / R0A claim |

### Explicit non-claims (fail-close)

- Does **not** implement write create-only residual hardening beyond existing empty-old guard (VC005).
- Does **not** implement bash / external mutation snippet expiry (VC005).
- Does **not** eagerly expire all path snippets on successful edit as a separate VC005 law claim (version drift + re-read is the VC004 story; product may still purge later).
- Does **not** prove Path A multi-edit R0A / heart regression under real `snippet_id` tables (VC006 / R0A).
- Does **not** persist/restore snippet tables across resume/fork (VC006).
- Does **not** cut **`5.2.0`** (already used on live `main`), **`5.3.0`**, or any other SemVer; does **not** bump product version.
- Does **not** re-plan board tracks on `main` (board residual still lists Spec 45 cut as 5.2.0 in places).
- Does **not** claim public `deepseek-build`/`dsb` wire harness R0A unless that harness is run and captured with honest labels.
- Thin `dsb-tools` greens are **oracle only**, not Path A proof.

---

## 5. Security / cache boundaries

| Concern | Rule |
|---------|------|
| Spec 10 stable prefix | Snippet table **must not** appear in stable-prefix bytes (session `Resources` only; already true for VC003 store) |
| Cross-session leakage | IDs bind to owning session `Resources` / `SharedResources`; no process-global map |
| Permission | Existing Spec 90 / path policy gates stay before mutation; this story does not weaken them |
| Symlink / path | Compare authorized snippet path to resolved edit path; fail closed on mismatch |
| Dual CLI | No change to `deepseek-build` / `dsb` packaging names |
| Dependencies | **No new crates**; reuse VC003 helpers (`SessionSnippetStore`, `is_valid_snippet_id`, sha256 already in tree) |

---

## 6. Validation commands

```bash
# Whitespace / conflict markers
git diff --check
git status --short
git diff --stat

# Path A unit (xai-grok-tools)
cd third_party/grok-build
cargo test -p xai-grok-tools --lib vc004
cargo test -p xai-grok-tools --lib snippet_safe
cargo test -p xai-grok-tools --lib vc003   # mint regressions must stay green

# Format
cargo fmt --manifest-path third_party/grok-build/Cargo.toml -p xai-grok-tools -- --check

# Thin oracle still green (reference; not Path A proof)
# from repo root:
cargo test -p dsb-tools snippets path_a_edit

# Required project gates
./scripts/test-owner-bar.sh
./scripts/check-path-a-linkage.sh
./scripts/test-heart-regression.sh
```

**R0A public wire:** not claimed in this story unless a later amendment appends wire artifacts with honest labels.

---

## 7. Implementation evidence (filled after code)

### 7.1 Atomic commits on `vc004-snippet-id-require`

Post-rebase onto `origin/main` `@4696e28` (SHAs rewritten; subjects preserved):

| Order | SHA (prefix) | Subject | Contents honesty |
|------:|--------------|---------|------------------|
| 1 | `bb165aa` | `docs(product): VC004 Path A snippet_id require plan + evidence` | Plan only (this file first) |
| 2 | `258d052` | `feat(tools): require session snippet_id on Path A search_replace` | Contract/impl + minimal compile-side field wiring + existing `snippet_safe` suite adapted to hard `snippet_id` |
| 3 | `cbd8849` | `test(tools): VC004 snippet_id require and fail-closed regressions` | Focused `vc004_*` unit tests only (+255 lines) |
| 4 | `88acb68` | `style(tools): cargo fmt Path A search_replace snippet_id sources` | rustfmt only |
| 5 | `7eafa3c` | `docs(product): record VC004 Path A require gate evidence` | Pre-rebase validation fill |
| 6 | `66db923` | `docs(product): record VC004 adversarial READY verdict` | Independent review READY / no P0/P1 |
| 7 | tip (`git log -1`) | `docs(product): record VC004 rebase onto main 5.2.1 floor` | Base/head/floor honesty + post-rebase revalidation |

**No VC005/VC006 behavior** in these commits: no write create-only residual laws, no bash/external expire, no eager path-wide snippet purge claim, no R0A harness, no SemVer bump, no resume/fork table restore.

### 7.2 What shipped (code)

| Piece | Location / behavior |
|-------|---------------------|
| Wire arg | `SearchReplaceInput.snippet_id: Option<String>` (serde default; model-visible schema description) |
| Product gate | When `SearchReplaceParams.snippet_safe` and non-create edit: **hard require** valid session `snippet_id` |
| Authorization | Lookup in session `SessionSnippetStore` on `Resources`; shape via `is_valid_snippet_id`; path match; full-file sha256 vs stored `version` |
| Compat alias | Optional `file_version` still accepted; if present must equal current sha256; **not** sufficient alone |
| Scope | Matches filtered to inclusive 1-based `[start_line, end_line]` recorded at mint |
| Fail-closed | Missing / malformed / unknown / path mismatch / stale / file_version mismatch → `InvalidInput`; **no disk write** |
| Create path | Empty `old_string` new-file create still does **not** require `snippet_id` |
| Dual CLI | Unchanged (`deepseek-build` / `dsb`) |
| Dependencies | **None** added |

### 7.3 Acceptance matrix

| Check | Result | Evidence class |
|-------|--------|----------------|
| Evidence doc committed first | **PASS** — `bb165aa` (post-rebase) | commit |
| `SearchReplaceInput.snippet_id` + `snippet_safe` require gate | **PASS** — `258d052` (post-rebase) | commit |
| Scope-limited match for authorized id | **PASS** — `258d052` + unit | commit + unit |
| Valid id edit succeeds | **PASS** (`vc004_valid_snippet_id_edits_within_scope`, `snippet_safe_accepts_valid_snippet_id`) | **unit** |
| Missing / malformed / unknown fail closed | **PASS** (`vc004_missing_*`, `vc004_malformed_*`, `vc004_unknown_*`) | **unit** |
| Stale / version mismatch fail closed | **PASS** (`snippet_safe_stale_*`, `vc004_file_version_mismatch_*`) | **unit** |
| Path mismatch fail closed | **PASS** (`vc004_path_mismatch_rejected`) | **unit** |
| No partial write | **PASS** (all fail-closed cases assert original bytes) | **unit** |
| Session isolation | **PASS** (`vc004_session_isolation_unknown_in_other_resources`) | **unit** |
| Create without id still ok | **PASS** (`vc004_create_new_file_without_snippet_id_allowed`) | **unit** |
| Scope outside-range is no-match | **PASS** (`vc004_occurrence_outside_scope_is_no_match`) | **unit** |
| Broader `search_replace::tests` suite | **PASS** — **85** tests, 0 failed | **unit** |
| VC003 mint regressions still green | **PASS** — **11** tests, 0 failed | **unit** |
| Thin oracle still green | **PASS** — `dsb-tools` `snippets` 9 + `path_a_edit` 8 | thin oracle (**not** Path A proof) |
| Public Path A R0A wire | **not run / not claimed** | — |
| SemVer bump | **none** | no VC004 version bump; current `origin/main` is **`5.2.2`** after PR #127 (do not cut) |

### 7.4 Commands actually run (exact)

```bash
# Rebase (clean, no conflicts)
git fetch origin main
git rebase origin/main
# base origin/main @4696e28; six VC004 commits replayed with new SHAs

# Atomic history after rebase
git log --oneline origin/main..HEAD
# bb165aa docs plan
# 258d052 feat require
# cbd8849 test vc004
# 88acb68 style fmt
# 7eafa3c gate evidence
# 66db923 adversarial READY
# (+ post-rebase evidence update commit)

# Focused + broader (post-rebase)
cd third_party/grok-build
CARGO_INCREMENTAL=0 RUSTC_WRAPPER= cargo test -p xai-grok-tools --lib vc004
# ok — 9 passed

CARGO_INCREMENTAL=0 RUSTC_WRAPPER= cargo test -p xai-grok-tools --lib snippet_safe
# ok — 3 passed

CARGO_INCREMENTAL=0 RUSTC_WRAPPER= cargo test -p xai-grok-tools --lib search_replace::tests
# ok — 85 passed

CARGO_INCREMENTAL=0 RUSTC_WRAPPER= cargo test -p xai-grok-tools --lib vc003
# ok — 11 passed

cargo fmt --manifest-path third_party/grok-build/Cargo.toml -p xai-grok-tools -- --check
# exit 0

# Thin oracle (repo root; not Path A proof)
cargo test -p dsb-tools snippets
# ok — 9 passed
cargo test -p dsb-tools path_a_edit
# ok — 8 passed

# Required project gates (post-rebase HEAD; TSV side-effects restored to HEAD after)
./scripts/test-owner-bar.sh
# exit 0 — ALL PASS (PASS=60 FAIL=0 NOT_RUN=0)

./scripts/check-path-a-linkage.sh
# exit 0 — PASS

./scripts/test-heart-regression.sh
# exit 0 — PASS
# SKIPs (documented, not green-washed): live L3.1–L3.5 (no credentials);
# PATH_A_E2E (not requested; needs --with-e2e)

# Live floor re-check
git show origin/main:Cargo.toml | rg version
# 5.2.1
# npm latest / gh Latest still 5.2.0 lag — separate release lane

# Pre-merge refresh after PR #127 landed (no build/test run)
git merge --no-ff origin/main
# merge base origin/main @d1f942f; current product floor 5.2.2 inherited from main
# npm latest / gh Latest: 5.2.2 / v5.2.2
```

**Honesty:** All Path A require claims above are **unit tests inside `xai-grok-tools`**. No public `deepseek-build`/`dsb` agent wire harness (R0A) was run for this story. Gate TSV rewrites (`OWNER_BAR_STATUS.tsv`, heart regression last TSV) were **restored to HEAD** and **not** committed. Live product floor on `origin/main` is **`5.2.2`** after PR #127 — VC004 does not ship any SemVer cut or release packaging; remaining Spec 45 completion stays at the **next free feature minor (`5.3.0` under current live floor)**.

### Required project gates (verified post-rebase)

| Gate | Exit | Result | Honesty |
|------|------|--------|---------|
| `./scripts/test-owner-bar.sh` | **0** (re-run after rebase) | **ALL PASS** — `PASS=60 FAIL=0 NOT_RUN=0` | Owner-bar green on post-rebase HEAD; linkage + forbidden-evidence green |
| `./scripts/check-path-a-linkage.sh` | **0** | **PASS** | NOTE: third_party/grok-build has no dsb-* Cargo dep (expected until F1) |
| `./scripts/test-heart-regression.sh` | **0** | **PASS** | Live L3.1–L3.5 **SKIP** (no credentials); `PATH_A_E2E` **SKIP** |
| `cargo fmt … xai-grok-tools -- --check` | **0** | clean | VC004 vendor crate only |
| `git diff --check` (branch range) | **0** | no trailing whitespace / conflict markers | re-verified after rebase evidence update |

### 7.5 Independent adversarial review (read-only Grok)

| Field | Value |
|-------|--------|
| Reviewer | Separate read-only Grok code-reviewer lane (not the implementer self-approve) |
| Scope | Branch `vc004-snippet-id-require` / Path A `search_replace` require gate + tests + evidence + Spec/ADR §5 |
| **Verdict** | **READY** |
| **P0** | **none** |
| **P1** | **none** |
| P2 residuals (non-blocking) | (1) authorize `Err(_)` fallthrough should prefer NotFound-only; (2) TOCTOU double-read before write; (3) add dual-accept lock unit (`file_version` alone still rejected); (4) normalized-fallback scope uses `old_string.len()`; (5) path compare via host canonicalize vs tool FS; (6) stale “require file_version” comments in Standard toolset / FileContent docs |
| VC005/VC006 smuggling | **none** found |
| Honesty | Unit lane only; no R0A claim; floor **5.2.1** used; next free feature minor **5.3.0** |

P2 items are optional polish / later stories — **not** required to call VC004 implementation-ready under the unit evidence bar.

---

## 8. References

- ADR: [0010-spec-45-snippet-store](../../adr/0010-spec-45-snippet-store.md)
- Spec: [45-snippet-edit](../../specs/45-snippet-edit.md)
- VC003 evidence: [VC003_PATH_A_SNIPPET_ID_2026-08-08.md](./VC003_PATH_A_SNIPPET_ID_2026-08-08.md)
- VC002 ADR evidence: [VC002_SPEC45_ADR_2026-08-07.md](./VC002_SPEC45_ADR_2026-08-07.md)
- Prior spirit: [H45_PATH_A_SNIPPET_2026-08-07.md](./H45_PATH_A_SNIPPET_2026-08-07.md) · [G004_SNIPPET_LIVE_2026-08-07.md](./G004_SNIPPET_LIVE_2026-08-07.md)
- Path A code: `third_party/grok-build/.../implementations/grok_build/search_replace/`
- Session store: `third_party/grok-build/.../types/snippet_store.rs`
- Planning: [ULTRAGOAL_PR_PLANNING](../ULTRAGOAL_PR_PLANNING.md) · [WAVE_5x_VISION_PR_DAG](../WAVE_5x_VISION_PR_DAG.md)
