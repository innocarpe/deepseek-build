# Grok Build base spike (W0)

**Status:** Research evidence for `grokbase-2x` G003 / units `2x-W0-2` + `2x-W0-3`  
**Date:** 2026-08-06  
**ADR:** [0008-grok-build-base.md](../adr/0008-grok-build-base.md) (strategy **B** — subtree under `third_party/grok-build/`)  
**Local spike tree:** sibling `../grok-build` (not a release dependency)

---

## 1. Purpose

Map open-source Grok Build crates and injection points so product 2.0.0 can:

1. Vendor the tree per ADR-0008  
2. Make `deepseek-build` / `dsb` the **Grok pager composition root**  
3. Default model traffic to **DeepSeek** without rewriting the agent loop  
4. Keep 1.x `crates/dsb-*` as **overlay** (credentials, L1/L2 policy, legacy REPL)

---

## 2. Upstream pin (spike machine)

| Field | Value |
|-------|--------|
| Path | `../grok-build` |
| `SOURCE_REV` | `4d6d11372ab8f73026a78c45a7b7e7b1310eb39f` |
| License | Apache-2.0 (`LICENSE`, `THIRD-PARTY-NOTICES`) |
| Upstream | `https://github.com/xai-org/grok-build` |
| Toolchain | `rust-toolchain.toml` channel **1.94.0** (+ rustfmt, clippy) |
| Workspace members | ~70+ crates under `crates/codegen`, `crates/common`, `crates/build`, `prod/mc`, `third_party/*` |

---

## 3. Crate map (product-relevant)

### Entry / TUI

| Crate | Path | Role |
|-------|------|------|
| **`xai-grok-pager-bin`** | `crates/codegen/xai-grok-pager-bin` | **Composition root** (`main`). Product 2.0 bins must be this class of entry. |
| `xai-grok-pager` | `crates/codegen/xai-grok-pager` | Full-screen TUI app, CLI args, event loop |
| `xai-grok-pager-minimal` | `crates/codegen/xai-grok-pager-minimal` | Lighter pager surfaces / auth views |
| `xai-grok-pager-render` | `crates/codegen/xai-grok-pager-render` | Rendering helpers |
| `xai-ratatui-*` | `crates/codegen/xai-ratatui-*` | Terminal widgets |

### Agent runtime

| Crate | Role |
|-------|------|
| `xai-grok-shell` | Agent session, config, auth wiring, tools bridge, headless/stdio/leader modes |
| `xai-grok-agent` | Agent builder, prompts, skills |
| `xai-agent-lifecycle` | Lifecycle control |
| `xai-chat-state` | Conversation state |
| `xai-prompt-queue` | Prompt queue |
| `xai-grok-workspace` (+ client/types) | Workspace session, capability modes, tool config filter |

### Auth / HTTP / models

| Crate | Role |
|-------|------|
| `xai-grok-auth` | `AuthCredentialProvider`, bearer stamp + 401 retry middleware |
| `xai-grok-http` | HTTP client surface |
| `xai-grok-sampler` + `xai-grok-sampling-types` | Inference clients (chat completions **and** Responses backends) |
| `xai-grok-models` | Embedded `default_models.json` (upstream defaults to Grok model ids) |
| `xai-grok-env` | Production endpoint constants (`cli-chat-proxy.grok.com`, etc.) |
| `xai-grok-config` (+ types) | Config load layers; **`~/.grok` / `$GROK_HOME`** |
| `xai-grok-secrets` | Secrets handling |

### Tools / shell / L3

| Crate | Role |
|-------|------|
| `xai-grok-tools` (+ `xai-grok-tools-api`) | Tool registry: read/edit/bash/search, hashline edit, apply_patch, subagents, etc. |
| `xai-grok-shell-base` / `xai-grok-shell-session-support` | Shell plumbing |
| `xai-grok-sandbox` | Sandbox |
| `xai-grok-subagent-resolution` | Subagent resolution |
| `xai-fast-worktree` | Worktree support |
| `xai-grok-mcp` | MCP |
| `xai-workflow` | Workflows |

### Build / proto

| Crate | Role |
|-------|------|
| `xai-proto-build` | Shared protoc configure helper |
| `xai-grok-tools-api` | **build.rs** compiles `proto/grok-tools.proto` → needs **`protoc`** (or repo `bin/protoc` via **dotslash**) |

---

## 4. Injection points for DeepSeek (product overlay)

These are the concrete knobs W1–W2 should use. Prefer config/env/composition-root wiring over forking the entire sampler.

### 4.1 Config home (must not stay `~/.grok` for product)

