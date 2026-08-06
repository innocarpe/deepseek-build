# Grok Build vendor layout (ADR-0008)

**Status:** Normative for product 2.0.0 base integration  
**Story:** `grokbase-2x` G004 / unit `2x-W1-1`  
**Spike:** [GROK_BASE_SPIKE.md](./GROK_BASE_SPIKE.md)

---

## Layout

| Path | Role |
|------|------|
| `third_party/grok-build/` | Vendored open-source Grok Build workspace (own `Cargo.toml`) |
| `third_party/grok-build/SOURCE_REV` | Upstream monorepo/sync pin SHA |
| `third_party/grok-build/LICENSE` | Apache-2.0 |
| `third_party/grok-build/THIRD-PARTY-NOTICES` | Upstream third-party notices |
| Root `NOTICE` | Product attribution pointing at this vendor tree |
| `crates/dsb-*` | **Overlay** (provider, config, L1/L2 policy, legacy thin REPL) |
| Product workspace root `Cargo.toml` | 1.x overlay crates only (for now) |

**Two Cargo workspaces on purpose:**

1. **Product workspace** (repo root) — `dsb-*` crates, SemVer product line, npm/install surface.  
2. **Vendor workspace** (`third_party/grok-build`) — Grok pager/agent/tools as upstream ships them.

W1+ product binaries build from the **vendor** workspace (pager composition root) and install as `deepseek-build` / `dsb`. Overlay crates stay in the product workspace for credentials, policy tests, and `repl-legacy`.

---

## Build (local)

Host tools (see spike):

- Rust **1.94.0** (vendor `rust-toolchain.toml`)
- **protoc** on PATH (or `cargo install dotslash` so vendor `bin/protoc` works)
- Recommended: `export PATH="/opt/homebrew/bin:$HOME/.cargo/bin:$PATH"` on macOS Homebrew

```bash
# From product repo root:
./scripts/build-grok-pager.sh check   # cargo check -p xai-grok-pager-bin
./scripts/build-grok-pager.sh release # cargo build -p xai-grok-pager-bin --release
```

Evidence for G004: the script must exit 0 for `check` on a machine with the host tools above.

---

## Refresh procedure

Never silent-copy. Always a dedicated PR:

1. Update sibling or clone upstream Grok Build to the desired rev.  
2. `rsync -a --delete --exclude target --exclude .git <src>/ third_party/grok-build/`  
   (or `git subtree pull` if that workflow is adopted later).  
3. Confirm `SOURCE_REV` matches the intended pin.  
4. Re-run `./scripts/build-grok-pager.sh check`.  
5. PR title: `chore(vendor): refresh grok-build to <short-sha>` with license note unchanged.

---

## CI plan

### Default product-ci (existing)

Path filter for **product** Rust jobs remains root `crates/**`, root `Cargo.toml`, etc.  
**Does not** build the full Grok vendor on every docs or overlay-only PR (clone/build time).

### Vendor check (required when vendor tree changes)

| Item | Plan |
|------|------|
| Trigger | Path filter: `third_party/grok-build/**`, `scripts/build-grok-pager.sh`, this doc |
| Job | Install `protoc` + `dotslash`; run `./scripts/build-grok-pager.sh check` |
| Timeout | Long (30–60+ min cold; cache `third_party/grok-build/target` when practical) |
| Gate | Optional separate check name `grok-vendor-check` — enable as required once stable on ubuntu-latest |

Until the dedicated workflow is green on GitHub-hosted runners, **local** `./scripts/build-grok-pager.sh check` is the merge evidence for vendor PRs, recorded in the PR Testing section.

### Why not single fused workspace yet

- Vendor graph is large (proto, aws-lc, many crates).  
- Overlay SemVer / npm still track product `Cargo.toml` workspace package version.  
- Fusion remains possible later via ADR amendment if dual-root becomes unmaintainable (ADR-0008 notes revisiting strategy A only if B fails operationally).

---

## SemVer

Integration branch / first vendor land may ship product version **`2.0.0-alpha.N`**.  
Tag **`v2.0.0` only** at G012 with REPLAN P0 green.

---

## Related

- ADR-0008, REPLAN_2.0, WAVE_2x W1  
- Install dual bins: G005+ / `scripts/install.sh` evolution  
- npm human publish: ADR 0007
