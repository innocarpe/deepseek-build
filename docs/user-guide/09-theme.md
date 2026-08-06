# 09 — Theme (DeepSeek blue)

**Product version:** `0.9.0`+

Default terminal styling optimizes **readability**, with brand accent **DeepSeek blue** (`#4D6BFE` / RGB 77,107,254).

| Role | Color use |
|------|-----------|
| Content | Unstyled (max readability on light/dark terminals) |
| Tool / accent | DeepSeek blue |
| Model / epoch | Lighter blue |
| Reasoning | Slate secondary |
| Error | Soft red |
| Warn | Amber |

## Disable color

```bash
NO_COLOR=1 deepseek-build chat
```

Non-TTY output is plain by default.

## Spec / design SSOT

- `docs/product/DESIGN.md`
- Implementation: `crates/dsb-cli/src/theme.rs`
