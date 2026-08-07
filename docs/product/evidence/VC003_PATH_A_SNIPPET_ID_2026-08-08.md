# VC003 — Path A `read_file` mints session-local `snippet_id`

| Field | Value |
|-------|--------|
| **Story** | **VC003** — Path A public `read_file` issues Spec 45 `snippet_id` |
| **Plan** | `vision-complete-5x` |
| **Date** | 2026-08-08 |
| **Status** | **IMPLEMENTATION** (runtime mint on Path A tool path; unit/integration evidence) |
| **SemVer** | **none** (no version bump in this story; product remains **`5.1.0`** on disk) |
| **Depends on** | **PR #125** (VC002 — ADR 0010 Spec 45 session SnippetStore design) · branch base `spec/vc002-snippet-store` |
| **Board** | [`VISION_COMPLETE_5X_GOALS.md`](../VISION_COMPLETE_5X_GOALS.md) · DAG [`WAVE_5x_VISION_PR_DAG.md`](../WAVE_5x_VISION_PR_DAG.md) |
| **Normative design** | [`docs/adr/0010-spec-45-snippet-store.md`](../../adr/0010-spec-45-snippet-store.md) §4 issuance |
| **Semantics SSOT** | [`docs/specs/45-snippet-edit.md`](../../specs/45-snippet-edit.md) |
| **Binding** | [`HEART_3X_SPEC_BINDING.md`](../../architecture/HEART_3X_SPEC_BINDING.md) · [`HARNESS_PHILOSOPHY.md`](../../architecture/HARNESS_PHILOSOPHY.md) §4.1 |

**This file is the mandatory ultragoal PR unit plan for VC003 plus implementation evidence.**  
It does **not** claim VISION L1 complete, owner-bar re-cut, Path A multi-edit R0A with required `snippet_id`, write/bash invalidation, resume/fork, or any **`5.2.0`** ship.

---

## 0. Floor and dependency facts (2026-08-08)

