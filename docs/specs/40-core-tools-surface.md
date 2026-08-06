# Spec 40 — Core tools surface

| Field | Value |
|-------|--------|
| Status | **ready-for-impl** |
| Philosophy | HARNESS §4.1 Pillar A (tools shape); §4.3 small predictable action language; L1 Deep Code + L2 cache-stable schemas |
| Gate | No dedicated G-number; Wave B `0.8.0` product milestone. Specs **45** / **90** / **70** refine edit, permissions, skills |
| Tests | **Automated required:** registry name set, schema key stability, parse aliases, negative arg paths |

## 0. Purpose

Define the **small, predictable** built-in tool surface the model may call during coding sessions.

This spec owns:

1. **Which tools exist** (canonical names + allowed aliases).  
2. **Wire argument schemas** (OpenAI-style function tools).  
3. **Dispatch / response shape** at the tools runtime boundary.  
4. **Stability rules** so tool schemas stay in the **stable prefix** (spec 10) without thrash.

It does **not** re-specify snippet version algorithms (spec 45), permission scopes (spec 90), or skills discovery (spec 70) — those remain authoritative for their domains.

---

## 1. Catalog (normative minimum)

| Canonical name | Role | Mutating? | Primary specs |
|----------------|------|-----------|---------------|
| `read` | Read text file; **issues** `snippet_id` for `edit` | no | 45, 90 |
| `edit` | Snippet-scoped text replace | yes (in-scope) | 45, 90 |
| `write` | **Create-new only** file write | yes | 45, 90 |
| `grep` | Literal workspace text search | no | 40 (this), 90 |
| `bash` | Shell command (classify + optional execute) | maybe | 90, 45 (snippet expiry) |
| `skill` | On-demand skill body load by name | no | 70 |

**Out of this catalog (not Wave B `0.8.0`):**

| Tool | Status |
|------|--------|
| `ask_user` / interactive confirm | deferred (permissions UX `0.9.0`) |
| `web_search` | deferred |
| Parallel multi-tool fan-out | **G4** / Wave C |
| Subagent tools | **G5** / Wave C |
| MCP-mounted tools | **G6c** / `0.11.0` |
| Plan update tool | **G6d** / `0.11.0` |

### 1.1 Aliases (parse only)

The runtime **may** accept aliases when **parsing** tool call names. Schemas advertised to the model use **canonical** names only.

| Alias | Canonical |
|-------|-----------|
| `search` | `grep` |
| `load_skill` | `skill` |

Unknown names → error `unknown_tool` (do not invent mappings).

### 1.2 Count discipline

Default product goal: **≤ 8** built-in tools in the stable prefix. Adding a tool requires:

1. This catalog update (spec 40).  
2. A documented cache epoch if schema JSON changes mid-session (spec 10).  
3. Tests updating the registry golden set.

---

## 2. Wire schemas (normative)

Schemas are OpenAI-compatible function tools:

```text
{ "type": "function", "function": { "name", "description?", "parameters?" } }
```

`parameters` is a JSON Schema object with `"type": "object"`, property map, `required` array, and `"additionalProperties": false` for built-ins.

### 2.1 `read`

| Arg | Required | Type | Notes |
|-----|----------|------|-------|
| `path` | **yes** | string | Workspace-relative preferred |
| `start_line` | no | integer | 1-based inclusive |
| `end_line` | no | integer | 1-based inclusive |

**Success content (JSON fields):** `path`, `snippet_id`, `version`, `start_line`, `end_line`, `scope`, `preview`, `content` — see spec 45 §1.2.

### 2.2 `edit`

| Arg | Required | Type | Notes |
|-----|----------|------|-------|
| `snippet_id` | **yes** | string | From prior `read` |
| `old_string` | **yes** | string | Non-empty (M2 default) |
| `new_string` | **yes** | string | Replacement |
| `expected_count` | no | integer | Occurrence count inside scope |

Free-form whole-file edit **without** `snippet_id` is **not** a supported primary path (spec 45). Missing `snippet_id` → args error.

### 2.3 `write`

| Arg | Required | Type | Notes |
|-----|----------|------|-------|
| `path` | **yes** | string | |
| `content` | **yes** | string | Full file body |

If path already exists → structured error `path_exists_use_edit` (no overwrite). Overwrite-with-force is **not** a free model argument in this band.

### 2.4 `grep`

| Arg | Required | Type | Notes |
|-----|----------|------|-------|
| `pattern` | **yes** | string | Non-empty **literal** substring (not PCRE) |
| `path` | no | string | File or directory under workspace; default `"."` |
| `glob` | no | string | Extension filter without star, e.g. `"rs"` / `"md"` |
| `case_insensitive` | no | boolean | Default `false` |
| `max_matches` | no | integer | Default **50**, clamp **[1, 500]** |

**Success content:** `pattern`, `path`, `match_count`, `files_scanned`, `truncated`, `matches[]` where each match has `path`, `line` (1-based), `text` (may truncate long lines).

