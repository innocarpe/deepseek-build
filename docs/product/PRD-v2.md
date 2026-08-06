# PRD v2 — DeepSeek Build **2.x** (Grok base + DeepSeek product shell)

| Field | Value |
|-------|--------|
| **SemVer line** | **`2.0.0` – `2.x.y`** (current on disk may be `2.0.3+`) |
| Status | **Shipped product base** (with known residuals) |
| Tags / npm | `v2.0.0` … `v2.0.3` · `@innocarpe/deepseek-build@2.x` |
| Owner | @innocarpe |
| Last updated | 2026-08-07 |
| Index | [versions/README.md](./versions/README.md) |
| Intent replan | [REPLAN_2.0.md](./REPLAN_2.0.md) |
| Board (historical train) | [GROKBASE_2X_GOALS.md](./GROKBASE_2X_GOALS.md) |

---

## 1. Problem

1.x thin REPL is not the product. Owner needs:

1. **`dsb` opens a full-screen coding agent** (not clap-missing-subcommand / not REPL-only).  
2. **Base runtime from open-source Grok Build** (real agent machine, not “vibes”).  
3. **DeepSeek** as default provider + product identity (name, theme, install).  

Full L1 Deep Code + L2 Reasonix fusion **inside** that shell is the long-term identity ([HARNESS_PHILOSOPHY.md](../architecture/HARNESS_PHILOSOPHY.md)) — **targeted for 3.x**, not fully delivered in 2.0.0.

---

## 2. Goals for the 2.x line

### P0 (2.0.0 cut intent)

| # | Goal | Shipped? |
|---|------|----------|
| 1 | No-args TTY `dsb` → full-screen agent | **Yes** (exec `deepseek-build-agent`) |
| 2 | Runtime derived from Grok Build OSS | **Yes** (`third_party/grok-build/`) |
| 3 | DeepSeek default provider + setup/auth | **Yes** (config seed + credentials 0600) |
| 4 | L1 minimum **under that shell** (snippet + perms) | **Partial** — contracts live in 1.x crates/tests; **not fully enforced on Grok tool path** |
| 5 | L2 minimum **under that shell** (stable prefix) | **Partial** — `dsb-context` + thin-path evidence; **not fused into Grok context stack** |
| 6 | Install dogfoodable (`npm i -g` / install.sh) | **Yes** (2.0.3 postinstall builds agent; needs Rust/protoc) |

### P1 (2.x may include as minors)

| # | Goal | Status |
|---|------|--------|
| 7 | DeepSeek product chrome (name, whale, `#4D6BFE`) | **Yes** (2.0.1+) |
| 8 | CLI product language (`Usage: dsb`, hide `repl-legacy`) | **Yes** (2.0.2+) |
| 9 | npm → bare `dsb` opens TUI without manual agent steps | **Yes** (2.0.3 postinstall agent build) |

### Explicit non-goals for **2.0.0**

- Multi-vendor identity  
- Gajae multi-stage harness  
- Unpublishing 1.x  
- Completing full L1/L2 fusion (→ **3.0.0**, [PRD-v3.md](./PRD-v3.md))  
- Perfect pixel clone of every Grok screen  

---

## 3. Architecture (2.x as shipped)

```text
npm / install.sh
  → ~/.deepseek-build/bin/{dsb, deepseek-build, deepseek-build-agent}

dsb (no args, TTY)
  → wrapper dsb-cli (setup, splash, GROK_THEME=deepseeknight)
  → exec deepseek-build-agent  (= branded xai-grok-pager composition root)
       → Grok agent runtime + tools/shell/subagent (L3 machine)
       → DeepSeek models via config (chat_completions, api.deepseek.com)
       → DeepSeekNight UI + whale assets

1.x crates (overlay, still in repo):
  dsb-tools / dsb-context / dsb-provider-deepseek / dsb-agent
  → still power `dsb chat` / `dsb run` (line-mode / one-shot)
  → L1/L2 contracts and tests; fusion into Grok path is 3.x work
```

**Honest identity statement:**

> 2.x = **Grok Build coding agent machine** + **DeepSeek product shell** (entry, provider, brand, install).  
> It is **not yet** “Grok base with Reasonix + Deep Code hearts fully melted in.”

---

## 4. Claimed vs real (2.0.0 cut)

| REPLAN P0 | Reality after 2.0.0–2.0.3 |
|-----------|---------------------------|
| Full-screen agent entry | **Met** |
| Grok-derived base | **Met** (vendor pin) |
| DeepSeek default + auth | **Met** |
| L1 under real shell | **Residual** — default Grok edit/perm path ≠ full Spec 45/90 enforcement |
| L2 under real shell | **Residual** — prefix epoch proven on thin path; Grok stack not fully instrumented |
| Install opens agent | **Met** with Rust toolchain (2.0.3) |

Evidence pointers: `docs/product/evidence/`, CHANGELOG, tags `v2.0.0`–`v2.0.3`.

---

## 5. 2.x freeze / residual policy

| On 2.x | Move to 3.x+ |
|--------|----------------|
| Critical install/auth/UI bugs | L1 snippet+perm **on Grok tools** |
| Docs honesty / messaging | L2 prefix/Flash-Pro/repair **on Grok stack** |
| Optional 2.0.x polish | L3 productization of worktree fleets as identity (4.x) |

Do **not** use 2.x trains to re-implement greenfield agent loops.

---

## 6. Release log (2.x)

| Version | Notes |
|---------|--------|
| **2.0.0** | Product cut: vendor Grok base, entry, DeepSeek defaults |
| **2.0.1** | DeepSeekNight + whale + product strings |
| **2.0.2** | `Usage: dsb`; product language; hide `repl-legacy` |
| **2.0.3** | npm postinstall builds agent so bare `dsb` opens TUI |

---

## 7. Next major

→ [PRD-v3.md](./PRD-v3.md) — **Heart fusion** (Deep Code L1 + Reasonix L2 under Grok shell).
