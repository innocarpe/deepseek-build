# VC006 — Path A multi-edit R0A + Spec 45 Deep Code cut evidence

| Field | Value |
|-------|--------|
| **Story** | **VC006** — public Path A (`deepseek-build` / `dsb` → product agent) multi-edit + stale-id / invalidation R0A proof, heart/owner/path-linkage honesty, then dedicated Spec 45 Deep Code release cut |
| **Plan** | `vision-complete-5x` |
| **Date** | 2026-08-08 |
| **Status** | **READY** — Path A R0A multi-edit + stale-id green; gates green; on-branch cut **`5.3.0`** landed; PR not merged |
| **SemVer** | on-branch **`5.3.0`** (unit 4) under live floor — do **not** reuse **5.2.0 / 5.2.1 / 5.2.2**; npm/GitHub publish is a separate lane |
| **Depends on** | **VC005** Path A write/bash invalidation (open PR **#137** `vc005-snippet-invalidation`) which stacks on **VC004** (#135) and **VC003** (#130 MERGED) |
| **Board** | [`VISION_COMPLETE_5X_GOALS.md`](../VISION_COMPLETE_5X_GOALS.md) · DAG [`WAVE_5x_VISION_PR_DAG.md`](../WAVE_5x_VISION_PR_DAG.md) (board still may name Spec 45 cut as `5.2.0` — **stale vs live floor**; this story does not re-plan tracks) |
| **Normative design** | [`docs/adr/0010-spec-45-snippet-store.md`](../../adr/0010-spec-45-snippet-store.md) §4–§6 |
| **Semantics SSOT** | [`docs/specs/45-snippet-edit.md`](../../specs/45-snippet-edit.md) |
| **Binding** | [`HEART_3X_SPEC_BINDING.md`](../../architecture/HEART_3X_SPEC_BINDING.md) · [`HARNESS_PHILOSOPHY.md`](../../architecture/HARNESS_PHILOSOPHY.md) §4.1 |

**This file is the mandatory ultragoal PR unit plan for VC006 plus implementation evidence.**
It does **not** claim VISION L1 complete until Path A R0A multi-edit + stale-id/invalidation artifacts are captured with honest labels, required gates stay green, and (only then) a dedicated **`5.3.0`** cut lands. Thin `dsb-tools` greens are **not** R0A proof.

---

## 0. Floor and dependency facts

### 0.1 Live floor (story open; 2026-08-08)

| Probe | Live result |
|-------|-------------|
| This worktree branch | `vc006-heart-r0a` (forked at VC005 tip `1ce7dcc`) |
| Stack base for feature commits / PR base | **`vc005-snippet-invalidation`** (open PR **#137**); **not** `origin/main` until after #137 merges |
| `git show origin/main:Cargo.toml` version | **`5.2.2`** (`origin/main` @ `d71c1b3`) |
| `package.json` on `origin/main` | **`5.2.2`** |
| Working tree product version (stack tip) | **`5.2.1`** on VC004/VC005 stack (pre-main-5.2.2 packaging; **not** bumped by units 1–3) |
| `npm view @innocarpe/deepseek-build version` | **`5.2.1`** — lag behind `main` |
| `gh release list` Latest | **`v5.2.1`** — lag; **5.2.2 packaging is a separate release lane** (not this story) |
| Board text residual | Still may document Spec 45 cut as **`5.2.0`** (VC006) — **stale vs live main floor** |
| VC003 | **on main** via #130; Path A mint prerequisite |
| VC004 | open stack PR **#135** on `vc004-snippet-id-require` |
| VC005 | open stack PR **#137** on `vc005-snippet-invalidation` (base #135) |
| Thin Path B | `crates/dsb-tools` `SnippetStore` remains **reference/oracle**, not Path A proof |

### 0.2 Floor interpretation (fail-close)

- **Live product floor is `origin/main` = `5.2.2`.**
- **`5.2.0`, `5.2.1`, and `5.2.2` are already used.** This story must **never** reuse or re-cut them.
- npm **5.2.1** / GitHub Latest **v5.2.1** lag and the separate **5.2.2 packaging lane** are **out of scope**.
- Spec 45 Deep Code completion cut for this train is the **next free feature minor → `5.3.0`** under the current floor (unless a later floor re-check shows another free `5.Y.0`).
- Units 1–3 stay **unversioned**. **Only unit 4** (dedicated release cut) may bump SemVer — and **only after** Path A R0A multi-edit + stale-id/invalidation proof is green with honest labels.
- If public runtime / linkage / credentials / agent binary cannot provide R0A → **record exact residual and fail closed**. Do **not** invent wire green or claim release readiness.
- Owner-bar **`file_version` (sha256)** remains a **compatibility alias** of snippet `version`; do not remove it.
- **Open as a stacked PR** with base **`vc005-snippet-invalidation`** and body **`Depends on #137`**. **Do not** rebase onto `origin/main` before open. **Rebase / retarget after #137 merges.**

---

## 1. Why this PR (one sentence)

Close remaining Spec 45 Deep Code Path A criteria with a **real public `deepseek-build` / `dsb` agent R0A** multi-edit sequence using session-local **`snippet_id`** (not thin-oracle only, not `file_version`-only primary), prove **stale-id / invalidation** after edit or bash mutation, keep **heart / owner-bar / path-linkage** gates green, and only then cut **`5.3.0`**.

---

## 2. Call-path map (inspected before design)

| Layer | Path | Role today (pre-VC006) |
|-------|------|------------------------|
| Public product | `deepseek-build` / `dsb` → `deepseek-build-agent` (vendored Grok) | **Path A** product default |
| Product Standard toolset | `xai-grok-shell` `FileToolset::Standard::tool_configs` | `snippet_safe: true` + empty-old guard |
| Mint (VC003) | Path A `read_file` → `SessionSnippetStore` + `FileContent.snippet_id` | Model-visible `snippet_id:` + `file_version:` in tool results |
| Edit require (VC004) | Path A `search_replace` | Hard `snippet_id` require when `snippet_safe`; scope-limited replace |
| Invalidation (VC005) | `expire_path` / `expire_all` + write/bash laws | Eager expire after success; `path_exists_use_edit`; bash known/unknown |
| Historical R0A liveness (G004) | `scripted_deepseek_server.py` `liveness-3edits` | **`file_version`-only** 3 edits — **pre-VC004**; **not** Spec 45 `snippet_id` multi-edit proof under current require gate |
| Historical R0A bash (G005) | `bash-stale` scenario | Stale **`file_version`** after bash — spirit useful; must re-prove with **`snippet_id`** primary |
| Public entry harness | `scripts/test-path-a-public-entry-e2e.sh` | text-pong public entry; does not yet drive multi-edit `snippet_id` R0A |
| Thin oracle | `crates/dsb-tools` | Unit greens only — **not** R0A |
| Dual CLI | `deepseek-build` / `dsb` | Must both keep working |

### Target VC006 R0A contract (Path A public agent)

#### 2.1 Multi-edit (V1-45 heart / Spec 45 multi-edit)

Public CLI → agent_launch → product agent (no `DEEPSEEK_BUILD_AGENT_BIN` override for the public-entry claim) with hermetic home + scripted DeepSeek wire:

1. Seed workspace files `a.txt` / `b.txt` (≥2 files).
2. Scripted model issues **`read_file`** before each authorized edit so Path A mints real **`snippet_id`**.
3. Scripted model applies **≥3 successful `search_replace`** edits using those **`snippet_id`** values (not free-form, not `file_version`-only primary).
4. Final disk: multi-file content matches expected sequence (e.g. a: `hello`→`hello1`→`hello2`, b: `world`→`world1`).
5. Wire JSONL + meta captured under `docs/product/evidence/` with honest **R0A** labels.
6. Agent exit 0 (or documented soft exit with wire+disk proof still pass).

#### 2.2 Stale-id / invalidation (V1-45-3/4 spirit)

At least one public Path A R0A scenario proves **fail-closed reuse** of an invalidated identity:

| Variant | Sequence | Pass |
|---------|----------|------|
| **Edit-expire** (preferred) | `read` → valid edit with `snippet_id` → reuse **same** id on same path without re-read | Tool error class **`snippet_not_found`** (or equivalent expired/unknown id); **disk unchanged** on the failing attempt |
| **Bash-expire** (optional second) | `read` → mint id → mutating bash → `search_replace` with old id | Fail closed (`snippet_not_found` and/or `snippet_stale`); disk not advanced by the bad edit |

**Honesty:** Re-using historical G005 `file_version`-only bash-stale wire is **not** enough for VC006. New wire must show **`snippet_id`** in the failing or succeeding edit args as product primary.

#### 2.3 Gates (must stay green)

| Gate | Command |
|------|---------|
| Owner-bar | `./scripts/test-owner-bar.sh` |
| Path A linkage | `./scripts/check-path-a-linkage.sh` |
| Heart regression | `./scripts/test-heart-regression.sh` |

Restore generated TSV side-effects to HEAD; **do not commit** them.

#### 2.4 Release cut (unit 4 only; conditional)

Only after §2.1–2.3 PASS:

| Item | Value |
|------|--------|
| SemVer | **`5.3.0`** full form (never `5.3`) |
| Touch | `Cargo.toml` / `Cargo.lock` / `package.json` / `CHANGELOG.md` (+ versions README if prior cuts do) |
| Non-cut | Do **not** package npm or publish GitHub Release in this story unless a separate release lane is explicitly run and recorded |
| Non-reuse | Never claim packaging of **5.2.0–5.2.2** |

---

## 3. PR unit plan (four sections)

Per [`ULTRAGOAL_PR_PLANNING.md`](../ULTRAGOAL_PR_PLANNING.md).

### 3.1 PR units (ordered)

#### PR unit 1 — `docs(product): VC006 Path A heart R0A plan + evidence` **(this file)**
- **Intent:** Lock stack base #137, live SemVer floor, acceptance matrix, security/cache boundaries, non-claims **before** source edits.
- **Touches:** `docs/product/evidence/VC006_PATH_A_HEART_R0A_2026-08-08.md` only
- **Depends on:** VC005 tip / #137
- **SemVer:** none

#### PR unit 2 — `test(scripts): Path A R0A multi-edit + stale-id snippet_id harness`
- **Intent:** Extend hermetic scripted DeepSeek server + public-entry driver so scenarios **read → mint `snippet_id` → multi-edit** and **stale-id after expire** run through public `deepseek-build` / agent resolution; capture wire/meta artifacts; fail closed on missing ids.
- **Touches:** primarily `scripts/lib/scripted_deepseek_server.py`, new or extended `scripts/test-path-a-*.sh` (VC006 R0A driver), evidence wire/meta paths under `docs/product/evidence/`
- **Depends on:** unit 1; runtime laws from VC003–VC005 on stack
- **SemVer:** none

#### PR unit 3 — `docs(product): record VC006 Path A R0A + gate evidence`
- **Intent:** Fill acceptance matrix with exact commands/results, honest R0A labels, residuals; restore TSV side-effects.
- **Touches:** this evidence file + captured wire/meta (if policy allows committing redacted last-run artifacts)
- **Depends on:** unit 2 green
- **SemVer:** none

#### PR unit 4 — `chore(release): bump product to 5.3.0` **(conditional)**
- **Intent:** Dedicated Spec 45 Deep Code cut at next free minor **only if** units 2–3 prove Path A R0A multi-edit + stale-id and gates green.
- **Touches:** version files + CHANGELOG (+ versions README as prior cuts)
- **Depends on:** unit 3 PASS with no R0A residual that blocks cut
- **SemVer:** **`5.3.0`**
- **Skip / fail closed:** if R0A residual remains → **do not** bump; leave status FAIL-CLOSED with residual list

#### Out of this story (explicit)

| Unit | Status here |
|------|-------------|
| VC007–VC015 | **not implemented** |
| npm publish / GitHub Release publish for 5.3.0 | optional separate lane; **not required** to open PR; do not claim packaging without doing it |
| Resume/fork snippet table persistence | **not required** for VC006 R0A cut; fail-closed empty table on transcript-only resume remains ADR residual honesty |
| Board SemVer rebase of WAVE tracks | separate docs PR if needed |

### 3.2 Sequential vs parallel

#### Sequential (must order)

1. **VC005 / #137** → **VC006 unit 1 (docs)**.
2. **unit 1** → **unit 2** (R0A harness) → **unit 3** (evidence fill).
3. **unit 3 green** → **unit 4** (SemVer **5.3.0**) **only if** no R0A residual.
4. Independent read-only Grok adversarial review **before** READY / PR open.

#### Parallel (safe concurrent)

- None on the same scripted R0A harness / public-entry evidence surface.
- Pure docs that do not redefine Spec 45 cut SemVer may proceed independently.

```text
VC003 (#130) ──► VC004 (#135) ──► VC005 (#137) ──► VC006 (R0A + cut @ 5.3.0)
```

### 3.3 Atomic commits (on `vc006-heart-r0a`)

```text
docs(product): VC006 Path A heart R0A plan + evidence
test(scripts): Path A R0A multi-edit + stale-id snippet_id harness
docs(product): record VC006 Path A R0A + gate evidence
chore(release): bump product to 5.3.0   # only if R0A + gates allow
```

Optional follow-ups only if justified:

```text
fix(scripts): harden Path A VC006 R0A harness …
docs(product): record VC006 adversarial READY verdict
```

| Do | Do not |
|----|--------|
| One concern per commit | Mix VC007+ Reasonix / L3 dogfood into this branch |
| Public Path A R0A with `snippet_id` primary | Claim thin oracle or unit-only as R0A |
| English Conventional subjects | Reuse **5.2.0 / 5.2.1 / 5.2.2** |
| Fail closed on residual | Invent wire green / silent dual-accept as complete |
| Restore gate TSV side-effects | Commit generated OWNER_BAR / HEART TSV rewrites |

### 3.4 Chaining / stacking

| Pattern | Choice for VC006 |
|---------|------------------|
| **Base (at open)** | **`vc005-snippet-invalidation`** (stacked on open PR **#137**) — **not** `origin/main` |
| **Branch** | `vc006-heart-r0a` |
| **After #137 merges** | **Rebase / retarget** onto updated **`main`** (do **not** do this before open) |
| **Merge order** | #130 → #135 → #137 → VC006 → later vision units |
| **Conflict lock** | Path A R0A harness + Spec 45 cut evidence + optional `5.3.0` bump owned by VC006 |

**Planned PR title (when opened later):** `test+chore(release): Path A Spec 45 multi-edit R0A + 5.3.0 cut`  
(or split title if unit 4 skipped: `test(scripts): Path A Spec 45 multi-edit R0A + heart evidence`)  
**Label kind:** `test` and/or `chore` as justified by final diff  
**Body:** Problem / What changed / Testing honesty / AI review / Security / Notes; **`Depends on #137`**; base **`vc005-snippet-invalidation`**; SemVer **none** or **`5.3.0`** only if unit 4 included; does **not** reuse **5.2.0–5.2.2**.

---

## 4. Acceptance criteria (VC006)

| ID | Criterion | Pass condition |
|----|-----------|----------------|
| **VC006-A1** | Plan committed first | Atomic docs commit before harness/source edits |
| **VC006-A2** | Public Path A multi-edit R0A | ≥3 successful edits / ≥2 files via public CLI → agent; edits use real **`snippet_id`**; disk goldens match; wire+meta captured |
| **VC006-A3** | Stale-id / invalidation R0A | After expire (edit and/or bash), reused id fails closed; no unauthorized disk write |
| **VC006-A4** | Wire honesty | Artifacts show Path A tool names (`read_file` / `search_replace` / bash family as applicable) and `snippet_id` args/results; labeled **R0A**, not thin oracle |
| **VC006-A5** | Dual CLI | `deepseek-build` and `dsb` still resolve / version; public entry uses product agent resolution (no forced `DEEPSEEK_BUILD_AGENT_BIN` for the claim) |
| **VC006-A6** | Owner-bar gate | `./scripts/test-owner-bar.sh` exit 0 |
| **VC006-A7** | Path-linkage gate | `./scripts/check-path-a-linkage.sh` exit 0 |
| **VC006-A8** | Heart regression gate | `./scripts/test-heart-regression.sh` exit 0 (live L3 SKIPs remain honest) |
| **VC006-A9** | Unit regressions intact | VC003/VC004/VC005 focused suites still green on stack |
| **VC006-A10** | Thin oracle not claimed as R0A | Any `dsb-tools` greens labeled oracle only |
| **VC006-A11** | SemVer cut discipline | No version bump until A2–A8 green; cut is **`5.3.0`** only; never reuse 5.2.0–5.2.2 |
| **VC006-A12** | No VC007–VC015 scope | Diff does not implement Reasonix effort wire, L3 R0A dogfood, freeze cut, etc. |
| **VC006-A13** | Adversarial review | Independent read-only Grok review before READY |
| **VC006-A14** | Stacked PR discipline | Base **`vc005-snippet-invalidation`**, body **Depends on #137**, English public text, labels, **do not merge** |

### Explicit non-claims (fail-close)

- Thin Path B unit greens alone do **not** complete VISION L1 / Spec 45.
- Do **not** claim R0A without real public wire artifacts and honest labels.
- Do **not** claim historical G004 `file_version`-only liveness as VC006 `snippet_id` multi-edit proof.
- Do **not** claim packaging/npm/GitHub Release of a version not cut in this story.
- Do **not** reuse **5.2.0 / 5.2.1 / 5.2.2**.
- Do **not** open or merge PR until READY (R0A + gates + adversarial).
- Do **not** implement VC007–VC015.
- Do **not** require resume/fork table persistence for this cut (ADR §9.3 residual honesty remains unless separately proven).
- Board WAVE still listing Spec 45 cut as **5.2.0** is a **docs residual**, not authority over live SemVer floor.

---

## 5. Security / cache boundaries

| Concern | Rule |
|---------|------|
| Spec 10 stable prefix | Snippet table **must not** appear in stable-prefix bytes |
| Cross-session leakage | IDs bind to owning session `Resources`; no process-global table |
| Public wire artifacts | Redact secrets; scripted key `sk-scripted-*` only |
| Agent resolution | Product paths; unset `DEEPSEEK_BUILD_AGENT_BIN` for public-entry claim |
| Force overwrite | Host-only (VC005); not model free boolean |
| Dual CLI | No rename of `deepseek-build` / `dsb` |
| Dependencies | Prefer zero new crates; harness is Python/bash only |
| Permissions | Spec 90 gates remain; R0A may use yolo/always-approve hermetic config (document honesty) |

---

## 6. Validation commands

```bash
# Floor re-check
git fetch origin main
git show origin/main:Cargo.toml | rg 'version = "'
npm view @innocarpe/deepseek-build version
gh release list -R innocarpe/deepseek-build --limit 8

# Whitespace / status
git diff --check
git status --short
git diff --stat

# Path A unit regressions (stack)
cd third_party/grok-build
CARGO_INCREMENTAL=0 RUSTC_WRAPPER= cargo test -p xai-grok-tools --lib vc005
CARGO_INCREMENTAL=0 RUSTC_WRAPPER= cargo test -p xai-grok-tools --lib vc004
CARGO_INCREMENTAL=0 RUSTC_WRAPPER= cargo test -p xai-grok-tools --lib vc003

# Thin oracle (reference only)
cargo test -p dsb-tools snippets
cargo test -p dsb-tools path_a_edit

# Required project gates
./scripts/test-owner-bar.sh
./scripts/check-path-a-linkage.sh
./scripts/test-heart-regression.sh

# VC006 Path A R0A (public CLI + scripted server) — exact command filled after harness lands
# ./scripts/test-path-a-vc006-r0a.sh   # planned

# Restore generated gate TSV side-effects to HEAD (do not commit)
git checkout HEAD -- docs/product/evidence/OWNER_BAR_STATUS.tsv \
  docs/product/evidence/PATH_A_R0_G010_HEART_REGRESSION_last.tsv 2>/dev/null || true
```

**R0A public wire:** claimed **only** when unit 2 runs and §7 records artifacts with honest labels.

---

## 7. Implementation evidence (filled after code)

### 7.1 Atomic commits on `vc006-heart-r0a`

| Order | SHA (prefix) | Subject | Contents honesty |
|------:|--------------|---------|------------------|
| 1 | `4642a3d` | `docs(product): VC006 Path A heart R0A plan + evidence` | Plan only (this file first) |
| 2 | `97171b4` | `test(scripts): Path A R0A multi-edit + stale-id snippet_id harness` | Scripted scenarios + public driver + wire/meta |
| 3 | `f25fd6c` | `docs(product): record VC006 Path A R0A + gate evidence` | Validation fill + refreshed wire @ harness SHA |
| 4 | `95ad990` | `chore(release): bump product to 5.3.0` | Dedicated cut after R0A + gates |
| 5 | tip (`git log -1`) | `docs(product): record VC006 adversarial READY + harness harden` | Review fill + multi-edit wire assert |

**No VC007–VC015 behavior** in these commits: no Reasonix effort wire, no L3 dogfood R0A, no freeze cut.

### 7.2 What shipped (code / harness)

| Piece | Location / behavior |
|-------|---------------------|
| Scripted scenarios | `scripts/lib/scripted_deepseek_server.py` — `snippet-multiedit`, `snippet-stale-id`, `snippet-bash-stale` |
| Public R0A driver | `scripts/test-path-a-vc006-r0a.sh` — public `deepseek-build`/`dsb` → hermetic home → stack-built `xai-grok-pager` as agent |
| Multi-edit | read → edit ×3 across `a.txt`/`b.txt` using real `snippet_id` (re-read after expire) |
| Stale-id | valid edit expires id → reuse same id → `snippet_not_found`; disk stays `edited-once` |
| Bash-stale | mint → bash mutate → old id → `snippet_not_found`; disk stays `mutated-by-bash` |
| Dual CLI | Unchanged (`deepseek-build` / `dsb`) |
| SemVer | unit 4 bumps to **`5.3.0`**; live `origin/main` floor **`5.2.2`**; does not reuse **5.2.0–5.2.2** |

### 7.3 Acceptance matrix

| Check | Result | Evidence class |
|-------|--------|----------------|
| Evidence doc committed first | **PASS** — `4642a3d` | commit |
| Multi-edit Path A R0A (`snippet_id`) | **PASS** — `MULTIEDIT_PASS` a=`hello2` b=`world1`; ≥3 successful edits; mint meta present | **R0A** |
| Stale-id Path A R0A | **PASS** — `STALE_ID_PASS` disk=`edited-once`; wire `snippet_not_found` | **R0A** |
| Bash invalidation Path A R0A | **PASS** — `BASH_STALE_PASS` disk=`mutated-by-bash`; wire `snippet_not_found` | **R0A** |
| Owner-bar gate | **PASS** — `PASS=60 FAIL=0 NOT_RUN=0` | gate |
| Path-linkage gate | **PASS** | gate |
| Heart regression gate | **PASS** — live L3.1–L3.5 **SKIP**; PATH_A_E2E **SKIP** (VC006 R0A is separate script) | gate |
| VC003 unit regressions | **PASS** — **11** tests | unit |
| VC004 unit regressions | **PASS** — **9** tests | unit |
| VC005 unit regressions | **PASS** — **20** tests | unit |
| Thin oracle | **PASS** — snippets **9** + path_a_edit **8** | thin oracle (**not** Path A proof) |
| SemVer **5.3.0** cut | **LANDED** on branch — `95ad990` (`Cargo.toml` / `package.json` / lock / CHANGELOG); **not** npm/GitHub packaged in this story | release |

### 7.4 Artifacts (R0A)

| Scenario | Meta | Wire |
|----------|------|------|
| multi-edit | `PATH_A_R0_VC006_snippet-multiedit_META_last.txt` | `…_WIRE_last.jsonl` |
| stale-id | `PATH_A_R0_VC006_snippet-stale-id_META_last.txt` | `…_WIRE_last.jsonl` |
| bash-stale | `PATH_A_R0_VC006_snippet-bash-stale_META_last.txt` | `…_WIRE_last.jsonl` |

**Refresh run:** harness commit **`97171b4`**, agent `xai-grok-pager` sha256 `1962a3de2190613b54f983fe12bc00a834be6b7510a2fcb0c6d8775d26f3cdbd`, public CLI from worktree `target/release/deepseek-build`, `DEEPSEEK_BUILD_AGENT_BIN` unset.

**Wire honesty excerpts (tool results):**

- Multiedit: `snippet_id: snp_…` on read_file results; three “updated successfully” edits; final disk goldens.
- Stale-id / bash-stale fail tool text:

```text
snippet_not_found: unknown snippet_id for this session; re-read before edit
```

### 7.5 Commands actually run (exact)

```bash
# Floor
git fetch origin main
git show origin/main:Cargo.toml | rg 'version = "'
# → 5.2.2
npm view @innocarpe/deepseek-build version
# → 5.2.1 (lag)
gh release list -R innocarpe/deepseek-build --limit 8
# Latest v5.2.1

# Plan first
# commit 4642a3d docs(product): VC006 Path A heart R0A plan + evidence

# Agent build from stack
cd third_party/grok-build
CARGO_INCREMENTAL=0 RUSTC_WRAPPER= cargo build --release -p xai-grok-pager-bin
# Finished; binary target/release/xai-grok-pager

# R0A
./scripts/test-path-a-vc006-r0a.sh --skip-build
# PASS — snippet-multiedit / snippet-stale-id / snippet-bash-stale
# harness commit 97171b4; refresh re-run same SHA

# Units
cd third_party/grok-build
CARGO_INCREMENTAL=0 RUSTC_WRAPPER= cargo test -p xai-grok-tools --lib vc005  # 20 passed
CARGO_INCREMENTAL=0 RUSTC_WRAPPER= cargo test -p xai-grok-tools --lib vc004  # 9 passed
CARGO_INCREMENTAL=0 RUSTC_WRAPPER= cargo test -p xai-grok-tools --lib vc003  # 11 passed
cargo test -p dsb-tools snippets     # 9 passed (oracle)
cargo test -p dsb-tools path_a_edit  # 8 passed (oracle)

# Gates
./scripts/test-owner-bar.sh          # ALL PASS PASS=60
./scripts/check-path-a-linkage.sh    # PASS
./scripts/test-heart-regression.sh   # PASS (live L3 SKIP; PATH_A_E2E SKIP)

# Restore TSV side-effects (not committed)
git checkout HEAD -- docs/product/evidence/OWNER_BAR_STATUS.tsv \
  docs/product/evidence/PATH_A_R0_G010_HEART_REGRESSION_last.tsv
```

**Honesty:** Multi-edit / stale-id / bash-stale claims are **public Path A R0A** (CLI → agent_launch → product agent + scripted DeepSeek wire). Unit greens remain labeled unit; thin `dsb-tools` is oracle only. Gate TSV rewrites restored and not committed. Live product floor on `origin/main` is **`5.2.2`** at **`d71c1b3`**. Stack product version before unit 4 is **`5.2.1`** (inherited); next free feature minor cut is **`5.3.0`**. npm/GitHub Latest lag at **5.2.1**/**v5.2.1** and the separate **5.2.2 packaging lane** are not claimed here. **Open stacked** against **`vc005-snippet-invalidation`** with body **`Depends on #137`**.

### Required project gates (verified)

| Gate | Exit | Result | Honesty |
|------|------|--------|---------|
| `./scripts/test-owner-bar.sh` | **0** | **ALL PASS** — `PASS=60 FAIL=0 NOT_RUN=0` | Owner-bar green on HEAD |
| `./scripts/check-path-a-linkage.sh` | **0** | **PASS** | NOTE: third_party/grok-build has no dsb-* Cargo dep (expected until F1) |
| `./scripts/test-heart-regression.sh` | **0** | **PASS** | Live L3.1–L3.5 **SKIP**; `PATH_A_E2E` **SKIP** (VC006 R0A is `test-path-a-vc006-r0a.sh`) |
| `./scripts/test-path-a-vc006-r0a.sh` | **0** | **PASS** | Public Path A R0A multi-edit + stale-id + bash-stale |
| `git status` after restore | clean of TSV side-effects | — | wire/meta + evidence edits only |

### 7.6 Residuals (fail-closed / non-blocking)

| Residual | Severity | Note |
|----------|----------|------|
| Board WAVE still names Spec 45 cut as **5.2.0** | docs | Stale vs live floor; not authority for this cut |
| npm/GitHub Latest still **5.2.1** while main is **5.2.2** | ops | Separate packaging lane; this story cuts **5.3.0** on stack only |
| Live L3.1–L3.5 Path A R0A | out of scope | VC010+; honest SKIP |
| Resume/fork snippet table persistence | ADR §9.3 residual | Not required for this cut |
| Historical `liveness-3edits` still `file_version`-only | non-blocking | Superseded by VC006 `snippet-*` scenarios for Spec 45 proof |

### 7.7 Independent adversarial review (read-only Grok)

| Field | Value |
|-------|--------|
| Reviewer | Separate read-only Grok code-reviewer lane (not the implementer self-approve) |
| Scope | Branch `vc006-heart-r0a` / R0A harness + wire honesty + gates + SemVer cut discipline |
| **Verdict** | **READY** |
| **P0** | **none** |
| **P1 (addressed before PR)** | (1) Evidence lag after unit 4 cut → filled SHAs / LANDED / READY here; (2) multi-edit harness now asserts ≥3 `search_replace` tool_calls with `snippet_id` on wire; (3) board WAVE SemVer residual → PR body only, not authority; (4) CHANGELOG missing `5.2.2` on stack → reconcile on rebase after #137/main |
| **P2 residuals (non-blocking)** | META git_sha at harness `97171b4` not tip; dual CLI R0A via `deepseek-build` (`dsb --version` soft); scripted non-stream path latent; live L3 R0A out of scope; resume/fork table residual |
| SemVer / packaging | No false claim of npm/GitHub publish; on-branch **5.3.0** only; no reuse of **5.2.0–5.2.2** |
| VC007+ smuggling | **none** found |
| R0A residual blocking cut | **none** |

---

## 8. References

- ADR: [0010-spec-45-snippet-store](../../adr/0010-spec-45-snippet-store.md)
- Spec: [45-snippet-edit](../../specs/45-snippet-edit.md)
- VC005 evidence: [VC005_PATH_A_SNIPPET_INVALIDATION_2026-08-08.md](./VC005_PATH_A_SNIPPET_INVALIDATION_2026-08-08.md)
- VC004 evidence: [VC004_PATH_A_SNIPPET_ID_REQUIRE_2026-08-08.md](./VC004_PATH_A_SNIPPET_ID_REQUIRE_2026-08-08.md)
- VC003 evidence: [VC003_PATH_A_SNIPPET_ID_2026-08-08.md](./VC003_PATH_A_SNIPPET_ID_2026-08-08.md)
- Prior spirit: [G004_SNIPPET_LIVE_2026-08-07.md](./G004_SNIPPET_LIVE_2026-08-07.md) · [G005_WRITE_BASH_INVALIDATE_2026-08-07.md](./G005_WRITE_BASH_INVALIDATE_2026-08-07.md)
- Public entry: `scripts/test-path-a-public-entry-e2e.sh`
- Scripted wire: `scripts/lib/scripted_deepseek_server.py`
- Planning: [ULTRAGOAL_PR_PLANNING](../ULTRAGOAL_PR_PLANNING.md) · [WAVE_5x_VISION_PR_DAG](../WAVE_5x_VISION_PR_DAG.md)