**Skip dirs (implementation minimum):** `.git`, `target`, `node_modules`, `.deepseek-build`, `dist`, `build`.

Prefer `grep` over `bash` for workspace text search (cost + permission clarity).

### 2.5 `bash`

| Arg | Required | Type | Notes |
|-----|----------|------|-------|
| `command` | **yes** | string | Shell string |
| `side_effects` | **yes** | string[] | Model-declared scopes (advisory; classifier authoritative — spec 90) |

**Execution gate:** Runtime may classify/permission-check without running. Live execution requires an explicit product flag (e.g. `--bash-execute` or `--dogfood`). When execution is disabled, response may include `"dry_run": true` plus `classified` scopes.

**Snippet rule:** Mutating bash (write/delete/git-mutate/unknown effective scopes) **must** expire outstanding snippets for affected paths; product default may expire **all** workspace snippets on mutating bash (spec 45/90 tests).

### 2.6 `skill`

| Arg | Required | Type | Notes |
|-----|----------|------|-------|
| `name` | **yes** | string | Skill directory name from skills index |

Loads **body on demand**. Does **not** mutate the stable skills **index** in the prefix (spec 10 / 70). `mutated` on the tool response is always `false` for a pure load.

---

## 3. Runtime boundary

### 3.1 Request / response

```text
ToolRequest  = { name: ToolName, arguments: JSON object }
ToolResponse = { ok: bool, content: string, mutated: bool }
```

| Field | Meaning |
|-------|---------|
| `ok` | Tool completed without executor hard-error; semantic failures may still be `ok: false` with JSON error body (e.g. edit no-match) |
| `content` | UTF-8 JSON or structured text returned to the model as the tool result |
| `mutated` | `true` if filesystem may have changed (snippet invalidation already applied as required) |

Hard errors (permission deny/ask in headless, unknown tool, missing required args) may surface as executor `Err` and become model-visible error tool results via the agent loop.

### 3.2 Permission before mutation

Every tool path that touches the tree runs **permission decision** first (spec 90). Deny/ask must not leave partial writes.

### 3.3 Registry function

`tool_definitions()` (or equivalent) returns the ordered list of built-in `ToolDefinition`s used by the agent loop and prefix builder.

**Order stability:** Definition order is part of the stable prefix inputs. Implementations **must not** shuffle tool order across builds without a documented epoch. Recommended fixed order:

```text
read, edit, write, grep, skill, bash
```

(or document and test the chosen order).

### 3.4 Schema stability (cache)

Per spec 10:

- Tool schemas live in the **stable prefix**.  
- Canonical JSON for schemas must be **byte-stable** for identical registry code.  
- Changing any tool name, required field, or property shape starts a **new cache epoch** (document in release notes when shipping).

---

## 4. Non-goals (this spec)

- Parallel independent tool execution (spec **50**, G4).  
- Background shell collect (Wave C).  
- Subagent spawn tools (spec **60**, G5).  
- MCP tool mounting (spec **80**, G6c).  
- Interactive TTY permission prompts (product UX; still fail-closed headless per 90).  
- Full regex / structural search engines (literal `grep` is the minimum).  

---

## 5. Test plan (automated)

| ID | Case | Expect |
|----|------|--------|
| T1 | Registry name set | Exactly the six canonical names; no extras |
| T2 | Alias parse | `search`→grep, `load_skill`→skill; unknown → None |
| T3 | Schema required fields | Each tool’s `required` matches §2 |
| T4 | `edit` without `snippet_id` | Args error |
| T5 | `grep` empty pattern | Args error |
| T6 | `grep` finds literal matches | Match path/line stable under fixture |
| T7 | `write` out-of-cwd under write-in-cwd allow | Permission error; no file created |
| T8 | Denied bash | No snippet expiry |
| T9 | Allowed mutating bash | Snippets expired (or path-touched) |
| T10 | `skill` load | Body returned; `mutated` false |
| T11 | Schema JSON stability | Golden or hash of canonical tool schema document |

T1–T10 already covered or extended under `dsb-tools` / agent tests for Wave A. T11 may be added when aligning code to this spec (Wave B unit B).

---

## 6. Implementation map

| Area | Location (current) |
|------|--------------------|
| Dispatch + schemas | `crates/dsb-tools/src/tools.rs` |
| Snippets | `crates/dsb-tools/src/snippets.rs` |
| Permissions | `crates/dsb-tools/src/permissions.rs` |
| Agent wires `tool_definitions()` | `crates/dsb-agent/src/loop_.rs` |
| Prefix tools document | `crates/dsb-context` |

---

## 7. Ready-for-impl checklist

- [x] Catalog fixed and small  
- [x] Wire args named and required sets listed  
- [x] Edit path requires snippet_id (no free-form primary)  
- [x] write is create-only  
- [x] bash declare + classifier authority referenced  
- [x] skill on-demand, prefix index untouched  
- [x] Cache stability rules stated  
- [x] Automated test plan with negative cases  
- [x] Explicit non-goals for G4/G5/MCP  

**Status:** **ready-for-impl**.
