# ADR 0010 — Spec 45 session SnippetStore (Path A `snippet_id`)

- **Status:** Accepted
- **Date:** 2026-08-07
- **Story:** VC002 (`vision-complete-5x`)
- **SemVer impact of this ADR alone:** **none** (design document)
- **Target product cut for runtime:** **`5.2.0`** (VC003–VC006)
- **Normative companions:**
  - [Spec 45 — Snippet edit](../specs/45-snippet-edit.md) (semantics SSOT)
  - [Spec 40 — Core tools surface](../specs/40-core-tools-surface.md) (wire schemas)
  - [Spec 90 — Permissions](../specs/90-permissions.md) (permission-before-mutation)
  - [HARNESS_PHILOSOPHY](../architecture/HARNESS_PHILOSOPHY.md) §4.1 Pillar A
  - [HEART_3X_SPEC_BINDING](../architecture/HEART_3X_SPEC_BINDING.md)
  - Evidence / PR plan: [VC002_SPEC45_ADR_2026-08-07.md](../product/evidence/VC002_SPEC45_ADR_2026-08-07.md)

## Context

DeepSeek Build’s L1 Deep Code contract requires **session-local snippets**: `read` issues a `snippet_id`; `edit` is scope-bound and version-checked; free-form whole-file `old_string`/`new_string` is **not** the product primary path ([Spec 45](../specs/45-snippet-edit.md), [HARNESS_PHILOSOPHY](../architecture/HARNESS_PHILOSOPHY.md) §4.1).

### Two paths (honesty)

| Path | Entry | Spec 45 status (as of ADR date) |
|------|-------|----------------------------------|
| **A — Product agent (default)** | `dsb` / `deepseek-build` → `deepseek-build-agent` (vendored Grok) | Owner-bar **`file_version` (sha256) + `snippet_safe`** on `search_replace`; **no** full session `snippet_id` table on Path A wire |
| **B — Thin / legacy** | `dsb run` / `dsb chat` / `dsb-tools` | `crates/dsb-tools` **`SnippetStore`** implements issue/edit/expire with automated tests |

Heart binding already states: Path B green tests **do not** prove Path A. Vision-complete V1 still requires real **`snippet_id`** on Path A ([VISION_COMPLETE_5X_GOALS](../product/VISION_COMPLETE_5X_GOALS.md) V1-45-*).

### Why an ADR now

Spec 45 is **ready-for-impl** but leaves several product pins open (range representation, unknown-bash default, subagent ownership, dual-accept window, encoding fields). Overnight implementers risk:

1. Treating thin `SnippetStore` unit green as Path A complete.
2. Shipping Grok hashline/anchors that skip version/scope semantics.
3. Divergent error strings and invalidation rules across VC003–VC005.
4. Putting snippet tables into Spec 10 stable prefix (cache thrash).

This ADR pins **one implementable store contract** for Path A. It **does not** claim Path A or R0A multi-edit is already green.

### Reference implementation (not completion proof)

**Thin `crates/dsb-tools` `SnippetStore` is the reference design and test oracle** for algorithm shape:

- Opaque `snp_<ulid>` IDs
- Inclusive 1-based line ranges
- `version = hex(sha256(full_file_bytes))`
- Scope-only literal replace, ambiguity/count errors
- Atomic write (temp + rename)
- `expire_path` / `expire_all`
- Create-only `write_new` → `path_exists_use_edit`

Known thin gaps relative to full Spec 45 (must be closed on Path A, may be back-ported to thin):

