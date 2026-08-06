# Spec 45 — Snippet edit contract

| Field | Value |
|-------|--------|
| Status | **ready-for-impl** |
| Philosophy | HARNESS §4.1 Pillar A (Deep Code); bypass law for `write` / `bash` |
| Gate | Part of **G3** (blocks M2 mutating edit) |
| Tests | **Automated golden + negative required** |

## 1. Behavior

Editing text files **must** go through session-local **snippets**. Free-form whole-file `old_string`/`new_string` without a valid `snippet_id` is **not** the primary path and must not ship as default.

### 1.1 Snippet identity

A snippet is a session-owned record:

| Field | Meaning |
|-------|---------|
| `snippet_id` | Opaque, unique within the session (e.g. `snp_<ulid>`). Not a path. |
| `path` | Workspace-relative when under workspace root; else absolute normalized. |
| `range` | Inclusive line range `[start_line, end_line]` (1-based) **or** byte offsets — product picks one representation and documents it; tests pin the choice. **M2 default:** 1-based line range. |
| `version` | Monotonic content fingerprint of the file at issue time (see §1.3). |
| `scope` | `lines` \| `whole_file` \| `symbol` (M2 implements `lines` + `whole_file`; `symbol` optional later). |
| `preview` | Short text preview for model UX (may be truncated; not used for match). |
| `encoding` | `utf-8` default; non-UTF-8 text → fail closed or binary path (no silent mojibake edit). |
| `issued_at_turn` | Session turn index when created (audit only; not for cache prefix). |

Snippets live in a **session snippet table**. They are **not** written into the stable API prefix (spec 10).

### 1.2 `read` issues snippets

For text files under the allowed read policy (spec 90):

1. Load file bytes; reject or binary-handle non-text.  
2. Optionally accept `start_line` / `end_line` (default: whole file or a product max window — document default; large files must not dump unbounded content without range).  
3. Upsert session file state for `path` with current `version`.  
4. Return to the model:
   - content (or ranged content),
   - `snippet_id`,
   - metadata: path, range, version, scope, preview.  
5. Multiple reads of the same path may mint **new** `snippet_id`s; older IDs remain valid until expired (§1.6) if version still matches.

### 1.3 Version check

`version` **must** change when file content changes. Recommended M2 algorithm:

```text
version = hex(sha256(file_bytes))   // full file at read/edit time
```

Optional fast path: `size + mtime_ns` is **not** sufficient alone (clock skew / same-size edits). If used, must be combined with a content hash of the snippet range **and** full-file hash for the check at edit time.

At `edit` time:

1. Resolve `snippet_id` → snippet record. Missing → error `snippet_not_found`.  
2. Re-hash current file (or fail if path missing) → `current_version`.  
3. If `current_version != snippet.version` → error `snippet_stale`; **do not apply**. Optionally re-read and return a new snippet.

### 1.4 `edit` contract

Required tool arguments (normative names; wire schema in spec 40):

| Arg | Required | Notes |
|-----|----------|--------|
| `snippet_id` | **yes** | Session-valid |
| `old_string` | yes* | Exact text to find **within snippet scope only** |
| `new_string` | yes* | Replacement |
| `expected_count` | no | If set, must match occurrence count inside scope; else error |

\*Empty `old_string` is only allowed for documented insert-at-range modes; M2 default **forbids** empty `old_string`.

**Algorithm:**

1. Permission gate for write on path (spec 90) — before mutation.  
2. Version check (§1.3).  
3. Extract **scope text** = file content for snippet range (line range mapped to bytes with stable newline rules: prefer `\n` internally).  
4. Search `old_string` **only inside scope text** (literal match; no regex unless a separate tool).  
5. Occurrence handling:
   - **0 matches** → error `no_match`; return scope preview.  
   - **1 match** → replace that occurrence.  
   - **>1 matches** and no `expected_count` → error `ambiguous_match`; return **candidate snippets** (split ranges or previews) — **do not guess**.  
   - **>1 matches** with `expected_count` → replace all only if count equals `expected_count`; else error.  
