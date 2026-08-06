# Spec 90 — Side-effect permissions

| Field | Value |
|-------|--------|
| Status | **ready-for-impl (minimum)** — G3 / M2 shell gate; full surface extends in M3 |
| Philosophy | HARNESS §4.4 Pillar D (Deep Code); NON_GOALS: no YOLO-only default |
| Gate | **G3 minimum** required before shell mutates the tree; full polish → M3 |
| Tests | **Automated golden + negative required** (for minimum) |

## 0. Minimum vs full

| Tier | Required for | Includes |
|------|----------------|----------|
| **Minimum (this gate)** | **G3**, M2 `bash` + file mutate tools | Path scopes in/out workspace; decision allow/deny/ask; bash **declared** effects + **authoritative classifier**; fail-closed on mismatch; audit record; defaults that are not YOLO-only |
| **Full (M3)** | M3 exit | Persistent “always allow” UX, richer network/MCP profiles, settings UX, per-project profiles polish |

Implementations may ship full early, but **must not** ship mutating shell without the minimum table in §1–§4.

---

## 1. Behavior (minimum)

Before any tool runs, the product computes a set of **scopes** and a **decision**:

```text
decision ∈ { allow, deny, ask }
```

- **allow** — execute  
- **deny** — do not execute; structured error to model  
- **ask** — prompt user (TTY) or fail closed in headless unless pre-approved policy says allow  

### 1.1 Scope catalog (minimum set)

Aligned with Deep Code’s taxonomy (names may use these wire strings):

| Scope | Meaning |
|-------|---------|
| `read-in-cwd` | Read under workspace root |
| `read-out-cwd` | Read outside workspace root |
| `write-in-cwd` | Create/overwrite under workspace |
| `write-out-cwd` | Create/overwrite outside workspace |
| `delete-in-cwd` | Delete under workspace |
| `delete-out-cwd` | Delete outside workspace |
| `query-git` | Read-only git (`status`, `log`, `diff`, `show`, …) |
| `mutate-git` | Mutating git (`commit`, `rebase`, `push`, `reset --hard`, …) |
| `network` | Network egress (curl, package install, …) |
| `unknown` | Classifier cannot classify — **always ask** (or deny in non-interactive strict mode) |

M3 may add `mcp` and finer scopes; minimum must still treat unknown MCP as `ask`/`deny` when MCP exists.

### 1.2 Path classification (file tools)

For `read` / `write` / `edit` / delete-like tools:

1. Resolve path relative to workspace root (symlink policy: no escape without out-of-cwd scope).  
2. Map to `read-*` / `write-*` / `delete-*` in or out of cwd.  
3. `edit` and overwriting `write` always include the corresponding **write** scope (and delete if truncate/replace whole file — still `write-in-cwd`).

### 1.3 Bash: declare + classify (authoritative)

`bash` (or equivalent shell tool) arguments **must** include:

| Field | Role |
|-------|------|
| `command` | Shell command string |
| `side_effects` | Model-declared list of scopes (advisory) |

**Authoritative path:**

1. Run **static/command classifier** on `command` → `classified_scopes[]` (no execution yet).  
2. Union with declared scopes for **audit**, but decision uses:

```text
effective_scopes = classified_scopes
if declared contains higher-risk scopes not in classified:
    still use classified for allow, but audit "declare_over_class"
if classified is empty → effective_scopes = [unknown]
if mismatch where declared is *lower* risk than classified:
    effective_scopes = classified   // fail-closed to more dangerous
```

**Fail-closed rule (normative):**  
If declared scopes and classified scopes disagree, use the **more dangerous** interpretation (higher of ask/deny), never the looser one.

Examples of dangerous > safe: `mutate-git` > `query-git`; `write-out-cwd` > `write-in-cwd`; `network` > none; `unknown` forces ask/deny.

### 1.4 Policy config (minimum)

User/project config (paths product-specific; e.g. `~/.deepseek-build/config.toml` or settings JSON):

```toml
[permissions]
# default for scopes not listed:
default = "ask"   # product default for coding agent: NOT allow-all
allow = ["read-in-cwd", "query-git"]
ask = ["write-in-cwd", "delete-in-cwd", "network", "mutate-git"]
deny = ["write-out-cwd", "delete-out-cwd"]
```

