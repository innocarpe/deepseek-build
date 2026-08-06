# ADR 0007 — npm packaging for DeepSeek Build

- **Status:** Accepted  
- **Date:** 2026-08-06  
- **Gate:** Wave A `0.7.0` / packaging  
- **Supersedes:** open item “npm binary download strategy” in SYSTEM_ARCHITECTURE (decided here)

## Context

Overnight agents and humans need a **non-inventable** npm path: dual bins (`deepseek-build`, `dsb`), SemVer match with Cargo, and a clear split between **package correctness** (agent DoD) and **registry publish** (human).

## Decision

### Package identity

| Field | Value |
|-------|--------|
| npm name | **`@innocarpe/deepseek-build`** (scoped, public) |
| CLI commands | **`deepseek-build`**, **`dsb`** — not the scoped package id (ADR 0006) |
| Version | **Must equal** workspace `Cargo.toml` `[workspace.package].version` full SemVer |
| Bins | `deepseek-build` → `npm/bin/deepseek-build.js`; `dsb` → `npm/bin/dsb.js` |
| `publishConfig.access` | `public` |
| Node engines | `>=18` |
| License | Apache-2.0 |

**Rationale for scoped name:** org ownership (`@innocarpe`), avoid unscoped name squatting, clear brand ownership. Install UX is still short CLIs after install. Do **not** invent alternate package names overnight; change only via ADR amendment.

### Distribution strategy (v1 / `0.7.0`)

**Source-assisted install (chosen for v1):**

1. npm package contains Node **wrappers** + scripts that locate or **build** the native binary.  
2. Preferred resolution order (wrappers):  
   `DEEPSEEK_BUILD_BIN` → `~/.deepseek-build/bin/` → `~/.cargo/bin/` → dev `target/release/`.  
3. `postinstall` may invoke `scripts/install.sh` / cargo build when Rust is available.  
4. `DEEPSEEK_BUILD_SKIP_POSTINSTALL=1` allows skip; user runs `./scripts/install.sh` manually.

**Not in `0.7.0` (later ADR if needed):** prebuilt multi-platform optionalDependencies download matrix, signed artifacts, musl.

### Agent vs human DoD

| Done means | Who |
|------------|-----|
| `package.json` + wrappers + version sync scripts exist | Agent |
| `./scripts/check-semver.sh` + `npm run version-check` pass | Agent |
| `npm pack` succeeds; documented local `npm i -g .` smoke (needs Rust) | Agent |
| Dual bins `--version` match workspace SemVer | Agent |
| **`npm publish` to registry** | **Human only** (OTP/2FA) |

Ultragoal story **npm** may mark **complete** when agent DoD holds.  
If story text required public publish, checkpoint **`blocked-awaiting-human`** with exact publish commands — **do not invent tokens**.

### Publish procedure (human)

```bash
./scripts/check-semver.sh
npm run version-check
npm whoami   # must be member of @innocarpe org (or create org on npmjs.com)
npm pack
# smoke: npm i -g ./innocarpe-deepseek-build-<ver>.tgz  (or npm i -g .)
npm publish --access public
# verify:
npm view @innocarpe/deepseek-build version
npm i -g @innocarpe/deepseek-build
deepseek-build --version && dsb --version
```

### Non-goals

- Putting secrets or API keys in the package  
- Claiming registry install works without Rust until prebuilt strategy ships  

## Consequences

- Wave A G007 complete ≠ package on npmjs.com  
- Wave D `1.0.0` should prefer boring install; may still require Rust until prebuilt ADR  
- SYSTEM_ARCHITECTURE packaging diagram uses this decision  

## References

- [docs/user-guide/05-npm.md](../user-guide/05-npm.md)  
- [WAVE_A_PR_DAG.md](../product/WAVE_A_PR_DAG.md) §0.7.0  