6. Write file atomically (write temp + rename, or equivalent).  
7. Update session file state; **expire** all snippets for that path whose version is now stale (§1.6).  
8. Return success + optional new `snippet_id` for the edited region.

### 1.5 `write` bypass law

| Case | Behavior |
|------|----------|
| Path does **not** exist | Create allowed if permissions allow `write-in-cwd` / configured create scope. No snippet required. |
| Path **exists** | Default: **deny** overwrite via `write`. Model must `read` + `edit` with `snippet_id`. |
| Force overwrite | Only if **user/policy** grants an explicit capability (e.g. permission scope or confirmed flag). **Not** a free model boolean that skips version checks without policy. If force is granted, still revalidate permissions and expire snippets for path. |

### 1.6 Invalidation / expiry

Expire (remove or mark unusable) snippets when:

| Event | Scope |
|-------|--------|
| Successful `edit` / force-`write` on path | All snippets for that path |
| Detected external mtime/content change at next `edit`/`read` | Stale IDs fail version check |
| `bash` may have mutated tree (spec 90 high side-effect) | **Revalidate** touched paths; expire snippets for paths classifier/heuristic marks dirty; if unknown which paths → expire **all** workspace snippets or re-hash open paths (document product choice; M2 default: expire snippets for paths present in bash stdout/stderr path heuristics **and** any path the model declared; if classifier says file-mutation with unknown set → expire all session file snippets under workspace) |
| Session end | Table discarded |
| Subagent/worktree (spec 60 later) | Worker must not use parent `snippet_id`s; parent invalidates on path touch from worker results |

### 1.7 Concurrency (single agent process, M2)

M2 is single-threaded tool dispatch unless spec 50 lands. Still:

- Do not apply two edits to the same path without version bump between them.  
- Parallel tools (spec 50): edits on same path serialize or fail; first writer wins + others get `snippet_stale`.

### 1.8 Non-text / binary

- Binary or undecodable files: `read` may return metadata only; **no** `snippet_id` for binary edit.  
- Line-ending policy: preserve original file newline style on write when possible; tests pin `\n`-only fixtures.

## 2. Non-goals

- Primary free-form whole-file edit without snippets  
- Multi-file transactional edit in one tool call (M2+)  
- Language-aware AST edit (optional later)  
- Grok hashline as a **replacement** for `snippet_id` (may only implement if §1 semantics hold)

## 3. Failure modes

| Case | Behavior |
|------|----------|
| `edit` without `snippet_id` | Schema/validation error; **no disk write** |
| Stale version | `snippet_stale`; no write |
| Ambiguous match | `ambiguous_match` + candidates; no write |
| Path escape outside policy | Permission deny (spec 90); no write |
| Snippet path ≠ resolved path after symlink policy | Fail closed; document symlink resolution once |

## 4. Test plan (automated)

| Test | Expect |
|------|--------|
| `read_returns_snippet_id` | metadata includes id, path, version, range |
| `edit_requires_snippet_id` | missing id → no file change |
| `edit_applies_within_scope` | only scope region changes; golden file bytes |
| `edit_stale_version_rejected` | external/file change → error, content unchanged |
| `edit_ambiguous_no_guess` | two identical blocks in scope → candidates, no write |
| `edit_expected_count` | count mismatch fails; match succeeds |
| `write_create_new_ok` | new path created |
| `write_existing_denied_by_default` | existing path not overwritten without policy force |
| `bash_mutation_expires_snippets` | after dirty bash, old snippet_id fails version/expiry |
| `snippet_not_in_stable_prefix` | snippet table not serialized into spec 10 stable prefix bytes |

## 5. Implementation notes

- Crate: `dsb-tools` (or `dsb-snippets` inside tools) per ADR 0004.  
- Wire tool schemas land in spec **40**; this spec owns **semantics**.  
- Symlink policy: resolve paths under workspace root; deny link escape unless `read-out-cwd` / `write-out-cwd` allowed.  
- Interact with spec **15**: never invent `snippet_id` during repair.  
- Interact with spec **90**: permission check before any mutation.  