| Gap in thin `Snippet` / edit today | ADR requirement |
|------------------------------------|-----------------|
| No `encoding` field | Required on record (default `utf-8`) |
| No `issued_at_turn` | Required (audit only) |
| Limited binary / non-UTF-8 handling | Fail closed; no snippet for binary edit |
| `ambiguous_match` without candidate snippets | Return candidate ranges/previews; no guess |
| `expected_count` ignored when occurrence count is **1** (thin accepts any set value) | If `expected_count` is **set**, it **must equal** actual scope occurrence count — including when count is **1**; else **`expected_count_mismatch`** (Spec 45 arg table + `edit_expected_count` test intent) |
| Subagent / worktree / resume ownership | Normative rules below |
| Symlink escape policy | Fail closed under Spec 90 |

**Port or adapt** into Grok Path A boundaries ([HEART_3X_SPEC_BINDING](../architecture/HEART_3X_SPEC_BINDING.md) §2 Spec 45: Adapt SearchReplace + optional port of store). Do **not** maintain two divergent edit engines forever.

---

## Decision

### 1. Ownership and binding (Path A)

| Concern | Owner |
|---------|--------|
| **Product default enforcement** | **Path A** — Grok tool path used by `deepseek-build-agent` |
| **Reference algorithm + goldens** | **Path B thin** — `crates/dsb-tools` (`snippets.rs`, `tools.rs`, `path_a_edit.rs` adapter) |
| **Wire tool names on Path A** | Product may expose Grok names (`read_file`, `search_replace`, …) while preserving Spec 45 **semantics** and Spec 40 canonical spirit |
| **Session store lifetime** | One store per **agent session** (see §9 resume/fork) |
| **Stable prefix (Spec 10)** | Snippet table **must not** appear in stable prefix bytes |

**Conflict rule:** L3 parallelism / subagents / worktrees **must not** override L1 snippet consistency ([HARNESS_PHILOSOPHY](../architecture/HARNESS_PHILOSOPHY.md) §3).

**Path A completion** is proven only by later runtime units (VC003+) with wire + R0A evidence — **not** by merging this ADR.

### 2. Snippet record (session-local)

A snippet is a **session-owned** record. It is **not** a filesystem object and **not** a stable-prefix entry.

| Field | Type / shape | Normative meaning |
|-------|--------------|-------------------|
| `snippet_id` | string | Opaque, unique within the session. Format: **`snp_` + ULID** ( Crockford base32 ULID as today). **Not** a path, hash, or line number. |
| `path` | path string | Workspace-relative when under workspace root after resolve; else absolute normalized. Stored path must match the path used for permission + version checks. |
| `start_line` | usize (1-based) | Inclusive range start. |
| `end_line` | usize (1-based) | Inclusive range end; `end_line >= start_line`. |
| `version` | string | **`hex(sha256(full_file_bytes))` at issue time** — full file, not range-only. |
| `scope` | enum string | `lines` \| `whole_file` (M2). `symbol` deferred. Issue sets `whole_file` when range covers all lines of the file; else `lines`. |
| `preview` | string | Truncated text for model UX (**not** used for matching). Cap: **200** Unicode scalars + ellipsis (match thin default unless product docs raise it). |
| `encoding` | string | Default **`utf-8`**. Non-UTF-8 text → fail closed or binary path (no silent mojibake edit). |
| `issued_at_turn` | u64 / turn index | Session turn index at mint time. **Audit only** — never used for version equality or cache prefix. |

**Range representation pin:** product uses **inclusive 1-based line ranges** only for M2 / 5.2.0. Byte-offset ranges are **out of scope** unless a superseding ADR amends this.

**Multiple IDs:** repeated `read` of the same path **mints new** `snippet_id`s. Older IDs remain usable until expired (§6) **if** the stored `version` still matches the file.

### 3. Version algorithm

```text
version = hex(sha256(file_bytes))   // entire file at read or pre-edit check
```

| Allowed | Forbidden as sole version |
|---------|---------------------------|
| Full-file SHA-256 hex | `mtime` alone |
| | `size + mtime` alone |
| | Range-only hash without full-file check at edit |

Optional future fast path must still re-verify full-file hash at edit time before mutation.

### 4. `read` / `read_file` issuance (VC003)