| Probe | Result |
|-------|--------|
| Product version on disk (`Cargo.toml`) | **`5.1.0`** |
| VC002 design | ADR 0010 accepted; evidence `VC002_SPEC45_ADR_2026-08-07.md` |
| VC002 GitHub PR | **#125** `spec(45): Path A snippet_id ADR + store design` (`spec/vc002-snippet-store`) — **do not alter/merge that PR in this story** |
| This branch | `feat/vc003-path-a-snippet-id` forked from VC002 stack HEAD (stacked on #125) |
| Thin Path B | `crates/dsb-tools` `SnippetStore` remains **reference/oracle**, not Path A proof |

### Floor interpretation

- **VC002 is design-complete for implementers; VC003 is the first Path A runtime unit.**
- Next feature minor for full Spec 45 Path A remains **`5.2.0`** (VC003–VC006 together; SemVer only in VC006).
- Owner-bar **`file_version` (sha256)** mint stays as a **compatibility alias** of snippet `version`; do not remove or reinterpret it.

---

## 1. Why this PR (one sentence)

Issue a real session-local opaque **`snippet_id`** on successful Path A text `read_file` (with model-visible metadata) so VC004 can require it on edit without inventing a cross-session store.

---

## 2. PR unit plan (four sections)

Per [`ULTRAGOAL_PR_PLANNING.md`](../ULTRAGOAL_PR_PLANNING.md). **VC003 is one feature PR** — mint only; no edit/write/bash laws; no SemVer bump.

### 2.1 PR units (ordered)

#### PR unit 1 — `docs(product): VC003 Path A snippet_id mint plan + evidence` **(this file)**
- **Intent:** Lock dependency on PR #125, atomic unit list, validation, non-claims, release-floor facts **before** source edits.
- **Touches:** `docs/product/evidence/VC003_PATH_A_SNIPPET_ID_2026-08-08.md` only
- **Depends on:** PR #125 design (ADR 0010)
- **SemVer:** none

#### PR unit 2 — `feat(tools): session-local snippet table + FileContent.snippet_id`
- **Intent:** Ephemeral session store on Path A `Resources` (not Spec 10 prefix; not process-global); extend tool output with `snippet_id` + scope metadata; keep `file_version` sha256 alias.
- **Touches:** `third_party/grok-build/crates/codegen/xai-grok-tools` (types + store)
- **Depends on:** unit 1
- **SemVer:** none

#### PR unit 3 — `feat(tools): mint snippet_id on Path A text read_file`
- **Intent:** Successful text `read_file` inserts a store record and returns `snippet_id`; repeated reads mint distinct IDs; binary/image/PDF/PPTX/error paths unchanged (no id).
- **Touches:** Path A `read_file` implementation + model-visible `to_prompt_format`
- **Depends on:** unit 2
- **SemVer:** none

#### PR unit 4 — `test(tools): VC003 mint / multi-id / session-local / file_version regression`
- **Intent:** Focused regression tests for the five acceptance checks below.
- **Touches:** tests under `xai-grok-tools` (and thin `dsb-tools` only if still green as oracle)
- **Depends on:** unit 3
- **SemVer:** none

#### Forward mapping (out of this PR)

| Unit | Story | Status here |
|------|-------|-------------|
| VC004 | require `snippet_id` on Path A `search_replace` | **not implemented** |
| VC005 | write/bash invalidation | **not implemented** |
| VC006 | heart + multi-edit R0A + **`5.2.0`** | **not implemented** |

### 2.2 Sequential vs parallel

#### Sequential (must order)

1. **PR #125 (VC002)** → **VC003 unit 1 (docs)** — design before code.
2. **unit 1** → **unit 2** (store + types) → **unit 3** (mint wire) → **unit 4** (tests).
3. **VC003** → **VC004** — edit cannot require IDs that are not issued.

#### Parallel (safe concurrent)

- None on the same Path A `read_file` / `FileContent` surface.
- Pure docs elsewhere that do not redefine ADR 0010 semantics may proceed independently.

```text
PR #125 (VC002 ADR) ──► VC003 (mint) ──► VC004 ──► VC005 ──► VC006 (5.2.0)
```

### 2.3 Atomic commits (on `feat/vc003-path-a-snippet-id`)

```text
docs(product): VC003 Path A snippet_id mint plan + evidence
feat(tools): session snippet store + FileContent.snippet_id fields
feat(tools): mint snippet_id on Path A text read_file
test(tools): VC003 snippet_id mint and session-local regressions
```

| Do | Do not |
|----|--------|
| One concern per commit | Mix VC004 edit require into this branch |
| Keep `file_version` sha256 behavior | Remove or reinterpret `file_version` |
| Session-local Resources store only | Process-global / cross-session table |
| English Conventional subjects | Bump `Cargo.toml` / package SemVer |

### 2.4 Chaining / stacking

| Pattern | Choice for VC003 |
|---------|------------------|
| **Base** | Stacked on **PR #125** / `spec/vc002-snippet-store` (or `main` only after #125 merges) |
| **Branch** | `feat/vc003-path-a-snippet-id` |
| **Merge order** | Bottom-up: #125 → VC003 → VC004 → … |
| **Conflict lock** | Path A `read_file` + `FileContent` + session snippet store owned by VC003; edit path reserved for VC004 |

**Planned PR title (when leader opens later):** `feat(tools): mint snippet_id on Path A read_file`  
**Label kind:** `feat`  
**Body:** Problem / What changed / Testing honesty / AI review / Security / Notes; `Depends on #125`.

---

## 3. Call-path map (before design)

| Layer | Path | Role |
|-------|------|------|
| Public product | `deepseek-build` / `dsb` → `deepseek-build-agent` (vendored Grok) | **Path A** product default |
| Tool impl | `xai-grok-tools` `implementations/grok_build/read_file` | Issues content + `file_version` today |
| Output type | `types/output.rs` `FileContent` | Carries `file_version`; model-visible via `to_prompt_format` |
| Session DI | `Resources` / `SharedResources` per agent session | Ownership boundary for session-local table |
| Thin oracle | `crates/dsb-tools` `SnippetStore` | Path B algorithm reference — **not** Path A proof |

**Ownership decision (smallest safe):**

1. Host an ephemeral **session snippet table** on Path A `Resources` (`get_or_default`, **not** registered into Spec 10 stable-prefix persistence).
2. On successful **text** `read_file`, mint `snp_<opaque>` (ULID-class), store record (path, inclusive line range, version=sha256 full file, scope, preview, encoding, issuance counter), attach `snippet_id` (+ range/scope metadata) to `FileContent`.
3. Keep existing **`file_version` = hex(sha256(full file bytes))** as compatibility alias of record `version`.
4. Binary / image / PDF / PPTX / error / too-large paths: **no** `snippet_id` (unchanged spirit).
5. Do **not** implement edit require, write/bash expire, or resume/fork table restore.

---

## 4. Acceptance criteria (VC003 only)

| ID | Criterion | Pass condition |
|----|-----------|----------------|
| **VC003-A1** | Successful text `read_file` emits `snippet_id` | Unit: `FileContent.snippet_id` is `Some` and starts with `snp_` |
| **VC003-A2** | Repeated reads mint distinct IDs | Unit: two reads → two different ids; both present in session store |
| **VC003-A3** | IDs are session-local | Unit: separate `Resources` instances do not share tables (no process-global claim) |
| **VC003-A4** | `file_version` remains full SHA-256 hex | Unit: equals `sha256(file bytes)` and still model-visible |
| **VC003-A5** | Non-text / error paths unchanged where practical | Unit: binary/not-found still no `snippet_id` / structured errors |
| **VC003-A6** | Model-visible serialization explicit | `to_prompt_format` includes `snippet_id` + `file_version` for text success |
| **VC003-A7** | No SemVer / no VC004–VC006 behavior | Diff has no version bump; no edit-require flip |

### Explicit non-claims (fail-close)

- Does **not** prove Path A multi-edit R0A / heart regression under real `snippet_id` tables.
- Does **not** require `snippet_id` on `search_replace` (VC004).
- Does **not** implement write create-only / bash expire laws (VC005).
- Does **not** persist/restore snippet tables across resume/fork (VC006).
- Does **not** cut **`5.2.0`** or bump SemVer.
- Does **not** claim public `deepseek-build`/`dsb` wire harness R0A unless that harness is actually run and captured in this evidence file (unit/integration tests alone are **labeled** as unit/integration, not R0A).

---

## 5. Validation commands

```bash
# Whitespace / conflict markers on branch diff
git diff --check
git status --short
git diff --stat

# Unit: Path A mint + regressions (crate)
cd third_party/grok-build
cargo test -p xai-grok-tools current_read_file_mints_file_version_sha256
cargo test -p xai-grok-tools vc003

# Thin oracle still green (reference; not Path A proof)
cargo test -p dsb-tools snippets

# Link / presence
test -f docs/product/evidence/VC003_PATH_A_SNIPPET_ID_2026-08-08.md
test -f docs/adr/0010-spec-45-snippet-store.md

# Optional linkage smoke (mint residual only if any)
./scripts/check-path-a-linkage.sh || true
```

**R0A public wire:** not claimed in this story unless a later amendment appends wire artifacts under this filename or a companion evidence file with honest labels.

---

## 6. Implementation evidence (filled after code)

| Check | Result | Evidence class |
|-------|--------|----------------|
| Branch created from VC002 HEAD | `feat/vc003-path-a-snippet-id` (from `spec/vc002-snippet-store` @ `9da03a1`) | git |
| Evidence doc committed first | **yes** — `56a249a` | commit |
| Store + FileContent fields | **yes** — `73b0bc8` | commit |
| Path A text mint | **yes** — `89564c8` | commit |
| Text UTF-8 read mints `snippet_id` | **PASS** (`vc003_current_read_file_mints_snippet_id`) | **unit** |
| ID shape ADR 0010 §2 exact | **PASS** — `snp_` + 26 Crockford-base32 ULID; alphabet `0123456789ABCDEFGHJKMNPQRSTVWXYZ`; UUID-v7-simple rejected (`uuid_v7_simple_is_not_valid_snippet_id_shape`) | **unit** |
| Repeated reads differ | **PASS** (`vc003_repeated_reads_mint_distinct_snippet_ids`) | **unit** |
| Session-local store | **PASS** (`vc003_snippet_store_is_session_local_not_process_global` + store unit; no static/global) | **unit** |
| `file_version` sha256 preserved | **PASS** (`current_read_file_mints_file_version_sha256` + VC003 mint test) | **unit** |
| Non-text / invalid UTF-8 / error no-id | **PASS** (binary, not-found, `vc003_invalid_utf8_does_not_mint_snippet_id`) | **unit** |
| Thin Path B oracle still green | **PASS** (`cargo test -p dsb-tools snippets` — 9 ok) | thin oracle (**not** Path A proof) |
| Public Path A R0A wire | **not run / not claimed** | — |
| SemVer bump | **none** | cargo version still `5.1.0` |

### Design that shipped (code)

| Piece | Location |
|-------|----------|
| Session store | `third_party/grok-build/.../types/snippet_store.rs` — ephemeral `SessionSnippetStore` via `SharedResources` → `Resources::get_or_default` (**not** static, **not** process-global, **not** Spec 10 persistence) |
| Output fields | `FileContent.{snippet_id,snippet_start_line,snippet_end_line,snippet_scope}` + existing `file_version` |
| Mint path | Successful **UTF-8** text only (incl. empty UTF-8 file); invalid UTF-8 may lossy-display but **no** id; PDF/PPTX/image/binary/error unchanged (no id) |
| Model-visible | `to_prompt_format` appends `snippet_id`, optional range/scope, and `file_version` |
| ID shape | **ADR 0010 §2 exact:** `snp_` + Crockford-base32 ULID (26 chars). Local encoder (no new crate); 48-bit ms timestamp + 80-bit entropy from existing `uuid` randomness |

### Commands actually run

```bash
# Branch
git checkout -b feat/vc003-path-a-snippet-id   # from clean spec/vc002-snippet-store

# Path A unit (xai-grok-tools)
cd third_party/grok-build
cargo test -p xai-grok-tools --lib vc003
# ok — 6 passed (mint+ULID shape, multi-id, session-local, not-found, binary, invalid-utf8)

cargo test -p xai-grok-tools --lib current_read_file_mints_file_version_sha256
# ok — 1 passed

cargo test -p xai-grok-tools --lib 'snippet_store::'
# ok — 9 passed (Crockford 26-char alphabet exact + UUID-v7-simple rejected)

cargo test -p xai-grok-tools --lib read_empty_file_prompt
# ok — 1 passed

# Thin oracle (not Path A proof)
cargo test -p dsb-tools snippets
# ok — 9 passed

# Docs / whitespace
git diff --check
test -f docs/product/evidence/VC003_PATH_A_SNIPPET_ID_2026-08-08.md
test -f docs/adr/0010-spec-45-snippet-store.md
rg -n 'version' Cargo.toml | head -1   # 5.1.0
```

**Honesty:** All Path A mint claims above are **unit/integration tests inside `xai-grok-tools`**. No public `deepseek-build`/`dsb` agent wire harness (R0A) was run for this story.

---

## 7. References

- ADR: [0010-spec-45-snippet-store](../../adr/0010-spec-45-snippet-store.md)
- Prior Path A mint: [G003_MINT_FILE_VERSION_2026-08-07.md](./G003_MINT_FILE_VERSION_2026-08-07.md)
- VC002 plan: [VC002_SPEC45_ADR_2026-08-07.md](./VC002_SPEC45_ADR_2026-08-07.md)
- Thin reference: `crates/dsb-tools/src/snippets.rs`
- Path A code: `third_party/grok-build/.../implementations/grok_build/read_file/`
- Planning: [ULTRAGOAL_PR_PLANNING](../ULTRAGOAL_PR_PLANNING.md) · [WAVE_5x_VISION_PR_DAG](../WAVE_5x_VISION_PR_DAG.md)
