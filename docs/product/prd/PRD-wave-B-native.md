# PRD — Wave B: DeepSeek-native surface

| Field | Value |
|-------|--------|
| SemVer band | **`0.8.0` – `0.11.0`** |
| Plan id | `native-0x` |
| Status | Planned (starts when Wave A complete) |
| Depends on | Wave A dogfood-usable; G3 already green |

## Problem

Dogfood core is not yet “Deep Code–class”: permissions UX is headless-heavy, skills/MCP/plan thin or missing, and the terminal look may still feel harsh (Grok-black syndrome).

## Goal

Daily work feels **DeepSeek-native**: safe permissions with interactive ask, skills, thinking/effort UX, light plan, MCP without cache thrash, and a **readable DeepSeek blue default theme**.

## Non-goals

- Subagents / parallel tool fan-out (Wave C)  
- `1.0.0`  
- Multi-vendor identity  

## User stories

1. I get prompted for dangerous scopes and can allow-once / allow-always.  
2. Skills discover and load without dumping all bodies into the prefix.  
3. Thinking is collapsible or clearly separated; effort is user-settable.  
4. MCP tools mount with explicit cache epoch on schema change.  
5. Default colors use DeepSeek blue accents and readable contrast.  

## Exit criteria

- [ ] Specs **40**, **70**, **80**, **110** (light) ready-for-impl where required by features shipped  
- [ ] Theme v1 default = DeepSeek blue / high readability (not Grok near-black)  
- [ ] Interactive permissions path works on TTY  
- [ ] Skills index in stable prefix; bodies on demand  
- [ ] MCP documented + epoch rules enforced  
- [ ] Ultragoal `native-0x` complete; SemVer in **`0.8.0`–`0.11.0`** band  

## Suggested minors

| SemVer | Theme |
|--------|--------|
| `0.8.0` | Spec 40 + tool surface polish |
| `0.9.0` | Permissions UX + **theme v1** |
| `0.10.0` | Skills product |
| `0.11.0` | MCP + plan light |

## Design acceptance (theme)

| Check | Pass |
|-------|------|
| Default background/text contrast sufficient for long sessions | yes |
| Accent identifiable as DeepSeek blue family | yes |
| content vs reasoning vs tool lines distinguishable | yes |
| Optional dark theme does not become the only theme | yes |
