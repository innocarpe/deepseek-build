# 09 — Theme (DeepSeek blue)

**Product version:** `0.9.0`+ (tokens) · **banner v2:** `1.2.0`+

Default terminal styling optimizes **readability**, with brand accent **DeepSeek blue** (`#4D6BFE` / RGB 77,107,254) — the same accent used on DeepSeek’s product chrome.

| Role | Color use |
|------|-----------|
| Content | Unstyled (max readability on light/dark terminals) |
| Tool / accent / whale / box / prompt | DeepSeek blue |
| Model / epoch / card meta | Lighter blue |
| Reasoning | Slate secondary |
| Error | Soft red |
| Warn | Amber |

## Welcome banner

Interactive chat opens with a **whale mark + product card** (DeepSeek-blue borders). The mark is a braille raster of the official DeepSeek whale silhouette — no image assets required.

```bash
deepseek-build          # or: dsb / deepseek-build chat
```

## Disable color

```bash
NO_COLOR=1 deepseek-build chat
```

Non-TTY output is plain by default. With `NO_COLOR`, the box and whale still print; ANSI is omitted.

## Theme choice

The product default theme is **DeepSeek Night v2** (measured C-balanced DeepSeek
palette). The original look is still available as **DeepSeek Night (classic)** — the
blue-tinted DeepSeek palette that shipped before v2.

- Switch any time inside the TUI with `/theme` or the Settings theme sheet.
- On first launch, the picker offers both (`1` = classic, `2` = v2 default).

Both themes are DeepSeek blue (`#4D6BFE`) based; v2 tunes the palette ramp for
legibility, classic keeps the original tinted signature.

## Terminal tab (iTerm2)

Running `deepseek-build` / `dsb` sets the terminal tab title to **DeepSeek Build** (OSC 0),
and on iTerm2 (macOS 15+ Tahoe tab style) the tab shows the **official DeepSeek whale
logo** — no border or background.

The tab logo is an iTerm2 per-process icon mapping. It is **installed automatically** on
the first TUI launch (the logo is embedded in the binary), so a fresh `npm i -g` works
with no extra step. For verification or manual control, the repo script covers the rest:

```bash
./scripts/install-iterm-tab-icon.sh        # install (idempotent)
./scripts/install-iterm-tab-icon.sh check  # verify state
./scripts/install-iterm-tab-icon.sh remove # uninstall
```

This writes `graphic_deepseek.png` plus merged `graphic_icons.json` /
`graphic_colors.json` under `~/Library/Application Support/iTerm2/`, mapping the
`deepseek-build-agent` process name to the official logo in DeepSeek blue (`#4D6BFE`).
The title works in any terminal; the logo requires iTerm2.

## Spec / design SSOT

- `docs/product/DESIGN.md`
- Implementation: `crates/dsb-cli/src/theme.rs`, `crates/dsb-cli/src/banner.rs`,
  `crates/dsb-cli/src/terminal_tab_icon.rs`, `scripts/install-iterm-tab-icon.sh`
