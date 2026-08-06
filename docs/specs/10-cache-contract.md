# Spec 10 — Cache contract (stable prefix)

| Field | Value |
|-------|--------|
| Status | **ready-for-impl** |
| Philosophy | HARNESS §4.2, §5; Deep Code pillar B; Reasonix cache-first |
| Gate | Part of **G2** |
| Tests | **Automated golden + negative required** |

## 1. Behavior

The product builds each model request as:

```text
messages_to_api =
  stable_prefix_messages   // byte-stable across turns when inputs unchanged
  + volatile_tail_messages // user, tool results, dynamic reminders
```

### 1.1 Stable prefix contents (ordered)

1. System prompt body (product template; no wall-clock, no random IDs).  
2. Tool schemas document (canonical JSON; **sorted object keys** recursively).  
3. Skills **index only** (name + one-line description; deterministic sort by name).  
4. Small environment summary (OS family, cwd **as normalized path**, not hostname unless user opts in).  
5. Standing project instructions (discovered files; deterministic order — see §1.4).  

### 1.2 Volatile tail

- Current user turn  
- Assistant/tool messages for the active turn chain  
- Dynamic reminders, large tool outputs (may be snipped)  
- Anything containing timestamps used for UX only  

### 1.3 Byte stability

Define `stable_prefix_bytes = UTF-8 encoding of canonicalize(stable_prefix_messages)`.

**Canonicalize rules (normative):**

- JSON: keys sorted lexicographically; no insignificant whitespace beyond single separators as implemented in a single `serde_json::to_vec` / equivalent **documented** function.  
- Newlines: `\n` only inside stored strings.  
- Paths: prefer project-relative when under workspace root; else absolute normalized (no trailing slash except root).  
- No `SystemTime::now`, random UUIDs, or process id in stable sections.

**Invariant:** For identical inputs to the prefix builder, two consecutive builds produce **equal** `stable_prefix_bytes`.

### 1.4 Project instruction discovery (minimal for M1)

Load if present, in order, concatenate with clear separators:

1. `./DEEPSEEK.md` or `./DEEPSEEK_BUILD.md` (first found)  
2. `./AGENTS.md`  
3. `./.deepseek-build/instructions.md` if present  

Missing files skip. Content changes → **new cache epoch** (prefix hash changes; expected).

### 1.5 Epochs

An **epoch** is the hash of `stable_prefix_bytes`. Log `prefix_epoch=sha256_hex` per request (or first 16 hex chars). Mid-session tool schema change **must** bump epoch (new tools list → new prefix).

### 1.6 Session replay

Persist turns as JSONL (or equivalent) under user state dir. On load, **repair tool pairs** (spec 15) before send.

### 1.7 Compaction (M1 stub)

M1 may omit full compaction. If context overflows, fail with clear error **or** drop oldest **volatile** tail only — **never** mutate stable prefix in place without epoch bump. Full compaction policy → later ADR/spec 10 extension.

### 1.8 Cache evidence (with ADR 0005)

M1 acceptance:

1. Golden: `stable_prefix_bytes` equality test passes.  
2. Provider: parse cache hit/miss from usage when present; else dual-call substitute protocol logged.

## 2. Non-goals

- Guaranteeing 100% provider cache hits (server policy)  
- Stuffing full skill bodies into stable prefix  
- User-managed cache keys  

## 3. Failure modes

| Case | Behavior |
|------|----------|
| Non-deterministic field in system template | **Bug**; tests must catch |
| Tool schema key order shuffle | **Bug** |
| Compaction rewrites system tools mid-epoch without bump | **Bug** |

## 4. Test plan (automated)

| Test | Expect |
|------|--------|
| `prefix_stable_across_two_builds` | bytes equal |
| `prefix_changes_when_tool_added` | bytes differ; epoch differs |
| `prefix_no_timestamp` | fixture system template without clock |
| `sorted_tool_schema_keys` | permuting input map → same bytes |
| Negative: inject `Utc::now` into builder path | test fails if someone reintroduces |

## 5. Implementation notes

- Crate: `dsb-context` (ADR 0004).  
- Hash: SHA-256 of raw bytes for epoch logging.  
