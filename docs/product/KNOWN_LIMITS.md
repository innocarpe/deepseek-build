# Known limitations

**On-disk SemVer:** read root `Cargo.toml` (do not hardcode).  
**Major line PRDs:** [versions/README.md](./versions/README.md)  
**Owner-bar (true complete product):** [OWNER_BAR_ACCEPTANCE.md](./OWNER_BAR_ACCEPTANCE.md) · train **[PRD-v5.md](./PRD-v5.md)** (`5.0.0` / `owner-bar-5x`)  
**Tagged but not owner-bar green:** [PRD-v4.md](./PRD-v4.md) (`4.x`) · [PRD-v3.md](./PRD-v3.md) (`3.x`)  
**Legacy:** `2.x` shell — [PRD-v2.md](./PRD-v2.md) · `1.x` scaffold — [PRD-v1.md](./PRD-v1.md)  
**Active vision train:** [VISION_COMPLETE_5X_GOALS.md](./VISION_COMPLETE_5X_GOALS.md) · DAG [WAVE_5x_VISION_PR_DAG.md](./WAVE_5x_VISION_PR_DAG.md)

## Honesty: majors and live floor

| Cut | Meaning |
|-----|---------|
| **2.x** | **Shell cut** — Grok-derived full-screen agent + DeepSeek entry/UI/npm. Hearts residual at the time. |
| **3.0.0 (tagged)** | Heart fusion *attempt* — **owner-bar NOT MET** (library/thin Path A claims; dead snippet wiring at the time). |
| **4.0.0 / 4.0.1 (tagged)** | L3 productization *attempt* — machinery + docs; **owner-bar NOT MET**. Historical only — **not** the current residual story. |
| **5.0.0** | Owner-bar complete product — [OWNER_BAR_P0_LEDGER.md](./OWNER_BAR_P0_LEDGER.md) all PASS on Path A. |
| **5.0.1 / 5.1.0** | Patches/chrome after owner-bar; **not** vision-complete. |
| **5.2.0 – 5.2.2** | Live packaging line on **`main` / npm / GitHub Latest`** as of the VC014 docs pass (**`5.2.2`**). Includes early vision floor work; **not** the full unmerged vision stack. |
| **5.3.0 (on-branch stack)** | Spec 45 Path A `snippet_id` Deep Code cut (VC006) — **may be unmerged** relative to live main. |
| **5.4.0 (on-branch stack)** | L3 Path A R0A train cut (VC010–VC013) — **may be unmerged** relative to live main. |
| **5.5.0** | Vision freeze — **not claimed** until VC015 dual review + CUT. |

**Floor rule:** live install from npm/GitHub Latest can still report **`5.2.2`** while this monorepo stack tip already carries **`5.4.0`**. Treat stack SemVer and registry SemVer as different until merge + human-gated publish.

User-facing behavior for the vision stack is documented under [user-guide](../user-guide/README.md). Docs pass evidence: [VC014_USER_GUIDE_KNOWN_LIMITS_2026-08-08.md](./evidence/VC014_USER_GUIDE_KNOWN_LIMITS_2026-08-08.md).

## Path A vision stack — what is evidenced (not residual)

These are **on Path A** (public `deepseek-build` / `dsb` → product agent) under the unmerged vision train. They are **not** “wait for 4.0.0” items.

| Area | Evidenced | Pointer |
|------|-----------|---------|
| Dual CLI names | `deepseek-build` primary + `dsb` alias | ADR 0006 · user-guide |
| Spec 45 `snippet_id` mint/require/invalidation + multi-edit R0A | VC003–VC006 (stack cut **5.3.0**) | [VC006 evidence](./evidence/VC006_PATH_A_HEART_R0A_2026-08-08.md) |
| Spec 10 assembly on Grok Path A turns | VC007 | [VC007 evidence](./evidence/VC007_SPEC10_ASSEMBLY_PATH_A_2026-08-08.md) |
| `reasoning_effort` on DeepSeek wire (product seed/repair) | VC008 | [VC008 evidence](./evidence/VC008_REASONING_EFFORT_WIRE_2026-08-08.md) |
| Cache-hit visibility (chip path + Path A stamp) | VC009 | [VC009 evidence](./evidence/VC009_CACHE_VISIBILITY_2026-08-08.md) |
| Multi-tool parallel + bg collect-by-id | VC010 · re-prove VC013 | [VC010](./evidence/VC010_L3_MULTI_TOOL_BG_PATH_A_2026-08-08.md) · [VC013](./evidence/VC013_L3_5_4_0_CUT_2026-08-08.md) |
| Subagent explore/implement + worker epoch stamp | VC011 · re-prove VC013 | [VC011](./evidence/VC011_SUBAGENT_WORKER_CACHE_PATH_A_2026-08-08.md) |
| Worktree opt-in dogfood + headless no-create | VC012 · re-prove VC013 | [VC012](./evidence/VC012_WORKTREE_DOGFOOD_PATH_A_2026-08-08.md) |

Owner-bar and heart regression gates must stay green on the stack:

```bash
./scripts/test-owner-bar.sh
./scripts/test-heart-regression.sh
./scripts/check-path-a-linkage.sh
./scripts/check-semver.sh
```

## Evidenced residuals (current)

Only items below are treated as **open residual** for vision honesty. Do **not** close them in docs without **fresh Path A proof**.

| Topic | Reality | Where |
|-------|---------|-------|
| **V3-60-3** parent snippet expiry after worker mutation | Disk mutation by implement-class child is proven; **Path A parent snippet-table expire** is **not** sole-green (thin unit support only) | VC011 residual · carried by VC013 |
| **Interactive TTY worktree create** | Product flag forward + headless no-create + opt-in stamp proven; **interactive create after process `exec`** not asserted as sole green | VC012 residual · carried by VC013 |
| **Non-darwin packaging / assets** | Prebuilt platform is **`darwin-arm64`** (Apple Silicon macOS) only; other targets deferred | Install / ADR 0009 · [05-npm.md](../user-guide/05-npm.md) |
| **Human-gated npm / GitHub publish** | Registry publish and Release attach remain **human-gated** (ADR 0007). Docs and on-branch cuts do **not** auto-publish | Release lane |
| **Stack vs live SemVer lag** | On-branch stack may be **`5.4.0`** while live `main` / npm / GitHub Latest remain **`5.2.2`** until stack merges and publish runs | Floor re-check every session |

### L2 scope notes (not blockers if over-claimed elsewhere)

| Topic | Honest scope |
|-------|----------------|
| Spec 30 full thinking body field on every Grok wire shape | **Not claimed** as Path A complete — effort string ≠ full Spec 30 object |
| CLI `--effort` override on hermetic wire | Default seed/repair path is wire-proven; override is primarily argv/unit |
| Live DeepSeek cache hit rates | Server policy; hermetic fixture ≠ live hit rate |
| Agent binary without product CLI repair | Direct agent may skip product config repair path |

## What older tagged attempts still meant (history)

### 3.0.0 hearts (historical P0 — superseded by owner-bar + vision stack)

| Heart | At 3.0.0 | Superseded by |
|-------|----------|---------------|
| L1 snippet-safe | `file_version` + product adapter | Vision Spec 45 **`snippet_id`** Path A (VC003–VC006) |
| L1 permissions | Headless Ask→Deny; yolo default false | Still load-bearing; heart regression |
| L2 prefix/epoch | Product assembly API | Vision Spec 10 on Grok Path A turns (VC007) |
| L2 repair + Flash/Pro | Call sites + router | Still load-bearing; effort seed VC008 |

Binding map: [HEART_3X_SPEC_BINDING.md](../architecture/HEART_3X_SPEC_BINDING.md).

### 2.x shell (still true)

- No-args TTY `dsb` → `deepseek-build-agent`
- Vendor tree `third_party/grok-build/` (ADR-0008)
- DeepSeek models + `base_url = https://api.deepseek.com` (load-bearing)
- Product chrome + dual CLI names

