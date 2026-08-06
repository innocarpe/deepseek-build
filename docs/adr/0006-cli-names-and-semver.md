# ADR 0006 — CLI command names and SemVer product identity

- **Status:** Accepted  
- **Date:** 2026-08-06  
- **Supersedes (partial):** [ADR 0004](./0004-toolchain.md) § Decision item 2 (binary name only)  
- **Related:** [versioning.md](../contributing/versioning.md)

## Context

1. **SemVer discipline:** Casual “1.0” / “v1” language confuses tags, npm, and cargo. The product needs a fail-close rule: always `MAJOR.MINOR.PATCH`.  
2. **CLI naming:** Peer tools install as memorable single tokens (`claude`, `codex`, `grok`). `dsb` is short but opaque; `deepseek-build` is clear but long. Users may want both.

ADR 0004 pinned binary name to `dsb` only; that was enough for M1 scaffolding but is weak for public install identity.

## Decision

### A. Semantic Versioning (product-wide)

1. The product version is always **SemVer 2.0.0** full form: `MAJOR.MINOR.PATCH` (optional `-pre` / `+build`).  
2. **Forbidden** as a version id in docs, PRs, ultragoal, tags-without-patch, cargo, or npm: bare `1.0`, `0.2`, `v1` (except git tag **prefix** `v` on a full triple: `v1.0.0`).  
3. Workspace `[workspace.package] version` is the Rust source of truth until npm exists; then npm **must match** the release SemVer.  
4. Normative process doc: [`docs/contributing/versioning.md`](../contributing/versioning.md).

### B. Dual CLI commands

| Command | Status |
|---------|--------|
| **`deepseek-build`** | **Primary** public invocation name |
| **`dsb`** | Supported **alias** (same program, same flags, same version) |

1. Cargo package `dsb-cli` builds **two** binaries from the same entrypoint: `deepseek-build` and `dsb`.  
2. Crate/library prefixes remain `dsb-*` (short, stable in Rust).  
3. Config dir stays `~/.deepseek-build/` (product name, not command name).  
4. npm `bin` (when added) **must** register both names.  
5. Docs and README lead with `deepseek-build`; mention `dsb` as alias.

### C. Version string shape

`--version` output should identify the product and full SemVer, e.g.:

```text
deepseek-build 0.1.0
```

or for the alias binary:

```text
dsb 0.1.0
```

Both report the **same** `MAJOR.MINOR.PATCH`.

## Consequences

- ADR 0004 item “Binary / CLI name: dsb only” is **replaced** by this dual-name policy.  
- Install docs and future packaging must not ship only one of the two names without a follow-up ADR.  
- Agents must not plan releases as “1.0”; use **1.0.0**.

## References

- SemVer: https://semver.org/  
- Peer CLIs: `claude`, `codex`, `grok` (invocation tokens, not copy of their stacks)  
