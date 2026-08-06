# Design — DeepSeek Build terminal theme

**Status:** Normative product design (theme v1)  
**SemVer:** ships with **0.9.0**  
**Audience:** agents + humans styling CLI output

## 1. Goals

1. **Readable by default** for long coding sessions (contrast + hierarchy).  
2. Brand accent in the **DeepSeek blue** family (not Grok near-black monochrome).  
3. Distinguish **roles**: content, reasoning, tool, model meta, error.  
4. Respect `NO_COLOR` and non-TTY (plain text, no ANSI).

## 2. Tokens (theme v1)

| Token | RGB | Role |
|-------|-----|------|
| `deepseek.blue` | `77, 107, 254` (`#4D6BFE`) | Accent, tool lines |
| `model.blue` | `99, 140, 255` | Model / epoch meta lines |
| `reasoning.slate` | `148, 163, 184` | Reasoning deltas (secondary) |
| `content` | *terminal default* | Assistant content (unstyled for max readability) |
| `error` | `248, 113, 113` | Tool / hard errors |
| `warn` | `251, 191, 36` | Warnings |

Implementation: `crates/dsb-cli/src/theme.rs` (`Theme`, `Role`, truecolor ANSI `38;2;r;g;b`).

## 3. Default vs optional dark

| Mode | Behavior |
|------|----------|
| **Default** | Color on TTY when `NO_COLOR` unset; content unstyled; accent DeepSeek blue |
| **Plain** | `NO_COLOR=1` or non-TTY → identity paint |
| **Optional dark terminal** | User terminal theme; we do **not** force a near-black app chrome |

Default is **not** “Grok black glass”. Agents must not reintroduce monochrome-only styling as the product default.

## 4. Role mapping (CLI)

| Event | Role |
|-------|------|
| Assistant content deltas | Content (plain) |
| Reasoning deltas (`--show-reasoning`) | Reasoning |
| `[tool] …` | Tool (DeepSeek blue) |
| `[model=…]` / prefix epoch / session | Model |
| `[tool-error]` | Error |
| `[warn]` | Warn |
| Chat banner | Accent |

## 5. Permission prompt chrome

Permission asks use stderr labels `[permission]` (readable, high priority). Color may match `Warn` / `Accent` in later polish; v1 prioritizes clear text choices:

```text
[permission] scopes need approval: write-in-cwd
  [a] allow once   [A] allow always   [d] deny
```

## 6. Evidence checklist (PR / release)

- [ ] TTY chat banner uses DeepSeek blue accent  
- [ ] Tool lines distinguishable from content  
- [ ] `NO_COLOR=1` produces no ANSI escapes  
- [ ] Non-TTY `run` remains plain  

## 7. Non-goals (v1)

- Full TUI framework  
- Custom background painting  
- Theme file format / user CSS  
- Windows console legacy code-page workarounds beyond ANSI when supported  

## 8. Related

- [MASTER_PLAN.md](./MASTER_PLAN.md) §5 Design track  
- [PRD-wave-B-native.md](./prd/PRD-wave-B-native.md) design acceptance  
- Specs 40 / 90 (behavior; not colors)
