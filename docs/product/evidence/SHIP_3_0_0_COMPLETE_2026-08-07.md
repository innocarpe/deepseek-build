# Ship complete — DeepSeek Build **3.0.0**

**Date:** 2026-08-07  
**Plan:** `ship-3.0.0`  
**Tag:** `v3.0.0` → `bcb92d2969cf9ef1c7a071cde8b4ff5f3023a661` (`origin/main`)

## Checklist (user desired end state)

| Item | Status | Proof |
|------|--------|--------|
| Git tag peels to cut commit | **PASS** | `v3.0.0^{}` == `origin/main` == PR #83 merge |
| On-disk SemVer 3.0.0 | **PASS** | Cargo.toml + package.json |
| GitHub Release | **PASS** | https://github.com/innocarpe/deepseek-build/releases/tag/v3.0.0 |
| npm `@innocarpe/deepseek-build@3.0.0` | **PASS** | `npm view` → version 3.0.0, `dist-tags.latest=3.0.0` (already published; re-publish correctly E403) |
| Local install wrappers 3.0.0 | **PASS** | `~/.deepseek-build/bin/{deepseek-build,dsb} --version` → 3.0.0 |
| Agent binary present | **PASS** | `~/.deepseek-build/bin/deepseek-build-agent` (T2.1) |
| Product offline baseline | **PASS** | `./scripts/test-product-offline.sh` / `test-pre3x-baseline.sh` → PASS=9 FAIL=0 |
| Path A heart contracts | **PASS** | dsb-tools path_a 15, dsb-context path_a 5, dsb-agent path_a 8 |
| Live API smoke (T3/T4) | **SKIP** | No `DEEPSEEK_API_KEY` / credentials.json in this environment |
| heart-3x ultragoal | **PASS** | 8/8 complete (prior plan) |

## Commands re-run at ship close

```bash
git rev-parse origin/main v3.0.0^{}
export PATH="$HOME/.deepseek-build/bin:$PATH"
deepseek-build --version   # 3.0.0
dsb --version              # 3.0.0
./scripts/test-pre3x-baseline.sh
cargo test -p dsb-tools path_a
cargo test -p dsb-context path_a
cargo test -p dsb-agent path_a
npm view @innocarpe/deepseek-build@3.0.0 version
gh release view v3.0.0
```

## npm

- Dry-run: ok (`+ @innocarpe/deepseek-build@3.0.0`)
- Live publish attempt: **E403 already published** → registry already has **3.0.0** as `latest` (desired end state)

## Live residual

When API key is available on the machine:

```bash
export PATH="$HOME/.deepseek-build/bin:$PATH"
./scripts/test-pre3x-baseline.sh --live
```

Not required for ship-close offline gate; recorded as intentional SKIP.

## Related

- Cut evidence: [CUT_3_0_0_2026-08-07.md](./CUT_3_0_0_2026-08-07.md)
- Tag verify: [SHIP_3_0_0_TAG_VERIFY_2026-08-07.md](./SHIP_3_0_0_TAG_VERIFY_2026-08-07.md)