For text files under allowed read policy (Spec 90):

1. Resolve path (symlink policy §8).
2. Load file bytes; reject or binary-handle non-text (§7) — **no** `snippet_id` for binary edit.
3. Decode as UTF-8 (or documented encoding); fail closed on undecodable text for edit paths.
4. Optional `start_line` / `end_line` (default: whole file or product max window — large files must not dump unbounded content without range; pin window in VC003 tests).
5. Compute `version` = sha256(full file).
6. Insert record into session table; return to model:
   - content (or ranged content),
   - `snippet_id`,
   - `path`, `start_line`, `end_line`, `version`, `scope`, `preview`
   - (and on Path A wire, continue exposing `file_version` as an alias of `version` during any dual-accept window — see §5.1).
7. Do **not** serialize the table into Spec 10 stable prefix.

**Permission:** read scopes only; issuance is not a mutation.

### 5. `edit` / `search_replace` contract (VC004)

#### 5.1 Required identity

| Mode | Rule |
|------|------|
| **Product default (target 5.2.0)** | Edit of existing text **requires** valid session `snippet_id`. Missing → **no disk write**. |
| **Dual-accept window** | Only if product must keep owner-bar `file_version` temporarily: accept **either** valid `snippet_id` **or** matching full-file `file_version` with scope rules. **Deadline:** remove dual-accept no later than **`5.2.0` cut (VC006)** unless a superseding ADR extends with explicit residual. Prefer hard `snippet_id` as soon as VC003 mints IDs. |
| **Free-form primary** | **Rejected** when product snippet-safe policy is on (owner-bar residual becomes full Spec 45). |

Wire arg names (Spec 40 spirit; Path A may map):

| Arg | Required | Notes |
|-----|----------|--------|
| `snippet_id` | **yes** (default) | Session-valid |
| `old_string` | **yes** | Non-empty (M2 default forbids empty) |
| `new_string` | **yes** | Replacement |
| `expected_count` | no | If **set**, **must equal** actual occurrence count **inside scope** (Spec 45). Count `1` is **not** exempt. |

#### 5.2 Algorithm (normative order)

1. **Permission gate** for write on snippet path (Spec 90) — **before** any mutation. Deny/ask must leave no partial write.
2. Resolve `snippet_id` → record. Missing / expired → error **`snippet_not_found`**.
3. Re-hash current file (path missing → IO / not found class) → `current_version`.
4. If `current_version != snippet.version` → **`snippet_stale`**; **do not apply**. Optionally hint re-read (new snippet).
5. Extract **scope text** from file for `[start_line, end_line]` with stable newline rules (§7).
6. Literal search for `old_string` **only inside scope text** (no regex).
7. Occurrence handling (**fail-close on `expected_count`** — Spec 45: if supplied, **must equal** actual scope occurrence count; do **not** mirror thin-store `count==1 ⇒ ignore expected_count`). Count **0** is explicit: absent or `expected_count=0` → **`no_match`** (nothing to replace even when the count matches 0); nonzero expected with actual 0 → **`expected_count_mismatch`**:

| Count | `expected_count` | Result |
|------:|------------------|--------|
| 0 | absent | **`no_match`** (+ scope preview); no write |
| 0 | set to `0` | **`no_match`** (+ scope preview); no write — actual count matches, but nothing can be replaced |
| 0 | set, nonzero | **`expected_count_mismatch`**; no write |
| 1 | absent | replace that single occurrence |
| 1 | set, equals `1` | replace that single occurrence |
| 1 | set, ≠ `1` | **`expected_count_mismatch`**; no write |
| >1 | absent | **`ambiguous_match`** + **candidate snippets** (split ranges or previews); **do not guess**; no write |
| >1 | set, equals count | replace all occurrences left-to-right (exactly `expected_count` times) |
| >1 | set, ≠ count | **`expected_count_mismatch`**; no write |

