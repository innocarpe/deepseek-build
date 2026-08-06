# crates/

Cargo workspace members for **DeepSeek Build** (`dsb`). Layout follows
[`docs/adr/0004-toolchain.md`](../docs/adr/0004-toolchain.md).

| Crate | Role | Status |
|-------|------|--------|
| `dsb-cli` | Binary entry (`dsb`) | M1 scaffold |
| `dsb-config` | Config + credentials load | M1 |
| `dsb-provider-deepseek` | DeepSeek Chat Completions client | M1 |
| `dsb-agent` | Turn / agent loop | planned |
| `dsb-context` | Stable prefix / cache epochs (spec 10) | planned |
| `dsb-tools` | Tool runtime | M2+ |

Build / run from repo root:

```bash
cargo build -p dsb-cli
cargo run -p dsb-cli -- --version
cargo test
```
