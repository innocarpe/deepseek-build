# Design — DeepSeek Build terminal theme

**Status:** Normative product design (theme v1 + banner v2)  
**SemVer:** theme tokens from **0.9.0**; whale welcome banner from **1.2.0**  
**Audience:** agents + humans styling CLI output

## 1. Goals

1. **Readable by default** for long coding sessions (contrast + hierarchy).  
2. Brand accent in the **DeepSeek blue** family (not Grok near-black monochrome).  
3. Distinguish **roles**: content, reasoning, tool, model, error.  
4. Respect `NO_COLOR` and non-TTY (plain text, no ANSI).  
5. **Welcome chrome** that reads as DeepSeek (whale mark + brand blue), at peer-CLI quality without a full TUI.

## 2. Tokens (theme v1)

Official brand accent matches DeepSeek product chrome (`#4D6BFE` / RGB 77,107,254 — same family as deepseek.com wordmark fill and public brand listings).

| Token | RGB | Hex | Role |
|-------|-----|-----|------|
| `deepseek.blue` | `77, 107, 254` | `#4D6BFE` | Accent, tool lines, whale mark, box chrome, REPL prompt |
| `model.blue` | `99, 140, 255` | `#638CFF` | Model / epoch / meta lines inside the card |
| `reasoning.slate` | `148, 163, 184` | `#94A3B8` | Reasoning deltas (secondary) |
| `content` | *terminal default* | — | Assistant content (unstyled for max readability) |
| `error` | `248, 113, 113` | `#F87171` | Tool / hard errors |
| `warn` | `251, 191, 36` | `#FBBF24` | Warnings |

Implementation:

- `crates/dsb-cli/src/theme.rs` — `Theme`, `Role`, truecolor ANSI `38;2;r;g;b`
- `crates/dsb-cli/src/banner.rs` — whale mark + welcome card

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
| Whale mark / card border / product title / `❯` prompt | Accent |
| Card meta (cwd, profile, tips) | Model |

## 5. Welcome banner (v2)

Chat / bare interactive start prints a **boxed card**:

```text
╭──────────────────────────────────────────────╮
│      ⣠⣾⣿⣿⣷⣄                              │
│    ⣰⣿⠋ ⠈⠙⣿⣆   DeepSeek Build  vX.Y.Z    │
│   ⢸⣿⣇⣀  ⣀⣸⣿  DeepSeek-native coding agent│
│   ⠈⣿⣿⣿⣿⣿⣿⡿⠁  cmd / cwd / profile / epoch │
│      ⠈⠉⠁       /help · /pro · /flash · /quit│
╰──────────────────────────────────────────────╯
❯
```

Rules:

1. **Whale mark** — braille raster of the official DeepSeek whale silhouette (body curve, belly cutout, eye/smile, fluke, fin) at CLI scale; not an embedded PNG/SVG binary.  
2. **Brand blue only** for the mark, box edges, title, and prompt — no second “hero” color.  
3. **Narrow terminals** (`COLUMNS` < 64) use `WHALE_MARK_COMPACT`.  
4. **`NO_COLOR=1`** keeps the box + mark structure, strips ANSI.  
5. Still **line-oriented** — no ratatui frame, no forced background fill (see non-goals).

## 6. Permission prompt chrome

Permission asks use stderr labels `[permission]` (readable, high priority). Color may match `Warn` / `Accent` in later polish; v1 prioritizes clear text choices:

```text
[permission] scopes need approval: write-in-cwd
  [a] allow once   [A] allow always   [d] deny
```

## 7. Evidence checklist (PR / release)

- [ ] TTY chat banner shows whale mark + DeepSeek blue box  
- [ ] Product title uses `#4D6BFE` truecolor when color is on  
- [ ] Tool lines distinguishable from content  
- [ ] `NO_COLOR=1` produces no ANSI escapes (structure remains)  
- [ ] Non-TTY `run` remains plain (no forced banner)  
- [ ] REPL prompt `❯` uses Accent when color is on  

## 8. Non-goals (v1 / banner v2)

- Full TUI framework  
- Custom background painting / alternate screen  
- Theme file format / user CSS  
- Shipping the official SVG/PNG logo binary inside the crate **for the banner**  
  (scoped change: the official PNG logo is now shipped under `npm/assets/` for the
  terminal **tab icon** — see `scripts/install-iterm-tab-icon.sh`; the in-terminal
  banner stays line-art/braille only)
- Windows console legacy code-page workarounds beyond ANSI when supported  

## 9. Related

- [MASTER_PLAN.md](./MASTER_PLAN.md) §5 Design track  
- [prd/PRD-wave-B-native.md](./prd/PRD-wave-B-native.md) design acceptance  
- Specs 40 / 90 (behavior; not colors)  
- User guide: [09-theme.md](../user-guide/09-theme.md)