## Ops limits (carry forward)

### Install / packaging

- **`4.0.1`+ packaging design:** `npm i -g` downloads **prebuilt** natives from GitHub Releases (ADR 0009) — seconds, no Rust on default path.
- Source compile only with `DEEPSEEK_BUILD_ALLOW_SOURCE_BUILD=1` or `./scripts/install.sh` (dev).
- Prebuilt platform: **`darwin-arm64` only** (see residual table).
- **npm registry publish** remains **human-gated** (ADR 0007).
- **GitHub Release assets** for a new SemVer are a separate release lane after merge — an on-branch cut is not a published Release.

### Auth / network

- Requires DeepSeek API key for live turns.
- Each `[model.deepseek-*]` must set `base_url = "https://api.deepseek.com"`.

### Everyday tests

```bash
./scripts/test-pre3x-baseline.sh --live   # when key present
cargo test -p dsb-tools path_a
cargo test -p dsb-context path_a
cargo test -p dsb-agent path_a
./scripts/test-l3-smoke.sh --offline-only
# Path A hermetic (dev):
# ./scripts/test-path-a-vc010-r0a.sh
# ./scripts/test-path-a-vc011-r0a.sh
# ./scripts/test-path-a-vc012-r0a.sh
```

Do **not** run vendor-full cargo as everyday gate (disk bomb).

## Explicit non-claims

- **Not** vision-complete freeze (**5.5.0** / VC015) until dual adversarial review + CUT.
- **Not** “residuals closed” for V3-60-3 or interactive worktree create without fresh Path A R0A.
- **Not** claiming live npm/GitHub Latest already ships unmerged stack SemVers.
- **Not** conflating thin Path B unit greens with Path A sole proof.