**Priority when a call has multiple scopes:**

1. Any scope in `deny` → **deny**  
2. Else any scope in `ask` → **ask**  
3. Else all scopes in `allow` → **allow**  
4. Else → `default`

**Product default (M2 shipping profile):**

| Scope | Default |
|-------|---------|
| `read-in-cwd` | allow |
| `query-git` | allow |
| `write-in-cwd` / `delete-in-cwd` | **ask** |
| `read-out-cwd` | ask |
| `write-out-cwd` / `delete-out-cwd` | **deny** |
| `mutate-git` | ask |
| `network` | ask |
| `unknown` | ask (headless: deny unless `--yes` / explicit allow policy) |

**Anti-pattern:** shipping with `default = allow` for all scopes as the **only** mode (YOLO-only). A power-user “yolo” profile may exist only as **opt-in**.

### 1.5 Headless / CI

| Mode | ask behavior |
|------|----------------|
| Interactive TTY | Prompt user |
| Headless without approval store | **deny** on ask (fail closed) unless env/flag grants a scoped allow list |
| Pre-approved session grants | Allowed if recorded in audit |

### 1.6 Audit record (minimum)

Every gated call logs (debug or session audit file — not secrets):

```text
tool, command_or_path, declared_scopes, classified_scopes, decision, reason
```

Required for bash; recommended for file tools.

### 1.7 Interaction with snippets (spec 45)

After bash with any of `write-*`, `delete-*`, `mutate-git`, or `unknown` that may touch files:

1. Mark session file dirty.  
2. Expire or revalidate snippets per spec 45 §1.6.  

Permission **deny** must not mutate disk or expire snippets (no side effects).

### 1.8 Classifier requirements (minimum)

M2 classifier need not be perfect ML; a **deterministic rule engine** is enough:

- Tokenize command; detect redirects (`>`, `>>`), `rm`, `mv`, `cp`, `git commit|push|rebase|reset`, network clients (`curl`, `wget`, `npm i`, `cargo install`, …).  
- Heuristic path extraction for expiry (optional).  
- Unrecognized → `unknown`.  

Golden tests pin rule outcomes for a fixture table of commands.

## 2. Non-goals (minimum)

- Perfect semantic understanding of all shell scripts  
- OS sandbox / seccomp (nice later; not G3)  
- Full MCP matrix (spec 80)  
- Replacing user OS permissions  

## 3. Failure modes

| Case | Behavior |
|------|----------|
| Classifier says write, model declared only read | Use write (dangerous); ask/deny per policy |
| Missing `side_effects` on bash | Treat as empty declare; classifier still runs; prefer `unknown` if classify empty |
| User denies ask | Tool error `permission_denied`; turn continues |
| Path escapes workspace via `..` / symlink | out-of-cwd scope or deny |

## 4. Test plan (automated — minimum)

| Test | Expect |
|------|--------|
| `path_read_in_cwd_allow` | workspace read → allow under default profile |
| `path_write_out_cwd_deny` | outside write → deny under default profile |
| `bash_rm_classified_write_or_delete` | `rm file` → delete/write-in-cwd in classified set |
| `bash_declare_lower_than_class_fail_closed` | declare read-only but `rm` → dangerous class wins |
| `bash_unknown_asks_or_denies` | nonsense binary → unknown → not silent allow |
| `policy_deny_beats_allow` | scope in both → deny |
| `headless_ask_is_deny` | ask decision without TTY → deny |
| `denied_bash_no_snippet_expiry` | deny path does not clear snippet table |
| `allowed_mutating_bash_flags_dirty` | allow write bash → snippet invalidation hook called |

## 5. Full (M3) extensions (not required to flip G3)

- Interactive “always allow” persistence into project config  
- `mcp` scope + per-server rules  
- Network allowlists  
- Rich `/permissions` UX  

These must not weaken minimum fail-closed defaults.

## 6. Implementation notes

- Crate: `dsb-tools` permission module or `dsb-permissions`.  
- Config keys under user home (ADR 0004) + optional project `.deepseek-build/`.  
- Spec **40** wires tool schemas including bash `side_effects`.  
- Spec **45** consumes dirty notifications.  
