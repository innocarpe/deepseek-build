# W2 edit/tool loop dogfood (G008)

**Date:** 2026-08-06  
**Band:** 2.0.0

## Command

```bash
dsb --cwd <tmp> --dogfood --bash-execute run \
  "Read hello.rs and use the edit tool to change the string hi to hello-deepseek."
```

## Result

| Check | Evidence |
|-------|----------|
| Model | `deepseek-v4-flash` |
| Tools | `read` then `edit` (snippet-safe) |
| File after | `fn main() { println!("hello-deepseek"); }` |

### Transcript (abbrev)

```text
[model=deepseek-v4-flash thinking=on effort=high]
[prefix_epoch=a466e624fa7fb6b4]
[tool] read
[tool] edit
Done. Changed `hi` to `hello-deepseek` in `hello.rs`:

```rust
fn main() { println!("hello-deepseek"); }
```
[model_used=deepseek-v4-flash]

```

## Grok full-screen path

No-args `dsb` opens Grok pager tools (`SearchReplace` / hashline / bash). Same product DeepSeek defaults via `~/.deepseek-build/config.toml` seed.

## W2 exit

Owner can `dsb` → chat/run → real code changes with DeepSeek. **PASS**.