8. Empty `old_string` → **`empty_old_string`** (default); insert-at-range modes only if separately documented later.
9. **Atomic write** (write temp in same directory + rename, or equivalent).
10. **Expire all snippets for that path** (successful mutation).
11. Return success; optionally mint a **new** `snippet_id` for the edited region (recommended, not required for VC004 green).

### 6. Invalidation / expiry

Expire (remove or mark unusable — product may delete from map) when:

| Event | Scope |
|-------|--------|
| Successful `edit` / force-`write` on path | All snippets for that path |
| Version mismatch at next edit/read check | Stale IDs fail (`snippet_stale`) even if not yet purged |
| Session end | Entire table discarded |
| Subagent / worktree path touch (§9) | Parent invalidates touched paths from worker results |

#### 6.1 `write` bypass law (VC005)

| Case | Behavior |
|------|----------|
| Path does **not** exist | Create allowed if permissions allow create/`write-in-cwd` (etc.). **No** `snippet_id` required. |
| Path **exists** | Default: **deny** overwrite via `write` / empty-old overwrite spirit → error **`path_exists_use_edit`**. Model must `read` + `edit` with `snippet_id`. |
| Force overwrite | Only if **user/policy** grants explicit capability (permission scope or confirmed session grant). **Not** a free model boolean that skips version checks without policy. If granted: revalidate permissions, write, **expire all snippets for path**. |

#### 6.2 `bash` / external mutation (VC005)

After bash (or equivalent shell tool) with effective scopes including `write-*`, `delete-*`, `mutate-git`, or **`unknown`** that may touch files:

| Knowledge of touched paths | Action |
|----------------------------|--------|
| Known set (declared paths + classifier/heuristic path extraction from command / tool metadata) | **`expire_path`** for each touched path |
| **Unknown** mutation set (classifier says file-mutation but paths unclear) | **Fail closed:** **`expire_all`** session file snippets under workspace (product M2 default) |
| Permission **deny** / no execution | **No** disk mutation and **no** snippet expiry |

External editors / user edits: detected at next version check → `snippet_stale` (lazy). Eager mtime watchers are optional, not required for 5.2.0.

Permission deny must not expire snippets (Spec 90 §1.7).

### 7. Non-text, newlines, encoding

| Topic | Decision |
|-------|----------|
| Binary / undecodable | `read` may return metadata only; **no** `snippet_id` for binary edit. Attempted edit without text snippet → fail closed. |
| Encoding | Default UTF-8; store `encoding` on record. No silent mojibake. |
| Newlines | Prefer `\n` internally for scope split; **preserve original file newline style on write when possible**. Tests pin `\n`-only fixtures. Trailing-newline fidelity must match thin-store goldens unless Path A documents a deliberate fix. |
| Empty file | Allowed; whole_file scope with `start_line=1`, `end_line=1` or empty range policy pinned in VC003 tests. |

### 8. Symlink and workspace policy

1. Resolve paths relative to **workspace root**.
2. Symlink that escapes workspace → require out-of-cwd read/write scopes (Spec 90); else **deny**.
3. Snippet `path` after resolve must be the path used for version hash and permission.
4. If symlink policy cannot reconcile snippet path with resolved path → **fail closed** (no write).

### 9. Concurrency, subagents, worktrees, resume/fork

#### 9.1 Single agent process

- Until Spec 50 parallel tools land fully: treat mutating tools as **serialized** for the same path.
- With parallel tools: same-path edits serialize or second writer gets **`snippet_stale`** after first write expires/bumps version.
- Never apply two edits to the same path without a version bump between successful writes.

#### 9.2 Subagents / workers (Spec 60 spirit; full dogfood may lag to 5.4.0)