| Upstream | Product target |
|----------|----------------|
| `xai_grok_config::paths::grok_home()` → `$GROK_HOME` or `~/.grok` | Product path **`~/.deepseek-build/`** (ADR / REPLAN). Set **`GROK_HOME`** to the DeepSeek home **or** patch paths at composition root. |
| `~/.grok/config.toml` layers | Ship / synthesize product `config.toml` under DeepSeek home |
| 1.x `dsb-config` | Keep for credentials.json **0600** + `DEEPSEEK_API_KEY`; bridge into Grok auth on first run |

**Bridge sketch:** on `dsb` start → ensure `BuildHome` exists → if API key present, write Grok-compatible config + export key into the scheme Grok sampler expects (see 4.3).

### 4.2 Inference base URL

| Knob | Location | DeepSeek value |
|------|----------|----------------|
| `endpoints.xai_api_base_url` | `xai-grok-shell` `EndpointsConfig` / config.toml `[endpoints]` | `https://api.deepseek.com` (or `/v1` form matching sampler path join rules) |
| Env `GROK_XAI_API_BASE_URL` | `EndpointsConfig::…` defaults | Same override for CI/headless |
| CLI `--xai-api-base-url` (pager-bin agent args) | `xai-grok-pager-bin` → `apply_agent_endpoint_args` | Optional override |

**Do not** point product inference at `cli-chat-proxy.grok.com` for default chat. Proxy URL remains separate (`cli_chat_proxy_base_url`); for DeepSeek-native BYOK, drive **`xai_api_base_url` + model overrides**, not the Grok cloud proxy.

### 4.3 API key / auth

| Knob | Notes |
|------|--------|
| Model entry `api_key` / `env_key` | Config model overrides; `env_key = "DEEPSEEK_API_KEY"` preferred |
| `AuthManager` + `AuthCredentialProvider` | Bearer stamp middleware (`xai-grok-auth`) |
| First-run UI | Pager auth views (`xai-grok-pager-minimal`); product should route missing key → setup that writes **`~/.deepseek-build/credentials.json` mode 0600** then Grok config |
| Headless | Fail-closed without key (no YOLO default) |

1.x reuse: `dsb_config::Credentials::{load,save}` already implements 0600 save.

### 4.4 Default models

| Upstream | Product |
|----------|---------|
| `xai-grok-models` embeds `default_models.json` with default **`grok-4.5`**, `api_backend: "responses"` | Override via config `[models] default = "…"` **and** per-model tables with **`api_backend = "chat_completions"`** (DeepSeek Chat Completions, ADR 0005) |
| Models | `deepseek-v4-flash` (default), `deepseek-v4-pro` (escalate) — match `dsb-provider-deepseek` |

Sampler supports both Responses and Chat Completions streams (`xai-grok-sampler` attribution). DeepSeek product path must select **chat_completions**, not Responses.

### 4.5 Branding

