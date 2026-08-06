# crates/

Cargo workspace members for **DeepSeek Build** (`dsb`). Layout follows
[`docs/adr/0004-toolchain.md`](../docs/adr/0004-toolchain.md).

| Crate | Role | Status |
|-------|------|--------|
| `dsb-cli` | Binary entry (`dsb`) | M1 scaffold |
| `dsb-config` | Config + credentials load | M1 |
| `dsb-provider-deepseek` | DeepSeek Chat Completions client | M1 |
| `dsb-agent` | Turn / agent loop, repair, routing | M1+ |
| `dsb-context` | Stable prefix / cache epochs (spec 10) | M1 |
| `dsb-tools` | Snippets (45) + permissions (90) + read/edit/write/bash | M2 |

Build / run from repo root:

```bash
cargo build -p dsb-cli
cargo run -p dsb-cli -- --version
cargo test
```