| Rule | Behavior |
|------|----------|
| Worker **must not** use parent `snippet_id` values | Parent IDs are invalid in worker tool calls |
| Worker has its **own** session snippet table (or empty + mint on worker reads) | No shared mutable map without ownership |
| Worker mutation of a path | Parent **must invalidate** snippets for that path when applying worker results |
| Worktree isolation | Snippets are bound to the **workspace root / worktree** they were issued under; cross-worktree reuse of IDs is forbidden |

#### 9.3 Resume / fork

| Event | Snippet table |
|-------|----------------|
| **Hard session end** | Discard |
| **Resume same session** with intact runtime state | Table may persist if process state persisted |
| **Resume from transcript only** (no store snapshot) | Table **empty**; model must re-`read` (fail closed on old IDs → `snippet_not_found`) |
| **Fork session** | Child starts **empty** table (do not clone live IDs) |

Persisting the full snippet table to disk across processes is **optional** and must not enter Spec 10 stable prefix; if added later, needs security review (path + previews).

### 10. Stable prefix exclusion (Spec 10 / L2)

| Must stay out of stable prefix | May appear on turn tail / tool results |
|--------------------------------|----------------------------------------|
| Entire snippet table | Individual tool result JSON including `snippet_id` |
| Per-turn previews bulk dump as “system memory” | Normal tool responses |

Repair (Spec 15): **never invent** `snippet_id` during repair. Missing required id → validation error, not hallucinated ULID.

### 11. Exact error names (wire / structured)

Use these **stable string identities** (match Spec 45 + thin `EditError` / `WriteError` display). Path A may wrap with human text but must remain grep-stable in tests.

| Name | When | Disk write? |
|------|------|-------------|
| `snippet_not_found` | Unknown / expired id | no |
| `snippet_stale` | version mismatch | no |
| `no_match` | 0 occurrences in scope and `expected_count` is **absent** or **set to 0** (count matches 0 but nothing can be replaced) | no |
| `ambiguous_match` | >1 occurrences and `expected_count` **absent** | no |
| `expected_count_mismatch` | `expected_count` **set** and ≠ actual scope occurrence count (including actual **0** with nonzero expected, and actual **1** with wrong expected) | no |
| `empty_old_string` | empty `old_string` under default policy | no |
| `path_exists_use_edit` | `write` (or write-spirit) on existing path without force policy | no |
| `free_form_primary_rejected` / schema args error | edit without required `snippet_id` (and dual-accept not applicable) | no |
| Permission deny / ask (Spec 90 names) | policy blocks | no |
| IO errors | read/write/rename failures | no (failed atomic write must not leave corrupt primary path) |

Thin store today maps: `NotFound`→`snippet_not_found`, `Stale`→`snippet_stale`, `NoMatch`→`no_match`, `Ambiguous`→`ambiguous_match`, `CountMismatch`→`expected_count_mismatch`, `EmptyOld`→`empty_old_string`, `WriteError::Exists`→`path_exists_use_edit`. **Caveat:** thin currently returns `CountMismatch` mainly for **n>1** mismatches; for **n=1** with a wrong set `expected_count` it still applies (see thin gap table). Path A **must not** copy that hole.

### 12. Follow-on unit mapping

| Unit | Story | Implements | Acceptance sketch |
|------|-------|------------|-------------------|
| **VC002** | this ADR + evidence | Design only | ADR accepted; no SemVer |
| **VC003** | mint on Path A `read_file` | §4 issuance, multi-ID, encoding/turn fields, binary no-id | Wire shows `snippet_id`; unit mint; thin regression green |
| **VC004** | require on Path A edit | §5 algorithm, errors, dual-accept endgame | Free-form reject; stale/ambiguous goldens; scope-only replace |
| **VC005** | write + bash laws | §6.1–6.2 | `path_exists_use_edit`; expire_path; unknown → expire_all |
| **VC006** | **`5.2.0` cut** | Heart regression + multi-edit R0A + SemVer bump | V1-45-* evidence; owner-bar still green |

**Do not** implement VC003–VC005 in the VC002 docs PR.

