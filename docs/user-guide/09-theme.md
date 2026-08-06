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

## Spec / design SSOT

- `docs/product/DESIGN.md`
- Implementation: `crates/dsb-cli/src/theme.rs`, `crates/dsb-cli/src/banner.rs`
