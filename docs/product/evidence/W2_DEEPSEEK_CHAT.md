# W2 DeepSeek default + chat evidence (G007)

**Date:** 2026-08-06  
**Band:** 2.0.0  

## Default product wiring

| Knob | Value |
|------|-------|
| Models | `deepseek-v4-flash` (default), `deepseek-v4-pro` |
| Base URL | `https://api.deepseek.com` |
| API backend | `chat_completions` (Grok agent config seed) |
| Env key | `DEEPSEEK_API_KEY` |
| Credentials | `~/.deepseek-build/credentials.json` (0600) |
| Overlay | `dsb-provider-deepseek` for thin `run` / `repl-legacy` |
| Agent | Grok pager with product `config.toml` seed (same defaults) |

## Live chat turn

Command: `dsb run "Reply with exactly one word: pong"`

```text
[model=deepseek-v4-flash thinking=on effort=high]
[prefix_epoch=0048c45a4b840731]
pong[cache_evidence=usage_fields]

[model_used=deepseek-v4-flash model=deepseek-v4-flash thinking=on effort=high]

```

## Offline/smoke excerpt (redacted)

```text
ersion 2.0.0-alpha.2
== Build dual bins ==
== Dual --version ==
  deepseek-build 2.0.0-alpha.2
  dsb 2.0.0-alpha.2
smoke-dogfood OK: dual version contains 2.0.0-alpha.2
== Help ==
smoke-dogfood OK: help
== Offline tests (workspace) ==

running 30 tests
..............................
test result: ok. 30 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s


running 18 tests
..................
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 18 tests
..................
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 9 tests
.........
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 15 tests
...............
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 17 tests
.................
test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s


running 43 tests
...........................................
test result: ok. 43 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

smoke-dogfood OK: cargo test
== npm version match (if package.json present) ==
smoke-dogfood OK: npm version-check
== Live API (optional) ==
[model=deepseek-v4-flash thinking=on effort=high]
[prefix_epoch=0048c45a4b840731]
pong[cache_evidence=usage_fields]

[model_used=deepseek-v4-flash model=deepseek-v4-flash thinking=on effort=high]
smoke-dogfood OK: live run attempted
== dogfood-usable §3 mapping ==
1 install/build bins: OK (this script builds or uses target)
2 auth: env key optional above
3 chat/run: help OK; live optional
4 tools: covered by cargo test (snippet/grep/bash units)
5 write profile: --dogfood flag exists (cli --help)
6 docs: see README + docs/user-guide
7 SemVer: 2.0.0-alpha.2
smoke-dogfood OK: ALL PASSED (offline core)

```