---

## Alternatives considered

| ID | Option | Verdict |
|----|--------|---------|
| **A** | Keep Path A on **`file_version` only** forever (no `snippet_id` table) | **Rejected** for vision V1 — Spec 45 and VISION goals require session snippet identity and multi-snippet UX; file_version is owner-bar **spirit**, not full Deep Code contract |
| **B** | Make free-form Grok `search_replace` / hashline the primary path | **Rejected** — L1 Deep Code wins over L3 tool shape ([HARNESS_PHILOSOPHY](../architecture/HARNESS_PHILOSOPHY.md) §4.1 Grok note) |
| **C** | Greenfield second edit engine on Path A ignoring thin store | **Rejected** — duplicates bugs; thin store is reference + oracle |
| **D** | Port thin store **only** on Path B and claim vision complete | **Rejected** — product default is Path A |
| **E** | mtime/size versioning for speed | **Rejected** as sole version (Spec 45) |
| **F** | Put snippet table in stable API prefix for “memory” | **Rejected** — thrash + L2 cache invariant |
| **G** | Share parent snippet IDs with subagents for convenience | **Rejected** — ownership / stale races; fail-closed separate tables |
| **H** | Unknown bash mutation leaves snippets alive | **Rejected** — version checks would lie; expire_all fail-closed |

**Chosen:** Path A enforces Spec 45 via **session SnippetStore** (port/adapt from thin reference); full-file sha256; line-range scopes; write/bash laws as above.

---

## Consequences

### Positive

- VC003–VC005 have a single decision surface (record fields, algorithm order, error names, invalidation defaults).
- Thin store remains valuable as oracle; Path A implementers know **adapt/port**, not invent.
- Cache (Spec 10) stays protected from snippet thrash.
- Subagent/worktree rules prevent silent L3 override of L1.

### Negative / costs

- Path A must grow session state beyond today’s `file_version` string on tool args.
- Dual-accept (if used) adds temporary complexity — must end by **`5.2.0`** cut unless amended.
- Unknown bash expire-all is blunt (extra re-reads) — accepted for safety.
- Candidate-snippet UX on `ambiguous_match` needs careful tool-result size limits.

### Non-consequences (explicit)

- Accepting this ADR **does not** prove Path A R0A, owner-bar re-validation, or vision-complete.
- Does **not** bump SemVer.
- Does **not** amend Spec 45 text unless a later contradiction PR is required; this ADR **pins product choices** Spec 45 left open.
- Does **not** authorize YOLO permissions or inventing `snippet_id` in repair.

### Amendment path

Supersede named sections of this ADR with a new accepted ADR if:

- Dual-accept needs extension past 5.2.0,
- Byte-offset ranges become product default,
- Persist-across-resume store format is introduced,
- Or Path A tool names diverge enough to need a dedicated wire ADR.

---

## Implementation notes (non-normative)

- Prefer reusing `dsb_tools::SnippetStore` behind Path A hooks where linkage cost is low; otherwise re-host the same algorithm in the Grok tools crate with **shared golden cases**.
- Keep `path_a_edit.rs` as a contract adapter for tests until Grok path is fully bound.
- Interaction with Spec 40: schemas advertise required `snippet_id` on edit once dual-accept ends (cache epoch if schema changes mid-session).
- Interaction with Spec 90: permission before mutation; deny does not expire.
- Interaction with Spec 50/60: serialize same-path mutate; worker invalidates parent paths.

## References

- Thin code: `crates/dsb-tools/src/snippets.rs`, `tools.rs`, `path_a_edit.rs`
- Path A today: Grok `search_replace` `snippet_safe` + `file_version`
- Prior evidence: `docs/product/evidence/G003_*`, `G004_*`, `G005_*`, `H45_*`
- Planning: `docs/product/WAVE_5x_VISION_PR_DAG.md` (VC002–VC006)