| Surface | Action |
|---------|--------|
| Pager-bin strings (“Grok Build”, “Couldn't start Grok”) | Replace at product composition root / fork thin bin crate wrapping pager |
| Product name in UI chrome | DeepSeek Build (not Grok as product name) — W1 G006 |
| Binary install names | Always `deepseek-build` + `dsb` (ADR 0006) |

### 4.6 Tools / L1 permissions

| Surface | Notes |
|---------|--------|
| Tool registry | `xai-grok-tools` — real edit path is SearchReplace / hashline / apply_patch, **not** 1.x free-form whole-file primary |
| Capability modes | `xai-grok-workspace::capability::CapabilityMode` (read-only / no-edit / full) |
| Permission prompts | Agent reverse-requests (permission / question / plan-approval) in shell session |
| Headless | Must fail-closed on destructive tools without allowlist (W3) |
| 1.x overlay | Port Spec 20/30 contract tests onto Grok tool kinds; do not reintroduce thin-REPL edit as default |

### 4.7 Prefix / L2

| Surface | Notes |
|---------|--------|
| Compaction / context | `xai-grok-compaction`, chat-state, agent prompts |
| Product L2 | Stable system/tool prefix discipline from Reasonix / `dsb-context` — either enforce via Grok config/prompt layering or document Grok-equivalent with tests (W3 G010) |

---

## 5. Binary production story (from spike → W1)

```text
dsb / deepseek-build
  → product composition root (Grok pager-bin class)
      → xai-grok-pager + xai-grok-shell agent loop
          → sampler (DeepSeek base URL + chat_completions models)
          → tools (Grok registry)
          → config/auth home under ~/.deepseek-build (via GROK_HOME bridge)
```

| Stage | Plan |
|-------|------|
| W1 integrate | Land tree at `third_party/grok-build/` with `SOURCE_REV` + NOTICE |
| W1 entry | Dual bins point at pager composition root; thin 1.x REPL → `repl-legacy` only |
| Build | From product repo: build vendored `xai-grok-pager-bin` (or product rename crate) with documented toolchain |
| npm | ADR 0007: wrappers resolve native agent bin; postinstall/build must produce **agent** binary for 2.x |

---

## 6. Local build spike evidence

### 6.1 First attempt (fail)

```bash
cd ../grok-build
cargo check -p xai-grok-pager-bin
```

| | |
|--|--|
| **Result** | **FAIL** (exit 101) |
| **Cause** | `xai-grok-tools-api` build.rs needs `protoc`. Repo `bin/protoc` is a **dotslash** script; `dotslash` and system `protoc` were missing. |
| **Error excerpt** | `bin/protoc … env: dotslash: No such file or directory` → fallback `protoc not found` |

### 6.2 Toolchain install (this machine)

```bash
brew install protobuf          # protoc libprotoc 35.1
cargo install dotslash --locked  # DotSlash 0.5.7
export PATH="/opt/homebrew/bin:$HOME/.cargo/bin:$PATH"
```

### 6.3 Second attempt (pass)

```bash
cd ../grok-build
cargo check -p xai-grok-pager-bin
```

| | |
|--|--|
| **Result** | **PASS** |
| **Duration** | ~2m 35s (cold-ish after dependency download; second run after protoc fix) |
| **rustc** | 1.94.0 (4a4ef493e 2026-03-02) |
| **cargo** | 1.94.0 (85eff7c80 2026-01-15) |
| **protoc** | libprotoc 35.1 (`/opt/homebrew/bin/protoc`) |
| **dotslash** | 0.5.7 (`~/.cargo/bin/dotslash`) |
| **Host** | macOS arm64 (Apple Silicon) |
| **SOURCE_REV** | `4d6d11372ab8f73026a78c45a7b7e7b1310eb39f` |
| **Tail** | `Finished dev profile [unoptimized + debuginfo] target(s) in 2m 35s` |

### 6.4 CI / product notes for G004+

1. Document **required host tools**: Rust 1.94.0 (via `rust-toolchain.toml`), **protoc**, **dotslash** (or PATH protoc only if build.rs fallback is used).  
2. Full `cargo check -p xai-grok-pager-bin` is heavy (large dependency graph, aws-lc, etc.) — product CI may:
   - run on a path filter when `third_party/grok-build/**` changes, or  
   - use a dedicated workflow with longer timeout / cached `target/`.  
3. Do **not** depend on sibling `../grok-build` for release (ADR-0008 rejects strategy C for product).  
4. Subtree will grow clone size significantly; keep `target/` out of git; consider sparse checkout later if needed (not required for W0).

---

## 7. Risks / open questions for later waves

| Risk | Mitigation |
|------|------------|
| Upstream defaults to Grok Responses API + grok.com proxy | Product default config forces DeepSeek chat_completions + api.deepseek.com |
| Branding strings hard-coded in pager-bin | Thin product bin crate or systematic string layer in W1 |
| `GROK_HOME` vs `~/.deepseek-build` dual semantics | Always set `GROK_HOME` to product home at process start; document env |
| Tree size / CI time | Path-filtered CI; cache; optional “docs-only” jobs skip cargo |
| Proto/dotslash on contributors’ machines | CONTRIBUTING + install.sh install notes |
| Full behavioral parity of DeepSeek thinking/tools with Grok sampler | Dogfood in W2; adapt request shaping if needed |

---

## 8. W0 exit checklist

- [x] ADR-0008 strategy B accepted (G002 / PR #60)  
- [x] Crate map for pager, agent, auth, http, models, tools, shell, subagent, worktree  
- [x] Auth / provider / config injection points listed for DeepSeek  
- [x] `cargo check -p xai-grok-pager-bin` on sibling with **pass** evidence + toolchain notes  
- [ ] Subtree land — **G004**  
- [ ] Dual-bin product entry — **G005**

---

## 9. References

- ADR-0008, REPLAN_2.0 §3–§4, WAVE_2x_PR_DAG W0  
- Sibling `../grok-build/README.md`, `SOURCE_REV`, `LICENSE`  
- Product overlay: `crates/dsb-provider-deepseek`, `crates/dsb-config`, `crates/dsb-tools`, `crates/dsb-context`
