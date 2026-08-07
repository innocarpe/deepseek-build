# Known limitations

**On-disk SemVer:** read root `Cargo.toml` (do not hardcode).  
**Major line PRDs:** [versions/README.md](./versions/README.md)  
**Owner-bar (true complete product):** [OWNER_BAR_ACCEPTANCE.md](./OWNER_BAR_ACCEPTANCE.md) · train **[PRD-v5.md](./PRD-v5.md)** (`5.0.0` / `owner-bar-5x`)  
**Tagged but not owner-bar green:** [PRD-v4.md](./PRD-v4.md) (`4.x`) · [PRD-v3.md](./PRD-v3.md) (`3.x`)  
**Legacy:** `2.x` shell — [PRD-v2.md](./PRD-v2.md) · `1.x` scaffold — [PRD-v1.md](./PRD-v1.md)

## Honesty: majors

| Cut | Meaning |
|-----|---------|
| **2.x** | **Shell cut** — Grok-derived full-screen agent + DeepSeek entry/UI/npm. Hearts residual. |
| **3.0.0 (tagged)** | Heart fusion *attempt* — **owner-bar NOT MET** (library/thin Path A claims; dead snippet wiring). |
| **4.0.0 / 4.0.1 (tagged)** | L3 productization *attempt* — machinery + docs; **owner-bar NOT MET**. |
| **5.0.0** | Owner-bar complete product — [OWNER_BAR_P0_LEDGER.md](./OWNER_BAR_P0_LEDGER.md) all PASS on Path A. |
| **5.0.1 / 5.1.0** | Patches/chrome after owner-bar; **not** vision-complete. Vision residual train: [VISION_COMPLETE_5X_GOALS.md](./VISION_COMPLETE_5X_GOALS.md) (**next feature minor `5.2.0`+**). |

## What 3.0.0 delivers (P0)

| Heart | Shipped | Evidence |
|-------|---------|----------|
| L1 snippet-safe (Spec 45 spirit) | Grok `search_replace` **snippet_safe** + `file_version`; free-form primary fail-closed; product `path_a_edit` | [H45_PATH_A_SNIPPET_2026-08-07.md](./evidence/H45_PATH_A_SNIPPET_2026-08-07.md) |
| L1 permissions (Spec 90 spirit) | Headless Ask→Deny; TTY Ask; product `yolo = false` default; capability×policy matrix | [H90_PATH_A_PERMS_2026-08-07.md](./evidence/H90_PATH_A_PERMS_2026-08-07.md) |
| L2 prefix/epoch (Spec 10 spirit) | `assemble_path_a_context` hash-stable prefix + volatile isolation | [H10_PATH_A_PREFIX_2026-08-07.md](./evidence/H10_PATH_A_PREFIX_2026-08-07.md) |
| L2 repair + Flash/Pro (15/20) | `prepare_path_a_tool_call` + Flash-default router / Pro once | [H15_H20_PATH_A_2026-08-07.md](./evidence/H15_H20_PATH_A_2026-08-07.md) |

Binding map: [HEART_3X_SPEC_BINDING.md](../architecture/HEART_3X_SPEC_BINDING.md).

## Honest residual after 3.0.0

| Topic | Reality | Where next |
|-------|---------|------------|
| Full Spec 45 **snippet_id** mint inside Grok `read_file` | 3.0.0 uses **file_version (sha256) equivalent** + product `SnippetStore` adapter | 3.x minor polish if needed |
| Every Grok compaction path byte-identical to `assemble_path_a_context` | Contract + tests on product assembly API; deep shell prompt paths may still differ | dogfood / minor |
| Live agent dogfood without API key / agent binary | Contract tests green offline; live T4/T5 env-gated | ops |
| L3 worktree forced on bare `dsb` | Worktree remains **opt-in** (`--worktree`); bare `dsb` is single-session TUI | product choice (4.0.0 / G010 honesty) |
| Live L3 extended smoke without API key | Offline CLI green (`test-l3-smoke --offline-only` + heart regression); live env-gated | ops |
| Hollow `~/.deepseek-build/bin/deepseek-build-agent` | Prefer runnable `xai-grok-pager` / rebuilt agent; `find_agent_bin` probes `--help` for worktree | install / G011 |
| TUI update banner showed `v1.0.0` while product is `5.0.0` | **Fixed:** product SemVer via `DEEPSEEK_BUILD_VERSION` + update checks against `@innocarpe/deepseek-build` / `innocarpe/deepseek-build` (not Grok/`0.2.x` vs stale `1.0.0` cache). Rebuild agent + clear `~/.deepseek-build/version.json` after upgrade | product version/update |
| Skills thrash-free full (Spec 70) | Index in stable prefix; thrash-free body load polish | 3.x minor if non-breaking |

## What 2.x still is (shell)

- No-args TTY `dsb` → `deepseek-build-agent`
- Vendor tree `third_party/grok-build/` (ADR-0008)
- DeepSeek models + `base_url = https://api.deepseek.com` (load-bearing)
- Product chrome DeepSeekNight / dual CLI names

## Ops limits (carry forward)

### Install / packaging

- **`4.0.1`+:** `npm i -g` downloads **prebuilt** natives from GitHub Releases (ADR 0009) — seconds, no Rust on default path.
- Source compile only with `DEEPSEEK_BUILD_ALLOW_SOURCE_BUILD=1` or `./scripts/install.sh` (dev).
- Prebuilt platform: `darwin-arm64` (Apple Silicon macOS only); other targets are deferred.
- **npm registry publish** remains **human-gated** (ADR 0007).

### Auth / network

- Requires DeepSeek API key for live turns.
- Each `[model.deepseek-*]` must set `base_url = "https://api.deepseek.com"`.

### Everyday tests

```bash
./scripts/test-pre3x-baseline.sh --live   # when key present
cargo test -p dsb-tools path_a
cargo test -p dsb-context path_a
cargo test -p dsb-agent path_a
```

Do **not** run vendor-full cargo as everyday gate (disk bomb).
