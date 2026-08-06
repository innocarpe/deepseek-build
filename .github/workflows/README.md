# GitHub Actions

## Product CI (Wave D `0.15.0`+)

| Workflow | File | What it verifies |
|----------|------|------------------|
| **ci** | [`ci.yml`](./ci.yml) | `cargo fmt`, `clippy -D warnings`, `cargo test --workspace`, `./scripts/smoke-dogfood.sh` (offline) |

This is **product** CI: build, tests, dual-CLI smoke. It is **not** process-police (no PR title regex, no kind-label counters, no markdown path inventories).

### Local equivalent

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
./scripts/smoke-dogfood.sh
```

### When to extend

| Goal | Plausible job |
|------|----------------|
| Release artifacts | Upload dual bins on tag |
| npm package layout | `npm run version-check` only (publish stays human) |
| Live API | Optional secret job; never block PR green on missing key |

Process quality (PR narrative, kind labels, Orca-level body) remains:

- `docs/contributing/*`
- root `AGENTS.md` + `skills/pr-authoring/`
- Review / self-merge checklist
