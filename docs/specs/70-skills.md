# Spec 70 — Skills as structured context

| Field | Value |
|-------|--------|
| Status | **ready-for-impl** (product; G6b green since **0.6.0**, expanded **0.10.0**) |
| Philosophy | HARNESS §4.3 Pillar C |
| Gate | **G6b** |
| Tests | Automated: index determinism, body not in prefix, opt-out, path safety |

## 1. Behavior

### 1.1 Discovery roots (deterministic order)

| Order | Path | Notes |
|-------|------|--------|
| 1 | `{workspace}/skills/*/SKILL.md` | Project skills |
| 2 | `{workspace}/.deepseek-build/skills/*/SKILL.md` | Project-local |
| 3 | `~/.deepseek-build/skills/*/SKILL.md` | User skills (when dir exists) |

**Same name:** later roots **override** earlier for both index entry and body load (user can override project).

Directory name is the skill **name** (not frontmatter `name:`). Names with `/`, `\`, or `..` are rejected on load.

### 1.2 Index (stable prefix)

Each skill contributes:

```text
{ "name": "<dir>", "description": "<≤200 chars>" }
```

- Description from YAML frontmatter `description:` when present; else first non-heading prose line; else `"(no description)"`.  
- Index is **sorted by name** (BTree / sort).  
- Index may sit in the **stable prefix** (spec 10).  
- Changing the index content → new cache **epoch**.

### 1.3 Bodies (volatile / on-demand)

- Full `SKILL.md` bodies load only via tool **`skill`** `{ "name": "…" }` (spec 40).  
- Body must **not** appear in the stable prefix builder output.  
- Mid-session body load does **not** rewrite the stable index.

### 1.4 Opt-out of model discovery

Frontmatter flags (any of these truthy) **exclude** the skill from the index:

```yaml
---
description: Internal helper
disable-model-invocation: true
# or
disable_model_invocation: true
---
```

Body may still be loaded if the user/agent knows the name (optional future: block load too — product v1 only hides from index).

### 1.5 CLI product surface (**0.10.0+**)

| Command | Behavior |
|---------|----------|
| `deepseek-build skills list` / `dsb skills list` | Print discovered index (name + description), sorted |
| (agent) `skill` tool | On-demand body load |

## 2. Non-goals

- Full marketplace / network skill install  
- Skill versioning registry  
- Automatic skill invocation without model tool call  

## 3. Test plan

| Test | Expect |
|------|--------|
| `index_sorted_stable` | same inputs → same index order/bytes |
| `body_not_in_stable_prefix` | body text absent from prefix builder output |
| `opt_out_excluded_from_index` | disable-model-invocation skills omitted |
| `path_traversal_rejected` | `../` names fail load |
| `user_overrides_project` | same name: user root wins body |

## 4. Implementation map

| Area | Location |
|------|----------|
| Discover / load | `crates/dsb-context/src/skills.rs` |
| Prefix document | `crates/dsb-context/src/prefix.rs` |
| Tool | `crates/dsb-tools` `skill` |
| CLI list | `crates/dsb-cli` `skills list` |

## 5. Ready-for-impl checklist

- [x] Index name+description only  
- [x] Bodies on demand  
- [x] Deterministic roots + sort  
- [x] Path safety  
- [x] Opt-out frontmatter  
- [x] CLI list  
- [x] Automated tests  

**Status:** **ready-for-impl** (product expansion complete for Wave B).
